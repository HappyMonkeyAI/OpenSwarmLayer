# TensorSwarm MVP Architecture

```text
ts-cli
  ├── ts-format       parse GGUF/Safetensors and build manifests
  ├── ts-core         identities, descriptors, manifest primitives
  ├── ts-store        content-addressed objects and materialization
  ├── ts-p2p          libp2p discovery and chunk transfer
  └── ts-proxy        HTTP Range facade and WebSeed fallback
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
