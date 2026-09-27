use std::collections::HashSet;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{ArtifactFormat, Hash32, Manifest};

pub const MODEL_CARD_SCHEMA_V1: u32 = 1;
pub const MODEL_RELEASE_SCHEMA_V1: u32 = 1;

/// How a descriptive value was obtained. For remote descriptors, this label is
/// untrusted metadata until the client independently verifies the value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MetadataOrigin {
    ParsedFromArtifact,
    PublisherDeclared,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataValue<T> {
    pub value: T,
    pub origin: MetadataOrigin,
}

impl<T> MetadataValue<T> {
    pub fn parsed(value: T) -> Self {
        Self {
            value,
            origin: MetadataOrigin::ParsedFromArtifact,
        }
    }

    pub fn declared(value: T) -> Self {
        Self {
            value,
            origin: MetadataOrigin::PublisherDeclared,
        }
    }
}

/// Publisher-provided license information; this is not a rights determination.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LicenseClaim {
    pub spdx_expression: Option<String>,
    pub original_name: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationMetric {
    pub name: String,
    pub value: String,
    pub dataset: Option<String>,
}

/// Human-authored information about a logical model, separate from artifact
/// identity. Catalog review/moderation status must be stored separately and may
/// not be asserted by this publisher-controlled card.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCard {
    pub schema: u32,
    pub name: String,
    pub publisher: Option<String>,
    pub summary: Option<String>,
    pub description_markdown: Option<String>,
    pub task: Option<MetadataValue<String>>,
    pub architecture: Option<MetadataValue<String>>,
    pub parameter_count: Option<MetadataValue<u64>>,
    pub context_length: Option<MetadataValue<u32>>,
    pub languages: Vec<String>,
    pub base_models: Vec<String>,
    pub datasets: Vec<String>,
    pub evaluation: Vec<EvaluationMetric>,
    pub intended_uses: Vec<String>,
    pub limitations: Vec<String>,
    pub tags: Vec<String>,
    pub license: Option<LicenseClaim>,
    pub source_repository: Option<String>,
    pub source_revision: Option<String>,
}

impl ModelCard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: MODEL_CARD_SCHEMA_V1,
            name: name.into(),
            publisher: None,
            summary: None,
            description_markdown: None,
            task: None,
            architecture: None,
            parameter_count: None,
            context_length: None,
            languages: Vec::new(),
            base_models: Vec::new(),
            datasets: Vec::new(),
            evaluation: Vec::new(),
            intended_uses: Vec::new(),
            limitations: Vec::new(),
            tags: Vec::new(),
            license: None,
            source_repository: None,
            source_revision: None,
        }
    }

    pub fn validate(&self) -> Result<(), ModelDescriptorError> {
        if self.schema != MODEL_CARD_SCHEMA_V1 {
            return Err(ModelDescriptorError::UnsupportedModelCardSchema(
                self.schema,
            ));
        }
        if self.name.trim().is_empty() {
            return Err(ModelDescriptorError::EmptyModelName);
        }
        Ok(())
    }
}

/// Zero-based shard index and total shard count for one artifact file.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardPosition {
    pub index: u32,
    pub count: u32,
}

/// Untrusted catalog reference. Compare its size, tensor count, format, and
/// root with the fetched and root-verified content manifest before trusting it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelArtifactFile {
    /// Normalized, release-relative path using `/` separators.
    pub path: String,
    pub size_bytes: u64,
    pub tensor_count: u64,
    pub manifest_root: Hash32,
    pub shard: Option<ShardPosition>,
}

impl ModelArtifactFile {
    /// Compare a publisher claim with a locally root-verified single-file
    /// manifest. This does not verify the artifact bytes or publisher rights.
    pub fn matches_manifest(&self, manifest: &Manifest, format: ArtifactFormat) -> bool {
        manifest.verify_root()
            && manifest.root == self.manifest_root
            && manifest.format == format
            && manifest.files.len() == 1
            && manifest.files.first().is_some_and(|file| {
                file.path == self.path && file.file_size == self.size_bytes && file.format == format
            })
            && u64::try_from(manifest.tensors.len()).ok() == Some(self.tensor_count)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactVariant {
    /// Publisher-selected stable label within this release, e.g. `q4-k-m`.
    pub id: String,
    pub format: ArtifactFormat,
    pub quantization: Option<MetadataValue<String>>,
    pub files: Vec<ModelArtifactFile>,
}

/// Versioned descriptive release record. Its artifact roots refer to existing
/// content manifests; card metadata does not alter those manifest identities.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelReleaseDescriptor {
    pub schema: u32,
    pub release: String,
    pub card: ModelCard,
    pub variants: Vec<ArtifactVariant>,
}

impl ModelReleaseDescriptor {
    pub fn new(
        card: ModelCard,
        release: impl Into<String>,
        variants: Vec<ArtifactVariant>,
    ) -> Self {
        Self {
            schema: MODEL_RELEASE_SCHEMA_V1,
            release: release.into(),
            card,
            variants,
        }
    }

    pub fn validate(&self) -> Result<(), ModelDescriptorError> {
        if self.schema != MODEL_RELEASE_SCHEMA_V1 {
            return Err(ModelDescriptorError::UnsupportedModelReleaseSchema(
                self.schema,
            ));
        }
        self.card.validate()?;
        if self.release.trim().is_empty() {
            return Err(ModelDescriptorError::EmptyReleaseLabel);
        }
        if self.variants.is_empty() {
            return Err(ModelDescriptorError::NoArtifactVariants);
        }

        let mut variant_ids = HashSet::new();
        for variant in &self.variants {
            if variant.id.trim().is_empty() {
                return Err(ModelDescriptorError::EmptyVariantId);
            }
            if !variant_ids.insert(&variant.id) {
                return Err(ModelDescriptorError::DuplicateVariantId(variant.id.clone()));
            }
            if variant.files.is_empty() {
                return Err(ModelDescriptorError::NoFilesForVariant(variant.id.clone()));
            }

            let mut paths = HashSet::new();
            let mut shard_count = None;
            let mut shard_indices = HashSet::new();
            let mut has_unsharded_file = false;
            for file in &variant.files {
                if !is_safe_relative_path(&file.path) {
                    return Err(ModelDescriptorError::InvalidArtifactPath {
                        path: file.path.clone(),
                    });
                }
                if !paths.insert(&file.path) {
                    return Err(ModelDescriptorError::DuplicateArtifactPath {
                        variant: variant.id.clone(),
                        path: file.path.clone(),
                    });
                }
                if file.size_bytes == 0 {
                    return Err(ModelDescriptorError::EmptyArtifactFile {
                        path: file.path.clone(),
                    });
                }
                if file.tensor_count == 0 {
                    return Err(ModelDescriptorError::ArtifactHasNoTensors {
                        path: file.path.clone(),
                    });
                }
                if file.manifest_root == Hash32::ZERO {
                    return Err(ModelDescriptorError::EmptyManifestRoot {
                        path: file.path.clone(),
                    });
                }
                if let Some(shard) = file.shard {
                    if shard.count == 0 || shard.index >= shard.count {
                        return Err(ModelDescriptorError::InvalidShardPosition {
                            path: file.path.clone(),
                            index: shard.index,
                            count: shard.count,
                        });
                    }
                    if let Some(expected_count) = shard_count {
                        if expected_count != shard.count {
                            return Err(ModelDescriptorError::ShardCountMismatch {
                                variant: variant.id.clone(),
                                expected: expected_count,
                                actual: shard.count,
                            });
                        }
                    } else {
                        shard_count = Some(shard.count);
                    }
                    if !shard_indices.insert(shard.index) {
                        return Err(ModelDescriptorError::DuplicateShardIndex {
                            variant: variant.id.clone(),
                            index: shard.index,
                        });
                    }
                } else {
                    has_unsharded_file = true;
                }
            }
            if let Some(count) = shard_count {
                if has_unsharded_file {
                    return Err(ModelDescriptorError::InconsistentShardGroup {
                        variant: variant.id.clone(),
                    });
                }
                if usize::try_from(count).ok() != Some(variant.files.len()) {
                    return Err(ModelDescriptorError::IncompleteShardGroup {
                        variant: variant.id.clone(),
                        expected: count,
                        actual: variant.files.len(),
                    });
                }
            }
        }
        Ok(())
    }
}

pub const SIGNED_MODEL_RELEASE_SCHEMA_V1: u32 = 1;
pub const MAX_SIGNED_MODEL_RELEASE_BYTES: usize = 4 * 1024 * 1024;
const SIGNED_RELEASE_DOMAIN: &[u8] = b"TensorSwarm signed model release v1\0";

/// An Ed25519-signed publisher descriptor. The self-asserted key proves only
/// that the same key signed these exact descriptor bytes; it does not establish
/// a real-world publisher identity, redistribution rights, or artifact safety.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedModelRelease {
    pub schema: u32,
    pub descriptor: ModelReleaseDescriptor,
    pub signer_public_key: [u8; 32],
    pub signature: Vec<u8>,
}

impl SignedModelRelease {
    /// Sign with a caller-managed key. This method neither generates nor stores
    /// the private key; callers must obtain it from a secure local key source.
    pub fn sign(
        descriptor: ModelReleaseDescriptor,
        signing_key: &SigningKey,
    ) -> Result<Self, ModelSignatureError> {
        descriptor.validate()?;
        let signer_public_key = signing_key.verifying_key().to_bytes();
        let mut signed = Self {
            schema: SIGNED_MODEL_RELEASE_SCHEMA_V1,
            descriptor,
            signer_public_key,
            signature: Vec::new(),
        };
        signed.signature = signing_key
            .sign(&signed.signature_payload()?)
            .to_bytes()
            .to_vec();
        Ok(signed)
    }

    pub fn verify(&self) -> Result<(), ModelSignatureError> {
        if self.schema != SIGNED_MODEL_RELEASE_SCHEMA_V1 {
            return Err(ModelSignatureError::UnsupportedSignedReleaseSchema(
                self.schema,
            ));
        }
        self.descriptor.validate()?;
        let verifying_key = VerifyingKey::from_bytes(&self.signer_public_key)
            .map_err(|_| ModelSignatureError::InvalidPublicKey)?;
        let signature_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| ModelSignatureError::InvalidSignatureLength)?;
        let signature = Signature::from_bytes(&signature_bytes);
        verifying_key
            .verify_strict(&self.signature_payload()?, &signature)
            .map_err(|_| ModelSignatureError::InvalidSignature)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ModelSignatureError> {
        self.verify()?;
        let bytes = serde_cbor::to_vec(self).map_err(|_| ModelSignatureError::EncodingFailure)?;
        if bytes.len() > MAX_SIGNED_MODEL_RELEASE_BYTES {
            return Err(ModelSignatureError::DescriptorTooLarge);
        }
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ModelSignatureError> {
        if bytes.len() > MAX_SIGNED_MODEL_RELEASE_BYTES {
            return Err(ModelSignatureError::DescriptorTooLarge);
        }
        let signed: Self = serde_cbor::from_slice(bytes)
            .map_err(|_| ModelSignatureError::InvalidDescriptorEncoding)?;
        signed.verify()?;
        Ok(signed)
    }

    /// Stable identifier of this exact signed envelope, not of its model bytes.
    pub fn release_id(&self) -> Result<Hash32, ModelSignatureError> {
        let bytes = self.to_bytes()?;
        Ok(Hash32(Sha256::digest(bytes).into()))
    }

    fn signature_payload(&self) -> Result<Vec<u8>, ModelSignatureError> {
        let descriptor = serde_cbor::to_vec(&self.descriptor)
            .map_err(|_| ModelSignatureError::EncodingFailure)?;
        let length =
            u64::try_from(descriptor.len()).map_err(|_| ModelSignatureError::DescriptorTooLarge)?;
        let mut payload = Vec::with_capacity(
            SIGNED_RELEASE_DOMAIN.len() + 4 + self.signer_public_key.len() + 8 + descriptor.len(),
        );
        payload.extend_from_slice(SIGNED_RELEASE_DOMAIN);
        payload.extend_from_slice(&self.schema.to_be_bytes());
        payload.extend_from_slice(&self.signer_public_key);
        payload.extend_from_slice(&length.to_be_bytes());
        payload.extend_from_slice(&descriptor);
        Ok(payload)
    }
}

/// Local-only, immutable release identifier URI. It identifies an exported
/// signed descriptor but does not resolve or fetch it without a future service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelShareLink {
    descriptor_id: Hash32,
}

impl ModelShareLink {
    pub fn for_release(release: &SignedModelRelease) -> Result<Self, ModelSignatureError> {
        Ok(Self {
            descriptor_id: release.release_id()?,
        })
    }

    pub fn descriptor_id(&self) -> Hash32 {
        self.descriptor_id
    }

    pub fn to_uri(&self) -> String {
        let mut uri = String::from("tswarm://v1/");
        for byte in self.descriptor_id.0 {
            use std::fmt::Write;
            let _ = write!(uri, "{byte:02x}");
        }
        uri
    }

    pub fn parse(uri: &str) -> Result<Self, ModelShareLinkError> {
        const PREFIX: &str = "tswarm://v1/";
        if !uri.starts_with(PREFIX) || uri.len() != PREFIX.len() + 64 {
            return Err(ModelShareLinkError::InvalidSyntax);
        }
        let encoded = &uri[PREFIX.len()..];
        let mut digest = [0; 32];
        for (index, byte) in digest.iter_mut().enumerate() {
            let offset = index * 2;
            let high =
                hex_nibble(encoded.as_bytes()[offset]).ok_or(ModelShareLinkError::InvalidSyntax)?;
            let low = hex_nibble(encoded.as_bytes()[offset + 1])
                .ok_or(ModelShareLinkError::InvalidSyntax)?;
            *byte = (high << 4) | low;
        }
        Ok(Self {
            descriptor_id: Hash32(digest),
        })
    }

    pub fn verify_release(&self, release: &SignedModelRelease) -> Result<(), ModelShareLinkError> {
        if self.descriptor_id != release.release_id()? {
            return Err(ModelShareLinkError::ReleaseIdMismatch);
        }
        Ok(())
    }
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ModelSignatureError {
    #[error(transparent)]
    Descriptor(#[from] ModelDescriptorError),
    #[error("unsupported signed model-release schema: {0}")]
    UnsupportedSignedReleaseSchema(u32),
    #[error("invalid Ed25519 public key")]
    InvalidPublicKey,
    #[error("invalid Ed25519 signature length")]
    InvalidSignatureLength,
    #[error("Ed25519 signature verification failed")]
    InvalidSignature,
    #[error("signed model release exceeds the size limit")]
    DescriptorTooLarge,
    #[error("signed model release could not be encoded")]
    EncodingFailure,
    #[error("signed model release has invalid encoding")]
    InvalidDescriptorEncoding,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ModelShareLinkError {
    #[error("invalid TensorSwarm share-link syntax")]
    InvalidSyntax,
    #[error("share-link identifier does not match the signed release")]
    ReleaseIdMismatch,
    #[error(transparent)]
    Signature(#[from] ModelSignatureError),
}

fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

#[cfg(test)]
mod signed_release_tests {
    use ed25519_dalek::SigningKey;

    use super::{
        ArtifactVariant, ModelArtifactFile, ModelCard, ModelReleaseDescriptor, ModelShareLink,
        SignedModelRelease,
    };
    use crate::{ArtifactFormat, Hash32};

    fn descriptor() -> ModelReleaseDescriptor {
        ModelReleaseDescriptor::new(
            ModelCard::new("small-test-model"),
            "v1",
            vec![ArtifactVariant {
                id: "q4".into(),
                format: ArtifactFormat::Gguf,
                quantization: None,
                files: vec![ModelArtifactFile {
                    path: "model-q4.gguf".into(),
                    size_bytes: 4096,
                    tensor_count: 3,
                    manifest_root: Hash32([9; 32]),
                    shard: None,
                }],
            }],
        )
    }

    #[test]
    fn signed_release_round_trips_and_share_link_binds_the_envelope() {
        let key = SigningKey::from_bytes(&[7; 32]);
        let signed = SignedModelRelease::sign(descriptor(), &key).unwrap();
        let bytes = signed.to_bytes().unwrap();
        let imported = SignedModelRelease::from_bytes(&bytes).unwrap();
        assert_eq!(imported, signed);

        let link = ModelShareLink::for_release(&imported).unwrap();
        assert_eq!(ModelShareLink::parse(&link.to_uri()).unwrap(), link);
        link.verify_release(&imported).unwrap();
    }

    #[test]
    fn signed_release_rejects_mutated_descriptor_and_signature() {
        let key = SigningKey::from_bytes(&[8; 32]);
        let signed = SignedModelRelease::sign(descriptor(), &key).unwrap();

        let mut changed_descriptor = signed.clone();
        changed_descriptor.descriptor.card.name = "substituted-model".into();
        assert!(changed_descriptor.verify().is_err());

        let mut changed_root = signed.clone();
        changed_root.descriptor.variants[0].files[0].manifest_root = Hash32([12; 32]);
        assert!(changed_root.verify().is_err());

        let mut changed_signature = signed;
        changed_signature.signature[0] ^= 1;
        assert!(changed_signature.verify().is_err());

        let mut wrong_public_key =
            SignedModelRelease::sign(descriptor(), &SigningKey::from_bytes(&[14; 32])).unwrap();
        wrong_public_key.signer_public_key =
            SigningKey::from_bytes(&[15; 32]).verifying_key().to_bytes();
        assert!(wrong_public_key.verify().is_err());
    }

    #[test]
    fn signed_release_rejects_unsupported_schema_and_oversized_input() {
        let key = SigningKey::from_bytes(&[9; 32]);
        let mut signed = SignedModelRelease::sign(descriptor(), &key).unwrap();
        signed.schema += 1;
        assert!(signed.verify().is_err());

        assert!(SignedModelRelease::from_bytes(&vec![0; 4 * 1024 * 1024 + 1]).is_err());
    }

    #[test]
    fn signed_envelope_rejects_unknown_fields_and_bad_signature_shape() {
        let key = SigningKey::from_bytes(&[12; 32]);
        let signed = SignedModelRelease::sign(descriptor(), &key).unwrap();

        let mut value = serde_cbor::value::to_value(&signed).unwrap();
        let serde_cbor::Value::Map(fields) = &mut value else {
            panic!("signed release must serialize as a map");
        };
        fields.insert(
            serde_cbor::Value::Text("future_field".to_owned()),
            serde_cbor::Value::Bool(true),
        );
        let encoded = serde_cbor::to_vec(&value).unwrap();
        assert!(SignedModelRelease::from_bytes(&encoded).is_err());

        let mut invalid_signature = signed;
        invalid_signature.signature.pop();
        assert!(invalid_signature.verify().is_err());
    }

    #[test]
    fn share_link_rejects_malformed_or_nonmatching_envelope() {
        let first =
            SignedModelRelease::sign(descriptor(), &SigningKey::from_bytes(&[10; 32])).unwrap();
        let second =
            SignedModelRelease::sign(descriptor(), &SigningKey::from_bytes(&[11; 32])).unwrap();
        let link = ModelShareLink::for_release(&first).unwrap();
        assert!(ModelShareLink::parse("tswarm://v1/not-a-digest").is_err());
        assert!(link.verify_release(&second).is_err());
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ModelDescriptorError {
    #[error("unsupported model-card schema: {0}")]
    UnsupportedModelCardSchema(u32),
    #[error("unsupported model-release schema: {0}")]
    UnsupportedModelReleaseSchema(u32),
    #[error("model name cannot be empty")]
    EmptyModelName,
    #[error("release label cannot be empty")]
    EmptyReleaseLabel,
    #[error("release must contain at least one artifact variant")]
    NoArtifactVariants,
    #[error("artifact variant id cannot be empty")]
    EmptyVariantId,
    #[error("duplicate artifact variant id: {0}")]
    DuplicateVariantId(String),
    #[error("artifact variant has no files: {0}")]
    NoFilesForVariant(String),
    #[error("artifact path is not a safe relative path: {path}")]
    InvalidArtifactPath { path: String },
    #[error("duplicate artifact path in variant {variant}: {path}")]
    DuplicateArtifactPath { variant: String, path: String },
    #[error("artifact file has zero size: {path}")]
    EmptyArtifactFile { path: String },
    #[error("artifact file has no tensors: {path}")]
    ArtifactHasNoTensors { path: String },
    #[error("artifact has an empty manifest root: {path}")]
    EmptyManifestRoot { path: String },
    #[error("invalid shard position for {path}: index {index}, count {count}")]
    InvalidShardPosition {
        path: String,
        index: u32,
        count: u32,
    },
    #[error("shard count mismatch in variant {variant}: expected {expected}, got {actual}")]
    ShardCountMismatch {
        variant: String,
        expected: u32,
        actual: u32,
    },
    #[error("duplicate shard index in variant {variant}: {index}")]
    DuplicateShardIndex { variant: String, index: u32 },
    #[error("sharded and unsharded files are mixed in variant {variant}")]
    InconsistentShardGroup { variant: String },
    #[error("incomplete shard group in variant {variant}: expected {expected}, got {actual}")]
    IncompleteShardGroup {
        variant: String,
        expected: u32,
        actual: usize,
    },
}
