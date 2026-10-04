# TensorSwarm progress handoff

Latest continuation: 2026-10-03. The dated entries at the end of this file record
native Windows private download/re-seeding, Cloudflare TCP relay acceptance,
and the CLI layout-verification fix. Current working source is uncommitted;
separate-network and current-package acceptance remain open. The summary below
is the earlier 2026-09-28 handoff, retained as historical context.

Last updated: 2026-09-28
Branch: master
Baseline HEAD at slice start: `3194bfc` (`master` was nine commits ahead of `origin/master`)
This handoff records local verification and a current-source native Deepin package/runtime smoke; existing modified files and untracked `scripts/` preserved, no commit or push performed

## Completed this session

- Validated this host's Ubuntu WSL2 as a Linux CLI/client test lane. Rust/Cargo
  1.98.1 built and ran `ts-cli` from the mounted checkout using a task-owned
  `/tmp` target directory. `cargo test -p ts-p2p` passed all 15 tests, including
  local-node chunk transfer, DHT discovery, cancellation, and malformed-peer
  rejection. Linux `ts-cli verify-release` accepted the signed descriptor with
  its matching manifest and rejected a mismatched manifest, altered signature,
  and wrong share identifier. `receive-release` wrote one metadata-only record;
  readback confirmed the saved bundle exactly matched the source. This is same-
  operator/same-host evidence, not the independent-user pilot or a model-byte
  fetch. WSL could reach the test desktop's P2P TCP listener, but the current
  desktop library was empty, so no live chunk exchange occurred.
- On the extracted Deepin desktop, the signed-release export UI saved a test
  `.tsrelease`; local CLI verification confirmed the bundle/manifest match and
  rejected tampering. The native import picker did not complete: it displayed
  an empty list and “File or directory not found” despite the file's presence.
  The extracted app's local health endpoint returned 200, and its control
  listener remained loopback-only. Do not count this as native import-preview,
  package-install, or tray acceptance.
- On 2026-09-28, the extracted Deepin app's ordinary model-file importer
  imported `native-model-b.safetensors` (75 bytes). The library count changed
  from 0 to 1 and the row showed 100% availability and `Verified`. This proves
  ordinary local model import/readback; the separate signed-release recipient
  result is recorded below.
  The first signed-release chooser attempt hit a stale search filter and lost
  focus. After relaunching the test-owned app with isolated XDG paths, the UI
  selected the 715-byte `TensorSwarm-release.tsrelease` and the 700-byte local
  `native-model-b` manifest. It reported the signature verified and saved
  `native-model-b · v1` to the private recipient inbox. Filesystem readback
  found one record and confirmed byte-for-byte equality with both source files.
  The active library remained at 2 models; no model bytes were fetched. The
  pre-fix UI showed `0 / 1` manifest matches because both recipient summary
  paths passed an empty manifest list. A regression test reproduced this and
  now requires `1 / 1` for save and inbox readback; the implementation
  summarizes against the verified supplied/stored manifest. The focused test,
  `cargo fmt --all -- --check`, `cargo check -p ts-desktop`,
  `cargo build -p ts-desktop`, and all 21 Windows `ts-desktop` tests pass. On
  2026-09-28, the corrected recipient-summary changes were applied to the
  test-owned Deepin source mirror. Remote `cargo fmt --all -- --check`,
  `cargo test -p ts-desktop --offline` (20 passed), and
  `cargo build -p ts-desktop --release --offline` succeeded; the release binary
  SHA-256 is `6ef37646b2f85f2e5a4b8744c013d17a92d631fb706109ed116194853dd8a8d2`.
  Replaying the existing bundle/700-byte manifest showed `1 / 1` matching local
  manifest roots. The UI rejected a wrong 696-byte manifest and a one-byte-
  tampered bundle, saying no new record was saved; the visible errors identified
  a signed-manifest mismatch and Ed25519 verification failure. Filesystem
  readback remained one inbox record (715-byte bundle, 700-byte manifest) and
  two active-library manifests. No artifact bytes were fetched. The executable
  ran from the test source mirror, not an installed DEB; the test process exited,
  its loopback listeners disappeared, and prior browser focus was restored.
  M7-03 remains open for Linux Secret Service/native package/tray acceptance;
  the independent-user M7-06 transfer also remains open.
- Closed a signed-release export no-clobber gap: the native picker path used
  `fs::write`, which could truncate an existing target if it appeared after
  picker confirmation. Export now writes and fsyncs a unique same-directory
  staging file, reads it back, atomically hard-links to a new target only, reads
  back the target, and removes staging. The focused regression test passes for
  successful readback, pre-existing target preservation, and staging cleanup.
  Verification after the fix: `cargo fmt --all -- --check`, the focused test,
  `cargo check --workspace`, `cargo test --workspace` (21 desktop tests plus
  CLI/daemon process suites and all workspace crates), `cargo build --workspace`,
  inline UI `node --check`, Windows Tauri debug build, and `git diff --check`
  passed. Native picker execution remains open.
- Added `ROADMAP.md` as the ordered, agent-readable next-slice guide without
  duplicating the existing `SPEC.md`, `PLAN.md`, or `TASKS.md`. Updated
  `AGENTS.md` and `BOOTSTRAP.md` to require those canonical sources plus the
  roadmap and evidence log, and linked the documentation roles from `README.md`.
  The roadmap orders live signed-release acceptance, the private two-user pilot,
  multi-model UI,
  native Linux acceptance, then policy-gated public sharing; local Kev/Jev and
  vision models are explicitly optional QA aids. Documentation-only; prior
  dirty work and untracked `scripts/` preserved, no commit or push.
- Hardened `ts-cli fetch-received` output commit: materialize in a temporary
  sibling directory, then hard-link into a new destination without replacement;
  staging is removed on failure/success. The unit test covers an output created
  after preparation; independent-process acceptance checks bytes and staging
  cleanup. Deepin SSH TCP was unreachable from this host (connect_ex 10035), so
  native Linux acceptance and a real second-user pilot remain open.
- Added desktop recipient-inbox readback: list only checked signed bundle/manifest
  pairs with matching directory IDs; show persisted, separately marked metadata
  in the Model library on startup and refresh. Focused tests cover two records,
  interrupted staging, renamed IDs, and tampered metadata. Native UI readback
  and a second-host recipient remain open.
- Exercised the Model library inbox in a real local browser with a synthetic
  Tauri-command fixture: Refresh displayed a received record, HTML-escaped a
  hostile model name, and on command failure cleared that record and rendered
  an explicit verification error. This is renderer evidence only, not native
  picker, OS credential-store, or persisted desktop-command readback.
- Added manifest-aware `proxy-manifest` CLI support and hardened manifest range assembly.
- Added cancellable and bounded LAN fetch primitives.
- Added requester-side `Have` availability exchange and deterministic LAN peer scoring.
- Added deterministic manifest-path and HTTPS-origin resolution.
- Connected the proxy to the LAN client with tensor identity and chunk-index propagation.
- Added verified-fetch provider notifications and LAN tensor DHT publication.
- Added optional CLI peer identity and multiaddress configuration.
- An initial CLI callback attempted to announce fetched tensor hashes, but the CLI proxy did not serve inbound P2P chunk requests; this was removed in the 2026-09-27 correction below. Only the daemon runtime announces after verified insertion into its live provider inventory.
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
- Committed the Linux packaging verification slice as `4e3caae` and pushed it to `origin/master`; the remote branch SHA was read back and matched the local commit.
- Probed the available Deepin 25 notebook at `Stephen@192.168.5.68` over read-only SSH: the host is x86_64 with an active Xorg session, Git 2.51.0, and Node 20.15.1. No checkout or Cargo/Tauri toolchain is present yet, and passwordless sudo is unavailable; the notebook is a viable next host for native Linux GUI and install/readback acceptance after user-approved setup.
- User-confirmed native desktop acceptance on the Deepin 25 notebook: the OpenSwarmLayer app worked on the Linux desktop. This confirms GUI launch/use at a high level; package install/readback and daemon live-data verification remain separately unrecorded.
- Completed native Deepin verification from the pushed `master` checkout: `cargo check --workspace` passed, the Tauri DEB bundle was produced, the desktop executable launched under the real Xorg session and terminated cleanly, and `cargo test -p ts-daemon -- --test-threads=1` passed (3 unit tests plus 2 process-acceptance tests). DEB install/remove through `dpkg` remains open because the notebook requires interactive sudo authentication.
- Retested the Windows release executable's native Close control against the actual `ts-desktop.exe` PID: the window and process exited, so close-to-tray remains unaccepted. The result is recorded as an acceptance gap rather than treated as evidence of tray persistence.
- Fast-forwarded the local checkout to remote `master` commit `12825a3`, which adds exact-origin daemon CORS validation and production-daemon live-data acceptance coverage. The remote tip is current locally; the untracked `scripts/` directory is preserved and not included here.
- Confirmed the Deepin acceptance shell path: the built daemon is at `~/Documents/development/OpenSwarmLayer/target/debug/ts-daemon`, but it is not installed in `PATH`; the DEB acceptance script is paused at `sudo -v`, and `open-swarm-layer` is not installed yet.
- Confirmed the daemon does not auto-start with the Tauri desktop shell. No `ts-daemon` process or control/proxy listener was present on Deepin; a bearer token exists only when supplied through `TS_DAEMON_AUTH_TOKEN` at daemon launch. A real `.tswarm` manifest and store are still required before manual live-data readback.
- Recorded a shell portability pitfall from the Deepin command attempt: a continuation backslash must be the final character on its line. Trailing spaces cause each following option to execute as a separate command; a single-line daemon command avoids this failure.

## Recent commits

- `1af5518` — Record Deepin daemon setup progress
- `12825a3` — Secure daemon CORS and verify live desktop data
- `5efa5b6` — Record tray acceptance gap
- `abd346f` — Record Deepin native verification
- `03bfccc` — Record Deepin Linux verification host
- `4e3caae` — Verify Linux desktop packaging
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

Latest workspace and desktop verification passed on 2026-09-24:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace` — CLI process 1; daemon 5 unit + 3 process;
  desktop 8; core 1; format 9; P2P 14; proxy 10; store 7; doc-tests passed.
- `cargo tauri build --debug --no-bundle`
- Node parse of the inline UI script (1 script)
- `git diff --check`

## Windows NSIS packaged acceptance — 2026-09-24

- Built the current unsigned per-user installer with
  `cargo tauri build --bundles nsis --no-sign`; artifact:
  `target/release/bundle/nsis/OpenSwarmLayer_0.1.1_x64-setup.exe`.
- Removed the previous per-user installation and verified that its executable,
  install directory, and HKCU uninstall registration were absent. Application
  data was empty before first launch. Installed the current package and read
  back version `0.1.1` and the installed executable.
- Launched the installed package: `/healthz` returned 200 and unauthenticated
  `/v1/status` returned 401. The native file picker imported a synthetic
  91-byte Safetensors fixture; the UI showed one verified model and 1/1 chunks.
- Prepared the complete file through the UI and verified the 91-byte output was
  byte-identical to the source. An independent `ts-cli fetch-chunk` process
  exited successfully against the packaged desktop P2P listener. The exact
  payload/hash comparison was not retained, so this Windows package run is not
  counted as verified P2P payload acceptance.
- Force-stopped the test process, confirmed its health endpoint disappeared,
  relaunched the installed package, and verified health 200/auth boundary 401,
  one persisted manifest, and visible UI state of one model at 100% availability.
  This proves restart persistence, not graceful Quit.
- Ran the current package's silent uninstaller; after its delayed cleanup, the
  program directory and HKCU uninstall registration were absent. Reinstalled
  the package to leave the current app available; no desktop process or engine
  listener remains. Removed the synthetic manifest, CAS object, and fixtures.
- Verified separately with the installed Windows package: occupied-port error
  state and recovery after releasing the conflicting listener. Not accepted in
  this slice: Windows packaged close/background, explicit tray Quit, GGUF UI
  import, and native Linux packaged lifecycle/sharing. No release code signing
  was tested.

## Ubuntu WSLg packaged DEB runtime and sharing — 2026-09-24

- Rebuilt and inspected the Linux DEB from Ubuntu WSL, installed it, and launched
  `/usr/bin/ts-desktop` in WSLg. The packaged UI rendered the online overview;
  `/healthz` returned 200 and unauthenticated `/v1/status` returned 401.
- Used the native GTK file picker to import a synthetic 91-byte Safetensors
  fixture. Fresh UI state showed one model, `Verified`, and 1/1 available chunks.
- An independent Linux `ts-cli fetch-chunk` process retrieved the 23-byte tensor
  payload over loopback from the packaged desktop P2P provider. Its SHA-256
  matched the manifest chunk hash and the fetched bytes matched the source
  tensor range.
- Sent SIGTERM to the packaged desktop and confirmed `/healthz` became
  unavailable. Relaunched the installed package from a test-owned output
  directory; the overview retained one model at 100% availability and the
  health/auth readback remained 200/401.
- Closed the packaged WSLg window and verified the window disappeared while
  `/healthz` remained 200. SIGTERM then stopped the process and removed the
  endpoint. This is close-to-background evidence, not explicit tray Quit.
- Uninstalled the WSL DEB and read back that the package and executable were
  absent. Removed the test library/runtime manifests, CAS chunk, WSL home fixture,
  and Windows scratch fixture. App WebKit/cache files were left intact.
- WSLg has no notification area in this session; no tray action was tested. This
  proves packaged UI/runtime/share behavior in WSLg only, not native Linux
  distro, tray, or package compatibility. Complete-file preparation was not
  verified on Linux; the UI prompt was opened and canceled.

Earlier supported-host fuzz and HTTPS WebSeed/provider-discovery acceptance
remain recorded in the history below; those commands were not rerun in this
desktop slice. The Criterion harness previously compiled and ran; its recorded
short run measured approximately 50 ns for 64 MiB chunk planning and 1.96 ms for
4 MiB SHA-256 on the documented host.

## MVP decision

The owner approved MVP closure based on the clean-checkout, supported-host fuzz,
format-fixture, and HTTPS-to-live-DHT evidence. M5 is accepted with documented
limitations in `docs/owner-adversary-review.md`. The next roadmap work is
post-MVP: transparent proxy rewriting or the Tauri daemon/control-plane track.

Post-MVP limitations and deferred work:

1. Transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting is explicitly deferred; MVP clients use the explicit localhost proxy URL.
2. M6-01 through M6-03 are complete. The daemon control surface now covers inventory, verification, preparation, cache repair, peer, metrics, settings, and bounded single-chunk transfer operations, with independent successful-transfer acceptance.

## Recommended next slice

Run M7-06 with an independent user/identity and a hash/byte-verified artifact
transfer from a reachable TensorSwarm peer; the current Deepin UI smoke verified
signed metadata receipt only, not peer fetch. Then continue with M6-09 selected-
model UI and M6-08 native Linux install/removal, Secret Service, and tray
acceptance. Public directory/rendezvous remains blocked on M7-02 legal and
operational gates. Headless daemon use remains optional; transparent proxy
rewriting and broader P2 features stay deferred.

## Product direction update — 2026-09-23

- Reframed M6 around the desktop app supervising its local Rust engine, like a
  conventional desktop transfer client; terminal-started daemon/service setup
  is no longer the expected desktop workflow.
- Kept standalone/headless `ts-daemon` as an optional server/operator mode.
- Added M6-06 lifecycle, M6-07 user-facing model sharing, and M6-08 packaged
  Windows/Linux lifecycle acceptance to `TASKS.md`; updated the post-MVP order
  in `PLAN.md`.
- No implementation/runtime acceptance was performed in this documentation
  slice. Windows close/tray and Deepin install/live-readback gaps remain open.

## Desktop engine lifecycle implementation — 2026-09-23

- Made the manifest optional for `ts-daemon`; without one it starts with an
  empty Safetensors manifest/library and uses `https://localhost/` as the
  placeholder origin. Existing manifest-backed operation remains unchanged.
- Updated daemon CLI usage so `--manifest` and `--origin` are optional; bind
  addresses, store path, and `TS_DAEMON_AUTH_TOKEN` remain required.
- Added an independent-process acceptance test that starts the daemon without
  a manifest, reads authenticated status, and confirms `/v1/models` returns an
  empty list.
- `cargo test -p ts-daemon`: passed (4 unit tests, 3 process-acceptance tests).
- Tauri does not yet launch or supervise this runtime. M6-06 remains in
  progress; sidecar packaging, secret delivery into the UI, and graceful
  shutdown/readback are not yet implemented or verified.

## Tauri-owned engine startup — 2026-09-23

- Linked `ts-daemon` into the Tauri app and start it during app setup using the
  app-data cache directory, loopback-only control/proxy binds, empty manifest,
  and a fresh UUID bearer token for each app session.
- Added a narrow Tauri `desktop_session` command for the local URL/token. The
  packaged UI requests it automatically, hides manual connection fields, waits
  up to 10 seconds for `/healthz`, then loads authenticated daemon state.
- Built and launched the real Windows debug desktop binary. Live loopback
  readback returned `/healthz` HTTP 200 and protected `/v1/status` HTTP 401
  without credentials. After stopping the app process, `/healthz` returned no
  response (HTTP 000), confirming the in-process runtime ended with it.
- `cargo fmt --all -- --check`, `cargo check --workspace`, and
  `cargo test --workspace` passed; `git diff --check` passed.
- Native window inspection was blocked because the desktop-capture driver
  refused to target its authorization process. The startup request/rendered UI,
  app-owned authenticated response, packaged install, close/tray behavior,
  startup-failure recovery, and graceful Quit remain unverified.

## Packaged release engine smoke — 2026-09-23

- Built the Windows release NSIS bundle with
  `cargo tauri build --bundles nsis --no-sign` from `desktop/src-tauri`; output:
  `target/release/bundle/nsis/OpenSwarmLayer_0.1.1_x64-setup.exe`.
- Launched the release executable (not only the debug build). Its local
  `/healthz` returned HTTP 200 and protected `/v1/status` returned HTTP 401
  without a bearer token. Stopping the release process tree caused the health
  endpoint to become unavailable (HTTP 000).
- This proves release-binary engine startup/auth boundary/process termination,
  not NSIS install-launch-uninstall or visual UI acceptance. Those remain open.

## Packaged per-user launch readback — 2026-09-23

- Invoked the built NSIS setup (`OpenSwarmLayer_0.1.1_x64-setup.exe`) in silent
  mode, then read the registered per-user install location from the Windows
  uninstall registry key. The executable is `ts-desktop.exe` under
  `%LOCALAPPDATA%\OpenSwarmLayer`.
- Launched that packaged executable and observed `/healthz` HTTP 200,
  unauthenticated `/v1/status` HTTP 401, and HTTP 000 after stopping the app
  process. The installer invocation targeted a custom path but NSIS retained
  its registered per-user default; because a clean isolated install was not
  established, record this as installed-app launch/API smoke, not clean-install
  acceptance.
- Earlier CUA attempts could not read rendered UI state; the live-window test
  recorded below later verified the error and recovery flow. Tray Quit remains
  unverified.

## Engine bind-conflict and reopen recovery — 2026-09-23

- Held the desktop control port (`127.0.0.1:9090`) with a test-owned TCP
  listener and launched the installed app. The desktop process remained alive;
  the engine endpoint did not become available.
- Released the test listener, closed the app, and relaunched it. The engine
  recovered with `/healthz` HTTP 200 and unauthenticated `/v1/status` HTTP 401;
  after app exit, `/healthz` returned HTTP 000.
- This verifies process-level recovery by reopening after a bind conflict, not
  an in-app restart or rendered error-state flow. Native error-state readback
  and explicit tray Quit remain unverified.

## Visible engine failure recovery and release acceptance — 2026-09-23

- Ran the current debug desktop app with the control port held by a test-owned
  listener. The rendered UI showed the connection error and `Restart local
  engine` action. Retrying while the port remained occupied returned to the
  visible error state. After releasing the listener, clicking Restart brought
  the UI to `Daemon online`; the empty library and live overview were visible.
- Clicking the window Close control hid the window while the desktop process
  and `/healthz` remained live (HTTP 200). Force-stopping the process tree
  removed the endpoint (HTTP 000). This verifies close-to-background behavior,
  not graceful tray Quit.
- Rebuilt the Windows NSIS bundle from current sources:
  `target/release/bundle/nsis/OpenSwarmLayer_0.1.1_x64-setup.exe`. Release
  executable smoke returned `/healthz` HTTP 200 and unauthenticated
  `/v1/status` HTTP 401; after process exit, `/healthz` returned HTTP 000.
- M6-06 remains in progress until tray Quit/graceful shutdown and Linux
  desktop acceptance are verified. This is UI/release evidence, not a claim of
  full M6-06 completion or clean-install acceptance.

## Tray Quit acceptance attempt and handoff — 2026-09-23

- Launched the current debug desktop app and confirmed the overview rendered.
  Closing its main window hid the app as designed; the engine stayed alive
  until the test process tree was stopped.
- Tried to reach the Windows notification area to select the tray `Quit` item.
  The desktop automation target remained the file-manager/remote-desktop
  surface; attempts to target the full-screen or taskbar surface were rejected
  or produced no verified state change. No tray menu item was selected, so
  graceful tray Quit is still unverified.
- Stopped the test-owned process tree and read back `/healthz` HTTP 000; no
  `ts-desktop` test process remained. The prior foreground ZeroG terminal was
  restored after the desktop inspection.
- No source changes were made for this acceptance attempt. Continue M6-06 only
  when the Windows taskbar/notification area is addressable; separately keep
  native Linux graphical acceptance and M6-07 model import/sharing open.

## Reporting note

The LAN announcement daemon at `192.168.5.229:4100` was unreachable during the final
handoff check-in and its one retry. No work was blocked by the reporting channel.

## WSLg Linux desktop runtime smoke — 2026-09-24

- Found the existing Ubuntu WSL distribution has WSLg (`DISPLAY=:0`,
  `WAYLAND_DISPLAY=wayland-0`) and the GTK/WebKit Tauri build dependencies.
- `cargo check -p ts-desktop` passed in Ubuntu WSL; `cargo tauri build --debug
  --no-bundle` produced `target/debug/ts-desktop` for Linux.
- Launched that Linux binary in the WSLg desktop. The rendered overview showed
  the local daemon online, empty model library, and live overview state;
  `/healthz` returned HTTP 200 and unauthenticated `/v1/status` returned HTTP
  401.
- Closed the WSLg app window and verified it disappeared while the Linux app
  process and `/healthz` remained live (HTTP 200). Sent SIGTERM to the
  test-owned process and read back HTTP 000 afterward; no test process remained.
- This is interactive WSLg debug-runtime evidence, not Deepin/native desktop,
  packaged install, or tray evidence. WSLg did not provide a Linux notification
  area for tray interaction. Windows tray Quit/graceful shutdown and native
  Linux tray/package acceptance remain open; M6-06 stays in progress.
- No source/runtime code changed in this acceptance slice; the Linux build
  output remains under ignored `target/`.

## Desktop model import implementation slice — 2026-09-24

- Added native file and recursive-folder selection for GGUF/Safetensors. Folder
  scanning ignores symlinks, keeps nested relative paths, and rejects folders
  with no supported models.
- The importer parses every selected artifact and validates paths/manifests
  before writing any new library manifest; chunk bytes are hash-verified into
  the existing CAS, per-model manifests are persisted, a runtime catalog is
  rebuilt, and the desktop restarts the engine against that catalog. The
  native picker and parsing/store work run on blocking workers, not the UI
  thread.
- Added tests for valid import and verified chunk persistence, catalog rebuild,
  nested folder paths, duplicate-name and mixed-format rejection, and invalid
  folder imports not publishing a manifest. The daemon cancellation test also
  verifies that cancelling the runtime releases control and proxy listeners.
- Fixed complete-file preparation: `ts-store` now assembles tensor ranges from
  the manifest's hash-verified CAS chunks instead of looking for an unstored
  whole-tensor object. Added multi-chunk materialization and desktop import-to-
  prepare regression tests.
- Final verification passed: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace` (8 desktop tests, 7
  store tests, and daemon/process suites), Windows debug
  `cargo tauri build --debug --no-bundle`, and inline UI JavaScript parse.
- The Windows debug UI imported the synthetic 108-byte Safetensors fixture
  through both native file and recursive-folder pickers. The library showed the
  verified model (1 tensor, 1/1 verified chunk); complete-file preparation
  produced 108 bytes identical to the source. A separately spawned `ts-cli
  fetch-chunk` requester then fetched the 40-byte tensor payload from the
  running desktop P2P provider over loopback. The fetched SHA-256 matched the
  manifest chunk hash and its bytes matched the fixture's tensor payload.
- V3 process readback returned `/healthz` HTTP 200 and unauthenticated
  `/v1/status` HTTP 401. After stopping the desktop process, no listener
  remained on ports 9090/9091 and `/healthz` was unavailable. The temporary
  fixture manifests, CAS object, and scratch files were removed afterward.
- M6-07 is accepted for the Safetensors slice. Packaged Windows GGUF UI coverage
  is recorded below; broader packaged/cross-platform sharing acceptance remains
  under M6-08. The catalog schema permits one artifact format per library.
  Windows tray Quit and native Linux package/tray acceptance remain open under
  M6-06/M6-08.
- No commit or push was made. The pre-existing untracked `scripts/` directory
  remains untouched.

## Windows NSIS GGUF UI follow-up — 2026-09-24

- Launched the installed Windows NSIS app and used its native file picker to
  import the synthetic `tiny.gguf` fixture (136 bytes). Fresh UI state showed
  one library model, an import toast for one verified chunk, `Gguf · 136 B`,
  one tensor, `Verified`, and `1 / 1 chunks`. The detail pane reported one
  available peer; no peer transfer was attempted in this follow-up.
- Used `Prepare complete file` in the UI. The prepared output was 136 bytes and
  byte-identical to the source; both SHA-256 values were
  `0a5c0a060c65bb9c112e39c4a5336bad2faf1c36084bac5e4ed6c9b8d021809f`.
- The packaged app returned `/healthz` 200 and unauthenticated `/v1/status`
  401. After force-stopping the test app, ports 9090/9091 were closed. Removed
  the temporary GGUF fixture and prepared output. Forced termination is not
  evidence of graceful tray Quit.
- Post-acceptance checks passed: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace`,
  `cargo tauri build --debug --no-bundle`, and `git diff --check`.
- Windows packaged close/background and explicit tray Quit are followed up
  below. Native Linux packaged/runtime/tray acceptance remains open; WSLg
  evidence does not substitute for native Linux acceptance.
  No source code changed; no commit or push was made. The pre-existing
  untracked `scripts/` directory remains untouched.

## Windows NSIS lifecycle follow-up — 2026-09-25

- Started the installed Windows NSIS app with no existing `ts-desktop.exe`
  process or listeners on 9090/9091. The packaged app returned `/healthz` 200
  and unauthenticated `/v1/status` 401.
- Sent `WM_CLOSE` to the exact visible `OpenSwarmLayer` main-window HWND. The
  window became not visible while the app process remained alive and `/healthz`
  stayed 200. This verifies the close-to-background handler and engine
  persistence by native window-message/API readback; it was not a physical
  mouse-click test.
- Read the Windows notification-area registration for the installed executable
  (`InitialTooltip: TensorSwarm`, UID 2), opened its native tray menu, and
  visually confirmed `Show TensorSwarm` and `Quit`. The native menu exposed
  `Quit` as command ID 1001.
- A coordinate-based attempt to select Quit activated the Windows keyboard
  layout flyout instead; it is not counted as a successful menu click. A
  `WM_COMMAND` for the verified Quit item was dispatched, but immediate
  readback still showed the app online. Later readback found PID 26808 gone,
  `/healthz` unavailable, and ports 9090/9091 closed. Because the app's exit
  could not be causally tied to selecting Quit from the menu, explicit tray
  Quit/graceful-shutdown acceptance remains inconclusive and open.
- The desktop UI automation tool was unavailable in this session. No process
  was force-stopped after the menu attempt. No source code changed; no commit
  or push was made. The pre-existing untracked `scripts/` directory remains
  untouched.

## Windows tray Quit and native Deepin package follow-up — 2026-09-25

- Retested the installed Windows NSIS app from a clean process state. `/healthz`
  returned 200 and unauthenticated `/v1/status` returned 401. Sending `WM_CLOSE`
  hid the main window while the process and health endpoint remained live.
- Opened the app's actual native tray menu through its tray callback because
  desktop UI automation was unavailable. A fresh screenshot showed `Show
  TensorSwarm` and `Quit`; moving the pointer over the Quit row visibly
  highlighted it. Clicking that row with a Win32 mouse down/up event exited the
  app without force termination. Readback showed the process gone and ports
  9090, 9091, and 60659 closed. This verifies the native menu's Quit handler;
  physically right-clicking the notification-area icon was not separately
  exercised.
- Prepared a tracked-source archive from commit `3194bfc` and transferred it to
  an isolated Deepin 25 acceptance directory; local and remote archive SHA-256
  both equaled `b65cee534a2de62bfb324ab1da98b4679d7008a7d26ae791aa15943756dbbc2a`.
  Native `cargo check -p ts-desktop` passed. The native DEB build command
  exceeded the terminal's 420-second limit, but the process continued and
  produced a valid `OpenSwarmLayer_0.1.1_amd64.deb`; `dpkg-deb` verified package
  metadata/content, and `ldd` found no unresolved dependencies on Deepin
  (glibc 2.38).
- The DEB is not installed: `sudo -n` is unavailable. It was extracted into a
  test-owned directory, and launch was requested in the active Xorg session
  with isolated XDG data/config/cache paths (reported PID 24837). SSH and ping
  to the notebook timed out immediately afterward, before process, window,
  `/healthz`, protected API, or tray readback. Do not count this as a successful
  native Linux runtime launch; the test process may require cleanup when the
  host is reachable again.
- Native Linux package installation, live GUI/API readback, and tray Quit remain
  open. No project source changed, no commit or push was made, and the
  pre-existing untracked `scripts/` directory remains untouched.

## Model metadata, Internet discovery, and abuse-resilience research — 2026-09-25

- Researched official Hugging Face model-card/release metadata and GGUF
  documentation, SPDX license identifiers, BitTorrent BEP 3/5/9 discovery
  semantics, Hugging Face content reporting, and model-file security guidance.
  The cited research note is `docs/model-discovery-publication-and-abuse.md`;
  its evidence citations pass `sources.py verify --evidence`.
- Current-code review found that `Manifest` binds format, file size, tensor
  descriptors, chunks, file recipes, and a root (`crates/ts-core/src/lib.rs`),
  while daemon model status exposes only basic file metadata and root. Existing
  Kademlia provider lookup is keyed by exact manifest/tensor hashes
  (`crates/ts-p2p/src/lib.rs`); there is no user-searchable public catalog.
- Added a proposed M7 roadmap for a model-card/release/variant schema, rights
  and abuse/privacy gates, signed descriptors and share links, and then a
  searchable directory plus expiring peer-rendezvous leases. This distinguishes
  a model catalog from a torrent-style tracker and does not commit to a URI,
  service operator, or federation model.
- Planning guardrails now state that hashes, supported formats, and signatures
  do not prove safety, legality, or redistribution rights; legal review and
  reporting/delisting/appeal procedures are required before public publishing.
  Updated `PLAN.md`, `TASKS.md`, `CONTEXT.md`, `ARCHITECTURE.md`, and `README.md`.
- This was a documentation/research slice only; no implementation or public
  service was created. No commit or push was made. The pre-existing untracked
  `scripts/` directory and prior uncommitted acceptance notes remain preserved.

## M7-01 model publication schema slice — 2026-09-25

- Added `crates/ts-core/src/publication.rs` with versioned `ModelCard` and
  `ModelReleaseDescriptor` schemas, license/provenance claims, per-variant
  quantization/format, and per-file manifest roots, sizes, tensor counts, and
  shard positions. A release can list multiple files and mixed GGUF/Safetensors
  variants without changing the existing `Manifest` structure or root.
- Added validation for known schema versions, non-empty identities, safe
  release-relative paths, duplicate variants/paths, non-empty artifact
  references, and consistent shard groups. Unknown serialized fields are
  rejected. Review/moderation state remains separate from publisher-controlled
  cards, and metadata-origin labels remain untrusted until locally verified.
- Added tests for mixed-format/multi-shard CBOR round-trip, invalid paths and
  shard states, unknown fields, schema rejection, and the existing manifest
  root's fixed golden value. The focused `cargo test -p ts-core` run passed 4
  tests; the initial test-first run failed on the missing schema types as
  expected. Final V0 verification passed: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace` (CLI process acceptance
  1; core 4; daemon 5 unit + 3 process; desktop 8; format 9; P2P 14; proxy 10;
  store 7), `cargo build --workspace`, and `git diff --check`.
- Updated `DESIGN.md`, `TASKS.md`, and the research note with the schema and
  trust-boundary details. No catalog, persistence, UI, signature, or public
  sharing service was added; M7-02 remains a hard gate for public publishing.
- No commit or push was made. The pre-existing untracked `scripts/` directory
  and earlier documentation/acceptance modifications remain preserved.

## M7-02 abuse/privacy policy draft — 2026-09-25

- Drafted `docs/model-sharing-abuse-policy-draft.md` for owner and qualified
  legal review. It proposes the public service scope, publisher rights
  attestations, separation of technical verification from policy review,
  report/takedown/appeal workflow, tracker privacy controls, and an adversary
  matrix with future acceptance probes and residual limitations.
- The draft explicitly excludes movies, pornography, and other non-model media;
  recommends keeping explicit-content models out of the initial public catalog
  unless separately approved; and does not claim that hashes, signatures, or
  parser acceptance establish legal rights or safety. It also states that a
  managed delisting cannot recall content already cached or shared elsewhere.
- Updated `TASKS.md`, `PLAN.md`, `README.md`, and the research note with the
  draft link and `IN_PROGRESS` status. No policy was approved and no enforcement,
  registry, or tracker was implemented. Owner approval, qualified legal review,
  operational coverage, and privacy/retention decisions are still outstanding;
  M7-03/M7-04 remain gated.
- Documentation checks and citation verification are recorded after completion
  below. No commit or push was made; existing dirty files and `scripts/` remain
  preserved.

## M7-03 local signed-descriptor codec — 2026-09-25

- After asking for an explicit policy decision, the owner approved the draft's
  product-policy direction and authorized local-only M7-03 work. Qualified
  legal review and final operations/privacy decisions remain outstanding; this
  does not enable public publishing or close M7-02.
- Added Ed25519 `SignedModelRelease` support in `crates/ts-core`: a domain-
  separated signature binds schema, self-asserted public key, and CBOR
  descriptor bytes including all artifact manifest roots. Envelope export and
  import validate the descriptor/schema/key/signature and enforce a 4 MiB
  serialized-size limit. Caller supplies the key; this core does not generate,
  persist, or recover private keys.
- Added `ModelShareLink` encode/parse for `tswarm://v1/<sha256>`. The digest
  pins the canonical serialized signed envelope; an imported envelope can be
  checked against the link. There is no OS deep-link handler, UI, network
  resolver, directory, or fetch path. A valid signature/link does not prove
  publisher identity, redistribution rights, lawful content, safety, or the
  actual artifact bytes; clients still verify content against each manifest
  root.
- Added adversarial tests for altered descriptor/root/signature, unsupported
  schema, oversized input, unknown envelope fields, malformed share URI,
  signature-shape errors, envelope mismatch, and signed-envelope/link round-trip.
  TDD red phase produced the expected missing-type compile error before
  implementation. Final checks passed: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace` (CLI process 1; core 9;
  daemon unit 5 + process 3; desktop 8; format 9; P2P 14; proxy 10; store 7),
  `cargo build --workspace`, and `git diff --check`.
- Updated `DESIGN.md`, `TASKS.md`, `PLAN.md`, the policy draft, and research
  roadmap with local-only scope and the remaining public/legal gates. No commit
  or push was made; prior dirty work and untracked `scripts/` remain preserved.

## M7-03 OS key lifecycle and local descriptor UI — 2026-09-25 (in progress)

- Chose a device-local Ed25519 signing identity in the OS credential store. The
  desktop uses `keyring` with native Windows/macOS stores and Linux Secret
  Service, generates the key only after an explicit UI confirmation, exposes a
  SHA-256 public-key fingerprint, runs synchronous keyring operations on
  blocking workers, and fails closed without a plaintext fallback. There is no
  private-key export, backup, recovery, or rotation; store loss creates a new
  identity without continuity.
- Added native Tauri commands for checking/creating the identity, signing a
  descriptor from a verified local single-file manifest, exporting a bounded
  `.tsrelease` file, and previewing an imported signed descriptor. Import checks
  the signature and compares descriptor path/size/format/tensor count/root to
  local manifest metadata; it does not re-hash all artifact bytes or persist an
  imported catalog record.
- Added Settings UI for key status/fingerprint and explicit key creation, plus
  model-card claim fields, signed-bundle export, and import preview. The UI
  warns that license fields are claims, the embedded key is self-asserted, the
  share identifier does not resolve, and the descriptor bundle must be shared
  separately.
- Verification: Windows `cargo test --workspace` passed 75 tests;
  `cargo check --workspace`, `cargo build --workspace`, and
  `cargo tauri build --debug --no-bundle` passed. The embedded UI JavaScript
  passed `node --check`. Ubuntu WSL passed `cargo check -p ts-desktop` and
  `cargo test -p ts-desktop` (17 tests), establishing Linux compilation but not
  a live Secret Service or GUI walkthrough.
- Live Windows M7-03 identity acceptance on 2026-09-25: the Settings UI first
  showed `No key created`; the explicit create action stored a device-local key
  and visibly changed status to `Ready` with a public fingerprint. After
  stopping and restarting the test-owned desktop process, Settings showed the
  same fingerprint and `Ready`, confirming credential-store persistence. No
  private key was displayed. The test-owned process was stopped afterward and
  listeners 9090/9091 were absent.
- M7-03 remains `IN_PROGRESS`: live signed-bundle export/import and preview
  readback, plus live Linux Secret Service/native packaging acceptance, remain
  open. Native UI text entry was unreliable through the available automation
  path, so no release bundle was exported or imported and no live tamper result
  is claimed. Automated desktop tests cover valid envelopes, byte tampering,
  oversize rejection, acceptance of a valid release signed by another
  self-asserted key, and rejection when the embedded public key is substituted
  without resigning. There is no trust anchor, so another self-asserted signer
  is not inherently invalid.
- Verification checkpoint (2026-09-25): `cargo fmt --all -- --check`,
  `cargo test -p ts-desktop` (18 passed), `cargo check -p ts-desktop`, and
  `git diff --check` passed after adding the signer-key substitution regression
  test. These are automated checks, not live export/import acceptance.
- `cargo-audit` initially found the baseline `rustls` and `hickory-proto`
  vulnerabilities; locked `rustls` is now 0.23.45. Updated all workspace
  `libp2p` constraints from 0.56 to 0.57, moving mDNS to `hickory-proto` 0.26.3
  and resolving both Hickory advisories. `libp2p` 0.57 declares Rust 1.88 as
  its minimum; the host toolchain is Rust 1.98.1. Current `cargo audit` exits 0
  with no vulnerabilities; ten unmaintained/unsound warnings remain. `cargo fmt`,
  workspace check/test/build, the Windows Tauri debug build, and Ubuntu WSL
  workspace check/test all pass; WSL ran 75 tests, including the independent
  process P2P and daemon acceptance tests. `ts-p2p` focused tests pass (14).
- Public directory, resolver, and Internet publishing remain blocked by the
  outstanding M7-02 legal/operational review. No commit or push was made;
  existing dirty work and the untracked `scripts/` directory remain preserved.

## Multi-model inventory and selected preparation — 2026-09-26 (in progress)

- Identified that the daemon previously returned the combined catalog root and
  aggregate tensor count for every `/v1/models` row; the desktop used aggregate
  availability for every model and selected-file preparation silently wrote the
  first catalog file. Added per-file root/count/availability projection and
  exact-path selection to `/v1/prepare`; single-file requests without `path`
  retain compatibility. Missing/unknown paths in a multi-file catalog reject
  without creating output. Verification now reads and hashes CAS objects rather
  than treating an existing object filename as proof of validity.
- Added a router regression with two distinct models and one missing chunk,
  corrupted-object readback, bad and omitted path rejection, and byte-exact
  preparation of the second model. UI library rows and detail now use the
  per-file values; overview verification remains catalog-wide. Blocking
  inventory/verification/preparation work runs off the async request executor.
- V0: `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo test --workspace`, `cargo build --workspace`, Windows
  `cargo tauri build --debug --no-bundle`, inline UI `node --check`, and
  `git diff --check` passed. The focused test failed before the fix and passed
  afterward. Router/API tests are not native UI or packaged-install acceptance.
- V3 still open: native UI multi-model import/status/selection and exact
  prepared-file readback, live signed-bundle export/import, native Linux
  install/runtime/tray and Linux packaged preparation. Deepin host did not
  respond to a single ping during this slice. Public directory/trackers remain
  gated by M7-02 legal and operational approval. No commit or push; pre-existing
  dirty and untracked work was preserved. Recommended next slice is live
  Windows signed-bundle export/import and multi-model UI readback.

## Offline signed-release/manifest check — 2026-09-26 (in progress)

- Added `ts-cli verify-release <bundle.tsrelease> <manifest.tswarm>
  [tswarm://v1/<digest>]`. It bounds input before decoding, checks the
  self-asserted signature and optional envelope identifier, verifies the
  manifest root, and matches path/format/size/tensor count/root against a
  referenced release artifact. The same manifest-match predicate now serves
  desktop import preview and CLI verification. It does not fetch or hash model
  bytes, persist a recipient record, or establish legal rights or identity.
- Extended independent-process CLI acceptance: real node/fetch/prepare plus
  signed descriptor verification, optional identifier, mismatched path/root,
  malformed/wrong identifier and tampered-bundle rejection. The test failed
  before the new command existed and passed after implementation. This is a
  private file-mediated prerequisite, not the two-user private sharing pilot.
- V0: `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo test --workspace`, `cargo build --workspace`, `git diff --check`,
  and Windows `cargo tauri build --debug --no-bundle` passed. Native UI
  export/import and a second independent user's exchange remain unverified.
  This CLI command alone does not complete M7-06.

## Selected-model desktop manifest export — 2026-09-26 (in progress)

- The Model library now exports the selected imported model's single-file
  root-verified `.tswarm` manifest through a native save picker. The write
  refuses an existing destination and reads back the exported bytes; it does
  not contain model bytes. Signed descriptor export uses the same exact-path
  local selection helper. A two-model unit test verifies the second model's
  root/recipe, overwrite refusal, and unknown-model failure without output.
- V0: focused desktop test, `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace`, `cargo build --workspace`, inline
  JavaScript `node --check`, `git diff --check`, and Windows
  `cargo tauri build --debug --no-bundle` passed. Live picker/export and
  recipient import remain unverified; this does not close the private pilot.

## Offline recipient metadata inbox — 2026-09-26 (in progress)

- Added `ts-cli receive-release <bundle.tsrelease> <manifest.tswarm>
  <inbox-dir> [tswarm://v1/<digest>]` using the same bounded verification as
  `verify-release`. Valid signed metadata is staged in a temporary directory
  then moved into an immutable release-ID-named inbox record. Repeat receipt
  of identical bytes succeeds; an existing mismatched record is not replaced.
  A recipient record is not activated in the model library and contains no
  downloaded or verified model bytes.
- Independent-process acceptance rejects tampered bundles, mismatched manifests
  and wrong identifiers before inbox creation, then reads back the saved files
  and checks idempotency and nonreplacement after corruption. The test failed
  before the command was added and passed afterward. Live two-user transfer,
  peer discovery bound to the signed root, desktop recipient UI, and native
  Linux acceptance remain open.
- V0: `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo test --workspace`, `cargo build --workspace`, and `git diff --check`
  passed on Windows. No native recipient UI or second-host acceptance was run.

## Signed-root-bound CLI recipient fetch — 2026-09-26 (in progress)

- Added `ts-cli fetch-received <record-dir> <peer-id> <peer-address>
  <store-root> <output>`. The command rechecks the bounded signed release and
  single-file manifest, requires the record directory to match the release ID,
  fetches missing chunks from the explicitly provided peer, checks their
  length and hash before storage, and prepares the file from the recipient's
  independently verified cache. It refuses pre-existing output/partial paths
  and fails closed on a corrupted cached object. Unknown CLI commands now
  fail rather than printing usage and exiting successfully.
- Independent-process integration exercises the provider and recipient CLI,
  byte-equal prepared output and cache readback, offline cache reuse after
  provider shutdown, wrong record directory, corrupted CAS and manifest, and
  negative unknown-command status. Node cleanup is guarded even on assertion
  failure. Focused integration test, workspace format/check/test/build, and
  `git diff --check` passed on Windows.
- This is an explicit-address local-process transfer, not independent peer
  discovery or a second-host two-user pilot. Desktop recipient UI, live
  signed-bundle picker acceptance, native Linux packaging, and public-service
  gates remain open; no commit, push, or deployment.

## Desktop private recipient inbox — 2026-09-26 (in progress)

- Added the Model library's separate recipient action: optional independently
  supplied `tswarm://` identifier, native bundle and manifest pickers, bounded
  signature/root/descriptor matching, then staged metadata persistence under
  app data `models/inbox/<release-id>/`. It never activates the manifest or
  model bytes in the local engine. UI reports a metadata-only inbox receipt.
- Unit test covers invalid identifier, a valid but mismatched manifest, valid
  save/readback, idempotent repeat, mismatched existing record rejection, and
  absence of active-library mutation. The focused test failed before the
  implementation and passed after it. Workspace format/check/test/build,
  HTML nesting/control check, inline JS syntax, and Windows Tauri debug
  no-bundle build passed. No live native picker/readback was exercised.
## Signed-root-bound CLI recipient fetch — 2026-09-26 (in progress)

- Added `ts-cli fetch-received <record-dir> <peer-id> <peer-address>
  <store-root> <output>`. The command rechecks the bounded signed release and
  single-file manifest, requires the record directory to match the release ID,
  fetches missing chunks from the explicitly provided peer, checks their
  length and hash before storage, and prepares the file from the recipient's
  independently verified cache. It refuses pre-existing output/partial paths
  and fails closed on a corrupted cached object. Unknown CLI commands now
  fail rather than printing usage and exiting successfully.
- Independent-process integration exercises the provider and recipient CLI,
  byte-equal prepared output and cache readback, offline cache reuse after
  provider shutdown, wrong record directory, corrupted CAS and manifest, and
  negative unknown-command status. Node cleanup is guarded even on assertion
  failure. Focused integration test, workspace format/check/test/build, and
  `git diff --check` passed on Windows.
- This is an explicit-address local-process transfer, not independent peer
  discovery or a second-host two-user pilot. Desktop recipient UI, live
  signed-bundle picker acceptance, native Linux packaging, and public-service
  gates remain open; no commit, push, or deployment.
- Private two-user live acceptance, automatic signed-root peer discovery,
  native Linux package acceptance, and public-service policy gates remain open.
- The focused test now passes after implementation. Workspace format/check/test/build,
  HTML nesting/control check, inline JS syntax, and Windows Tauri debug no-bundle build passed. No live native picker/readback was exercised.

## Continuation verification — 2026-09-27

- On the current Windows worktree, `cargo fmt --all -- --check`,
  `git diff --check`, `cargo check --workspace`, and `cargo test --workspace`
  passed. The workspace suite included CLI process transfer/signature checks,
  daemon process acceptance, 20 desktop tests, and the parser, P2P, proxy, and
  store suites. Focused CLI process, desktop, and daemon test runs also passed.
  The UI's single inline JavaScript block passed `node --check`.
- The Ubuntu-built DEB was not used for Deepin acceptance: the extracted binary
  requires `GLIBC_2.39`, while the native host is Deepin with glibc 2.38.
- Built a replacement DEB on the Deepin host from a source archive of the
  current worktree (excluding machine-local configuration and untracked
  `scripts/`). Archive SHA-256:
  `033440879db0bdc8fe0ef0bb7e5c2f58df8759f6014233f8c095a371ebb8f09d`.
  The native `OpenSwarmLayer_0.1.1_amd64.deb` passed package metadata checks;
  its SHA-256 is
  `9edeecf4a80e3294e842733a480ff07fa3626dca9d4fa33f262faa982aebe7d6`.
  `ldd` found no unresolved libraries and no GLIBC 2.39 requirement.
- Passwordless sudo remains unavailable, so the package was not installed.
  Extracted the package to a test-owned directory, launched its desktop binary
  with isolated XDG data/config/cache paths, and read back `/healthz` HTTP 200,
  unauthenticated `/v1/status` HTTP 401, and listeners on 9090/9091. Sent
  SIGTERM to the test-owned process; it exited and both endpoints/listeners
  disappeared. This is native process/API evidence only—not visible UI, model
  import/preparation, package installation/removal, or tray acceptance.
- Current-source acceptance still open: install/uninstall and visible native
  Linux UI/tray checks; packaged multi-model UI selection/preparation (M6-09);
  live signed-release export/import and preview plus the independent-user
  private exchange (M7-03/M7-06). Public directory/rendezvous work remains
  blocked pending M7-02 qualified legal and operational review. No commit or
  push was made; the pre-existing dirty worktree and untracked `scripts/` were
  preserved.

## Two-host private-sharing smoke — 2026-09-27

- Confirmed SSH access to the Deepin 25 test host as case-sensitive user
  `Stephen@192.168.5.68`. Its existing `master` checkout was clean at `9f62701`;
  it was not changed. The current Windows worktree was at `3194bfc` with its
  pre-existing dirty files preserved.
- Built an isolated source snapshot from local `HEAD` plus only the current
  CLI/core/P2P/proxy changes and the new `ts-core` publication module. The
  snapshot archive SHA-256 was
  `73de0520fcd7857096dac6f523d69f725c3721a85ffbc0352bdbbd6d1aae074c`; the
  remote readback matched. In that temporary Linux snapshot,
  `cargo test --offline -p ts-cli --test process_acceptance -- --test-threads=1`
  passed (1 test, 0 failed; 2m19s). The remote working checkout remained
  untouched.
- Generated a test-owned 130-byte synthetic Safetensors artifact, root-verified
  manifest, CAS chunk, and signed release using a deterministic test key (not
  either host's OS signing identity). Transferred the `.tswarm` and `.tsrelease`
  files separately to an isolated Deepin `/tmp` inbox; local/remote SHA-256
  readback matched for both files. Linux `verify-release` and `receive-release`
  succeeded, and the resulting inbox contained one separate record with only
  the expected manifest and signed descriptor. A wrong expected share ID was
  rejected without creating another inbox.
- Ran a Windows CLI P2P provider on `192.168.5.229`; Deepin CLI fetched from its
  explicitly supplied peer ID and LAN address into a separate cache/output.
  Sender and recipient artifact SHA-256 both equaled
  `c9a3370799eb2df12aefff65dbeb82909ef131390ee577c7f0c2ccc1057b536b`.
  After stopping the Windows provider, a second Deepin fetch succeeded from the
  verified local cache and produced the same hash. The Windows TCP listener then
  refused connections (`connect_ex=10061`).
- Removed the temporary fixture, helper source, and both remote `/tmp` staging
  directories. Also found a lingering desktop process from an earlier isolated
  acceptance run; verified its executable and XDG paths were under the
  test-owned acceptance directory, sent SIGTERM, and read back zero
  `ts-desktop` processes and zero listeners on 9090/9091. No source checkout was
  cleaned, committed, or pushed.
- This demonstrates same-operator, two-host CLI receipt, explicit-peer fetch,
  wrong-ID rejection, and offline cache reuse. It is not an independent-user
  pilot; the signer was a deterministic test key, and the native Windows
  signed-release picker/import UI and Linux package installation remain open.
  M7-03, M7-06, M6-09, and native Linux install/tray acceptance are still
  incomplete; public publishing remains blocked on M7-02 legal and operational
  signoff.

## Current-source Deepin package build — 2026-09-27

- Refreshed Windows verification after removing the temporary fixture generator:
  `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo test --workspace`, `cargo build --workspace`, and `git diff --check`
  passed. The desktop suite ran 21 tests; the CLI/daemon process suites ran 1
  and 3 tests respectively. No temporary example source remains.
- From the dirty Windows worktree at baseline `3194bfc`, staged an 85-file
  temporary source snapshot limited to Cargo manifests/lock plus `crates/`,
  `desktop/src-tauri/`, `desktop/ui/`, and the three new Rust modules. Snapshot
  SHA-256: `fac7060f9d4aad01d4f0b7122af44d40b6d5d68b8e98872525ad878d20238e4f`.
  Neither the Deepin Git checkout nor the Windows worktree was modified by the
  snapshot operation.
- On Deepin 25 x86_64 with Rust/Cargo 1.98.1 and Tauri CLI 2.11.4, built
  `OpenSwarmLayer_0.1.1_amd64.deb` successfully from that snapshot. Package
  SHA-256: `a47fb65c6b40935b75d9a6d6ade1f9b66967899364533314a72bdd1fc7db4986`.
  The package was 110,345,698 bytes; extracted `usr/bin/ts-desktop` was
  433,171,288 bytes. `ldd` reported no missing libraries, and maximum required
  `GLIBC_2.38` matched the host's glibc 2.38. This is package-build/preflight
  evidence only, not package-install acceptance.
- First packaging attempt filled Deepin's 3.9 GiB `/tmp` tmpfs while staging
  the 433 MB unstripped debug binary; the build had compiled before bundling
  failed with ENOSPC. Moved only this task's build output to its unique
  `$HOME/.cache/tensorswarm-roadmap-fac7060f/` directory and set `TMPDIR`,
  `TMP`, and `TEMP` there; the clean rebuild and DEB bundle then passed. `/tmp`
  returned to 1% use. The build target/cache remains in the named scratch path.
- `sudo -n true` failed on Deepin, so no package installation/removal was
  attempted. An isolated Xorg launch of the extracted binary was started as PID
  52125 with XDG config/data/cache/state under the same task-owned cache, but
  SSH then timed out before process, listener, visible-window, or API readback.
  Do not report the launch as successful or verified; the isolated app may still
  be running, and the remote scratch/cache could not yet be cleaned or stopped.
  The Deepin source checkout was not used or changed.
- Remaining owner/platform gates: restore Deepin reachability and stop/read back
  only PID 52125 (confirm executable path first); obtain owner-authorized package
  install/remove interaction; verify native UI/import/persistence/tray; exercise
  Windows signed-release pickers without silently creating a user-global OS
  credential; and recruit a genuinely independent recipient for M7-06. Public
  publishing remains blocked pending qualified legal and operational review.

## Isolated Windows Tauri startup probe — 2026-09-27

- Built the current Windows desktop with a one-off Tauri config identifier
  `org.tensorswarm.desktop.acceptance.20260927`, keeping app data separate from
  the default profile. Launched PID 43520; `/healthz` returned HTTP 200, and the
  process owned listeners on 9090/9091. The signing-identity check is read-only;
  no key was created or changed.
- WebView2 did not expose the requested remote-debugging port 9229, and Windows
  UI Automation found only two pane elements rather than interactive controls.
  Native release/model picker actions were not exercised; this is process/API
  smoke only, not M7-03 or M6-09 acceptance.
- Verified PID 43520 still pointed to this checkout's `target/debug/ts-desktop.exe`,
  force-stopped only that test-owned process, and confirmed ports 9090/9091 were
  closed. Removed only the two identifier-specific AppData profiles. The default
  TensorSwarm profile and OS credential were not touched. No code files changed.

## Isolated Windows native window render readback — 2026-09-27

- Built and launched the desktop with unique Tauri identifier
  `org.tensorswarm.desktop.acceptance.20260927c`; exact PID 45968 ran this
  checkout's `target/debug/ts-desktop.exe`. `/healthz` returned 200 and that
  PID owned listeners 9090/9091.
- Captured the background app window with Win32 `PrintWindow` (screen-copy
  initially captured an unrelated topmost window and was discarded). The
  screenshot showed Overview, node health Online, running P2P/proxy, and an
  empty model library. This verifies native window rendering only; no click,
  file picker, model import, signing-key read, or signed-release action occurred.
- Stopped only the checked test PID, confirmed it was gone and listeners
  9090/9091 were closed, then removed only that identifier's Local/Roaming
  AppData profiles and task-owned screenshot/log/config scratch. No key was
  created or touched.

## M6-09 partial-cache startup acceptance — 2026-09-27

- A stronger live daemon-process fixture exposed a startup defect: the daemon
  builds its LAN `ChunkProvider` from the complete catalog and rejected startup
  when any referenced chunk was missing. Changed `ChunkProvider::from_manifest`
  to skip only `Missing` and `HashMismatch` store errors, retaining manifest
  availability while admitting verified chunks only. Other store I/O errors and
  verified chunks with invalid lengths remain hard failures. The daemon and CLI
  now announce a tensor in DHT only when the local provider has at least one
  verified chunk for it, avoiding false provider leases for wholly absent/corrupt
  tensors.
- Added a P2P regression test with one missing and one corrupt object. The
  manifest remains available, while requests for both unavailable chunks return
  `NotFound`. Expanded `daemon_process_tracks_and_prepares_each_selected_model`
  to start the real daemon with two complete models and a third missing-chunk
  model; authenticated inventory reports 2/2, 2/2, and 0/1 availability, the
  incomplete model cannot be prepared, omitted/unknown paths do not create
  outputs, and both complete selections produce byte-exact files.
- Test-first run failed because the provider returned `StoreError::Missing`; the
  focused provider and daemon-process tests passed after the fix. Full checks
  passed: `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo test --workspace` (15 P2P tests, 6 daemon unit tests, 4 daemon process
  tests, and 21 desktop tests among the workspace suites),
  `cargo build --workspace`, and `git diff --check`.
- This is provider-unit and independent daemon-process/API evidence, not
  packaged UI acceptance; M6-09 live multi-model UI readback remains open.

## Runtime provider freshness after transfer — 2026-09-27

- Fixed the daemon's startup-snapshot provider gap: `ChunkProvider` now shares a
  synchronized inventory across the daemon control handler and P2P loop. The
  authenticated transfer route validates the requested tensor/index/hash
  against the active manifest, checks fetched length, commits verified bytes to
  CAS, inserts them into the live provider, and queues tensor DHT publication.
- Added `daemon_serves_a_chunk_after_transfer_commits_it`: the real daemon starts
  with the chunk missing, rejects a wrong manifest hash, receives the valid
  chunk from a source peer, persists it, then serves it to an independent peer;
  a third peer's DHT lookup returns the daemon for that tensor.
- Independently reran the focused process test: 1 passed. Full workspace
  verification passed: `cargo fmt --all -- --check`, `cargo test --workspace`
  (including 5 daemon process tests, 15 P2P tests, 10 proxy tests, and 21 desktop
  tests), `cargo check --workspace`, `cargo build --workspace`, and
  `git diff --check`. The post-run `cargo test -p ts-proxy` rerun passed all 10
  proxy tests after removing the unused `mut` warning.
- This closes provider freshness for `/v1/transfers`. At that point the runtime
  proxy/WebSeed callback was still a separate gap. Packaged UI and native host
  acceptance remained independently tracked.

## 2026-09-27 — Runtime proxy provider freshness

- Extended verified-fetch notifications to carry the manifest tensor hash,
  chunk index, chunk hash, and verified bytes. Wired the daemon's manifest proxy
  to admit notifications only through active-manifest validation, update the
  live P2P provider inventory, and queue tensor DHT publication.
- Added proxy coverage for exact notification identity and daemon coverage that
  a verified manifest fetch becomes immediately P2P-servable and queues DHT
  publication; a forged/wrong-manifest chunk is neither inserted nor announced.
- Strengthened `https_fallback_publishes_provider_for_second_node_discovery`:
  a verified HTTPS WebSeed fetch inserts the manifest-identified chunk into a
  live provider, a distinct tensor hash is discovered through a separate DHT
  peer, and a separate `LanClient` retrieves byte-identical content. The test
  passed once and then passed three consecutive repeat runs.
- Added daemon-runtime HTTP acceptance using an injected TLS trust client: the
  in-process runtime serves a manifest-backed HTTPS WebSeed, returns exact
  bytes, updates authenticated `/v1/verification`, serves a subsequent byte
  range from CAS without another origin request, and releases both HTTP
  listeners on shutdown.
- Focused callback tests passed: `cargo test -p ts-proxy
  manifest_fetch_notifies_with_verified_chunk_identity` (1 passed) and
  `cargo test -p ts-daemon
  proxy_verified_fetch_refreshes_runtime_provider_before_announcement` (1
  passed). Full workspace verification after the strengthened swarm test passed:
  `cargo fmt --all -- --check`, `cargo test --workspace` (8 daemon unit tests, 5
  daemon process tests, 15 P2P tests, 11 proxy tests, plus CLI/core/desktop/
  format/store suites), `cargo check --workspace`, `cargo build --workspace`,
  and `git diff --check`. The swarm test uses real local TLS and libp2p sockets
  in-process and proves third-peer DHT discovery/fetch. The runtime HTTP test
  also runs in-process with an injected test trust client; it does not spawn the
  daemon binary or a native UI.

## 2026-09-27 — Prevent fetch-only peers from advertising availability

- Reproduced a false provider announcement: `LanClient::fetch_chunk` called
  `publish_tensor` after download verification, although `LanClient` does not
  serve inbound chunk requests. The same invalid path was wired into the
  fetch-only `ts-cli proxy-manifest` command.
- Added a regression assertion to the real two-node transfer test. Before the
  fix it observed a Kademlia `StartProviding` event from the fetcher and failed;
  after removing client-side publication, the transfer passes without an
  announcement. Removed the CLI callback for the same reason; daemon publication
  remains tied to its manifest-validated live provider inventory.
- Verification passed: `cargo fmt --all -- --check`, all `ts-p2p` tests (15),
  `ts-cli` unit/process tests, all `ts-proxy` tests (11),
  `cargo check --workspace`, and `git diff --check`. No full workspace test/build
  was run for this correction.

## 2026-09-27 — Exercise runtime WebSeed-to-third-peer publication

- Extended the daemon's in-process HTTPS runtime acceptance to inject a loopback
  libp2p listener and connect an independent third `LanClient` before the
  manifest-backed WebSeed fetch.
- After the HTTP fetch, the third peer finds the daemon as a provider for the
  manifest tensor through Kademlia and retrieves the chunk byte-exactly over
  P2P. The test still checks authenticated verified-chunk readback, CAS range
  reuse without another origin request, and runtime listener shutdown.
- The focused `runtime_manifest_proxy_fetches_verified_https_bytes` test passed
  once, then three consecutive repeat runs. This is in-process runtime/P2P
  evidence; it does not spawn the daemon binary or exercise the desktop UI.

## 2026-09-27 — Isolated Windows signed-release UI acceptance

- Built and launched the Windows Tauri debug app with a unique application
  identifier and isolated app-data roots. `/healthz` returned HTTP 200. The
  visible Model library imported a synthetic 198-byte Safetensors fixture; UI
  readback showed 1 tensor, 1/1 verified chunks, and the import confirmation.
- Read Settings without changing the OS credential store: an existing signing
  identity was `Ready`. No key-creation action was invoked.
- Used the native save pickers to export the selected model's `.tsrelease`
  bundle and `.tswarm` manifest to test-owned scratch files. The files read back
  at 730 and 713 bytes respectively. The UI reported a verified signature and
  1/1 matching local manifest roots. `ts-cli verify-release` exited 0 with the
  exported files and the exact share identifier; its result explicitly says
  artifact bytes were not checked. Existing-destination refusal is covered by
  the automated export regression, not by this live picker run.
- Used the native Open picker to import the signed descriptor for preview. The
  UI reported `Imported descriptor preview`, signature verification, and 1/1
  matching local manifest roots. It also stated the preview was not saved to a
  catalog; the active library remained the one synthetic fixture. This did not
  verify model bytes, signer identity, redistribution rights, or safety.
- Completed the native recipient-inbox flow with the exact share identifier,
  exported bundle, and matching manifest. The UI reported a verified signature,
  saved the receipt, and stated that no model bytes were fetched and the record
  was not in the active library. Its generic local-library match counter showed
  0/1, consistent with the inbox record remaining separate from the active
  library; the supplied bundle/manifest pair passed receipt validation.
- Stopped and relaunched the exact isolated debug app. After restart, `/healthz`
  and proxy health returned HTTP 200; the Model library again listed the
  `Received` metadata-only receipt and still showed only the one synthetic
  model at 100% verified availability. Read-only filesystem inspection found
  the persisted bundle and manifest in the unique-identifier inbox profile.
- Cleanup completed: stopped only the verified test PID, confirmed ports
  9090/9091 were released, removed the unique-identifier Local/Roaming profiles,
  isolated scratch root, synthetic artifact, exported metadata, screenshots,
  and test helper scripts. The default TensorSwarm Roaming profile remained
  present, and no signing-key creation action was invoked.
- Live Windows export, descriptor preview, recipient receipt, and restart
  readback are now evidenced, but M7-03 remains open for adversarial UI cases
  (tampered bundle, wrong manifest/identifier, and destination conflicts) and
  Linux Secret Service/native acceptance. The independent-user M7-06 pilot also
  remains open. No source code changed; no commit or push was made.

## 2026-09-27 — Commit and push verified working set

- Re-ran `cargo fmt --all -- --check`, `cargo test --workspace`,
  `cargo check --workspace`, `cargo build --workspace`, and `git diff --check`;
  all passed on Windows. A staged credential-pattern scan found no matches.
- Committed the reviewed tracked changes and required new source/docs as
  `bf5ce6f` (`feat: add signed private model release sharing`, 32 files).
  The untracked `scripts/` directory was deliberately left out.
- Pushed `master` to `origin`. `git ls-remote` read back the exact pushed
  commit `bf5ce6f0257073b4891b9b7845a47d9887b30adb`; local ahead/behind is
  0/0. The only remaining working-tree item is untracked `scripts/`.

## 2026-10-03 — Live three-machine LAN download and re-seeding

- Owner clarified the beta acceptance target: a desktop at one Internet location
  seeds a model, a desktop at another downloads/prepares it, and the recipient
  serves it to a third peer after the original seed stops. UI polish is secondary
  to this working flow. The machines used today share one LAN; this run does
  not close Internet, native desktop recipient, or independent-user acceptance.
- Preserved pre-existing changes in this file, `TASKS.md`, and
  `desktop/src-tauri/src/main.rs`, plus untracked `scripts/`. Local source HEAD
  was `b73d961`. Deepin's original checkout was clean at `9f62701` and was not
  edited. An archive of current tracked working-tree files was built in an
  isolated Deepin source mirror; no signing credential or real model was copied.
- Windows: `cargo build -p ts-cli --locked --offline` passed. Deepin: Cargo
  1.98.1, native kernel 6.6.155-amd64-desktop-hwe, GLIBC 2.38;
  `CARGO_TARGET_DIR=<test-root>/target ~/.cargo/bin/cargo build -p ts-cli
  --locked --offline` passed in 2m23s. The Linux CLI used only libgcc_s, libm,
  and libc and ran successfully on Mint (GLIBC 2.39); no Rust installation or
  system package change was needed on Mint. Ubuntu WSL2 was running but was
  not used as a peer in this run.
- A scratch Rust helper generated a valid synthetic U8 Safetensors file,
  8,388,686 bytes including its header, with one tensor/eight 1 MiB chunks.
  It built a manifest, populated the Windows sender CAS through
  `ObjectStore::put_verified`, and signed metadata with public deterministic
  TEST key `[41; 32]` (not a device/publisher credential). Release ID:
  `f465d36dae4023433f5b2514aea8ab9cd534d2872e36ce7b6e052696b8f3e24a`.
  Only `model.tswarm`, `release.tsrelease`, and `share-uri.txt` went to recipients
  before download; source payload and sender CAS were not copied to them.
- Live commands, with host addresses and peer IDs supplied from node readback:
  `ts-cli node model.tswarm sender-store` on Windows;
  `ts-cli verify-release release.tsrelease model.tswarm <exact-share-uri>` and
  `ts-cli receive-release release.tsrelease model.tswarm inbox <exact-share-uri>`
  on each Linux recipient;
  `ts-cli fetch-received inbox/<release-id> <windows-peer> <windows-tcp-address>
  recipient-store downloaded.safetensors` on Deepin. These passed and Deepin
  had eight hash-named CAS objects. Then `ts-cli node model.tswarm recipient-store`
  on Deepin exposed that downloaded inventory. Stopped only the identified
  Windows test PID and confirmed its listener was gone before Mint ran
  `ts-cli fetch-received inbox/<release-id> <deepin-peer> <deepin-tcp-address>
  recipient-store downloaded.safetensors`. Mint passed with eight verified objects.
- Read back both Linux outputs to Windows and compared every byte using
  `[System.Linq.Enumerable]::SequenceEqual[byte]`; both equal the source, length
  8,388,686. All source/download/offline/resumed SHA-256 values:
  `1788d92a73e01541536871fb48db6b88bfa3aaaea2a3c0cb851b3b10d71b2d72`.
- Mint adversarial live checks: wrong share identifier and one-byte modified
  signed bundle each exited 1 and created no receipt; existing output exited 1
  and retained its sentinel bytes; a corrupt cached chunk exited 1 with an
  object-hash mismatch and created no prepared output. Terminated only a
  test-owned downloader after one verified object; no prepared file existed.
  Rerunning `fetch-received` against its partial cache completed all eight
  verified objects and produced byte-identical output. No sibling materialization
  staging directories remained. A process killed during CAS insertion can leave
  a `.tmp-*` staging file; it is not a hash-addressed verified object. The first
  test helper incorrectly counted this as an object and failed its count check;
  corrected the helper to count 64-character hash names and reran in fresh
  scratch paths successfully. No product repair was needed for that assertion.
- Stopped the exact Deepin test PID after checking its working directory and
  confirmed its TCP listener disappeared. With both serving nodes stopped,
  both recipients ran `fetch-received` with their full caches and unavailable
  former seed addresses, prepared `offline.safetensors`, and passed `cmp` and
  SHA-256 checks. No Windows `ts-cli` process remained.
- Discovered a separate CLI limitation: `ts-cli verify downloaded.safetensors
  model.tswarm` exits 1 (`manifest root mismatch`) for this valid 1 MiB-chunk
  manifest because the command rebuilds using a hardcoded 16 MiB chunk size.
  The P2P/CAS checks and independent complete-file byte comparisons passed;
  do not treat this failed extra CLI command as a passed verification. Fix and
  regression coverage remain open.
- Evidence layers: unit/integration suites were not rerun (no implementation
  edits); e2e/live are real Windows -> Deepin -> Mint CLI processes, signed
  metadata binding, CAS verification, full-file readback, rejection, interruption,
  re-seeding, offline reuse, and listener cleanup. No GUI, package install,
  credential-store, WAN/NAT, public discovery, or independent operator acceptance
  is claimed. No commit or push. Task-owned scratch/logs remain in ignored
  `target/lan-20261003` locally and `.cache/tensorswarm-lan-20261003` on each Linux
  host; no test serving process remains. Machine addresses stay out of tracked docs.

## 2026-10-03 — Desktop private download implementation and native acceptance

- Added explicit-peer recipient download primitives in `ts-daemon::download`;
  Tauri selects a native output path and manages one cancellable job while Rust
  engine crates own networking, verification, and storage. Signed inbox receipt,
  immutable ID, single-file recipe, peer/address, cache hash/length, and library
  compatibility are rechecked before publication. Progress counts verified
  durable chunks. A separate per-manifest cache preserves partial progress.
- Complete-file preparation stages beside the chosen output, parses the actual
  model container, compares tensor metadata and canonical tensor hashes, then
  commits with a no-replace hard link. This accepts nondefault chunk layouts
  without rebuilding a root with default chunks. The exact received manifest
  and verified bytes enter the library through the existing import lock and
  runtime restart. Existing control authentication/origin boundaries remain.
- Exposed current local listener addresses through authenticated `/v1/peers` and
  the desktop Peers & sources screen. Added recipient selection, peer/address
  inputs, progress, failure/cancel/retry states. Polling updates controls without
  recreating receipt rows. Native review found CSP-blocked inline styling in
  receipt/progress presentation; moved the relevant styles into CSS. Manifest
  roots now render/copy as hex rather than comma-separated byte arrays.
- Unit/integration: `cargo fmt --all -- --check`, `cargo check --workspace
  --offline`, `cargo test --workspace --offline`, and `git diff --check` passed.
  Daemon suite: 11 tests plus 5 independent-process tests; desktop: 24 tests.
  New real libp2p integration covers missing-chunk fetch with a preverified
  partial cache, complete-file byte comparison, recipient re-seeding after
  original shutdown, offline cache reuse, corrupt cache refusal, and cancellation
  after one commit. Adversarial tests cover malformed peers/layout, existing
  output preservation, false tensor identity, tampered receipt, conflicting
  library paths, and exclusive job start. A test fixture initially assigned a
  raw array to `Hash32`; corrected the test type and recipe references, then
  reran the focused suite successfully. No product behavior was weakened.
- Build: `cargo tauri build --debug --no-bundle --config <scratch-config>` passed
  twice after final implementation and presentation edits. Extracted inline
  JavaScript passed `node --check`. The test app used identifier
  `org.tensorswarm.downloadtest20261003`; no default profile or signing
  credential was modified. No package install, key creation, commit, or push.
- Live native Windows: current debug app started with empty library and online
  engine. Native pickers saved the deterministic TEST-key signed release and
  matching 1 MiB-chunk manifest, showing 1/1 metadata match. Invalid peer was
  rejected before output selection. Save-picker UI automation's `set_value`
  changed the display without updating the dialog's selected output; the app
  refused the resulting existing destination with zero chunks fetched and kept
  the source intact. Actual click/select/type input resolved the picker state.
  The earlier Computer Use run was stopped by physical Escape; owner explicitly
  authorized retry, and the updated helper completed this run.
- Deepin SSH was unavailable. Mint's test CLI seed started from its previously
  verified recipient CAS in isolated `.cache/tensorswarm-lan-20261003`. Its
  wildcard TCP listener existed, but direct Windows TCP connection timed out;
  the native job exhausted retries, published no output or library entry, and
  displayed the error. A temporary loopback SSH forward to that same listener
  allowed native retry. This is tunnel-assisted transport evidence, not direct
  incoming LAN or Internet acceptance. No firewall/router settings changed.
- The native job completed 8/8 verified chunks, prepared
  `target/lan-20261003/native-received.safetensors`, activated one library model,
  and read back online engine plus 100%/Verified availability. Readback found
  eight download-cache objects and the exact received library manifest. Full
  `[System.Linq.Enumerable]::SequenceEqual[byte]` against the source passed;
  both lengths are 8,388,686 and SHA-256 is
  `1788d92a73e01541536871fb48db6b88bfa3aaaea2a3c0cb851b3b10d71b2d72`.
- Stopped the identified Mint seed after checking `/proc/<pid>/cwd`; its TCP
  listener disappeared. Then a fresh Mint `desktop-third-store` used
  `ts-cli fetch-received` with the Windows desktop's UI-read peer ID and direct
  LAN TCP address. It fetched all eight chunks and prepared
  `desktop-third.safetensors`; remote `cmp`, SHA-256, and local full-byte readback
  in `target/lan-20261003/native-third.safetensors` all matched. This proves the
  desktop recipient serves the downloaded model after the original seed stops.
  The third peer is a separate CLI process/cache on the original Linux host,
  not a third native desktop or independent operator.
- Stopped the isolated Windows test process, rebuilt/copy-launched the updated
  app against the same profile, and read back persistent signed inbox, one model,
  eight verified chunks, 100% availability, online engine, and readable hex root.
  The temporary SSH tunnel was closed. Same-operator native Windows + Linux CLI
  e2e/live acceptance is established; native Linux desktop recipient, native
  desktop-to-desktop, independent-user/key-holder pilot, direct incoming network
  reachability, and separate-Internet/router/NAT acceptance remain open.
- Limitations: private download caches have no quota/automatic cleanup; libraries
  require one artifact format and reject filename conflicts; hard-link support
  is required at the output. The separate CLI `verify` nondefault-chunk bug remains
  open. Pre-existing dirty work and untracked `scripts/` were preserved.
- Final cleanup: stopped the rebuilt isolated Windows test process; no test
  control/proxy/tunnel listener remained. Deepin became reachable on the final
  cleanup check, so its earlier test seed was identified by exact command and
  working directory and stopped; its TCP listener disappeared. Task-owned
  synthetic files, profiles, and logs remain for review. No test seed is left
  running on either reachable Linux host.

## 2026-10-03 — Accountless Cloudflare TCP relay acceptance

- Owner proposed a temporary Quick Tunnel. The initial assumption that TCP required a Cloudflare account/domain was disproved by live acceptance: `cloudflared tunnel --no-autoupdate --url tcp://127.0.0.1:<seed-port>` returned a temporary hostname, and `cloudflared access tcp --hostname <temporary-hostname> --url 127.0.0.1:<bridge-port>` carried the actual libp2p connection. No account, domain, or application transport change was needed.
- Windows used the current local `target/debug/ts-cli.exe`; Mint used the previously built isolated Linux CLI. Windows downloaded the official standalone cloudflared 2026.9.3 into ignored `target/wan-20261003`; Mint already had cloudflared 2025.8.1. Its pre-existing unrelated HTTP tunnel was preserved.
- Started Windows `ts-cli node` over the synthetic eight-chunk fixture and verified sender CAS. Exposed only its peer TCP listener, not the daemon control API or model HTTP proxy. Mint ran the client bridge and `ts-cli fetch-received` with the exact signed inbox ID and sender peer ID into a previously absent `.cache/tensorswarm-wan-20261003/recipient-store`. Download/preparation succeeded; eight verified objects were read back. Remote `cmp` against the previously byte-verified fixture output passed.
- Stopped the original Windows node and confirmed its TCP listener was absent. Started a Mint node using only the new recipient cache, exposed it through a second TCP Quick Tunnel, and ran a Windows client bridge. A fresh Windows signed inbox and `third-store` fetched and prepared the model through the second tunnel. Full `[System.Linq.Enumerable]::SequenceEqual[byte]` against the original source returned True; eight objects were read back. Both complete outputs have SHA-256 `1788d92a73e01541536871fb48db6b88bfa3aaaea2a3c0cb851b3b10d71b2d72`.
- E2e/live: actual Windows -> Cloudflare -> Mint transfer, followed by Mint -> Cloudflare -> fresh Windows peer after original seed shutdown. All peer addresses supplied to downloaders were loopback client bridges; no direct LAN peer address was used. Machines remain on the same LAN: this proves external relay transport, not separate-network/router/NAT or independent-user acceptance. CLI peers were used; native desktop Cloudflare transfer remains open. No new adversarial cases or unit/integration suites were run because no source behavior changed.
- Closed Windows node/tunnels and SSH sessions. SSH closure left three test-owned remote processes; checked their exact command lines and stopped only those PIDs. Mint peer/bridge listeners disappeared; Windows original-seed/client-bridge listeners were absent. Preserved the unrelated Mint HTTP tunnel. Scratch outputs remain for review. No system service, firewall/router change, commit, or push.

## 2026-10-03 — Native Windows desktop through Cloudflare

- Built current source with `cargo tauri build --debug --no-bundle --config <ignored WAN desktop config>`; passed. The first config write used the wrong working-directory-relative path and failed before build; corrected it to the absolute scratch path. Used fresh identifier `org.tensorswarm.wantest20261003` and ignored `target/wan-20261003/ts-desktop-wan.exe`; default profile untouched.
- Computer Use native readback showed empty library and online engine. Native pickers received the deterministic TEST-key signed bundle and matching manifest with exact share identifier, retaining metadata separately. Competing input, stale accessibility geometry, and save-dialog focus required refreshed screenshots and actual filename typing; these were harness issues, not accepted transfer evidence.
- Mint seeded its verified synthetic cache through an accountless TCP Quick Tunnel. Windows cloudflared client exposed a loopback TCP bridge. Native Download from peer used the Mint peer ID and bridge address, selected a new `desktop-received.safetensors` output, and completed 8/8 verified chunks, 8 MiB, one library model, 100% Verified availability and online engine. Readback found eight download-cache objects. Full source/output SequenceEqual passed; SHA-256 `1788d92a73e01541536871fb48db6b88bfa3aaaea2a3c0cb851b3b10d71b2d72`.
- Stopped exact original Mint seed and tunnel. First third-peer attempt timed out: Cloudflare origin log reported refused local TCP connection. Closing the exec session that contained the download bridge also terminated the app launched as its child. Relaunched the app separately; a hidden shell launch could not provide native UI readback, so stopped only that identified test process and launched the existing test executable with Computer Use. Native Overview readback showed persisted one model, 100%, 8/8, online. Refreshed peer identity/listener details from native Peers screen rather than reusing pre-restart values.
- Exposed the restarted desktop peer TCP listener through a new Quick Tunnel. Mint client bridge plus `ts-cli fetch-received` into previously absent `desktop-third-retry-store` succeeded after checking original seed listener absent. Remote cmp/SHA-256 matched; scp readback to ignored local scratch and full source/third-output SequenceEqual passed. This closes the failed harness stage with independent live evidence of desktop re-seeding after original shutdown.
- Evidence: build passed; no source behavior changed, so unit/integration suites were not rerun. E2e/live establishes Mint CLI -> Cloudflare -> native Windows desktop -> Cloudflare -> independent fresh Mint CLI peer, full-byte equality and native restart persistence. The third peer is a distinct process/cache, not an independent operator or Linux desktop. Both hosts remain on one LAN: separate-network/NAT and native desktop-to-desktop acceptance remain open. Cloudflare is an external harness, not an integrated app feature.
- Closed task tunnels, identified and stopped remote test bridges left after SSH closure, and stopped only the isolated desktop process. Preserved Mint's unrelated pre-existing HTTP tunnel and retained scratch files/profile for review. No credentials, system service, firewall/router change, commit, or push. Known CLI verify default-chunk bug was inspected but remains unfixed.

## 2026-10-03 — CLI layout verification and beta preparation

- Fixed CLI `verify` false rejection for nondefault chunk sizes. Added read-only `ts-format::verify_model`: root/schema check, parsed container metadata and canonical tensor identities, contiguous exact chunk coverage with checked arithmetic, streaming hash comparison against every supplied range, and comparison with the builder-generated single-file recipe. Local filename may differ. CLI manifest reads now use the existing 64 MiB limit. Alternative recipe encodings remain outside this verifier's scope.
- Independent CLI-process regression accepts a renamed output with 16-byte chunks and rejects invalid root, gaps, oversized ranges, incomplete coverage, wrong chunk hash, changed literal recipe bytes and changed source payload. Representative Safetensors/GGUF fixture test now also invokes the verifier. Unit/integration: formatter, workspace check/test and build passed; 94 workspace tests. Focused format tests passed after the final test addition. No desktop UI or networking behavior changed in this slice.
- Live/readback: rebuilt current `ts-cli` and verified retained `target/wan-20261003/desktop-received.safetensors` and `desktop-third-retry.safetensors` against the original eight-chunk/1 MiB `model.tswarm`; both commands exited 0. This closes the original live false-rejection case.
- Owner selected MIT to match existing LICENSE; changed workspace Cargo metadata from dual-license declaration to MIT. `cargo metadata --offline --no-deps --format-version 1` read back MIT for all eight packages. Added model-artifact and .env ignore rules; `git check-ignore` confirmed model, environment and existing build-output exclusions. Tracked filename inspection found no model artifacts, .env, executable/DEB or private key files. A common private-key/GitHub-token/AWS-access-ID pattern scan excluding build/Git/untracked scripts returned no matches; this is a limited pattern check, not a full security/history/dependency audit.
- Added `docs/private-beta.md` with native recipient steps, verified Cloudflare TCP harness commands, re-sharing acceptance, cache/output/format limitations and release gates. README links it. Corrected `docs/demo.md`: manifest generation does not populate a verified seed store; desktop import or an already populated operator store is required.
- Current Windows NSIS packaging was started; final result follows below. Separate Internet connections, current-package clean-install/download acceptance, native Linux desktop and independent-user gates remain open. No commit, staging, push or GitHub visibility change. All pre-existing dirty work and untracked scripts preserved.
- Packaging result: `cargo tauri build --bundles nsis` passed; optimized compile took 3m14s and NSIS completed. Artifact `target/release/bundle/nsis/OpenSwarmLayer_0.1.1_x64-setup.exe`, 5,677,788 bytes, SHA-256 `d395fa0b0a53d5a7b5b3f5e05e457d6cca4d788828663d5f74ca57fee43524f4`. Authenticode readback is NotSigned. No install or runtime acceptance is claimed for this newly built package; prior package evidence predates the download changes. No artifact was uploaded or published.

## 2026-10-04 — Isolated current-source Windows package acceptance

- Built on October 3 with `cargo tauri build --bundles nsis --config <ignored acceptance config>`; passed. Configuration changes only the product name, application identifier and window title/geometry. Package `TensorSwarm Beta Acceptance 20261003_0.1.1_x64-setup.exe` SHA-256 is `80b4fa028542b5eaefe55c43fbd3d19486ca1ff1d9533fc2a5c1f2a16b20c956`; installed executable SHA-256 is `1cde23316b53e5917b4ab7001c966245223333d8c97124b466bbec6993b17b45`. The production installer above remains intact; the release executable now carries the acceptance identifier and must not be distributed as the production binary.
- Silent NSIS install into ignored `target/package-acceptance-20261003/install` exited 0. Distinct uninstall registration and fresh `org.tensorswarm.packageacceptance20261003` profile were verified. Native first launch showed zero library entries/chunks and online engine. Checked synthetic metadata was staged with `ts-cli receive-release`; native Refresh inbox listed it without adding model bytes. This package run does not establish native metadata-picker acceptance, which has earlier debug evidence.
- Native Download from peer through an accountless Cloudflare TCP bridge completed eight verified 1 MiB chunks, prepared the 8,388,686-byte synthetic Safetensors file and activated one complete library model. Source/output full-byte SequenceEqual passed, and the rebuilt CLI verified the output against the nondefault-chunk manifest. No default installation or profile was modified.
- On October 4 the installed executable was relaunched independently. Native readback showed persisted one model, eight chunks, 100% availability and online engine. Original Mint test seed was absent. Exposed the current desktop TCP listener with `cloudflared tunnel --no-autoupdate --url tcp://127.0.0.1:<port>`; Mint ran `cloudflared access tcp` and `ts-cli fetch-received` into a previously absent `package-third-store`. Fetch/preparation exited 0; readback found eight objects, remote cmp passed, and scp followed by local full-byte SequenceEqual returned True. Output SHA-256 is `1788d92a73e01541536871fb48db6b88bfa3aaaea2a3c0cb851b3b10d71b2d72`.
- E2e/live: installed Windows recipient downloaded through an external relay, retained verified state across restart, and re-seeded to a fresh Linux CLI cache after original seed shutdown. Hosts remain on one LAN and one operator controls both. This is not separate-network, independent-user, Linux desktop or desktop-to-desktop acceptance. Owner deferred separate-internet-connection testing to a later friend pilot. Default production-profile install/upgrade and current-package tray behavior remain untested.
- Cleanup: stopped the exact test bridge and desktop process, preserved Mint's unrelated pre-existing HTTP tunnel, then ran the isolated uninstaller `/S`; exit 0. Test binary/registration and test listeners were absent; default installation binary/registration remained present. Test profile, fixture outputs and logs remain for review. No recursive scratch deletion or system configuration change.
- Unit/integration: no runtime source changes in this package slice; prior formatter, check, 94 workspace tests and build remain the source evidence. Documentation reconciled to these live results. No staging, commit, push, upload or GitHub visibility change; pre-existing dirty work and untracked scripts preserved.

## 2026-10-04 — Plain-language beta introduction

- Owner requested a more human repository description and README with clear usage instructions and explicit technical-test status. Rewrote README around purpose, developer/tester setup, signing-key creation, sender export, recipient download, re-sharing verification and current limits. Retained links to detailed evidence, architecture, roadmap and CLI/proxy documentation rather than reproducing the development history at the entry point.
- Updated the GitHub description with `gh repo edit SPhillips1337/OpenSwarmLayer --description ...`; readback matched: "Share LLM model files between computers, then share them onward. A technical test beta built with Rust and Tauri; not ready for public release." Readback confirms the repository remains private.
- Checked README steps against current UI labels and build configuration; all relative file links resolve and `git diff --check` passed. Documentation only; no runtime tests rerun. README changes remain local alongside the existing uncommitted implementation. No staging, commit, source push, installer upload or visibility change.

## 2026-10-04 — Versioned beta install scripts

- Owner requested tagged file releases and curl/PowerShell-style installation. Added root `install.ps1` (Windows x64, PowerShell 5.1+) and `install.sh` (Debian/Ubuntu-family Linux amd64). Defaults pin proposed `v0.1.1-beta.1`; alternate beta tags are validated. Scripts download exact named packages plus SHA256SUMS, require a unique matching checksum, and invoke the native installer only after verification. Download-only modes retain verified packages. Failed/unverified downloads are removed. No automatic latest selection, credential handling, app launch, service or firewall configuration.
- Windows uses the normal NSIS UI and reports unsigned status; Linux uses apt/sudo and declares incomplete native Linux acceptance. No Linux package is newly built or accepted here. Checksums come from the same release, so are corruption/mismatch checks rather than independent signatures. Added line-ending attributes for shell/PowerShell files, README links and docs/releasing.md with asset naming, annotated beta tags, draft pre-release workflow, checksums, exact proposed one-liners and explicit private/unpublished limitations. Architecture/task status updated.
- Integration/adversarial fixture tests: Windows PowerShell 5.1 `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/test-installers.ps1` passed nine cases; WSL Ubuntu `bash scripts/test-installers.sh` passed nine cases. Both cover success, corrupt/missing/duplicate checksums, unavailable release, installer failure, download-only, unsupported architecture and invalid version. Download/process/package-manager calls were mocked; no package installed. Cleanup checked. Bash syntax and git diff whitespace checks passed. Initial PowerShell mock scope failure was repaired in the test harness; WSL sandbox access required approved escalation. No Rust runtime changes or repeated Rust suite.
- Live release readback: `gh release list` and REST release count show zero releases; repository remains private. Hosted script downloads and installer e2e are therefore untested. Current implementation is uncommitted; tagging old HEAD would omit the tested sharing flow. Tag creation/package publication remains pending reviewed source commit and release preparation. No staging, commit, tag, push, upload or visibility change. Pre-existing files in untracked scripts were preserved.

## 2026-10-04 — Authorized beta 1 release preparation

- Owner explicitly requested "tag and publish it". Reviewed runtime/source diff and new download orchestration; retained all earlier evidence boundaries. Refreshed origin/master with no upstream divergence. Formatter, workspace check and all 94 tests passed; Windows PowerShell 5.1 and WSL Bash installer fixture tests passed all 18 cases. Limited private-key/GitHub-token/AWS-access-ID pattern scan returned no matches; this does not establish a full history or dependency security audit.
- Rebuilt using the default Tauri configuration with `cargo tauri build --bundles nsis`; passed in 1m02s plus packaging. This restores the production identifier after isolated acceptance builds. Staged `OpenSwarmLayer-v0.1.1-beta.1-windows-x64-setup.exe` (5,678,246 bytes), install.ps1, install.sh and SHA256SUMS under ignored target/releases/v0.1.1-beta.1. Windows installer SHA-256 `880bf10491259d2568f43b203bd6611524d06239922cf7daab41e693c0f94e3d`; Authenticode remains NotSigned. Earlier production package checksum is historical; this rebuild replaced that bundle output.
- Release scope is Windows-only, marked technical test pre-release and not ready for public release. No Linux DEB included. Live transfer/install evidence remains the recorded same-source isolated-name/identifier Windows run; default-profile install/upgrade and a newly hosted installer run are not claimed. README and release guide explain authenticated manual downloads, private visibility and unusable anonymous one-liners. Both scripts are included for future accessible releases; Linux cannot install beta 1 because no DEB exists.
- The intended commit includes reviewed sharing implementation, verifier repair, MIT metadata and beta documentation/install scripts. Only explicit reviewed paths will be staged; pre-existing untracked Deepin scripts remain untouched. Publication result and remote asset readback follow separately. Repository visibility will remain private.

### Beta 1 publication result

- Committed 27 explicit reviewed paths as `dec8e5099e782c9ddd6be75e8fecb3d65907989f` (`feat: prepare verified sharing technical beta 1`). Created annotated `v0.1.1-beta.1` and pushed master/tag atomically. Remote tag dereferences to that exact source commit. Untracked pre-existing Deepin scripts remain unstaged and unchanged. Initial sandbox index-write denial was resolved with authorized escalation; no automatic-review rejection occurred.
- Created a draft GitHub pre-release with the Windows installer, install.ps1, install.sh and SHA256SUMS. Authenticated `gh release download` read back all four uploaded files; each SHA-256 exactly matched local staging. Then published with `gh release edit ... --draft=false --prerelease --latest=false`.
- Live GitHub readback confirms published at `2026-10-04T10:39:07Z`, `isDraft=false`, `isPrerelease=true`, tag `v0.1.1-beta.1`, four uploaded assets and unchanged private repository visibility. Release: https://github.com/SPhillips1337/OpenSwarmLayer/releases/tag/v0.1.1-beta.1 . Notes identify the exact source commit, unsigned Windows status, isolated-package evidence boundaries, absent Linux DEB and private-authentication limitations. No tag moved, no credentials/models uploaded and no public visibility change.
- This completes authorized tag/release publication and hosted asset integrity readback. It does not establish a new hosted installer run, anonymous script installation, native Linux or separate-network acceptance. Publication evidence is recorded in a subsequent documentation commit; the release tag remains fixed on the tested implementation snapshot.

## 2026-10-04 — Linux beta packages from the fixed tag

- Owner requested DEB and AppImage additions. Deepin SSH timed out; Mint has no Rust/WebKit development dependencies. Used existing Ubuntu 24.04.5 WSL tooling (glibc 2.39, Rust 1.98.1, Tauri CLI 2.11.4, WebKitGTK 2.52.6) without installing system dependencies or changing existing checkouts. Exported exact annotated v0.1.1-beta.1 source (`dec8e5099e782c9ddd6be75e8fecb3d65907989f`) with git archive into a fresh user-cache directory.
- Build: `APPIMAGE_EXTRACT_AND_RUN=1 CARGO_NET_OFFLINE=true cargo tauri build --bundles deb,appimage` passed; optimized compilation 2m01s, both bundles completed. Initial Windows/WSL quoting, script-final-line CRLF and non-login cargo PATH problems were corrected in the ignored harness. No product source/config was altered. Existing packaging-tool cache was used.
- DEB: open-swarm-layer 0.1.1 amd64; declared dependencies libayatana-appindicator3-1, libwebkit2gtk-4.1-0 and libgtk-3-0. Desktop entry/icons/binary extracted; ldd found no unresolved libraries. readelf confirms binary requirement GLIBC_2.39. Generic generated package Description is `(none)`; release/README supply the human description. Built on a recent base, so no older-Deepin or broad-distribution compatibility claim.
- Integration/live process: extracted DEB executable and actual AppImage each launched under Xvfb with separate HOME/XDG directories, returned `ok` from /healthz, opened loopback control/proxy listeners and released both after targeted test-process termination. AppImage structure/extraction and AppRun/desktop entry checked. Initial smoke used the wrong /health endpoint, then checked shutdown too early; corrected exact /healthz response and allowed shutdown to settle, reran both successfully. Headless graphics warnings are retained in scratch logs. No visible desktop controls, tray action or OS signing-store acceptance claimed.
- Native Mint: copied only AppImage and test harness into a fresh cache directory. Actual AppImage under Xvfb passed /healthz/listener checks and targeted shutdown first with APPIMAGE_EXTRACT_AND_RUN=1, then with normal FUSE mounting in another fresh profile. File checksum matched the local staging artifact. No native DEB install/removal, full native UI or sharing test in this slice. Neither host's default profile was modified; scratch profiles/logs remain.
- Staged `OpenSwarmLayer-v0.1.1-beta.1-linux-amd64.deb` (11,145,498 bytes), SHA-256 `1cc92bd27bd285581633cf5a60be31aa692bcf9e10a24b2848662e7c2fa999c9`; `OpenSwarmLayer-v0.1.1-beta.1-linux-x86_64.AppImage` (85,273,080 bytes), SHA-256 `a6be8295b1b11f17d8f0f750a45eebc1e384ba26623cb92727bb72650cf62ac6`. README/release workflow now document manual install/run and glibc 2.39 floor. Linux install.sh remains unchanged and installs the exact DEB name; anonymous scripts still cannot access this private repository. Prior 94 Windows Rust and 18 script tests remain source evidence; no new Linux full Rust suite run.
- Adding these same-tag platform assets does not move the tag or replace any published package/script. SHA256SUMS will be extended to cover both additions, and release notes updated with compatibility/evidence boundaries. Hosted readback result follows below.

### Linux publication and hosted script readback

- Uploaded both Linux artifacts to existing v0.1.1-beta.1 and extended SHA256SUMS. Updated release notes without changing pre-release state or tag. Authenticated download of all six assets matched local staging hashes, including unchanged Windows installer and scripts.
- GitHub metadata now reports `isPrivate=false`, observed after Linux uploads. No visibility-changing command was issued by this agent; documentation/notes were reconciled to the observed public state. Public access still does not establish production readiness. The fixed tag's README is historical; master/release notes carry the Linux additions and current access instructions.
- Actual published-script download-only testing exposed Windows CRLF in our generated SHA256SUMS: the Linux parser refused the entry before installation. Regenerated the checksum asset as ASCII with Unix LF and corrected the release-guide PowerShell generator. GitHub initially served the old 505-byte copy while a cache-busted URL returned the corrected 500-byte copy. After propagation, unmodified Linux install.sh `--download-only` downloaded the DEB from its ordinary published URL and verified its SHA-256; published install.ps1 `-DownloadOnly` downloaded/verified the Windows installer against the same corrected manifest. Neither path installed a package. This closes the failed hosted stage without modifying package/script assets or moving the tag.
- Final release remains published pre-release; six assets include both Linux packages. Native Mint and WSL smoke-test processes are absent, and Mint's unrelated pre-existing HTTP tunnel remains running. Scratch artifacts/profiles and verified download-only files are retained. Full native Linux installed/UI/tray/sharing acceptance remains open. Only documentation changes are committed/pushed for this addition; pre-existing untracked Deepin scripts remain untouched.

## 2026-10-04 — npm/npx native installer wrapper

- Owner requested npm/npx distribution. Added npm/ package openswarmlayer@0.1.1-beta.1 with a dependency-free executable, explicit install/download/help actions and allowlisted six-file publish payload. Node.js >=20 required. No install/postinstall lifecycle hooks; native model networking/storage remain in Rust/Tauri. The wrapper pins v0.1.1-beta.1 assets, exact sizes and SHA-256 values in release.json, bounds streamed downloads, allows only known HTTPS release hosts/redirects, and removes failed partial files. Windows invokes the regular unsigned installer; Linux installation checks glibc >=2.39 and uses apt/sudo. AppImage is download-only.
- Unit/integration: `npm test --prefix npm` passed 11 groups covering invalid inputs/platforms/glibc, redirect boundaries/loops, streamed byte equality, corrupt/short/oversized/unavailable/interrupted downloads, unsafe metadata, help/download-only no-execution and installer success/failure cleanup. Initial sandbox Node test-worker spawn denial resolved by authorized escalation. Node syntax checks and git diff whitespace checks pass. `npm publish ./npm --dry-run --tag beta --access public` passed prepublish tests and reviewed packlist. Only README, LICENSE, package.json, release.json, bin and lib are packed; no models, tests, machine config or credentials.
- E2e/live: actual packed npm exec entry point on Windows Node 24.19.0 printed help and fetched/verified the published Windows installer without installing it. Ubuntu WSL Node 24.20.0/npm 12.0.2 ran the same packed entry point and fetched/verified the actual 85 MB AppImage. Installer processes were mocked in tests; no new OS installation or app launch claimed. No Rust source changes or repeated Rust suites.
- Registry readback: npm view openswarmlayer returned E404 (name appears unclaimed); npm whoami returned E401 repeatedly. Asked owner to sign in with npm login without sharing credentials. No account/token configuration changed or secret read. Registry publication remains pending authentication; no short registry npx command is claimed live. Prepared a fixed-version GitHub npm tarball alternative and documented that distinction. Publication/readback result follows below. Native release tag remains unchanged; separate npm tag will identify wrapper source.

### npm wrapper publication and final readback

- Initial wrapper source committed/pushed as 9cd137e with separate npm-v0.1.1-beta.1 tag. Actual registry publish ran prepublish tests but failed E404 on PUT; npm whoami remained E401. No npm registry release is claimed. GitHub then reported the repository had moved to HappyMonkeyAI/OpenSwarmLayer; no transfer command was issued by this agent. Updated origin, current documentation and wrapper URL metadata to the canonical address. Wrapper version bumped to 0.1.1-beta.2, committed/pushed as 3303822 with annotated npm-v0.1.1-beta.2; native tag/assets remain unchanged.
- Re-ran all 11 npm test groups and packed the six-file beta 2 payload. Published openswarmlayer-0.1.1-beta.2.tgz on the existing GitHub pre-release and extended SHA256SUMS using ASCII/LF. Hosted download readback matched SHA-256 255db4530878e450ddf8fba565e071d8607c3410ce6ef38afcd6c2bc18e168bd. Release instructions distinguish GitHub URL-based npx from pending registry publication.
- Live/e2e: npx --yes --package=<published tarball URL> openswarmlayer download passed on Windows, downloading and verifying the actual native installer. WSL npm 12.0.2 refused remote packages by default (EALLOWREMOTE); documented and tested per-command --allow-remote=root. With that explicit opt-in the published npx entry point downloaded and verified the actual Linux DEB. No persistent npm configuration changed; no native installer executed in these readbacks. Earlier AppImage download and mocked installer cleanup tests remain recorded above.
- Final npm whoami still returned E401. Owner login is required for short npx openswarmlayer@beta installation. GitHub alternative is published and verified on Windows/Linux. No credentials, model files or build output committed; pre-existing Deepin scripts remain untouched. Native UI/install/tray/sharing acceptance gaps are unchanged.
