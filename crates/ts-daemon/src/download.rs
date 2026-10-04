//! Explicit-peer private downloads. Networking stays in the Rust engine.

use anyhow::{ensure, Context, Result};
use libp2p::{multiaddr::Protocol, swarm::dial_opts::DialOpts, Multiaddr, PeerId};
use std::path::Path;
pub use tokio_util::sync::CancellationToken;
use ts_core::Manifest;
use ts_store::{ObjectStore, StoreError};

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct DownloadProgress {
    pub verified_chunks: usize,
    pub total_chunks: usize,
    pub verified_bytes: u64,
    pub total_bytes: u64,
}

pub fn validate_peer(peer: &str, address: &str) -> Result<(PeerId, Multiaddr)> {
    ensure!(
        peer.len() <= 128 && address.len() <= 2048,
        "peer input is too long"
    );
    let peer: PeerId = peer.parse().context("invalid peer ID")?;
    let address: Multiaddr = address.parse().context("invalid peer address")?;
    ensure!(!address.is_empty(), "peer address is empty");
    for protocol in address.iter() {
        if let Protocol::P2p(found) = protocol {
            ensure!(found == peer, "address identifies a different peer");
        }
    }
    Ok((peer, address))
}

pub fn validate_download_manifest(manifest: &Manifest) -> Result<DownloadProgress> {
    ensure!(
        manifest.files.len() == 1,
        "download requires a single-file manifest"
    );
    let recipe = ts_proxy::validate_manifest_recipe(manifest)?;
    ensure!(recipe.format == manifest.format, "artifact format mismatch");
    ensure!(!manifest.tensors.is_empty(), "model contains no tensors");
    let mut names = std::collections::HashSet::new();
    let mut hashes = std::collections::HashSet::new();
    let mut progress = DownloadProgress::default();
    for tensor in &manifest.tensors {
        ensure!(
            names.insert(&tensor.descriptor.name) && hashes.insert(tensor.tensor_hash),
            "duplicate tensor identity"
        );
        let mut cursor = 0u64;
        for (index, chunk) in tensor.chunks.iter().enumerate() {
            ensure!(
                chunk.index as usize == index && chunk.offset == cursor,
                "invalid chunk layout"
            );
            ensure!(
                chunk.length > 0 && chunk.length <= ts_p2p::MAX_CHUNK_BYTES as u64,
                "unsupported chunk length"
            );
            cursor = cursor
                .checked_add(chunk.length)
                .context("chunk length overflow")?;
            progress.total_chunks = progress
                .total_chunks
                .checked_add(1)
                .context("chunk count overflow")?;
            progress.total_bytes = progress
                .total_bytes
                .checked_add(chunk.length)
                .context("download size overflow")?;
        }
        ensure!(
            cursor == tensor.descriptor.byte_len,
            "chunks do not cover tensor"
        );
    }
    Ok(progress)
}

/// Rechecks cached chunks, fetches only missing ones, and reports durable verified progress.
/// The caller supplies a separate cache so library repair cannot erase a partial download.
pub async fn fetch_manifest_chunks(
    manifest: &Manifest,
    store: &ObjectStore,
    peer: &str,
    address: &str,
    cancel: CancellationToken,
    mut notify: impl FnMut(DownloadProgress) + Send,
) -> Result<()> {
    let (peer, address) = validate_peer(peer, address)?;
    let mut progress = validate_download_manifest(manifest)?;
    notify(progress);
    let mut client = None;
    for tensor in &manifest.tensors {
        for chunk in &tensor.chunks {
            ensure!(!cancel.is_cancelled(), "download cancelled");
            match store.get(chunk.hash) {
                Ok(bytes) => ensure!(
                    bytes.len() as u64 == chunk.length,
                    "cached chunk length mismatch"
                ),
                Err(StoreError::Missing(_)) => {
                    if client.is_none() {
                        let mut connected =
                            ts_p2p::LanClient::new(ts_p2p::build_lan_swarm_with_listeners(&[])?);
                        connected.swarm_mut().dial(
                            DialOpts::peer_id(peer)
                                .addresses(vec![address.clone()])
                                .build(),
                        )?;
                        client = Some(connected);
                    }
                    let bytes = client
                        .as_mut()
                        .unwrap()
                        .fetch_chunk_with_cancel(
                            peer,
                            ts_p2p::ChunkRequest {
                                request_id: chunk.index as u64,
                                tensor_hash: tensor.tensor_hash,
                                chunk_index: chunk.index,
                                expected_hash: chunk.hash,
                            },
                            3,
                            cancel.clone(),
                        )
                        .await?;
                    ensure!(!cancel.is_cancelled(), "download cancelled");
                    ensure!(
                        bytes.len() as u64 == chunk.length,
                        "fetched chunk length mismatch"
                    );
                    store.put_verified(chunk.hash, &bytes)?;
                    ensure!(store.get(chunk.hash)? == bytes, "chunk readback differs");
                }
                Err(error) => return Err(error.into()),
            }
            progress.verified_chunks += 1;
            progress.verified_bytes += chunk.length;
            notify(progress);
        }
    }
    ensure!(!cancel.is_cancelled(), "download cancelled");
    Ok(())
}

/// Prepare and inspect a real model in a sibling staging directory, then commit
/// with a no-replace hard link. An existing destination is never truncated.
pub fn prepare_verified_download(
    manifest: &Manifest,
    store: &ObjectStore,
    output: &Path,
) -> Result<()> {
    validate_download_manifest(manifest)?;
    ensure!(
        !output.exists() && !output.with_extension("partial").exists(),
        "output or partial path exists; choose a new output"
    );
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let staged = tempfile::Builder::new()
        .prefix(".tswarm-received-")
        .tempdir_in(parent)?;
    let prepared = staged.path().join("model");
    store.materialize(manifest, &prepared)?;
    let index =
        ts_format::inspect(&prepared).context("downloaded file is not a supported model")?;
    ensure!(
        index.format == manifest.format && index.tensors.len() == manifest.tensors.len(),
        "model metadata differs from manifest"
    );
    let mut prepared_file = std::fs::File::open(&prepared)?;
    for tensor in &index.tensors {
        ensure!(
            manifest
                .tensors
                .iter()
                .any(|node| node.descriptor == tensor.descriptor),
            "model tensor metadata differs from manifest"
        );
        // Tensor identity is independent of chunk size. Verify it against the
        // real container, rather than rebuilding a root with default chunks.
        let block_size = if tensor.descriptor.dtype.starts_with("ggml:") {
            32
        } else {
            1
        };
        let actual = ts_format::hash_tensor(
            &mut prepared_file,
            tensor,
            ts_p2p::MAX_CHUNK_BYTES as u64,
            block_size,
        )?;
        ensure!(
            manifest
                .tensors
                .iter()
                .any(|node| node.descriptor == actual.descriptor
                    && node.tensor_hash == actual.tensor_hash),
            "model tensor identity differs from manifest"
        );
    }
    std::fs::hard_link(&prepared, output)
        .context("could not commit download without replacing an existing file")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::{Read, Seek, SeekFrom, Write},
    };

    fn fixture(root: &Path) -> (Manifest, ObjectStore, Vec<u8>) {
        let payload = b"private downloaded model with multiple verified chunks";
        let header = format!(
            r#"{{"weight":{{"dtype":"U8","shape":[{}],"data_offsets":[0,{}]}}}}"#,
            payload.len(),
            payload.len()
        );
        let source = root.join("model.safetensors");
        let mut file = fs::File::create(&source).unwrap();
        file.write_all(&(header.len() as u64).to_le_bytes())
            .unwrap();
        file.write_all(header.as_bytes()).unwrap();
        file.write_all(payload).unwrap();
        drop(file);
        let manifest = ts_format::build_manifest(&source, 16).unwrap();
        let offset = ts_format::inspect(&source).unwrap().tensors[0].offset;
        let store = ObjectStore::open(root.join("source-store")).unwrap();
        let mut file = fs::File::open(&source).unwrap();
        for chunk in &manifest.tensors[0].chunks {
            let mut bytes = vec![0; chunk.length as usize];
            file.seek(SeekFrom::Start(offset + chunk.offset)).unwrap();
            file.read_exact(&mut bytes).unwrap();
            store.put_verified(chunk.hash, &bytes).unwrap();
        }
        (manifest, store, fs::read(source).unwrap())
    }

    struct Node(tokio::task::JoinHandle<anyhow::Result<()>>);
    impl Drop for Node {
        fn drop(&mut self) {
            self.0.abort();
        }
    }

    async fn seed(manifest: &Manifest, store: &ObjectStore) -> (Node, String, String) {
        let swarm = ts_p2p::build_lan_swarm_with_listeners(&["/ip4/127.0.0.1/tcp/0"]).unwrap();
        let peer = swarm.local_peer_id().to_string();
        let provider = ts_p2p::ChunkProvider::from_manifest(store, manifest).unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(ts_p2p::run_lan_node_with_provider_and_notify(
            swarm,
            provider,
            Some(tx),
        ));
        let address = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .unwrap()
            .unwrap()
            .to_string();
        (Node(task), peer, address)
    }

    #[test]
    fn rejects_bad_peer_and_layout_and_preserves_existing_output() {
        let root = tempfile::tempdir().unwrap();
        let (manifest, store, _) = fixture(root.path());
        let peer = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id()
            .to_string();
        let other = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id()
            .to_string();
        assert!(validate_peer("bad", "/ip4/127.0.0.1/tcp/1").is_err());
        assert!(validate_peer(&peer, "bad").is_err());
        assert!(validate_peer(&peer, &format!("/ip4/127.0.0.1/tcp/1/p2p/{other}")).is_err());
        let output = root.path().join("existing.safetensors");
        fs::write(&output, b"preserve").unwrap();
        assert!(prepare_verified_download(&manifest, &store, &output).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"preserve");
        let mut broken = manifest.clone();
        broken.files[0].segments.clear();
        broken = Manifest::new(
            broken.format,
            broken.file_size,
            broken.tensors,
            broken.files,
        );
        assert!(validate_download_manifest(&broken).is_err());
        assert!(prepare_verified_download(&broken, &store, &root.path().join("bad")).is_err());
        assert!(!root.path().join("bad").exists());
        let mut wrong_identity = manifest.clone();
        wrong_identity.tensors[0].tensor_hash = ts_core::Hash32([9; 32]);
        for segment in &mut wrong_identity.files[0].segments {
            if let ts_core::Segment::Tensor { tensor_hash, .. } = segment {
                *tensor_hash = ts_core::Hash32([9; 32]);
            }
        }
        let wrong_identity = Manifest::new(
            wrong_identity.format,
            wrong_identity.file_size,
            wrong_identity.tensors,
            wrong_identity.files,
        );
        assert!(
            prepare_verified_download(&wrong_identity, &store, &root.path().join("wrong"))
                .unwrap_err()
                .to_string()
                .contains("tensor identity")
        );
        assert!(!root.path().join("wrong").exists());
    }

    #[tokio::test]
    async fn downloads_resumes_reseeds_and_reuses_cache_offline() {
        let root = tempfile::tempdir().unwrap();
        let (manifest, source, expected) = fixture(root.path());
        let (sender, peer, address) = seed(&manifest, &source).await;
        let recipient = ObjectStore::open(root.path().join("recipient")).unwrap();
        let first = &manifest.tensors[0].chunks[0];
        recipient
            .put_verified(first.hash, &source.get(first.hash).unwrap())
            .unwrap();
        let mut progress = Vec::new();
        fetch_manifest_chunks(
            &manifest,
            &recipient,
            &peer,
            &address,
            CancellationToken::new(),
            |p| progress.push(p),
        )
        .await
        .unwrap();
        assert_eq!(
            progress.last().unwrap().verified_chunks,
            manifest.tensors[0].chunks.len()
        );
        assert_eq!(
            progress.last().unwrap().verified_bytes,
            manifest.tensors[0].descriptor.byte_len
        );
        let output = root.path().join("received.safetensors");
        prepare_verified_download(&manifest, &recipient, &output).unwrap();
        assert_eq!(fs::read(output).unwrap(), expected);
        sender.0.abort();
        let (second, peer, address) = seed(&manifest, &recipient).await;
        let third = ObjectStore::open(root.path().join("third")).unwrap();
        fetch_manifest_chunks(
            &manifest,
            &third,
            &peer,
            &address,
            CancellationToken::new(),
            |_| {},
        )
        .await
        .unwrap();
        let output = root.path().join("third.safetensors");
        prepare_verified_download(&manifest, &third, &output).unwrap();
        assert_eq!(fs::read(output).unwrap(), expected);
        second.0.abort();
        fetch_manifest_chunks(
            &manifest,
            &third,
            &peer,
            "/ip4/127.0.0.1/tcp/1",
            CancellationToken::new(),
            |_| {},
        )
        .await
        .unwrap();
        let chunk = &manifest.tensors[0].chunks[0];
        fs::write(third.object_path(chunk.hash), b"corrupt").unwrap();
        assert!(fetch_manifest_chunks(
            &manifest,
            &third,
            &peer,
            "/ip4/127.0.0.1/tcp/1",
            CancellationToken::new(),
            |_| {}
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("object hash mismatch"));
    }

    #[tokio::test]
    async fn cancelling_after_one_commit_preserves_only_verified_partial_cache() {
        let root = tempfile::tempdir().unwrap();
        let (manifest, source, _) = fixture(root.path());
        let (_sender, peer, address) = seed(&manifest, &source).await;
        let store = ObjectStore::open(root.path().join("partial")).unwrap();
        let token = CancellationToken::new();
        let notify_token = token.clone();
        let result =
            fetch_manifest_chunks(&manifest, &store, &peer, &address, token, move |progress| {
                if progress.verified_chunks == 1 {
                    notify_token.cancel();
                }
            })
            .await;
        assert!(result.unwrap_err().to_string().contains("cancelled"));
        assert!(store.get(manifest.tensors[0].chunks[0].hash).is_ok());
        assert!(matches!(
            store.get(manifest.tensors[0].chunks[1].hash),
            Err(StoreError::Missing(_))
        ));
        assert!(!root.path().join("output").exists());
    }
}
