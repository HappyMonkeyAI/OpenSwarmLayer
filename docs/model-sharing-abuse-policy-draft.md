# Public model-sharing abuse and rights policy draft

Status: DRAFT. The owner approved the product-policy direction in-session (non-model media excluded, affirmative redistribution basis, explicit-content models excluded by default); qualified legal review and final operational decisions remain pending. This is not legal advice, an approved final policy, or an implemented enforcement mechanism. Do not launch a public directory or tracker based on this draft alone.

## 1. Purpose and scope

TensorSwarm's managed public discovery service is for lawful exchange of model artifacts, not general-purpose file hosting. This draft applies to publisher listings and peer rendezvous operated by TensorSwarm. It does not claim that the service can recall files already downloaded or control independent peers sharing outside the service.

The initial public listing scope should be limited to model artifacts in the currently supported GGUF and Safetensors formats, with every file referenced by a validated content manifest. Any future tokenizer, configuration, dataset, or other sidecar-file support needs its own explicit allowlist and threat review; arbitrary attachments are out of scope.

Prohibit using the managed service to distribute movies, pornography, or other non-model media, as well as unlawful material and non-consensual sexual content. The initial public directory should also exclude models intended to generate explicit adult content unless the owner and qualified counsel explicitly approve a policy for that scope and target jurisdictions.

## 2. Publication eligibility

A publisher must provide, for every release:

- A model name, publisher-controlled description, supported task/architecture information where known, and artifact-variant/file list.
- Upstream source and revision where available, base-model lineage where applicable, and any known datasets/evaluation references.
- A license claim, preserving the publisher's original label and source URL; a normalized SPDX identifier/expression may supplement it.
- An affirmative attestation that the publisher has authority to distribute every listed artifact under the stated terms.

A license label, signature, supported file extension, parser success, or matching hash is not legal clearance. Under this draft, a release with unknown or materially ambiguous redistribution rights is not publicly listed. How the operator verifies publisher identity and rights, and what exceptions are acceptable, requires owner/legal approval.

`ParsedFromArtifact` and `PublisherDeclared` are different provenance labels, but remote labels remain claims until locally recomputed. Catalog review/moderation status must be written by the service operator, not accepted from a publisher card. Preserve the distinction between technical verification, provenance, rights review, and content-policy disposition in both API state and UI copy.

## 3. Discovery and enforcement boundary

For the first managed Internet pilot, use a moderated directory plus tracker-like rendezvous; do not enable global DHT announcements. The directory owns searchable metadata and publication disposition. The rendezvous returns expiring provider leases for an eligible descriptor/artifact root and stores no model bytes. These may share an operator initially but should have separate records and authorization rules.

A listing marked under review or delisted must disappear from directory search and must not receive new managed tracker leases. Keep a tombstone keyed to descriptor/root and publisher identity so ordinary re-announcement is rejected. A later signed share link, direct peer connection, third-party registry, or public DHT may still expose the identifier or cached bytes; do not promise universal recall, erasure, or prevention of off-service transfer.

## 4. Report, review, and appeal workflow

Provide a report/notice intake tied to a descriptor ID and artifact roots, with categories for non-model media, rights/IP, unlawful or non-consensual material, security concern, impersonation, and other policy violations. Request only the evidence and contact details needed to assess the report; restrict access to reporter information and preserve an audit trail for authorized case handling.

Proposed listing states:

- `Draft`: publisher-only; not searchable and not announced.
- `Published`: passed the defined publication gate; searchable and eligible for managed rendezvous.
- `UnderReview`: hidden from search and new managed leases while a report or security concern is assessed.
- `Delisted`: no new managed discovery; a tombstone remains to reject routine re-announcement.
- `Restored`: returned to `Published` after review/appeal, with the decision recorded.

The operator should acknowledge reports, triage urgent safety/legal issues, record the decision and rationale, notify affected parties where appropriate, and provide an appeal/counter-notice path. Applicable notice procedures, deadlines, evidence retention, disclosure, escalation, and who can authorize restoration require qualified legal review and an identified operational owner. Do not invent a response-time promise until staffing and coverage exist.

## 5. Minimum abuse and security controls

Before public pilot:

- Require an authenticated publisher identity or signing key for publication; set per-account and per-key publication/announcement quotas and rate limits. Define recovery and revocation for compromised keys. Identity or reputation is not a substitute for rights review.
- Bound descriptor bytes before deserialization, string lengths, collection counts, nesting, and report uploads. Reject unsupported schemas and malformed paths before writing durable state. Return a bounded failure, not partial publication.
- Treat titles, Markdown, URLs, tags, license labels, and source links as untrusted input. Render safely; never fetch arbitrary publisher URLs from a privileged server without a separately reviewed SSRF-safe design.
- Parse supported artifact formats without executing model files. Never load untrusted pickle or execute uploaded code. Security scanning, where used, is a distinct check with explicit `passed`, `failed`, or `unavailable` state; unavailable scanning must not silently become a clean result.
- Keep report review separate from automated format/integrity checks. Heuristics may flag items for review but cannot prove that a valid-looking model file is lawful or that a file is not general media encoded in model-shaped data.
- Record the moderation actor, decision, time, affected roots, and resulting listing/lease action. Redact secrets and restrict report evidence. Establish retention and deletion rules before collecting unnecessary personal data.
- Tell users that a managed tracker receives their network address to return peers. Minimize lease lifetime and retained connection logs; specify what is visible to other peers and what is retained by the operator.

## 6. Abuse threat and acceptance matrix

| Threat | Required control | Future acceptance probe | Residual limitation |
|---|---|---|---|
| Movie, pornography, or unrelated media offered as a model | Supported-format and manifest checks plus policy review/report path; no generic attachments | Unsupported media extension is rejected; a synthetic flagged listing is hidden after review | A file can contain semantically unrelated or harmful data while still parsing as a supported container |
| False license or rights claim | Publisher attestation, source/provenance fields, separate review disposition | Unknown/ambiguous license cannot appear as reviewed/approved; report moves listing to `UnderReview` | Automated checks cannot establish ownership or resolve every license dispute |
| Spoofed `ParsedFromArtifact` or publisher identity | Recompute supported facts locally; signed identity/revocation design; separate catalog-owned review | Altered card and signature fail; unverified provenance stays visibly unverified | A signature proves control of a key, not legal rights or safety |
| Spam/Sybil publishing or tracker announcements | Authentication, quotas, rate limits, bounded work, operational abuse response | Burst publishing/announce fixture is throttled; other publishers and downloads remain available | Account creation may be cheap; limits and human review need operations |
| Re-announcement after delisting | Durable tombstone and deny rule for descriptor/root and relevant signer | Delist, then retry publish/announce from the same and a new session; no listing or managed lease is returned | Repacked/modified bytes and independent DHTs can evade a single-root tombstone |
| Malformed or oversized descriptors and report bodies | Byte caps before parsing, schema/path validation, bounded error handling | Oversize, deeply nested, invalid-path, duplicate-ID, and unknown-version fixtures fail without state changes or unbounded resource use | Limits must be tuned and tested on supported hardware |
| Malicious weights or unsafe downstream loader behavior | Parse-only intake, no execution, security advisories and clear warnings | Malformed format fixtures fail closed; importer never executes a model during review | A parseable weight file may still trigger vulnerabilities in downstream inference software |
| IP exposure and excess logging | Short-lived leases, data minimization, documented retention and user notice | Inspect peer/rendezvous response and logs; confirm lease expiry and log deletion behavior | Peers necessarily learn addresses used for direct connections; network anonymity is not promised |
| Copies persist after takedown | Disable managed listing/leases and explain limits | Directory search and managed rendezvous stop after delist; UI states copies may remain | Already cached or independently seeded bytes cannot be recalled globally |

These are future service acceptance criteria, not evidence that any such controls currently exist. The release gate should include invalid-input, unavailable-scanner, report/appeal, delist/reannounce, privacy-readback, and two-independent-client tests before a public pilot.

## 7. Decisions and approvals required

- Owner approval of prohibited content scope, including the default exclusion of explicit-content models from the initial public directory.
- Qualified legal review of target jurisdictions, redistribution attestations, license policy, notices/counter-notices, unlawful-content escalation, privacy/retention, and appeal handling.
- Decision on required publisher identity/accountability, publisher key recovery, report response ownership, and operational coverage.
- Decision on the public listing gate for unknown licenses. Draft recommendation: do not publicly list releases without a documented, affirmative redistribution basis.
- Decision on whether the initial service is invite-only or publicly self-serve, and which public APIs are rate-limited or authenticated.
- Approval of tracker address visibility, lease/log retention, data deletion schedule, and transparency to users.

The owner-approved product-policy direction does not substitute for qualified legal review or final operational decisions. M7-02 remains open until those reviews are recorded. Local-only M7-03 descriptor/signature work may proceed, but M7-03/M7-04 must not enable public publishing before M7-02 closes.

## References

- Research and product proposal: [`model-discovery-publication-and-abuse.md`](model-discovery-publication-and-abuse.md)
- Current roadmap gates: [`../PLAN.md`](../PLAN.md) and [`../TASKS.md`](../TASKS.md)
