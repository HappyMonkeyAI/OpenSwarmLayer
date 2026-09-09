//! Local HTTP range planning and file-serving primitives.

use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use reqwest::Client;
use std::ops::Range;
use std::path::PathBuf;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
