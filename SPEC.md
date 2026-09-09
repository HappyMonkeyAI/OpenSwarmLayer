# TensorSwarm MVP Specification

## Scope

The MVP accepts a local GGUF or Safetensors file, extracts tensor metadata without loading all weights, computes deterministic tensor and chunk identities, transfers verified chunks over a local P2P network, and serves materialized files through a localhost HTTP endpoint with HTTPS range fallback.

## Identity

All canonical encodings use explicit versioning, length prefixes, stable field ordering, and big-endian integer encoding unless the source format requires otherwise. Tensor identity includes tensor name, shape, dtype/layout, payload length, and payload bytes. Chunk identity includes tensor identity, chunk index, byte range, and payload bytes.

The initial cryptographic digest is SHA-256. The implementation must hash incrementally.

## Manifest requirements

A `.tswarm` manifest must contain:

- schema version;
- artifact format;
- tensor descriptors and chunk references;
- Merkle root;
- original file size;
- a file recipe containing literal, tensor, and zero-fill segments;
- optional upstream origin metadata.

Tensor descriptors are sorted by canonical tensor name. A manifest generated twice from the same source must have identical bytes.

## Verification requirements

- Every source tensor range must be within file bounds.
- Chunk ranges must cover each tensor exactly once.
- Remote chunks must be hash-verified before durable storage.
- Materialized output must match the manifest recipe.

## MVP network contract

The initial protocol uses versioned request/response messages for manifest and chunk retrieval. mDNS discovers local peers. Kademlia provider records locate peers for manifests and tensors. The wire protocol must support bounded frames, cancellation, timeouts, and resumable chunk transfer.

## MVP proxy contract

The daemon binds to `127.0.0.1:9090`, supports normal GET and byte-range requests, prefers verified swarm data, and falls back to HTTPS Range requests when peer data is unavailable. Fallback bytes become seedable only after verification.
