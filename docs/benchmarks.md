# TensorSwarm benchmark snapshot

This snapshot was measured on `stephen-desktop` using Windows/MSYS, 16 logical
CPUs, and Rust 1.98.1. It is a smoke benchmark for primitive operations, not a
LAN throughput claim.

Command:

```text
cargo bench -p ts-format --bench format_primitives -- --warm-up-time 0.1 --measurement-time 0.2 --sample-size 10
```

Results from the recorded run:

- `plan_chunks_64MiB`: 50.446 ns median estimate, interval 49.940–51.690 ns.
- `sha256_4MiB`: 1.9629 ms median estimate, interval 1.9540–1.9690 ms.

The benchmark harness lives at `crates/ts-format/benches/format_primitives.rs`.
Run with Criterion's default measurement settings for more stable comparisons;
the short settings above are intended for quick verification.
