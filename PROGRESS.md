# TensorSwarm progress handoff

Last updated: 2026-09-10
Branch: master
Working tree: clean
HEAD: 6a7ed74 Record MVP owner adversary review

## Completed this session

- Added manifest-aware `proxy-manifest` CLI support and hardened manifest range assembly.
- Added cancellable and bounded LAN fetch primitives.
- Added requester-side `Have` availability exchange and deterministic LAN peer scoring.
- Added deterministic manifest-path and HTTPS-origin resolution.
- Connected the proxy to the LAN client with tensor identity and chunk-index propagation.
- Added verified-fetch provider notifications and LAN tensor DHT publication.
- Added optional CLI peer identity and multiaddress configuration.
- Wired configured CLI peers to receive WebSeed verification publication notifications.
- Added a real loopback HTTP-over-LAN proxy acceptance test.
- Added Python direct-proxy documentation and a clean-checkout demo guide.
- Added a bounded 512-case malformed parser corpus and generated `proptest` chunk-coverage checks.
- Added Criterion benchmarks and recorded a reproducible primitive benchmark snapshot.
- Added the owner/adversary review; MVP remains explicitly blocked.

## Recent commits

- `6a7ed74` — Record MVP owner adversary review
- `de9ee25` — Add reproducible format benchmarks
- `6031357` — Add generated chunk planning properties
- `e7755d2` — Correct demo acceptance commands
- `f9b6758` — Document clean checkout two-node demo
- `dc9946f` — Disconnect peers after invalid chunk responses
- `e482e74` — Add bounded malformed parser corpus
- `f9cbfb4` — Test proxy origin outage safety
- `bf30ff7` — Wire WebSeed provider publication in CLI
- `26205df` — Add CLI LAN peer configuration
- `6a682f7` — Verify proxy HTTP delivery from LAN peer

## Verification status

The latest full workspace verification passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `git diff --check`

Current unit-test counts are 1 (`ts-core`), 8 (`ts-format`), 10 (`ts-p2p`), 8 (`ts-proxy`), 4 (`ts-store`), and 0 (`ts-cli`), with doc-tests passing. The Criterion harness also compiled and ran; the recorded short run measured approximately 50 ns for 64 MiB chunk planning and 1.96 ms for 4 MiB SHA-256 on the documented host.

## MVP decision

Do not declare the MVP complete. The owner/adversary review in
`docs/owner-adversary-review.md` marks M5 `BLOCKED`.

Remaining acceptance gaps:

1. Add a real two-process or DHT provider-discovery acceptance test.
2. Implement wire-level cancellation and complete in-flight backpressure accounting.
3. Add malicious-peer integration fixtures for wrong hashes and oversized frames.
4. Add and run a dedicated bounded fuzz target; current coverage is corpus plus `proptest` only.
5. Prove successful HTTPS fallback, provider publication, and second-node discovery.
6. Decide whether transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting is required; it is currently unsupported and documented as such.
7. Re-run the review from a clean checkout and record owner approval.

## Recommended first slice tomorrow

Start with the two-process/DHT acceptance test. Reuse the existing loopback provider and
proxy fixtures, move the provider and requester into independently spawned processes or
independent test binaries, verify provider lookup returns the reachable peer, then issue
a proxy range request and read back the verified CAS object. Keep the test bounded and
leave the repository clean after formatting, workspace tests, and `git diff --check`.

## Reporting note

The LAN announcement daemon at `192.168.5.229:4100` was unreachable during the final
handoff check-in and its one retry. No work was blocked by the reporting channel.
