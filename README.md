# TensorSwarm

TensorSwarm is an experimental, tensor-native P2P distribution engine for GGUF and Safetensors model artifacts.

## MVP status

The project is currently at the foundation stage. The roadmap targets a local end-to-end vertical slice:

```text
parse -> manifest -> verified cache -> LAN transfer -> HTTP/WebSeed proxy
```

See [PLAN.md](PLAN.md) for milestones and [TASKS.md](TASKS.md) for the executable task board.

To run a seeding node from a manifest and verified chunk store:

```text
ts-cli node model.tswarm .tswarm-cache
```

To run the manifest-aware localhost proxy with HTTPS WebSeed fallback:

```text
ts-cli proxy-manifest model.tswarm .tswarm-cache https://model.example/model.safetensors
ts-cli proxy-manifest 127.0.0.1:9090 model.tswarm .tswarm-cache https://model.example/model.safetensors
```

Clients can request `http://127.0.0.1:9090/file/<manifest-path>` directly. Transparent
`HTTP_PROXY`/`HTTPS_PROXY` origin-url handling remains a tracked M4-09 task.

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
