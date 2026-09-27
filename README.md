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

## Desktop application status

The Windows Tauri desktop app starts the local Rust engine automatically; users
do not need to start a daemon in a terminal. The engine exposes its authenticated
control API on loopback port 9090 and its model proxy on port 9091, and stores its
cache under the application data directory. Closing the main window hides the
app; the tray menu provides Show and Quit actions. The overview and local-engine
retry flow are implemented. The desktop has GGUF/Safetensors file and
recursive-folder import controls.
Windows debug acceptance verified file/folder selection, verified import,
byte-identical preparation, and an independent P2P chunk fetch. The current
unsigned Windows NSIS package was clean-installed and accepted through first
launch, Safetensors and GGUF fixture import/verification, byte-identical
preparation, process-restart persistence, uninstall readback, occupied-port
recovery, close-to-background, and native tray-menu Quit shutdown. Its
independent CLI fetch exited successfully, but the payload/hash comparison was
not retained, so it is not counted as verified package transfer. An Ubuntu
WSLg run of the packaged DEB verified visible model import/state, restart
persistence, and an independent 23-byte P2P fetch whose hash and bytes matched
the source tensor range. A current-source Deepin DEB built on 2026-09-27 and passed package metadata,
dependency, and GLIBC 2.38 preflight checks; installation/removal remains
sudo-gated. A later isolated extracted-package launch could not be read back
after SSH to Deepin timed out. WSLg is not native Linux acceptance. Native Linux
install/runtime/tray acceptance remains open. See [TASKS.md](TASKS.md) and
[PROGRESS.md](PROGRESS.md) for evidence and open gates. Headless
`ts-daemon` and `ts-cli` workflows remain available for server/operator use.
Internet-wide model search, share links, and public tracker/rendezvous are not
implemented; public publishing is planned behind rights, abuse, and privacy
gates. See the [research and roadmap note](docs/model-discovery-publication-and-abuse.md).
For a locally exchanged signed `.tsrelease` bundle and its separately supplied
single-file `.tswarm` manifest, an operator can check their binding with
`ts-cli verify-release bundle.tsrelease model.tswarm [tswarm://v1/<digest>]`.
Desktop signed-bundle export stages and reads back the bundle, then commits it
without replacing an existing destination.
In the desktop Model library, select an imported model and use "Export selected
.tswarm manifest" to save that model's manifest separately from its signed
release bundle. Export refuses to overwrite an existing file. The manifest
contains metadata and chunk hashes, not model bytes or a network address.
CLI verification checks the self-asserted signature, optional exact share identifier, and
manifest metadata/root; it does not fetch or validate the model bytes, establish
publisher identity or rights, or make the identifier a download link.
For an offline recipient inbox, use
`ts-cli receive-release bundle.tsrelease model.tswarm <inbox-dir> [tswarm://v1/<digest>]`.
It saves the checked release and manifest together under the immutable release
ID, rejects different existing records, and does not add a model to the active
library or trust its bytes. Pass the optional identifier when one was supplied
through a separate channel to bind the exchange to that exact signed envelope.
With an explicitly supplied LAN peer ID/address, a recipient can then run
`ts-cli fetch-received <inbox-dir>/<release-id> <peer-id> <peer-address> <store-root> <output>`.
This rechecks the signed record and manifest before fetching its referenced
chunks, stores only hash-verified bytes, and prepares the single file. It uses
an independent recipient cache and refuses existing output/partial files.
Preparation stages beside the destination and commits with a no-replace
hard-link; it fails closed without overwriting a file created while fetching.
The destination filesystem must support hard links.
Peer identity and address must be supplied out of band; this is not discovery,
a trust decision about publisher identity, or a public sharing service.
The desktop Model library also has a private recipient-inbox action: optionally
enter the independently received `tswarm://` identifier, then choose the signed
bundle and matching `.tswarm` manifest. It saves checked metadata under app
data `models/inbox/<release-id>/` without adding a model to the active library.
The desktop rechecks every stored signature, manifest binding, and record ID on
startup or "Refresh inbox" before listing receipts; a changed record produces an
inbox error rather than a partial list. Interrupted staging directories are not
receipts. This does not verify model bytes. An isolated Windows debug acceptance
used the native pickers to save a metadata-only receipt and read it back after
app restart; the active library remained unchanged and no bytes were fetched.
This is not an independent-user pilot, packaged-install acceptance, or Linux
Secret Service acceptance. A browser fixture separately exercised refresh,
escaped display, and the verification-error state.

See [ROADMAP.md](ROADMAP.md) for the ordered next development slices,
[SPEC.md](SPEC.md) for product/protocol requirements, [PLAN.md](PLAN.md) for
milestone strategy, and [TASKS.md](TASKS.md) for the executable task board.
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
- [ROADMAP.md](ROADMAP.md) — ordered next slices and acceptance focus
- [TASKS.md](TASKS.md) — task board and acceptance criteria
- [PROGRESS.md](PROGRESS.md) — dated implementation and verification evidence
- [docs/model-discovery-publication-and-abuse.md](docs/model-discovery-publication-and-abuse.md) — proposed model-card, directory/tracker, and abuse-resilience direction
- [docs/model-sharing-abuse-policy-draft.md](docs/model-sharing-abuse-policy-draft.md) — DRAFT only; pending owner and qualified legal review
