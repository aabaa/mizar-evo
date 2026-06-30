# mizar-ir Artifact Projection

> Canonical language: English. Japanese companion:
> [../ja/projection.md](../ja/projection.md).

## Purpose

This document specifies the artifact-projection boundary for `mizar-ir`.

`ArtifactProjectionService` reads sealed `PhaseOutputRef<T>` values and
producer-supplied stable projection inputs, then constructs unpublished
`VerifiedArtifactDraft` values backed by the `mizar-artifact`
`VerifiedArtifact` schema. The draft is a publication candidate only. It is not
written to the artifact store, not committed to a manifest transaction, and not
proof authority.

The projection boundary exists so downstream consumers see stable external
schemas instead of compiler-internal IR storage. It may map internal ids to
fully qualified names, labels, source ranges, normalized signatures,
obligation metadata, proof-witness references, diagnostics, explanation
references, documentation references, and LSP metadata. It must not expose raw
IR values or storage handles.

Real producer projection payloads, `mizar-diagnostics` integration, artifact
publication tokens, and phase-15 manifest transactions remain
`external_dependency_gap` items until their owning crates expose real seams.
`mizar-ir` must not add placeholder producer-token, diagnostics-token,
driver-session, or publication-token APIs, and it must not depend on
`mizar-driver`.

## Ownership

`mizar-ir` owns:

- validation that every projected source item comes from sealed, readable
  outputs in the intended current snapshot;
- conversion from crate-local projection input records to the stable
  `mizar-artifact` `VerifiedArtifact` schema shape. Until real producer
  crates expose richer typed records, those crate-local records may directly
  wrap strongly typed `mizar-artifact` schema structs;
- deterministic ordering of projected exports, expression metadata,
  obligations, witness references, diagnostics, and dependency provenance;
- rejection of raw internal IR leakage before a draft is returned;
- non-authoritative projection errors that block the current draft without
  mutating sealed IR outputs.

`mizar-ir` does not own:

- `mizar-artifact` schema versions, canonical JSON serialization, manifest
  publication, artifact-store writes, or artifact hash framing;
- proof acceptance, trusted status, verifier-policy selection, deterministic
  proof winner selection, kernel acceptance, or witness payload validation;
- `mizar-cache` `CacheKey`, dependency fingerprint, dependency slice, or
  proof-reuse validation;
- real diagnostic rendering/registry ownership;
- source parsing, name resolution, type checking, VC generation, ATP
  translation, or kernel-internal state.

## Draft Model

`VerifiedArtifactDraft` is an unpublished in-memory candidate owned by
`mizar-ir` task 12. It contains a `mizar-artifact` `VerifiedArtifact` value and
projection-local provenance sufficient to explain how the draft was produced.

The draft is current only for the target `BuildSnapshotId` used during
projection. An obsolete snapshot draft must not be returned as a current
artifact candidate. A retained stale output may be read for diagnostics,
explanation requests, or cache validation, but using it for a current
projection requires an explicit current-snapshot validation path. Cache hits
and rehydrated handles do not upgrade proof trust or publication authority.

Phase 15, owned outside `mizar-ir`, is responsible for serializing the draft,
validating manifest publication preconditions, and committing or rejecting the
artifact. Projection failure leaves sealed IR outputs unchanged and leaves the
previous published manifest authoritative.

## Inputs

A projection request requires:

- the target current `BuildSnapshotId`;
- package/module identity, normalized source file path, source hash, language
  edition, toolchain id, lockfile hash reference, verifier configuration hash
  reference, and dependency artifact hash references supplied by the build
  owner;
- sealed phase-output handles for the producer outputs used to justify the
  projection;
- stable producer projection records for exports, expression metadata,
  obligations, proof-witness references, diagnostics, and optional
  documentation/explanation references;
- explicit currentness information from the publisher/snapshot replacement
  boundary.

The projection request must not contain task-local mutable builders, raw AST
arena nodes, raw type-table rows, raw `CoreIr`, raw `VcIr`, raw `AtpProblem`,
kernel-internal proof state, storage slot ids, storage generations, memory
addresses, worker ids, temporary paths, or manifest publication tokens.

Until real producer crates expose typed projection records, task 12 may define
crate-local projection input records whose fields are already stable external
strings, source ranges, and `mizar-artifact` hash references. These records are
not placeholder downstream APIs; they are the local validation boundary for
tests and future integration.

Crate-local projection input records must be strongly typed to the
`mizar-artifact` fields listed in this document. Task 12 may use the
`mizar-artifact` schema structs themselves as that strongly typed boundary.
They must not include
arbitrary extension maps, raw JSON blobs, byte payloads, or passthrough
producer objects. This structural rule prevents raw IR from being smuggled
through an "extension" escape hatch; tests may still use raw-looking sentinel
strings to verify the leakage guard fails before a draft is returned.

## Projected Data

### Exports

Exports are projected into `VerifiedExport` records. Each export includes a
stable origin id, fully qualified name, namespace path, visibility, producer
owned export kind, source range, rendered importer-visible signature,
interface fingerprint, optional projected proof status, and optional
documentation reference.

Signatures must be normalized or rendered by the producer boundary before they
reach `mizar-ir`. Projection may sort and validate them, but it must not
re-run type checking or decide overload semantics.

### Expression Metadata

Expression metadata is stable, source-shaped metadata for IDE,
documentation, and AI tooling. It may contain rendered surface text, rendered
inferred type, resolved symbol summary, inserted coercion summaries, active
thesis summaries, and overload-resolution summaries.

These fields are projections. They must not include serialized `TypedAst` or
`ResolvedTypedAst` nodes, arena indexes, debug formatter dumps, checker-local
object identities, or raw type-fact tables.

### Obligations And Witnesses

Obligations are projected into `ObligationMetadata` records. They include
obligation id, optional cross-edit anchor, owner export origin id, source
range, producer-owned obligation kind, rendered statement summary, obligation
fingerprint, VC fingerprint, local context fingerprint, dependency-slice
fingerprint, verifier-policy fingerprint, projected status, optional accepted
witness obligation id, optional deterministic no-witness discharge hash, and
optional diagnostic reference.

`mizar-ir` records proof status as supplied projected data. It must not decide
that a proof is accepted, trusted, kernel accepted, or policy compatible. An
accepted obligation may be included only when the supplied `ProofWitnessRef`
and obligation metadata satisfy the `mizar-artifact` schema consistency rules.
`mizar-ir` may validate schema consistency; it must not validate witness
payload bytes or replay kernel evidence.

All obligation, VC, local-context, dependency-slice, and verifier-policy
fingerprints are opaque hash references supplied by the producer/cache/proof
owners. `mizar-ir` may check that they have the `mizar-artifact` schema shape
and are internally consistent with witness references, but it must not
construct, recompute, reinterpret, or policy-validate those fingerprints.

### Diagnostics And Explanation References

Diagnostics are projected into `ArtifactDiagnostic` records with stable ids,
codes, severities, source ranges, message keys, rendered messages, related
locations, and optional explanation references.

`mizar-diagnostics` is not present in this checkout. Therefore task 12 must use
crate-local stable diagnostic projection records and classify real diagnostic
registry/renderer integration as `external_dependency_gap`. Projection must not
invent diagnostic publication tokens or treat rendered diagnostics as proof
authority.

### Provenance

Build provenance records the toolchain, language edition, lockfile hash,
verifier configuration hash, dependency artifact hashes, and optional
non-authoritative cache key string accepted by the `mizar-artifact` schema.

The optional cache key is local metadata excluded from stable artifact hashes.
`mizar-ir` must not construct `mizar-cache` keys or dependency fingerprints for
this field. If an upstream owner supplies no cache-key string, projection uses
`None`.

## Leakage Guard

Published artifacts must never contain raw:

- `SurfaceAst`;
- `TypedAst` or `ResolvedTypedAst`;
- `CoreIr` or `ControlFlowIr`;
- `VcIr`;
- `AtpProblem`;
- ATP backend process logs or certificates as inline payloads;
- proof witness payload bytes;
- kernel-internal proof state;
- storage handles, storage slot ids, blob ids, retain owners, or local worker
  state.

Task 12 must reject projection inputs whose stable string fields attempt to
carry these raw internal categories. Crate-local projection input records must
not provide arbitrary extension records or raw byte/JSON passthrough fields.
It is acceptable for internal cache records to contain raw IR bytes, but the
projection service must drop or reject them before constructing a
`VerifiedArtifactDraft`.

## Errors

Projection returns a draft only when all required inputs are sealed, readable,
current for the target snapshot, schema-compatible, sorted into canonical
order, and free of raw IR leakage.

Failure conditions include:

| Condition | Handling |
|---|---|
| missing or unsealed output | fail projection; return no draft |
| output from obsolete snapshot used as current | fail projection; return no draft |
| collected or unreadable handle | fail projection; return no draft |
| schema/version mismatch with `mizar-artifact` | fail projection; return no draft |
| duplicate projected ids | fail projection; return no draft |
| unsorted projected collections | canonicalize before draft validation |
| raw IR or storage internals detected in projected fields | fail projection; return no draft |
| witness/obligation schema inconsistency | fail projection; return no draft |
| missing real diagnostics/producer/publication integration | classify as `external_dependency_gap`; do not add stubs |

Projection errors are not proof failures and do not mutate storage. The
scheduler may rerun the producer or leave the previous artifact manifest in
place.

## Public Enum Forward-Compatibility

`ProjectionExternalDependencyGap` and `ProjectionError` are
`#[non_exhaustive]` for downstream crates. Future deferred integration gaps and
fail-closed projection rejection reasons may be added without breaking external
exhaustive matches. This module has no intentional exhaustive public-enum
exception; `mizar-ir` internal matches may remain exhaustive where they are
crate-local checks.

## Tests

Task 12 must cover:

- construction of a draft whose `VerifiedArtifact` can be written and read by
  `mizar-artifact`;
- exports, expression metadata, obligations, witness references, diagnostics,
  explanation references, and provenance ordering;
- rejection of unsealed, collected, stale, obsolete, or wrong-snapshot handles;
- rejection of raw `SurfaceAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`,
  `VcIr`, `AtpProblem`, kernel-state, storage-handle, and inline witness bytes
  leakage;
- accepted-obligation witness consistency delegated to `mizar-artifact`
  schemas without making `mizar-ir` proof authority;
- cache-key and dependency-fingerprint non-ownership;
- absence of `mizar-driver` and placeholder diagnostics/publication-token
  APIs.
