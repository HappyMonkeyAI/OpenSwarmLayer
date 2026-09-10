//! Bounded, metadata-only readers for model container formats.

use anyhow::{bail, ensure, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use ts_core::{
    ArtifactFormat, ChunkRef, FileRecipe, HashWriter, Manifest, Segment, TensorDescriptor,
    TensorNode,
};

pub const MAX_HEADER_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TENSORS: u64 = 10_000_000;
pub const MAX_METADATA_ENTRIES: u64 = 1_000_000;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TensorRange {
    pub descriptor: TensorDescriptor,
    pub offset: u64,
    pub length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ModelIndex {
    pub format: ArtifactFormat,
    pub file_len: u64,
    pub tensors: Vec<TensorRange>,
}

pub fn supported_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("gguf") | Some("safetensors")
    )
}

pub fn inspect(path: impl AsRef<Path>) -> Result<ModelIndex> {
    let path = path.as_ref();
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let file_len = file.metadata()?.len();
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic)?;
    file.seek(SeekFrom::Start(0))?;
    if &magic == b"GGUF" {
        parse_gguf(&mut file, file_len)
    } else {
        parse_safetensors(&mut file, file_len)
    }
}

pub fn plan_chunks(length: u64, block_size: u64, target_size: u64) -> Result<Vec<(u64, u64)>> {
    ensure!(length > 0, "cannot chunk an empty tensor");
    ensure!(
        block_size > 0 && target_size >= block_size,
        "invalid chunk sizing"
    );
    let size = (target_size / block_size).max(1) * block_size;
    let mut chunks = Vec::new();
    let mut offset = 0_u64;
    while offset < length {
        let remaining = length - offset;
        let chunk_len = remaining.min(size);
        ensure!(
            chunk_len % block_size == 0 || chunk_len == remaining,
            "chunk is not block aligned"
        );
        chunks.push((offset, chunk_len));
        offset += chunk_len;
    }
    Ok(chunks)
}

pub fn hash_tensor(
    file: &mut File,
    range: &TensorRange,
    chunk_size: u64,
    block_size: u64,
) -> Result<TensorNode> {
    let planned = plan_chunks(range.length, block_size, chunk_size)?;
    let mut tensor_hasher = HashWriter::default();
    tensor_hasher.update(b"TS-TENSOR\0");
    tensor_hasher.update(&(range.descriptor.name.len() as u32).to_be_bytes());
    tensor_hasher.update(range.descriptor.name.as_bytes());
    tensor_hasher.update(&(range.descriptor.shape.len() as u32).to_be_bytes());
    for dimension in &range.descriptor.shape {
        tensor_hasher.update(&dimension.to_be_bytes());
    }
    tensor_hasher.update(&(range.descriptor.dtype.len() as u32).to_be_bytes());
    tensor_hasher.update(range.descriptor.dtype.as_bytes());
    tensor_hasher.update(&range.length.to_be_bytes());

    let mut chunks = Vec::with_capacity(planned.len());
    let mut buffer = vec![0_u8; 1024 * 1024];
    for (index, (relative, length)) in planned.into_iter().enumerate() {
        file.seek(SeekFrom::Start(range.offset + relative))?;
        let mut remaining = length;
        let mut chunk_hasher = HashWriter::default();
        while remaining > 0 {
            let wanted = remaining.min(buffer.len() as u64) as usize;
            file.read_exact(&mut buffer[..wanted])?;
            tensor_hasher.update(&buffer[..wanted]);
            chunk_hasher.update(&buffer[..wanted]);
            remaining -= wanted as u64;
        }
        chunks.push(ChunkRef {
            index: index as u32,
            offset: relative,
            length,
            hash: chunk_hasher.finalize(),
        });
    }
    Ok(TensorNode {
        descriptor: range.descriptor.clone(),
        tensor_hash: tensor_hasher.finalize(),
        chunks,
    })
}

pub fn build_manifest(path: impl AsRef<Path>, chunk_size: u64) -> Result<Manifest> {
    let path = path.as_ref();
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let index = inspect(path)?;
    let mut tensors = Vec::with_capacity(index.tensors.len());
    for range in &index.tensors {
        let block_size = if range.descriptor.dtype.starts_with("ggml:") {
            32
        } else {
            1
        };
        tensors.push(hash_tensor(&mut file, range, chunk_size, block_size)?);
    }
    let segments = build_file_recipe_segments(&mut file, &index, &tensors)?;
    let recipe = FileRecipe {
        path: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("model")
            .to_string(),
        format: index.format,
        file_size: index.file_len,
        segments,
    };
    Ok(Manifest::new(
        index.format,
        index.file_len,
        tensors,
        vec![recipe],
    ))
}

fn build_file_recipe_segments(
    file: &mut File,
    index: &ModelIndex,
    nodes: &[TensorNode],
) -> Result<Vec<Segment>> {
    let hashes = nodes
        .iter()
        .map(|node| (&node.descriptor.name, node.tensor_hash))
        .collect::<BTreeMap<_, _>>();
    let mut ranges = index
        .tensors
        .iter()
        .map(|tensor| (tensor.offset, tensor.length, tensor))
        .collect::<Vec<_>>();
    ranges.sort_by_key(|(offset, _, _)| *offset);
    let mut segments = Vec::new();
    let mut cursor = 0_u64;
    for (offset, length, tensor) in ranges {
        ensure!(cursor <= offset, "overlapping tensor ranges");
        if cursor < offset {
            segments.push(Segment::Literal {
                offset: cursor,
                bytes: read_range(file, cursor, offset - cursor)?,
            });
        }
        let tensor_hash = *hashes
            .get(&tensor.descriptor.name)
            .context("missing tensor hash")?;
        segments.push(Segment::Tensor {
            offset,
            tensor_hash,
            tensor_offset: 0,
            length,
        });
        cursor = offset.checked_add(length).context("file recipe overflow")?;
    }
    if cursor < index.file_len {
        segments.push(Segment::Literal {
            offset: cursor,
            bytes: read_range(file, cursor, index.file_len - cursor)?,
        });
    }
    Ok(segments)
}

fn read_range(file: &mut File, offset: u64, length: u64) -> Result<Vec<u8>> {
    ensure!(
        length <= MAX_HEADER_BYTES,
        "literal file segment is too large"
    );
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0_u8; length as usize];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn parse_gguf(file: &mut File, file_len: u64) -> Result<ModelIndex> {
    ensure!(
        file.read_u32::<LittleEndian>()? == u32::from_le_bytes(*b"GGUF"),
        "invalid GGUF magic"
    );
    let version = file.read_u32::<LittleEndian>()?;
    ensure!(
        (2..=3).contains(&version),
        "unsupported GGUF version {version}"
    );
    let tensor_count = file.read_u64::<LittleEndian>()?;
    let metadata_count = file.read_u64::<LittleEndian>()?;
    ensure!(tensor_count <= MAX_TENSORS, "too many tensors");
    ensure!(
        metadata_count <= MAX_METADATA_ENTRIES,
        "too many metadata entries"
    );
    let mut alignment = 32_u64;
    for _ in 0..metadata_count {
        let key = read_string(file)?;
        let value_type = file.read_u32::<LittleEndian>()?;
        if key == "general.alignment" && value_type == 4 {
            alignment = file.read_u32::<LittleEndian>()? as u64;
            ensure!(
                alignment.is_power_of_two() && alignment <= MAX_HEADER_BYTES,
                "invalid GGUF alignment"
            );
        } else {
            skip_metadata_value(file, value_type, 0)?;
        }
    }
    let mut infos = Vec::with_capacity(tensor_count as usize);
    let mut names = HashSet::with_capacity(tensor_count as usize);
    for _ in 0..tensor_count {
        let name = read_string(file)?;
        ensure!(
            names.insert(name.clone()),
            "duplicate GGUF tensor name: {name}"
        );
        let rank = file.read_u32::<LittleEndian>()?;
        ensure!(rank <= 8, "invalid GGUF rank");
        let mut shape = Vec::with_capacity(rank as usize);
        for _ in 0..rank {
            shape.push(file.read_u64::<LittleEndian>()?);
        }
        infos.push((
            name,
            shape,
            file.read_u32::<LittleEndian>()?,
            file.read_u64::<LittleEndian>()?,
        ));
    }
    let tensor_data_offset = align_up(file.stream_position()?, alignment);
    let mut tensors = Vec::with_capacity(infos.len());
    for (name, shape, dtype, relative_offset) in infos {
        let offset = tensor_data_offset
            .checked_add(relative_offset)
            .context("GGUF offset overflow")?;
        ensure!(offset % alignment == 0, "GGUF tensor offset is not aligned");
        let length = ggml_tensor_size(dtype, &shape)?;
        ensure!(
            offset
                .checked_add(length)
                .is_some_and(|end| end <= file_len),
            "GGUF tensor out of bounds"
        );
        tensors.push(TensorRange {
            descriptor: TensorDescriptor {
                name,
                shape,
                dtype: format!("ggml:{dtype}"),
                byte_len: length,
            },
            offset,
            length,
        });
    }
    tensors.sort_by(|a, b| a.descriptor.name.cmp(&b.descriptor.name));
    Ok(ModelIndex {
        format: ArtifactFormat::Gguf,
        file_len,
        tensors,
    })
}

fn parse_safetensors(file: &mut File, file_len: u64) -> Result<ModelIndex> {
    let header_len = file.read_u64::<LittleEndian>()?;
    ensure!(
        header_len <= MAX_HEADER_BYTES,
        "Safetensors header is too large"
    );
    let data_offset = 8_u64
        .checked_add(header_len)
        .context("header offset overflow")?;
    ensure!(data_offset <= file_len, "Safetensors header out of bounds");
    let mut bytes = vec![0_u8; header_len as usize];
    file.read_exact(&mut bytes)?;
    let header: BTreeMap<String, SafeTensorEntry> = serde_json::from_slice(&bytes)?;
    let mut tensors = Vec::with_capacity(header.len());
    for (name, entry) in header {
        if name == "__metadata__" {
            continue;
        }
        ensure!(entry.data_offsets.len() == 2, "invalid Safetensors offsets");
        let start = entry.data_offsets[0];
        let end = entry.data_offsets[1];
        ensure!(start <= end, "reversed Safetensors range");
        let offset = data_offset
            .checked_add(start)
            .context("Safetensors offset overflow")?;
        let length = end - start;
        ensure!(
            offset
                .checked_add(length)
                .is_some_and(|value| value <= file_len),
            "Safetensors tensor out of bounds"
        );
        tensors.push(TensorRange {
            descriptor: TensorDescriptor {
                name,
                shape: entry.shape,
                dtype: entry.dtype,
                byte_len: length,
            },
            offset,
            length,
        });
    }
    tensors.sort_by(|a, b| a.descriptor.name.cmp(&b.descriptor.name));
    let mut by_offset = tensors.iter().collect::<Vec<_>>();
    by_offset.sort_by_key(|tensor| tensor.offset);
    for pair in by_offset.windows(2) {
        let previous_end = pair[0]
            .offset
            .checked_add(pair[0].length)
            .context("Safetensors range overflow")?;
        ensure!(
            previous_end <= pair[1].offset,
            "overlapping Safetensors tensor ranges"
        );
    }
    Ok(ModelIndex {
        format: ArtifactFormat::Safetensors,
        file_len,
        tensors,
    })
}

#[derive(Deserialize)]
struct SafeTensorEntry {
    dtype: String,
    shape: Vec<u64>,
    data_offsets: Vec<u64>,
}

fn read_string(file: &mut File) -> Result<String> {
    let len = file.read_u64::<LittleEndian>()?;
    ensure!(len <= MAX_HEADER_BYTES, "GGUF string is too large");
    let mut bytes = vec![0_u8; len as usize];
    file.read_exact(&mut bytes)?;
    String::from_utf8(bytes).context("invalid UTF-8 string")
}

fn skip_metadata_value(file: &mut File, kind: u32, depth: u8) -> Result<()> {
    ensure!(depth < 32, "metadata nesting too deep");
    match kind {
        0 | 1 | 7 => {
            file.seek(SeekFrom::Current(1))?;
        }
        2 | 3 => {
            file.seek(SeekFrom::Current(2))?;
        }
        4 | 5 | 6 => {
            file.seek(SeekFrom::Current(4))?;
        }
        10 | 11 | 12 => {
            file.seek(SeekFrom::Current(8))?;
        }
        8 => {
            let len = file.read_u64::<LittleEndian>()?;
            ensure!(len <= MAX_HEADER_BYTES, "metadata string too large");
            file.seek(SeekFrom::Current(len as i64))?;
        }
        9 => {
            let element_type = file.read_u32::<LittleEndian>()?;
            let count = file.read_u64::<LittleEndian>()?;
            ensure!(count <= MAX_METADATA_ENTRIES, "metadata array too large");
            for _ in 0..count {
                skip_metadata_value(file, element_type, depth + 1)?;
            }
        }
        _ => bail!("unknown GGUF metadata type {kind}"),
    }
    Ok(())
}

fn align_up(value: u64, alignment: u64) -> u64 {
    value + (alignment - value % alignment) % alignment
}

fn ggml_tensor_size(dtype: u32, shape: &[u64]) -> Result<u64> {
    let elements = shape
        .iter()
        .try_fold(1_u64, |acc, value| acc.checked_mul(*value))
        .context("tensor element count overflow")?;
    let (block, bytes) = match dtype {
        0 => (1, 4),
        1 => (1, 2),
        2 => (32, 18),
        3 => (32, 20),
        6 => (32, 22),
        7 => (32, 24),
        8 => (32, 34),
        9 => (32, 36),
        10 => (256, 84),
        11 => (256, 110),
        12 => (256, 144),
        13 => (256, 176),
        14 => (256, 210),
        15 => (256, 292),
        24 => (1, 1),
        25 => (1, 2),
        26 => (1, 4),
        27 | 28 => (1, 8),
        30 => (1, 2),
        _ => bail!("unsupported GGML dtype {dtype}"),
    };
    ensure!(
        block == 1 || elements % block == 0,
        "tensor elements not aligned to dtype block"
    );
    (elements / block)
        .checked_mul(bytes)
        .context("tensor byte length overflow")
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use proptest::prelude::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_safetensors_header_without_loading_payload() {
        let header = br#"{"weight":{"dtype":"F32","shape":[2],"data_offsets":[0,8]}}"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&(header.len() as u64).to_le_bytes())
            .unwrap();
        file.write_all(header).unwrap();
        file.write_all(&[0; 8]).unwrap();
        let index = inspect(file.path()).unwrap();
        assert_eq!(index.format, ArtifactFormat::Safetensors);
        assert_eq!(index.tensors[0].descriptor.name, "weight");
        assert_eq!(index.tensors[0].offset, 8 + header.len() as u64);
        assert_eq!(index.tensors[0].length, 8);
    }

    #[test]
    fn parses_gguf_alignment_metadata_without_loading_payload() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.write_u32::<LittleEndian>(3).unwrap();
        bytes.write_u64::<LittleEndian>(1).unwrap();
        bytes.write_u64::<LittleEndian>(1).unwrap();
        bytes.write_u64::<LittleEndian>(17).unwrap();
        bytes.extend_from_slice(b"general.alignment");
        bytes.write_u32::<LittleEndian>(4).unwrap();
        bytes.write_u32::<LittleEndian>(64).unwrap();
        bytes.write_u64::<LittleEndian>(1).unwrap();
        bytes.extend_from_slice(b"x");
        bytes.write_u32::<LittleEndian>(1).unwrap();
        bytes.write_u64::<LittleEndian>(2).unwrap();
        bytes.write_u32::<LittleEndian>(0).unwrap();
        bytes.write_u64::<LittleEndian>(0).unwrap();
        bytes.resize(128, 0);
        bytes.extend_from_slice(&[0; 8]);
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&bytes).unwrap();
        let index = inspect(file.path()).unwrap();
        assert_eq!(index.tensors[0].offset, 128);
        assert_eq!(index.tensors[0].length, 8);
    }

    #[test]
    fn rejects_safetensors_range_outside_file() {
        let header = br#"{"weight":{"dtype":"F32","shape":[2],"data_offsets":[0,80]}}"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&(header.len() as u64).to_le_bytes())
            .unwrap();
        file.write_all(header).unwrap();
        assert!(inspect(file.path()).is_err());
    }

    #[test]
    fn malformed_bounded_inputs_never_panic() {
        for seed in 0_u16..512 {
            let mut bytes = Vec::with_capacity(96);
            let mut state = u32::from(seed).wrapping_add(1);
            for _ in 0..96 {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                bytes.push((state >> 24) as u8);
            }
            if seed % 2 == 0 {
                bytes[..4].copy_from_slice(b"GGUF");
            }
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(&bytes).unwrap();
            let result = std::panic::catch_unwind(|| inspect(file.path()));
            assert!(result.is_ok(), "parser panicked for seed {seed}");
            assert!(result.unwrap().is_err(), "accepted malformed seed {seed}");
        }
    }

    #[test]
    fn dtype_size_checks_block_alignment() {
        assert_eq!(ggml_tensor_size(0, &[2]).unwrap(), 8);
        assert!(ggml_tensor_size(2, &[31]).is_err());
        assert_eq!(ggml_tensor_size(2, &[32]).unwrap(), 18);
    }

    #[test]
    fn chunk_plan_covers_tensor_without_gaps() {
        let chunks = plan_chunks(100, 4, 16).unwrap();
        assert_eq!(
            chunks,
            vec![
                (0, 16),
                (16, 16),
                (32, 16),
                (48, 16),
                (64, 16),
                (80, 16),
                (96, 4)
            ]
        );
        let total: u64 = chunks.iter().map(|(_, length)| *length).sum();
        assert_eq!(total, 100);
    }

    proptest! {
        #[test]
        fn chunk_plan_property_preserves_coverage(
            length in 1_u64..=1_000_000,
            block_size in 1_u64..=4096,
            target_blocks in 1_u64..=64,
        ) {
            let target_size = block_size.saturating_mul(target_blocks);
            let chunks = plan_chunks(length, block_size, target_size).unwrap();
            prop_assert!(!chunks.is_empty());
            prop_assert_eq!(chunks.first().unwrap().0, 0);
            prop_assert_eq!(chunks.iter().map(|(_, size)| *size).sum::<u64>(), length);
            for pair in chunks.windows(2) {
                prop_assert_eq!(pair[0].0 + pair[0].1, pair[1].0);
            }
            for (offset, size) in &chunks[..chunks.len() - 1] {
                prop_assert_eq!(*offset % block_size, 0);
                prop_assert_eq!(*size % block_size, 0);
            }
        }
    }

    #[test]
    fn tensor_hashing_is_incremental_and_chunked() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[7_u8; 64]).unwrap();
        let range = TensorRange {
            descriptor: TensorDescriptor {
                name: "x".into(),
                shape: vec![64],
                dtype: "U8".into(),
                byte_len: 64,
            },
            offset: 0,
            length: 64,
        };
        let mut source = File::open(file.path()).unwrap();
        let node = hash_tensor(&mut source, &range, 16, 1).unwrap();
        assert_eq!(node.chunks.len(), 4);
        assert_eq!(
            node.chunks.iter().map(|chunk| chunk.length).sum::<u64>(),
            64
        );
        assert_ne!(node.tensor_hash, ts_core::Hash32::ZERO);
    }
}
