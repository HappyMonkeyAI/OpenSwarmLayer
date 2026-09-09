use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

fn usage() {
    eprintln!("usage:\n  ts-cli node\n  ts-cli proxy [bind] <root>\n  ts-cli inspect <model>\n  ts-cli manifest <model> <output.tswarm>\n  ts-cli verify <model> <manifest.tswarm>\n  ts-cli diff <old.tswarm> <new.tswarm>");
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args_os();
    let _program = args.next();
    let command = args.next().and_then(|value| value.into_string().ok());
    match command.as_deref() {
        Some("node") => {
            let swarm = ts_p2p::build_lan_swarm()?;
            ts_p2p::run_lan_node(swarm).await?;
        }
        Some("proxy") => {
            let bind = args
                .next()
                .and_then(|value| value.into_string().ok())
                .unwrap_or_else(|| ts_proxy::DEFAULT_BIND.to_string());
            let root = PathBuf::from(args.next().context("missing proxy root directory")?);
            println!("proxy: http://{bind}/file/<path>");
            ts_proxy::serve(&bind, root).await?;
        }
        Some("inspect") => {
            let path = PathBuf::from(args.next().context("missing model path")?);
            let index = ts_format::inspect(&path)?;
            println!(
                "format: {:?}\nfile_bytes: {}\ntensors: {}",
                index.format,
                index.file_len,
                index.tensors.len()
            );
            for tensor in index.tensors {
                println!(
                    "{}\t{} bytes\t@{}\t{:?}",
                    tensor.descriptor.name, tensor.length, tensor.offset, tensor.descriptor.shape
                );
            }
        }
        Some("manifest") => {
            let path = PathBuf::from(args.next().context("missing model path")?);
            let output = PathBuf::from(args.next().context("missing output path")?);
            let manifest = ts_format::build_manifest(&path, 16 * 1024 * 1024)?;
            fs::write(&output, manifest.to_bytes())
                .with_context(|| format!("write {}", output.display()))?;
            println!(
                "manifest: {}\nroot: {:?}\ntensors: {}",
                output.display(),
                manifest.root,
                manifest.tensors.len()
            );
        }
        Some("verify") => {
            let path = PathBuf::from(args.next().context("missing model path")?);
            let manifest_path = PathBuf::from(args.next().context("missing manifest path")?);
            let expected: ts_core::Manifest = serde_cbor::from_slice(&fs::read(&manifest_path)?)?;
            let actual = ts_format::build_manifest(&path, 16 * 1024 * 1024)?;
            anyhow::ensure!(expected.root == actual.root, "manifest root mismatch");
            anyhow::ensure!(expected.verify_root(), "manifest self-check failed");
            println!("verified {:?}", expected.root);
        }
        Some("diff") => {
            let old: ts_core::Manifest =
                serde_cbor::from_slice(&fs::read(args.next().context("missing old manifest")?)?)?;
            let new: ts_core::Manifest =
                serde_cbor::from_slice(&fs::read(args.next().context("missing new manifest")?)?)?;
            let old_by_name = old
                .tensors
                .iter()
                .map(|tensor| (&tensor.descriptor.name, tensor.tensor_hash))
                .collect::<std::collections::BTreeMap<_, _>>();
            let mut changed = 0;
            for tensor in &new.tensors {
                if old_by_name.get(&tensor.descriptor.name) != Some(&tensor.tensor_hash) {
                    println!("changed {}", tensor.descriptor.name);
                    changed += 1;
                }
            }
            println!("changed_tensors: {changed}");
        }
        _ => usage(),
    }
    Ok(())
}
