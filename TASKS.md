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
| M4 — HTTP proxy and WebSeed fallback | IN_PROGRESS | Axum/Tokio localhost daemon, streamed full/range file responses, path traversal guard, deterministic manifest-path and HTTPS-origin resolver, corrected identity-carrying `PeerFetch` contract, `LanClient`-backed peer fetch adapter, CLI `proxy`, `proxy-manifest`, and `prepare` commands, verified reqwest/rustls WebSeeder, verified WebSeed-to-object-store helper, `FetchEngine` CAS→peer→WebSeed source selection, identity-carrying verified-chunk notifications wired into the daemon's live provider inventory and tensor DHT publication, and regression coverage proving daemon proxy-fetched chunks are immediately served and announced, with a third peer discovering the runtime through DHT and fetching the exact chunk. The fetch-only CLI proxy does not announce itself as a P2P provider. In-process daemon-runtime acceptance covers trusted local HTTPS WebSeed fetch, authenticated verification readback, cached range response, and control/proxy listener shutdown; manifest-router integration test, real loopback HTTP-over-LAN acceptance test, origin-outage/no-cache-write regression test, and complete-file `prepare` acceptance also return correct bytes where applicable. Recipe validation rejects gaps, overlaps, overflow, malformed chunk lengths, and uncovered ranges; transparent client integration remains. |
| M5 — Verification and demonstration | DONE | Owner-approved clean-checkout verification, supported-host libFuzzer execution, representative format fixtures, bounded cancellation/backpressure and malicious-peer fixtures, combined HTTPS fallback-to-live-DHT second-node discovery, and independent-process LAN transfer evidence. Transparent proxy rewriting remains deferred scope. |
| M6 — Tauri daemon and control plane | IN_PROGRESS | M6-01 through M6-03 are complete. M6-04 has a build-verified Tauri v2 shell and static control UI; Windows native rendering, release launch, and browser-level UI readback are verified. M6-05 NSIS install/launch/upgrade/uninstall passes; Linux DEB/RPM/AppImage packaging and DEB install/remove pass in WSL. Deepin native `cargo check`, DEB build, and daemon tests pass; the current native DEB passed metadata/content/`ldd` checks but install/live GUI readback is incomplete because sudo is interactive and the host became unreachable. Windows installed-app close-to-background and native tray-menu Quit handler now have process/listener readback; physical tray-icon activation was not separately exercised. |
| M7 — Model catalog and responsible Internet sharing | IN_PROGRESS | M7-01 schema is implemented. M7-02 policy/threat draft is prepared for owner and qualified legal review; no moderation, registry, or tracker enforcement exists. Public publishing remains blocked pending that review. See `docs/model-discovery-publication-and-abuse.md` and the explicitly unapproved `docs/model-sharing-abuse-policy-draft.md`. |

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
| M6-03 | P1 | Expose model inventory, verification, preparation, cache repair, transfer, peer, and metric operations. | M6-02 | DONE — independent process acceptance covers all operations, including successful daemon-to-daemon transfer and hash-verified CAS persistence. A missing-at-startup chunk is now admitted to the shared live provider only after active-manifest tensor/index/hash/length validation; a third peer fetched it byte-exactly and found its provider through DHT. A mismatched manifest hash is rejected before fetching. Runtime proxy/WebSeed identity-to-provider wiring remains open under M4. |
| M6-04 | P1 | Add the minimal Tauri shell over the daemon control API. | M6-03 | IN_PROGRESS — Tauri v2 shell now provides a qBittorrent-inspired overview, model library/search/detail, transfer metrics, peer/source view, settings, cache repair, preparation actions, and live daemon status. Native window rendering and populated fixture readback pass; exact-origin CORS and production-daemon live-data acceptance coverage are merged, while Deepin manual daemon launch/readback remains pending a real manifest/store fixture and local bearer token. |
| M6-05 | P1 | Package and acceptance-test Windows/Linux desktop installs. | M6-04 | IN_PROGRESS — Windows unsigned 0.1.1 NSIS install/uninstall and program/registry readback pass. WSL DEB/RPM/AppImage packaging and WSLg DEB runtime/readback pass, but the Ubuntu-built executable requires GLIBC_2.39 and is not compatible with Deepin's glibc 2.38. A current-worktree DEB built on Deepin passed metadata and `ldd` checks and its extracted binary passed live health/auth API readback. It is not installed because passwordless sudo is unavailable; install/remove and visible UI acceptance remain open. |
| M6-06 | P1 | Make the desktop application own the local engine lifecycle. | M6-03, M6-04 | IN_PROGRESS — Tauri embeds and starts the Rust engine on launch, creates an app-data cache and per-session UUID bearer token, and the UI obtains connection details automatically. Windows debug/release and installed NSIS API checks returned `/healthz` 200 and unauthenticated `/v1/status` 401; process stop removed listeners. Installed Windows port-conflict recovery showed the error/Restart UI and recovered to Daemon online. Installed Windows `WM_CLOSE` hid the main window while the engine remained live. The native Windows tray menu's Quit item was visually hovered and selected; the app process and ports 9090/9091/60659 closed without force termination. The menu was opened through the app's tray callback; physical notification-area icon activation was not separately tested. Under WSLg, debug and packaged-DEB window close hid the UI while the process and `/healthz` remained live; stopping the process removed the endpoint. Native Linux tray/runtime acceptance remains open. Headless daemon use remains optional. |
| M6-07 | P1 | Deliver the end-user model sharing flow. | M6-03, M6-06 | DONE — Windows debug UI acceptance verified native file and recursive-folder pickers, import/readback of a synthetic 108-byte Safetensors model (1 tensor, 1/1 verified chunk), and byte-identical complete-file preparation. A separate `ts-cli fetch-chunk` retrieved the 40-byte tensor payload from the desktop P2P provider with matching manifest hash and source bytes. The packaged WSLg DEB also imported a 91-byte fixture; an independent Linux CLI fetched a 23-byte tensor chunk whose hash and bytes matched the source range. The installed Windows NSIS app imported a 136-byte synthetic GGUF (1 tensor, 1/1 verified chunk) and prepared a byte-identical file. Unit tests cover import/persistence/rebuild, nested paths, rejection cases, and preparation. The current catalog schema requires one artifact format per library. |
| M6-08 | P1 | Verify desktop lifecycle and sharing in packaged Windows/Linux installs. | M6-05–M6-07 | IN_PROGRESS — Windows packaged lifecycle/import/preparation, including native tray-menu Quit, is verified. WSLg installed-DEB UI/import/P2P evidence is recorded, but WSLg is not native Linux acceptance. Current-source Deepin DEB built 2026-09-27 passed metadata, dependency, and GLIBC 2.38 preflight. Earlier isolated extracted-binary health/auth smoke returned `/healthz` 200 and `/v1/status` 401, then the process/listeners were stopped. A new isolated Xorg launch (PID 52125) timed out before readback; do not count it as live acceptance, and remote test scratch/process cleanup remains pending reconnection. `sudo -n true` fails, so package install/remove was not attempted. Still open: authorized native install/remove, GUI/import/persistence/tray acceptance, current launch readback/cleanup, and Linux packaged complete-file preparation. |
| M6-09 | P1 | Make multi-model inventory and selected-file preparation truthful. | M6-07 | IN_PROGRESS — daemon `/v1/models` now derives each file's manifest root, tensor/chunk totals, and verified availability from the combined catalog; the UI renders per-file availability and passes the selected path to `/v1/prepare`. Multi-file requests without a valid path fail before writing. Router tests cover two models, missing/corrupt chunks, selected-file output, and bad paths. The four-test independent-process suite verifies `/v1/models` roots/counts for two complete models and one model with a missing chunk, rejects omitted/unknown selections without writing, refuses preparation of the incomplete file, and prepares each complete file byte-exactly. The P2P provider test confirms missing/corrupt chunks are omitted from its serving inventory while manifest metadata remains available. This is daemon/process and provider-unit evidence, not packaged UI acceptance; live multi-model UI readback remains open. Mixed-format libraries remain unsupported by the v1 combined manifest. |

## M7 — Model repository, discovery, and responsible sharing (proposed)

Public Internet publishing must not start until M7-02 is accepted. The existing
hash-keyed DHT is a peer-discovery mechanism, not the model-search directory.

| ID | Priority | Task | Dependencies | Acceptance |
|---|---:|---|---|---|
| M7-01 | P1 | Specify a versioned model card, repository/release descriptor, and artifact-variant schema. | M6-07 | DONE — `ts-core` now defines versioned cards/releases, model metadata, license claims, mixed-format variants, multi-file shard references, and descriptor validation. Parsed vs publisher-declared origin is explicit but untrusted remotely; catalog review status is separate. Tests cover round-trip, unknown fields, invalid paths/shards/schema, mixed variants, and a fixed legacy manifest root. |
| M7-02 | P1 | Approve rights, abuse, and privacy requirements before public publishing. | M7-01 | IN_PROGRESS — Owner approved the draft's product-policy direction (non-model media excluded, affirmative redistribution basis, explicit-content models excluded by default). Public publishing remains BLOCKED pending qualified legal review, operational owner/coverage, and final retention/response decisions; no enforcement is implemented. |
| M7-03 | P1 | Add signed release descriptors and portable share-link export/import. | M7-01; explicit owner authorization for local-only slice | IN_PROGRESS — `ts-core` signs bounded CBOR releases and `ts-desktop` uses an OS credential store for an explicit device-local key, exposes a public fingerprint, signs/exports `.tsrelease` bundles, and previews signature-verified imports with local-manifest metadata matching. Windows workspace check/test/build, Windows Tauri debug build, and Ubuntu WSL workspace checks pass. Current Windows `cargo test --workspace` passes, including 21 desktop tests. Signed-bundle export stages/readbacks and commits without replacement; regression test covers existing-destination preservation and cleanup. On 2026-09-27, an isolated Windows debug app returned `/healthz` 200 and imported a synthetic 198-byte Safetensors fixture; the library showed 1/1 verified chunks. Settings showed an existing signing identity Ready; no key was created. Native save pickers exported the selected model's `.tswarm` manifest and signed `.tsrelease` bundle. `ts-cli verify-release` exited 0 with the manifest and optional share identifier. Native import preview reported a verified signature and 1/1 matching local manifest roots; preview was not saved to a catalog and did not verify model bytes. Native recipient pickers accepted the matching bundle/manifest and identifier, saved metadata-only receipt outside the active library, and the receipt remained visible after an isolated app restart while the active library still contained only the fixture. The exact process and test-owned profiles/scratch were removed; the default app profile remained. M7-03 remains open for adversarial live UI cases and Linux Secret Service/native packaging acceptance. No key backup/recovery/rotation, URI handler, resolver, directory, or network fetch; signatures do not prove identity, rights, safety, or policy review. M7-02 legal/operational gates continue to block public publishing. |
| M7-04 | P1 | Pilot a searchable model directory and peer-rendezvous service. | M7-02, M7-03 | Search by card metadata returns eligible releases; provider leases are keyed to descriptor/artifact roots, expire, and are rate-limited. The service stores listings/peer hints, not model bytes. Two independent users publish, discover, and fetch a permitted synthetic model with verified chunks. Delisted records stop appearing and new announcements are rejected. |
| M7-05 | P2 | Evaluate registry federation and public DHT/relay discovery. | M7-04 | Document privacy and revocation limits, provide compatible interoperability tests, and keep direct peer transfer available. Do not promise recall or universal takedown of cached P2P bytes. |
| M7-06 | P1 | Pilot private, file-mediated descriptor/manifest exchange between two independent users before Internet discovery. | M7-03 | IN_PROGRESS — `ts-cli verify-release` checks a bounded signed bundle against a separately supplied root-verified single-file manifest and optional immutable share identifier. Desktop exports the selected library model's `.tswarm` manifest and the native recipient action now has Windows isolated-debug evidence for bundle/manifest selection, exact identifier binding, metadata-only inbox receipt, and UI readback after app restart; the active library remained unchanged and no bytes were fetched. `ts-cli receive-release` saves checked metadata to a separate inbox; `ts-cli fetch-received` rechecks the record and fetches hash-verified chunks from an explicitly supplied peer into a separate cache, then materializes the file. Independent-process acceptance proves byte-equal output, offline cache reuse and rejection of wrong record ID, corrupted cache, and manifest. On 2026-09-27, a 130-byte synthetic Safetensors fixture was exchanged from the Windows host to Deepin 25 using a deterministic test signing key: Linux CLI receipt, explicit-peer fetch, wrong-identifier rejection, offline cache reuse after sender shutdown, and byte-exact SHA-256 readback passed. That is same-operator two-host evidence; the required independent-user pilot, independent key holder, and native Linux acceptance remain open. No automatic peer discovery or public resolver. |

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
