# MCP.md — TensorSwarm intent routing

This is a portable routing map, not a catalogue dump. Listed servers must be confirmed in the live session before use.

| Intent | Preferred tool/server | TensorSwarm use |
|---|---|---|
| Tool discovery and activation | `dynamic_proxy` | Find or activate specialty MCPs before falling back to shell improvisation. |
| Capacity before heavy work | `resource_sentinel` | Full Cargo suites, packaging, fuzzing, indexing, or parallel agents. |
| Project ports and runtime registration | `launcher_registry` | Check daemon, proxy, Tauri, and fixture ports before binding. |
| Security review | `auditscan` | Release, installer, auth-boundary, and dependency checks when mounted. |
| External research | `article_research` | Architecture/library research with cited evidence. |
| Browser/native acceptance | BrowserOS or Playwright MCP | Tauri/control-surface interaction and state readback when available. |
| Source-of-truth repository work | `git` / GitHub tooling | Diff, history, remote comparison, and CI evidence. |

## Rules

1. Route by intent, then verify the live catalogue; listed does not mean mounted.
2. Keep secrets, tokens, and raw credential paths out of this file and `MCP.local.md`.
3. Keep machine-specific mounts, gaps, and LAN hints in ignored `MCP.local.md`.
4. Use the repository's documented Cargo and Windows acceptance commands when no specialized MCP is available.

See `BOOTSTRAP.md` and `docs/adr/0004-mcp-intent-map-and-bootstrap-discovery.md`.
