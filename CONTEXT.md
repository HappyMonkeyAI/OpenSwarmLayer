# TensorSwarm Context

## Purpose

TensorSwarm is an open-source, tensor-native distribution engine for GGUF and Safetensors artifacts. It is intended to reduce repeated centralized downloads while retaining HTTPS compatibility and ordinary runtime file formats.

## Current state

The repository is at M0 foundation. The Rust workspace and crate boundaries exist; format parsing, manifests, networking, storage, and proxy behavior are not implemented yet.

## MVP constraints

- Rust and Tokio are the implementation baseline.
- Integrity must be verifiable before data enters durable storage.
- The first network target is two nodes on one LAN.
- The first client compatibility target is ordinary HTTP and HTTP Range behavior.
- Existing runtime compatibility takes priority over transparent lazy mmap behavior.
- MVP uses fixed-size chunks and SHA-256; dynamic quantization is deferred.

## What not to do yet

- Do not build a global relay/bootstrap service before the local vertical slice works.
- Do not add a custom runtime or inference engine.
- Do not claim tensor identity alone can reconstruct a file; the manifest must include a file layout recipe.
- Do not commit or cache unverified remote bytes.
