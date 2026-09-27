use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

use ts_core::{
    sha256, ArtifactFormat, ArtifactVariant, ChunkRef, FileRecipe, Manifest, ModelArtifactFile,
    ModelCard, ModelReleaseDescriptor, ModelShareLink, ModelSigningKey, Segment,
    SignedModelRelease, TensorDescriptor, TensorNode,
};
use ts_store::ObjectStore;

fn hex(hash: ts_core::Hash32) -> String {
    hash.0.iter().map(|byte| format!("{byte:02x}")).collect()
}

struct NodeGuard(Child);

impl Drop for NodeGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
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
    assert!(!Command::new(executable)
        .arg("nonexistent-command")
        .output()
        .unwrap()
        .status
        .success());
    let mut node = NodeGuard(
        Command::new(executable)
            .args([
                "node",
                manifest_path.to_str().unwrap(),
                store_root.to_str().unwrap(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stdout = node.0.stdout.take().unwrap();
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

    assert!(
        fetched.status.success(),
        "fetch process failed: {}",
        String::from_utf8_lossy(&fetched.stderr)
    );
    assert_eq!(std::fs::read(output_path).unwrap(), payload);

    let prepared_path = directory.path().join("prepared.safetensors");
    let prepared = Command::new(executable)
        .args([
            "prepare",
            manifest_path.to_str().unwrap(),
            store_root.to_str().unwrap(),
            prepared_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        prepared.status.success(),
        "prepare process failed: {}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    assert_eq!(std::fs::read(prepared_path).unwrap(), payload);

    let descriptor = ModelReleaseDescriptor::new(
        ModelCard::new("Private fixture"),
        "v1",
        vec![ArtifactVariant {
            id: "default".into(),
            format: ArtifactFormat::Safetensors,
            quantization: None,
            files: vec![ModelArtifactFile {
                path: "model.safetensors".into(),
                size_bytes: payload.len() as u64,
                tensor_count: 1,
                manifest_root: manifest.root,
                shard: None,
            }],
        }],
    );
    let signed =
        SignedModelRelease::sign(descriptor, &ModelSigningKey::from_bytes(&[41; 32])).unwrap();
    let bundle = directory.path().join("private.tsrelease");
    std::fs::write(&bundle, signed.to_bytes().unwrap()).unwrap();
    let uri = ModelShareLink::for_release(&signed).unwrap().to_uri();
    let verify = |bundle: &std::path::Path, manifest: &std::path::Path, uri: &str| {
        Command::new(executable)
            .args([
                "verify-release",
                bundle.to_str().unwrap(),
                manifest.to_str().unwrap(),
                uri,
            ])
            .output()
            .unwrap()
    };
    let accepted = verify(&bundle, &manifest_path, &uri);
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert!(String::from_utf8_lossy(&accepted.stdout).contains("descriptor and manifest match"));
    assert!(Command::new(executable)
        .args([
            "verify-release",
            bundle.to_str().unwrap(),
            manifest_path.to_str().unwrap()
        ])
        .status()
        .unwrap()
        .success());

    let different = Manifest::new(
        ArtifactFormat::Safetensors,
        payload.len() as u64,
        manifest.tensors.clone(),
        vec![FileRecipe {
            path: "other.safetensors".into(),
            ..manifest.files[0].clone()
        }],
    );
    let different_path = directory.path().join("different.tswarm");
    std::fs::write(&different_path, different.to_bytes()).unwrap();
    assert!(!verify(&bundle, &different_path, &uri).status.success());
    let mut invalid_root = manifest.clone();
    invalid_root.root = sha256(b"invalid root");
    let invalid_root_path = directory.path().join("invalid-root.tswarm");
    std::fs::write(&invalid_root_path, invalid_root.to_bytes()).unwrap();
    assert!(!verify(&bundle, &invalid_root_path, &uri).status.success());
    assert!(!verify(&bundle, &manifest_path, "tswarm://v1/bad")
        .status
        .success());
    let wrong_uri = ModelShareLink::for_release(
        &SignedModelRelease::sign(
            signed.descriptor.clone(),
            &ModelSigningKey::from_bytes(&[42; 32]),
        )
        .unwrap(),
    )
    .unwrap()
    .to_uri();
    assert!(!verify(&bundle, &manifest_path, &wrong_uri).status.success());
    let mut tampered = signed.to_bytes().unwrap();
    *tampered.last_mut().unwrap() ^= 1;
    let tampered_path = directory.path().join("tampered.tsrelease");
    std::fs::write(&tampered_path, tampered).unwrap();
    assert!(!verify(&tampered_path, &manifest_path, &uri)
        .status
        .success());

    let inbox = directory.path().join("recipient-inbox");
    let receive = |bundle: &std::path::Path, manifest: &std::path::Path, uri: &str| {
        Command::new(executable)
            .args([
                "receive-release",
                bundle.to_str().unwrap(),
                manifest.to_str().unwrap(),
                inbox.to_str().unwrap(),
                uri,
            ])
            .output()
            .unwrap()
    };
    assert!(!receive(&tampered_path, &manifest_path, &uri)
        .status
        .success());
    assert!(!receive(&bundle, &different_path, &uri).status.success());
    assert!(!receive(&bundle, &manifest_path, &wrong_uri)
        .status
        .success());
    assert!(!inbox.exists(), "invalid input must not create an inbox");
    let received = receive(&bundle, &manifest_path, &uri);
    assert!(
        received.status.success(),
        "{}",
        String::from_utf8_lossy(&received.stderr)
    );
    let record = inbox.join(hex(signed.release_id().unwrap()));
    assert_eq!(
        std::fs::read(record.join("release.tsrelease")).unwrap(),
        signed.to_bytes().unwrap()
    );
    assert_eq!(
        std::fs::read(record.join("manifest.tswarm")).unwrap(),
        manifest.to_bytes()
    );
    assert!(
        receive(&bundle, &manifest_path, &uri).status.success(),
        "repeat receive is idempotent"
    );
    let recipient_store = directory.path().join("recipient-store");
    let recipient_output = directory.path().join("recipient-model.safetensors");
    let fetch_received = || {
        Command::new(executable)
            .args([
                "fetch-received",
                record.to_str().unwrap(),
                &peer,
                &address,
                recipient_store.to_str().unwrap(),
                recipient_output.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let wrong_record = inbox.join("wrong-release-id");
    std::fs::rename(&record, &wrong_record).unwrap();
    let wrong_name = Command::new(executable)
        .args([
            "fetch-received",
            wrong_record.to_str().unwrap(),
            &peer,
            &address,
            recipient_store.to_str().unwrap(),
            recipient_output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!wrong_name.status.success());
    assert!(!recipient_store.exists() && !recipient_output.exists());
    std::fs::rename(wrong_record, &record).unwrap();
    let fetched_received = fetch_received();
    assert!(
        fetched_received.status.success(),
        "{}",
        String::from_utf8_lossy(&fetched_received.stderr)
    );
    assert_eq!(std::fs::read(&recipient_output).unwrap(), payload);
    assert!(std::fs::read_dir(directory.path())
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".tswarm-received-")));
    assert_eq!(
        ObjectStore::open(&recipient_store)
            .unwrap()
            .get(chunk_hash)
            .unwrap(),
        payload
    );
    node.0.kill().unwrap();
    node.0.wait().unwrap();
    std::fs::remove_file(&recipient_output).unwrap();
    assert!(
        fetch_received().status.success(),
        "verified recipient cache should survive provider shutdown"
    );
    assert_eq!(std::fs::read(&recipient_output).unwrap(), payload);
    std::fs::remove_file(&recipient_output).unwrap();
    let object = ObjectStore::open(&recipient_store)
        .unwrap()
        .object_path(chunk_hash);
    std::fs::write(&object, b"bad cached bytes").unwrap();
    assert!(
        !fetch_received().status.success(),
        "corrupted cache must fail closed"
    );
    assert!(!recipient_output.exists());
    assert_eq!(std::fs::read(&object).unwrap(), b"bad cached bytes");
    std::fs::write(record.join("manifest.tswarm"), b"corrupted").unwrap();
    assert!(
        !receive(&bundle, &manifest_path, &uri).status.success(),
        "existing mismatched record must not be replaced"
    );
    assert_eq!(
        std::fs::read(record.join("manifest.tswarm")).unwrap(),
        b"corrupted"
    );
    assert!(
        !fetch_received().status.success(),
        "corrupted recipient metadata must be rejected before cached output"
    );
    assert!(!recipient_output.exists());
}
