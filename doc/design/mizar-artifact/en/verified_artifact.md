# mizar-artifact VerifiedArtifact Schema

> Canonical language: English. Japanese companion:
> [../ja/verified_artifact.md](../ja/verified_artifact.md).

`VerifiedArtifact` is the stable published projection for one verified source
file. It is the primary artifact read by downstream packages, LSP features,
documentation generation, and AI tooling when they need source-shaped verified
metadata without loading compiler-internal IR.

This document defines the task 10 specification only. Task 11 implements the
schema, projection-input contract, writer, validating reader, and tests.

## Ownership

The verified-artifact schema owns:

- the stable top-level artifact shape for a verified source file;
- source identity, source hash, and module identity used for stale-artifact
  diagnostics and manifest validation;
- dependency-facing exports and local expression metadata projected from earlier
  phases;
- verification obligation metadata and accepted proof status as projected data;
- references to external `ProofWitnessRef` values, never witness payload bytes;
- stable projected diagnostics for downstream tooling;
- compatibility and hash-participation rules for the artifact projection.

It does not own:

- raw AST arenas, typed ASTs, core IR, control-flow IR, VC IR, ATP problems, or
  kernel state;
- proof search, proof acceptance, kernel replay, or verifier policy decisions;
- proof witness payload byte schemas or witness file I/O;
- internal cache records, cache-key lookup, proof-reuse validation, or scheduler
  state;
- manifest transactions, atomic writes, or artifact-store file I/O.

The schema family is `mizar-artifact/verified-artifact`. Version `1.0` is the
initial version for task 11. Task 11 readers support the range `1.0..=1.0` and
reject missing, malformed, different-major, and newer-minor versions unless a
later spec explicitly declares forward-compatible minor-version handling.

## Top-Level Shape

Task 11 uses this canonical JSON field shape:

```text
verified_artifact = {
  "schema_version": "1.0",
  "module": module,
  "source_file": string,
  "source_hash": source_hash_string,
  "verified_at": string | null,
  "interface_hash": interface_hash_string,
  "implementation_hash": implementation_hash_string,
  "exports": [verified_export, ...],
  "expressions": [expression_metadata, ...],
  "obligations": [obligation_metadata, ...],
  "proof_witnesses": [proof_witness_ref, ...],
  "diagnostics": [artifact_diagnostic, ...],
  "provenance": build_provenance
}
```

The `module` field uses the same identity shape defined by
[module_summary.md](./module_summary.md). `source_file` is a normalized
package- or workspace-relative path. It must not be absolute and must not escape
the package root after normalization. Task 11 validates the lexical portable path
shape at the `CanonicalJson` boundary; filesystem escape checks remain store
I/O work.

`source_hash` records the exact source text used to produce the artifact. It is
used for stale-artifact diagnostics and manifest consistency checks. It is not
module identity.

`verified_at` is optional user-visible local metadata. When present, it is an
RFC 3339 UTC timestamp using `Z` and whole-second precision, for example
`2026-06-22T14:03:05Z`. Readers reject timezone offsets other than `Z`,
subsecond precision, malformed dates, and empty strings. `verified_at` is
excluded from `interface_hash`, `implementation_hash`, artifact publication
equivalence, and reproducibility comparisons. Readers must not use it to accept
a proof result, validate a dependency, or decide publication eligibility.

Every listed field is required. Fields whose absent value is meaningful are
encoded as JSON `null`; readers reject omitted required fields and unknown fields
at every schema object.

`source_range` uses the same byte-offset shape as `ModuleSummary`:

```text
source_range = {
  "start_byte": non_negative_integer,
  "end_byte": non_negative_integer
}
```

Readers reject ranges whose start is greater than their end.

## Exports

`exports` contains the externally visible declarations and signatures projected
from the verified source file. Each entry records enough stable metadata for
downstream packages and tools to identify, display, and depend on the export
without inspecting raw compiler IR.

Task 11 uses this canonical JSON field shape:

```text
verified_export = {
  "origin_id": string,
  "fully_qualified_name": string,
  "namespace_path": [string, ...],
  "visibility": "public" | "reexported",
  "export_kind": string,
  "source_range": source_range,
  "rendered_signature": string,
  "interface_fingerprint": interface_hash_string,
  "proof_status": "accepted" | "not_accepted" | "not_required" | null,
  "documentation_ref": diagnostic_hash_string | null
}
```

`exports` is broader than `ModuleSummary` only in local artifact detail. It must
not include implementation bodies, proof bodies, raw type-checker facts, raw
resolution tables, or kernel proof state. When an export is visible to importers,
its dependency-facing interpretation must agree with the corresponding
`ModuleSummary` projection.

`proof_status` is projected from proof-producing phases. `VerifiedArtifact`
records the status but does not decide whether a proof is accepted.

## Expression Metadata

`expressions` contains stable, source-shaped metadata for IDEs, documentation,
and AI tooling. It is intentionally a projection, not a serialized `TypedAst` or
`ResolvedTypedAst`.

Task 11 uses this canonical JSON field shape:

```text
expression_metadata = {
  "expression_id": string,
  "source_range": source_range,
  "expression_kind": string,
  "rendered_surface": string,
  "inferred_type": string | null,
  "resolved_symbol": string | null,
  "inserted_coercions": [string, ...],
  "active_thesis": string | null,
  "overload_resolution": overload_metadata | null
}

overload_metadata = {
  "root_symbol": string,
  "selected_candidate": string,
  "active_refinements": [string, ...],
  "coercion_summary": string | null
}
```

All type, symbol, thesis, coercion, and overload fields are stable rendered
summaries or producer-owned fingerprints. They must not expose arena indexes,
debug formatter output, raw type-table rows, raw AST nodes, or checker-local
object identities.

`expression_id` is deterministic for identical source, dependency graph,
toolchain, and verifier settings. It is not proof evidence and is not a
cross-edit proof-reuse identity.

## Obligations And Witness References

`obligations` records verification obligations projected from VC/proof phases.
It preserves source navigation, display, and publication status without
embedding VC IR, ATP problems, or proof certificates.

Task 11 uses this canonical JSON field shape:

```text
obligation_metadata = {
  "obligation_id": string,
  "obligation_anchor": string | null,
  "owner_origin_id": string | null,
  "source_range": source_range,
  "obligation_kind": string,
  "statement_summary": string,
  "vc_fingerprint": interface_hash_string,
  "local_context_fingerprint": interface_hash_string,
  "dependency_slice_fingerprint": interface_hash_string,
  "verifier_policy_fingerprint": interface_hash_string,
  "status": "accepted" | "open" | "rejected" | "externally_attested" | "not_required",
  "accepted_witness_obligation_id": string | null,
  "deterministic_discharge_hash": interface_hash_string | null,
  "diagnostic_ref": diagnostic_hash_string | null
}
```

`obligation_id` is the stable id used by `ProofWitnessRef`. `obligation_anchor`
is a best-effort cross-edit identity used for diagnostics, repair, and cache
candidate matching. It is not proof evidence, not sufficient for reuse, and not
trusted by the kernel.

Every obligation with `status = "accepted"` must set
`accepted_witness_obligation_id` to the same string as its own `obligation_id`.
That id must resolve to exactly one `ProofWitnessRef.obligation_id` in
`proof_witnesses`. This covers both ATP certificate witnesses and accepted
built-in or kernel-primitive discharges, because task 9 represents those
accepted evidence classes as `ProofWitnessRef` values.

Task 11 validates the accepted-witness consistency tuple:

- the witness `obligation_id` equals the obligation `obligation_id`;
- the witness `obligation_fingerprint` equals the obligation `vc_fingerprint`;
- the witness
  `kernel_acceptance.verifier_policy_fingerprint` equals the obligation
  `verifier_policy_fingerprint`;
- the obligation `local_context_fingerprint` and
  `dependency_slice_fingerprint` are present as interface fingerprints in the
  obligation metadata and are included in `implementation_hash` participation;
- the witness `kernel_acceptance.accepted_result_hash` is an interface hash
  supplied by the proof/kernel producer and is included through the referenced
  `ProofWitnessRef`.

`VerifiedArtifact` verifies only reference consistency among projected fields.
It does not replay the witness, recompute kernel acceptance, or decide proof
authority.

`status = "not_required"` is the only trusted no-witness case. It is used for
items that have no proof obligation under the active language and verifier
rules. Such entries must have `accepted_witness_obligation_id = null`; when a
producer supplies a deterministic discharge fingerprint, it is recorded in
`deterministic_discharge_hash` and uses class `interface`.

Open, rejected, and externally attested obligations must set
`accepted_witness_obligation_id = null` and `deterministic_discharge_hash =
null`. Externally attested evidence, if preserved for development workflows,
must use a separate diagnostic or development schema and does not satisfy
release policies that require kernel-checked witnesses.

The `proof_witnesses` collection contains task-9 `ProofWitnessRef` objects. It
uses the ordering and duplicate-obligation rules from
[proof_witness.md](./proof_witness.md): sort by obligation id, fingerprint,
status, evidence kind, path, and artifact hash, and reject duplicate
`obligation_id` values within one verified artifact unless a later schema
explicitly supports multiple accepted witnesses for a single obligation.

`VerifiedArtifact` never loads or inlines witness payloads. Consumers load
witness files only through artifact-store and manifest readers when replay,
audit, diagnosis, or hash validation is explicitly requested.

## Diagnostics

`diagnostics` contains stable projected diagnostics emitted during the completed
build pass. Diagnostics are artifact metadata, not proof authority.

Task 11 uses this canonical JSON field shape:

```text
artifact_diagnostic = {
  "diagnostic_id": string,
  "code": string,
  "severity": "error" | "warning" | "info" | "hint",
  "primary_range": source_range | null,
  "message_key": string,
  "rendered_message": string,
  "related": [diagnostic_related, ...],
  "explanation_ref": diagnostic_hash_string | null
}

diagnostic_related = {
  "source_range": source_range,
  "message_key": string,
  "rendered_message": string
}
```

`code`, `severity`, ordering keys, and `message_key` are stable. Rendered
messages may change when wording improves; consumers that need stable behavior
must key on diagnostic codes and structured fields. Diagnostics must not expose
raw debug dump field names, raw IR node indexes, or proof/kernel-internal state.

## Provenance Envelope

`provenance` records reproducibility and debugging metadata. It helps consumers
understand how the artifact was produced, but it is not a substitute for
validating dependency artifacts, source hashes, verifier policy fingerprints, or
proof witness hashes.

Task 10 specifies this envelope; task 15 may refine the producer-owned field
vocabulary for full provenance records:

```text
build_provenance = {
  "toolchain": string,
  "language_edition": string,
  "lockfile_hash": artifact_hash_string,
  "verifier_config_hash": interface_hash_string,
  "dependency_artifact_hashes": [dependency_artifact_hash, ...],
  "cache_key": string | null
}

dependency_artifact_hash = {
  "module": module,
  "interface_hash": interface_hash_string,
  "implementation_hash": implementation_hash_string | null,
  "artifact_hash": artifact_hash_string | null
}
```

The cache key is an opaque producer-owned fingerprint. `mizar-artifact` records
it only when the producer supplies it; cache lookup, cache invalidation, and
proof-reuse validation remain owned by `mizar-cache`. `cache_key` is
hash-excluded local/cache metadata: it is parsed as a non-empty string or JSON
`null`, but it does not participate in `interface_hash`, `implementation_hash`,
artifact publication equivalence, or reproducibility comparisons.

## Hash String Domains

`source_hash` uses the same exact source text hash string as `ModuleSummary`:

```text
source_hash_string =
  "mizar-session/hash-text/v1:" lower_hex_32_byte_digest
```

All other hash fields in this schema use the task-3 artifact-framed hash string
form:

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

`<digest>` is a 64-character lowercase hexadecimal digest. Artifact-framed
`schema_family` strings follow the grammar in `registration_summary.md`: they
are non-empty, colon-free, slash-separated identifiers whose segments use only
ASCII letters, ASCII digits, hyphen, underscore, or dot.

Task 11 validates the following domains:

| Field | Required construction/class | Required schema family/version | Notes |
|---|---|---|---|
| `source_hash` | `mizar-session/hash-text/v1` | none | Exact source text hash. |
| `interface_hash` | artifact-framed `interface` | `mizar-artifact/verified-artifact`, the artifact `schema_version` | Stored dependency-facing artifact hash; readers recompute and compare it. |
| `implementation_hash` | artifact-framed `implementation` | `mizar-artifact/verified-artifact`, the artifact `schema_version` | Stored full stable projection hash; readers recompute and compare it. |
| `verified_export.interface_fingerprint` | artifact-framed `interface` | producer-owned, valid grammar | Export fingerprint. |
| `verified_export.documentation_ref` | artifact-framed `diagnostic` | producer-owned, valid grammar | Optional documentation/diagnostic payload reference. |
| `obligation_metadata.vc_fingerprint` | artifact-framed `interface` | producer-owned, valid grammar | VC semantic fingerprint. |
| `obligation_metadata.local_context_fingerprint` | artifact-framed `interface` | producer-owned, valid grammar | Local proof context fingerprint. |
| `obligation_metadata.dependency_slice_fingerprint` | artifact-framed `interface` | producer-owned, valid grammar | Dependency slice fingerprint. |
| `obligation_metadata.verifier_policy_fingerprint` | artifact-framed `interface` | producer-owned, valid grammar | Verifier policy fingerprint. |
| `obligation_metadata.deterministic_discharge_hash` | artifact-framed `interface` | producer-owned, valid grammar | Optional no-witness discharge fingerprint for `not_required`. |
| `obligation_metadata.diagnostic_ref` | artifact-framed `diagnostic` | producer-owned, valid grammar | Optional diagnostic/explanation payload reference. |
| `artifact_diagnostic.explanation_ref` | artifact-framed `diagnostic` | producer-owned, valid grammar | Optional structured explanation payload reference. |
| `provenance.lockfile_hash` | artifact-framed `artifact` | producer-owned, valid grammar | Lockfile byte/projection hash. |
| `provenance.verifier_config_hash` | artifact-framed `interface` | producer-owned, valid grammar | Verifier configuration fingerprint. |
| `dependency_artifact_hash.interface_hash` | artifact-framed `interface` | dependency-owned, valid grammar | Dependency-facing module hash. |
| `dependency_artifact_hash.implementation_hash` | artifact-framed `implementation` | dependency-owned, valid grammar | Optional dependency implementation hash. |
| `dependency_artifact_hash.artifact_hash` | artifact-framed `artifact` | dependency-owned, valid grammar | Optional dependency artifact file hash. |

Top-level `interface_hash` and `implementation_hash` are the only hash fields
whose schema family and version are fixed by this schema. Producer-owned and
dependency-owned hash references preserve their own schema family and version so
that readers can validate class and spelling without rewriting another crate's
domain.

`ProofWitnessRef` values inside `proof_witnesses` keep the hash-domain rules
from [proof_witness.md](./proof_witness.md).

## Hash Participation

`interface_hash` is computed over the importer-visible projection:

- module identity fields that affect import interpretation;
- exported signatures, visibility, and dependency-facing fingerprints;
- accepted proof status for exported theorem/registration facts that importers
  can observe;
- dependency interface hashes and schema versions that affect importers.

It excludes expression metadata, local diagnostics, implementation bodies,
diagnostic/navigation `source_range` fields, source path metadata,
`source_hash`, `verified_at`, proof witness paths and byte hashes except where
an accepted exported status requires a dependency-facing fingerprint, local
provenance, `cache_key`, and `implementation_hash`.

`implementation_hash` is computed over the full stable published projection for
the source file, excluding hash-excluded local fields and the stored
`interface_hash` / `implementation_hash` fields themselves to avoid
self-reference. It includes expression metadata, obligations, diagnostics,
stable provenance inputs (`toolchain`, `language_edition`, `lockfile_hash`,
`verifier_config_hash`, and `dependency_artifact_hashes`), and
`ProofWitnessRef` values. It excludes `verified_at`, `cache_key`, and any future
local/cache-only provenance field.

Both hashes use task-3 artifact-framed hash strings and domain-separated hash
classes. The manifest's artifact hash validates the published file bytes.

## Canonical Ordering

Writers sort collections deterministically before serialization. Readers reject
unsorted collections and duplicate identity keys.

The initial ordering keys are:

- `exports`: `origin_id`, `fully_qualified_name`, `export_kind`,
  `source_range`;
- `expressions`: `expression_id`, `source_range`;
- `obligations`: `obligation_id`, `source_range`;
- `proof_witnesses`: the order specified by `proof_witness.md`;
- `diagnostics`: `diagnostic_id`, `code`, `primary_range`;
- `provenance.dependency_artifact_hashes`: module identity.

Source traversal order, hash map iteration order, ATP completion order,
diagnostic emission race timing, and filesystem order must not affect serialized
bytes.

## Reader And Writer Requirements

Task 11 writers use the canonical UTF-8 JSON rules from [store.md](./store.md)
and emit the current schema version. Readers:

- require every field listed above, including JSON-null fields;
- reject unknown fields at every schema object;
- check schema-version compatibility before interpreting fields;
- reject empty required strings, malformed source ranges, invalid path shapes,
  malformed hash strings, wrong hash classes, duplicate identities, and unsorted
  collections;
- validate that `accepted_witness_obligation_id` values resolve to matching
  `ProofWitnessRef.obligation_id` values when a trusted witness is required,
  and that the accepted witness id equals the containing obligation id;
- require every accepted obligation to name exactly one matching trusted witness;
- reject trusted witness references for open, rejected, externally attested, or
  not-required obligations, and allow deterministic discharge hashes only for
  not-required obligations;
- keep `verified_at` and any future local-only fields hash-excluded;
- never read raw IR, internal cache records, ATP logs, witness payloads, or
  kernel state while validating the artifact schema.

Reader failures are artifact diagnostics. They do not establish proof authority,
do not silently fall back to internal cache records, and do not upgrade
externally attested evidence into kernel-verified evidence.

## Deferred Implementation

Task 10 adds this specification only. Task 11 implements the
`VerifiedArtifact` schema, projection-input contract, writer, validating reader,
and tests for round-trips, raw-IR-shaped payload rejection, stable diagnostic
ordering, schema-version compatibility, source-range validation, hash
class/domain validation, witness-reference consistency, hash participation, and
ownership-boundary field rejection.

Real producer integration remains an `external_dependency_gap` until resolver,
checker, VC, proof, and kernel crates expose stable projection inputs. Full
provenance records remain deferred to task 15. Interface and implementation
hash input helpers remain deferred to task 16. Manifest/file I/O and atomic
publication remain deferred to tasks 13 and 14.
