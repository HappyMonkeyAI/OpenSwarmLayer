//! Content-addressed object storage and manifest materialization.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use ts_core::{sha256, Hash32, Manifest, Segment};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VerifiedChunks {
    pub manifest: Hash32,
    pub chunks: Vec<u32>,
    #[serde(default)]
    pub tensors: Vec<(Hash32, Vec<u32>)>,
}

impl VerifiedChunks {
    pub fn contains(&self, index: u32) -> bool {
        self.chunks.binary_search(&index).is_ok()
    }

    pub fn mark(&mut self, index: u32) {
        if let Err(position) = self.chunks.binary_search(&index) {
            self.chunks.insert(position, index);
        }
    }

    pub fn contains_tensor(&self, tensor: Hash32, index: u32) -> bool {
        self.tensors.iter().find_map(|(hash, chunks)| {
            (*hash == tensor).then(|| chunks.binary_search(&index).is_ok())
        }) == Some(true)
    }

    pub fn mark_tensor(&mut self, tensor: Hash32, index: u32) {
        let position = self.tensors.iter().position(|(hash, _)| *hash == tensor);
        let chunks = if let Some(position) = position {
            &mut self.tensors[position].1
        } else {
            self.tensors.push((tensor, Vec::new()));
            &mut self.tensors.last_mut().unwrap().1
        };
        if let Err(position) = chunks.binary_search(&index) {
            chunks.insert(position, index);
        }
    }
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("object hash mismatch: expected {expected:?}, got {actual:?}")]
    HashMismatch { expected: Hash32, actual: Hash32 },
    #[error("object is missing: {0:?}")]
    Missing(Hash32),
    #[error("manifest contains an invalid segment")]
    InvalidSegment,
}

#[derive(Clone, Debug)]
pub struct ObjectStore {
    root: PathBuf,
}

impl ObjectStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let root = root.into();
        fs::create_dir_all(root.join("objects"))?;
        Ok(Self { root })
    }

    pub fn object_path(&self, hash: Hash32) -> PathBuf {
        let encoded = hash
            .0
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        self.root.join("objects").join(&encoded[..2]).join(encoded)
    }

    pub fn contains(&self, hash: Hash32) -> bool {
        self.object_path(hash).is_file()
    }

    pub fn put_verified(&self, expected: Hash32, bytes: &[u8]) -> Result<PathBuf, StoreError> {
        let actual = sha256(bytes);
        if actual != expected {
            return Err(StoreError::HashMismatch { expected, actual });
        }
        let destination = self.object_path(expected);
        if destination.is_file() {
            return Ok(destination);
        }
        let parent = destination.parent().ok_or(StoreError::InvalidSegment)?;
        fs::create_dir_all(parent)?;
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(".tmp-{}-{sequence}", std::process::id()));
        {
            let mut file = File::create(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        match fs::rename(&temporary, &destination) {
            Ok(()) => Ok(destination),
            Err(_error) if destination.is_file() => {
                let _ = fs::remove_file(&temporary);
                Ok(destination)
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                Err(StoreError::Io(error))
            }
        }
    }

    pub fn get(&self, hash: Hash32) -> Result<Vec<u8>, StoreError> {
        let path = self.object_path(hash);
        let bytes = fs::read(&path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                StoreError::Missing(hash)
            } else {
                StoreError::Io(error)
            }
        })?;
        let actual = sha256(&bytes);
        if actual != hash {
            return Err(StoreError::HashMismatch {
                expected: hash,
                actual,
            });
        }
        Ok(bytes)
    }

    /// Report object files not referenced by the supplied live hash set.
    /// Deletion is intentionally left to a separate maintenance operation.
    pub fn unreferenced_objects(
        &self,
        live: impl IntoIterator<Item = Hash32>,
    ) -> Result<Vec<PathBuf>, StoreError> {
        let live = live.into_iter().collect::<HashSet<_>>();
        let mut garbage = Vec::new();
        for prefix in fs::read_dir(self.root.join("objects"))? {
            let prefix = prefix?;
            if !prefix.file_type()?.is_dir() {
                continue;
            }
            for entry in fs::read_dir(prefix.path())? {
                let entry = entry?;
                if !entry.file_type()?.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let parsed = (name.len() == 64)
                    .then(|| {
                        let mut bytes = [0_u8; 32];
                        for (index, slot) in bytes.iter_mut().enumerate() {
                            *slot = u8::from_str_radix(&name[index * 2..index * 2 + 2], 16).ok()?;
                        }
                        Some(Hash32(bytes))
                    })
                    .flatten();
                if parsed.is_none_or(|hash| !live.contains(&hash)) {
                    garbage.push(entry.path());
                }
            }
        }
        garbage.sort();
        Ok(garbage)
    }

    pub fn remove_unreferenced_objects(
        &self,
        live: impl IntoIterator<Item = Hash32>,
    ) -> Result<Vec<PathBuf>, StoreError> {
        let garbage = self.unreferenced_objects(live)?;
        for path in &garbage {
            fs::remove_file(path)?;
        }
        Ok(garbage)
    }

    pub fn save_state(&self, name: &str, state: &VerifiedChunks) -> Result<PathBuf, StoreError> {
        let destination = self.root.join(format!("{name}.tsstate"));
        let temporary = destination.with_extension("tsstate.tmp");
        let bytes = serde_json::to_vec_pretty(state)
            .map_err(|error| StoreError::Io(io::Error::new(io::ErrorKind::InvalidData, error)))?;
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, &destination)?;
        Ok(destination)
    }

    pub fn load_state(&self, name: &str) -> Result<Option<VerifiedChunks>, StoreError> {
        let path = self.root.join(format!("{name}.tsstate"));
        match fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| StoreError::Io(io::Error::new(io::ErrorKind::InvalidData, error))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(StoreError::Io(error)),
        }
    }

    pub fn materialize(
        &self,
        manifest: &Manifest,
        output: impl AsRef<Path>,
    ) -> Result<(), StoreError> {
        if !manifest.verify_root() {
            return Err(StoreError::InvalidSegment);
        }
        let recipe = manifest.files.first().ok_or(StoreError::InvalidSegment)?;
        validate_recipe(recipe)?;
        let output = output.as_ref();
        let temporary = output.with_extension("partial");
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&temporary)?;
        file.set_len(recipe.file_size)?;
        for segment in &recipe.segments {
            match segment {
                Segment::Literal { offset, bytes } => write_at(&mut file, *offset, bytes)?,
                Segment::ZeroFill { offset, length } => {
                    file.seek(SeekFrom::Start(*offset))?;
                    let zeroes = vec![0_u8; (*length).min(1024 * 1024) as usize];
                    let mut remaining = *length;
                    while remaining > 0 {
                        let n = remaining.min(zeroes.len() as u64) as usize;
                        file.write_all(&zeroes[..n])?;
                        remaining -= n as u64;
                    }
                }
                Segment::Tensor {
                    offset,
                    tensor_hash,
                    tensor_offset,
                    length,
                } => {
                    let bytes = self.get(*tensor_hash)?;
                    let end = tensor_offset
                        .checked_add(*length)
                        .ok_or(StoreError::InvalidSegment)? as usize;
                    let start = *tensor_offset as usize;
                    if start > end || end > bytes.len() {
                        return Err(StoreError::InvalidSegment);
                    }
                    write_at(&mut file, *offset, &bytes[start..end])?;
                }
            }
        }
        file.sync_all()?;
        fs::rename(temporary, output)?;
        Ok(())
    }
}

fn write_at(file: &mut File, offset: u64, bytes: &[u8]) -> Result<(), StoreError> {
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(bytes)?;
    Ok(())
}

fn validate_recipe(recipe: &ts_core::FileRecipe) -> Result<(), StoreError> {
    for segment in &recipe.segments {
        let (offset, length) = match segment {
            Segment::Literal { offset, bytes } => (*offset, bytes.len() as u64),
            Segment::ZeroFill { offset, length } => (*offset, *length),
            Segment::Tensor { offset, length, .. } => (*offset, *length),
        };
        if offset
            .checked_add(length)
            .is_none_or(|end| end > recipe.file_size)
        {
            return Err(StoreError::InvalidSegment);
        }
        if let Segment::Tensor {
            tensor_offset,
            length,
            ..
        } = segment
        {
            if tensor_offset.checked_add(*length).is_none() {
                return Err(StoreError::InvalidSegment);
            }
        }
    }
    Ok(())
}

pub fn cache_root_is_valid(path: &Path) -> bool {
    path.is_absolute()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ts_core::{ArtifactFormat, ChunkRef, FileRecipe, TensorDescriptor, TensorNode};

    #[test]
    fn verified_store_rejects_wrong_bytes() {
        let root = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(root.path()).unwrap();
        let expected = sha256(b"good");
        assert!(matches!(
            store.put_verified(expected, b"bad"),
            Err(StoreError::HashMismatch { .. })
        ));
    }

    #[test]
    fn reachability_scan_reports_only_unreferenced_objects() {
        let root = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(root.path()).unwrap();
        let live = sha256(b"live");
        let dead = sha256(b"dead");
        store.put_verified(live, b"live").unwrap();
        store.put_verified(dead, b"dead").unwrap();
        let garbage = store.unreferenced_objects([live]).unwrap();
        assert_eq!(garbage, vec![store.object_path(dead)]);
        assert!(store.object_path(live).is_file());
        assert!(store.object_path(dead).is_file());
    }

    #[test]
    fn repair_removes_only_unreferenced_objects() {
        let root = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(root.path()).unwrap();
        let live = sha256(b"live");
        let dead = sha256(b"dead");
        store.put_verified(live, b"live").unwrap();
        store.put_verified(dead, b"dead").unwrap();

        let removed = store.remove_unreferenced_objects([live]).unwrap();

        assert_eq!(removed, vec![store.object_path(dead)]);
        assert!(store.contains(live));
        assert!(!store.contains(dead));
    }

    #[test]
    fn materializer_reconstructs_literal_and_tensor_segments() {
        let root = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(root.path()).unwrap();
        let payload = b"tensor";
        let hash = sha256(payload);
        store.put_verified(hash, payload).unwrap();
        let node = TensorNode {
            descriptor: TensorDescriptor {
                name: "x".into(),
                shape: vec![6],
                dtype: "U8".into(),
                byte_len: 6,
            },
            tensor_hash: hash,
            chunks: vec![ChunkRef {
                index: 0,
                offset: 0,
                length: 6,
                hash,
            }],
        };
        let manifest = Manifest::new(
            ArtifactFormat::Safetensors,
            10,
            vec![node],
            vec![FileRecipe {
                path: "x".into(),
                format: ArtifactFormat::Safetensors,
                file_size: 10,
                segments: vec![
                    Segment::Literal {
                        offset: 0,
                        bytes: b"head".to_vec(),
                    },
                    Segment::Tensor {
                        offset: 4,
                        tensor_hash: hash,
                        tensor_offset: 0,
                        length: 6,
                    },
                ],
            }],
        );
        let output = root.path().join("materialized.bin");
        store.materialize(&manifest, &output).unwrap();
        assert_eq!(fs::read(output).unwrap(), b"headtensor");
    }

    #[test]
    fn verified_chunk_state_is_sorted_and_resumable() {
        let root = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(root.path()).unwrap();
        let mut state = VerifiedChunks {
            manifest: sha256(b"manifest"),
            chunks: Vec::new(),
            tensors: Vec::new(),
        };
        state.mark(8);
        state.mark(2);
        state.mark(8);
        store.save_state("model", &state).unwrap();
        let restored = store.load_state("model").unwrap().unwrap();
        assert_eq!(restored.chunks, vec![2, 8]);
        assert!(restored.contains(8));
        assert!(!restored.contains(3));
    }

    #[test]
    fn per_tensor_state_is_sorted_isolated_and_resumable() {
        let root = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(root.path()).unwrap();
        let first = sha256(b"first-tensor");
        let second = sha256(b"second-tensor");
        let mut state = VerifiedChunks {
            manifest: sha256(b"manifest"),
            chunks: Vec::new(),
            tensors: Vec::new(),
        };
        state.mark_tensor(first, 8);
        state.mark_tensor(first, 2);
        state.mark_tensor(first, 8);
        state.mark_tensor(second, 2);
        store.save_state("model", &state).unwrap();

        let restored = store.load_state("model").unwrap().unwrap();
        assert!(restored.contains_tensor(first, 2));
        assert!(restored.contains_tensor(first, 8));
        assert!(!restored.contains_tensor(first, 3));
        assert!(restored.contains_tensor(second, 2));
        assert!(!restored.contains_tensor(second, 8));
        assert_eq!(
            restored
                .tensors
                .iter()
                .find(|(hash, _)| *hash == first)
                .unwrap()
                .1,
            vec![2, 8]
        );
        assert_eq!(
            restored
                .tensors
                .iter()
                .find(|(hash, _)| *hash == second)
                .unwrap()
                .1,
            vec![2]
        );
    }
}
