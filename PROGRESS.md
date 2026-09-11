# TensorSwarm progress handoff

Last updated: 2026-09-11
Branch: master
Working tree: clean
HEAD: 78a501e Add LAN transfer metrics

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
- Added a channel-backed `FetchEngine` notification adapter so verified WebSeed chunks can feed the live publication channel.
- Added the combined HTTPS fallback, live publication, and second-node discovery acceptance test.
- Re-ran the workspace and bounded fuzz-smoke verification from a clean detached checkout; all checks passed.
- Ran the actual `format_inspect` libFuzzer target under Ubuntu WSL2 nightly Rust: 1,000 bounded runs, 4 KiB maximum input, 219 coverage features, no crash.
- Added representative `.gguf` and `.safetensors` fixture routing/metadata coverage and removed a local HTTPS test startup race exposed by the full suite.
- Added and passed the independent-process `ts-cli` LAN acceptance test: a spawned provider node serves a verified chunk to a separately spawned fetch process.
- Added selective unreferenced-object repair to the content-addressed store; the live object is preserved and the dead object is removed by regression test.
- Added JSON-safe per-tensor verified-chunk bitmaps with restart persistence and same-index isolation coverage.
- Added `LanClient` transfer metric snapshots and asserted attempts, success, failures, cancellations, and bytes on the real two-node transfer.

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

Current unit-test counts are 1 (`ts-core`), 9 (`ts-format`), 14 (`ts-p2p`), 10 (`ts-proxy`), 6 (`ts-store`), and 1 (`ts-cli` integration test), with doc-tests passing. The Criterion harness also compiled and ran; the recorded short run measured approximately 50 ns for 64 MiB chunk planning and 1.96 ms for 4 MiB SHA-256 on the documented host.

## MVP decision

The owner approved MVP closure based on the clean-checkout, supported-host fuzz,
format-fixture, and HTTPS-to-live-DHT evidence. M5 is accepted with documented
limitations in `docs/owner-adversary-review.md`.

Post-MVP limitations and deferred work:

1. Transparent `HTTP_PROXY`/`HTTPS_PROXY` rewriting is explicitly deferred; MVP clients use the explicit localhost proxy URL.

## Recommended first slice tomorrow

Start with the delayed-peer cancellation/backpressure integration fixture. Keep the
fixture bounded and leave the repository clean after formatting, workspace tests, and
`git diff --check`.

## Reporting note

The LAN announcement daemon at `192.168.5.229:4100` was unreachable during the final
handoff check-in and its one retry. No work was blocked by the reporting channel.
