use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod publication;
pub use ed25519_dalek::SigningKey as ModelSigningKey;
pub use publication::{
    ArtifactVariant, EvaluationMetric, LicenseClaim, MetadataOrigin, MetadataValue,
    ModelArtifactFile, ModelCard, ModelDescriptorError, ModelReleaseDescriptor, ModelShareLink,
    ModelShareLinkError, ModelSignatureError, ShardPosition, SignedModelRelease,
    MAX_SIGNED_MODEL_RELEASE_BYTES, MODEL_CARD_SCHEMA_V1, MODEL_RELEASE_SCHEMA_V1,
    SIGNED_MODEL_RELEASE_SCHEMA_V1,
};

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
        assert_eq!(
            first.root.0,
            [
                0xfe, 0xce, 0x85, 0x9b, 0xc2, 0x16, 0xc1, 0xd8, 0x66, 0x36, 0xbc, 0x49, 0x75, 0x17,
                0x83, 0xb4, 0xd2, 0xf5, 0xba, 0x35, 0xda, 0x2f, 0x66, 0xab, 0x64, 0xdc, 0xad, 0xd5,
                0x00, 0x57, 0x83, 0x40,
            ]
        );
        let mut changed = first;
        changed.tensors[0].descriptor.byte_len += 1;
        assert!(!changed.verify_root());
    }

    #[test]
    fn model_release_descriptor_roundtrips_mixed_variants() {
        let mut card = ModelCard::new("Example model");
        card.publisher = Some("TensorSwarm fixtures".to_owned());
        card.summary = Some("A tiny test model".to_owned());
        card.description_markdown = Some("# Test card".to_owned());
        card.task = Some(MetadataValue::parsed("text-generation".to_owned()));
        card.architecture = Some(MetadataValue::declared("ExampleTransformer".to_owned()));
        card.parameter_count = Some(MetadataValue::parsed(70_000_000_000));
        card.context_length = Some(MetadataValue::declared(8192));
        card.languages = vec!["en".to_owned()];
        card.base_models = vec!["publisher/base-model".to_owned()];
        card.datasets = vec!["publisher/example-dataset".to_owned()];
        card.evaluation = vec![EvaluationMetric {
            name: "example-score".to_owned(),
            value: "0.75".to_owned(),
            dataset: Some("publisher/eval-set".to_owned()),
        }];
        card.intended_uses = vec!["testing".to_owned()];
        card.limitations = vec!["synthetic fixture".to_owned()];
        card.tags = vec!["example".to_owned(), "text-generation".to_owned()];
        card.license = Some(LicenseClaim {
            spdx_expression: Some("Apache-2.0".to_owned()),
            original_name: Some("Apache License 2.0".to_owned()),
            source_url: Some("https://example.test/LICENSE".to_owned()),
        });
        card.source_repository = Some("https://example.test/model".to_owned());
        card.source_revision = Some("abc123".to_owned());
        let descriptor = ModelReleaseDescriptor::new(
            card,
            "v1",
            vec![
                ArtifactVariant {
                    id: "q4-k-m".to_owned(),
                    format: ArtifactFormat::Gguf,
                    quantization: Some(MetadataValue::declared("Q4_K_M".to_owned())),
                    files: vec![
                        ModelArtifactFile {
                            path: "gguf/model-q4-00001-of-00002.gguf".to_owned(),
                            size_bytes: 1024,
                            tensor_count: 1,
                            manifest_root: sha256(b"gguf-root-1"),
                            shard: Some(ShardPosition { index: 0, count: 2 }),
                        },
                        ModelArtifactFile {
                            path: "gguf/model-q4-00002-of-00002.gguf".to_owned(),
                            size_bytes: 1024,
                            tensor_count: 1,
                            manifest_root: sha256(b"gguf-root-2"),
                            shard: Some(ShardPosition { index: 1, count: 2 }),
                        },
                    ],
                },
                ArtifactVariant {
                    id: "bf16".to_owned(),
                    format: ArtifactFormat::Safetensors,
                    quantization: Some(MetadataValue::declared("BF16".to_owned())),
                    files: vec![ModelArtifactFile {
                        path: "safetensors/model-00001-of-00001.safetensors".to_owned(),
                        size_bytes: 2048,
                        tensor_count: 2,
                        manifest_root: sha256(b"safetensors-root"),
                        shard: Some(ShardPosition { index: 0, count: 1 }),
                    }],
                },
            ],
        );

        descriptor.validate().unwrap();
        let encoded = serde_cbor::to_vec(&descriptor).unwrap();
        let decoded: ModelReleaseDescriptor = serde_cbor::from_slice(&encoded).unwrap();
        assert_eq!(decoded, descriptor);
        assert_eq!(decoded.variants[0].format, ArtifactFormat::Gguf);
        assert_eq!(decoded.variants[1].format, ArtifactFormat::Safetensors);
        assert_eq!(
            decoded.variants[0].quantization.as_ref().unwrap().origin,
            MetadataOrigin::PublisherDeclared
        );
        assert_eq!(
            decoded.card.task.as_ref().unwrap().origin,
            MetadataOrigin::ParsedFromArtifact
        );
    }

    #[test]
    fn model_release_descriptor_rejects_unsafe_paths() {
        let mut descriptor = ModelReleaseDescriptor::new(
            ModelCard::new("Example model"),
            "v1",
            vec![ArtifactVariant {
                id: "main".to_owned(),
                format: ArtifactFormat::Gguf,
                quantization: None,
                files: vec![ModelArtifactFile {
                    path: "../outside.gguf".to_owned(),
                    size_bytes: 1024,
                    tensor_count: 1,
                    manifest_root: sha256(b"model-root"),
                    shard: None,
                }],
            }],
        );

        assert!(matches!(
            descriptor.validate(),
            Err(ModelDescriptorError::InvalidArtifactPath { .. })
        ));
        descriptor.variants[0].files[0].path = "model.gguf".to_owned();
        descriptor.variants[0].files[0].shard = Some(ShardPosition { index: 1, count: 1 });
        assert!(matches!(
            descriptor.validate(),
            Err(ModelDescriptorError::InvalidShardPosition { .. })
        ));
        descriptor.variants[0].files[0].shard = Some(ShardPosition { index: 0, count: 2 });
        assert!(matches!(
            descriptor.validate(),
            Err(ModelDescriptorError::IncompleteShardGroup { .. })
        ));
        descriptor.schema = MODEL_RELEASE_SCHEMA_V1 + 1;
        assert!(matches!(
            descriptor.validate(),
            Err(ModelDescriptorError::UnsupportedModelReleaseSchema(_))
        ));
    }

    #[test]
    fn model_card_rejects_unknown_fields() {
        let mut value = serde_cbor::value::to_value(ModelCard::new("Example model")).unwrap();
        let serde_cbor::Value::Map(fields) = &mut value else {
            panic!("model card must serialize as a map");
        };
        fields.insert(
            serde_cbor::Value::Text("future_field".to_owned()),
            serde_cbor::Value::Bool(true),
        );
        let encoded = serde_cbor::to_vec(&value).unwrap();

        assert!(serde_cbor::from_slice::<ModelCard>(&encoded).is_err());
    }
}
