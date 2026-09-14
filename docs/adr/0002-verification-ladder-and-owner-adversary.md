# ADR-0002: Verification Ladder and Owner-as-Adversary

## Status

Accepted.

## Decision

Non-trivial behavior changes use a risk-scaled Verification Ladder: V0 deterministic checks, V1 contract/ripple review, V2 independent adversary tests, V3 live/runtime evidence when applicable, and V4 fix-loop closure. The owner verifies acceptance criteria independently; worker self-report is never proof of Done.

## TensorSwarm application

V2 targets malformed manifests, corrupt peer bytes, range/path failures, auth failures, empty/error UI states, and partial daemon wiring. V3 is required for changed daemon processes, Tauri windows, installers, or control APIs. Pure documentation changes require Markdown/diff checks but do not imply runtime acceptance.

## Consequences

Acceptance evidence is reported by layer (`unit`, `integration`, `e2e`, `live`), and known gaps remain visible in `TASKS.md` and `PROGRESS.md`.
