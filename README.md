# TensorSwarm

TensorSwarm is an experimental, tensor-native P2P distribution engine for GGUF and Safetensors model artifacts.

## MVP status

The local end-to-end MVP is accepted with limitations. The implemented vertical slice is:

```text
parse -> manifest -> verified cache -> LAN transfer -> HTTP/WebSeed proxy
```

M0 through M3 and M5 are complete. M4's explicit localhost proxy, LAN fetch,
HTTPS WebSeed fallback, complete-file preparation, and verification paths are
implemented and tested. Transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting is
deferred as post-MVP work.

See [PLAN.md](PLAN.md) for milestones and [TASKS.md](TASKS.md) for the executable task board.
See [docs/demo.md](docs/demo.md) for clean-checkout two-node demonstration steps.
See [docs/benchmarks.md](docs/benchmarks.md) for the measured primitive benchmark snapshot.
See [docs/owner-adversary-review.md](docs/owner-adversary-review.md) for the current MVP gate review.
See [PROGRESS.md](PROGRESS.md) for the current continuation handoff.

To run a seeding node from a manifest and verified chunk store:

```text
ts-cli node model.tswarm .tswarm-cache
```

To run the manifest-aware localhost proxy with HTTPS WebSeed fallback:

```text
ts-cli proxy-manifest model.tswarm .tswarm-cache https://model.example/model.safetensors
ts-cli proxy-manifest 127.0.0.1:9090 model.tswarm .tswarm-cache https://model.example/model.safetensors
# Optional LAN peer: <peer-id> <multiaddress>
ts-cli proxy-manifest 127.0.0.1:9090 model.tswarm .tswarm-cache https://model.example/model.safetensors <peer-id> <multiaddress>
```

Clients must request `http://127.0.0.1:9090/file/<manifest-path>` directly. Transparent
`HTTP_PROXY`/`HTTPS_PROXY` origin-url rewriting is intentionally out of scope for the MVP;
the explicit localhost URL avoids hidden routing and environment-dependent behavior.

### Python client example

The supported client integration uses the proxy URL explicitly. This works with
`requests` and preserves normal range requests used by model loaders:

```python
import requests

proxy_file = "http://127.0.0.1:9090/file/model.safetensors"
with requests.get(proxy_file, stream=True, timeout=30) as response:
    response.raise_for_status()
    with open("model.safetensors", "wb") as output:
        for block in response.iter_content(chunk_size=1024 * 1024):
            if block:
                output.write(block)
```

Do not set `HTTP_PROXY` or `HTTPS_PROXY` and expect arbitrary origin URLs to be
rewritten yet; transparent environment-variable integration is not implemented.

## Local verification

Once Rust is installed, run:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

Do not call the MVP complete until the owner/adversary verification tasks in `TASKS.md` have passed.

## Agents Protocol integration

This application adopts the project-shaped Agents Protocol spine for grounding, MCP intent routing, isolated parallel work, and independent acceptance. Start with [AGENTS.md](AGENTS.md) and [BOOTSTRAP.md](BOOTSTRAP.md); machine-local discovery belongs in ignored `MCP.local.md`.

## Documentation spine

- [AGENTS.md](AGENTS.md) — agent behavior and Verification Ladder
- [MCP.md](MCP.md) — intent-to-tool routing
- [CONTEXT.md](CONTEXT.md) — project context and constraints
- [SPEC.md](SPEC.md) — externally observable MVP contract
- [ARCHITECTURE.md](ARCHITECTURE.md) — component boundaries and data flow
- [DESIGN.md](DESIGN.md) — implementation design
- [PLAN.md](PLAN.md) — milestones and gates
- [TASKS.md](TASKS.md) — task board and acceptance criteria
