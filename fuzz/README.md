# TensorSwarm fuzz target

`format_inspect` is a cargo-fuzz/libFuzzer target for the bounded GGUF and
Safetensors inspection boundary. Inputs are capped at 4 MiB and parser panics
are treated as failures.

Run on a host with cargo-fuzz/libFuzzer support:

    cargo fuzz run --manifest-path fuzz/Cargo.toml format_inspect -- -max_len=4194304

On Windows MSVC, `libfuzzer-sys` may compile but fail to link because the
libFuzzer entry point is unavailable. The target can still be compile-checked:

    cargo check --manifest-path fuzz/Cargo.toml --bin format_inspect

The bounded fallback smoke run is portable and exercises the existing 512-case
malformed corpus:

    cargo run --manifest-path fuzz/Cargo.toml --bin format_inspect_smoke
