# TensorSwarm progress handoff

Last updated: 2026-09-11
Branch: master
Working tree: clean
HEAD: 8a62ea4 Wire live DHT publication events

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
- Added a bounded two-node Kademlia provider-discovery acceptance test that proves a second node finds a seeded tensor provider.
- Added bounded LAN request accounting and wire cancellation signaling for chunk fetches.
- Added malicious-peer integration fixtures covering wrong hashes and oversized responses.
- Added an excluded `fuzz` package with a bounded libFuzzer `format_inspect` target, portable fallback smoke runner, and usage documentation.
- Added a delayed-peer cancellation fixture and fixed peer-associated in-flight cleanup plus bounded cancel-frame flushing.
- Added a local HTTPS WebSeed acceptance test covering range retrieval, hash verification, CAS persistence, and provider notification.
- Added a live LAN-node publication event channel and changed the DHT discovery fixture to publish through that runtime path.

## Recent commits

- `e7c012c` — Verify delayed peer cancellation
- `a7e5a79` — Ignore workspace build artifacts
- `7efb0cb` — Ignore fuzz build artifacts
- `370bc55` — Add bounded parser fuzz target
- `6a7ed74` — Record MVP owner adversary review
- `c39904c` — Add malicious peer rejection fixtures
- `2e0ca4d` — Bound LAN requests and signal cancellation
- `3a97328` — Record malicious peer acceptance evidence
- `85cc3da` — Add DHT provider discovery acceptance test
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
- `cargo check --manifest-path fuzz/Cargo.toml --bin format_inspect`
- `cargo run --manifest-path fuzz/Cargo.toml --bin format_inspect_smoke`
- `cargo test -p ts-proxy https_webseed_fallback_verifies_stores_and_notifies`

Current unit-test counts are 1 (`ts-core`), 8 (`ts-format`), 14 (`ts-p2p`), 9 (`ts-proxy`), 4 (`ts-store`), and 0 (`ts-cli`), with doc-tests passing. The Criterion harness also compiled and ran; the recorded short run measured approximately 50 ns for 64 MiB chunk planning and 1.96 ms for 4 MiB SHA-256 on the documented host.

## MVP decision

Do not declare the MVP complete. The owner/adversary review in
`docs/owner-adversary-review.md` marks M5 `BLOCKED`.

Remaining acceptance gaps:

1. Execute the libFuzzer target on a host with a working cargo-fuzz/libFuzzer linker; this Windows MSVC host only proves target compilation and the 512-case fallback smoke run.
2. Wire HTTPS fallback notifications into the live publication event channel and prove the complete second-node discovery path.
3. Decide whether transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting is required; it is currently unsupported and documented as such.
4. Re-run the review from a clean checkout and record owner approval.

## Recommended first slice tomorrow

Start with the delayed-peer cancellation/backpressure integration fixture. Keep the
fixture bounded and leave the repository clean after formatting, workspace tests, and
`git diff --check`.

## Reporting note

The LAN announcement daemon at `192.168.5.229:4100` was unreachable during the final
handoff check-in and its one retry. No work was blocked by the reporting channel.
