# TensorSwarm MVP Task Board

Status values: `TODO`, `IN_PROGRESS`, `BLOCKED`, `DONE`.

Priority values: `P0` required for MVP, `P1` important for a credible demo, `P2` post-MVP.

## Current Progress

| Milestone | State | Evidence |
|---|---|---|
| M0 — Foundation | DONE | Workspace, six crate boundaries, documentation spine, README, ignore rules, ADR-0001, `cargo check`, `cargo test`, formatting, and clean-diff checks completed. |
| M1 — Parsers and manifests | IN_PROGRESS | GGUF/Safetensors metadata readers, bounded range extraction, SHA-256 helper, CLI inspection/manifest/verify/diff commands, chunk planning, tensor hashing, manifest roots, parser/hash tests, and a 512-case bounded malformed-input no-panic corpus added. Remaining work is broader negative/property coverage, fuzz-target execution, and format-fixture validation. |
| M2 — Store and materializer | IN_PROGRESS | Content-addressed object store, hash-verified atomic writes, object reads, manifest/recipe bounds validation, literal/tensor/zero-fill materialization, resumable state, and report-only reachability scanning implemented. Per-tensor bitmap semantics and repair integration remain. |
| M3 — LAN P2P | IN_PROGRESS | Versioned peer protocol, domain-separated DHT keys, provider publication/lookup APIs, manifest-to-store provider inventory, scheduler with bounded batch popping, deterministic peer scoring with same-LAN preference and health/latency weighting, libp2p 0.56 QUIC/TCP transport, Noise, Yamux, Identify, mDNS, Kademlia, runtime node loop, verified chunk-provider handler, bounded request batches, retrying hash-verifying `LanClient`, cancellation-aware chunk fetching, requester-side bounded `Have` queries with availability filtering, and a passing loopback two-node transfer test added. Wire-level cancel propagation, full in-flight backpressure accounting, score updates from live transfer outcomes, and transfer metrics remain. |
| M4 — HTTP proxy and WebSeed fallback | IN_PROGRESS | Axum/Tokio localhost daemon, streamed full/range file responses, path traversal guard, deterministic manifest-path and HTTPS-origin resolver, corrected identity-carrying `PeerFetch` contract, `LanClient`-backed peer fetch adapter, CLI `proxy` and manifest-backed `proxy-manifest` commands with optional peer identity/address, verified reqwest/rustls WebSeeder, verified WebSeed-to-object-store helper, `FetchEngine` CAS→peer→WebSeed source selection, post-verification provider notifications, LAN tensor DHT publication, and CLI wiring for WebSeed-to-live-publisher notifications added. Runtime smoke test, manifest-router integration test, real loopback HTTP-over-LAN acceptance test, and origin-outage/no-cache-write regression test returned 200/206 with correct Content-Range, Content-Length, and bytes where applicable. Recipe validation now rejects gaps, overlaps, overflow, malformed chunk lengths, and uncovered ranges; transparent client integration remains. |
| M5 — Verification and demonstration | TODO | — |

## Definition of Done

A task is complete only when:

- The implementation and relevant documentation agree.
- Automated tests cover the normal path and the principal failure path.
- Errors are bounded and actionable.
- No unverified bytes enter the durable cache.
- The owner has run the task's acceptance command independently.

## M0 — Foundation

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M0-01 | P0 | Create Cargo workspace with `ts-core`, `ts-format`, `ts-store`, `ts-p2p`, `ts-proxy`, and `ts-cli` crates. | — | `cargo check --workspace` |
| M0-02 | P0 | Add `CONTEXT.md`, `SPEC.md`, `ARCHITECTURE.md`, `DESIGN.md`, `PLAN.md`, and this task board. | — | All documents exist and cross-reference one another. |
| M0-03 | P0 | Define common `Hash32`, error, format, and identifier types. | M0-01 | Unit tests cover serialization and display. |
| M0-04 | P0 | Add local verification commands and fixture policy. | M0-01 | Clean checkout commands are documented. |
| M0-05 | P1 | Add ADR template and first ADR for manifest identity. | M0-02 | ADR is linked from `SPEC.md`. |

## M1 — Parsers and manifests

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M1-01 | P0 | Implement bounded little-endian reader with positional reads. | M0-03 | Truncated and oversized reads fail safely. |
| M1-02 | P0 | Implement GGUF fixed header and metadata parser. | M1-01 | Valid fixture metadata is extracted without payload allocation. |
| M1-03 | P0 | Implement GGUF tensor-info parser and absolute offset calculation. | M1-02 | Tensor ranges are within file bounds and alignment is validated. |
| M1-04 | P0 | Implement GGML datatype byte-size/block-size calculation. | M1-03 | Known tensor sizes match reference fixtures. |
| M1-05 | P0 | Implement Safetensors header-length and JSON parser. | M1-01 | Header-only parse succeeds for a large fixture. |
| M1-06 | P0 | Validate Safetensors offsets, shapes, dtypes, and data-region bounds. | M1-05 | Malformed offsets and overlapping ranges are rejected. |
| M1-07 | P0 | Define canonical tensor descriptor serialization. | M0-03 | Same descriptor produces identical bytes on repeated runs. |
| M1-08 | P0 | Implement fixed-size chunk planner with datatype-block alignment. | M1-04, M1-06 | Chunk ranges cover the tensor exactly with no gaps or overlaps. |
| M1-09 | P0 | Implement incremental SHA-256 tensor and chunk hashing. | M1-07, M1-08 | Hashing does not require whole-tensor allocation. |
| M1-10 | P0 | Implement Merkle trie/root computation over sorted tensor names. | M1-07, M1-09 | Reordered input descriptors yield the same root. |
| M1-11 | P0 | Define deterministic `.tswarm` CBOR manifest and file recipe. | M1-10 | Manifest bytes are stable and round-trip correctly. |
| M1-12 | P0 | Implement CLI commands `inspect`, `manifest`, `verify`, and `diff`. | M1-11 | End-to-end parser CLI works on both formats. |
| M1-13 | P1 | Add mutation, truncation, endian, duplicate-name, and unknown-dtype tests. | M1-02–M1-12 | Negative test suite passes. |

## M2 — Store and materializer

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M2-01 | P0 | Implement content-addressed object-store paths and atomic writes. | M1-11 | Object path is derived only from hash. |
| M2-02 | P0 | Implement chunk verification before durable commit. | M2-01 | Corrupt bytes never become available objects. |
| M2-03 | P0 | Implement materialization state and verified chunk bitmap. | M2-02 | Restart preserves completed verified chunks. |
| M2-04 | P0 | Implement GGUF/Safetensors file-recipe materializer. | M1-11, M2-03 | Output is byte-identical to fixture. |
| M2-05 | P1 | Add cache repair and garbage-collection reachability scan. | M2-01, M1-11 | Unreferenced objects can be identified safely. |

## M3 — LAN P2P

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M3-01 | P0 | Create libp2p transport with QUIC, TCP fallback, Noise, and stream multiplexing. | M0-01 | Two nodes establish authenticated connections. |
| M3-02 | P0 | Add Identify and mDNS behaviours. | M3-01 | Nodes discover and record each other's addresses. |
| M3-03 | P0 | Add Kademlia provider records for manifests and tensors. | M3-02 | Provider lookup returns a reachable peer. |
| M3-04 | P0 | Define versioned manifest and chunk request/response codecs. | M3-01 | Codec round-trips bounded messages. |
| M3-05 | P0 | Implement chunk-serving policy with local hash verification. | M2-02, M3-04 | Peer serves only verified objects. |
| M3-06 | P0 | Implement chunk fetcher with timeout, retry, cancellation, and backpressure. | M3-04, M3-05 | Failed requests do not leak tasks or file handles. |
| M3-07 | P0 | Implement `Have` availability exchange for tensors/chunks. | M3-04 | Requester avoids known-missing chunks. |
| M3-08 | P0 | Implement LAN-preferred peer scoring. | M3-02, M3-06 | Same-LAN peer is preferred when available. |
| M3-09 | P0 | Add two-process integration test using local fixture. | M3-01–M3-08 | Node B retrieves and verifies a layer from Node A. |
| M3-10 | P1 | Add transfer metrics and structured tracing. | M3-06 | Throughput, retries, verification failures, and source are visible. |

## M4 — HTTP proxy and WebSeed fallback

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M4-01 | P0 | Implement daemon lifecycle and bind safety on `127.0.0.1:9090`. | M0-01 | Health endpoint responds and duplicate bind fails clearly. |
| M4-02 | P0 | Implement upstream URL and model-path resolver. | M1-11 | A request maps deterministically to a manifest/origin pair. |
| M4-03 | P0 | Implement HTTP full-file and single-range responses. | M2-04 | `curl` receives correct body and range headers. |
| M4-04 | P0 | Map requested byte ranges to manifest chunks. | M1-11, M4-03 | Only required chunks are scheduled. |
| M4-05 | P0 | Connect proxy fetches to swarm scheduler. | M3-06, M4-04 | Peer-backed HTTP response succeeds. |
| M4-06 | P0 | Implement HTTPS Range WebSeed fallback. | M4-04 | Origin bytes are fetched only for missing chunks. |
| M4-07 | P0 | Verify WebSeed bytes and commit them to the object store. | M2-02, M4-06 | Bad origin bytes are rejected. |
| M4-08 | P0 | Publish verified tensor/chunk provider availability. | M3-03, M4-07 | A second node can discover the newly seeded content. |
| M4-09 | P1 | Add proxy configuration and integration examples for Python clients. | M4-03 | Documented `HTTP_PROXY`/`HTTPS_PROXY` flow works. |
| M4-10 | P1 | Add an explicit `tswarm prepare` command for runtimes that require complete files. | M2-04 | Command exits only after required ranges are verified. |

## M5 — Verification and demonstration

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M5-01 | P0 | Add parser fuzz/property tests for lengths, counts, offsets, and nesting. | M1-13 | Fuzz target runs without panic under bounded input. |
| M5-02 | P0 | Add malicious-peer tests for wrong hashes and oversized frames. | M3-09 | Peer is penalized or disconnected without cache corruption. |
| M5-03 | P0 | Add proxy tests for partial ranges, retries, origin outage, and restart. | M4-01–M4-08 | Test suite passes deterministically. |
| M5-04 | P0 | Benchmark LAN transfer, HTTPS fallback, hashing, and disk write rates. | M3-09, M4-08 | Results are recorded with hardware and fixture details. |
| M5-05 | P0 | Write clean-checkout two-node demo instructions. | M5-01–M5-04 | A new contributor can reproduce the demo. |
| M5-06 | P0 | Perform owner/adversary review against `SPEC.md`. | All P0 tasks | Acceptance evidence is recorded before MVP is called complete. |

## Deferred Backlog

- `P2` NAT traversal with AutoNAT, DCUtR, and Relay v2.
- `P2` publisher signatures and trusted namespaces.
- `P2` architecture-specific execution plans and layer prefetch.
- `P2` runtime adapters for Ollama, vLLM, and Unsloth.
- `P2` sharded repository manifests.
- `P2` content-defined chunking.
- `P2` dynamic quantization peers.
- `P2` browser/WebRTC nodes.
- `P2` telemetry-resistant and privacy-preserving DHT operation.
