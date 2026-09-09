# ADR-0001: Manifest and Tensor Identity

## Status

Accepted for MVP.

## Decision

TensorSwarm uses SHA-256 over a versioned canonical encoding containing tensor name, shape, dtype/layout, payload length, and payload bytes. Chunk hashes are separate identities containing the tensor identity, chunk index, range, and bytes. Manifest descriptors are sorted by canonical tensor name before Merkle-root calculation.

## Rationale

The semantic tensor identity must remain stable when a model is repackaged, sharded, or given different metadata/file offsets. Including the name and shape prevents accidental reuse of byte-identical payloads with incompatible meaning. Explicit versioning leaves room for future digest or canonicalization upgrades.

## Consequences

- Hashing must be incremental.
- The manifest must contain a file-layout recipe because tensor hashes alone cannot reconstruct source files.
- Changes to canonicalization require a new manifest schema or identity version.
- SHA-256 is not a publisher-authenticity mechanism; signatures remain a later decision.
