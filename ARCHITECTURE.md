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

## Desktop direction

The planned cross-platform desktop application uses Tauri and a long-running
local Rust daemon. Tauri is a client shell, not a second networking runtime:

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

The daemon must also support headless CLI/server operation. The control API
will expose inventory, transfer control, peer and metric status, verification,
cache repair, preparation, and proxy/runtime settings. Local authentication
and origin boundaries are required before UI control operations are enabled.

The initial daemon slice provides a public `/healthz` probe and an authenticated
`/v1/status` endpoint. P2P and proxy runtime ownership remain the next daemon
integration step; the status response reports those components as not started
until they are wired into the long-running process.
