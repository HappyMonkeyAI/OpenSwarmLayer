//! Desktop orchestration over the daemon's private-download primitives.

use super::*;
use ts_daemon::download::{self, CancellationToken, DownloadProgress};

#[derive(Clone, serde::Serialize)]
pub(crate) struct DownloadStatus {
    phase: String,
    share_uri: String,
    message: String,
    output: Option<String>,
    model_path: Option<String>,
    progress: DownloadProgress,
}

impl Default for DownloadStatus {
    fn default() -> Self {
        Self {
            phase: "idle".into(),
            share_uri: String::new(),
            message: String::new(),
            output: None,
            model_path: None,
            progress: DownloadProgress::default(),
        }
    }
}

#[derive(Default)]
pub(crate) struct DownloadManager(Mutex<(DownloadStatus, CancellationToken)>);

impl DownloadManager {
    fn update(&self, phase: &str, message: impl Into<String>) {
        if let Ok(mut job) = self.0.lock() {
            job.0.phase = phase.into();
            job.0.message = message.into();
        }
    }

    fn begin(&self, share_uri: String) -> Result<CancellationToken, String> {
        let mut job = self.0.lock().map_err(|_| "Download manager unavailable")?;
        if is_active(&job.0.phase) {
            return Err("A private download is already active".into());
        }
        *job = (
            DownloadStatus {
                phase: "choosing_output".into(),
                share_uri,
                ..Default::default()
            },
            CancellationToken::new(),
        );
        Ok(job.1.clone())
    }
}

fn is_active(phase: &str) -> bool {
    matches!(
        phase,
        "choosing_output" | "downloading" | "preparing" | "adding_to_library"
    )
}

fn checked_inbox_manifest(app_data: &Path, uri: &str) -> anyhow::Result<Manifest> {
    let link = ts_core::ModelShareLink::parse(uri)?;
    let record = app_data
        .join("models/inbox")
        .join(hash_hex(link.descriptor_id()));
    let (signed, manifest) = inspect_received_release_files(
        &record.join("release.tsrelease"),
        &record.join("manifest.tswarm"),
    )?;
    link.verify_release(&signed)?;
    download::validate_download_manifest(&manifest)?;
    validate_model_path(&manifest.files[0].path)?;
    Ok(manifest)
}

fn check_library_compatibility(app_data: &Path, manifest: &Manifest) -> anyhow::Result<Manifest> {
    let mut library = read_library_manifests(&app_data.join("models/library"))?;
    if !library.iter().any(|local| local.root == manifest.root) {
        library.push(manifest.clone());
    }
    combine_manifests(&library)?.context("received model library is empty")
}

fn activate_received_model(
    app_data: &Path,
    manifest: &Manifest,
    received_cache: &Path,
) -> anyhow::Result<PathBuf> {
    download::validate_download_manifest(manifest)?;
    let catalog = check_library_compatibility(app_data, manifest)?;
    let source = ts_store::ObjectStore::open(received_cache)?;
    let store = ts_store::ObjectStore::open(app_data.join("cache"))?;
    // All input and library checks precede publication. Only verified bytes enter
    // the active cache; the serving provider is rebuilt by the existing engine.
    for tensor in &manifest.tensors {
        for chunk in &tensor.chunks {
            let bytes = source.get(chunk.hash)?;
            anyhow::ensure!(
                bytes.len() as u64 == chunk.length,
                "received chunk length mismatch"
            );
            store.put_verified(chunk.hash, &bytes)?;
            anyhow::ensure!(
                store.get(chunk.hash)? == bytes,
                "active cache readback differs"
            );
        }
    }
    let catalog_path = app_data
        .join("models/runtime")
        .join(format!("{}.tswarm", hash_hex(catalog.root)));
    persist_manifest(&catalog_path, &catalog)?;
    persist_manifest(
        &app_data
            .join("models/library")
            .join(format!("{}.tswarm", hash_hex(manifest.root))),
        manifest,
    )?;
    Ok(catalog_path)
}

#[tauri::command]
pub(crate) fn private_download_status(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<DownloadStatus, String> {
    Ok(manager
        .0
        .lock()
        .map_err(|_| "Download manager unavailable")?
        .0
        .clone())
}

#[tauri::command]
pub(crate) fn cancel_private_download(
    manager: State<'_, Arc<DownloadManager>>,
) -> Result<(), String> {
    let mut job = manager
        .0
        .lock()
        .map_err(|_| "Download manager unavailable")?;
    if job.0.phase != "downloading" {
        return Err(
            "Download cannot be cancelled during file preparation or library activation".into(),
        );
    }
    job.1.cancel();
    job.0.message = "Cancelling; verified chunks remain available for retry".into();
    Ok(())
}

#[tauri::command]
pub(crate) async fn download_received_model(
    app: tauri::AppHandle,
    engine: State<'_, Arc<LocalEngine>>,
    manager: State<'_, Arc<DownloadManager>>,
    share_uri: String,
    peer_id: String,
    peer_address: String,
) -> Result<bool, String> {
    download::validate_peer(&peer_id, &peer_address)
        .map_err(|e| format!("Peer rejected: {e:#}"))?;
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let preflight_data = app_data.clone();
    let preflight_uri = share_uri.clone();
    let manifest = tauri::async_runtime::spawn_blocking(move || {
        let manifest = checked_inbox_manifest(&preflight_data, &preflight_uri)?;
        check_library_compatibility(&preflight_data, &manifest)?;
        Ok::<_, anyhow::Error>(manifest)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("Download rejected: {e:#}"))?;
    let manager = manager.inner().clone();
    let cancel = manager.begin(share_uri)?;
    manager
        .0
        .lock()
        .map_err(|_| "Download manager unavailable")?
        .0
        .model_path = Some(manifest.files[0].path.clone());
    let picker_app = app.clone();
    let name = Path::new(&manifest.files[0].path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let selection = tauri::async_runtime::spawn_blocking(move || {
        picker_app
            .dialog()
            .file()
            .set_file_name(name)
            .blocking_save_file()
    })
    .await;
    let output = match selection {
        Ok(Some(selected)) => match selected.into_path() {
            Ok(path) => path,
            Err(error) => {
                manager.update("failed", error.to_string());
                return Err(error.to_string());
            }
        },
        Ok(None) => {
            manager.update("cancelled", "No output chosen; no model bytes fetched");
            return Ok(false);
        }
        Err(error) => {
            manager.update("failed", error.to_string());
            return Err(error.to_string());
        }
    };
    if output.exists() || output.with_extension("partial").exists() {
        manager.update("failed", "Output already exists; choose a new file");
        return Err("Output already exists; choose a new file".into());
    }
    manager
        .0
        .lock()
        .map_err(|_| "Download manager unavailable")?
        .0
        .output = Some(output.to_string_lossy().into_owned());
    manager.update("downloading", "Downloading and verifying chunks");
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn(async move {
        let result: anyhow::Result<()> = async {
            // Recheck the on-disk receipt after the picker, before network access.
            let manifest_uri = manager
                .0
                .lock()
                .map_err(|_| anyhow::anyhow!("Download manager unavailable"))?
                .0
                .share_uri
                .clone();
            let current = checked_inbox_manifest(&app_data, &manifest_uri)?;
            anyhow::ensure!(
                current.to_bytes() == manifest.to_bytes(),
                "receipt changed while choosing output"
            );
            let cache_root = app_data
                .join("models/download-cache")
                .join(hash_hex(manifest.root));
            let store = ts_store::ObjectStore::open(&cache_root)?;
            let progress_manager = manager.clone();
            download::fetch_manifest_chunks(
                &manifest,
                &store,
                &peer_id,
                &peer_address,
                cancel.clone(),
                move |progress| {
                    if let Ok(mut job) = progress_manager.0.lock() {
                        job.0.progress = progress;
                    }
                },
            )
            .await?;
            {
                let mut job = manager
                    .0
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Download manager unavailable"))?;
                anyhow::ensure!(!cancel.is_cancelled(), "download cancelled");
                job.0.phase = "preparing".into();
                job.0.message =
                    "Preparing verified file; cancellation is no longer available".into();
            }
            let finish_manager = manager.clone();
            tauri::async_runtime::spawn_blocking(move || -> anyhow::Result<()> {
                let _guard = engine
                    .import_lock
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Model importer unavailable"))?;
                // Library imports may have happened during this download.
                check_library_compatibility(&app_data, &manifest)?;
                download::prepare_verified_download(&manifest, &store, &output)?;
                finish_manager.update(
                    "adding_to_library",
                    "File prepared; adding verified model to library",
                );
                let catalog = activate_received_model(&app_data, &manifest, &cache_root)
                    .context("file prepared, but library activation failed")?;
                let config = {
                    let mut config = engine
                        .config
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Engine configuration unavailable"))?;
                    config.manifest_path = Some(catalog);
                    config.clone()
                };
                restart_engine_task(&engine, config).map_err(anyhow::Error::msg)?;
                Ok(())
            })
            .await
            .context("download preparation worker failed")??;
            Ok(())
        }
        .await;
        match result {
            Ok(()) => manager.update("complete", "File prepared and added to library. The engine is restarting; check it is online before sharing."),
            Err(_error) if cancel.is_cancelled() => manager.update("cancelled", "Download cancelled; choose a new output and retry to reuse verified chunks"),
            Err(error) => manager.update("failed", format!("{error:#}")),
        }
    });
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(root: &Path) -> (Manifest, PathBuf) {
        let file = root.join("private.safetensors");
        let payload = b"private desktop model";
        let header = format!(
            r#"{{"weight":{{"dtype":"U8","shape":[{}],"data_offsets":[0,{}]}}}}"#,
            payload.len(),
            payload.len()
        );
        let mut bytes = (header.len() as u64).to_le_bytes().to_vec();
        bytes.extend(header.as_bytes());
        bytes.extend(payload);
        fs::write(&file, bytes).unwrap();
        let donor = root.join("donor");
        import_model_into_store(&file, &donor).unwrap();
        (
            read_library_manifests(&donor.join("models/library"))
                .unwrap()
                .remove(0),
            donor.join("cache"),
        )
    }

    #[test]
    fn activation_preserves_the_received_manifest_and_builds_a_serving_inventory() {
        let root = tempfile::tempdir().unwrap();
        let (manifest, cache) = source(root.path());
        let recipient = root.path().join("recipient");
        let catalog = activate_received_model(&recipient, &manifest, &cache).unwrap();
        assert_eq!(
            read_library_manifests(&recipient.join("models/library")).unwrap()[0].to_bytes(),
            manifest.to_bytes()
        );
        assert_eq!(
            activate_model_catalog(&recipient).unwrap().unwrap(),
            catalog
        );
        // Inventory readback independently proves the received bytes are seedable.
        let store = ts_store::ObjectStore::open(recipient.join("cache")).unwrap();
        let combined: Manifest = serde_cbor::from_slice(&fs::read(catalog).unwrap()).unwrap();
        let output = root.path().join("prepared.safetensors");
        download::prepare_verified_download(&combined, &store, &output).unwrap();
        assert_eq!(
            fs::read(output).unwrap(),
            fs::read(root.path().join("private.safetensors")).unwrap()
        );
        // Idempotent activation does not duplicate catalog files.
        activate_received_model(&recipient, &manifest, &cache).unwrap();
        assert_eq!(
            read_library_manifests(&recipient.join("models/library"))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn collision_rejected_before_active_cache_or_library_mutation() {
        let root = tempfile::tempdir().unwrap();
        let (manifest, cache) = source(root.path());
        let recipient = root.path().join("recipient");
        activate_received_model(&recipient, &manifest, &cache).unwrap();
        let mut other = manifest.clone();
        other.tensors[0].descriptor.name = "different".into();
        let other = Manifest::new(other.format, other.file_size, other.tensors, other.files);
        assert!(activate_received_model(&recipient, &other, &cache)
            .unwrap_err()
            .to_string()
            .contains("already in the library"));
        assert_eq!(
            read_library_manifests(&recipient.join("models/library"))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            activate_model_catalog(&recipient).unwrap(),
            Some(
                recipient
                    .join("models/runtime")
                    .join(format!("{}.tswarm", hash_hex(manifest.root)))
            )
        );
    }

    #[test]
    fn download_receipt_is_rechecked_and_job_start_is_exclusive() {
        let root = tempfile::tempdir().unwrap();
        let (manifest, _) = source(root.path());
        let descriptor = ts_core::ModelReleaseDescriptor::new(
            ts_core::ModelCard::new("Private test"),
            "v1",
            vec![ts_core::ArtifactVariant {
                id: "default".into(),
                format: manifest.format,
                quantization: None,
                files: vec![ts_core::ModelArtifactFile {
                    path: manifest.files[0].path.clone(),
                    size_bytes: manifest.file_size,
                    tensor_count: 1,
                    manifest_root: manifest.root,
                    shard: None,
                }],
            }],
        );
        let signed = ts_core::SignedModelRelease::sign(
            descriptor,
            &ts_core::ModelSigningKey::from_bytes(&[41; 32]),
        )
        .unwrap();
        let uri = ts_core::ModelShareLink::for_release(&signed)
            .unwrap()
            .to_uri();
        let app_data = root.path().join("recipient");
        let record = app_data
            .join("models/inbox")
            .join(hash_hex(signed.release_id().unwrap()));
        fs::create_dir_all(&record).unwrap();
        fs::write(record.join("release.tsrelease"), signed.to_bytes().unwrap()).unwrap();
        fs::write(record.join("manifest.tswarm"), manifest.to_bytes()).unwrap();
        assert_eq!(
            checked_inbox_manifest(&app_data, &uri).unwrap().root,
            manifest.root
        );
        assert!(checked_inbox_manifest(&app_data, "../../other").is_err());
        let mut bytes = signed.to_bytes().unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        fs::write(record.join("release.tsrelease"), bytes).unwrap();
        assert!(checked_inbox_manifest(&app_data, &uri).is_err());
        assert!(!app_data.join("models/library").exists());
        let manager = DownloadManager::default();
        let token = manager.begin(uri.clone()).unwrap();
        assert!(manager.begin(uri.clone()).is_err());
        manager.update("downloading", "test");
        assert!(manager.begin(uri.clone()).is_err());
        token.cancel();
        manager.update("cancelled", "test");
        assert!(!manager.begin(uri).unwrap().is_cancelled());
    }
}
