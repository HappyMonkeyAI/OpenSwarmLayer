# ADR-0004: MCP Intent Map and Bootstrap Discovery

## Status

Accepted.

## Decision

`MCP.md` is the small, portable intent map. `BOOTSTRAP.md` requires live discovery before declaring a server unavailable, and ignored `MCP.local.md` records machine-specific mounted/wanted state. Live catalogue results override assumptions in the portable map.

## TensorSwarm application

Relevant intents are capacity admission, launcher/port registration, security review, external research, GitHub/source control, and browser/native acceptance. Cargo and Windows-native tools remain the fallback when the corresponding MCP is not mounted.

## Consequences

Tool routing stays explicit without copying a stale or secret-bearing catalogue into the repository. Machine-local findings do not pollute commits.
