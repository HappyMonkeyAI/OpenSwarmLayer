# TensorSwarm MVP Development Plan

## Objective

Build a working proof of concept that can parse a local GGUF or Safetensors model, produce a deterministic tensor-addressed manifest, transfer verified tensor chunks between two local peers, and expose the result through a localhost HTTP service with HTTPS range fallback.

The MVP is successful when a second machine can obtain a model through TensorSwarm without knowing whether each byte came from a LAN peer or the original HTTP origin.

## MVP Scope

### Included

- Rust workspace and documentation spine.
- GGUF metadata and tensor-index parsing without loading full weights.
- Safetensors header parsing without loading full weights.
- Canonical tensor descriptors and SHA-256 identities.
- Fixed-size, datatype-safe chunking.
- Deterministic `.tswarm` manifests and Merkle roots.
- Content-addressed local object cache.
- libp2p QUIC/TCP transport with Noise authentication.
- mDNS LAN discovery.
- Kademlia provider lookup for manifests and tensors.
- Request/response transfer of verified chunks.
- Resumable transfers and local availability bitmaps.
- Local HTTP proxy on `127.0.0.1:9090`.
- HTTP `Range` handling.
- HTTPS WebSeed fallback.
- Provider registration after successful hash verification.

### Explicitly deferred

- Dynamic FP16/BF16-to-quantized conversion.
- Global production bootstrap infrastructure.
- Browser WebRTC clients.
- TURN service operation.
- FUSE/filesystem minifilter integration.
- Transparent lazy `mmap` page-fault handling.
- Full runtime integration with Ollama/vLLM/Unsloth.
- Publisher identity, signatures, revocation, and trust policies beyond basic integrity.
- Content-defined chunking.
- General distributed inference.

## Delivery Strategy

Implement vertical slices in this order:

```text
documentation and workspace
    -> local parser and manifest
    -> cache and materializer
    -> one-peer LAN transfer
    -> HTTP proxy and WebSeed fallback
    -> end-to-end interoperability and hardening
```

Each milestone must produce a runnable command, automated tests, and a short evidence record in the task or ADR documentation.

## Milestones

### M0 — Project foundation

Create the Rust workspace, documentation spine, test conventions, error model, and CI-quality local commands.

Exit criteria:

- `cargo check --workspace` succeeds.
- Documentation defines the first MVP contract.
- No implementation decision is left implicit in code.

### M1 — Format inspection and manifest generation

Implement streaming GGUF and Safetensors inspection, tensor metadata normalization, chunking, hashing, and deterministic `.tswarm` output.

Exit criteria:

- `tswarm inspect <file>` prints tensor metadata.
- `tswarm manifest <file> -o <manifest>` is deterministic.
- `tswarm verify <manifest> <file>` verifies every tensor and chunk.
- A one-tensor mutation is detected.
- A manifest diff identifies unchanged and changed tensors.

### M2 — Local cache and materialization

Implement content-addressed object storage, verified writes, resumable state, and reconstruction of the source artifact from a manifest recipe.

Exit criteria:

- Objects are stored by hash, independent of source URL.
- Interrupted writes resume safely.
- Reconstructed output is byte-identical to the input fixture.
- Corrupt cache objects are rejected and repairable.

### M3 — LAN peer transfer

Implement libp2p behaviours, mDNS discovery, manifest/tensor provider lookup, chunk request/response, availability exchange, and transfer scheduling.

Exit criteria:

- Two local nodes discover one another through mDNS.
- Node B retrieves a manifest from Node A.
- Node B downloads a tensor or layer from Node A.
- Hash failures terminate the individual transfer and do not poison the cache.
- Restart resumes from verified chunks.

### M4 — HTTP proxy and WebSeed fallback

Implement the localhost service, upstream URL mapping, Range support, swarm-first resolution, HTTPS range fallback, and post-verification seeding.

Exit criteria:

- `curl` can download a model through `127.0.0.1:9090`.
- Standard HTTP ranges return correct status and content-range headers.
- A peer outage causes an HTTPS fallback rather than a failed download.
- Verified fallback data becomes available to another peer.
- A second request is served from the local content-addressed cache.

### M5 — MVP hardening and demonstration

Run adversarial verification, benchmark LAN versus HTTPS, document limitations, and produce a repeatable two-node demo.

Exit criteria:

- Parser fuzz/property tests cover malformed headers and bounds.
- Protocol tests cover timeouts, cancellation, retries, and corrupt peers.
- The demo works from a clean checkout using documented commands.
- Performance and known limitations are recorded.

## Critical Design Gates

### Gate 1 — Identity correctness

Before networking, manifest identity must be deterministic across machines, architectures, and repeated runs.

### Gate 2 — Byte reconstruction correctness

Before proxy work, a manifest must reconstruct an artifact byte-for-byte, including headers, padding, metadata, and tensor offsets.

### Gate 3 — Peer integrity

Before accepting remote data, every chunk must be verified against the manifest before it enters the durable cache.

### Gate 4 — Origin equivalence

Before calling the proxy complete, bytes served through swarm and HTTPS fallback must produce identical artifact output.

## Post-MVP Direction

After M5, prioritize an approachable desktop sharing client before expanding
network topology or advanced model features:

1. Finish the desktop-owned engine lifecycle and platform acceptance.
2. Deliver the end-user model import, verify, publish/seed, and transfer flow.
3. Add multi-file repository manifests and sharded model support.
4. Add publisher signatures and trusted repository lineage before relying on
   publisher identity as a trust signal.
5. Add NAT traversal and relay deployment when there is a concrete cross-LAN
   use case and operational plan.
6. Explore layer-aware prefetch/runtime APIs and dynamic quantization as later,
   separately scoped tracks.

### Desktop application direction

The preferred desktop shell is Tauri rather than Electron because the project
already has a Rust networking, storage, format, and proxy core. The desktop
application must remain a client of that core and must not duplicate P2P or
content-addressed storage logic.

The standard desktop install must behave like a conventional torrent client:
the user launches the desktop application, and the application starts,
supervises, and connects to its local transfer engine. Starting a daemon from a
terminal or installing an OS service is not a prerequisite. Keep headless
`ts-daemon` operation available as an optional server/operator mode. The daemon
is an implementation boundary, not a required end-user workflow.

The desktop track should be delivered in this order:

1. Extract a long-running `ts-daemon` runtime around `ts-store`, `ts-p2p`, and
   `ts-proxy`.
2. Expose a local-only control API for model inventory, transfer control,
   peer/metric status, verification, cache repair, and complete-file prepare.
3. Add local authentication and explicit origin/permission boundaries before
   exposing control operations to a UI.
4. Make Tauri own daemon launch, readiness, health, restart/error handling, and
   clean shutdown. Background/tray persistence must be an explicit user
   preference with visible running state; ordinary close/exit behavior must be
   documented and tested per platform.
5. Build the model-sharing workflow: import a model file/folder, inspect and
   verify it, create/share its manifest, seed it, and show transfer/peer
   activity. Use daemon APIs; do not duplicate store or P2P logic in the UI.
6. Add library/search, transfer controls, cache/destination and bandwidth
   settings, notifications, and honest empty/error/recovery states.
7. Package and acceptance-test clean install, first launch, engine startup,
   sharing a fixture, restart/reopen, optional background operation, engine
   failure recovery, and uninstall on Windows and Linux.

The CLI, future web dashboard, and Tauri UI should consume the same daemon
control surface. The daemon must remain usable without the desktop shell for
headless and server deployments. Tauri sidecar packaging is a candidate
implementation for desktop-owned lifecycle, not an architectural requirement;
evaluate process supervision and packaging constraints before choosing it.
