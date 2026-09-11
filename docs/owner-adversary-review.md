# Owner/adversary MVP review

Review basis: `SPEC.md`, `PLAN.md`, `TASKS.md`, and the workspace test results on the current branch.

Status vocabulary:

- PASS: implementation and automated evidence match the requirement.
- PARTIAL: the normal path exists, but an acceptance condition or failure path is not fully proven.
- BLOCKED: a required condition is absent or contradicts the specification.

## Specification review

| Area | Status | Evidence and remaining risk |
|---|---|---|
| Local GGUF/Safetensors inspection | PASS | Header-only parsers, bounds checks, malformed corpus, and generated chunk properties pass. Dedicated fuzz-target execution and broad real-format fixtures remain. |
| Deterministic identity and manifests | PASS | Manifest root and serialization tests pass; parser and manifest commands are implemented. |
| Verified storage and materialization | PASS | Hash-verified CAS writes and byte reconstruction tests pass. Repair and bitmap integration are not fully demonstrated. |
| LAN authenticated transfer | PARTIAL | Loopback nodes transfer verified chunks and use QUIC/TCP, Noise, and Yamux. The bounded acceptance coverage is still in-process rather than two independently spawned OS processes. |
| Kademlia provider discovery | PASS | `second_node_discovers_seeded_tensor_provider_through_dht` connects a second swarm, waits for routing-table admission, and verifies the seeded tensor provider is returned by `get_providers`. |
| Bounded transfer behavior | PARTIAL | Chunk fetches now send `Cancel` on cancellation/timeout and account for bounded outbound requests. A delayed-peer integration fixture is still needed to prove peer-observed cancellation and sustained in-flight limits. |
| Corrupt-peer handling | PASS | Malicious-peer integration fixtures send wrong hashes and oversized payloads; the client rejects both and does not accept the payload. |
| HTTP proxy and ranges | PASS | Full and single-range route tests verify status, Content-Range, Content-Length, and bytes. |
| Swarm-first proxy path | PASS | A real loopback HTTP-over-LAN test exercises the LanClient adapter and verified response assembly. |
| HTTPS fallback | PARTIAL | WebSeed range retrieval, verification, and origin-outage safety are implemented. A successful fallback-to-second-node publication test is still missing. |
| Python integration | PARTIAL | Direct proxy URL usage is documented and tested through the route. Transparent HTTP_PROXY/HTTPS_PROXY rewriting is not implemented. |

## Decision

The MVP remains BLOCKED. The current branch demonstrates the core parser, CAS,
LAN transfer, and proxy path, but it does not satisfy every P0 acceptance criterion
in `TASKS.md` or every M5 exit criterion in `PLAN.md`.

Required follow-up before declaring MVP complete:

1. Add an integration fixture proving peer-observed cancellation and bounded in-flight behavior.
2. Add a dedicated bounded fuzz target or equivalent independently runnable fuzz job.
3. Exercise successful HTTPS fallback, provider publication, and second-node discovery through the production fallback path.
4. Re-run the review from a clean checkout and record owner approval.
