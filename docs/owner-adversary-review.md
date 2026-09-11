# Owner/adversary MVP review

Review basis: `SPEC.md`, `PLAN.md`, `TASKS.md`, workspace test results on the current branch, and a clean detached-checkout verification run at `50b325c`.

Status vocabulary:

- PASS: implementation and automated evidence match the requirement.
- PARTIAL: the normal path exists, but an acceptance condition or failure path is not fully proven.
- BLOCKED: a required condition is absent or contradicts the specification.
- ACCEPTED WITH LIMITATIONS: the owner approved MVP closure while explicitly retaining documented non-MVP or host/process limitations.

## Specification review

| Area | Status | Evidence and remaining risk |
|---|---|---|
| Local GGUF/Safetensors inspection | PASS | Header-only parsers, bounds checks, representative `.gguf`/`.safetensors` fixture routing and metadata tests, generated chunk properties, and a bounded libFuzzer target are present. Ubuntu WSL2 executed 1,000 bounded runs with 219 coverage features and no crash. |
| Deterministic identity and manifests | PASS | Manifest root and serialization tests pass; parser and manifest commands are implemented. |
| Verified storage and materialization | PASS | Hash-verified CAS writes, byte reconstruction, restart-persistent per-tensor verified state with index isolation, reachability scanning, and selective unreferenced-object repair tests pass. |
| LAN authenticated transfer | PASS | The independent-process `ts-cli` acceptance test spawns a manifest-backed provider node and a separate fetch process; Node B retrieves and verifies the chunk over the authenticated LAN transport. |
| Kademlia provider discovery | PASS | `second_node_discovers_seeded_tensor_provider_through_dht` connects a second swarm, waits for routing-table admission, and verifies the seeded tensor provider is returned by `get_providers`. |
| Bounded transfer behavior | PASS | Chunk fetches send `Cancel` on cancellation/timeout, flush the wire signal within a bounded window, release peer-associated in-flight state, and the delayed-peer integration fixture verifies peer observation. |
| Corrupt-peer handling | PASS | Malicious-peer integration fixtures send wrong hashes and oversized payloads; the client rejects both and does not accept the payload. |
| HTTP proxy and ranges | PASS | Full and single-range route tests verify status, Content-Range, Content-Length, and bytes. |
| Swarm-first proxy path | PASS | A real loopback HTTP-over-LAN test exercises the LanClient adapter and verified response assembly. |
| HTTPS fallback | PASS | The combined acceptance test proves HTTPS range retrieval, hash verification, CAS persistence, channel-backed live publication, and second-node DHT discovery. |
| Python integration | PASS for MVP scope | Direct proxy URL usage is documented and tested through the route. Transparent HTTP_PROXY/HTTPS_PROXY rewriting is intentionally deferred as P1/M4-09, not required for the MVP contract. |

## Decision

The MVP is ACCEPTED WITH LIMITATIONS. The owner approved the current branch after
reviewing the clean-checkout, supported-host fuzz, format-fixture, and combined
HTTPS-to-live-DHT evidence.

Documented post-MVP limitations:

1. No MVP action remains for transparent HTTP_PROXY/HTTPS_PROXY rewriting; it is explicitly deferred as P1/M4-09.
