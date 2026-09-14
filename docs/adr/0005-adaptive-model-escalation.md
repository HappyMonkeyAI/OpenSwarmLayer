# ADR-0005: Adaptive Model Escalation

## Status

Accepted.

## Decision

Use the least expensive adequate reasoning path for routine work. Classify failures before retrying: diagnose environmental/flaky failures, make one focused repair for local implementation failures, and escalate structural contract or cross-module failures after two materially unsuccessful repairs. A failed escalation stops the loop and triggers re-planning.

## Guardrails

Escalation never replaces the Verification Ladder, worktree isolation, commit hygiene, or user approval for irreversible actions. Evidence passed to an escalated worker includes the task, acceptance criteria, changed paths, exact failure, dependency/ripple map, and prior attempts.
