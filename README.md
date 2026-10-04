# OpenSwarmLayer / TensorSwarm

Share LLM model files directly between computers, and let each recipient share
those files onward. The aim is a model-sharing desktop app with a workflow
similar to BitTorrent, using verified pieces of GGUF and Safetensors files.
OpenSwarmLayer is the desktop app; TensorSwarm is the Rust engine behind it.

> **Technical test beta — not ready for public release.**
> This project is still being developed and tested. It is intended for developers
> and invited testers who are comfortable building software and troubleshooting
> connections. The interface, setup and network support are unfinished. Please
> do not rely on it for production use or treat it as a finished model service.

## What can I try today?

You can import a model on one computer, download it on another, and then share
it from the recipient after the original sender stops. The app checks each
piece against the model's manifest before storing it, then checks the assembled
file before adding it to the local library.

Sharing currently takes some manual setup: you exchange two small metadata
files, a share identifier, and the sender's connection details. There is no
public model catalogue or automatic internet connection setup. A `tswarm://`
identifier identifies the signed metadata; clicking it does not start a download.

Windows desktop testing has demonstrated download, restart persistence and
re-sharing through temporary Cloudflare tunnels, with the final file matching
every byte of the source. The other peer used the Linux CLI, and both computers
were on the same local network. Testing with independent users on separate
internet connections is still pending. Native Linux desktop installation and
tray behaviour also need further testing. The exact evidence is in
[PROGRESS.md](PROGRESS.md).

## Getting started

### Using npx (Node.js 20+)

A small npm-compatible wrapper downloads and verifies the native app; Node.js
does not run its model-sharing engine. The wrapper is available as a tarball
on the GitHub beta release:

```sh
npx --package=https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/v0.1.1-beta.1/openswarmlayer-0.1.1-beta.1.tgz openswarmlayer install
```

Use `download` instead of `install` to fetch a verified package without installing.
On Linux, `download --format appimage` selects the portable AppImage.
Windows installation opens its normal installer; Linux DEB installation uses
apt/sudo. No app is installed merely by adding the npm package or requesting help.

The shorter `npx openswarmlayer@beta install` command is **pending npm registry
publication**, which needs an authenticated maintainer. See
[the npm wrapper guide](npm/README.md) for details. Platform limits below apply.

The [v0.1.1-beta.1 technical test release](https://github.com/SPhillips1337/OpenSwarmLayer/releases/tag/v0.1.1-beta.1)
provides a Windows x64 installer and experimental Linux x64 DEB/AppImage
packages. Linux builds target Ubuntu 24.04-era systems and
require glibc 2.39 or newer; they do not support the older Deepin host yet.

[install.ps1](install.ps1) and [install.sh](install.sh) download a specific beta
and check its SHA-256 before installation. You can download packages manually
from the release page, or use the versioned commands below. These commands
execute a remote script; download and read it first if you want to inspect it.
See [tagged beta releases](docs/releasing.md) for download-only options and the
release checklist.

Windows PowerShell:

```powershell
irm https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/v0.1.1-beta.1/install.ps1 | iex
```

Debian/Ubuntu-family Linux x64:

```sh
curl --proto '=https' --proto-redir '=https' -fsSL https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/v0.1.1-beta.1/install.sh | bash
```

On compatible Debian/Ubuntu-family Linux, install the downloaded DEB with:

```sh
sudo apt install ./OpenSwarmLayer-v0.1.1-beta.1-linux-amd64.deb
```

Alternatively, make the AppImage executable and run it:

```sh
chmod +x OpenSwarmLayer-v0.1.1-beta.1-linux-x86_64.AppImage
./OpenSwarmLayer-v0.1.1-beta.1-linux-x86_64.AppImage
```

If FUSE is unavailable, try the AppImage with `--appimage-extract-and-run`.
Package contents and isolated engine startup were checked in WSL; AppImage
startup also passed on native Mint. Full native Linux installation, desktop
controls, signing-key storage, tray and sharing acceptance remain open.

For this beta, start with a small test model that you have permission to share
and two computers you control. Allow disk space for the imported model, verified
cache and downloaded output. Windows has the most complete desktop test coverage.

If you have been given a test installer, install and launch OpenSwarmLayer.
Current Windows test installers are unsigned. The app starts its local engine
automatically; you do not need to start a separate daemon.

To build from source, you need Rust and the native build dependencies for
Tauri 2 on your operating system. Windows requires the Microsoft C++ build tools
and WebView2 runtime; Linux requires its Tauri/WebKit system dependencies.
The desktop UI is plain HTML and JavaScript, so there is no npm build step.

From the repository root:

```sh
cargo build --workspace
cd desktop/src-tauri
cargo run
```

To make a Windows installer, install the Tauri CLI and build from
`desktop/src-tauri`:

```sh
cargo install tauri-cli --version '^2' --locked
cargo tauri build --bundles nsis
```

The installer is written under `target/release/bundle/nsis/` in the repository.
Build your own current package rather than assuming an older test installer
contains the latest changes.

## Share a model

### On the sender's computer

1. Open **Model library**, choose **Import file**, and select a GGUF or
   Safetensors file. Start with one file and one format.
2. Wait until the model shows **100% verified availability** and the local
   engine is online.
3. In **Settings**, use **Create local signing key** if you do not already have
   one. The key stays in the operating system's credential store. Key backup
   and rotation are not available yet.
4. Select the model. Use **Export selected .tswarm manifest**, then fill in the
   signed descriptor fields and choose **Sign and export bundle** to save its
   `.tsrelease` file. Choose new filenames; export does not overwrite files.
5. Send the recipient both metadata files and the resulting `tswarm://`
   identifier. In **Peers & sources**, copy your current peer ID and a reachable
   listening address and send those too. Keep the app running while they download.

The `.tswarm` file describes the model and its pieces; the `.tsrelease` file
contains signed release metadata. Neither contains the model's weights.
A signature helps check that metadata has not changed; it does not prove a
publisher's real identity, redistribution rights or a model's safety.

### On the recipient's computer

1. Open **Model library**. Under **Private recipient inbox**, enter the received
   identifier in **Expected identifier received separately**.
2. Choose **Choose signed bundle, then matching manifest** and select the two
   files from the sender. This saves checked metadata; it has not downloaded
   the model yet.
3. Choose **Download from peer**, enter the sender's current peer ID and
   reachable address, and select a new output filename.
4. Wait for the download to complete. Confirm **100% verified availability**
   and an online engine before sharing it onward.

For a local-network test, use a reachable address such as
`/ip4/192.168.1.20/tcp/50000`, replacing the IP and port with the sender's actual
values. An address beginning with `127.0.0.1` refers to your own computer and
only works for a local connection or a deliberately configured tunnel.

For a temporary internet relay, follow the tested
[Cloudflare TCP bridge instructions](docs/private-beta.md#temporary-cloudflare-tcp-bridge).
The app does not create or manage these tunnels. Keep the local control API
private; the tunnel should expose only the peer listener.

### Check that re-sharing works

Stop the original sender. On the recipient, read its current peer ID and
listening address from **Peers & sources**, then use a third peer with a fresh
cache to download from it. Compare the final file with the source. This is the
core beta test: the downloaded model should remain available from its recipient.
Connection details can change after the engine restarts, so read them again.

For the full test procedure and CLI commands, see
[the beta testing guide](docs/private-beta.md).

## Current limits

- One private download runs at a time. Cancellation works during fetching;
  verified partial pieces remain available for retry, including after restart.
- Output filenames must be new, and the destination filesystem must support
  hard links. Existing files are not overwritten.
- A library currently needs one artifact format and rejects conflicting model
  paths. Mixed-format libraries and complete multi-file model repositories
  need more work.
- Download caches have no automatic quota or cleanup. Watch disk usage.
- There is no public tracker, model search, automatic NAT traversal or integrated
  relay. Connecting across the internet requires additional network setup.
- The interface still needs polish. Native Linux desktop, desktop-to-desktop,
  independent-user and separate-network acceptance remain open.

On Windows, closing the main window leaves the app running in the background.
Use **Quit** from its tray menu to stop it. This behaviour has earlier Windows
acceptance evidence; native Linux tray behaviour is still unverified.

## CLI and developer notes

The Rust CLI and daemon are available for technical testing and headless use.
See [docs/demo.md](docs/demo.md) for peer/proxy examples. A manifest alone cannot
seed a model: the sender also needs a populated, verified chunk store.

The localhost model proxy supports explicit file URLs and HTTP byte ranges,
with HTTPS fallback. Transparent rewriting through `HTTP_PROXY` or `HTTPS_PROXY`
is deferred. The desktop uses loopback ports 9090 for its authenticated control
API and 9091 for the model proxy.

To check a source checkout:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

When reporting a test problem, include your operating system, build or commit,
the steps you took, and the error shown. Do not include private keys, credentials
or model files in a report.

## Project documentation

- [Beta testing guide](docs/private-beta.md): desktop steps, temporary tunnels and remaining test gates.
- [Tagged beta releases](docs/releasing.md): versioned packages, install scripts and release preparation.
- [Roadmap](ROADMAP.md): what comes next.
- [Tasks](TASKS.md) and [progress log](PROGRESS.md): outstanding work and dated test evidence.
- [Architecture](ARCHITECTURE.md) and [specification](SPEC.md): how the Rust components work and the protocol requirements.
- [Benchmarks](docs/benchmarks.md) and [adversarial review](docs/owner-adversary-review.md): measurements and verification limits.
- [Discovery and publication plans](docs/model-discovery-publication-and-abuse.md): future catalogue/rendezvous work and its policy gates.
- [Agent instructions](AGENTS.md): contributor workflow; see also [CONTEXT.md](CONTEXT.md), [MCP.md](MCP.md) and [PLAN.md](PLAN.md).

## License

The software is licensed under [MIT](LICENSE). Model files have their own
licenses; the software's license does not grant permission to redistribute them.
