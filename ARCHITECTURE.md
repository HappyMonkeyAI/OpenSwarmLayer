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
- Hashes establish content integrity; publisher signatures are post-MVP.

## Storage

Objects are stored by content hash. Materialized files and resumable state are derived views over manifests and verified objects.

## Networking

The MVP is LAN-first: authenticated libp2p connections, mDNS discovery, and a small request/response protocol. NAT traversal, public relays, and WebRTC are later layers.

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
the app and runtime. Its live graceful-shutdown acceptance remains open.

The daemon also supports headless CLI/server operation. The authenticated
control API exposes inventory, transfer, peer and metric status, verification,
cache repair, preparation, and proxy/runtime operations. Keep local
authentication and origin boundaries intact. The engine currently supports an
empty library when no manifest is configured, allowing first launch to reach a
usable desktop state. Model import and sharing through the UI remain future
work; see `TASKS.md` for milestone status.
