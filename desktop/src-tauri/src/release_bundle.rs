use anyhow::{ensure, Context};
use serde::Serialize;
use ts_core::{
    ArtifactVariant, LicenseClaim, Manifest, MetadataValue, ModelArtifactFile, ModelCard,
    ModelReleaseDescriptor, ModelShareLink, SignedModelRelease,
};

use crate::signing_identity;

const MODEL_NAME_MAX_BYTES: usize = 200;
const SUMMARY_MAX_BYTES: usize = 4_096;
const RELEASE_LABEL_MAX_BYTES: usize = 100;
const QUANTIZATION_MAX_BYTES: usize = 128;
const LICENSE_NAME_MAX_BYTES: usize = 500;
const LICENSE_SPDX_MAX_BYTES: usize = 1_000;
const CLAIM_URL_MAX_BYTES: usize = 2_048;
const REPOSITORY_URL_MAX_BYTES: usize = 2_048;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReleaseExportInput {
    pub model_path: String,
    pub model_name: String,
    pub summary: Option<String>,
    pub release_label: String,
    pub quantization: Option<String>,
    pub license_name: Option<String>,
    pub license_spdx: Option<String>,
    pub license_source_url: Option<String>,
    pub source_repository: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SignedReleaseSummary {
    pub model_name: String,
    pub release_label: String,
    pub signer_fingerprint: String,
    pub share_uri: String,
    pub file_count: usize,
    pub local_manifest_matches: usize,
}

pub(crate) fn descriptor_for_manifest(
    manifest: &Manifest,
    input: &ReleaseExportInput,
) -> anyhow::Result<ModelReleaseDescriptor> {
    ensure!(
        manifest.verify_root(),
        "local model manifest root is invalid"
    );
    ensure!(
        manifest.files.len() == 1,
        "selected local manifest must describe one file"
    );
    let recipe = manifest
        .files
        .first()
        .context("local manifest has no file")?;
    ensure!(
        recipe.path == input.model_path,
        "selected path does not match the local manifest"
    );

    validate_text("model name", &input.model_name, MODEL_NAME_MAX_BYTES, true)?;
    validate_text(
        "release label",
        &input.release_label,
        RELEASE_LABEL_MAX_BYTES,
        true,
    )?;
    validate_optional_text("summary", input.summary.as_deref(), SUMMARY_MAX_BYTES)?;
    validate_optional_text(
        "quantization",
        input.quantization.as_deref(),
        QUANTIZATION_MAX_BYTES,
    )?;
    validate_optional_text(
        "license name",
        input.license_name.as_deref(),
        LICENSE_NAME_MAX_BYTES,
    )?;
    validate_optional_text(
        "SPDX claim",
        input.license_spdx.as_deref(),
        LICENSE_SPDX_MAX_BYTES,
    )?;
    validate_optional_text(
        "license source URL",
        input.license_source_url.as_deref(),
        CLAIM_URL_MAX_BYTES,
    )?;
    validate_optional_text(
        "source repository",
        input.source_repository.as_deref(),
        REPOSITORY_URL_MAX_BYTES,
    )?;

    let tensor_count = u64::try_from(manifest.tensors.len())
        .context("local manifest tensor count is too large")?;
    let artifact_file = ModelArtifactFile {
        path: recipe.path.clone(),
        size_bytes: recipe.file_size,
        tensor_count,
        manifest_root: manifest.root,
        shard: None,
    };

    let mut card = ModelCard::new(input.model_name.trim());
    card.summary = input.summary.as_deref().and_then(nonempty);
    card.source_repository = input.source_repository.as_deref().and_then(nonempty);
    if input.license_name.as_deref().and_then(nonempty).is_some()
        || input.license_spdx.as_deref().and_then(nonempty).is_some()
        || input
            .license_source_url
            .as_deref()
            .and_then(nonempty)
            .is_some()
    {
        card.license = Some(LicenseClaim {
            spdx_expression: input.license_spdx.as_deref().and_then(nonempty),
            original_name: input.license_name.as_deref().and_then(nonempty),
            source_url: input.license_source_url.as_deref().and_then(nonempty),
        });
    }

    let variant = ArtifactVariant {
        id: "default".to_owned(),
        format: recipe.format,
        quantization: input
            .quantization
            .as_deref()
            .and_then(nonempty)
            .map(MetadataValue::declared),
        files: vec![artifact_file],
    };
    let descriptor = ModelReleaseDescriptor::new(card, input.release_label.trim(), vec![variant]);
    descriptor.validate()?;
    Ok(descriptor)
}

pub(crate) fn summarize(
    release: &SignedModelRelease,
    local_manifests: &[Manifest],
) -> anyhow::Result<SignedReleaseSummary> {
    release.verify()?;
    let files = release
        .descriptor
        .variants
        .iter()
        .flat_map(|variant| {
            variant
                .files
                .iter()
                .map(move |artifact| (artifact, variant.format))
        })
        .collect::<Vec<_>>();
    let local_manifest_matches = files
        .iter()
        .filter(|(artifact, format)| {
            local_manifests
                .iter()
                .any(|manifest| manifest_matches_file(manifest, artifact, *format))
        })
        .count();
    let share_uri = ModelShareLink::for_release(release)?.to_uri();
    Ok(SignedReleaseSummary {
        model_name: release.descriptor.card.name.clone(),
        release_label: release.descriptor.release.clone(),
        signer_fingerprint: signing_identity::fingerprint_public_key(&release.signer_public_key),
        share_uri,
        file_count: files.len(),
        local_manifest_matches,
    })
}

fn manifest_matches_file(
    manifest: &Manifest,
    artifact: &ModelArtifactFile,
    format: ts_core::ArtifactFormat,
) -> bool {
    artifact.matches_manifest(manifest, format)
}

fn nonempty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn validate_text(name: &str, value: &str, max_bytes: usize, required: bool) -> anyhow::Result<()> {
    ensure!(value.len() <= max_bytes, "{name} exceeds {max_bytes} bytes");
    ensure!(!required || !value.trim().is_empty(), "{name} is required");
    Ok(())
}

fn validate_optional_text(name: &str, value: Option<&str>, max_bytes: usize) -> anyhow::Result<()> {
    if let Some(value) = value {
        validate_text(name, value, max_bytes, false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{descriptor_for_manifest, summarize, ReleaseExportInput};
    use std::io::Write;
    use tempfile::tempdir;
    use ts_core::{ArtifactFormat, Manifest, ModelSigningKey, SignedModelRelease};

    fn write_safetensors(path: &std::path::Path, payload: &[u8]) {
        let header = format!(
            "{{\"weight\":{{\"dtype\":\"U8\",\"shape\":[{}],\"data_offsets\":[0,{}]}}}}",
            payload.len(),
            payload.len()
        );
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(&(header.len() as u64).to_le_bytes())
            .unwrap();
        file.write_all(header.as_bytes()).unwrap();
        file.write_all(payload).unwrap();
    }

    fn input(path: &str) -> ReleaseExportInput {
        ReleaseExportInput {
            model_path: path.to_owned(),
            model_name: "Example model".to_owned(),
            summary: Some("Local summary".to_owned()),
            release_label: "v1".to_owned(),
            quantization: Some("Q4_K_M".to_owned()),
            license_name: Some("Publisher claim".to_owned()),
            license_spdx: Some("Apache-2.0".to_owned()),
            license_source_url: None,
            source_repository: None,
        }
    }

    #[test]
    fn descriptor_binds_claims_to_the_exact_local_manifest() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("example.safetensors");
        write_safetensors(&source, b"verified payload");
        let manifest = ts_format::build_manifest(&source, 16 * 1024 * 1024).unwrap();

        let descriptor = descriptor_for_manifest(&manifest, &input("example.safetensors")).unwrap();
        let artifact = &descriptor.variants[0].files[0];
        assert_eq!(artifact.manifest_root, manifest.root);
        assert_eq!(artifact.size_bytes, manifest.file_size);
        assert_eq!(artifact.tensor_count, manifest.tensors.len() as u64);
        assert_eq!(descriptor.variants[0].format, ArtifactFormat::Safetensors);
        assert_eq!(
            descriptor
                .card
                .license
                .as_ref()
                .unwrap()
                .spdx_expression
                .as_deref(),
            Some("Apache-2.0")
        );
        assert_eq!(
            descriptor.variants[0].quantization.as_ref().unwrap().value,
            "Q4_K_M"
        );
    }

    #[test]
    fn descriptor_creation_rejects_wrong_paths_and_oversized_claims() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("example.safetensors");
        write_safetensors(&source, b"verified payload");
        let manifest = ts_format::build_manifest(&source, 16 * 1024 * 1024).unwrap();

        assert!(descriptor_for_manifest(&manifest, &input("other.safetensors")).is_err());
        let mut too_long = input("example.safetensors");
        too_long.summary = Some("x".repeat(4_097));
        assert!(descriptor_for_manifest(&manifest, &too_long).is_err());
    }

    #[test]
    fn release_summary_verifies_signature_and_matches_local_roots_strictly() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("example.safetensors");
        write_safetensors(&source, b"verified payload");
        let manifest: Manifest = ts_format::build_manifest(&source, 16 * 1024 * 1024).unwrap();
        let descriptor = descriptor_for_manifest(&manifest, &input("example.safetensors")).unwrap();
        let key = ModelSigningKey::from_bytes(&[7; 32]);
        let signed = SignedModelRelease::sign(descriptor, &key).unwrap();

        let matching = summarize(&signed, std::slice::from_ref(&manifest)).unwrap();
        assert_eq!(matching.file_count, 1);
        assert_eq!(matching.local_manifest_matches, 1);
        assert!(matching.share_uri.starts_with("tswarm://v1/"));

        let mut mismatched = manifest.clone();
        mismatched.files[0].file_size += 1;
        mismatched.root = mismatched.compute_root();
        let not_matching = summarize(&signed, &[mismatched]).unwrap();
        assert_eq!(not_matching.local_manifest_matches, 0);
    }
}
