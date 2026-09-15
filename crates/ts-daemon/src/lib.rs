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
        }
    }

    fn with_manifest(mut self, manifest: ts_core::Manifest) -> Self {
        self.manifest = Arc::new(manifest);
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

pub struct RuntimeConfig {
    pub control_bind: String,
    pub proxy_bind: String,
    pub manifest_path: PathBuf,
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

pub fn router(state: DaemonState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
            let bytes = origin.as_bytes();
            bytes.starts_with(b"http://127.0.0.1")
                || bytes.starts_with(b"http://localhost")
                || bytes.starts_with(b"https://127.0.0.1")
                || bytes.starts_with(b"https://localhost")
                || bytes.starts_with(b"tauri://")
                || bytes.starts_with(b"http://tauri.localhost")
                || bytes.starts_with(b"https://tauri.localhost")
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

async fn models(State(state): State<DaemonState>) -> axum::Json<Vec<ModelPayload>> {
    axum::Json(
        state
            .manifest
            .files
            .iter()
            .map(|file| ModelPayload {
                path: file.path.clone(),
                format: file.format,
                file_size: file.file_size,
                tensors: state.manifest.tensors.len(),
                manifest_root: state.manifest.root,
            })
            .collect(),
    )
}

async fn verification(State(state): State<DaemonState>) -> axum::Json<VerificationPayload> {
    let store = ts_store::ObjectStore::open(&state.store_root);
    let (total_chunks, verified_chunks) = state
        .manifest
        .tensors
        .iter()
        .flat_map(|tensor| tensor.chunks.iter())
        .fold((0, 0), |(total, verified), chunk| {
            let is_verified = store.as_ref().is_ok_and(|store| store.contains(chunk.hash));
            (total + 1, verified + usize::from(is_verified))
        });
    axum::Json(VerificationPayload {
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
    let store = ts_store::ObjectStore::open(&state.store_root).map_err(internal_error)?;
    store
        .materialize(&state.manifest, &request.output)
        .map_err(internal_error)?;
    Ok(Json(PreparePayload {
        output: request.output.display().to_string(),
        complete: true,
    }))
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
    let store = ts_store::ObjectStore::open(&state.store_root).map_err(internal_error)?;
    store
        .put_verified(request.chunk.expected_hash, &bytes)
        .map_err(internal_error)?;
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
    let manifest: ts_core::Manifest = serde_cbor::from_slice(&fs::read(&config.manifest_path)?)?;
    anyhow::ensure!(manifest.verify_root(), "manifest self-check failed");
    let store = ts_store::ObjectStore::open(&config.store_root)?;
    let provider = ts_p2p::ChunkProvider::from_manifest(&store, &manifest)?;
    let mut swarm = ts_p2p::build_lan_swarm()?;
    ts_p2p::publish_manifest(&mut swarm, &manifest.root)?;
    for tensor in &manifest.tensors {
        ts_p2p::publish_tensor(&mut swarm, &tensor.tensor_hash)?;
    }
    let proxy_state = ts_proxy::ManifestProxyState {
        manifest,
        origin_url: ts_proxy::resolve_origin_url(&config.origin_url, "")?,
        engine: ts_proxy::FetchEngine::new(store),
        peer: None,
    };
    let peer_id = *swarm.local_peer_id();
    let state = DaemonState::new(&config.store_root, config.auth_token)
        .with_manifest(proxy_state.manifest.clone())
        .with_peer_id(peer_id)
        .with_settings(&config.control_bind, &config.proxy_bind, &config.origin_url)
        .running();
    let p2p_task = tokio::spawn(ts_p2p::run_lan_node_with_provider(swarm, provider));
    let proxy_bind = config.proxy_bind;
    let proxy_task =
        tokio::spawn(async move { ts_proxy::serve_manifest(&proxy_bind, proxy_state).await });
    let control_result = serve(&config.control_bind, state).await;
    p2p_task.abort();
    proxy_task.abort();
    control_result
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
    use tower::ServiceExt;

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
