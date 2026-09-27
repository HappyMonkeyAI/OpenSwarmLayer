# Agents Protocol Bootstrap

This repository adopts the portable Agents Protocol spine while keeping TensorSwarm as the application authority.

## Grounding order

Read `README.md`, `CONTEXT.md`, `AGENTS.md`, `MCP.md`, `SPEC.md`, `ARCHITECTURE.md`, `DESIGN.md`, `PLAN.md`, `ROADMAP.md`, `TASKS.md`, and `PROGRESS.md`. Inspect `git status --short --branch` before edits. Use the roadmap for next-slice order, the task board for acceptance/status, and progress for verified evidence; reconcile contradictions rather than duplicating or assuming completion.

## MCP discovery

`MCP.md` describes intent routing; it is not proof that a server is mounted. Before claiming a tool is unavailable, query the live Hermes/Dynamic MCP catalogue when it is exposed in the session. Record machine-specific results only in the ignored `MCP.local.md` overlay. Never copy credentials or tokens into this repository.

## Context mining

Use current source, tests, task acceptance criteria, and recent handoffs as ground truth. Record durable architectural decisions in `docs/adr/`; record reusable procedural lessons in `.agent/memories/patterns_and_lessons.md`.

## Acceptance

For implementation slices, run the relevant Verification Ladder stages from `AGENTS.md`. The parent/owner verifies the actual changed tree; a task-board status or worker narrative is not acceptance. Commit only coherent, independently verified slices, and do not push unless explicitly authorized.
