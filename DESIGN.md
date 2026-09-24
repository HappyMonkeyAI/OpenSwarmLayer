# TensorSwarm MVP Design

## Crate responsibilities

`ts-core` owns stable identity and serialization primitives. `ts-format` owns bounded source readers and format-specific tensor ranges. `ts-store` owns atomic verified objects and materialization. `ts-p2p` owns peer discovery and transfer. `ts-proxy` owns HTTP semantics and origin fallback. `ts-cli` is the user-facing composition layer.

## Parser design

Readers use positional reads against a file handle. Header counts, string lengths, metadata counts, tensor ranks, and offsets are bounded before allocation. Tensor payloads are read in fixed buffers only while hashing or copying a requested chunk.

## Chunk design

MVP uses fixed-size chunks with boundaries aligned to the source datatype block where applicable. Each tensor records ordered chunks. This supports resumability and delta pulls without requiring content-defined chunking.

## Materialization design

The materializer writes literal and tensor segments at their specified file offsets. Tensor ranges are assembled from their ordered, hash-verified CAS chunks; a separate whole-tensor CAS object is not required. It maintains a durable verified-chunk bitmap and uses atomic temporary files for metadata/state updates. The output is exposed to clients only for ranges that are present and verified.

## P2P design

Control messages are bounded and metadata-oriented. Chunk payloads are length-delimited and independently hashed. A peer may serve only objects it has locally verified. The scheduler retries failed peers but never substitutes a different expected hash.

## Proxy design

The proxy first maps an upstream artifact to a manifest. An HTTP range is translated into the minimum required chunks. Missing chunks are fetched from peers, then HTTPS origin. The response is emitted only after the requested range is available.
