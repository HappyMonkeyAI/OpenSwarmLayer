# TensorSwarm progress handoff

Last updated: 2026-09-13
Branch: master
Working tree: modified (M6 daemon/Tauri implementation and documentation)
HEAD: f4f2be9 Correct MIT copyright holder

## Completed this session

- Added manifest-aware `proxy-manifest` CLI support and hardened manifest range assembly.
- Added cancellable and bounded LAN fetch primitives.
- Added requester-side `Have` availability exchange and deterministic LAN peer scoring.
- Added deterministic manifest-path and HTTPS-origin resolution.
- Connected the proxy to the LAN client with tensor identity and chunk-index propagation.
- Added verified-fetch provider notifications and LAN tensor DHT publication.
- Added optional CLI peer identity and multiaddress configuration.
- Wired configured CLI peers to receive WebSeed verification publication notifications.
- Added a real loopback HTTP-over-LAN proxy acceptance test.
- Added Python direct-proxy documentation and a clean-checkout demo guide.
- Added a bounded 512-case malformed parser corpus and generated `proptest` chunk-coverage checks.
- Added Criterion benchmarks and recorded a reproducible primitive benchmark snapshot.
- Added the owner/adversary review; MVP was accepted with documented limitations.
- Added a bounded two-node Kademlia provider-discovery acceptance test that proves a second node finds a seeded tensor provider.
- Added bounded LAN request accounting and wire cancellation signaling for chunk fetches.
- Added malicious-peer integration fixtures covering wrong hashes and oversized responses.
- Added an excluded `fuzz` package with a bounded libFuzzer `format_inspect` target, portable fallback smoke runner, and usage documentation.
- Added a delayed-peer cancellation fixture and fixed peer-associated in-flight cleanup plus bounded cancel-frame flushing.
- Added a local HTTPS WebSeed acceptance test covering range retrieval, hash verification, CAS persistence, and provider notification.
- Added a live LAN-node publication event channel and changed the DHT discovery fixture to publish through that runtime path.
- Added a channel-backed `FetchEngine` notification adapter so verified WebSeed chunks can feed the live publication channel.
- Added the combined HTTPS fallback, live publication, and second-node discovery acceptance test.
- Re-ran the workspace and bounded fuzz-smoke verification from a clean detached checkout; all checks passed.
- Ran the actual `format_inspect` libFuzzer target under Ubuntu WSL2 nightly Rust: 1,000 bounded runs, 4 KiB maximum input, 219 coverage features, no crash.
- Added representative `.gguf` and `.safetensors` fixture routing/metadata coverage and removed a local HTTPS test startup race exposed by the full suite.
- Added and passed the independent-process `ts-cli` LAN acceptance test: a spawned provider node serves a verified chunk to a separately spawned fetch process.
- Added selective unreferenced-object repair to the content-addressed store; the live object is preserved and the dead object is removed by regression test.
- Added JSON-safe per-tensor verified-chunk bitmaps with restart persistence and same-index isolation coverage.
- Added `LanClient` transfer metric snapshots and asserted attempts, success, failures, cancellations, and bytes on the real two-node transfer.
- Added `ts-cli prepare` and verified complete-file materialization in the real CLI acceptance fixture.
- Added mutable peer-score tracking; successful and failed `LanClient` outcomes now update the score registry, with real-transfer success coverage.
- Reconciled the task ledger: M1, M2, and M3 are complete; prior cancellation/backpressure gap text was stale relative to the passing fixtures.
- Recorded the post-MVP desktop direction: Tauri UI over a local authenticated Rust daemon/control API, with headless CLI/server operation preserved.
- Added the initial `ts-daemon` crate and authenticated local control surface: public `/healthz` plus bearer-protected `/v1/status`, with tests for the auth boundary.
- Added `ts-daemon` runtime composition for manifest/store validation, verified provider setup, LAN node ownership, and manifest-proxy ownership; process-level lifecycle acceptance remains open.
- Added authenticated `/v1/models` inventory and `/v1/verification` store-state endpoints, with independent process acceptance covering model metadata and complete/incomplete chunk accounting.
- Added authenticated `/v1/prepare` and `/v1/cache/repair` operations; process acceptance verifies byte-identical preparation and selective removal of an unreferenced object.
- Added authenticated `/v1/peers` and `/v1/metrics` read APIs; process acceptance verifies the live daemon peer identity and truthful zero outbound-transfer counters.
- Added authenticated `POST /v1/transfers` for bounded single-chunk peer fetches; the daemon verifies the response and commits it through the CAS before reporting success, with malformed-request and successful daemon-to-daemon process coverage.
- Added the initial Tauri v2 desktop shell under `desktop/src-tauri`, a deterministic bundle icon, and a static control UI for daemon models, verification, peers, metrics, settings, and cache repair; the crate builds successfully in the workspace.
- Installed the Tauri CLI and produced Windows debug MSI and NSIS bundles at `target/debug/bundle/msi/TensorSwarm_0.1.0_x64_en-US.msi` and `target/debug/bundle/nsis/TensorSwarm_0.1.0_x64-setup.exe`.
- Launched `target/debug/ts-desktop.exe` for a live Windows process acceptance window and terminated it explicitly; no lingering desktop process remained. GUI control-data readback is still unverified.
- Administratively extracted the MSI to a temporary directory with `msiexec.exe` (exit 0), verified the packaged executable, launched the extracted binary, and terminated it cleanly.
- Produced release MSI (2,772,992 bytes), NSIS installer (1,822,510 bytes), and executable (8,338,432 bytes); launched the release executable successfully and terminated it cleanly.
- Exercised the static control UI in a real browser against a temporary local API fixture: Connect/Refresh populated models, verification, peers, metrics, and settings, and cache repair returned a successful result; the temporary harness was stopped afterward.
- Captured the native Tauri window with the desktop driver and verified the rendered model, verification, peers, metrics, settings, connection, refresh, and cache controls; native live-data readback remains separate from the browser evidence.
- Installed the release NSIS package into the user-local TensorSwarm location, launched the installed executable, ran the generated uninstaller, and verified delayed cleanup removed both the executable and uninstaller.
- Added a Tauri tray menu with Show TensorSwarm and Quit actions; workspace tests and release MSI/NSIS builds pass, and the release binary launches. Notification-area interaction remains unverified because the available desktop driver does not expose the tray icon.
- Performed a real Windows NSIS upgrade: installed 0.1.0, built and installed 0.1.1 over it, verified the replaced installed executable and uninstaller, launched 0.1.1, then uninstalled and verified cleanup. Linux packaging remains blocked on this Windows host, which has only the Windows Rust target and Tauri's Windows bundle targets available.
- Added close-to-tray handling to the Tauri shell: close requests are prevented and the window is hidden while the tray process remains alive. The desktop driver delivered a close attempt to a different PID, so this behavior compiles but remains unaccepted by a targeted native interaction test.
- Reworked the Tauri control UI into a qBittorrent-inspired model exchange workspace: overview metrics, searchable model library, manifest detail, preparation actions, transfer metrics, peer/source view, settings, cache repair, connection state, and honest empty/error states. Added restricted Tauri-origin CORS handling to the authenticated daemon API and covered preflight/authorized readback with a unit test.
- Built Linux DEB, RPM, and AppImage bundles from WSL Ubuntu after installing the GTK/WebKit prerequisites through the WSL root entrypoint; verified DEB metadata, packaged executable presence, and clean install/remove. Linux GUI launch/live-data readback remains unverified because WSL has no graphical display. Added square Linux icons to the Tauri bundle configuration after the initial all-target build exposed the AppImage icon requirement.
- Earlier WSL packaging attempt was blocked by missing GTK/WebKit dependencies and unavailable passwordless sudo; the later root-entrypoint installation and package build are recorded above.

## Recent commits

- `e7c012c` — Verify delayed peer cancellation
- `a7e5a79` — Ignore workspace build artifacts
- `7efb0cb` — Ignore fuzz build artifacts
- `370bc55` — Add bounded parser fuzz target
- `6a7ed74` — Record MVP owner adversary review
- `c39904c` — Add malicious peer rejection fixtures
- `2e0ca4d` — Bound LAN requests and signal cancellation
- `3a97328` — Record malicious peer acceptance evidence
- `85cc3da` — Add DHT provider discovery acceptance test
- `de9ee25` — Add reproducible format benchmarks
- `6031357` — Add generated chunk planning properties
- `e7755d2` — Correct demo acceptance commands
- `f9b6758` — Document clean checkout two-node demo
- `dc9946f` — Disconnect peers after invalid chunk responses
- `e482e74` — Add bounded malformed parser corpus
- `f9cbfb4` — Test proxy origin outage safety
- `bf30ff7` — Wire WebSeed provider publication in CLI
- `26205df` — Add CLI LAN peer configuration
- `6a682f7` — Verify proxy HTTP delivery from LAN peer

## Verification status

The latest full workspace verification passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `git diff --check`
- `cargo check --manifest-path fuzz/Cargo.toml --bin format_inspect`
- `cargo run --manifest-path fuzz/Cargo.toml --bin format_inspect_smoke`
- `cargo test -p ts-proxy https_webseed_fallback_verifies_stores_and_notifies`

Current unit-test counts are 1 (`ts-core`), 9 (`ts-format`), 14 (`ts-p2p`), 10 (`ts-proxy`), 6 (`ts-store`), and 1 (`ts-cli` integration test), with doc-tests passing. The Criterion harness also compiled and ran; the recorded short run measured approximately 50 ns for 64 MiB chunk planning and 1.96 ms for 4 MiB SHA-256 on the documented host.

## MVP decision

The owner approved MVP closure based on the clean-checkout, supported-host fuzz,
format-fixture, and HTTPS-to-live-DHT evidence. M5 is accepted with documented
limitations in `docs/owner-adversary-review.md`. The next roadmap work is
post-MVP: transparent proxy rewriting or the Tauri daemon/control-plane track.

Post-MVP limitations and deferred work:

1. Transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting is explicitly deferred; MVP clients use the explicit localhost proxy URL.
2. M6-01 through M6-03 are complete. The daemon control surface now covers inventory, verification, preparation, cache repair, peer, metrics, settings, and bounded single-chunk transfer operations, with independent successful-transfer acceptance.

## Recommended next slice

Complete native tray interaction and live-data readback acceptance, then rerun Linux dependency installation/package verification in WSL2 and add a Linux clean-install check. Keep transparent proxy rewriting and the broader P2 backlog deferred.

## Reporting note

The LAN announcement daemon at `192.168.5.229:4100` was unreachable during the final
handoff check-in and its one retry. No work was blocked by the reporting channel.
