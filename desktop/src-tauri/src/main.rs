use anyhow::Context;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, State, WindowEvent,
};
use tauri_plugin_dialog::DialogExt;
use ts_core::{FileRecipe, Manifest, TensorNode};

const MODEL_CHUNK_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Clone, serde::Serialize)]
struct DesktopSession {
    base_url: String,
    auth_token: String,
}

struct LocalEngine {
    config: Mutex<ts_daemon::RuntimeConfig>,
    task: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    import_lock: Mutex<()>,
}

#[derive(Clone, serde::Serialize)]
struct ImportedModel {
    path: String,
    manifest_root: String,
    total_chunks: usize,
}

fn hash_hex(hash: ts_core::Hash32) -> String {
    hash.0.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn persist_manifest(path: &Path, manifest: &Manifest) -> anyhow::Result<()> {
    anyhow::ensure!(
        manifest.verify_root(),
        "refusing to persist an invalid manifest"
    );
    let bytes = manifest.to_bytes();
    if path.is_file() {
        anyhow::ensure!(
            fs::read(path)? == bytes,
            "manifest path contains different data"
        );
        return Ok(());
    }

    let parent = path.parent().context("manifest path has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }
    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(_error) if path.is_file() => {
            let _ = fs::remove_file(&temporary);
            anyhow::ensure!(
                fs::read(path)? == bytes,
                "manifest path contains different data"
            );
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(error.into())
        }
    }
}

fn read_library_manifests(library_dir: &Path) -> anyhow::Result<Vec<Manifest>> {
    if !library_dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(library_dir)?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tswarm"))
        .collect::<Vec<_>>();
    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let manifest: Manifest = serde_cbor::from_slice(&fs::read(&path)?)?;
            anyhow::ensure!(
                manifest.verify_root(),
                "invalid saved model manifest: {}",
                path.display()
            );
            anyhow::ensure!(
                manifest.files.len() == 1,
                "saved model manifest must describe one file"
            );
            Ok(manifest)
        })
        .collect()
}

fn combine_manifests(manifests: &[Manifest]) -> anyhow::Result<Option<Manifest>> {
    let Some(first) = manifests.first() else {
        return Ok(None);
    };
    let mut tensors = Vec::<TensorNode>::new();
    let mut files = Vec::<FileRecipe>::new();
    let mut file_names = std::collections::HashSet::new();
    let mut total_size = 0_u64;
    for manifest in manifests {
        anyhow::ensure!(manifest.verify_root(), "invalid model manifest");
        anyhow::ensure!(
            manifest.format == first.format,
            "a library cannot mix GGUF and Safetensors files yet"
        );
        anyhow::ensure!(
            manifest
                .files
                .iter()
                .all(|file| file.format == first.format),
            "model file format does not match its manifest"
        );
        total_size = total_size
            .checked_add(manifest.file_size)
            .context("model library size overflow")?;
        for recipe in &manifest.files {
            validate_model_path(&recipe.path)?;
            anyhow::ensure!(
                file_names.insert(recipe.path.clone()),
                "a model named '{}' is already in the library; rename the file before importing it",
                recipe.path
            );
            files.push(recipe.clone());
        }
        tensors.extend(manifest.tensors.iter().cloned());
    }
    Ok(Some(Manifest::new(
        first.format,
        total_size,
        tensors,
        files,
    )))
}

fn activate_model_catalog(app_data: &Path) -> anyhow::Result<Option<PathBuf>> {
    let library_dir = app_data.join("models").join("library");
    let manifests = read_library_manifests(&library_dir)?;
    let Some(catalog) = combine_manifests(&manifests)? else {
        return Ok(None);
    };
    let catalog_path = app_data
        .join("models")
        .join("runtime")
        .join(format!("{}.tswarm", hash_hex(catalog.root)));
    persist_manifest(&catalog_path, &catalog)?;
    Ok(Some(catalog_path))
}

struct PreparedImport {
    source: PathBuf,
    model_path: String,
    index: ts_format::ModelIndex,
    manifest: Manifest,
}

fn validate_model_path(model_path: &str) -> anyhow::Result<()> {
    let path = Path::new(model_path);
    anyhow::ensure!(
        !model_path.is_empty()
            && !path.is_absolute()
            && path
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_))),
        "model path must be a safe relative path"
    );
    Ok(())
}

#[cfg(test)]
fn import_model_into_store(
    source: &Path,
    app_data: &Path,
) -> anyhow::Result<(ImportedModel, PathBuf)> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .context("model filename is not valid UTF-8")?
        .to_string();
    let mut imported = import_models_into_store(&[(source.to_path_buf(), name)], app_data)?;
    Ok((imported.0.remove(0), imported.1))
}

fn import_models_into_store(
    sources: &[(PathBuf, String)],
    app_data: &Path,
) -> anyhow::Result<(Vec<ImportedModel>, PathBuf)> {
    anyhow::ensure!(
        !sources.is_empty(),
        "no supported model files were selected"
    );
    let mut prepared = Vec::with_capacity(sources.len());
    for (source, model_path) in sources {
        validate_model_path(model_path)?;
        anyhow::ensure!(
            ts_format::supported_path(source),
            "choose GGUF or Safetensors model files"
        );
        anyhow::ensure!(source.is_file(), "selected model file is unavailable");
        let index = ts_format::inspect(source)
            .with_context(|| format!("inspect model file {}", source.display()))?;
        anyhow::ensure!(
            !index.tensors.is_empty(),
            "{} contains no tensors",
            source.display()
        );
        let parsed = ts_format::build_manifest(source, MODEL_CHUNK_SIZE)
            .with_context(|| format!("build manifest for {}", source.display()))?;
        anyhow::ensure!(
            parsed.verify_root() && parsed.files.len() == 1,
            "model manifest verification failed for {}",
            source.display()
        );
        let manifest = Manifest::new(
            parsed.format,
            parsed.file_size,
            parsed.tensors,
            vec![FileRecipe {
                path: model_path.replace('\\', "/"),
                ..parsed.files.into_iter().next().unwrap()
            }],
        );
        prepared.push(PreparedImport {
            source: source.clone(),
            model_path: model_path.replace('\\', "/"),
            index,
            manifest,
        });
    }

    let library_dir = app_data.join("models").join("library");
    let mut manifests = read_library_manifests(&library_dir)?;
    let mut roots = manifests
        .iter()
        .map(|manifest| manifest.root)
        .collect::<std::collections::HashSet<_>>();
    for item in &prepared {
        if roots.insert(item.manifest.root) {
            manifests.push(item.manifest.clone());
        }
    }
    let catalog = combine_manifests(&manifests)?.context("model library is empty")?;

    let store = ts_store::ObjectStore::open(app_data.join("cache"))?;
    let mut imported = Vec::with_capacity(prepared.len());
    for item in &prepared {
        anyhow::ensure!(
            item.index.tensors.len() == item.manifest.tensors.len(),
            "model tensor index does not match its manifest"
        );
        let mut source_file = File::open(&item.source)?;
        let mut total_chunks = 0;
        for (range, tensor) in item.index.tensors.iter().zip(&item.manifest.tensors) {
            anyhow::ensure!(
                range.descriptor.name == tensor.descriptor.name,
                "model tensor ordering does not match its manifest"
            );
            for chunk in &tensor.chunks {
                let length: usize = chunk
                    .length
                    .try_into()
                    .context("model chunk is too large")?;
                let offset = range
                    .offset
                    .checked_add(chunk.offset)
                    .context("model chunk offset overflow")?;
                source_file.seek(SeekFrom::Start(offset))?;
                let mut bytes = vec![0_u8; length];
                source_file.read_exact(&mut bytes)?;
                store.put_verified(chunk.hash, &bytes)?;
                total_chunks += 1;
            }
        }
        imported.push(ImportedModel {
            path: item.model_path.clone(),
            manifest_root: hash_hex(item.manifest.root),
            total_chunks,
        });
    }

    for item in &prepared {
        let manifest_path = library_dir.join(format!("{}.tswarm", hash_hex(item.manifest.root)));
        persist_manifest(&manifest_path, &item.manifest)?;
    }
    let catalog_path = app_data
        .join("models")
        .join("runtime")
        .join(format!("{}.tswarm", hash_hex(catalog.root)));
    persist_manifest(&catalog_path, &catalog)?;
    Ok((imported, catalog_path))
}

fn discover_model_files(root: &Path) -> anyhow::Result<Vec<(PathBuf, String)>> {
    anyhow::ensure!(root.is_dir(), "select a model folder");
    let root = root.canonicalize()?;
    let mut pending = vec![root.clone()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_dir() {
                pending.push(path);
            } else if file_type.is_file() && ts_format::supported_path(&path) {
                let relative = path
                    .strip_prefix(&root)?
                    .to_str()
                    .context("model folder contains a filename that is not valid UTF-8")?
                    .replace('\\', "/");
                validate_model_path(&relative)?;
                files.push((path, relative));
            }
        }
    }
    files.sort_by(|left, right| left.1.cmp(&right.1));
    anyhow::ensure!(
        !files.is_empty(),
        "no GGUF or Safetensors files were found in that folder"
    );
    Ok(files)
}

fn spawn_engine(
    config: ts_daemon::RuntimeConfig,
    previous: Option<tauri::async_runtime::JoinHandle<()>>,
) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        if let Some(previous) = previous {
            previous.abort();
            let _ = previous.await;
        }
        if let Err(error) = ts_daemon::serve_runtime(config).await {
            eprintln!("TensorSwarm local engine stopped: {error:#}");
        }
    })
}

#[tauri::command]
fn desktop_session(session: tauri::State<'_, DesktopSession>) -> DesktopSession {
    session.inner().clone()
}

#[tauri::command]
fn restart_local_engine(engine: State<'_, Arc<LocalEngine>>) -> Result<(), String> {
    let config = engine
        .config
        .lock()
        .map_err(|_| "Local engine configuration is unavailable".to_string())?
        .clone();
    restart_engine_task(engine.inner().as_ref(), config)
}

fn restart_engine_task(
    engine: &LocalEngine,
    config: ts_daemon::RuntimeConfig,
) -> Result<(), String> {
    let mut task = engine
        .task
        .lock()
        .map_err(|_| "Local engine supervisor is unavailable".to_string())?;
    let previous = task.take();
    *task = Some(spawn_engine(config, previous));
    Ok(())
}

async fn import_sources(
    engine: Arc<LocalEngine>,
    app_data: PathBuf,
    sources: Vec<(PathBuf, String)>,
) -> Result<Vec<ImportedModel>, String> {
    let import_engine = engine.clone();
    let imported = tauri::async_runtime::spawn_blocking(move || {
        let _import_guard = import_engine
            .import_lock
            .lock()
            .map_err(|_| "Model importer is unavailable".to_string())?;
        let (imported, catalog_path) = import_models_into_store(&sources, &app_data)
            .map_err(|error| format!("Model import failed: {error:#}"))?;
        let config = {
            let mut config = import_engine
                .config
                .lock()
                .map_err(|_| "Local engine configuration is unavailable".to_string())?;
            config.manifest_path = Some(catalog_path);
            config.clone()
        };
        restart_engine_task(import_engine.as_ref(), config)?;
        Ok::<_, String>(imported)
    })
    .await
    .map_err(|error| format!("Model import worker failed: {error}"))??;
    Ok(imported)
}

#[tauri::command]
async fn import_model(
    app: tauri::AppHandle,
    engine: State<'_, Arc<LocalEngine>>,
) -> Result<Option<Vec<ImportedModel>>, String> {
    let picker_app = app.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
        picker_app
            .dialog()
            .file()
            .add_filter("Model files", &["gguf", "safetensors"])
            .blocking_pick_file()
    })
    .await
    .map_err(|error| format!("Model picker failed: {error}"))?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let source = selected
        .into_path()
        .map_err(|error| format!("Could not read selected model path: {error}"))?;
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Selected model filename is not valid UTF-8".to_string())?
        .to_string();
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not locate TensorSwarm data: {error}"))?;
    import_sources(engine.inner().clone(), app_data, vec![(source, name)])
        .await
        .map(Some)
}

#[tauri::command]
async fn import_model_folder(
    app: tauri::AppHandle,
    engine: State<'_, Arc<LocalEngine>>,
) -> Result<Option<Vec<ImportedModel>>, String> {
    let picker_app = app.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
        picker_app.dialog().file().blocking_pick_folder()
    })
    .await
    .map_err(|error| format!("Folder picker failed: {error}"))?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let root = selected
        .into_path()
        .map_err(|error| format!("Could not read selected folder path: {error}"))?;
    let sources = tauri::async_runtime::spawn_blocking(move || {
        discover_model_files(&root).map_err(|error| format!("Folder scan failed: {error:#}"))
    })
    .await
    .map_err(|error| format!("Folder scan worker failed: {error}"))??;
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not locate TensorSwarm data: {error}"))?;
    import_sources(engine.inner().clone(), app_data, sources)
        .await
        .map(Some)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            desktop_session,
            restart_local_engine,
            import_model,
            import_model_folder
        ])
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let store_root = app_data.join("cache");
            std::fs::create_dir_all(&store_root)?;
            let manifest_path = activate_model_catalog(&app_data)?;
            let session = DesktopSession {
                base_url: "http://127.0.0.1:9090".into(),
                auth_token: uuid::Uuid::new_v4().to_string(),
            };
            app.manage(session.clone());
            let runtime_config = ts_daemon::RuntimeConfig {
                control_bind: "127.0.0.1:9090".into(),
                proxy_bind: "127.0.0.1:9091".into(),
                manifest_path,
                store_root,
                origin_url: "https://localhost/".into(),
                auth_token: session.auth_token,
            };
            app.manage(Arc::new(LocalEngine {
                config: Mutex::new(runtime_config.clone()),
                task: Mutex::new(Some(spawn_engine(runtime_config, None))),
                import_lock: Mutex::new(()),
            }));

            let show = MenuItem::with_id(app, "show", "Show TensorSwarm", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("TensorSwarm")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running TensorSwarm desktop application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

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

    #[test]
    fn importing_a_supported_model_persists_manifest_and_verified_chunks() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("tiny.safetensors");
        let payload = b"verified tensor payload";
        write_safetensors(&source, payload);

        let (imported, catalog_path) = import_model_into_store(&source, directory.path()).unwrap();

        assert_eq!(imported.path, "tiny.safetensors");
        assert_eq!(imported.total_chunks, 1);
        let manifest: ts_core::Manifest =
            serde_cbor::from_slice(&std::fs::read(catalog_path).unwrap()).unwrap();
        assert!(manifest.verify_root());
        assert_eq!(manifest.files.len(), 1);
        let store = ts_store::ObjectStore::open(directory.path().join("cache")).unwrap();
        assert_eq!(
            store.get(manifest.tensors[0].chunks[0].hash).unwrap(),
            payload
        );
    }

    #[test]
    fn imported_safetensors_can_be_prepared_from_the_verified_store() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("tiny.safetensors");
        write_safetensors(&source, b"verified tensor payload");
        let (_, catalog_path) = import_model_into_store(&source, directory.path()).unwrap();
        let manifest: Manifest =
            serde_cbor::from_slice(&std::fs::read(catalog_path).unwrap()).unwrap();
        let store = ts_store::ObjectStore::open(directory.path().join("cache")).unwrap();
        let output = directory.path().join("prepared.safetensors");

        store.materialize(&manifest, &output).unwrap();

        assert_eq!(
            std::fs::read(output).unwrap(),
            std::fs::read(source).unwrap()
        );
    }

    #[test]
    fn importing_invalid_model_does_not_add_a_shareable_manifest() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("broken.safetensors");
        std::fs::write(&source, b"not a safetensors model").unwrap();

        assert!(import_model_into_store(&source, directory.path()).is_err());
        let library = directory.path().join("models").join("library");
        assert!(!library.exists() || std::fs::read_dir(library).unwrap().next().is_none());
        let store = ts_store::ObjectStore::open(directory.path().join("cache")).unwrap();
        assert!(store.unreferenced_objects([]).unwrap().is_empty());
    }

    #[test]
    fn imported_models_survive_catalog_rebuild_and_are_combined() {
        let directory = tempdir().unwrap();
        let first = directory.path().join("first.safetensors");
        let second = directory.path().join("second.safetensors");
        write_safetensors(&first, b"first model payload");
        write_safetensors(&second, b"second model payload");

        let (_, first_catalog) = import_model_into_store(&first, directory.path()).unwrap();
        let (_, second_catalog) = import_model_into_store(&second, directory.path()).unwrap();
        let restarted_catalog = activate_model_catalog(directory.path()).unwrap().unwrap();
        assert_eq!(second_catalog, restarted_catalog);

        let manifest: Manifest =
            serde_cbor::from_slice(&std::fs::read(restarted_catalog).unwrap()).unwrap();
        assert!(manifest.verify_root());
        assert_eq!(manifest.files.len(), 2);
        assert_ne!(first_catalog, second_catalog);
    }

    #[test]
    fn import_rejects_a_different_model_with_an_existing_filename() {
        let directory = tempdir().unwrap();
        let first_dir = directory.path().join("first");
        let second_dir = directory.path().join("second");
        std::fs::create_dir_all(&first_dir).unwrap();
        std::fs::create_dir_all(&second_dir).unwrap();
        let first = first_dir.join("same.safetensors");
        let second = second_dir.join("same.safetensors");
        write_safetensors(&first, b"first model payload");
        write_safetensors(&second, b"different model payload");

        import_model_into_store(&first, directory.path()).unwrap();
        assert!(import_model_into_store(&second, directory.path()).is_err());
        assert_eq!(
            read_library_manifests(&directory.path().join("models/library"))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn folder_import_preserves_nested_paths_and_rebuilds_after_restart() {
        let directory = tempdir().unwrap();
        let folder = directory.path().join("models");
        let first_dir = folder.join("a");
        let second_dir = folder.join("b");
        std::fs::create_dir_all(&first_dir).unwrap();
        std::fs::create_dir_all(&second_dir).unwrap();
        write_safetensors(&first_dir.join("model.safetensors"), b"first payload");
        write_safetensors(&second_dir.join("model.safetensors"), b"second payload");
        std::fs::write(folder.join("readme.txt"), b"not a model").unwrap();

        let sources = discover_model_files(&folder).unwrap();
        assert_eq!(sources.len(), 2);
        let (imported, catalog_path) =
            import_models_into_store(&sources, directory.path()).unwrap();
        assert_eq!(imported.len(), 2);
        let restarted = activate_model_catalog(directory.path()).unwrap().unwrap();
        assert_eq!(catalog_path, restarted);
        let manifest: Manifest =
            serde_cbor::from_slice(&std::fs::read(restarted).unwrap()).unwrap();
        let mut paths = manifest
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();
        paths.sort_unstable();
        assert_eq!(paths, ["a/model.safetensors", "b/model.safetensors"]);
        assert!(manifest.verify_root());
    }

    #[test]
    fn invalid_file_in_folder_prevents_any_model_manifest_from_being_added() {
        let directory = tempdir().unwrap();
        let folder = directory.path().join("models");
        std::fs::create_dir_all(&folder).unwrap();
        write_safetensors(&folder.join("good.safetensors"), b"verified payload");
        std::fs::write(folder.join("broken.gguf"), b"not a GGUF file").unwrap();

        let sources = discover_model_files(&folder).unwrap();
        assert!(import_models_into_store(&sources, directory.path()).is_err());
        let library = directory.path().join("models/library");
        assert!(!library.exists() || std::fs::read_dir(library).unwrap().next().is_none());
        let store = ts_store::ObjectStore::open(directory.path().join("cache")).unwrap();
        assert!(store.unreferenced_objects([]).unwrap().is_empty());
    }

    #[test]
    fn library_rejects_mixed_artifact_formats() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("model.safetensors");
        write_safetensors(&source, b"verified payload");
        let (first, _) = import_model_into_store(&source, directory.path()).unwrap();
        let first: Manifest = serde_cbor::from_slice(
            &std::fs::read(
                directory
                    .path()
                    .join("models/runtime")
                    .join(format!("{}.tswarm", first.manifest_root)),
            )
            .unwrap(),
        )
        .unwrap();
        let mixed = Manifest::new(
            ts_core::ArtifactFormat::Gguf,
            first.file_size,
            first.tensors.clone(),
            first.files.clone(),
        );

        assert!(combine_manifests(&[first, mixed]).is_err());
    }
}
