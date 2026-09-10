# TensorSwarm

TensorSwarm is an experimental, tensor-native P2P distribution engine for GGUF and Safetensors model artifacts.

## MVP status

The project is currently at the foundation stage. The roadmap targets a local end-to-end vertical slice:

```text
parse -> manifest -> verified cache -> LAN transfer -> HTTP/WebSeed proxy
```

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

Clients can request `http://127.0.0.1:9090/file/<manifest-path>` directly. Transparent
`HTTP_PROXY`/`HTTPS_PROXY` origin-url handling remains a tracked M4-09 task.

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

## Documentation spine

- [CONTEXT.md](CONTEXT.md) — project context and constraints
- [SPEC.md](SPEC.md) — externally observable MVP contract
- [ARCHITECTURE.md](ARCHITECTURE.md) — component boundaries and data flow
- [DESIGN.md](DESIGN.md) — implementation design
- [PLAN.md](PLAN.md) — milestones and gates
- [TASKS.md](TASKS.md) — task board and acceptance criteria
