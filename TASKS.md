# TensorSwarm MVP Task Board

Status values: `TODO`, `IN_PROGRESS`, `BLOCKED`, `DONE`.

Priority values: `P0` required for MVP, `P1` important for a credible demo, `P2` post-MVP.

## Current Progress

| Milestone | State | Evidence |
|---|---|---|
| M0 — Foundation | DONE | Workspace, six crate boundaries, documentation spine, README, ignore rules, ADR-0001, `cargo check`, `cargo test`, formatting, and clean-diff checks completed. |
| M1 — Parsers and manifests | DONE | GGUF/Safetensors metadata readers, representative `.gguf` and `.safetensors` fixture routing tests, bounded range extraction, SHA-256 helper, CLI inspection/manifest/verify/diff commands, chunk planning, tensor hashing, manifest roots, parser/hash tests, a 512-case bounded malformed-input no-panic corpus, generated chunk-coverage properties, and supported-host libFuzzer evidence are complete. |
| M2 — Store and materializer | DONE | Content-addressed object store, hash-verified atomic writes, object reads, manifest/recipe bounds validation, literal/tensor/zero-fill materialization, restart-persistent verified state with isolated per-tensor bitmaps, reachability scanning, and selective unreferenced-object repair are complete. |
| M3 — LAN P2P | DONE | Versioned peer protocol, domain-separated DHT keys, provider publication/lookup APIs, manifest-to-store provider inventory, bounded scheduler, deterministic peer scoring, authenticated QUIC/TCP transport, runtime node loop, verified chunk serving, bounded request batches, hash-verifying `LanClient`, wire cancellation, bounded in-flight accounting, `Have` exchange, in-process and independent-process transfer tests, transfer metrics, and live success/failure score updates are complete. |
| M4 — HTTP proxy and WebSeed fallback | IN_PROGRESS | Axum/Tokio localhost daemon, streamed full/range file responses, path traversal guard, deterministic manifest-path and HTTPS-origin resolver, corrected identity-carrying `PeerFetch` contract, `LanClient`-backed peer fetch adapter, CLI `proxy`, `proxy-manifest`, and `prepare` commands, verified reqwest/rustls WebSeeder, verified WebSeed-to-object-store helper, `FetchEngine` CAS→peer→WebSeed source selection, post-verification provider notifications, LAN tensor DHT publication, and CLI wiring for WebSeed-to-live-publisher notifications added. Runtime smoke test, manifest-router integration test, real loopback HTTP-over-LAN acceptance test, origin-outage/no-cache-write regression test, and complete-file `prepare` acceptance returned correct bytes where applicable. Recipe validation rejects gaps, overlaps, overflow, malformed chunk lengths, and uncovered ranges; transparent client integration remains. |
| M5 — Verification and demonstration | DONE | Owner-approved clean-checkout verification, supported-host libFuzzer execution, representative format fixtures, bounded cancellation/backpressure and malicious-peer fixtures, combined HTTPS fallback-to-live-DHT second-node discovery, and independent-process LAN transfer evidence. Transparent proxy rewriting remains deferred scope. |
| M6 — Tauri daemon and control plane | IN_PROGRESS | M6-01 through M6-03 are complete. M6-04 has a build-verified Tauri v2 shell and static control UI; Windows native rendering, release launch, and browser-level UI readback are verified. M6-05 NSIS install/launch/upgrade/uninstall passes; Linux DEB/RPM/AppImage packaging and DEB install/remove pass in WSL. Deepin native `cargo check`, DEB build, Xorg GUI launch, and daemon unit/process acceptance pass; interactive DEB install/remove/readback and targeted Windows tray interaction remain. The latest native Close test confirmed process exit rather than close-to-tray persistence. |

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
| M2-03 | P0 | Implement materialization state and verified chunk bitmap. | M2-02 | Restart preserves completed verified chunks with per-tensor index isolation. |
| M2-04 | P0 | Implement GGUF/Safetensors file-recipe materializer. | M1-11, M2-03 | Output is byte-identical to fixture. |
| M2-05 | P1 | Add cache repair and garbage-collection reachability scan. | M2-01, M1-11 | Unreferenced objects can be identified and selectively removed while live objects remain available. |

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
| M3-09 | P0 | Add two-process integration test using local fixture. | M3-01–M3-08 | `ts-cli` Node B retrieves and verifies a chunk from independently spawned Node A. |
| M3-10 | P1 | Add transfer metrics and structured tracing. | M3-06 | `LanClient` exposes attempts, successful chunks, failures, cancellations, and transferred bytes; real transfer coverage asserts the snapshot. |

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
| M4-09 | P1 | Add proxy configuration and integration examples for Python clients. | M4-03 | Deferred for post-MVP; MVP clients use the explicit localhost proxy URL. |
| M4-10 | P1 | Add an explicit `tswarm prepare` command for runtimes that require complete files. | M2-04 | `ts-cli prepare` exits only after required ranges are verified; acceptance compares the complete output to the fixture. |

## M5 — Verification and demonstration

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M5-01 | P0 | Add parser fuzz/property tests for lengths, counts, offsets, and nesting. | M1-13 | Fuzz target runs without panic under bounded input. |
| M5-02 | P0 | Add malicious-peer tests for wrong hashes and oversized frames. | M3-09 | Peer is penalized or disconnected without cache corruption. |
| M5-03 | P0 | Add proxy tests for partial ranges, retries, origin outage, and restart. | M4-01–M4-08 | Test suite passes deterministically. |
| M5-04 | P0 | Benchmark LAN transfer, HTTPS fallback, hashing, and disk write rates. | M3-09, M4-08 | Results are recorded with hardware and fixture details. |
| M5-05 | P0 | Write clean-checkout two-node demo instructions. | M5-01–M5-04 | A new contributor can reproduce the demo. |
| M5-06 | P0 | Perform owner/adversary review against `SPEC.md`. | All P0 tasks | Owner-approved acceptance evidence is recorded; known limitations are documented before MVP closure. |

## M6 — Tauri daemon and control plane

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M6-01 | P1 | Create the `ts-daemon` crate with a local authenticated control surface. | M2, M3, M4 | Public health succeeds; `/v1/status` rejects missing or invalid bearer tokens and returns a bounded JSON status for a valid token. |
| M6-02 | P1 | Move long-running P2P and proxy ownership into `ts-daemon`. | M6-01 | DONE — independent process acceptance verifies authenticated status, owned proxy health, clean shutdown, and control/proxy port release. |
| M6-03 | P1 | Expose model inventory, verification, preparation, cache repair, transfer, peer, and metric operations. | M6-02 | DONE — independent process acceptance covers all operations, including successful daemon-to-daemon transfer and hash-verified CAS persistence. |
| M6-04 | P1 | Add the minimal Tauri shell over the daemon control API. | M6-03 | IN_PROGRESS — Tauri v2 shell now provides a qBittorrent-inspired overview, model library/search/detail, transfer metrics, peer/source view, settings, cache repair, preparation actions, and live daemon status. Native window rendering and populated fixture readback pass; exact-origin CORS and production-daemon live-data acceptance coverage are merged, while Deepin manual daemon launch/readback remains pending a real manifest/store fixture and local bearer token. |
| M6-05 | P1 | Package and acceptance-test Windows/Linux desktop installs. | M6-04 | IN_PROGRESS — Windows release MSI and NSIS install/launch/upgrade/uninstall pass; Linux DEB, RPM, and AppImage bundles build, and DEB install/remove passes in WSL. Deepin native build, Xorg GUI launch, and daemon unit/process acceptance pass; the DEB script is currently sudo-gated before installation, manual live-daemon readback still needs a real fixture/token, and targeted Windows tray interaction remains. |
| M6-06 | P1 | Make the desktop application own the local engine lifecycle. | M6-03, M6-04 | IN_PROGRESS — Tauri embeds and starts the Rust engine on launch, creates an app-data cache and per-session UUID bearer token, and the UI obtains connection details automatically. Debug/release Windows binaries and the current NSIS build returned `/healthz` 200 and unauthenticated `/v1/status` 401; stopping each process removed the listener. A held control port produced a visible connection error and Restart button; retry failed while occupied, then succeeded after release and showed Daemon online. Windows close-to-background was verified; force-stopping the test process removed the endpoint. In Ubuntu WSLg, `cargo check -p ts-desktop` and `cargo tauri build --debug --no-bundle` passed; the Linux debug UI rendered its live empty-library/online state, and closing its window hid it while `/healthz` stayed 200; stopping the process removed the endpoint (HTTP 000). This is WSLg debug runtime evidence, not native distro/package acceptance. The tray code defines Show and Quit, but live tray interaction and graceful Quit remain unverified on Windows; the available WSLg session has no Linux notification-area shell for tray acceptance. Native Linux desktop/tray acceptance remains open. Headless daemon use remains optional. |
| M6-07 | P1 | Deliver the end-user model sharing flow. | M6-03, M6-06 | TODO — user can import a model file/folder, inspect and verify it, create a shareable manifest, seed/share it, and observe transfer/peer state through the UI; invalid input and incomplete/error states are actionable and never presented as successfully shared. |
| M6-08 | P1 | Verify desktop lifecycle and sharing in packaged Windows/Linux installs. | M6-05–M6-07 | TODO — clean install/first launch, engine startup, fixture sharing, restart/reopen, optional background mode, engine failure recovery, and uninstall are exercised with state readback on each supported platform. |

## Deferred Backlog

- `P2` NAT traversal with AutoNAT, DCUtR, and Relay v2.
- `P2` publisher signatures and trusted namespaces.
- `P2` architecture-specific execution plans and layer prefetch.
- `P2` runtime adapters for Ollama, vLLM, and Unsloth.
- `P2` sharded repository manifests.
- `P2` content-defined chunking.
- `P2` dynamic quantization peers.
- `P1` transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting; intentionally deferred because MVP clients configure the explicit localhost proxy URL.
- `P2` browser/WebRTC nodes.
- `P2` telemetry-resistant and privacy-preserving DHT operation.
- `P1` Desktop-owned engine lifecycle (M6-06), end-user model sharing workflow
  (M6-07), and packaged cross-platform lifecycle acceptance (M6-08). Users of
  the standard desktop install must not need to start a daemon in a terminal or
  install an OS service. Headless daemon operation remains supported as an
  optional server/operator path. The UI must not duplicate P2P or storage logic.
