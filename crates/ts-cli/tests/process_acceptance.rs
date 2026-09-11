use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use ts_core::{
    sha256, ArtifactFormat, ChunkRef, FileRecipe, Manifest, Segment, TensorDescriptor, TensorNode,
};
use ts_store::ObjectStore;

fn hex(hash: ts_core::Hash32) -> String {
    hash.0.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn two_cli_processes_transfer_and_verify_a_chunk() {
    let payload = b"independent-process-chunk";
    let chunk_hash = sha256(payload);
    let tensor_hash = chunk_hash;
    let manifest = Manifest::new(
        ArtifactFormat::Safetensors,
        payload.len() as u64,
        vec![TensorNode {
            descriptor: TensorDescriptor {
                name: "weight".into(),
                shape: vec![payload.len() as u64],
                dtype: "U8".into(),
                byte_len: payload.len() as u64,
            },
            tensor_hash,
            chunks: vec![ChunkRef {
                index: 0,
                offset: 0,
                length: payload.len() as u64,
                hash: chunk_hash,
            }],
        }],
        vec![FileRecipe {
            path: "model.safetensors".into(),
            format: ArtifactFormat::Safetensors,
            file_size: payload.len() as u64,
            segments: vec![Segment::Tensor {
                offset: 0,
                tensor_hash,
                tensor_offset: 0,
                length: payload.len() as u64,
            }],
        }],
    );
    let directory = tempfile::tempdir().unwrap();
    let store_root = directory.path().join("provider-store");
    let store = ObjectStore::open(&store_root).unwrap();
    store.put_verified(chunk_hash, payload).unwrap();
    let manifest_path = directory.path().join("model.tswarm");
    std::fs::write(&manifest_path, manifest.to_bytes()).unwrap();
    let output_path = directory.path().join("fetched.bin");

    let executable = env!("CARGO_BIN_EXE_ts-cli");
    let mut node = Command::new(executable)
        .args([
            "node",
            manifest_path.to_str().unwrap(),
            store_root.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = node.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();
    let peer = lines
        .next()
        .unwrap()
        .unwrap()
        .strip_prefix("peer: ")
        .unwrap()
        .to_owned();
    let address = lines
        .next()
        .unwrap()
        .unwrap()
        .strip_prefix("listen: ")
        .unwrap()
        .replace("/ip4/0.0.0.0/", "/ip4/127.0.0.1/");
    let address = format!("{address}/p2p/{peer}");

    let fetched = Command::new(executable)
        .args([
            "fetch-chunk",
            &peer,
            &address,
            &hex(tensor_hash),
            "0",
            &hex(chunk_hash),
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    node.kill().unwrap();
    assert!(
        fetched.status.success(),
        "fetch process failed: {}",
        String::from_utf8_lossy(&fetched.stderr)
    );
    assert_eq!(std::fs::read(output_path).unwrap(), payload);
}
