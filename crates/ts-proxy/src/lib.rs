//! Local HTTP range planning and file-serving primitives.

use anyhow::Context;
use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use reqwest::Client;
use std::future::Future;
use std::ops::Range;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio::net::TcpListener;
use tokio_util::io::ReaderStream;
use ts_core::{sha256, ChunkRef, Hash32, Manifest, Segment};

pub const DEFAULT_BIND: &str = "127.0.0.1:9090";

#[derive(Clone, Debug)]
pub struct WebSeeder {
    client: Client,
}

impl Default for WebSeeder {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl WebSeeder {
    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_verified(
        &self,
        url: &str,
        range: Range<u64>,
        expected: Hash32,
    ) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(range.start < range.end, "invalid WebSeed range");
        let header = format!("bytes={}-{}", range.start, range.end - 1);
        let response = self
            .client
            .get(url)
            .header(reqwest::header::RANGE, header)
            .send()
            .await?;
        anyhow::ensure!(
            response.status().is_success(),
            "WebSeed returned {}",
            response.status()
        );
        let bytes = response.bytes().await?.to_vec();
        anyhow::ensure!(
            bytes.len() as u64 == range.end - range.start,
            "WebSeed returned an unexpected length"
        );
        let actual = sha256(&bytes);
        anyhow::ensure!(
            actual == expected,
            "WebSeed hash mismatch: expected {expected:?}, got {actual:?}"
        );
        Ok(bytes)
    }

    pub async fn fetch_verified_into_store(
        &self,
        url: &str,
        range: Range<u64>,
        expected: Hash32,
        store: &ts_store::ObjectStore,
    ) -> anyhow::Result<Hash32> {
        let bytes = self.fetch_verified(url, range, expected).await?;
        store.put_verified(expected, &bytes)?;
        Ok(expected)
    }
}

pub type PeerFetch = Arc<
    dyn Fn(
            Hash32,
            u32,
            Range<u64>,
            Hash32,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>> + Send>>
        + Send
        + Sync,
>;

pub fn lan_peer_fetch(
    client: Arc<tokio::sync::Mutex<ts_p2p::LanClient>>,
    peer: libp2p::PeerId,
    max_attempts: u8,
) -> PeerFetch {
    Arc::new(move |tensor_hash, chunk_index, _range, expected_hash| {
        let client = Arc::clone(&client);
        Box::pin(async move {
            let mut client = client.lock().await;
            client
                .fetch_chunk(
                    peer,
                    ts_p2p::ChunkRequest {
                        request_id: 0,
                        tensor_hash,
                        chunk_index,
                        expected_hash,
                    },
                    max_attempts,
                )
                .await
        })
    })
}

#[derive(Clone, Debug)]
pub struct VerifiedChunkUpdate {
    pub tensor_hash: Hash32,
    pub chunk_index: u32,
    pub chunk_hash: Hash32,
    pub bytes: Vec<u8>,
}

pub type VerifiedChunkNotify = Arc<dyn Fn(VerifiedChunkUpdate) + Send + Sync>;

#[derive(Clone)]
pub struct FetchEngine {
    pub store: ts_store::ObjectStore,
    pub webseed: WebSeeder,
    pub verified_chunk_notify: Option<VerifiedChunkNotify>,
}

impl FetchEngine {
    pub fn new(store: ts_store::ObjectStore) -> Self {
        Self {
            store,
            webseed: WebSeeder::default(),
            verified_chunk_notify: None,
        }
    }

    pub fn with_verified_chunk_notify(mut self, notify: VerifiedChunkNotify) -> Self {
        self.verified_chunk_notify = Some(notify);
        self
    }

    pub fn with_verified_chunk_sender(self, sender: tokio::sync::mpsc::Sender<Hash32>) -> Self {
        self.with_verified_chunk_notify(Arc::new(move |update| {
            let _ = sender.try_send(update.chunk_hash);
        }))
    }

    /// Resolve one manifest chunk using CAS, then swarm, then HTTPS WebSeed.
    /// Only bytes whose expected hash matches are persisted.
    pub async fn fetch_chunk(
        &self,
        url: &str,
        range: Range<u64>,
        expected: Hash32,
        peer: Option<PeerFetch>,
    ) -> anyhow::Result<Vec<u8>> {
        self.fetch_chunk_with_identity(url, range, expected, expected, 0, peer)
            .await
    }

    async fn fetch_chunk_with_identity(
        &self,
        url: &str,
        range: Range<u64>,
        expected: Hash32,
        tensor_hash: Hash32,
        chunk_index: u32,
        peer: Option<PeerFetch>,
    ) -> anyhow::Result<Vec<u8>> {
        if self.store.contains(expected) {
            return Ok(self.store.get(expected)?);
        }
        let expected_len = range.end.checked_sub(range.start).unwrap_or(0);
        if expected_len == 0 {
            anyhow::bail!("empty chunk range");
        }
        let mut peer_error = None;
        if let Some(fetch) = peer {
            match fetch(tensor_hash, chunk_index, range.clone(), expected).await {
                Ok(bytes) if bytes.len() as u64 == expected_len && sha256(&bytes) == expected => {
                    self.store.put_verified(expected, &bytes)?;
                    if let Some(notify) = &self.verified_chunk_notify {
                        notify(VerifiedChunkUpdate {
                            tensor_hash,
                            chunk_index,
                            chunk_hash: expected,
                            bytes: bytes.clone(),
                        });
                    }
                    return Ok(bytes);
                }
                Ok(_) => peer_error = Some(String::from("peer returned invalid chunk bytes")),
                Err(error) => peer_error = Some(error.to_string()),
            }
        }
        match self
            .webseed
            .fetch_verified_into_store(url, range, expected, &self.store)
            .await
        {
            Ok(_) => {
                let bytes = self.store.get(expected)?;
                if let Some(notify) = &self.verified_chunk_notify {
                    notify(VerifiedChunkUpdate {
                        tensor_hash,
                        chunk_index,
                        chunk_hash: expected,
                        bytes: bytes.clone(),
                    });
                }
                Ok(bytes)
            }
            Err(webseed_error) => anyhow::bail!(
                "chunk unavailable from swarm and WebSeed: {}; {}",
                peer_error.unwrap_or_else(|| String::from("no peer source")),
                webseed_error
            ),
        }
    }

    pub async fn fetch_manifest_range(
        &self,
        manifest: &Manifest,
        origin_url: &str,
        requested: Range<u64>,
        peer: Option<PeerFetch>,
    ) -> anyhow::Result<Vec<u8>> {
        let recipe = validate_manifest_recipe(manifest)?;
        anyhow::ensure!(
            requested.start < requested.end && requested.end <= recipe.file_size,
            "invalid manifest range"
        );
        let output_len = usize::try_from(requested.end - requested.start)
            .context("requested range does not fit in memory")?;
        let mut output = Vec::with_capacity(output_len);
        for segment in &recipe.segments {
            let offset = match segment {
                Segment::Literal { offset, .. }
                | Segment::Tensor { offset, .. }
                | Segment::ZeroFill { offset, .. } => *offset,
            };
            let length = match segment {
                Segment::Literal { bytes, .. } => bytes.len() as u64,
                Segment::Tensor { length, .. } | Segment::ZeroFill { length, .. } => *length,
            };
            let segment_end = offset.checked_add(length).context("segment overflow")?;
            let start = requested.start.max(offset);
            let end = requested.end.min(segment_end);
            if start >= end {
                continue;
            }
            match segment {
                Segment::Literal { bytes, .. } => {
                    let local_start = (start - offset) as usize;
                    let local_end = (end - offset) as usize;
                    output.extend_from_slice(&bytes[local_start..local_end]);
                }
                Segment::ZeroFill { .. } => {
                    let fill_len = usize::try_from(end - start)
                        .context("zero-fill range does not fit in memory")?;
                    output.extend(std::iter::repeat_n(0, fill_len))
                }
                Segment::Tensor {
                    offset,
                    tensor_hash,
                    tensor_offset,
                    length,
                } => {
                    let wanted_start = tensor_offset
                        .checked_add(start - offset)
                        .context("tensor range overflow")?;
                    let wanted_end = tensor_offset
                        .checked_add(end - offset)
                        .context("tensor range overflow")?;
                    let node = manifest
                        .tensors
                        .iter()
                        .find(|node| node.tensor_hash == *tensor_hash)
                        .context("tensor missing from manifest")?;
                    for chunk in &node.chunks {
                        let chunk_end = chunk
                            .offset
                            .checked_add(chunk.length)
                            .context("chunk range overflow")?;
                        let chunk_start = wanted_start.max(chunk.offset);
                        let chunk_stop = wanted_end.min(chunk_end);
                        if chunk_start >= chunk_stop {
                            continue;
                        }
                        let origin_start = offset
                            .checked_add(chunk.offset - tensor_offset)
                            .context("origin range overflow")?;
                        let origin_end = origin_start
                            .checked_add(chunk.length)
                            .context("origin range overflow")?;
                        let bytes = self
                            .fetch_chunk_with_identity(
                                origin_url,
                                origin_start..origin_end,
                                chunk.hash,
                                *tensor_hash,
                                chunk.index,
                                peer.clone(),
                            )
                            .await?;
                        anyhow::ensure!(
                            bytes.len() as u64 == chunk.length,
                            "fetcher returned malformed chunk length"
                        );
                        let local_start = (chunk_start - chunk.offset) as usize;
                        let local_end = (chunk_stop - chunk.offset) as usize;
                        output.extend_from_slice(&bytes[local_start..local_end]);
                    }
                    anyhow::ensure!(*length <= recipe.file_size, "invalid tensor segment length");
                }
            }
        }
        anyhow::ensure!(
            output.len() as u64 == requested.end - requested.start,
            "manifest recipe has uncovered bytes"
        );
        Ok(output)
    }
}

pub fn validate_manifest_recipe(manifest: &Manifest) -> anyhow::Result<&ts_core::FileRecipe> {
    anyhow::ensure!(manifest.verify_root(), "manifest self-check failed");
    let recipe = manifest
        .files
        .first()
        .context("manifest has no file recipe")?;
    anyhow::ensure!(recipe.file_size == manifest.file_size, "file size mismatch");

    let mut cursor = 0_u64;
    for segment in &recipe.segments {
        let (offset, length) = match segment {
            Segment::Literal { offset, bytes } => (*offset, bytes.len() as u64),
            Segment::Tensor { offset, length, .. } | Segment::ZeroFill { offset, length } => {
                (*offset, *length)
            }
        };
        anyhow::ensure!(offset == cursor, "manifest recipe has a gap or overlap");
        anyhow::ensure!(length > 0, "manifest recipe contains an empty segment");
        cursor = offset.checked_add(length).context("segment overflow")?;
        anyhow::ensure!(cursor <= recipe.file_size, "segment exceeds file size");

        if let Segment::Tensor {
            tensor_hash,
            tensor_offset,
            length,
            ..
        } = segment
        {
            let node = manifest
                .tensors
                .iter()
                .find(|node| node.tensor_hash == *tensor_hash)
                .context("tensor missing from manifest")?;
            let tensor_end = tensor_offset
                .checked_add(*length)
                .context("tensor segment overflow")?;
            anyhow::ensure!(
                tensor_end <= node.descriptor.byte_len,
                "tensor segment exceeds tensor"
            );
            let mut chunk_cursor = 0_u64;
            for (expected_index, chunk) in node.chunks.iter().enumerate() {
                anyhow::ensure!(
                    chunk.index == expected_index as u32,
                    "tensor chunks have invalid indexes"
                );
                anyhow::ensure!(
                    chunk.offset == chunk_cursor,
                    "tensor chunks have a gap or overlap"
                );
                anyhow::ensure!(chunk.length > 0, "tensor contains an empty chunk");
                chunk_cursor = chunk
                    .offset
                    .checked_add(chunk.length)
                    .context("chunk overflow")?;
                anyhow::ensure!(
                    chunk_cursor <= node.descriptor.byte_len,
                    "chunk exceeds tensor length"
                );
            }
            anyhow::ensure!(
                chunk_cursor == node.descriptor.byte_len,
                "tensor chunks do not cover tensor"
            );
        }
    }
    anyhow::ensure!(
        cursor == recipe.file_size,
        "manifest recipe has uncovered bytes"
    );
    Ok(recipe)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RangeError {
    #[error("invalid Range header")]
    Invalid,
    #[error("range is outside the artifact")]
    OutOfBounds,
    #[error("multiple ranges are not supported in MVP")]
    Multiple,
}

pub fn parse_single_range(header: &str, file_size: u64) -> Result<Range<u64>, RangeError> {
    let value = header.strip_prefix("bytes=").ok_or(RangeError::Invalid)?;
    if value.contains(',') {
        return Err(RangeError::Multiple);
    }
    let (start, end) = value.split_once('-').ok_or(RangeError::Invalid)?;
    let (start, end) = if start.is_empty() {
        let suffix = end.parse::<u64>().map_err(|_| RangeError::Invalid)?;
        if suffix == 0 {
            return Err(RangeError::Invalid);
        }
        (file_size.saturating_sub(suffix), file_size)
    } else {
        let start = start.parse::<u64>().map_err(|_| RangeError::Invalid)?;
        let end = if end.is_empty() {
            file_size
        } else {
            end.parse::<u64>()
                .map_err(|_| RangeError::Invalid)?
                .saturating_add(1)
        };
        (start, end.min(file_size))
    };
    if start >= end || start >= file_size {
        return Err(RangeError::OutOfBounds);
    }
    Ok(start..end)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedChunk {
    pub tensor_hash: Hash32,
    pub chunk: ChunkRef,
}

pub fn chunks_for_range(manifest: &Manifest, requested: Range<u64>) -> Vec<PlannedChunk> {
    let mut result = Vec::new();
    let Some(recipe) = manifest.files.first() else {
        return result;
    };
    for segment in &recipe.segments {
        let Segment::Tensor {
            offset,
            tensor_hash,
            tensor_offset,
            length,
        } = segment
        else {
            continue;
        };
        let segment_end = offset + length;
        if segment_end <= requested.start || *offset >= requested.end {
            continue;
        }
        let wanted_start = requested.start.saturating_sub(*offset).max(*tensor_offset);
        let wanted_end = (requested.end.saturating_sub(*offset)).min(*tensor_offset + *length);
        if wanted_start >= wanted_end {
            continue;
        }
        if let Some(node) = manifest
            .tensors
            .iter()
            .find(|node| node.tensor_hash == *tensor_hash)
        {
            for chunk in &node.chunks {
                let chunk_end = chunk.offset + chunk.length;
                if chunk_end > wanted_start && chunk.offset < wanted_end {
                    result.push(PlannedChunk {
                        tensor_hash: *tensor_hash,
                        chunk: chunk.clone(),
                    });
                }
            }
        }
    }
    result
}

#[derive(Clone, Debug)]
pub struct FileProxyState {
    pub root: Arc<PathBuf>,
}

pub fn resolve_manifest_path<'a>(
    manifest: &'a Manifest,
    requested_path: &str,
) -> anyhow::Result<&'a str> {
    let recipe = manifest
        .files
        .first()
        .context("manifest has no file recipe")?;
    anyhow::ensure!(!requested_path.is_empty(), "model path is empty");
    anyhow::ensure!(
        !requested_path.split('/').any(|part| part == ".."),
        "model path contains traversal"
    );
    anyhow::ensure!(
        requested_path == recipe.path,
        "model path is not in manifest"
    );
    Ok(&recipe.path)
}

pub fn resolve_origin_url(origin_url: &str, model_path: &str) -> anyhow::Result<String> {
    let mut url = reqwest::Url::parse(origin_url).context("origin URL is invalid")?;
    anyhow::ensure!(url.scheme() == "https", "origin URL must use HTTPS");
    if url.path().is_empty() || url.path().ends_with('/') {
        let base = url.path().trim_end_matches('/');
        let suffix = model_path.trim_start_matches('/');
        url.set_path(&format!("{base}/{suffix}"));
    }
    Ok(url.to_string())
}

#[derive(Clone)]
pub struct ManifestProxyState {
    pub manifest: Manifest,
    pub origin_url: String,
    pub engine: FetchEngine,
    pub peer: Option<PeerFetch>,
}

pub fn manifest_router(config: ManifestProxyState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { (StatusCode::OK, "ok\n") }))
        .route("/file/{*path}", get(serve_manifest_file))
        .with_state(config)
}

pub async fn serve_manifest(bind: &str, config: ManifestProxyState) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    axum::serve(listener, manifest_router(config)).await?;
    Ok(())
}

pub fn router(root: impl Into<PathBuf>) -> Router {
    let state = FileProxyState {
        root: Arc::new(root.into()),
    };
    Router::new()
        .route("/healthz", get(|| async { (StatusCode::OK, "ok\n") }))
        .route("/file/{*path}", get(serve_file))
        .with_state(state)
}

pub async fn serve(bind: &str, root: impl Into<PathBuf>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    axum::serve(listener, router(root)).await?;
    Ok(())
}

async fn serve_file(
    State(state): State<FileProxyState>,
    AxumPath(path): AxumPath<String>,
    request: Request<Body>,
) -> Result<Response, StatusCode> {
    let relative = PathBuf::from(path);
    if relative
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let file_path = state.root.join(relative);
    let metadata = tokio::fs::metadata(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if !metadata.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }
    let file_size = metadata.len();
    let range = request
        .headers()
        .get(header::RANGE)
        .map(|value| {
            value
                .to_str()
                .map_err(|_| StatusCode::RANGE_NOT_SATISFIABLE)
        })
        .transpose()?
        .map(|value| {
            parse_single_range(value, file_size).map_err(|_| StatusCode::RANGE_NOT_SATISFIABLE)
        })
        .transpose()?
        .unwrap_or(0..file_size);
    let length = range.end - range.start;
    let mut file = tokio::fs::File::open(file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    file.seek(SeekFrom::Start(range.start))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let stream = ReaderStream::new(file.take(length));
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = if range.start == 0 && range.end == file_size {
        StatusCode::OK
    } else {
        StatusCode::PARTIAL_CONTENT
    };
    let partial = response.status() == StatusCode::PARTIAL_CONTENT;
    let headers = response.headers_mut();
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&length.to_string()).unwrap(),
    );
    if partial {
        headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&format!(
                "bytes {}-{}/{}",
                range.start,
                range.end - 1,
                file_size
            ))
            .unwrap(),
        );
    }
    Ok(response)
}

async fn serve_manifest_file(
    State(state): State<ManifestProxyState>,
    AxumPath(path): AxumPath<String>,
    request: Request<Body>,
) -> Result<Response, StatusCode> {
    let recipe_path =
        resolve_manifest_path(&state.manifest, &path).map_err(|_| StatusCode::NOT_FOUND)?;
    let recipe = state
        .manifest
        .files
        .iter()
        .find(|recipe| recipe.path == recipe_path)
        .ok_or(StatusCode::NOT_FOUND)?;
    let file_size = recipe.file_size;
    let range = request
        .headers()
        .get(header::RANGE)
        .map(|value| {
            value
                .to_str()
                .map_err(|_| StatusCode::RANGE_NOT_SATISFIABLE)
        })
        .transpose()?
        .map(|value| {
            parse_single_range(value, file_size).map_err(|_| StatusCode::RANGE_NOT_SATISFIABLE)
        })
        .transpose()?
        .unwrap_or(0..file_size);
    let bytes = state
        .engine
        .fetch_manifest_range(
            &state.manifest,
            &state.origin_url,
            range.clone(),
            state.peer.clone(),
        )
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = if range.start == 0 && range.end == file_size {
        StatusCode::OK
    } else {
        StatusCode::PARTIAL_CONTENT
    };
    let partial = response.status() == StatusCode::PARTIAL_CONTENT;
    let headers = response.headers_mut();
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&(range.end - range.start).to_string()).unwrap(),
    );
    if partial {
        headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&format!(
                "bytes {}-{}/{}",
                range.start,
                range.end - 1,
                file_size
            ))
            .unwrap(),
        );
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    #[test]
    fn parses_open_and_suffix_ranges() {
        assert_eq!(parse_single_range("bytes=10-19", 100).unwrap(), 10..20);
        assert_eq!(parse_single_range("bytes=10-", 100).unwrap(), 10..100);
        assert_eq!(parse_single_range("bytes=-10", 100).unwrap(), 90..100);
        assert_eq!(
            parse_single_range("bytes=100-", 100),
            Err(RangeError::OutOfBounds)
        );
    }

    #[test]
    fn rejects_multi_ranges_for_mvp() {
        assert_eq!(
            parse_single_range("bytes=0-1,4-5", 10),
            Err(RangeError::Multiple)
        );
    }

    #[tokio::test]
    async fn fetch_engine_prefers_peer_then_hits_cas() {
        let root = tempfile::tempdir().unwrap();
        let notifications = Arc::new(AtomicUsize::new(0));
        let observed_notifications = Arc::clone(&notifications);
        let engine = FetchEngine::new(ts_store::ObjectStore::open(root.path()).unwrap())
            .with_verified_chunk_notify(Arc::new(move |_| {
                observed_notifications.fetch_add(1, Ordering::Relaxed);
            }));
        let payload = b"chunk".to_vec();
        let expected = sha256(&payload);
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let peer: PeerFetch = Arc::new(move |_, _, _, _| {
            observed.fetch_add(1, Ordering::Relaxed);
            let payload = payload.clone();
            Box::pin(async move { Ok(payload) })
        });
        assert_eq!(
            engine
                .fetch_chunk("https://invalid.example/chunk", 0..5, expected, Some(peer))
                .await
                .unwrap(),
            b"chunk"
        );
        assert_eq!(
            engine
                .fetch_chunk("https://invalid.example/chunk", 0..5, expected, None)
                .await
                .unwrap(),
            b"chunk"
        );
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert_eq!(notifications.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn manifest_fetch_notifies_with_verified_chunk_identity() {
        let root = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(root.path()).unwrap();
        let payload = b"runtime-provider".to_vec();
        let chunk_hash = sha256(&payload);
        let tensor_hash = sha256(b"runtime-tensor");
        let manifest = Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            payload.len() as u64,
            vec![ts_core::TensorNode {
                descriptor: ts_core::TensorDescriptor {
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
            vec![ts_core::FileRecipe {
                path: "model.safetensors".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size: payload.len() as u64,
                segments: vec![Segment::Tensor {
                    offset: 0,
                    tensor_hash,
                    tensor_offset: 0,
                    length: payload.len() as u64,
                }],
            }],
        );
        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel(1);
        let engine =
            FetchEngine::new(store.clone()).with_verified_chunk_notify(Arc::new(move |update| {
                notify_tx.try_send(update).unwrap()
            }));
        let peer_payload = payload.clone();
        let expected_len = payload.len() as u64;
        let peer: PeerFetch = Arc::new(move |requested_tensor, index, range, expected| {
            assert_eq!(requested_tensor, tensor_hash);
            assert_eq!(index, 0);
            assert_eq!(range, 0..expected_len);
            assert_eq!(expected, chunk_hash);
            let payload = peer_payload.clone();
            Box::pin(async move { Ok(payload) })
        });

        let bytes = engine
            .fetch_manifest_range(
                &manifest,
                "https://invalid.example/model.safetensors",
                0..payload.len() as u64,
                Some(peer),
            )
            .await
            .unwrap();
        let update = notify_rx.recv().await.unwrap();

        assert_eq!(bytes, payload);
        assert_eq!(store.get(chunk_hash).unwrap(), payload);
        assert_eq!(update.tensor_hash, tensor_hash);
        assert_eq!(update.chunk_index, 0);
        assert_eq!(update.chunk_hash, chunk_hash);
        assert_eq!(update.bytes, payload);
    }

    #[tokio::test]
    async fn fetch_engine_rejects_origin_outage_without_cache_write() {
        let root = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(root.path()).unwrap();
        let expected = sha256(b"missing");
        let engine = FetchEngine::new(store.clone());
        let error = engine
            .fetch_chunk("http://127.0.0.1:1/unavailable", 0..7, expected, None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("WebSeed"));
        assert!(!store.contains(expected));
    }

    #[tokio::test]
    async fn manifest_range_assembles_literal_and_tensor_bytes() {
        let root = tempfile::tempdir().unwrap();
        let engine = FetchEngine::new(ts_store::ObjectStore::open(root.path()).unwrap());
        let payload = b"tensor".to_vec();
        let chunk_hash = sha256(&payload);
        let tensor_hash = sha256(b"tensor-identity");
        let manifest = Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            10,
            vec![ts_core::TensorNode {
                descriptor: ts_core::TensorDescriptor {
                    name: "weight".into(),
                    shape: vec![6],
                    dtype: "U8".into(),
                    byte_len: 6,
                },
                tensor_hash,
                chunks: vec![ChunkRef {
                    index: 0,
                    offset: 0,
                    length: 6,
                    hash: chunk_hash,
                }],
            }],
            vec![ts_core::FileRecipe {
                path: "model.safetensors".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size: 10,
                segments: vec![
                    Segment::Literal {
                        offset: 0,
                        bytes: b"head".to_vec(),
                    },
                    Segment::Tensor {
                        offset: 4,
                        tensor_hash,
                        tensor_offset: 0,
                        length: 6,
                    },
                ],
            }],
        );
        let peer: PeerFetch = Arc::new(move |_, _, _, _| {
            let payload = payload.clone();
            Box::pin(async move { Ok(payload) })
        });
        let bytes = engine
            .fetch_manifest_range(&manifest, "https://invalid.example/model", 2..8, Some(peer))
            .await
            .unwrap();
        assert_eq!(bytes, b"adtens");
    }

    #[tokio::test]
    async fn manifest_router_serves_verified_tensor_range_with_headers() {
        let root = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(root.path()).unwrap();
        let payload = b"tensor".to_vec();
        let chunk_hash = sha256(&payload);
        store.put_verified(chunk_hash, &payload).unwrap();
        let tensor_hash = sha256(b"tensor-identity");
        let manifest = Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            10,
            vec![ts_core::TensorNode {
                descriptor: ts_core::TensorDescriptor {
                    name: "weight".into(),
                    shape: vec![6],
                    dtype: "U8".into(),
                    byte_len: 6,
                },
                tensor_hash,
                chunks: vec![ChunkRef {
                    index: 0,
                    offset: 0,
                    length: 6,
                    hash: chunk_hash,
                }],
            }],
            vec![ts_core::FileRecipe {
                path: "model.safetensors".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size: 10,
                segments: vec![
                    Segment::Literal {
                        offset: 0,
                        bytes: b"head".to_vec(),
                    },
                    Segment::Tensor {
                        offset: 4,
                        tensor_hash,
                        tensor_offset: 0,
                        length: 6,
                    },
                ],
            }],
        );
        let app = manifest_router(ManifestProxyState {
            manifest,
            origin_url: "https://invalid.example/model.safetensors".into(),
            engine: FetchEngine::new(store),
            peer: None,
        });
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/file/model.safetensors")
                    .header(header::RANGE, "bytes=2-7")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes 2-7/10");
        assert_eq!(response.headers()[header::CONTENT_LENGTH], "6");
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX).await.unwrap(),
            "adtens"
        );
    }

    #[tokio::test]
    async fn manifest_router_fetches_missing_tensor_from_lan_peer() {
        let provider_payload = b"tensor".to_vec();
        let tensor_hash = sha256(b"tensor-identity");
        let chunk_hash = sha256(&provider_payload);
        let provider = ts_p2p::ChunkProvider::default();
        provider
            .insert_chunk(tensor_hash, 0, chunk_hash, provider_payload)
            .unwrap();
        let server = ts_p2p::build_lan_swarm_with_listeners(&["/ip4/127.0.0.1/tcp/0"]).unwrap();
        let server_id = *server.local_peer_id();
        let (address_tx, address_rx) = tokio::sync::oneshot::channel();
        let server_task = tokio::spawn(ts_p2p::run_lan_node_with_provider_and_notify(
            server,
            provider,
            Some(address_tx),
        ));
        let address = address_rx
            .await
            .unwrap()
            .with(libp2p::multiaddr::Protocol::P2p(server_id.into()));
        let client = Arc::new(tokio::sync::Mutex::new(ts_p2p::LanClient::new(
            ts_p2p::build_lan_swarm_with_listeners(&[]).unwrap(),
        )));
        client.lock().await.swarm_mut().dial(address).unwrap();
        let root = tempfile::tempdir().unwrap();
        let manifest = Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            10,
            vec![ts_core::TensorNode {
                descriptor: ts_core::TensorDescriptor {
                    name: "weight".into(),
                    shape: vec![6],
                    dtype: "U8".into(),
                    byte_len: 6,
                },
                tensor_hash,
                chunks: vec![ChunkRef {
                    index: 0,
                    offset: 0,
                    length: 6,
                    hash: chunk_hash,
                }],
            }],
            vec![ts_core::FileRecipe {
                path: "model.safetensors".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size: 10,
                segments: vec![
                    Segment::Literal {
                        offset: 0,
                        bytes: b"head".to_vec(),
                    },
                    Segment::Tensor {
                        offset: 4,
                        tensor_hash,
                        tensor_offset: 0,
                        length: 6,
                    },
                ],
            }],
        );
        let app = manifest_router(ManifestProxyState {
            manifest,
            origin_url: "https://invalid.example/model.safetensors".into(),
            engine: FetchEngine::new(ts_store::ObjectStore::open(root.path()).unwrap()),
            peer: Some(lan_peer_fetch(client, server_id, 3)),
        });
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/file/model.safetensors")
                    .header(header::RANGE, "bytes=2-7")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        server_task.abort();
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes 2-7/10");
        assert_eq!(response.headers()[header::CONTENT_LENGTH], "6");
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX).await.unwrap(),
            "adtens"
        );
    }

    #[test]
    fn resolver_validates_manifest_path_and_normalizes_https_base() {
        let manifest = Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            1,
            vec![],
            vec![ts_core::FileRecipe {
                path: "models/model.safetensors".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size: 1,
                segments: vec![Segment::Literal {
                    offset: 0,
                    bytes: vec![1],
                }],
            }],
        );
        assert_eq!(
            resolve_manifest_path(&manifest, "models/model.safetensors").unwrap(),
            "models/model.safetensors"
        );
        assert_eq!(
            resolve_origin_url("https://cdn.example/models/", "model.safetensors").unwrap(),
            "https://cdn.example/models/model.safetensors"
        );
        assert!(resolve_manifest_path(&manifest, "../model.safetensors").is_err());
        assert!(resolve_origin_url("http://cdn.example/", "model.safetensors").is_err());
    }

    #[tokio::test]
    async fn https_webseed_fallback_verifies_stores_and_notifies() {
        use axum::routing::get;
        use axum::Router;
        use axum_server::tls_rustls::RustlsConfig;
        use rcgen::generate_simple_self_signed;
        use std::net::SocketAddr;

        let _ = rustls::crypto::ring::default_provider().install_default();

        let payload = b"https-fallback".to_vec();
        let expected = sha256(&payload);
        let app = Router::new().route(
            "/model.bin",
            get({
                let payload = payload.clone();
                move |request: Request<Body>| {
                    let payload = payload.clone();
                    async move {
                        let range = request
                            .headers()
                            .get(header::RANGE)
                            .and_then(|value| value.to_str().ok())
                            .unwrap_or("bytes=0-13");
                        assert_eq!(range, "bytes=0-13");
                        (StatusCode::PARTIAL_CONTENT, payload)
                    }
                }
            }),
        );
        let cert = generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.cert.der().to_vec();
        let client = Client::builder()
            .add_root_certificate(reqwest::Certificate::from_der(&cert_der).unwrap())
            .build()
            .unwrap();
        let tls = RustlsConfig::from_der(vec![cert_der], cert.key_pair.serialize_der())
            .await
            .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        drop(listener);
        let server =
            tokio::spawn(axum_server::bind_rustls(address, tls).serve(app.into_make_service()));
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if tokio::net::TcpStream::connect(address).await.is_ok() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("HTTPS test server did not start");
        let root = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(root.path()).unwrap();
        let (publish_tx, mut publish_rx) = tokio::sync::mpsc::channel(1);
        let engine = FetchEngine::new(store.clone()).with_verified_chunk_sender(publish_tx);
        let engine = FetchEngine {
            webseed: WebSeeder::with_client(client),
            ..engine
        };
        let url = format!("https://localhost:{}/model.bin", address.port());
        let bytes = engine
            .fetch_chunk(&url, 0..14, expected, None)
            .await
            .unwrap();
        server.abort();
        assert_eq!(bytes, b"https-fallback");
        assert_eq!(store.get(expected).unwrap(), b"https-fallback");
        assert_eq!(publish_rx.recv().await, Some(expected));
    }

    #[tokio::test]
    async fn https_fallback_publishes_provider_for_second_node_discovery() {
        use axum::routing::get;
        use axum::Router;
        use axum_server::tls_rustls::RustlsConfig;
        use libp2p::futures::StreamExt;
        use rcgen::generate_simple_self_signed;
        use std::net::SocketAddr;
        use ts_p2p::{find_tensor_providers, LanBehaviourEvent};

        let _ = rustls::crypto::ring::default_provider().install_default();
        let payload = b"published-fallback".to_vec();
        let chunk_hash = sha256(&payload);
        let tensor_hash = sha256(b"published tensor identity");
        let file_size = payload.len() as u64;
        let manifest = Arc::new(Manifest::new(
            ts_core::ArtifactFormat::Safetensors,
            file_size,
            vec![ts_core::TensorNode {
                descriptor: ts_core::TensorDescriptor {
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
            vec![ts_core::FileRecipe {
                path: "model.bin".into(),
                format: ts_core::ArtifactFormat::Safetensors,
                file_size,
                segments: vec![Segment::Tensor {
                    offset: 0,
                    tensor_hash,
                    tensor_offset: 0,
                    length: file_size,
                }],
            }],
        ));
        let app = Router::new().route(
            "/model.bin",
            get({
                let payload = payload.clone();
                move || {
                    let payload = payload.clone();
                    async move { (StatusCode::PARTIAL_CONTENT, payload) }
                }
            }),
        );
        let cert = generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.cert.der().to_vec();
        let client = Client::builder()
            .add_root_certificate(reqwest::Certificate::from_der(&cert_der).unwrap())
            .build()
            .unwrap();
        let tls = RustlsConfig::from_der(vec![cert_der], cert.key_pair.serialize_der())
            .await
            .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        drop(listener);
        let https_task =
            tokio::spawn(axum_server::bind_rustls(address, tls).serve(app.into_make_service()));
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if tokio::net::TcpStream::connect(address).await.is_ok() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("HTTPS test server did not start");

        let root = tempfile::tempdir().unwrap();
        let store = ts_store::ObjectStore::open(root.path()).unwrap();
        let provider = ts_p2p::ChunkProvider::from_manifest(&store, &manifest).unwrap();
        let server = ts_p2p::build_lan_swarm_with_listeners(&["/ip4/127.0.0.1/tcp/0"]).unwrap();
        let server_id = *server.local_peer_id();
        let (address_tx, address_rx) = tokio::sync::oneshot::channel();
        let (publish_tx, mut publish_rx) = tokio::sync::mpsc::channel(1);
        let live_provider = provider.clone();
        let provider_manifest = manifest.clone();
        let notify = Arc::new(move |update: VerifiedChunkUpdate| {
            if live_provider
                .insert_manifest_chunk(
                    &provider_manifest,
                    update.tensor_hash,
                    update.chunk_index,
                    update.chunk_hash,
                    update.bytes,
                )
                .is_ok()
            {
                let _ = publish_tx.try_send(update.tensor_hash);
            }
        });
        let server_task = tokio::spawn(async move {
            let mut server = server;
            let mut listen_tx = Some(address_tx);
            ts_p2p::run_lan_node_with_provider_and_events(
                &mut server,
                provider,
                &mut listen_tx,
                Some(&mut publish_rx),
            )
            .await
        });
        let server_address = address_rx
            .await
            .unwrap()
            .with(libp2p::multiaddr::Protocol::P2p(server_id.into()));
        let engine = FetchEngine {
            store: store.clone(),
            webseed: WebSeeder::with_client(client),
            verified_chunk_notify: Some(notify),
        };
        let url = format!("https://localhost:{}/model.bin", address.port());
        assert_eq!(
            engine
                .fetch_manifest_range(&manifest, &url, 0..file_size, None)
                .await
                .unwrap(),
            payload
        );

        let mut discovery = ts_p2p::build_lan_swarm_with_listeners(&[]).unwrap();
        discovery.dial(server_address).unwrap();
        let query_id = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match discovery.select_next_some().await {
                    libp2p::swarm::SwarmEvent::Behaviour(LanBehaviourEvent::Kad(
                        libp2p::kad::Event::RoutingUpdated { peer, .. },
                    )) if peer == server_id => {
                        break find_tensor_providers(&mut discovery, &tensor_hash);
                    }
                    _ => {}
                }
            }
        })
        .await
        .expect("DHT routing did not become ready");
        let discovered = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                if let libp2p::swarm::SwarmEvent::Behaviour(LanBehaviourEvent::Kad(event)) =
                    discovery.select_next_some().await
                {
                    if let libp2p::kad::Event::OutboundQueryProgressed {
                        id,
                        result:
                            libp2p::kad::QueryResult::GetProviders(Ok(
                                libp2p::kad::GetProvidersOk::FoundProviders { providers, .. },
                            )),
                        ..
                    } = event
                    {
                        if id == query_id {
                            return providers.contains(&server_id);
                        }
                    }
                }
            }
        })
        .await
        .expect("published provider lookup timed out");
        let mut client = ts_p2p::LanClient::new(discovery);
        let fetched = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.fetch_chunk(
                server_id,
                ts_p2p::ChunkRequest {
                    request_id: 1,
                    tensor_hash,
                    chunk_index: 0,
                    expected_hash: chunk_hash,
                },
                3,
            ),
        )
        .await
        .expect("independent P2P chunk fetch timed out")
        .unwrap();
        https_task.abort();
        server_task.abort();
        assert!(
            discovered,
            "second node did not discover published provider"
        );
        assert_eq!(fetched, payload);
    }
}
