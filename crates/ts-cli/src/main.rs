use anyhow::{Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAX_RECEIVED_MANIFEST_BYTES: usize = 64 * 1024 * 1024;

fn usage() {
    eprintln!("usage:\n  ts-cli node [manifest.tswarm] [store-root]\n  ts-cli fetch-chunk <peer-id> <peer-address> <tensor-hash-hex> <chunk-index> <chunk-hash-hex> <output>\n  ts-cli prepare <manifest.tswarm> <store-root> <output>\n  ts-cli verify-release <bundle.tsrelease> <manifest.tswarm> [tswarm://v1/<digest>]\n  ts-cli receive-release <bundle.tsrelease> <manifest.tswarm> <inbox-dir> [tswarm://v1/<digest>]\n  ts-cli fetch-received <record-dir> <peer-id> <peer-address> <store-root> <output>\n  ts-cli proxy [bind] <root>\n  ts-cli proxy-manifest [bind] <manifest.tswarm> <store-root> <origin-url> [peer-id] [peer-address]\n  ts-cli inspect <model>\n  ts-cli manifest <model> <output.tswarm>\n  ts-cli verify <model> <manifest.tswarm>\n  ts-cli diff <old.tswarm> <new.tswarm>");
}

fn read_bounded(path: &std::path::Path, max_bytes: usize) -> Result<Vec<u8>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take((max_bytes + 1) as u64).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= max_bytes, "input exceeds the size limit");
    Ok(bytes)
}

fn checked_release(
    bundle_path: &Path,
    manifest_path: &Path,
    uri: Option<&std::ffi::OsStr>,
) -> Result<(ts_core::SignedModelRelease, ts_core::Manifest)> {
    let release = ts_core::SignedModelRelease::from_bytes(&read_bounded(
        bundle_path,
        ts_core::MAX_SIGNED_MODEL_RELEASE_BYTES,
    )?)?;
    if let Some(uri) = uri {
        let link = ts_core::ModelShareLink::parse(
            uri.to_str()
                .context("share identifier must be valid UTF-8")?,
        )?;
        link.verify_release(&release)?;
    }
    let manifest: ts_core::Manifest =
        serde_cbor::from_slice(&read_bounded(manifest_path, MAX_RECEIVED_MANIFEST_BYTES)?)?;
    anyhow::ensure!(manifest.verify_root(), "manifest self-check failed");
    anyhow::ensure!(
        release.descriptor.variants.iter().any(|variant| variant
            .files
            .iter()
            .any(|file| file.matches_manifest(&manifest, variant.format))),
        "signed descriptor does not reference this manifest and file",
    );
    Ok((release, manifest))
}

fn receive_release(
    inbox: &Path,
    release: &ts_core::SignedModelRelease,
    manifest: &ts_core::Manifest,
) -> Result<PathBuf> {
    let id = ts_core::ModelShareLink::for_release(release)?.descriptor_id();
    let name =
        id.0.iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
    let record = inbox.join(name);
    let bundle_bytes = release.to_bytes()?;
    let manifest_bytes = manifest.to_bytes();
    if record.exists() {
        anyhow::ensure!(
            fs::read(record.join("release.tsrelease"))? == bundle_bytes
                && fs::read(record.join("manifest.tswarm"))? == manifest_bytes,
            "existing recipient record differs; refusing to replace it"
        );
        return Ok(record);
    }
    fs::create_dir_all(inbox).with_context(|| format!("create inbox {}", inbox.display()))?;
    let staged = tempfile::Builder::new()
        .prefix(".incoming-")
        .tempdir_in(inbox)?;
    for (name, bytes) in [
        ("release.tsrelease", bundle_bytes.as_slice()),
        ("manifest.tswarm", manifest_bytes.as_slice()),
    ] {
        let mut file = fs::File::create(staged.path().join(name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(staged.path(), &record)
        .with_context(|| format!("commit recipient record {}", record.display()))?;
    anyhow::ensure!(
        fs::read(record.join("release.tsrelease"))? == bundle_bytes
            && fs::read(record.join("manifest.tswarm"))? == manifest_bytes,
        "recipient record readback differs"
    );
    Ok(record)
}

async fn fetch_received(
    record: &Path,
    peer: libp2p::PeerId,
    address: libp2p::Multiaddr,
    store_root: &Path,
    output: &Path,
) -> Result<()> {
    let (release, manifest) = checked_release(
        &record.join("release.tsrelease"),
        &record.join("manifest.tswarm"),
        None,
    )?;
    let expected_id = ts_core::ModelShareLink::for_release(&release)?.descriptor_id();
    let expected_name = expected_id
        .0
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    anyhow::ensure!(
        record
            .file_name()
            .is_some_and(|name| name == std::ffi::OsStr::new(&expected_name)),
        "recipient record directory does not match the signed release ID"
    );
    anyhow::ensure!(
        manifest.files.len() == 1,
        "recipient manifest must describe one file"
    );
    anyhow::ensure!(
        !output.exists() && !output.with_extension("partial").exists(),
        "output or partial path exists; choose a new output path"
    );

    let store = ts_store::ObjectStore::open(store_root)?;
    let mut client = None;
    for tensor in &manifest.tensors {
        for chunk in &tensor.chunks {
            match store.get(chunk.hash) {
                Ok(bytes) => {
                    anyhow::ensure!(
                        bytes.len() as u64 == chunk.length,
                        "cached chunk length mismatch"
                    );
                    continue;
                }
                Err(ts_store::StoreError::Missing(_)) => {}
                Err(error) => return Err(error.into()),
            }
            if client.is_none() {
                let mut connected =
                    ts_p2p::LanClient::new(ts_p2p::build_lan_swarm_with_listeners(&[])?);
                connected.swarm_mut().dial(address.clone())?;
                client = Some(connected);
            }
            let bytes = client
                .as_mut()
                .unwrap()
                .fetch_chunk(
                    peer,
                    ts_p2p::ChunkRequest {
                        request_id: u64::from(chunk.index),
                        tensor_hash: tensor.tensor_hash,
                        chunk_index: chunk.index,
                        expected_hash: chunk.hash,
                    },
                    3,
                )
                .await?;
            anyhow::ensure!(
                bytes.len() as u64 == chunk.length,
                "fetched chunk length mismatch"
            );
            store.put_verified(chunk.hash, &bytes)?;
            anyhow::ensure!(
                store.get(chunk.hash)? == bytes,
                "fetched chunk store readback differs"
            );
        }
    }
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let staged = tempfile::Builder::new()
        .prefix(".tswarm-received-")
        .tempdir_in(parent)?;
    let prepared = staged.path().join("model");
    store.materialize(&manifest, &prepared)?;
    commit_received_output(&prepared, output)?;
    Ok(())
}

fn commit_received_output(prepared: &Path, output: &Path) -> Result<()> {
    fs::hard_link(prepared, output).with_context(|| {
        format!(
            "commit recipient output {} without replacing an existing file",
            output.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod recipient_output_tests {
    use super::*;

    #[test]
    fn recipient_commit_refuses_a_destination_created_after_preparation() {
        let directory = tempfile::tempdir().unwrap();
        let prepared = directory.path().join("prepared");
        let output = directory.path().join("output");
        fs::write(&prepared, b"verified model").unwrap();
        fs::write(&output, b"other user's file").unwrap();
        assert!(commit_received_output(&prepared, &output).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"other user's file");
        assert_eq!(fs::read(&prepared).unwrap(), b"verified model");
        fs::remove_file(&output).unwrap();
        commit_received_output(&prepared, &output).unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"verified model");
    }
}

fn parse_hash(value: &std::ffi::OsStr) -> Result<ts_core::Hash32> {
    let value = value.to_str().context("hash must be valid UTF-8")?;
    anyhow::ensure!(
        value.len() == 64,
        "hash must contain 64 hexadecimal characters"
    );
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .context("hash must be hexadecimal")?;
    }
    Ok(ts_core::Hash32(bytes))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args_os();
    let _program = args.next();
    let command = args.next().and_then(|value| value.into_string().ok());
    match command.as_deref() {
        Some("node") => {
            let mut swarm = ts_p2p::build_lan_swarm()?;
            let provider = match args.next() {
                Some(manifest_path) => {
                    let manifest: ts_core::Manifest =
                        serde_cbor::from_slice(&fs::read(manifest_path)?)?;
                    let store_root = PathBuf::from(args.next().context("missing store root")?);
                    let store = ts_store::ObjectStore::open(store_root)?;
                    let provider = ts_p2p::ChunkProvider::from_manifest(&store, &manifest)?;
                    ts_p2p::publish_manifest(&mut swarm, &manifest.root)?;
                    for tensor in &manifest.tensors {
                        if provider.has_verified_chunks_for_tensor(&tensor.tensor_hash) {
                            ts_p2p::publish_tensor(&mut swarm, &tensor.tensor_hash)?;
                        }
                    }
                    provider
                }
                None => ts_p2p::ChunkProvider::default(),
            };
            let peer_id = *swarm.local_peer_id();
            let (listen_tx, listen_rx) = tokio::sync::oneshot::channel();
            let node = tokio::spawn(ts_p2p::run_lan_node_with_provider_and_notify(
                swarm,
                provider,
                Some(listen_tx),
            ));
            println!("peer: {peer_id}");
            if let Ok(address) = listen_rx.await {
                println!("listen: {address}");
            }
            node.await??;
        }
        Some("fetch-chunk") => {
            let peer_id = args
                .next()
                .context("missing peer ID")?
                .to_str()
                .context("peer ID must be valid UTF-8")?
                .parse::<libp2p::PeerId>()
                .context("invalid peer ID")?;
            let peer_address = args
                .next()
                .context("missing peer address")?
                .to_str()
                .context("peer address must be valid UTF-8")?
                .parse::<libp2p::Multiaddr>()
                .context("invalid peer address")?;
            let tensor_hash = parse_hash(&args.next().context("missing tensor hash")?)?;
            let chunk_index = args
                .next()
                .context("missing chunk index")?
                .to_str()
                .context("chunk index must be valid UTF-8")?
                .parse()?;
            let expected_hash = parse_hash(&args.next().context("missing chunk hash")?)?;
            let output = PathBuf::from(args.next().context("missing output path")?);
            anyhow::ensure!(args.next().is_none(), "too many fetch-chunk arguments");
            let mut client = ts_p2p::LanClient::new(ts_p2p::build_lan_swarm_with_listeners(&[])?);
            client.swarm_mut().dial(peer_address)?;
            let bytes = client
                .fetch_chunk(
                    peer_id,
                    ts_p2p::ChunkRequest {
                        request_id: u64::from(chunk_index),
                        tensor_hash,
                        chunk_index,
                        expected_hash,
                    },
                    3,
                )
                .await?;
            fs::write(output, bytes)?;
        }
        Some("fetch-received") => {
            let values = args.collect::<Vec<_>>();
            let [record, peer, address, store_root, output] = values.as_slice() else {
                anyhow::bail!("usage: ts-cli fetch-received <record-dir> <peer-id> <peer-address> <store-root> <output>");
            };
            let peer: libp2p::PeerId = peer.to_str().context("peer ID must be UTF-8")?.parse()?;
            let address: libp2p::Multiaddr = address
                .to_str()
                .context("peer address must be UTF-8")?
                .parse()?;
            let output = PathBuf::from(output);
            fetch_received(
                &PathBuf::from(record),
                peer,
                address,
                &PathBuf::from(store_root),
                &output,
            )
            .await?;
            println!("prepared verified recipient file: {}", output.display());
        }
        Some("prepare") => {
            let manifest_path = PathBuf::from(args.next().context("missing manifest path")?);
            let store_root = PathBuf::from(args.next().context("missing store root")?);
            let output = PathBuf::from(args.next().context("missing output path")?);
            anyhow::ensure!(args.next().is_none(), "too many prepare arguments");
            let manifest: ts_core::Manifest = serde_cbor::from_slice(&fs::read(&manifest_path)?)?;
            anyhow::ensure!(manifest.verify_root(), "manifest self-check failed");
            let store = ts_store::ObjectStore::open(store_root)?;
            store.materialize(&manifest, &output)?;
            println!("prepared: {}", output.display());
        }
        Some("verify-release" | "receive-release") => {
            let values = args.collect::<Vec<_>>();
            let receive = command.as_deref() == Some("receive-release");
            let (bundle_path, manifest_path, inbox, uri) = match (receive, values.as_slice()) {
                (false, [bundle, manifest]) => (bundle, manifest, None, None),
                (false, [bundle, manifest, uri]) => (bundle, manifest, None, Some(uri.as_os_str())),
                (true, [bundle, manifest, inbox]) => (bundle, manifest, Some(inbox), None),
                (true, [bundle, manifest, inbox, uri]) => (bundle, manifest, Some(inbox), Some(uri.as_os_str())),
                (false, _) => anyhow::bail!("usage: ts-cli verify-release <bundle.tsrelease> <manifest.tswarm> [tswarm://v1/<digest>]"),
                (true, _) => anyhow::bail!("usage: ts-cli receive-release <bundle.tsrelease> <manifest.tswarm> <inbox-dir> [tswarm://v1/<digest>]"),
            };
            let (release, manifest) = checked_release(
                &PathBuf::from(bundle_path),
                &PathBuf::from(manifest_path),
                uri,
            )?;
            if let Some(inbox) = inbox {
                let record = receive_release(&PathBuf::from(inbox), &release, &manifest)?;
                println!(
                    "recipient metadata saved: {}; not in active library; model bytes not checked",
                    record.display()
                );
            } else {
                println!("descriptor and manifest match; self-asserted signature only; artifact bytes not checked");
            }
        }
        Some("proxy") => {
            let bind = args
                .next()
                .and_then(|value| value.into_string().ok())
                .unwrap_or_else(|| ts_proxy::DEFAULT_BIND.to_string());
            let root = PathBuf::from(args.next().context("missing proxy root directory")?);
            println!("proxy: http://{bind}/file/<path>");
            ts_proxy::serve(&bind, root).await?;
        }
        Some("proxy-manifest") => {
            let values = args.collect::<Vec<_>>();
            let (bind, manifest_path, store_root, origin_url, peer_config) = match values.as_slice() {
                [manifest_path, store_root, origin_url] => (
                    ts_proxy::DEFAULT_BIND.to_string(),
                    manifest_path,
                    store_root,
                    origin_url,
                    None,
                ),
                [bind, manifest_path, store_root, origin_url] => (
                    bind.to_str()
                        .context("proxy bind must be valid UTF-8")?
                        .to_string(),
                    manifest_path,
                    store_root,
                    origin_url,
                    None,
                ),
                [manifest_path, store_root, origin_url, peer_id, peer_address] => (
                    ts_proxy::DEFAULT_BIND.to_string(),
                    manifest_path,
                    store_root,
                    origin_url,
                    Some((peer_id, peer_address)),
                ),
                [bind, manifest_path, store_root, origin_url, peer_id, peer_address] => (
                    bind.to_str()
                        .context("proxy bind must be valid UTF-8")?
                        .to_string(),
                    manifest_path,
                    store_root,
                    origin_url,
                    Some((peer_id, peer_address)),
                ),
                _ => anyhow::bail!("usage: ts-cli proxy-manifest [bind] <manifest.tswarm> <store-root> <origin-url> [peer-id] [peer-address]"),
            };
            let manifest: ts_core::Manifest = serde_cbor::from_slice(&fs::read(manifest_path)?)?;
            anyhow::ensure!(manifest.verify_root(), "manifest self-check failed");
            let model_path = manifest
                .files
                .first()
                .context("manifest has no file recipe")?
                .path
                .clone();
            let origin_url = ts_proxy::resolve_origin_url(
                origin_url
                    .to_str()
                    .context("origin URL must be valid UTF-8")?,
                &model_path,
            )?;
            let store = ts_store::ObjectStore::open(PathBuf::from(store_root))?;
            let peer = if let Some((peer_id, peer_address)) = peer_config {
                let peer_id = peer_id
                    .to_str()
                    .context("peer ID must be valid UTF-8")?
                    .parse::<libp2p::PeerId>()
                    .context("invalid peer ID")?;
                let peer_address = peer_address
                    .to_str()
                    .context("peer address must be valid UTF-8")?
                    .parse::<libp2p::Multiaddr>()
                    .context("invalid peer multiaddress")?;
                let client = std::sync::Arc::new(tokio::sync::Mutex::new(ts_p2p::LanClient::new(
                    ts_p2p::build_lan_swarm_with_listeners(&[])?,
                )));
                client.lock().await.swarm_mut().dial(peer_address)?;
                Some(ts_proxy::lan_peer_fetch(client, peer_id, 3))
            } else {
                None
            };
            // This CLI proxy fetches bytes but does not host P2P chunk requests,
            // so it must not announce itself as a DHT provider.
            let engine = ts_proxy::FetchEngine::new(store);
            let config = ts_proxy::ManifestProxyState {
                manifest,
                origin_url,
                engine,
                peer,
            };
            println!("manifest proxy: http://{bind}/file/<manifest-path>");
            ts_proxy::serve_manifest(&bind, config).await?;
        }
        Some("inspect") => {
            let path = PathBuf::from(args.next().context("missing model path")?);
            let index = ts_format::inspect(&path)?;
            println!(
                "format: {:?}\nfile_bytes: {}\ntensors: {}",
                index.format,
                index.file_len,
                index.tensors.len()
            );
            for tensor in index.tensors {
                println!(
                    "{}\t{} bytes\t@{}\t{:?}",
                    tensor.descriptor.name, tensor.length, tensor.offset, tensor.descriptor.shape
                );
            }
        }
        Some("manifest") => {
            let path = PathBuf::from(args.next().context("missing model path")?);
            let output = PathBuf::from(args.next().context("missing output path")?);
            let manifest = ts_format::build_manifest(&path, 16 * 1024 * 1024)?;
            fs::write(&output, manifest.to_bytes())
                .with_context(|| format!("write {}", output.display()))?;
            println!(
                "manifest: {}\nroot: {:?}\ntensors: {}",
                output.display(),
                manifest.root,
                manifest.tensors.len()
            );
        }
        Some("verify") => {
            let path = PathBuf::from(args.next().context("missing model path")?);
            let manifest_path = PathBuf::from(args.next().context("missing manifest path")?);
            let expected: ts_core::Manifest = serde_cbor::from_slice(&read_bounded(
                &manifest_path,
                MAX_RECEIVED_MANIFEST_BYTES,
            )?)?;
            ts_format::verify_model(&path, &expected)?;
            println!("verified {:?}", expected.root);
        }
        Some("diff") => {
            let old: ts_core::Manifest =
                serde_cbor::from_slice(&fs::read(args.next().context("missing old manifest")?)?)?;
            let new: ts_core::Manifest =
                serde_cbor::from_slice(&fs::read(args.next().context("missing new manifest")?)?)?;
            let old_by_name = old
                .tensors
                .iter()
                .map(|tensor| (&tensor.descriptor.name, tensor.tensor_hash))
                .collect::<std::collections::BTreeMap<_, _>>();
            let mut changed = 0;
            for tensor in &new.tensors {
                if old_by_name.get(&tensor.descriptor.name) != Some(&tensor.tensor_hash) {
                    println!("changed {}", tensor.descriptor.name);
                    changed += 1;
                }
            }
            println!("changed_tensors: {changed}");
        }
        _ => {
            usage();
            anyhow::bail!("unknown or missing command");
        }
    }
    Ok(())
}
