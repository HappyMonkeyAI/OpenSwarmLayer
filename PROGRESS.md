# TensorSwarm progress handoff

Last updated: 2026-09-27
Branch: master
Baseline HEAD at slice start: `3194bfc` (`master` was nine commits ahead of `origin/master`)
This handoff records local verification and a current-source native Deepin package/runtime smoke; existing modified files and untracked `scripts/` preserved, no commit or push performed

## Completed this session

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

Continue with packaged multi-model UI readback (M6-09), then live signed-release
export/import and private two-user exchange (M7-03/M7-06). The current-source
Deepin DEB now launches from an extracted package and passes live health/auth
readback, but installation/removal, visible UI/import/prepare, and tray acceptance
remain open. Package installation is sudo-gated. Public directory/rendezvous work
remains blocked on the M7-02 legal and operational gates. Headless daemon use
remains optional; transparent proxy rewriting and broader P2 features stay
deferred.

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
