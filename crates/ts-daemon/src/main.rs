use anyhow::{Context, Result};
use std::path::PathBuf;

fn usage() -> &'static str {
    "usage: ts-daemon --control-bind <addr> --proxy-bind <addr> --manifest <path> --store <path> --origin <https-url>"
}

fn required(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .with_context(|| format!("missing value for {name}"))
}

fn parse_args() -> Result<ts_daemon::RuntimeConfig> {
    let mut args = std::env::args().skip(1);
    let mut control_bind = None;
    let mut proxy_bind = None;
    let mut manifest_path = None;
    let mut store_root = None;
    let mut origin_url = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--control-bind" => control_bind = Some(required(&mut args, &flag)?),
            "--proxy-bind" => proxy_bind = Some(required(&mut args, &flag)?),
            "--manifest" => manifest_path = Some(PathBuf::from(required(&mut args, &flag)?)),
            "--store" => store_root = Some(PathBuf::from(required(&mut args, &flag)?)),
            "--origin" => origin_url = Some(required(&mut args, &flag)?),
            "--help" => anyhow::bail!("{}", usage()),
            _ => anyhow::bail!("unknown argument {flag}; {usage}", usage = usage()),
        }
    }
    Ok(ts_daemon::RuntimeConfig {
        control_bind: control_bind.context("--control-bind is required")?,
        proxy_bind: proxy_bind.context("--proxy-bind is required")?,
        manifest_path: manifest_path.context("--manifest is required")?,
        store_root: store_root.context("--store is required")?,
        origin_url: origin_url.context("--origin is required")?,
        auth_token: std::env::var("TS_DAEMON_AUTH_TOKEN")
            .context("TS_DAEMON_AUTH_TOKEN must be set")?,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    ts_daemon::serve_runtime(parse_args()?).await
}
