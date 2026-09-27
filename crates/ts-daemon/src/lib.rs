//! Local control surface for the long-running TensorSwarm runtime.

use axum::extract::State;
use axum::http::{header, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct DaemonState {
    pub store_root: PathBuf,
    manifest: Arc<ts_core::Manifest>,
    peer_id: Arc<str>,
    control_bind: Arc<str>,
    proxy_bind: Arc<str>,
    origin_url: Arc<str>,
    auth_token: Arc<str>,
    p2p_status: Arc<str>,
    proxy_status: Arc<str>,
    metrics: Arc<RuntimeMetrics>,
    provider: ts_p2p::ChunkProvider,
    publish_tx: Option<tokio::sync::mpsc::Sender<ts_core::Hash32>>,
}

#[derive(Default)]
struct RuntimeMetrics {
    attempts: AtomicU64,
    successful_chunks: AtomicU64,
    failed_requests: AtomicU64,
    cancellations: AtomicU64,
    bytes_transferred: AtomicU64,
}

impl DaemonState {
    pub fn new(store_root: impl Into<PathBuf>, auth_token: impl Into<String>) -> Self {
        Self {
            store_root: store_root.into(),
            manifest: Arc::new(ts_core::Manifest::new(
                ts_core::ArtifactFormat::Safetensors,
                0,
                Vec::new(),
                Vec::new(),
            )),
            peer_id: Arc::from("not_started"),
            control_bind: Arc::from("not_started"),
            proxy_bind: Arc::from("not_started"),
            origin_url: Arc::from("not_configured"),
            auth_token: Arc::from(auth_token.into()),
            p2p_status: Arc::from("not_started"),
            proxy_status: Arc::from("not_started"),
            metrics: Arc::new(RuntimeMetrics::default()),
            provider: ts_p2p::ChunkProvider::default(),
            publish_tx: None,
        }
    }

    fn with_manifest(mut self, manifest: ts_core::Manifest) -> Self {
        self.manifest = Arc::new(manifest);
        self
    }

    fn with_provider_updates(
        mut self,
        provider: ts_p2p::ChunkProvider,
        publish_tx: tokio::sync::mpsc::Sender<ts_core::Hash32>,
    ) -> Self {
        self.provider = provider;
        self.publish_tx = Some(publish_tx);
        self
    }

    fn with_peer_id(mut self, peer_id: impl std::fmt::Display) -> Self {
        self.peer_id = Arc::from(peer_id.to_string());
        self
    }

    fn with_settings(
        mut self,
        control_bind: impl Into<String>,
        proxy_bind: impl Into<String>,
        origin_url: impl Into<String>,
    ) -> Self {
        self.control_bind = Arc::from(control_bind.into());
        self.proxy_bind = Arc::from(proxy_bind.into());
        self.origin_url = Arc::from(origin_url.into());
        self
    }

    fn running(mut self) -> Self {
        self.p2p_status = Arc::from("running");
        self.proxy_status = Arc::from("running");
        self
    }
}

#[derive(Clone)]
pub struct RuntimeConfig {
    pub control_bind: String,
    pub proxy_bind: String,
    pub manifest_path: Option<PathBuf>,
    pub store_root: PathBuf,
    pub origin_url: String,
    pub auth_token: String,
}

#[derive(Serialize)]
struct StatusPayload {
    service: &'static str,
    version: &'static str,
    store_root: String,
    p2p: String,
    proxy: String,
}

#[derive(Serialize)]
struct ModelPayload {
    path: String,
    format: ts_core::ArtifactFormat,
    file_size: u64,
    tensors: usize,
    manifest_root: ts_core::Hash32,
    total_chunks: usize,
    verified_chunks: usize,
    complete: bool,
}

#[derive(Serialize)]
struct VerificationPayload {
    manifest_root: ts_core::Hash32,
    total_chunks: usize,
    verified_chunks: usize,
    complete: bool,
}

#[derive(Deserialize)]
struct PrepareRequest {
    output: PathBuf,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Serialize)]
struct PreparePayload {
    output: String,
    complete: bool,
}

#[derive(Serialize)]
struct RepairPayload {
    removed: usize,
}

#[derive(Serialize)]
struct PeerPayload {
    peer_id: String,
    status: String,
}

#[derive(Serialize)]
struct MetricsPayload {
    attempts: u64,
    successful_chunks: u64,
    failed_requests: u64,
    cancellations: u64,
    bytes_transferred: u64,
}

#[derive(Serialize)]
struct SettingsPayload {
    control_bind: String,
    proxy_bind: String,
    origin_url: String,
}

#[derive(Deserialize)]
struct TransferRequest {
    peer_id: String,
    peer_address: String,
    chunk: ts_p2p::ChunkRequest,
    max_attempts: u8,
}

#[derive(Serialize)]
struct TransferPayload {
    chunk_index: u32,
    bytes: usize,
    verified: bool,
}

fn is_allowed_origin(origin_bytes: &[u8]) -> bool {
    let Ok(origin_str) = std::str::from_utf8(origin_bytes) else {
        return false;
    };
    let Ok(parsed) = url::Url::parse(origin_str) else {
        return false;
    };
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return false;
    }
    if parsed.path() != "" && parsed.path() != "/" {
        return false;
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return false;
    }
    match parsed.scheme() {
        "http" | "https" => {
            let Some(host) = parsed.host_str() else {
                return false;
            };
            host == "127.0.0.1" || host == "localhost" || host == "tauri.localhost"
        }
        "tauri" => {
            let Some(host) = parsed.host_str() else {
                return false;
            };
            host == "localhost" && parsed.port().is_none()
        }
        _ => false,
    }
}

pub fn router(state: DaemonState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
            is_allowed_origin(origin.as_bytes())
        }))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/status", get(status))
        .route("/v1/models", get(models))
        .route("/v1/verification", get(verification))
        .route("/v1/prepare", axum::routing::post(prepare))
        .route("/v1/cache/repair", axum::routing::post(repair))
        .route("/v1/peers", get(peers))
        .route("/v1/metrics", get(metrics))
        .route("/v1/settings", get(settings))
        .route("/v1/transfers", axum::routing::post(transfer))
        .layer(middleware::from_fn_with_state(state.clone(), authenticate))
        .layer(cors)
        .with_state(state)
}

pub async fn serve(bind: &str, state: DaemonState) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    axum::serve(listener, router(state)).await?;
    Ok(())
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok\n")
}

async fn status(State(state): State<DaemonState>) -> axum::Json<StatusPayload> {
    axum::Json(StatusPayload {
        service: "ts-daemon",
        version: env!("CARGO_PKG_VERSION"),
        store_root: state.store_root.display().to_string(),
        p2p: state.p2p_status.to_string(),
        proxy: state.proxy_status.to_string(),
    })
}

async fn models(
    State(state): State<DaemonState>,
) -> Result<Json<Vec<ModelPayload>>, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || model_payloads(&state))
        .await
        .map_err(internal_error)?
        .map(Json)
}

fn file_manifest(
    manifest: &ts_core::Manifest,
    file: &ts_core::FileRecipe,
) -> Result<ts_core::Manifest, (StatusCode, String)> {
    let referenced = file
        .segments
        .iter()
        .filter_map(|segment| match segment {
            ts_core::Segment::Tensor { tensor_hash, .. } => Some(*tensor_hash),
            _ => None,
        })
        .collect::<std::collections::HashSet<_>>();
    let tensors = manifest
        .tensors
        .iter()
        .filter(|tensor| referenced.contains(&tensor.tensor_hash))
        .cloned()
        .collect::<Vec<_>>();
    if referenced
        .iter()
        .any(|hash| !tensors.iter().any(|tensor| tensor.tensor_hash == *hash))
    {
        return Err(internal_error("model recipe refers to an unknown tensor"));
    }
    Ok(ts_core::Manifest::new(
        file.format,
        file.file_size,
        tensors,
        vec![file.clone()],
    ))
}

fn model_payloads(state: &DaemonState) -> Result<Vec<ModelPayload>, (StatusCode, String)> {
    let store = ts_store::ObjectStore::open(&state.store_root).map_err(internal_error)?;
    let mut models = Vec::with_capacity(state.manifest.files.len());
    for file in &state.manifest.files {
        let model_manifest = file_manifest(&state.manifest, file)?;
        let total_chunks = model_manifest.tensors.iter().map(|t| t.chunks.len()).sum();
        let verified_chunks = model_manifest
            .tensors
            .iter()
            .flat_map(|tensor| &tensor.chunks)
            .filter(|chunk| store.get(chunk.hash).is_ok())
            .count();
        models.push(ModelPayload {
            path: file.path.clone(),
            format: file.format,
            file_size: file.file_size,
            tensors: model_manifest.tensors.len(),
            manifest_root: model_manifest.root,
            total_chunks,
            verified_chunks,
            complete: total_chunks == verified_chunks,
        });
    }
    Ok(models)
}

async fn verification(
    State(state): State<DaemonState>,
) -> Result<Json<VerificationPayload>, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || verification_payload(&state))
        .await
        .map_err(internal_error)?
        .map(Json)
}

fn verification_payload(state: &DaemonState) -> Result<VerificationPayload, (StatusCode, String)> {
    let store = ts_store::ObjectStore::open(&state.store_root).map_err(internal_error)?;
    let (total_chunks, verified_chunks) = state
        .manifest
        .tensors
        .iter()
        .flat_map(|tensor| tensor.chunks.iter())
        .fold((0, 0), |(total, verified), chunk| {
            let is_verified = store.get(chunk.hash).is_ok();
            (total + 1, verified + usize::from(is_verified))
        });
    Ok(VerificationPayload {
        manifest_root: state.manifest.root,
        total_chunks,
        verified_chunks,
        complete: total_chunks == verified_chunks,
    })
}

async fn prepare(
    State(state): State<DaemonState>,
    Json(request): Json<PrepareRequest>,
) -> Result<Json<PreparePayload>, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        let recipe = match request.path.as_deref() {
            Some(path) => state
                .manifest
                .files
                .iter()
                .find(|file| file.path == path)
                .ok_or_else(|| invalid_request("model path is not in the active catalog"))?,
            None if state.manifest.files.len() == 1 => &state.manifest.files[0],
            None => {
                return Err(invalid_request(
                    "model path is required for a multi-file catalog",
                ))
            }
        };
        let manifest = file_manifest(&state.manifest, recipe)?;
        let store = ts_store::ObjectStore::open(&state.store_root).map_err(internal_error)?;
        store
            .materialize(&manifest, &request.output)
            .map_err(internal_error)?;
        Ok(Json(PreparePayload {
            output: request.output.display().to_string(),
            complete: true,
        }))
    })
    .await
    .map_err(internal_error)?
}

async fn repair(
    State(state): State<DaemonState>,
) -> Result<Json<RepairPayload>, (StatusCode, String)> {
    let store = ts_store::ObjectStore::open(&state.store_root).map_err(internal_error)?;
    let live = state
        .manifest
        .tensors
        .iter()
        .flat_map(|tensor| tensor.chunks.iter().map(|chunk| chunk.hash));
    let removed = store
        .remove_unreferenced_objects(live)
        .map_err(internal_error)?;
    Ok(Json(RepairPayload {
        removed: removed.len(),
    }))
}

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

async fn peers(State(state): State<DaemonState>) -> Json<Vec<PeerPayload>> {
    Json(vec![PeerPayload {
        peer_id: state.peer_id.to_string(),
        status: if state.peer_id.as_ref() == "not_started" {
            "not_started".into()
        } else {
            "running".into()
        },
    }])
}

async fn metrics(State(state): State<DaemonState>) -> Json<MetricsPayload> {
    let metrics = &state.metrics;
    Json(MetricsPayload {
        attempts: metrics.attempts.load(Ordering::Relaxed),
        successful_chunks: metrics.successful_chunks.load(Ordering::Relaxed),
        failed_requests: metrics.failed_requests.load(Ordering::Relaxed),
        cancellations: metrics.cancellations.load(Ordering::Relaxed),
        bytes_transferred: metrics.bytes_transferred.load(Ordering::Relaxed),
    })
}

async fn settings(State(state): State<DaemonState>) -> Json<SettingsPayload> {
    Json(SettingsPayload {
        control_bind: state.control_bind.to_string(),
        proxy_bind: state.proxy_bind.to_string(),
        origin_url: state.origin_url.to_string(),
    })
}

async fn transfer(
    State(state): State<DaemonState>,
    Json(request): Json<TransferRequest>,
) -> Result<Json<TransferPayload>, (StatusCode, String)> {
    state.metrics.attempts.fetch_add(1, Ordering::Relaxed);
    let peer = request
        .peer_id
        .parse::<libp2p::PeerId>()
        .map_err(invalid_request)?;
    let address = request
        .peer_address
        .parse::<libp2p::Multiaddr>()
        .map_err(invalid_request)?;
    if request.max_attempts == 0 {
        return Err(invalid_request("max_attempts must be positive"));
    }
    let Some(manifest_chunk) = state
        .manifest
        .tensors
        .iter()
        .find(|tensor| tensor.tensor_hash == request.chunk.tensor_hash)
        .and_then(|tensor| {
            tensor
                .chunks
                .iter()
                .find(|chunk| chunk.index == request.chunk.chunk_index)
        })
    else {
        return Err(invalid_request(
            "chunk is not referenced by the active manifest",
        ));
    };
    if manifest_chunk.hash != request.chunk.expected_hash {
        return Err(invalid_request(
            "chunk hash does not match the active manifest",
        ));
    }
    let mut swarm = ts_p2p::build_lan_swarm().map_err(internal_error)?;
    swarm.dial(address).map_err(invalid_request)?;
    let mut client = ts_p2p::LanClient::new(swarm);
    let bytes = client
        .fetch_chunk(peer, request.chunk.clone(), request.max_attempts)
        .await
        .map_err(|error| {
            state
                .metrics
                .failed_requests
                .fetch_add(1, Ordering::Relaxed);
            internal_error(error)
        })?;
    if bytes.len() as u64 != manifest_chunk.length {
        return Err(internal_error(
            "received chunk length does not match the active manifest",
        ));
    }
    let store = ts_store::ObjectStore::open(&state.store_root).map_err(internal_error)?;
    store
        .put_verified(request.chunk.expected_hash, &bytes)
        .map_err(internal_error)?;
    state
        .provider
        .insert_manifest_chunk(
            &state.manifest,
            request.chunk.tensor_hash,
            request.chunk.chunk_index,
            request.chunk.expected_hash,
            bytes.clone(),
        )
        .map_err(|error| internal_error(format!("manifest chunk validation failed: {error:?}")))?;
    if let Some(publish_tx) = &state.publish_tx {
        publish_tx
            .send(request.chunk.tensor_hash)
            .await
            .map_err(|_| internal_error("P2P provider publication queue is closed"))?;
    }
    state
        .metrics
        .successful_chunks
        .fetch_add(1, Ordering::Relaxed);
    state
        .metrics
        .bytes_transferred
        .fetch_add(bytes.len() as u64, Ordering::Relaxed);
    Ok(Json(TransferPayload {
        chunk_index: request.chunk.chunk_index,
        bytes: bytes.len(),
        verified: true,
    }))
}

fn invalid_request(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, error.to_string())
}

/// Start the control API and the two long-running core-owned services.
///
/// The daemon validates the manifest and store before binding the control API,
/// so a bad runtime configuration cannot present a healthy control surface.
pub async fn serve_runtime(config: RuntimeConfig) -> anyhow::Result<()> {
    serve_runtime_with_webseed(config, ts_proxy::WebSeeder::default()).await
}

async fn serve_runtime_with_webseed(
    config: RuntimeConfig,
    webseed: ts_proxy::WebSeeder,
) -> anyhow::Result<()> {
    let swarm = ts_p2p::build_lan_swarm()?;
    serve_runtime_with_swarm(config, webseed, swarm, None).await
}

async fn serve_runtime_with_swarm(
    config: RuntimeConfig,
    webseed: ts_proxy::WebSeeder,
    mut swarm: ts_p2p::LanSwarm,
    mut listen_tx: Option<tokio::sync::oneshot::Sender<libp2p::Multiaddr>>,
) -> anyhow::Result<()> {
    let manifest: ts_core::Manifest = match &config.manifest_path {
        Some(path) => serde_cbor::from_slice(&fs::read(path)?)?,
        None => ts_core::Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            0,
            Vec::new(),
            Vec::new(),
        ),
    };
    anyhow::ensure!(manifest.verify_root(), "manifest self-check failed");
    let store = ts_store::ObjectStore::open(&config.store_root)?;
    let provider = ts_p2p::ChunkProvider::from_manifest(&store, &manifest)?;
    let (publish_tx, mut publish_rx) = tokio::sync::mpsc::channel(64);
    let manifest = Arc::new(manifest);
    ts_p2p::publish_manifest(&mut swarm, &manifest.root)?;
    for tensor in &manifest.tensors {
        if provider.has_verified_chunks_for_tensor(&tensor.tensor_hash) {
            ts_p2p::publish_tensor(&mut swarm, &tensor.tensor_hash)?;
        }
    }
    let proxy_state = ts_proxy::ManifestProxyState {
        manifest: manifest.as_ref().clone(),
        origin_url: ts_proxy::resolve_origin_url(&config.origin_url, "")?,
        engine: ts_proxy::FetchEngine {
            store,
            webseed,
            verified_chunk_notify: Some(runtime_verified_chunk_notify(
                manifest.clone(),
                provider.clone(),
                publish_tx.clone(),
            )),
        },
        peer: None,
    };
    let peer_id = *swarm.local_peer_id();
    let state = DaemonState::new(&config.store_root, config.auth_token)
        .with_manifest(proxy_state.manifest.clone())
        .with_provider_updates(provider.clone(), publish_tx)
        .with_peer_id(peer_id)
        .with_settings(&config.control_bind, &config.proxy_bind, &config.origin_url)
        .running();
    let mut service_tasks = tokio::task::JoinSet::new();
    service_tasks.spawn(async move {
        ts_p2p::run_lan_node_with_provider_and_events(
            &mut swarm,
            provider,
            &mut listen_tx,
            Some(&mut publish_rx),
        )
        .await
    });
    let proxy_bind = config.proxy_bind;
    service_tasks.spawn(async move { ts_proxy::serve_manifest(&proxy_bind, proxy_state).await });
    let control_result = serve(&config.control_bind, state).await;
    service_tasks.abort_all();
    while service_tasks.join_next().await.is_some() {}
    control_result
}

fn runtime_verified_chunk_notify(
    manifest: Arc<ts_core::Manifest>,
    provider: ts_p2p::ChunkProvider,
    publish_tx: tokio::sync::mpsc::Sender<ts_core::Hash32>,
) -> ts_proxy::VerifiedChunkNotify {
    Arc::new(move |update| {
        let tensor_hash = update.tensor_hash;
        if provider
            .insert_manifest_chunk(
                &manifest,
                tensor_hash,
                update.chunk_index,
                update.chunk_hash,
                update.bytes,
            )
            .is_ok()
        {
            let publish_tx = publish_tx.clone();
            tokio::spawn(async move {
                let _ = publish_tx.send(tensor_hash).await;
            });
        }
    })
}

async fn authenticate(
    State(state): State<DaemonState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if request.method() == Method::OPTIONS || request.uri().path() == "/healthz" {
        return next.run(request).await;
    }
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| token.as_bytes().ct_eq(state.auth_token.as_bytes()));
    if authorized {
        next.run(request).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

trait ConstantTimeEq {
    fn ct_eq(&self, other: &[u8]) -> bool;
}

impl ConstantTimeEq for [u8] {
    fn ct_eq(&self, other: &[u8]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter()
            .zip(other)
            .fold(0_u8, |difference, (left, right)| {
                difference | (left ^ right)
            })
            == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use std::net::TcpListener as StdTcpListener;
    use std::time::Duration;
    use tower::ServiceExt;
    use ts_core::{sha256, ChunkRef, FileRecipe, Segment, TensorDescriptor, TensorNode};

    #[tokio::test]
    async fn model_inventory_reports_each_files_own_root_and_availability() {
        let directory = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(directory.path()).unwrap();
        let mut individual = Vec::new();
        for (name, payload) in [("first", b"one".as_slice()), ("second", b"two".as_slice())] {
            let hash = sha256(payload);
            let tensor = TensorNode {
                descriptor: TensorDescriptor {
                    name: name.into(),
                    shape: vec![3],
                    dtype: "U8".into(),
                    byte_len: 3,
                },
                tensor_hash: hash,
                chunks: vec![ChunkRef {
                    index: 0,
                    offset: 0,
                    length: 3,
                    hash,
                }],
            };
            let recipe = FileRecipe {
                path: format!("{name}.safetensors"),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size: 3,
                segments: vec![Segment::Tensor {
                    offset: 0,
                    tensor_hash: hash,
                    tensor_offset: 0,
                    length: 3,
                }],
            };
            individual.push(ts_core::Manifest::new(
                ts_core::ArtifactFormat::Safetensors,
                3,
                vec![tensor],
                vec![recipe],
            ));
        }
        store.put_verified(sha256(b"one"), b"one").unwrap();
        let combined = ts_core::Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            6,
            individual.iter().flat_map(|m| m.tensors.clone()).collect(),
            individual.iter().flat_map(|m| m.files.clone()).collect(),
        );
        let app = router(DaemonState::new(directory.path(), "test-token").with_manifest(combined));
        let response = app
            .clone()
            .oneshot(
                Request::get("/v1/models")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 16_384).await.unwrap();
        let models: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(models.as_array().unwrap().len(), 2);
        for (index, expected) in individual.iter().enumerate() {
            assert_eq!(
                models[index]["manifest_root"],
                serde_json::to_value(expected.root).unwrap()
            );
            assert_eq!(models[index]["tensors"], 1);
            assert_eq!(models[index]["total_chunks"], 1);
            assert_eq!(
                models[index]["verified_chunks"],
                if index == 0 { 1 } else { 0 }
            );
            assert_eq!(models[index]["complete"], index == 0);
        }
        std::fs::write(store.object_path(sha256(b"one")), b"bad").unwrap();
        let response = app
            .oneshot(
                Request::get("/v1/models")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 16_384).await.unwrap();
        let models: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(models[0]["verified_chunks"], 0);
        assert_eq!(models[0]["complete"], false);
        store.put_verified(sha256(b"two"), b"two").unwrap();
        let output = directory.path().join("selected.safetensors");
        let app = router(
            DaemonState::new(directory.path(), "test-token").with_manifest(ts_core::Manifest::new(
                ts_core::ArtifactFormat::Safetensors,
                6,
                individual.iter().flat_map(|m| m.tensors.clone()).collect(),
                individual.iter().flat_map(|m| m.files.clone()).collect(),
            )),
        );
        for path in [
            None,
            Some("missing.safetensors"),
            Some("../first.safetensors"),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::post("/v1/prepare")
                        .header(header::AUTHORIZATION, "Bearer test-token")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            serde_json::json!({"output": output, "path": path}).to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            assert!(!output.exists());
        }
        let response = app
            .oneshot(
                Request::post("/v1/prepare")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::json!({"output": output, "path": "second.safetensors"})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(std::fs::read(output).unwrap(), b"two");
    }

    fn free_address() -> String {
        let listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().to_string()
    }

    async fn wait_for_health(url: String) {
        let client = reqwest::Client::new();
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if client
                    .get(&url)
                    .send()
                    .await
                    .is_ok_and(|response| response.status().is_success())
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("runtime health endpoint did not start");
    }

    async fn wait_for_bind(address: String) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let Ok(listener) = tokio::net::TcpListener::bind(&address).await {
                    drop(listener);
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("runtime listener remained bound after cancellation");
    }

    #[tokio::test]
    async fn health_is_public_and_status_requires_local_token() {
        let app = router(DaemonState::new("cache", "test-token"));
        let health = app
            .clone()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::OK);

        let unauthorized = app
            .clone()
            .oneshot(Request::get("/v1/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let authorized = app
            .oneshot(
                Request::get("/v1/status")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(authorized.status(), StatusCode::OK);
        let body = to_bytes(authorized.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8(body.to_vec())
            .unwrap()
            .contains("\"service\":\"ts-daemon\""));
    }

    #[tokio::test]
    async fn proxy_verified_fetch_refreshes_runtime_provider_before_announcement() {
        let payload = b"proxy-provider-refresh".to_vec();
        let chunk_hash = sha256(&payload);
        let tensor_hash = sha256(b"proxy-tensor");
        let expected_len = payload.len() as u64;
        let manifest = Arc::new(ts_core::Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            expected_len,
            vec![TensorNode {
                descriptor: TensorDescriptor {
                    name: "weight".into(),
                    shape: vec![expected_len],
                    dtype: "U8".into(),
                    byte_len: expected_len,
                },
                tensor_hash,
                chunks: vec![ChunkRef {
                    index: 0,
                    offset: 0,
                    length: expected_len,
                    hash: chunk_hash,
                }],
            }],
            vec![FileRecipe {
                path: "model.safetensors".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size: expected_len,
                segments: vec![Segment::Tensor {
                    offset: 0,
                    tensor_hash,
                    tensor_offset: 0,
                    length: expected_len,
                }],
            }],
        ));
        let directory = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(directory.path()).unwrap();
        let provider = ts_p2p::ChunkProvider::from_manifest(&store, &manifest).unwrap();
        let (publish_tx, mut publish_rx) = tokio::sync::mpsc::channel(1);
        let notify = runtime_verified_chunk_notify(manifest.clone(), provider.clone(), publish_tx);
        notify(ts_proxy::VerifiedChunkUpdate {
            tensor_hash,
            chunk_index: 0,
            chunk_hash: sha256(b"wrong manifest chunk"),
            bytes: b"wrong manifest chunk".to_vec(),
        });
        assert!(!provider.has_verified_chunks_for_tensor(&tensor_hash));
        assert!(
            tokio::time::timeout(Duration::from_millis(10), publish_rx.recv())
                .await
                .is_err()
        );

        let engine = ts_proxy::FetchEngine::new(store).with_verified_chunk_notify(notify);
        let peer_payload = payload.clone();
        let peer: ts_proxy::PeerFetch =
            Arc::new(move |requested_tensor, index, range, expected| {
                assert_eq!(requested_tensor, tensor_hash);
                assert_eq!(index, 0);
                assert_eq!(range, 0..expected_len);
                assert_eq!(expected, chunk_hash);
                let bytes = peer_payload.clone();
                Box::pin(async move { Ok(bytes) })
            });

        let result = engine
            .fetch_manifest_range(
                &manifest,
                "https://invalid.example/model.safetensors",
                0..expected_len,
                Some(peer),
            )
            .await
            .unwrap();
        let response = provider.respond(ts_p2p::PeerRequest::GetChunks {
            tensor_hash,
            chunks: vec![ts_p2p::ChunkRequest {
                request_id: 1,
                tensor_hash,
                chunk_index: 0,
                expected_hash: chunk_hash,
            }],
        });
        let announced = tokio::time::timeout(Duration::from_secs(1), publish_rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result, payload);
        assert!(matches!(
            response,
            ts_p2p::PeerResponse::Chunk { payload: served, .. } if served == payload
        ));
        assert_eq!(announced, tensor_hash);
    }

    #[tokio::test]
    async fn running_state_reports_owned_core_services() {
        let app = router(DaemonState::new("cache", "test-token").running());
        let response = app
            .oneshot(
                Request::get("/v1/status")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("\"p2p\":\"running\""));
        assert!(body.contains("\"proxy\":\"running\""));
    }

    #[tokio::test]
    async fn cancelling_runtime_releases_control_and_proxy_listeners() {
        let directory = tempfile::tempdir().unwrap();
        let control_bind = free_address();
        let proxy_bind = free_address();
        let control_url = format!("http://{control_bind}/healthz");
        let proxy_url = format!("http://{proxy_bind}/healthz");
        let task = tokio::spawn(serve_runtime(RuntimeConfig {
            control_bind: control_bind.clone(),
            proxy_bind: proxy_bind.clone(),
            manifest_path: None,
            store_root: directory.path().join("cache"),
            origin_url: "https://localhost/".into(),
            auth_token: "runtime-cancel-token".into(),
        }));

        wait_for_health(control_url).await;
        wait_for_health(proxy_url).await;
        task.abort();
        let _ = task.await;
        wait_for_bind(control_bind).await;
        wait_for_bind(proxy_bind).await;
    }

    #[tokio::test]
    async fn runtime_manifest_proxy_fetches_verified_https_bytes() {
        use axum::routing::get;
        use axum::Router;
        use axum_server::tls_rustls::RustlsConfig;
        use futures::StreamExt;
        use libp2p::kad::{GetProvidersOk, QueryResult};
        use libp2p::swarm::SwarmEvent;
        use rcgen::generate_simple_self_signed;
        use std::net::SocketAddr;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let _ = rustls::crypto::ring::default_provider().install_default();
        let payload = b"runtime-https-proxy".to_vec();
        let chunk_hash = sha256(&payload);
        let tensor_hash = sha256(b"runtime-proxy-tensor");
        let file_size = payload.len() as u64;
        let manifest = ts_core::Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            file_size,
            vec![TensorNode {
                descriptor: TensorDescriptor {
                    name: "weight".into(),
                    shape: vec![file_size],
                    dtype: "U8".into(),
                    byte_len: file_size,
                },
                tensor_hash,
                chunks: vec![ChunkRef {
                    index: 0,
                    offset: 0,
                    length: file_size,
                    hash: chunk_hash,
                }],
            }],
            vec![FileRecipe {
                path: "model.safetensors".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size,
                segments: vec![Segment::Tensor {
                    offset: 0,
                    tensor_hash,
                    tensor_offset: 0,
                    length: file_size,
                }],
            }],
        );
        let directory = tempfile::tempdir().unwrap();
        let manifest_path = directory.path().join("manifest.cbor");
        fs::write(&manifest_path, serde_cbor::to_vec(&manifest).unwrap()).unwrap();

        let origin_requests = Arc::new(AtomicUsize::new(0));
        let origin_app = Router::new().route(
            "/model.safetensors",
            get({
                let payload = payload.clone();
                let origin_requests = origin_requests.clone();
                move || {
                    origin_requests.fetch_add(1, Ordering::Relaxed);
                    let payload = payload.clone();
                    async move { (StatusCode::PARTIAL_CONTENT, payload) }
                }
            }),
        );
        let cert = generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.cert.der().to_vec();
        let origin_client = reqwest::Client::builder()
            .add_root_certificate(reqwest::Certificate::from_der(&cert_der).unwrap())
            .build()
            .unwrap();
        let tls = RustlsConfig::from_der(vec![cert_der.clone()], cert.key_pair.serialize_der())
            .await
            .unwrap();
        let origin_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin_address: SocketAddr = origin_listener.local_addr().unwrap();
        drop(origin_listener);
        let origin_task = tokio::spawn(
            axum_server::bind_rustls(origin_address, tls).serve(origin_app.into_make_service()),
        );
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if tokio::net::TcpStream::connect(origin_address).await.is_ok() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("HTTPS origin did not start");

        let control_bind = free_address();
        let proxy_bind = free_address();
        let control_url = format!("http://{control_bind}/v1/verification");
        let proxy_url = format!("http://{proxy_bind}/file/model.safetensors");
        let auth_token = "runtime-https-token";
        let runtime_swarm =
            ts_p2p::build_lan_swarm_with_listeners(&["/ip4/127.0.0.1/tcp/0"]).unwrap();
        let runtime_peer = *runtime_swarm.local_peer_id();
        let (listen_tx, listen_rx) = tokio::sync::oneshot::channel();
        let runtime_task = tokio::spawn(serve_runtime_with_swarm(
            RuntimeConfig {
                control_bind: control_bind.clone(),
                proxy_bind: proxy_bind.clone(),
                manifest_path: Some(manifest_path),
                store_root: directory.path().join("cache"),
                origin_url: format!(
                    "https://localhost:{}/model.safetensors",
                    origin_address.port()
                ),
                auth_token: auth_token.into(),
            },
            ts_proxy::WebSeeder::with_client(origin_client),
            runtime_swarm,
            Some(listen_tx),
        ));
        wait_for_health(format!("http://{control_bind}/healthz")).await;
        wait_for_health(format!("http://{proxy_bind}/healthz")).await;

        let runtime_address = listen_rx
            .await
            .unwrap()
            .with(libp2p::multiaddr::Protocol::P2p(runtime_peer.into()));
        let mut third_peer =
            ts_p2p::LanClient::new(ts_p2p::build_lan_swarm_with_listeners(&[]).unwrap());
        third_peer.swarm_mut().dial(runtime_address).unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let SwarmEvent::Behaviour(ts_p2p::LanBehaviourEvent::Kad(
                    libp2p::kad::Event::RoutingUpdated { peer, .. },
                )) = third_peer.swarm_mut().select_next_some().await
                {
                    if peer == runtime_peer {
                        break;
                    }
                }
            }
        })
        .await
        .expect("third peer did not add the daemon to its DHT routing table");

        let client = reqwest::Client::new();
        let response = client.get(&proxy_url).send().await.unwrap();
        let status = response.status();
        let response_body = response.bytes().await.unwrap();
        assert_eq!(
            status,
            StatusCode::OK,
            "proxy returned {:?}; upstream requests: {}",
            response_body,
            origin_requests.load(Ordering::Relaxed)
        );
        assert_eq!(response_body.as_ref(), payload);
        let verification: serde_json::Value = client
            .get(&control_url)
            .header(header::AUTHORIZATION, format!("Bearer {auth_token}"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(verification["verified_chunks"], 1);
        assert_eq!(origin_requests.load(Ordering::Relaxed), 1);

        let query_id = ts_p2p::find_tensor_providers(third_peer.swarm_mut(), &tensor_hash);
        let providers = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let SwarmEvent::Behaviour(ts_p2p::LanBehaviourEvent::Kad(
                    libp2p::kad::Event::OutboundQueryProgressed {
                        id,
                        result:
                            QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                                providers,
                                ..
                            })),
                        ..
                    },
                )) = third_peer.swarm_mut().select_next_some().await
                {
                    if id == query_id {
                        return providers;
                    }
                }
            }
        })
        .await
        .expect("third peer did not receive the tensor provider record");
        assert!(providers.contains(&runtime_peer));
        let fetched = third_peer
            .fetch_chunk(
                runtime_peer,
                ts_p2p::ChunkRequest {
                    request_id: 404,
                    tensor_hash,
                    chunk_index: 0,
                    expected_hash: chunk_hash,
                },
                1,
            )
            .await
            .unwrap();
        assert_eq!(fetched, payload);

        let partial = client
            .get(&proxy_url)
            .header(header::RANGE, "bytes=4-7")
            .send()
            .await
            .unwrap();
        assert_eq!(partial.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            partial
                .headers()
                .get(header::CONTENT_RANGE)
                .and_then(|value| value.to_str().ok()),
            Some(format!("bytes 4-7/{file_size}").as_str())
        );
        assert_eq!(partial.bytes().await.unwrap().as_ref(), &payload[4..8]);
        assert_eq!(origin_requests.load(Ordering::Relaxed), 1);

        runtime_task.abort();
        let _ = runtime_task.await;
        wait_for_bind(control_bind).await;
        wait_for_bind(proxy_bind).await;
        origin_task.abort();
        let _ = origin_task.await;
        wait_for_bind(origin_address.to_string()).await;
    }

    #[test]
    fn allows_valid_local_origins_and_rejects_lookalikes() {
        assert!(is_allowed_origin(b"http://127.0.0.1:8080"));
        assert!(is_allowed_origin(b"http://localhost:3000"));
        assert!(is_allowed_origin(b"https://127.0.0.1"));
        assert!(is_allowed_origin(b"https://localhost"));
        assert!(is_allowed_origin(b"tauri://localhost"));
        assert!(is_allowed_origin(b"http://tauri.localhost"));
        assert!(is_allowed_origin(b"https://tauri.localhost"));

        // Hostile lookalike origins MUST be rejected
        assert!(!is_allowed_origin(b"http://localhost.evil.example"));
        assert!(!is_allowed_origin(b"http://127.0.0.1.evil.example"));
        assert!(!is_allowed_origin(b"http://tauri.localhost.evil.example"));
        assert!(!is_allowed_origin(b"http://localhost@evil.example"));
        assert!(!is_allowed_origin(b"http://127.0.0.1@evil.example"));
        assert!(!is_allowed_origin(b"http://user:pass@localhost"));
        assert!(!is_allowed_origin(b"tauri://evil.example"));
        assert!(!is_allowed_origin(b"http://evil.example/127.0.0.1"));
        assert!(!is_allowed_origin(b"http://localhost/path"));
        assert!(!is_allowed_origin(b"http://localhost?query"));
        assert!(!is_allowed_origin(b"not-a-url"));
    }

    #[tokio::test]
    async fn desktop_origin_can_negotiate_authenticated_control_requests() {
        let app = router(DaemonState::new("cache", "test-token"));
        let preflight = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/v1/status")
                    .header(header::ORIGIN, "tauri://localhost")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(preflight.status(), StatusCode::OK);
        assert_eq!(
            preflight
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok()),
            Some("tauri://localhost")
        );

        let response = app
            .oneshot(
                Request::get("/v1/status")
                    .header(header::ORIGIN, "tauri://localhost")
                    .header(header::AUTHORIZATION, "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok()),
            Some("tauri://localhost")
        );
    }
}
