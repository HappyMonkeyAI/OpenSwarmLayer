use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MANIFEST_SCHEMA_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub const ZERO: Self = Self([0; 32]);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArtifactFormat {
    Gguf,
    Safetensors,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TensorDescriptor {
    pub name: String,
    pub shape: Vec<u64>,
    pub dtype: String,
    pub byte_len: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChunkRef {
    pub index: u32,
    pub offset: u64,
    pub length: u64,
    pub hash: Hash32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TensorNode {
    pub descriptor: TensorDescriptor,
    pub tensor_hash: Hash32,
    pub chunks: Vec<ChunkRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Segment {
    Literal {
        offset: u64,
        bytes: Vec<u8>,
    },
    Tensor {
        offset: u64,
        tensor_hash: Hash32,
        tensor_offset: u64,
        length: u64,
    },
    ZeroFill {
        offset: u64,
        length: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileRecipe {
    pub path: String,
    pub format: ArtifactFormat,
    pub file_size: u64,
    pub segments: Vec<Segment>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub format: ArtifactFormat,
    pub file_size: u64,
    pub tensors: Vec<TensorNode>,
    pub files: Vec<FileRecipe>,
    pub root: Hash32,
}

impl Manifest {
    pub fn new(
        format: ArtifactFormat,
        file_size: u64,
        mut tensors: Vec<TensorNode>,
        files: Vec<FileRecipe>,
    ) -> Self {
        tensors.sort_by(|left, right| left.descriptor.name.cmp(&right.descriptor.name));
        let mut manifest = Self {
            schema: MANIFEST_SCHEMA_V1,
            format,
            file_size,
            tensors,
            files,
            root: Hash32::ZERO,
        };
        manifest.root = manifest.compute_root();
        manifest
    }

    pub fn compute_root(&self) -> Hash32 {
        let bytes = serde_cbor::to_vec(&(
            self.schema,
            &self.format,
            self.file_size,
            &self.tensors,
            &self.files,
        ))
        .expect("manifest identity serialization cannot fail");
        sha256(bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_cbor::to_vec(self).expect("manifest serialization cannot fail")
    }

    pub fn verify_root(&self) -> bool {
        self.root == self.compute_root()
    }
}

pub fn sha256(bytes: impl AsRef<[u8]>) -> Hash32 {
    let mut digest = Sha256::new();
    digest.update(bytes.as_ref());
    Hash32(digest.finalize().into())
}

pub struct HashWriter(Sha256);

impl Default for HashWriter {
    fn default() -> Self {
        Self(Sha256::new())
    }
}

impl HashWriter {
    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
    pub fn finalize(self) -> Hash32 {
        Hash32(self.0.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_root_is_order_independent_and_detects_mutation() {
        let tensor_a = TensorNode {
            descriptor: TensorDescriptor {
                name: "a".into(),
                shape: vec![2],
                dtype: "F32".into(),
                byte_len: 8,
            },
            tensor_hash: sha256(b"a"),
            chunks: vec![],
        };
        let tensor_b = TensorNode {
            descriptor: TensorDescriptor {
                name: "b".into(),
                shape: vec![2],
                dtype: "F32".into(),
                byte_len: 8,
            },
            tensor_hash: sha256(b"b"),
            chunks: vec![],
        };
        let recipe = FileRecipe {
            path: "model.bin".into(),
            format: ArtifactFormat::Gguf,
            file_size: 0,
            segments: vec![],
        };
        let first = Manifest::new(
            ArtifactFormat::Gguf,
            0,
            vec![tensor_a.clone(), tensor_b.clone()],
            vec![recipe.clone()],
        );
        let second = Manifest::new(
            ArtifactFormat::Gguf,
            0,
            vec![tensor_b, tensor_a],
            vec![recipe],
        );
        assert_eq!(first.root, second.root);
        assert!(first.verify_root());
        let mut changed = first;
        changed.tensors[0].descriptor.byte_len += 1;
        assert!(!changed.verify_root());
    }
}
