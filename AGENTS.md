# TensorSwarm Agent Protocol

## Prime directive

Make small, evidence-backed changes without damaging existing work. Preserve the Rust/Tauri architecture, keep unverified bytes out of durable storage, and report gaps plainly.

## Session grounding

Before non-trivial work, read `README.md`, `CONTEXT.md`, `MCP.md`, `SPEC.md`, `ARCHITECTURE.md`, `PLAN.md`, `ROADMAP.md`, `TASKS.md`, and `PROGRESS.md`. Check `git status --short --branch` first; never reset, clean, stash, or overwrite pre-existing dirty work. Use `SPEC.md` for requirements, `ROADMAP.md` for next-slice ordering, `TASKS.md` for executable status/acceptance, and `PROGRESS.md` for dated evidence; keep these roles distinct and reconcile contradictions against the latest verified evidence.

## Work modes

- Solo, tightly scoped work may use the shared checkout when files do not overlap.
- Parallel or delegated work uses `.worktrees/<task-id>` with branch `ag/<task-id>` (or `agent/<task-id>` for interoperability).
- Every handoff names the worktree, branch, changed paths, exact commands/results, skipped checks, and commit/push status.
- Stage only owned paths. Do not use `git add .` on this repository's dirty working tree.

## Verification Ladder

For behavior-changing work, apply the applicable stages:

- V0 deterministic: format, check, focused tests, and build.
- V1 contract/ripple: inspect affected crate/API/UI boundaries and documentation parity.
- V2 owner adversary: test invalid input, auth failures, empty/error states, partial wiring, and acceptance criteria independently of the implementer's narrative.
- V3 live: when a runtime or UI changes, exercise the real process/window/API and read state back.
- V4 close loop: convert failures into fixes and rerun the failed stage before calling the slice done.

Worker self-report and green unit tests alone are not acceptance. Report evidence by layer: `unit`, `integration`, `e2e`, and `live`.

## Engineering rules

- Rust and Tokio remain the implementation baseline; Tauri is a shell over the daemon, not a second networking runtime.
- Validate all untrusted format, peer, HTTP, and control-plane input at boundaries.
- Keep local control authentication and origin boundaries intact.
- Update `CONTEXT.md`, `ARCHITECTURE.md`, `TASKS.md`, or an ADR when a design/workflow decision changes.
- Do not commit secrets, `.env` files, build output, model artifacts, or machine-local `MCP.local.md`.
- Do not claim Linux packaging, tray interaction, or native live-data readback without corresponding evidence.

## Adaptive escalation

Use the least expensive adequate reasoning/tooling path. Classify failures before retrying: diagnose environment/flakiness first; make one focused repair for local implementation failures; escalate structural contract or cross-module failures after two unsuccessful repairs. If the escalated pass fails, preserve evidence and re-plan rather than looping.
