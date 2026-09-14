use std::net::TcpListener;
use std::process::Stdio;
use std::time::Duration;
use ts_core::{
    sha256, ArtifactFormat, ChunkRef, FileRecipe, Manifest, Segment, TensorDescriptor, TensorNode,
};

fn free_address() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().to_string()
}

#[tokio::test]
async fn daemon_process_owns_control_and_proxy_until_shutdown() {
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    let root = tempfile::tempdir().unwrap();
    let manifest = Manifest::new(
        ArtifactFormat::Safetensors,
        4,
        vec![],
        vec![FileRecipe {
            path: "model.safetensors".into(),
            format: ArtifactFormat::Safetensors,
            file_size: 4,
            segments: vec![Segment::Literal {
                offset: 0,
                bytes: b"test".to_vec(),
            }],
        }],
    );
    let manifest_path = root.path().join("model.tswarm");
    std::fs::write(&manifest_path, manifest.to_bytes()).unwrap();
    let store = ts_store::ObjectStore::open(root.path()).unwrap();
    let dead_hash = sha256(b"unreferenced");
    store.put_verified(dead_hash, b"unreferenced").unwrap();
    let prepared_path = root.path().join("prepared.safetensors");
    let control_bind = free_address();
    let proxy_bind = free_address();
    let binary = env!("CARGO_BIN_EXE_ts-daemon");
    let mut child = std::process::Command::new(binary)
        .args([
            "--control-bind",
            &control_bind,
            "--proxy-bind",
            &proxy_bind,
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--store",
            root.path().to_str().unwrap(),
            "--origin",
            "https://127.0.0.1:1/model.safetensors",
        ])
        .env("TS_DAEMON_AUTH_TOKEN", "acceptance-token")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let client = reqwest::Client::new();
    let control_url = format!("http://{control_bind}");
    let proxy_url = format!("http://{proxy_bind}/healthz");
    let result = AssertUnwindSafe(async {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if client
                    .get(format!("{control_url}/healthz"))
                    .send()
                    .await
                    .is_ok()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("daemon control endpoint did not start");
        let unauthorized = client
            .get(format!("{control_url}/v1/status"))
            .send()
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);
        let malformed_transfer = client
            .post(format!("{control_url}/v1/transfers"))
            .bearer_auth("acceptance-token")
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            malformed_transfer.status(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY
        );
        let status = client
            .get(format!("{control_url}/v1/status"))
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(status.status(), reqwest::StatusCode::OK);
        let body: serde_json::Value = status.json().await.unwrap();
        assert_eq!(body["p2p"], "running");
        assert_eq!(body["proxy"], "running");
        let models = client
            .get(format!("{control_url}/v1/models"))
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(models.status(), reqwest::StatusCode::OK);
        let models: serde_json::Value = models.json().await.unwrap();
        assert_eq!(models[0]["path"], "model.safetensors");
        assert_eq!(models[0]["file_size"], 4);
        let verification = client
            .get(format!("{control_url}/v1/verification"))
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(verification.status(), reqwest::StatusCode::OK);
        let verification: serde_json::Value = verification.json().await.unwrap();
        assert_eq!(verification["total_chunks"], 0);
        assert_eq!(verification["verified_chunks"], 0);
        assert_eq!(verification["complete"], true);
        let peers = client
            .get(format!("{control_url}/v1/peers"))
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(peers.status(), reqwest::StatusCode::OK);
        let peers: serde_json::Value = peers.json().await.unwrap();
        assert_eq!(peers[0]["status"], "running");
        assert!(!peers[0]["peer_id"].as_str().unwrap().is_empty());
        let metrics = client
            .get(format!("{control_url}/v1/metrics"))
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(metrics.status(), reqwest::StatusCode::OK);
        let metrics: serde_json::Value = metrics.json().await.unwrap();
        assert_eq!(metrics["attempts"], 0);
        assert_eq!(metrics["bytes_transferred"], 0);
        let settings = client
            .get(format!("{control_url}/v1/settings"))
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(settings.status(), reqwest::StatusCode::OK);
        let settings: serde_json::Value = settings.json().await.unwrap();
        assert_eq!(settings["control_bind"], control_bind);
        assert_eq!(settings["proxy_bind"], proxy_bind);
        assert_eq!(
            settings["origin_url"],
            "https://127.0.0.1:1/model.safetensors"
        );
        let prepared = client
            .post(format!("{control_url}/v1/prepare"))
            .bearer_auth("acceptance-token")
            .json(&serde_json::json!({"output": prepared_path}))
            .send()
            .await
            .unwrap();
        assert_eq!(prepared.status(), reqwest::StatusCode::OK);
        let prepared: serde_json::Value = prepared.json().await.unwrap();
        assert_eq!(prepared["complete"], true);
        assert_eq!(std::fs::read(&prepared_path).unwrap(), b"test");
        let repaired = client
            .post(format!("{control_url}/v1/cache/repair"))
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(repaired.status(), reqwest::StatusCode::OK);
        let repaired: serde_json::Value = repaired.json().await.unwrap();
        assert_eq!(repaired["removed"], 1);
        assert!(!store.contains(dead_hash));
        assert_eq!(
            client.get(proxy_url).send().await.unwrap().status(),
            reqwest::StatusCode::OK
        );
    })
    .catch_unwind()
    .await;
    let _ = child.kill();
    let _ = child.wait();
    result.unwrap();
    assert!(TcpListener::bind(&control_bind).is_ok());
    assert!(TcpListener::bind(&proxy_bind).is_ok());
}

#[tokio::test]
async fn daemon_transfer_fetches_and_persists_verified_chunk() {
    let provider_root = tempfile::tempdir().unwrap();
    let daemon_root = tempfile::tempdir().unwrap();
    let payload = b"peer-data";
    let chunk_hash = sha256(payload);
    let tensor_hash = sha256(payload);
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
    let provider_store = ts_store::ObjectStore::open(provider_root.path()).unwrap();
    provider_store.put_verified(chunk_hash, payload).unwrap();
    let provider_swarm = ts_p2p::build_lan_swarm_with_listeners(&["/ip4/127.0.0.1/tcp/0"]).unwrap();
    let provider_id = *provider_swarm.local_peer_id();
    let provider = ts_p2p::ChunkProvider::from_manifest(&provider_store, &manifest).unwrap();
    let (listen_tx, listen_rx) = tokio::sync::oneshot::channel();
    let provider_task = tokio::spawn(async move {
        ts_p2p::run_lan_node_with_provider_and_notify(provider_swarm, provider, Some(listen_tx))
            .await
    });
    let provider_address = tokio::time::timeout(Duration::from_secs(5), listen_rx)
        .await
        .unwrap()
        .unwrap();
    let manifest_path = daemon_root.path().join("model.tswarm");
    std::fs::write(&manifest_path, manifest.to_bytes()).unwrap();
    let daemon_seed_store = ts_store::ObjectStore::open(daemon_root.path()).unwrap();
    daemon_seed_store.put_verified(chunk_hash, payload).unwrap();
    let control_bind = free_address();
    let proxy_bind = free_address();
    let mut daemon = std::process::Command::new(env!("CARGO_BIN_EXE_ts-daemon"))
        .args([
            "--control-bind",
            &control_bind,
            "--proxy-bind",
            &proxy_bind,
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--store",
            daemon_root.path().to_str().unwrap(),
            "--origin",
            "https://127.0.0.1:1/model",
        ])
        .env("TS_DAEMON_AUTH_TOKEN", "transfer-token")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let client = reqwest::Client::new();
    let control_url = format!("http://{control_bind}");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if client
                .get(format!("{control_url}/healthz"))
                .send()
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    let response = client.post(format!("{control_url}/v1/transfers"))
        .bearer_auth("transfer-token")
        .json(&serde_json::json!({
            "peer_id": provider_id.to_string(),
            "peer_address": provider_address.to_string(),
            "chunk": { "request_id": 1, "tensor_hash": tensor_hash, "chunk_index": 0, "expected_hash": chunk_hash },
            "max_attempts": 2,
        }))
        .send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["verified"], true);
    assert_eq!(body["bytes"], payload.len());
    let metrics = client
        .get(format!("{control_url}/v1/metrics"))
        .bearer_auth("transfer-token")
        .send()
        .await
        .unwrap();
    assert_eq!(metrics.status(), reqwest::StatusCode::OK);
    let metrics: serde_json::Value = metrics.json().await.unwrap();
    assert_eq!(metrics["attempts"], 1);
    assert_eq!(metrics["successful_chunks"], 1);
    assert_eq!(metrics["bytes_transferred"], payload.len());
    let daemon_store = ts_store::ObjectStore::open(daemon_root.path()).unwrap();
    assert!(daemon_store.contains(chunk_hash));
    let _ = daemon.kill();
    let _ = daemon.wait();
    provider_task.abort();
}
