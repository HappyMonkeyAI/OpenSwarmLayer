# Model metadata, Internet discovery, and abuse-resistance planning

Status: research input for roadmap planning, not an implemented design, platform policy, or legal advice.

## Executive recommendation

Treat three capabilities as separate layers: (1) a model card that describes a model and its source, (2) a versioned share descriptor that binds a card to immutable artifact-manifest roots, and (3) a discovery service that supports both human search and live peer rendezvous. A torrent-style tracker can help locate peers for a known content identifier; it does not by itself provide the searchable, curated model directory users expect. TensorSwarm already has hash-keyed DHT provider lookup, but no public model catalog or publisher metadata.

Do not open public Internet publishing until the product has explicit rights/licence declarations, a content policy, abuse reporting and review, a delisting/appeal process, and operational controls. Hash verification, file-format validation, and publisher signatures address integrity or origin—not legality, licensing, or safety.[9]

## Research findings

Hugging Face model cards pair Markdown descriptions with YAML metadata for discovery. Its documented fields include task/pipeline, library, language, license, datasets, base model, and tags; the narrative is intended to explain uses, limitations, training/evaluation information, and related context.[1][3] Its GGUF documentation lists quantization types, which is a useful precedent for making quantization a property of an artifact variant rather than a single model-repository field.[2]

For license metadata, use a normalized SPDX identifier or expression where one applies, but retain the uploader's original name, source URL, and any supplied license text. SPDX provides standardized identifiers and canonical URLs; that normalization is a way to describe a claim, not proof that the publisher owns redistribution rights.[4]

BitTorrent's magnet design separates content identity from optional discovery hints: the `xt` identifier is mandatory, while display name and tracker URLs are optional; when no tracker is provided, the specification recommends DHT peer discovery.[5] The DHT stores peer contact information for an exact key,[6] and tracker requests are keyed by the metainfo info-hash.[7] The design implication is that a TensorSwarm tracker/rendezvous can help a client locate peers for a known share, while name/tag/model-card search needs a separate directory API. The two functions could initially be operated by one service, but should remain distinct interfaces and data models.

Hugging Face's public policy is an example of platform operations that include user reporting and an IP takedown-notice path; it is not a ready-made legal template for TensorSwarm.[8] Its security documentation also distinguishes signed origin from file safety, and warns against loading untrusted pickle files.[9] TensorSwarm should likewise distinguish cryptographic integrity, publisher identity, file-format/security checks, and rights/content-policy review.

## Current TensorSwarm baseline

- `Manifest` currently identifies format, file size, tensors, file recipes, and a Merkle-style root. `compute_root` covers those fields; there is no model-card, publisher, license, or release metadata in the manifest (`crates/ts-core/src/lib.rs:68-117`).
- The daemon's `/v1/models` payload exposes only path, format, file size, tensor count, and manifest root (`crates/ts-daemon/src/lib.rs:110-117,257-271`).
- P2P has manifest/tensor provider publish and lookup operations keyed by hash (`crates/ts-p2p/src/lib.rs:684-716`). Those are provider lookups, not text/tag search or a persistent public repository index.
- The desktop importer currently accepts GGUF and Safetensors paths (`crates/ts-format/src/lib.rs:34-52`), persists one-file manifests, and rejects mixed-format libraries (`desktop/src-tauri/src/main.rs:83-132`). This is adequate for the current vertical slice, but not a Hugging Face-style repository containing multiple files, shards, and quantized variants.

Keep the content manifest and descriptive card separate. The manifest root should continue to identify verifiable artifact content; editable descriptions or license claims should not silently change the meaning of a tensor-content identity. A versioned, signed release descriptor can bind a card revision and one or more artifact-manifest roots together.

In the publisher descriptor, provenance labels can distinguish values declared
by the publisher from values parsed by a client, but a remote label is not proof
that parsing occurred. File sizes, tensor counts, and roots in a descriptor are
also discovery claims until checked against the fetched, root-verified manifest.
Keep operator review/moderation status in a separate catalog-owned record;
never allow a publisher-controlled card to self-assert that it is approved.

## Proposed metadata model

Model a logical repository as one or more immutable releases; each release can contain multiple artifact variants. Keep parsed facts separate from uploader declarations and reviewer decisions.

Suggested card fields:

- Human name, namespace/publisher, short summary, Markdown description, and searchable tags.
- Intended task/pipeline, model architecture/type, supported languages, base/upstream model, and derivation lineage (fine-tune, merge, or quantization source where known).
- Training datasets and methods, evaluation results, intended uses, known limitations, and safety/content notices when supplied.
- Declared license/SPDX expression, original license name and URL/text, source repository/revision, publisher identity, and an explicit rights-to-redistribute attestation.

Suggested artifact-variant fields:

- File name/path, format, file size, shard grouping, tensor count, manifest root, and verified chunk count.
- Quantization type and other format-specific facts, attached to each file/variant; indicate whether each field was parsed from the file, supplied by the publisher, or reviewed by an operator.
- A descriptor revision and signature over the card revision and artifact roots, with signer identity and key-rotation/revocation status.

Expose license and provenance state as `publisher-declared`, `reviewed`, or `unknown`; never render a declaration as a guarantee. Use SPDX normalization only where applicable and allow a clearly identified custom license reference.[1][3][4]

## Share links and discovery service

A future share descriptor/link should identify a specific immutable release or artifact root, not just a display name. A candidate scheme could carry a versioned descriptor ID plus optional registry/tracker URLs and direct-peer hints; whether to use a custom `ts://` URI, an exported descriptor file, or both remains an open design decision. Follow the useful magnet pattern—stable content identity with optional peer-discovery hints—without implying BitTorrent wire compatibility.[5]

Separate the following responsibilities even if the first hosted deployment combines them:

1. Model directory: persistent, searchable card/release metadata; publication state; signer and provenance claims; version/variant listings; reports and review state.
2. Peer rendezvous/tracker: short-lived provider leases keyed by descriptor/artifact root, with expiry and rate limits. It stores discovery hints, not model bytes. Document that announcing to a tracker can expose network addresses; minimize retained data and make the discovery mode visible to users.
3. P2P data plane: continue to transfer hash-verified chunks through the daemon. Search results and tracker responses are untrusted hints; clients must verify descriptor signatures, manifest roots, and every chunk independently.

A centrally operated, moderated directory is the simplest first Internet pilot for search and takedown operations. Keep the peer protocol and descriptor format portable so independently operated registries can be considered later. The existing DHT can remain an optional provider-discovery path for known hashes; it is not a substitute for semantic catalog search and cannot provide reliable global delisting on its own.[5][6]

## Abuse-resilience and lawful-use gates

The purpose should remain model artifact exchange, not general file hosting. The public service should prohibit using it to distribute movies, pornography, or other non-model media, regardless of whether a particular copy might be lawful in one jurisdiction; it should also prohibit unlawful material and non-consensual sexual content. Whether model weights trained for or intended to generate explicit adult content are in scope is a separate owner/legal policy decision, not something a file hash or format parser can decide.

Minimum launch controls to design and test:

- Require publishers to identify the upstream source and declare rights to distribute each release; require a license field or explicitly mark it unknown. Keep these as claims, not automatic legal clearance.
- Provide a report/notice path, a documented review and escalation process, temporary hiding/delisting, a meaningful appeal/counter-notice path, and an operator audit trail. Delisting should stop new catalog results and peer announcements for the affected descriptor; do not promise deletion of copies already cached by independent peers.[8]
- Add publication/announce rate limits, quotas, account-abuse controls, and limits on metadata/file sizes. Test spam, Sybil-like account creation, spoofed publisher metadata, re-announcement after delisting, malformed descriptors, and denial-of-service inputs.
- Keep file-security scanning separate from content/rightsholder review. Never execute uploaded files or unpickle untrusted weights. A valid GGUF/Safetensors parse, a matching hash, or a valid signature does not certify that the content is lawful or safe.[9]
- Before enabling public DHT announcements, decide what can be revoked or suppressed and what cannot. A registry can remove its own listing and tracker lease, but cannot guarantee recall of already distributed bytes or erase arbitrary third-party DHT records.
- Minimize discovery logs and set retention/visibility rules for peer IP addresses and report evidence; document the privacy trade-off to publishers and downloaders.

This is a product-risk plan, not legal advice. Obtain legal review for the target jurisdictions, notice-handling obligations, privacy/retention, and the final content policy before operating a public index or tracker.

The proposed policy and threat/acceptance matrix are expanded in the explicitly
unapproved [public model-sharing policy draft](model-sharing-abuse-policy-draft.md).

## Proposed roadmap sequence

1. Specify a versioned model-card/release-descriptor schema, including per-variant quantization and declared-vs-parsed-vs-reviewed provenance. Preserve current manifest-root compatibility; resolve multi-file and mixed-format repository support.
2. Complete an abuse/privacy threat model and counsel-reviewed acceptable-use, rights-attestation, reporting, delisting, and appeal requirements before public publication is enabled.
3. Implement signed descriptor export/import and a versioned share-link contract. The local `ts-core` signature/envelope and `tswarm://v1/<sha256>` identifier codec now cover this first core slice; desktop key management/UI, remote resolution, unknown-field adversary coverage, and revocation semantics remain. Test tampering and exact root round-trips before any service launch.
4. Pilot a moderated searchable directory plus expiring peer-rendezvous leases. Verify two independent users can publish, discover, and retrieve a permitted synthetic model while invalid, delisted, or over-rate-limit entries are not returned or announced.
5. Evaluate federation, DHT-only discovery, and public relays only after the pilot demonstrates abuse operations, privacy choices, and revocation limits.

## Decisions still open

- Which license policy is required for public listings: any declared license, only an explicit redistribution grant, or an allowlist? Recommendation: do not present ambiguous/unknown rights as “approved”; have counsel decide the minimum public-listing gate.
- Should the first directory be centrally operated and moderated, or should a private/allowlisted federation pilot come first?
- The local share-link scheme is `tswarm://v1/<sha256>`; what public resolution/revocation semantics are required? Who controls signer keys, and how are compromised or abandoned publishers handled?
- Are models trained for or intended to generate explicit adult content in scope? Recommend excluding them from the initial public directory until the owner adopts an explicit policy after legal review.
- What peer-address data does a tracker need, what is shown to users, and how long is it retained?

## Sources

[1] https://huggingface.co/docs/hub/model-cards
    > "The metadata you add to the model card supports discovery and easier use of your model."
    > "license: "any valid license identifier" datasets: - dataset1 - dataset2 base_model: "base model Hub identifier""
[2] https://huggingface.co/docs/hub/gguf
    > "Q8_K GH 8-bit quantization ( q ). Each block has 256 weights. Only used for quantizing intermediate results."
[3] https://huggingface.co/docs/hub/en/model-release-checklist
    > "pipeline_tag: text-generation # Specify the task library_name: transformers # Specify the library language: - en # List languages your model supports license: apache-2.0 # Specify a license datasets: - username/dataset # List datasets used for training base_model: username/base-model # If applicable (your model is a fine-tune, quantized, merged version of another model)"
[4] https://spdx.org/ids
    > "The SPDX License List includes a standardized short identifier, the full name, the license text, and a canonical permanent URL for each license and exception."
    > "SPDX-License-Identifier: Apache-2.0 OR MIT The licensee may choose to use the file under either the Apache-2.0 license or the MIT license."
[5] https://bittorrent.org/beps/bep_0009.html
    > "xt is the only mandatory parameter."
    > "If no tracker is specified, the client SHOULD use the DHT ( BEP 0005 ) to acquire peers."
    > "dn , tr and x.pe are all optional."
[6] https://bittorrent.org/beps/bep_0005.html
    > "The DHT is composed of nodes and stores the location of peers."
    > "A get_peers query has two arguments, "id" containing the node ID of the querying node, and "info_hash" containing the infohash of the torrent. If the queried node has peers for the infohash, they are returned"
[7] https://www.bittorrent.org/beps/bep_0003.html
    > "Tracker GET requests have the following keys: info_hash The 20 byte sha1 hash of the bencoded form of the info value from the metainfo file."
[8] https://huggingface.co/content-guidelines
    > "If you see Content that violates this Policy, report it. It will be directly addressed by the Hugging Face Team on a case-by-case basis."
    > "If you believe that any Content on our website infringes upon your intellectual property rights , you can submit a Takedown notice to dmca@huggingface.co ."
[9] https://huggingface.co/docs/hub/security-pickle
    > "This does not guarantee that your file is safe, but it does guarantee the origin of the file."
