# TensorSwarm MVP Architecture

```text
ts-cli
  ├── ts-format       parse GGUF/Safetensors and build manifests
  ├── ts-core         identities, descriptors, manifest primitives
  ├── ts-store        content-addressed objects and materialization
  ├── ts-p2p          libp2p discovery and chunk transfer
  └── ts-proxy        HTTP Range facade and WebSeed fallback

ts-daemon             local authenticated control surface (initial slice)
```

## Runtime flow

```text
HTTP request
  -> manifest resolver
  -> range-to-chunk planner
  -> local object store
  -> LAN peer / DHT provider
  -> HTTPS WebSeed fallback
  -> verification
  -> materialized response
```

## Trust boundaries

- Source files are untrusted input and must be bounds-checked.
- Peers are untrusted transport participants.
- DHT records are discovery hints, not content authority.
- Hashes establish content integrity, not safety, ownership, or legal rights.
- Publisher signatures and trusted lineage are post-MVP; signatures establish
  origin, not permission to redistribute or safe behavior.
- A future public model directory/tracker is a separate discovery control plane;
  it must not weaken daemon authentication or chunk verification.

## Storage

Objects are stored by content hash. Materialized files and resumable state are derived views over manifests and verified objects.

## Networking

The MVP is LAN-first: authenticated libp2p connections, mDNS discovery, and a small request/response protocol. NAT traversal, public relays, and WebRTC are later layers.

Only a runtime that serves verified chunks may publish itself as a tensor DHT
provider. `LanClient` and the fetch-only CLI manifest proxy do not serve inbound
chunk requests, so successful downloads must not create provider announcements.

## Desktop runtime

The Tauri desktop application starts and supervises the local Rust engine as
part of the application lifecycle. Tauri does not implement a second networking
runtime; the UI uses the existing authenticated daemon API:

```text
Tauri UI
  -> local authenticated control API
      -> ts-daemon
          -> ts-store
          -> ts-p2p
          -> ts-proxy
```

The desktop shell is organized around operator jobs rather than raw endpoint
payloads: overview and node health, model library and manifest detail, transfer
activity, peer/source discovery, and runtime settings/cache maintenance. The UI
uses the existing authenticated control API and deliberately labels scheduler
features that are not yet exposed by the daemon instead of simulating them.

At startup, the desktop shell creates an app-data cache, generates a
per-session bearer token, and starts `ts-daemon` on loopback control/proxy
listeners (`127.0.0.1:9090` and `127.0.0.1:9091`). The main window can hide to
the tray without stopping the engine; explicit tray Quit is intended to exit
the app and runtime. The installed Windows app's tray-menu Quit handler exited
the process and released its listeners in live acceptance; Linux tray acceptance
remains open.

The daemon also supports headless CLI/server operation. The authenticated
control API exposes inventory, transfer, peer and metric status, verification,
cache repair, preparation, and proxy/runtime operations. Keep local
authentication and origin boundaries intact. The engine currently supports an
empty library when no manifest is configured, allowing first launch to reach a
usable desktop state. The desktop's file/folder import path uses the existing
format parser, writes hash-verified chunks to `ts-store`, persists individual model manifests and a
combined runtime catalog, then restarts the engine against that catalog. The
current catalog schema requires all imported files to use one artifact format.
The daemon derives a per-file manifest root and verified chunk count from each
file recipe and its referenced tensors when serving `/v1/models`; the separate
`/v1/verification` response remains a whole-catalog aggregate. A prepare
request selects a catalog file by exact relative path; omission is accepted
only for a single-file manifest. This preserves the legacy single-file API
while preventing a multi-file selection from silently preparing the first file.
The daemon's P2P provider shares a live verified-chunk inventory with the
authenticated transfer route and manifest-backed proxy/WebSeed fetches. A
received or fetched chunk must match the active manifest's tensor, index, hash,
and length before it is admitted to that inventory and its tensor is announced
to DHT; an independently tested peer can then fetch the chunk without a daemon
restart. An in-process runtime test also exercises the actual HTTP proxy listener
against a test-trusted HTTPS origin, checks authenticated verification readback,
then confirms a cached range is served without another origin request and both
listeners close on shutdown. This does not launch the daemon binary or native
UI. Only the transfer route has independent-process plus third-peer DHT
acceptance evidence; the proxy/WebSeed DHT path is tested with real in-process
libp2p peers.
Visible Windows debug/NSIS and Ubuntu WSLg packaged-DEB Safetensors import and
verification are accepted. The installed Windows UI also imported and prepared
a synthetic GGUF fixture byte-identically. Independent peer fetch/hash/byte
readback is verified for the Windows debug and WSLg packaged-DEB runs. Windows
packaged close/background and native tray-menu Quit are accepted. A native
Deepin DEB passed metadata/content/dependency checks, but install and live
runtime/tray acceptance remain open; see `TASKS.md`.
