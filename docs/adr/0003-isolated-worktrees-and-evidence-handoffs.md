# ADR-0003: Isolated Worktrees and Evidence Handoffs

## Status

Accepted.

## Decision

Parallel or delegated implementation uses one worktree per task under `.worktrees/<task-id>` and branch `ag/<task-id>` (with `agent/<task-id>` accepted for interoperability). The parent/owner accepts the actual worktree after reviewing an evidence-bearing handoff.

## Rules

Record the dirty baseline before work. Never reset, clean, stash, or overwrite pre-existing changes. Handoffs must include changed paths, exact verification commands and results, skipped checks, and commit/push status. Stage only owned paths.

## Consequences

Shared-checkout work remains appropriate for solo, non-overlapping slices; parallel work cannot silently clobber application or documentation changes.
