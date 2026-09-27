# TensorSwarm Development Roadmap

This file is the short, ordered guide for the next product-development slices. It does not replace the existing contract, architecture, plan, task board, or evidence log:

- `SPEC.md` — normative product/protocol requirements and trust boundaries.
- `ARCHITECTURE.md` and `DESIGN.md` — current implementation structure and design decisions.
- `PLAN.md` — milestone history and broad delivery strategy.
- `TASKS.md` — executable milestone IDs, owners, status, and acceptance criteria; update task state here.
- `PROGRESS.md` — dated implementation and acceptance evidence; record what was actually run and what remains open.
- `AGENTS.md` — mandatory workflow and repository safety rules.

When status differs, use the most recent independently verified evidence in `PROGRESS.md`, reconcile the relevant row in `TASKS.md`, and do not infer acceptance from a build or unit test alone.

## Product goal

Finish a trustworthy local-to-private model-sharing workflow and desktop acceptance before considering any public discovery service. Keep TensorSwarm Rust/Tokio networking and storage authoritative; Tauri remains a desktop shell. Do not store unverified bytes or present a self-asserted publisher signature as proof of identity, safety, or redistribution rights.

## Ordered work

### 1. Close M7-03 signed-release desktop acceptance — next slice

- Windows isolated-debug evidence now covers native export of the exact selected model's `.tswarm` manifest and signed `.tsrelease` bundle, descriptor preview, recipient receipt, and persisted inbox readback after app restart.
- Remaining: adversarially try a modified bundle, wrong manifest, wrong optional share identifier, and existing destinations. Verify visible rejection and no active-library mutation; exercise Linux Secret Service separately.
- Use the available vision-capable model only as an aid for inspecting UI screenshots; retain DOM/native state and filesystem readback as the evidence.
- Exit only after remaining adversarial UI cases and their persisted/API readback are demonstrated; codec/unit tests and browser fixtures alone do not close those error-state gates.

### 2. Complete M7-06 private, file-mediated two-user pilot

- Use a small synthetic or otherwise owner-approved artifact; do not use a model without a known source and redistribution basis.
- Exchange the release bundle and manifest out of band between independent users. Bind to the exact identifier when shared separately.
- On the recipient machine, verify and receive metadata, supply the sender's peer ID/address explicitly, fetch chunks, and compare the prepared output byte-for-byte and by manifest hashes.
- Read back recipient cache and inbox state. Verify cache reuse after sender shutdown, and rejection of bad ID, altered metadata, corrupt cached chunks, and an output created during transfer.
- Record sender/recipient OS, build, commands, and evidence. CLI independent-process acceptance is useful groundwork, not a two-user pilot. This step requires a reachable second user/host; record the blocker rather than simulating it if unavailable.

### 3. Close M6-09 multi-model UI acceptance

- Import at least two models of the currently supported same format; verify per-model root, tensor/chunk counts, and availability.
- Select each model and prepare it independently; compare bytes with each source. Exercise missing/corrupt chunks and invalid or omitted multi-file selection.
- Keep the existing mixed-format limitation explicit. Unit/router tests are not packaged UI acceptance.

### 4. Finish M6-08 native Linux package acceptance

- Build on a compatible native Linux host, install the package with owner-authorized privileges, launch the installed app, and read back the visible UI and daemon API.
- Verify model import, restart persistence, complete-file preparation, package removal, and tray/close behavior separately. WSLg and an extracted binary's health endpoint are not native install/UI/tray acceptance.
- Current evidence records a current-source Deepin DEB build and GLIBC/dependency preflight, but not installation. The host was reachable over SSH for that build, then SSH timed out after an isolated app launch; reconfirm reachability before native UI/package acceptance, and do not modify its existing checkout.

### 5. Resolve M7-02 public-service policy and operational gates

- Obtain qualified legal review and final decisions on rights/provenance requirements, moderation, reports/takedowns and appeals, operator coverage, privacy, and peer-address retention.
- Define enforcement and operational runbooks and verify fail-closed behavior before any public listing or announcement is enabled.
- Owner approval of a policy draft is not legal/operational signoff. Until the gate is explicitly accepted, do not implement or deploy public directory/rendezvous publishing.

### 6. Only after gates: scoped M7-04 pilot, then M7-05 evaluation

- If M7-02 and M7-03 are accepted and the owner explicitly authorizes it, design a limited allowlisted directory/rendezvous pilot with expiring, rate-limited provider leases; store metadata/peer hints, not model bytes.
- Test two independent users publishing, finding, fetching, delisting, and rejection of new announcements after delisting. Document that cached P2P bytes cannot be recalled universally.
- Treat federation, public DHT/relays, NAT traversal, and broad internet exposure as later decisions, not implied scope.

## Optional local-model-assisted QA

Operator-provided Kev/Jev decision endpoints and a vision-capable Gemma endpoint may help test application scenarios and inspect screenshots. They are test aids, not TensorSwarm dependencies or acceptance authorities.

- Discover exact model IDs and runtime metadata from the live `/v1/models` endpoint for each test session; do not assume aliases or the underlying run are independent.
- Keep decision-quality evaluation separate from inference latency, and do not treat one successful classification as reliability evidence. Preserve confidence/ambiguity and test repeatability where relevant.
- Use vision output to flag visual issues, then confirm behavior through deterministic UI state, API/filesystem readback, and human review as appropriate.
- Never infer that an inference endpoint exposes importable model-weight files. Before testing TensorSwarm distribution of a real model, establish the artifact path, format, hash, and permission to redistribute.
- Do not write machine-local endpoint addresses, credentials, model artifacts, or private runtime data into tracked project documents.

## Definition of done for each slice

1. Update the matching `TASKS.md` row only to the evidence actually achieved.
2. Run focused tests, relevant workspace checks/tests/build, formatter, and `git diff --check` for code changes.
3. For API/UI or packaging changes, verify the live target and read back the resulting state; name unit, integration, e2e, and live evidence separately.
4. Record command results, host/build details, exact remaining gaps, and whether commit/push/deployment occurred in `PROGRESS.md`.
5. Preserve pre-existing dirty and untracked work. Follow `AGENTS.md`; do not commit, push, or deploy without the applicable authorization.
