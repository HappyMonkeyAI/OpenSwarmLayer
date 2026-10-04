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

struct ChildGuard(std::process::Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::test]
async fn daemon_starts_without_a_manifest_for_an_empty_desktop_library() {
    let root = tempfile::tempdir().unwrap();
    let control_bind = free_address();
    let proxy_bind = free_address();
    let child = ChildGuard(
        std::process::Command::new(env!("CARGO_BIN_EXE_ts-daemon"))
            .args([
                "--control-bind",
                &control_bind,
                "--proxy-bind",
                &proxy_bind,
                "--store",
                root.path().to_str().unwrap(),
            ])
            .env("TS_DAEMON_AUTH_TOKEN", "empty-library-token")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );

    let client = reqwest::Client::new();
    let control_url = format!("http://{control_bind}");
    let result = tokio::time::timeout(Duration::from_secs(5), async {
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
    .await;
    if result.is_ok() {
        let status = client
            .get(format!("{control_url}/v1/status"))
            .bearer_auth("empty-library-token")
            .send()
            .await
            .unwrap();
        assert_eq!(status.status(), reqwest::StatusCode::OK);
        let models = client
            .get(format!("{control_url}/v1/models"))
            .bearer_auth("empty-library-token")
            .send()
            .await
            .unwrap();
        assert_eq!(models.status(), reqwest::StatusCode::OK);
        assert_eq!(
            models.json::<serde_json::Value>().await.unwrap(),
            serde_json::json!([])
        );
        let peers = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let peers = client
                    .get(format!("{control_url}/v1/peers"))
                    .bearer_auth("empty-library-token")
                    .send()
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await
                    .unwrap();
                if peers[0]["listen_addresses"]
                    .as_array()
                    .is_some_and(|addresses| !addresses.is_empty())
                {
                    break peers;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("daemon did not expose its listening addresses");
        let addresses = peers[0]["listen_addresses"].as_array().unwrap();
        assert!(addresses
            .iter()
            .any(|address| address.as_str().unwrap().contains("/tcp/")));
        assert!(peers[0]["peer_id"]
            .as_str()
            .unwrap()
            .parse::<libp2p::PeerId>()
            .is_ok());
        assert_eq!(
            client
                .get(format!("{control_url}/v1/peers"))
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::UNAUTHORIZED
        );
    }
    drop(child);
    result.expect("daemon without a manifest did not become healthy");
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
        // Verify authentication failure path across control endpoints
        for endpoint in [
            "/v1/status",
            "/v1/models",
            "/v1/verification",
            "/v1/peers",
            "/v1/metrics",
            "/v1/settings",
        ] {
            let res = client
                .get(format!("{control_url}{endpoint}"))
                .send()
                .await
                .unwrap();
            assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);
            let bad_auth = client
                .get(format!("{control_url}{endpoint}"))
                .bearer_auth("wrong-token")
                .send()
                .await
                .unwrap();
            assert_eq!(bad_auth.status(), reqwest::StatusCode::UNAUTHORIZED);
        }
        for endpoint in ["/v1/prepare", "/v1/cache/repair", "/v1/transfers"] {
            let res = client
                .post(format!("{control_url}{endpoint}"))
                .send()
                .await
                .unwrap();
            assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);
        }

        // Verify CORS negotiation for desktop and local web origins
        let cors_res = client
            .get(format!("{control_url}/v1/status"))
            .header("Origin", "http://127.0.0.1:8080")
            .bearer_auth("acceptance-token")
            .send()
            .await
            .unwrap();
        assert_eq!(cors_res.status(), reqwest::StatusCode::OK);
        assert_eq!(
            cors_res
                .headers()
                .get("Access-Control-Allow-Origin")
                .unwrap()
                .to_str()
                .unwrap(),
            "http://127.0.0.1:8080"
        );
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

#[tokio::test]
async fn daemon_process_tracks_and_prepares_each_selected_model() {
    let root = tempfile::tempdir().unwrap();
    let make_model = |path: &str, tensor_name: &str, payload: &[u8]| {
        let hash = sha256(payload);
        let tensor = TensorNode {
            descriptor: TensorDescriptor {
                name: tensor_name.into(),
                shape: vec![payload.len() as u64],
                dtype: "U8".into(),
                byte_len: payload.len() as u64,
            },
            tensor_hash: hash,
            chunks: vec![ChunkRef {
                index: 0,
                offset: 0,
                length: payload.len() as u64,
                hash,
            }],
        };
        let recipe = FileRecipe {
            path: path.into(),
            format: ArtifactFormat::Safetensors,
            file_size: payload.len() as u64,
            segments: vec![Segment::Tensor {
                offset: 0,
                tensor_hash: hash,
                tensor_offset: 0,
                length: payload.len() as u64,
            }],
        };
        let manifest = Manifest::new(
            ArtifactFormat::Safetensors,
            payload.len() as u64,
            vec![tensor],
            vec![recipe],
        );
        (manifest, hash)
    };
    let (first, first_hash) = make_model("first.safetensors", "first_weight", b"alpha");
    let (second, second_hash) = make_model("second.safetensors", "second_weight", b"beta");
    let (missing, _) = make_model("missing.safetensors", "missing_weight", b"gamma");
    let combined = Manifest::new(
        ArtifactFormat::Safetensors,
        first.file_size + second.file_size + missing.file_size,
        [
            first.tensors.clone(),
            second.tensors.clone(),
            missing.tensors.clone(),
        ]
        .concat(),
        [
            first.files.clone(),
            second.files.clone(),
            missing.files.clone(),
        ]
        .concat(),
    );
    let manifest_path = root.path().join("catalog.tswarm");
    std::fs::write(&manifest_path, combined.to_bytes()).unwrap();
    let store = ts_store::ObjectStore::open(root.path()).unwrap();
    store.put_verified(first_hash, b"alpha").unwrap();
    store.put_verified(second_hash, b"beta").unwrap();

    let control_bind = free_address();
    let proxy_bind = free_address();
    let child = ChildGuard(
        std::process::Command::new(env!("CARGO_BIN_EXE_ts-daemon"))
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
                "https://127.0.0.1:1/models",
            ])
            .env("TS_DAEMON_AUTH_TOKEN", "multi-model-token")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
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
    .expect("multi-model daemon did not become healthy");

    let models = client
        .get(format!("{control_url}/v1/models"))
        .bearer_auth("multi-model-token")
        .send()
        .await
        .unwrap();
    assert_eq!(models.status(), reqwest::StatusCode::OK);
    let models: serde_json::Value = models.json().await.unwrap();
    assert_eq!(models.as_array().unwrap().len(), 3);
    for (index, (model, manifest, path)) in [
        (&models[0], &first, "first.safetensors"),
        (&models[1], &second, "second.safetensors"),
        (&models[2], &missing, "missing.safetensors"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(model["path"], path);
        assert_eq!(
            model["manifest_root"],
            serde_json::to_value(manifest.root).unwrap()
        );
        assert_eq!(model["tensors"], 1);
        assert_eq!(model["total_chunks"], 1);
        let available = index < 2;
        assert_eq!(model["verified_chunks"], usize::from(available));
        assert_eq!(model["complete"], available);
    }

    let omitted_output = root.path().join("omitted-selection.safetensors");
    let omitted = client
        .post(format!("{control_url}/v1/prepare"))
        .bearer_auth("multi-model-token")
        .json(&serde_json::json!({"output": omitted_output}))
        .send()
        .await
        .unwrap();
    assert_eq!(omitted.status(), reqwest::StatusCode::BAD_REQUEST);
    assert!(!omitted_output.exists());

    let unknown_path_output = root.path().join("unknown-path.safetensors");
    let unknown_path = client
        .post(format!("{control_url}/v1/prepare"))
        .bearer_auth("multi-model-token")
        .json(&serde_json::json!({
            "output": unknown_path_output,
            "path": "not-in-catalog.safetensors"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(unknown_path.status(), reqwest::StatusCode::BAD_REQUEST);
    assert!(!unknown_path_output.exists());

    let missing_output = root.path().join("missing-output.safetensors");
    let unavailable = client
        .post(format!("{control_url}/v1/prepare"))
        .bearer_auth("multi-model-token")
        .json(&serde_json::json!({
            "output": missing_output,
            "path": "missing.safetensors"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        unavailable.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
    assert!(!missing_output.exists());

    for (path, expected) in [
        ("first.safetensors", b"alpha".as_slice()),
        ("second.safetensors", b"beta".as_slice()),
    ] {
        let output = root.path().join(format!("prepared-{path}"));
        let prepared = client
            .post(format!("{control_url}/v1/prepare"))
            .bearer_auth("multi-model-token")
            .json(&serde_json::json!({"output": output, "path": path}))
            .send()
            .await
            .unwrap();
        assert_eq!(prepared.status(), reqwest::StatusCode::OK);
        let body: serde_json::Value = prepared.json().await.unwrap();
        assert_eq!(body["complete"], true);
        assert_eq!(std::fs::read(output).unwrap(), expected);
    }

    drop(child);
    assert!(TcpListener::bind(&control_bind).is_ok());
    assert!(TcpListener::bind(&proxy_bind).is_ok());
}

#[tokio::test]
async fn daemon_serves_a_chunk_after_transfer_commits_it() {
    let provider_root = tempfile::tempdir().unwrap();
    let daemon_root = tempfile::tempdir().unwrap();
    let payload = b"newly-fetched-peer-data";
    let chunk_hash = sha256(payload);
    let tensor_hash = sha256(b"tensor-identity");
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

    let source_store = ts_store::ObjectStore::open(provider_root.path()).unwrap();
    source_store.put_verified(chunk_hash, payload).unwrap();
    let source_swarm = ts_p2p::build_lan_swarm_with_listeners(&["/ip4/127.0.0.1/tcp/0"]).unwrap();
    let source_peer = *source_swarm.local_peer_id();
    let source_provider = ts_p2p::ChunkProvider::from_manifest(&source_store, &manifest).unwrap();
    let (source_listen_tx, source_listen_rx) = tokio::sync::oneshot::channel();
    let source_task = tokio::spawn(async move {
        ts_p2p::run_lan_node_with_provider_and_notify(
            source_swarm,
            source_provider,
            Some(source_listen_tx),
        )
        .await
    });
    let source_address = tokio::time::timeout(Duration::from_secs(5), source_listen_rx)
        .await
        .unwrap()
        .unwrap();

    let manifest_path = daemon_root.path().join("model.tswarm");
    std::fs::write(&manifest_path, manifest.to_bytes()).unwrap();
    let control_bind = free_address();
    let proxy_bind = free_address();
    let mut daemon = ChildGuard(
        std::process::Command::new(env!("CARGO_BIN_EXE_ts-daemon"))
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
            .env("TS_DAEMON_AUTH_TOKEN", "freshness-token")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let mut stdout = daemon.0.stdout.take().unwrap();
    let (line_tx, line_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::BufRead;
        for line in std::io::BufReader::new(&mut stdout).lines().flatten() {
            let _ = line_tx.send(line);
        }
    });
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
    .expect("daemon did not become healthy with a missing manifest chunk");

    let daemon_address = tokio::task::spawn_blocking(move || loop {
        let line = line_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("daemon did not announce a P2P listen address");
        if let Some(address) = line.strip_prefix("listen: ") {
            if address.contains("/tcp/") {
                break address.to_owned();
            }
        }
    })
    .await
    .unwrap();

    let invalid_identity = client
        .post(format!("{control_url}/v1/transfers"))
        .bearer_auth("freshness-token")
        .json(&serde_json::json!({
            "peer_id": source_peer.to_string(),
            "peer_address": source_address.to_string(),
            "chunk": { "request_id": 0, "tensor_hash": tensor_hash, "chunk_index": 0, "expected_hash": sha256(b"not-the-manifest-chunk") },
            "max_attempts": 1,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid_identity.status(), reqwest::StatusCode::BAD_REQUEST);

    let transfer = client
        .post(format!("{control_url}/v1/transfers"))
        .bearer_auth("freshness-token")
        .json(&serde_json::json!({
            "peer_id": source_peer.to_string(),
            "peer_address": source_address.to_string(),
            "chunk": { "request_id": 1, "tensor_hash": tensor_hash, "chunk_index": 0, "expected_hash": chunk_hash },
            "max_attempts": 1,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(transfer.status(), reqwest::StatusCode::OK);
    let stored = ts_store::ObjectStore::open(daemon_root.path()).unwrap();
    assert_eq!(stored.get(chunk_hash).unwrap(), payload);

    let peers = client
        .get(format!("{control_url}/v1/peers"))
        .bearer_auth("freshness-token")
        .send()
        .await
        .unwrap();
    assert_eq!(peers.status(), reqwest::StatusCode::OK);
    let peers: serde_json::Value = peers.json().await.unwrap();
    let daemon_peer = peers[0]["peer_id"].as_str().unwrap().parse().unwrap();
    let mut fetch_swarm = ts_p2p::build_lan_swarm_with_listeners(&[]).unwrap();
    fetch_swarm
        .dial(daemon_address.parse::<libp2p::Multiaddr>().unwrap())
        .unwrap();
    let mut fetcher = ts_p2p::LanClient::new(fetch_swarm);
    let served = fetcher
        .fetch_chunk(
            daemon_peer,
            ts_p2p::ChunkRequest {
                request_id: 9,
                tensor_hash,
                chunk_index: 0,
                expected_hash: chunk_hash,
            },
            1,
        )
        .await
        .unwrap();
    assert_eq!(served, payload);

    use futures::StreamExt;
    use libp2p::kad::{GetProvidersOk, QueryResult};
    use libp2p::swarm::SwarmEvent;
    let mut discovery = ts_p2p::build_lan_swarm_with_listeners(&[]).unwrap();
    discovery
        .dial(daemon_address.parse::<libp2p::Multiaddr>().unwrap())
        .unwrap();
    let query_id = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match discovery.select_next_some().await {
                SwarmEvent::Behaviour(ts_p2p::LanBehaviourEvent::Kad(
                    libp2p::kad::Event::RoutingUpdated { peer, .. },
                )) if peer == daemon_peer => {
                    break ts_p2p::find_tensor_providers(&mut discovery, &tensor_hash);
                }
                _ => {}
            }
        }
    })
    .await
    .expect("DHT routing did not become ready after transfer");
    let found = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let SwarmEvent::Behaviour(ts_p2p::LanBehaviourEvent::Kad(event)) =
                discovery.select_next_some().await
            {
                if let libp2p::kad::Event::OutboundQueryProgressed {
                    id,
                    result:
                        QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                            providers, ..
                        })),
                    ..
                } = event
                {
                    if id == query_id {
                        break providers.contains(&daemon_peer);
                    }
                }
            }
        }
    })
    .await
    .expect("DHT lookup for transferred tensor timed out");
    assert!(found, "transferred tensor provider was not discoverable");

    source_task.abort();
    drop(daemon);
}
