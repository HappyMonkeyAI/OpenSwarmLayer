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

## M7-01 model publication schema

`ts-core::ModelCard` holds human-readable model information separately from
`ModelReleaseDescriptor`, which binds a card to one or more artifact variants.
Each variant has its own GGUF or Safetensors format and quantization label and
may contain multiple files/shards. Each file references its size, tensor count,
relative path, shard position, and existing manifest root. This permits mixed
formats across variants without changing the current single-format library
catalog or redefining any manifest identity. These file fields are untrusted
discovery metadata until compared with the fetched, root-verified manifest.

`MetadataOrigin` distinguishes parsed-from-artifact values from publisher-
declared values, but the label is untrusted when received remotely and must be
verified by the client. Catalog review/moderation state is not part of the
publisher-controlled card; a future directory must own that state separately.
Descriptor validation rejects unsupported schema versions, unknown serialized
fields, unsafe or duplicate paths, invalid shard positions/groups, and empty
artifact references.

`ts-core::SignedModelRelease` adds a versioned Ed25519 envelope around the
descriptor. Its domain-separated signature covers the schema, signer public
key, and CBOR descriptor bytes, including every artifact manifest root.
`to_bytes`/`from_bytes` provide bounded (4 MiB) signed-envelope export/import;
import validates the schema, descriptor, public key, and signature. The
`tswarm://v1/<sha256>` local share-link identifier pins the exact encoded signed
envelope and can be checked against the link. It is an identifier only; it does
not resolve or fetch a release.

The desktop's M7-03 local signer stores a random 32-byte Ed25519 seed in the OS
credential store via `keyring` (Windows Credential Manager, macOS Keychain, or
Linux Secret Service). It creates the key only after an explicit UI action,
performs synchronous store access on blocking workers, exposes only a SHA-256
public-key fingerprint, and fails closed if the OS store is unavailable or the
stored seed is malformed. There is no plaintext fallback, private-key export,
backup, recovery, or rotation; loss of the credential store means a new key has
no continuity with old releases.

The desktop can export a `.tsrelease` signed descriptor from a locally verified
single-file manifest, and import a bounded descriptor for signature validation
and preview. Import compares descriptor fields against local manifest roots,
file paths, sizes, formats, and tensor counts; this metadata match is not a
re-read of all artifact bytes. Imported descriptors are not persisted to a
catalog. Signatures and fingerprints prove only possession/continuity of a
self-asserted key over exact descriptor bytes; they do not establish real-world
identity, redistribution rights, legality, safety, independent artifact
verification, or moderation status. The `tswarm://v1/<sha256>` identifier is
still local-only; no URI handler, resolver, directory, or Internet publishing
path exists. Public publishing remains blocked pending qualified legal and
operational review.
