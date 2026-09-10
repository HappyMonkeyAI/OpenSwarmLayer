use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn usage() {
    eprintln!("usage:\n  ts-cli node [manifest.tswarm] [store-root]\n  ts-cli proxy [bind] <root>\n  ts-cli proxy-manifest [bind] <manifest.tswarm> <store-root> <origin-url> [peer-id] [peer-address]\n  ts-cli inspect <model>\n  ts-cli manifest <model> <output.tswarm>\n  ts-cli verify <model> <manifest.tswarm>\n  ts-cli diff <old.tswarm> <new.tswarm>");
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
                        ts_p2p::publish_tensor(&mut swarm, &tensor.tensor_hash)?;
                    }
                    provider
                }
                None => ts_p2p::ChunkProvider::default(),
            };
            ts_p2p::run_lan_node_with_provider(swarm, provider).await?;
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
            let config = ts_proxy::ManifestProxyState {
                manifest,
                origin_url,
                engine: ts_proxy::FetchEngine::new(store),
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
            let expected: ts_core::Manifest = serde_cbor::from_slice(&fs::read(&manifest_path)?)?;
            let actual = ts_format::build_manifest(&path, 16 * 1024 * 1024)?;
            anyhow::ensure!(expected.root == actual.root, "manifest root mismatch");
            anyhow::ensure!(expected.verify_root(), "manifest self-check failed");
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
        _ => usage(),
    }
    Ok(())
}
