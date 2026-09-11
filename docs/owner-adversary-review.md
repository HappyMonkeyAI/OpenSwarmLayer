# Owner/adversary MVP review

Review basis: `SPEC.md`, `PLAN.md`, `TASKS.md`, workspace test results on the current branch, and a clean detached-checkout verification run at `50b325c`.

Status vocabulary:

- PASS: implementation and automated evidence match the requirement.
- PARTIAL: the normal path exists, but an acceptance condition or failure path is not fully proven.
- BLOCKED: a required condition is absent or contradicts the specification.

## Specification review

| Area | Status | Evidence and remaining risk |
|---|---|---|
| Local GGUF/Safetensors inspection | PARTIAL | Header-only parsers, bounds checks, malformed corpus, generated chunk properties, and a bounded libFuzzer target are present. The target compiles and the fallback smoke runs on Windows MSVC, but libFuzzer execution requires a supported linker/host. |
| Deterministic identity and manifests | PASS | Manifest root and serialization tests pass; parser and manifest commands are implemented. |
| Verified storage and materialization | PASS | Hash-verified CAS writes and byte reconstruction tests pass. Repair and bitmap integration are not fully demonstrated. |
| LAN authenticated transfer | PARTIAL | Loopback nodes transfer verified chunks and use QUIC/TCP, Noise, and Yamux. The bounded acceptance coverage is still in-process rather than two independently spawned OS processes. |
| Kademlia provider discovery | PASS | `second_node_discovers_seeded_tensor_provider_through_dht` connects a second swarm, waits for routing-table admission, and verifies the seeded tensor provider is returned by `get_providers`. |
| Bounded transfer behavior | PASS | Chunk fetches send `Cancel` on cancellation/timeout, flush the wire signal within a bounded window, release peer-associated in-flight state, and the delayed-peer integration fixture verifies peer observation. |
| Corrupt-peer handling | PASS | Malicious-peer integration fixtures send wrong hashes and oversized payloads; the client rejects both and does not accept the payload. |
| HTTP proxy and ranges | PASS | Full and single-range route tests verify status, Content-Range, Content-Length, and bytes. |
| Swarm-first proxy path | PASS | A real loopback HTTP-over-LAN test exercises the LanClient adapter and verified response assembly. |
| HTTPS fallback | PASS | The combined acceptance test proves HTTPS range retrieval, hash verification, CAS persistence, channel-backed live publication, and second-node DHT discovery. |
| Python integration | PASS for MVP scope | Direct proxy URL usage is documented and tested through the route. Transparent HTTP_PROXY/HTTPS_PROXY rewriting is intentionally deferred as P1/M4-09, not required for the MVP contract. |

## Decision

The MVP remains BLOCKED. The current branch demonstrates the core parser, CAS,
LAN transfer, and proxy path, but it does not satisfy every P0 acceptance criterion
in `TASKS.md` or every M5 exit criterion in `PLAN.md`.

Required follow-up before declaring MVP complete:

1. Execute the bounded libFuzzer target on a supported linker/host; Windows MSVC currently has compile-only plus fallback-smoke evidence.
2. No MVP action remains for transparent HTTP_PROXY/HTTPS_PROXY rewriting; it is explicitly deferred as P1/M4-09.
3. Record explicit owner approval after reviewing the clean-checkout evidence.
