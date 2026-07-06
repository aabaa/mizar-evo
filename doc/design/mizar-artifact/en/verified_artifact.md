# mizar-artifact VerifiedArtifact Schema

> Canonical language: English. Japanese companion:
> [../ja/verified_artifact.md](../ja/verified_artifact.md).

`VerifiedArtifact` is the stable published projection for one verified source
file. It is the primary artifact read by downstream packages, LSP features,
documentation generation, and AI tooling when they need source-shaped verified
metadata without loading compiler-internal IR.

Task 10 introduced this specification, task 11 implemented the schema,
projection-input contract, writer, validating reader, and tests, and task 15
finalizes the provenance envelope plus store-level artifact-hash exclusions for
local verified-artifact metadata.

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
embedding VC IR, ATP problems, proof certificates, backend logs, or kernel
state.

Task 11 uses this canonical JSON field shape:

```text
obligation_metadata = {
  "obligation_id": string,
  "obligation_anchor": string | null,
  "owner_origin_id": string | null,
  "source_range": source_range,
  "obligation_kind": string,
  "statement_summary": string,
  "obligation_fingerprint": interface_hash_string,
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

`obligation_fingerprint` is the producer-owned composite proof-reuse input
fingerprint. It commits to the VC semantic fingerprint, local proof context
fingerprint, dependency-slice fingerprint, verifier-policy fingerprint, and the
kernel evidence handoff identity that affects whether an existing witness may be
reused. After the task-24 re-check, trusted formula/substitution witness
producers must derive this composite from the corrected kernel-evidence identity:
explicit proof-obligation goal polarity, the canonical formula-envelope handoff
hash, the task-28 non-imported-source `context_identity_hash()` when present, and
imported formula context/projection identities when present. `VerifiedArtifact`
validates the field's hash class and spelling and uses it for witness
consistency; the VC/proof producer remains responsible for constructing the
composite value until real producer integration lands.

Every obligation with `status = "accepted"` must set
`accepted_witness_obligation_id` to the same string as its own `obligation_id`.
That id must resolve to exactly one `ProofWitnessRef.obligation_id` in
`proof_witnesses`. Under task 23, an accepted trusted witness represents
formula/substitution/provenance/target-binding evidence that the kernel has
already accepted. Legacy backend certificates, resolution traces, built-in
certificate discharges, and kernel-primitive placeholders are not trusted
`ProofWitnessRef` values.

Task 11 validates the accepted-witness consistency tuple:

- the witness `obligation_id` equals the obligation `obligation_id`;
- the witness `obligation_fingerprint` equals the obligation
  `obligation_fingerprint`;
- the witness
  `kernel_acceptance.verifier_policy_fingerprint` equals the obligation
  `verifier_policy_fingerprint`;
- the obligation `vc_fingerprint`, `local_context_fingerprint`,
  `dependency_slice_fingerprint`, and `verifier_policy_fingerprint` are present
  as interface fingerprints in the obligation metadata and participate in the
  composite `obligation_fingerprint` contract and in `implementation_hash`
  participation;
- the witness `kernel_acceptance.target_binding_hash`,
  `formula_evidence_hash`, `substitution_evidence_hash`, `provenance_hash`, the
  optional `formula_context_hash`, and `accepted_result_hash` are interface
  hashes supplied by the producer/kernel boundary and included through the
  referenced `ProofWitnessRef`;
- task-24 goal-polarity and context-identity coverage is indirect: it is part of
  the producer-owned `obligation_fingerprint`, proof validation identity, and
  accepted-result metadata. `VerifiedArtifact` does not recompute the
  `mizar-vc` context-identity hash or replay kernel acceptance.

`VerifiedArtifact` verifies only reference consistency among projected fields.
It does not replay the witness, recompute kernel acceptance, or decide proof
authority. It also does not trust backend proof methods, backend logs, or
caller-supplied instantiated formulas or SAT problems.

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

Task 15 finalizes the crate-owned provenance envelope for published verified
artifacts:

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

`verifier_config_hash` records the stable producer-owned settings fingerprint
for verifier policy and configuration. The cache key is an opaque
producer-owned fingerprint. `mizar-artifact` records it only when the producer
supplies it; cache lookup, cache invalidation, and proof-reuse validation
remain owned by `mizar-cache`. `cache_key` is hash-excluded local/cache
metadata: it is parsed as a non-empty string or JSON `null`, but it does not
participate in `interface_hash`, `implementation_hash`, artifact publication
equivalence, or reproducibility comparisons.

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
| `obligation_metadata.obligation_fingerprint` | artifact-framed `interface` | producer-owned, valid grammar | Composite proof-reuse input fingerprint. |
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

Task 11 computes and validates both top-level hashes from filtered canonical JSON
projections. Task 16 exposes these projection builders as reusable helpers:
`interface_hash_input_json(&VerifiedArtifact)` and
`implementation_hash_input_json(&VerifiedArtifact)`. These helpers return the
canonical JSON values that are framed by `CanonicalHashDomain` when hashing;
they do not return the final artifact-framed preimage text and they do not
change the task-11 participation rules.

`interface_hash` is computed over exactly these importer-visible fields:

- `schema_version`;
- `module`;
- each export's `origin_id`, `fully_qualified_name`, `namespace_path`,
  `visibility`, `export_kind`, `rendered_signature`,
  `interface_fingerprint`, and `proof_status`;
- each dependency entry's `module` and `interface_hash`.

`interface_hash` excludes `source_file`, `source_hash`, `verified_at`, the
stored `interface_hash` and `implementation_hash`, export `source_range`,
export `documentation_ref`, expression metadata, obligations, proof witness
paths and byte hashes, diagnostics, `lockfile_hash`, `verifier_config_hash`,
dependency `implementation_hash`, dependency `artifact_hash`, `toolchain`,
`language_edition`, `cache_key`, and any future local/cache-only provenance
field.

`implementation_hash` is computed over exactly these stable published fields:

- `schema_version`, `module`, `source_file`, and `source_hash`;
- full `exports`, including `source_range` and `documentation_ref`;
- full `expressions`;
- full `obligations`, including `obligation_fingerprint` and its component
  fingerprints;
- full `proof_witnesses`;
- full `diagnostics`, including diagnostic ranges and related entries;
- stable provenance fields: `toolchain`, `language_edition`, `lockfile_hash`,
  `verifier_config_hash`, and the full `dependency_artifact_hashes` entries.

`implementation_hash` excludes only `verified_at`, the stored `interface_hash`
and `implementation_hash` fields themselves, `cache_key`, and any future
local/cache-only provenance field.

The interface hash input sorts `exports` by `origin_id`,
`fully_qualified_name`, `export_kind`, and `source_range`, and sorts
`dependency_artifact_hashes` by module identity while serializing only each
dependency `module` and `interface_hash`. The implementation hash input sorts
the full `exports`, `expressions`, `obligations`, `proof_witnesses`,
`diagnostics`, `diagnostic.related`, and `provenance.dependency_artifact_hashes`
collections by the canonical ordering keys below.

Both hashes use task-3 artifact-framed hash strings and domain-separated hash
classes. The manifest's artifact hash validates the store-level
publication-equivalent content: canonical JSON after declared hash exclusions,
not raw filesystem bytes.

## Canonical Ordering

Writers sort collections deterministically before serialization. Readers reject
unsorted collections and duplicate identity keys.

Nullable ranges sort with `null` before non-null ranges. Non-null ranges sort by
`start_byte` and then `end_byte`.

The initial ordering keys are:

- `exports`: `origin_id`, `fully_qualified_name`, `export_kind`,
  `source_range`;
- `expressions`: `expression_id`, `source_range`;
- `obligations`: `obligation_id`, `source_range`;
- `proof_witnesses`: the order specified by `proof_witness.md`;
- `diagnostics`: `diagnostic_id`, `code`, `primary_range`;
- `diagnostic.related`: `source_range`, `message_key`, `rendered_message`;
- `provenance.dependency_artifact_hashes`: module identity.

The initial duplicate identity keys are:

- `exports`: `origin_id`;
- `expressions`: `expression_id`;
- `obligations`: `obligation_id`;
- `proof_witnesses`: `obligation_id`, as specified by `proof_witness.md`;
- `diagnostics`: `diagnostic_id`;
- `diagnostic.related`: `source_range`, `message_key`, `rendered_message`;
- `provenance.dependency_artifact_hashes`: module identity.

Source traversal order, hash map iteration order, ATP completion order,
diagnostic emission race timing, and filesystem order must not affect serialized
bytes.

Readers reject duplicate identity keys at each listed collection boundary.

## Reader And Writer Requirements

Task 11 writers use the canonical UTF-8 JSON rules from [store.md](./store.md)
and emit the current schema version. Readers:

- require every field listed above, including JSON-null fields;
- reject unknown fields at every schema object;
- check schema-version compatibility before interpreting fields;
- reject empty required strings, malformed source ranges, invalid path shapes,
  malformed hash strings, wrong hash classes, duplicate identities, and unsorted
  collections;
- recompute `interface_hash` and `implementation_hash` from the filtered
  projections above and reject mismatches;
- validate that `accepted_witness_obligation_id` values resolve to matching
  `ProofWitnessRef.obligation_id` values when a trusted witness is required,
  that the accepted witness id equals the containing obligation id, and that the
  witness `obligation_fingerprint` equals the obligation
  `obligation_fingerprint`;
- require every accepted obligation to name exactly one matching trusted witness;
- reject trusted witness references for open, rejected, externally attested, or
  not-required obligations, and allow deterministic discharge hashes only for
  not-required obligations;
- keep `verified_at` and any future local-only fields hash-excluded;
- never read raw IR, internal cache records, ATP logs, witness payloads, or
  kernel state while validating the artifact schema;
- never treat backend proof methods, resolution traces, SMT proof objects,
  backend logs, or schema-version `1.0` certificate references as trusted
  acceptance material.

Reader failures are artifact diagnostics. They do not establish proof authority,
do not silently fall back to internal cache records, and do not upgrade
externally attested evidence into kernel-verified evidence.

## Public Enum Forward Compatibility

Task 19 applies the frontend task-25 public-enum procedure to verified-artifact
APIs. Every public enum owned by this module is a forward-compatible API surface
and must remain `#[non_exhaustive]`; downstream consumers must keep wildcard
fallback arms when matching them.

This is an API compatibility decision, not a reader leniency rule. Artifact
schema readers still reject unknown serialized enum values unless a later schema
revision and version policy explicitly document how to accept them.

| Enum | Forward-compatibility decision |
|---|---|
| `ExportVisibility` | Non-exhaustive so export visibility categories can grow. |
| `ExportProofStatus` | Non-exhaustive so importer-visible export proof states can grow. |
| `ObligationStatus` | Non-exhaustive so obligation status categories can grow without weakening current witness rules. |
| `DiagnosticSeverity` | Non-exhaustive so diagnostic severities can grow under documented artifact policy. |
| `VerifiedArtifactError` | Non-exhaustive so verified-artifact validation diagnostics can grow. |

This module has no exhaustive public enum exceptions.

## Deferred Implementation

Task 10 adds this specification only. Task 11 implements the
`VerifiedArtifact` schema, projection-input contract, writer, validating reader,
and tests for round-trips, raw-IR-shaped payload rejection, stable diagnostic
ordering, schema-version compatibility, source-range validation, hash
class/domain validation, witness-reference consistency, hash participation, and
ownership-boundary field rejection.

Task 15 finalizes provenance records and the store-level artifact-hash
exclusion helper for local verified-artifact fields. Task 16 exposes the
canonical interface and implementation hash input helpers. Real producer
integration remains an `external_dependency_gap` until resolver, checker, VC,
proof, and kernel crates expose stable projection inputs. Manifest/file I/O and
atomic publication were completed by tasks 13 and 14.
