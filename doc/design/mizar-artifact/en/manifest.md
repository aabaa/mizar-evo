# mizar-artifact Artifact Manifest Schema

> Canonical language: English. Japanese companion:
> [../ja/manifest.md](../ja/manifest.md).

## Purpose

The package artifact manifest is the publication index for `mizar-artifact`.
Downstream builds, LSP, documentation, extraction, and audit tools read the
manifest before reading any module artifact or proof witness file. They must not
discover published artifacts by scanning arbitrary files in the artifact root.

This document refines [store.md](./store.md),
[architecture 11](../../architecture/en/11.artifact_and_incremental_build.md),
and [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
for task 12. It specifies the manifest file schema and the transaction protocol
that later tasks implement.

## Scope

The manifest owns:

- package-level publication metadata;
- module artifact entries and their store-level artifact hashes;
- proof witness file reachability and producer-owned artifact hashes;
- optional development artifact reachability when those files are deliberately
  exposed through the artifact root;
- manifest transaction visibility, validation, and recovery rules.

The manifest does not own:

- the `VerifiedArtifact`, `ModuleSummary`, `RegistrationSummary`, or
  `ProofWitnessRef` object schemas;
- raw compiler IR dumps, scheduler state, internal cache records, or
  `mizar-cache` content-addressed blobs;
- proof authority, witness replay, kernel acceptance, or verifier policy
  decisions;
- the store file I/O implementation, which is owned by `store.md` and
  `src/store.rs`;
- scheduler ordering and cache promotion policy, which are represented only by
  caller-supplied transaction freshness checks and commit results.

The manifest is integrity and reachability metadata. A manifest entry can prove
that a referenced artifact has the publication-equivalent content named by its
store-level artifact hash, but it cannot turn externally attested evidence into
kernel-accepted proof evidence.

Manifest transactions may carry caller-supplied build snapshot freshness state
to reject obsolete writers. That state is transaction control data, not stable
artifact content, and `mizar-artifact` does not define scheduler ordering.

## File Name And Version

The standard package artifact manifest path is:

```text
artifact-manifest.json
```

The manifest path is relative to the package artifact root. Temporary manifest
paths are not published paths and must not be read by consumers.

The schema family is `mizar-artifact/manifest`. Version `1.0` is the initial
version. Task 14 readers support `1.0..=1.0` and reject missing, malformed,
different-major, and newer-minor versions unless a later spec explicitly
declares forward-compatible minor-version handling.

The manifest file's own artifact hash is computed by the store or manifest
manager over canonical manifest JSON using the task-3 artifact-framed hash
construction with class `artifact`, schema family `mizar-artifact/manifest`, and
the manifest `schema_version`. It follows the store hash-exclusion model, not a
raw filesystem-byte hash; schema version `1.0` declares no hash-excluded manifest
fields. The manifest does not store its own hash inside the JSON file.

## Top-Level Shape

Task 12 specifies this canonical JSON field shape:

```text
artifact_manifest = {
  "schema_version": "1.0",
  "package": package_identity,
  "artifact_root": string,
  "lockfile_hash": artifact_hash_string,
  "toolchain": string,
  "language_edition": string,
  "verifier_config_hash": interface_hash_string,
  "modules": [module_artifact_entry, ...],
  "development_artifacts": [development_artifact_entry, ...],
  "provenance": manifest_provenance
}

package_identity = {
  "package_id": string,
  "package_version": string | null,
  "lockfile_identity": string | null
}

manifest_provenance = {
  "generated_by": string,
  "manifest_policy": string,
  "transaction_format": string
}
```

Every listed field is required. Fields whose absent value is meaningful are
encoded as JSON `null`; readers reject omitted required fields and unknown fields
at every schema object.

`artifact_root` is a normalized package-relative path for the artifact root as
seen from the package root. It must not be absolute, must not contain empty,
`.` or `..` path segments, and must use `/` separators.

`toolchain`, `language_edition`, `generated_by`, `manifest_policy`, and
`transaction_format` are stable non-empty strings. They are manifest metadata,
not proof authority.

## Module Entries

`modules` contains one entry for each published module artifact reachable from
this manifest.

```text
module_artifact_entry = {
  "module": module,
  "source_file": string,
  "source_hash": source_hash_string,
  "artifact_file": string,
  "artifact_hash": artifact_hash_string,
  "interface_hash": interface_hash_string,
  "implementation_hash": implementation_hash_string,
  "module_summary_file": string | null,
  "module_summary_hash": artifact_hash_string | null,
  "module_summary_interface_hash": interface_hash_string | null,
  "registration_summary_file": string | null,
  "registration_summary_hash": artifact_hash_string | null,
  "registration_interface_hash": interface_hash_string | null,
  "proof_witnesses": [manifest_proof_witness_entry, ...],
  "diagnostics_hash": diagnostic_hash_string | null
}
```

`module` uses the same identity shape as `ModuleSummary` and `VerifiedArtifact`.
`source_file` follows the same portable path rules as `VerifiedArtifact`.
`artifact_file`, summary file paths, and proof witness paths are package artifact
root-relative paths. They must remain under the artifact root after
normalization and must not be absolute.

`artifact_file` points to the canonical main artifact selected by the declared enclosing profile below.
`artifact_hash` is the referenced file's store-level artifact hash: canonical
JSON after the referenced schema's declared hash exclusions, framed with class
`artifact`. It is not a raw filesystem-byte hash. `interface_hash` and
`implementation_hash` must equal the top-level hashes inside the referenced
main artifact envelope.

Summary file fields are optional because task staging may publish the verified
artifact before every sidecar summary is emitted. The three `module_summary_*`
fields are all `null` together or all non-null together. The
`registration_summary_*` fields follow the same rule.

`diagnostics_hash` is an optional diagnostic-payload hash for local diagnostic
indexes or explanation bundles. It is not proof authority and does not replace
the diagnostics embedded in `VerifiedArtifact`.

## Declared enclosing profiles

Manifest schema/current APIs remain 1.0: witness entries carry the existing id/fingerprint/path/hash tuple and no status discriminator. Referenced main artifacts select the known normal enclosing reader profile or explicit 2.0 reader profile through their declared artifact-hash schema version; unknown majors/minors remain rejected. Keep the actual store-framed hash, expected module/interface/implementation hashes, source/version checks and exact witness coverage, plus optional file reachability. Use the existing shared enclosing reader directly; do not add another transaction, token or acceptance path. Supported normal enclosing artifacts and all original manifest/transaction behavior remain intact; a structurally valid explicit reference is not source acceptance, complete Proof identity or committed Proof publication authorization.

Commented references select `mizar-artifact/commented-verified-artifact` version1.0 explicitly through `artifact_hash`; the interface/implementation references must use that same family/version and their existing classes. Other unknown families and unsupported commented versions are rejected without retry. The store uses the outer profile’s domain and [codec exclusions](./comment_projection.md#structural-codec-profile); reader expected hashes bind the outer envelope. Full module/source identity, witness coverage and sidecar checks use its validated normal1.2 inner artifact. The inner version and hashes are not compared with outer values. All existing transaction guards and atomic replacement apply; validation supplies no source authenticity, runtime currentness or new publication eligibility.

## Proof Witness Entries

Manifest proof witness entries make externally stored witness files reachable
from a module artifact entry.

```text
manifest_proof_witness_entry = {
  "obligation_id": string,
  "obligation_fingerprint": interface_hash_string,
  "witness_path": string,
  "witness_artifact_hash": artifact_hash_string
}
```

Each entry must match exactly one `ProofWitnessRef` in the referenced
`VerifiedArtifact`:

- `obligation_id` matches `ProofWitnessRef.obligation_id`;
- `obligation_fingerprint` matches `ProofWitnessRef.obligation_fingerprint`;
- `witness_path` matches `ProofWitnessRef.witness_path`;
- `witness_artifact_hash` matches `ProofWitnessRef.witness_artifact_hash`.

The `proof_witnesses` array in a module manifest entry must exactly cover the
referenced `VerifiedArtifact.proof_witnesses` collection. Readers reject missing
manifest witness entries, extra manifest witness entries, and entries whose
identity or hash tuple differs from the referenced artifact. This guarantees
that every accepted obligation requiring a trusted witness remains reachable
through the manifest.

The producer-owned witness writer records `witness_artifact_hash` values and
makes witness files reachable before the manifest transaction commits the
manifest. A committed manifest entry that names an accepted witness must make
the witness file reachable by `witness_path`, and readers reject missing witness
files or witness tuple mismatches when witness validation is requested. Task 14
does not recompute witness payload hashes; the concrete witness payload hash
construction remains producer-owned as described by
[proof_witness.md](./proof_witness.md).

## Development Artifact Entries

Development artifacts are optional files for diagnostics, missing-facts
workflows, ATP logs, explanation previews, and debug/audit tooling.

```text
development_artifact_entry = {
  "kind": string,
  "path": string,
  "artifact_hash": artifact_hash_string | null,
  "diagnostic_hash": diagnostic_hash_string | null,
  "related_module": module | null
}
```

Development entries are published only when deliberately listed in the manifest.
They must not contain raw compiler IR unless a later development-artifact schema
explicitly marks the file as local/debug-only and excludes it from semantic
artifact hashes. Development artifacts never satisfy proof witness requirements.

Exactly one of `artifact_hash` or `diagnostic_hash` must be non-null unless a
later schema defines a multi-hash development file. Entries with both hash
fields null, or with both fields non-null, are invalid. `related_module` is
optional navigation metadata.

## Hash String Domains

`source_hash` uses the source text hash string from `ModuleSummary` and
`VerifiedArtifact`:

```text
mizar-session/hash-text/v1:<digest>
```

All other hash fields use the task-3 artifact-framed form:

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

Task 14 validates at least these domains:

| Field | Required class | Required schema family/version | Notes |
|---|---|---|---|
| manifest file hash | `artifact` | `mizar-artifact/manifest`, manifest `schema_version` | Computed externally from canonical manifest JSON; not stored in the file. |
| `lockfile_hash` | `artifact` | producer-owned, valid grammar | Lockfile or lock projection hash. |
| `verifier_config_hash` | `interface` | producer-owned, valid grammar | Verifier configuration fingerprint. |
| `module_artifact_entry.source_hash` | `mizar-session/hash-text/v1` | none | Exact source text hash. |
| `module_artifact_entry.artifact_hash` | `artifact` | selected main family/version above | Published main-envelope store-level artifact hash. |
| `module_artifact_entry.interface_hash` | `interface` | selected main family/version above | Must match referenced envelope. |
| `module_artifact_entry.implementation_hash` | `implementation` | selected main family/version above | Must match referenced envelope. |
| `module_summary_hash` | `artifact` | `mizar-artifact/module-summary`, referenced summary schema version | Optional `ModuleSummary` store-level artifact hash. |
| `module_summary_interface_hash` | `interface` | `mizar-artifact/module-summary`, referenced summary schema version | Optional summary interface hash. |
| `registration_summary_hash` | `artifact` | `mizar-artifact/registration-summary`, referenced summary schema version | Optional `RegistrationSummary` store-level artifact hash. |
| `registration_interface_hash` | `interface` | `mizar-artifact/registration-summary`, referenced summary schema version | Optional registration interface hash. |
| `manifest_proof_witness_entry.obligation_fingerprint` | `interface` | producer-owned, valid grammar | Must match the `ProofWitnessRef`. |
| `manifest_proof_witness_entry.witness_artifact_hash` | `artifact` | producer-owned, valid grammar | Must match the witness payload hash recorded by `ProofWitnessRef`. |
| `diagnostics_hash` | `diagnostic` | producer-owned, valid grammar | Optional diagnostics/explanation hash. |
| `development_artifact_entry.artifact_hash` | `artifact` | producer-owned, valid grammar | Optional development artifact hash. |
| `development_artifact_entry.diagnostic_hash` | `diagnostic` | producer-owned, valid grammar | Optional diagnostic payload hash. |

When a field references another mizar-artifact schema, readers validate both the
class and the schema family/version recorded by the referenced file. Producer
owned hashes keep their own family and version but must use valid artifact-framed
spelling and the required class.

## Canonical Ordering

Writers sort collections before serialization. Readers reject unsorted
collections and duplicate identity keys.

Ordering keys:

- `modules`: module identity;
- `module_artifact_entry.proof_witnesses`: `obligation_id`,
  `obligation_fingerprint`, `witness_path`;
- `development_artifacts`: `kind`, `path`, `related_module`;

For nullable `related_module`, `null` sorts before any module identity; non-null
values sort by the same module identity ordering used by `modules`.

Duplicate identity keys:

- `modules`: module identity;
- `module_artifact_entry.proof_witnesses`: `obligation_id`;
- `development_artifacts`: `kind`, `path`.

Source traversal order, worker completion order, filesystem order, and manifest
transaction staging order must not affect canonical manifest bytes.

## Reader Requirements

Manifest readers:

- read only `artifact-manifest.json`, not temporary manifest paths;
- check schema-version compatibility before interpreting fields;
- reject unknown fields, missing required fields, malformed paths, malformed hash
  strings, wrong hash classes, duplicate identities, and unsorted collections;
- reject paths that leave the artifact root after normalization;
- load referenced module artifacts only through manifest entries;
- validate referenced artifact hashes when requested by the consuming command,
  publication policy, or transaction commit path;
- when validating a module entry, require the referenced `VerifiedArtifact`
  module identity, source hash, interface hash, implementation hash, and full
  proof witness reference set to agree exactly with the manifest entry;
- never fall back to internal cache records to repair or trust a bad manifest.

Reader failures are artifact diagnostics. A missing or corrupt referenced file is
not proof evidence and is not silently replaced by a cache hit.

## Manifest Transaction Protocol

The manifest manager is serialized per package. Parallel module artifact writes
produce independent module updates; the manifest manager folds those updates
into one package-level publication transaction.

### Begin

Beginning a transaction records:

- the package identity and artifact root;
- the current manifest hash, or `null` when no manifest exists;
- the caller-supplied build snapshot freshness guard for obsolete-writer
  rejection;
- local session/snapshot/task identifiers used for diagnostics and temporary
  file names.

Local session and snapshot identifiers, including the freshness guard, are
transaction state only. They are not serialized into the published manifest and
do not participate in manifest canonical bytes. The freshness guard is opaque to
the manifest schema; it is supplied by the build coordinator that owns snapshot
ordering.

### Stage

Before commit, every staged module update must have already written, flushed,
and hash-validated its referenced files. Staged updates record final paths and
hashes only. Temporary paths must not appear in the candidate manifest.

The manager rejects a staged update if:

- a path is outside the artifact root;
- a required referenced file is missing;
- a recorded hash does not match a referenced canonical artifact or referenced
  artifact field;
- a producer-owned payload path is unreachable when task-14 reachability
  validation applies. Payload hash recomputation waits for the producer-owned
  payload schema;
- two staged module updates have the same module identity but different
  canonical content or hashes;
- an update attempts to publish raw IR, scheduler state, internal cache records,
  or proof authority state as a manifest entry.

### Commit

Commit performs these steps:

1. Re-read the current manifest and compare its hash with the transaction's
   `base_manifest_hash`.
2. Check the transaction's snapshot freshness guard against the currently active
   package snapshot.
3. Abort if the transaction is obsolete. Abort or rebase if the base hash
   changed; a rebase must refresh staged content against the active snapshot and
   repeat the freshness check.
4. Merge unchanged current entries with staged updates, then sort all entries by
   canonical keys.
5. Serialize the candidate manifest using canonical JSON.
6. Validate every entry path and referenced canonical artifact hash from disk;
   validate producer-owned payload reachability and defer payload hash
   recomputation to the producer-owned schema.
7. Write candidate bytes to a temporary manifest path in the artifact root.
8. Flush the temporary manifest file.
9. Atomically rename the temporary manifest over `artifact-manifest.json`.
10. Flush the containing directory when the platform supports it.
11. Publish build events only after the final manifest path names the new
    entries.

Commit success is reached only after the required rename and supported directory
flush complete. If any step before the atomic rename fails, the previous
manifest remains authoritative. If the atomic rename succeeds but the containing
directory flush fails on a platform that supports directory flushing, the commit
returns an artifact I/O diagnostic, build events are not published, cache
promotion is not allowed, and callers treat the previous manifest hash as the
last durable authority. Recovery is still manifest-first: after a crash or
reopen, consumers read and validate only the final `artifact-manifest.json` that
survived, never temporary or orphaned files. If rename and required flush
succeed, the new manifest is authoritative even if cleanup of temporary or
orphaned files fails.

Successful manifest commit is the earliest point at which cache records or cache
indexes that depend on the new publication may be promoted. `mizar-artifact`
reports manifest commit success or failure; `mizar-cache` owns cache promotion
and discard policy.

### Retained commit result
`ManifestCommit` retains exactly the manifest and store write returned only after the existing `ManifestTransaction::commit` succeeds. Its existing fields are private; the public type/path and Debug/Clone/PartialEq/Eq remain. No caller constructor, mutable access or new token/transaction is added.
Existing-value access is `manifest(&self) -> &ArtifactManifest`, `write(&self) -> &PublishedArtifactWrite` and `into_manifest(self) -> ArtifactManifest`. References are read-only; owned extraction moves the manifest and discards the write. Build first reads the actual write hash, then consumes the manifest without cloning; its public summary remains plain metadata.
This records past transaction-origin values, not current file integrity, registry/native/snapshot binding, producer input authenticity, witness payload hash validation, proof acceptance, committed Proof publication or cache promotion. Existing validation/error/write/flush order, serialized schemas/profiles, caller freshness checks and all earlier guarantees remain unchanged.
#### Retained commit result tests
Append only `commit_result_retains_actual_transaction_values` to existing manifest/tests.rs. Run a real transaction with the existing empty structural seed (no proof/module producer), read its actual written manifest/hash and compare both complete accessor values plus owned extraction. Overwrite the temporary manifest once and show the retained result stays unchanged while a fresh read rejects corruption; no currentness credit follows.
Three external compile-fail examples in the existing ManifestCommit Rust doccomment reject caller field-literal construction and direct replacement of each field. They use typed function parameters, not fabricated successful records. Keep all existing tests/helpers/fixtures byte-identical; no new helper/type/adapter/dependency or lower-validator matrix.

## Recovery From Interrupted Commits

Recovery is manifest-first:

- an interrupted transaction before manifest rename leaves the previous manifest
  authoritative;
- a supported directory-flush failure after manifest rename is a failed commit
  for producer, event, and cache-promotion semantics; recovery still reads only
  the surviving final manifest path and validates it;
- completed artifact or witness files that are not reachable from the committed
  manifest are ignored by readers and may be garbage-collected later;
- temporary manifest files are never publication indexes and may be removed when
  no active transaction owns them;
- if the final manifest file is missing, unreadable, or has an unsupported
  schema, consumers report an artifact diagnostic instead of scanning the
  artifact root;
- if the final manifest references missing or hash-mismatched files, consumers
  report artifact integrity diagnostics for those entries;
- cache records or cache indexes staged for a failed manifest commit remain
  staged or are discarded by `mizar-cache`; they must not become proof authority.

This recovery rule means a package is observed through exactly one complete
manifest version at a time: the previous committed manifest or the new committed
manifest.

## Public Enum Forward Compatibility

Task 19 applies the frontend task-25 public-enum procedure to manifest APIs.
Every public enum owned by this module is a forward-compatible API surface and
must remain `#[non_exhaustive]`; downstream consumers must keep wildcard
fallback arms when matching them.

This is an API compatibility decision, not a reader leniency rule. Manifest
readers still reject unknown serialized enum values unless a later schema
revision and version policy explicitly document how to accept them.

| Enum | Forward-compatibility decision |
|---|---|
| `ManifestError` | Non-exhaustive so manifest schema, transaction, reference-validation, and store-wrapping diagnostics can grow. |

This module has no exhaustive public enum exceptions.

## Implementation Status

Task 12 added this specification. Task 13 implemented artifact-store writes and
corruption-detecting file reads. Task 14 implements the manifest schema,
validating reader, writer, and transaction manager in `src/manifest.rs`,
including manifest-first reads, deterministic transaction commits, referenced
`VerifiedArtifact` and summary validation, exact `ProofWitnessRef` coverage, and
development-artifact reachability checks. Concrete witness payload hash
construction and development payload hash recomputation remain
producer-owned external dependencies. Task 17 connects real producer projections
to store and manifest publication. Broader determinism coverage remains task 18
work.

## Imported Promotion Authority Boundary

Future imported promotion publication/loading transports authenticated Proof-produced evidence and exact algorithm/contract/equation/group/dependency identity; Proof remains the acceptance validator. Runtime export validation, manifest flags and hashes alone never enable promoted premises. Missing or stale proof provenance blocks the VC consumer without re-proving the exporting module.

| Existing transport producer/validator | Available binding; absent authority |
|---|---|
| [ProofWitnessRef, KernelAcceptanceMetadata and read_proof_witness_ref](../../../../crates/mizar-artifact/src/proof_witness.rs) | Obligation/fingerprint, status/evidence kind, payload hash, target/formula/substitution/provenance/result and policy/profile/schema metadata; validates shape, versions, hash domains and paths, not exporting Proof authenticity or aggregate promotion acceptance. |
| [VerifiedArtifact, VerifiedExport, BuildProvenance and DependencyArtifactHash](../../../../crates/mizar-artifact/src/verified_artifact.rs) | Source/module/export signature, interface/implementation identities, projected obligation status, witnesses, toolchain/lock/config/dependency hashes. Structural validation supplies no exact promotion-equation/group-to-proof binding. |
| [ModuleSummaryIdentity and ExportedSymbolSummary](../../../../crates/mizar-artifact/src/module_summary.rs), [project_summary](../../../../crates/mizar-resolve/src/module_summary_reuse.rs) | Package/version/lockfile and dependency interfaces, opaque exported signature/label identities and status projections. Resolver lowering creates imported shells, not authenticated premises. |
| [validate_module_references and validate_witness_coverage](../../../../crates/mizar-artifact/src/manifest.rs) | Main artifact hash/module/interface/implementation/source agreement and exact witness-reference coverage; optional witness-file validation checks reachability only. No witness payload recomputation, kernel replay or producer authentication occurs there. |

The portable producer and consumer must agree on resolved algorithm/export identity and exact signature/contracts; ordinary versus definitional promotion and eligible equations; applicable E1/E2/E4 obligations with their complete inputs/guards; every mutual-group member and its acceptance; exact source/library, package/dependency/version/lockfile provenance and permitted proof-policy/kernel/evidence versions. Existing fields above are binding inputs, not an authenticated complete tuple or a frozen wire schema. Semantic interface compatibility does not authenticate proof origin or permit stale evidence reuse.
Artifact owns publication/loading and preservation of these bindings from genuine Proof outputs; Proof owns acceptance and compatible-policy validation before VC59 receives any premise. No current producer or reader supplies that portable route. The [Proof trust-transfer rule](../../mizar-proof/en/policy.md#trust-transfer-rule) resolves the producer-authentication definition gap and owns acceptance; its boundary retains the six validation rows and ordered slices. Transport implementation still waits for genuine local aggregate producers; Proof import validation and VC59 connection remain separate later gates.
Existing [witness tests](../../../../crates/mizar-artifact/src/proof_witness/tests.rs) and [summary tests](../../../../crates/mizar-artifact/src/module_summary/tests.rs) check wire/hash/version/status/identity constraints, not accepted portable promotion. Logical contract-only import requires no execution artifact; computation artifact validation remains a separate execution-owner gate.

### Producer transport design

This is a future transport design under the approved Proof rule, not a change to the current manifest/witness schemas or executable acceptance. Use one signed publication descriptor per complete promotion group, including a singleton group. A future versioned manifest extension references its descriptor and detached signature alongside the existing module artifacts/summaries; existing readers cannot interpret that extension as legacy accepted metadata.

**Representation and authentication.** Use canonical JSON under the [store canonicalization rules](./store.md), with schema-defined ordering for member/prerequisite sets and preserved order inside logical terms, guards and equations. Reject duplicate identities/keys, unknown fields/versions, noncanonical bytes and unsupported suites. Choose Ed25519 signatures and SHA-256 content commitments for this transport. Sign an unambiguously framed domain, transport version, algorithm suite and complete canonical descriptor bytes; the signature is outside those bytes. Importer-controlled configuration maps the signing key to the independently authorized Proof execution authority and its permitted implementation/version/policy scope. A key identifier or bundled public key is only a lookup hint; it cannot establish trust or extend scope. This selects the mechanism; exact field names/encoding and API remain for implementation, without changing these bindings.

**Exact binding.** The descriptor commits the full tuple required by the [Proof boundary](../../mizar-proof/en/policy.md#imported-promotion-authority-boundary). Logical content may be inline or referenced by SHA-256 of exact canonical bytes; every reference needed to reconstruct that tuple is mandatory and covered transitively. The descriptor also binds SHA-256 of the exact published module artifact and logical summary bytes used for import. Hash complete referenced bytes, including provenance fields excluded from existing artifact identity projections; existing interface/hash/status labels alone cannot substitute. Private source/proof bodies need only authenticated content identities, not importer loading or replay. No runtime artifact is required for a logical-only export.

**Accepted-result-only publication.** Proof owns construction of the genuine, untainted complete-group result. An authority-controlled publisher derives the descriptor from that result and the exact immutable serialized objects; it accepts neither a caller-supplied accepted flag nor arbitrary bytes for signing. The signing key is restricted to that execution/publication path, not a general signing endpoint. Artifact does not manufacture acceptance from witnesses, summaries, cache entries or transaction success.

First commit the complete unsigned descriptor and bound objects through the existing transaction/freshness/flush protocol, without granting promotion authority. Only after a successful retained commit and verification that it contains that exact result-bound descriptor/object closure may the authority sign the descriptor. The signature attests this committed acceptance publication. Then publish the signature and promotion references through a second transaction under the same freshness checks. This second transaction provides local discoverability, not authenticated evidence of a second exporter commit. The descriptor binds the committed objects, not the final manifest containing its signature, avoiding a hash cycle. A rebase or changed snapshot must revalidate the accepted-result/object/dependency binding before signing or retrying publication; it cannot carry acceptance onto changed inputs. All group promotion references are installed together by the second manifest rename; commit success still requires the existing flush steps. Failure of the first commit forbids signing, and an unsigned descriptor grants no premise. Failure before the second rename exposes no new local promotion entry; a post-rename flush failure follows the existing failed-commit/final-manifest recovery rules and supplies no producer success claim. Recovery follows the existing final-manifest-only rules and never reconstructs authority by scanning temporary/orphan files; signing retries require the same genuine result and successful descriptor/object commit. Redistribution of an authentic already-committed signed descriptor through another valid manifest is permitted, but cannot alter its bound objects or enlarge its authority; an unsigned manifest alone never proves exporter commitment.

**Loading and handoff.** Promotion loading always validates the final manifest, every descriptor/signature/object reference and all required content commitments; the current optional reference-validation defaults are insufficient. Parse and hash the same immutable bytes retained for the Proof handoff, so a path cannot be replaced between checking and use. Compare the manifest's selected module/summary objects with the descriptor commitments, enforce root/path constraints and reject mixed publications, missing members/references, corruption or unsupported formats. This structural success produces only untrusted publication data plus the exact signed bytes/signature and bound objects. Proof's later import validator owns signature/authority verification under current importer trust, exact resolved-object/dependency comparison, policy/schema compatibility and retained imported provenance before exposing any member premise. Artifact reader success never becomes a fresh kernel result. Logical imports neither load runtime artifacts nor re-prove opaque modules; computation retains its separate gates.

Artifact reports structural/content/transaction failures; Proof owns authentication, authorization, semantic binding and policy refusal. Either failure withholds the entire group's promoted premises, with no status/hash/cache fallback. Legacy or absent descriptors can still supply existing opaque signatures, but grant no promotion authority. The design introduces no public diagnostic code. Genuine local aggregate E1/E2/E4/group producers, this transport's implementation, Proof import validation and VC59 integration remain absent; no executable acceptance or coverage credit follows from this design.
