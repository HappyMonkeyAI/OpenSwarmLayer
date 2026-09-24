# TensorSwarm progress handoff

Last updated: 2026-09-24
Branch: master
Baseline HEAD at slice start: `55938e0` (`master` was eight commits ahead of `origin/master`)
This handoff records M6-07 implementation and M6-08 Windows/WSLg acceptance; pre-existing untracked `scripts/` preserved, no push performed

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

Continue M6-08 with Windows installed-package close/background acceptance and
native Linux package/runtime acceptance using a controlled fixture. Keep M6-06's
Windows tray Quit and native Linux tray/package gates explicitly open until
directly exercised. Headless daemon use remains optional; transparent proxy
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
- M6-07 is accepted for the Safetensors slice. Live GGUF UI coverage and
  packaged/cross-platform sharing acceptance remain under M6-08. The catalog
  schema permits one artifact format per library. Windows tray Quit and native
  Linux package/tray acceptance remain open under M6-06/M6-08.
- No commit or push was made. The pre-existing untracked `scripts/` directory
  remains untouched.
