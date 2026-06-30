# mizar-ir Publisher

> Canonical language: English. Japanese companion:
> [../ja/publisher.md](../ja/publisher.md).

## Purpose

This document specifies the phase-output publisher boundary for `mizar-ir`.

The publisher is the narrow API that phase services use to turn a complete
producer-local output into a sealed `PhaseOutputRef<T>`. It validates the
producer context, derives content and side-table hashes from canonical inputs,
registers phase-output lineage, and delegates storage placement to
`IrStorageService`.

The publisher does not own scheduling, real driver sessions, diagnostics
rendering, artifact manifest publication tokens, proof acceptance, trusted
status, verifier policy selection, kernel acceptance, `mizar-cache` `CacheKey`,
dependency fingerprints, or proof-reuse validation.

## Inputs

A publish request contains:

- the target `BuildSnapshotId`;
- the producing `PipelinePhase`, `WorkUnit`, and `OutputKind`;
- the payload and its schema version;
- a canonical payload byte sequence for content hashing and optional blob
  placement;
- parent `PhaseOutputId`s and named non-output input hashes;
- side-table records for source maps, diagnostics, explanations, and
  documentation attachments;
- an output origin classification: package source, retained stale snapshot
  input, validated cache input, or open-buffer/editor-only input.
- a publication target: current/package output or retained internal-only
  output.

The payload must already be complete. The publisher never accepts task-local
builders, mutable ASTs, partially generated VC sets, partial ATP problems, or
kernel-internal mutable state.

Real producer publication tokens are an `external_dependency_gap` until the
owning phase/driver crates expose them. Task 8 therefore implements only
crate-local producer context validation over explicit input fields; it must not
create placeholder `mizar-driver` APIs.

## Snapshot And Work-Unit Validation

The publisher validates:

- the request snapshot is registered with the local publisher state;
- the request snapshot is not obsolete for current publication;
- the work unit appears in an explicit crate-local allowed work-unit context
  supplied with the publish request or registered with the publisher for that
  phase/output kind;
- every parent output id belongs to a sealed parent handle in the same current
  snapshot;
- the pending storage slot was allocated for the same snapshot, phase, work
  unit, output kind, and schema version;
- open-buffer/editor-only and retained stale-snapshot inputs are rejected for
  current/package publication. They may only be retained as internal-only
  outputs when a future integration needs editor-only handles or stale reads;
  Task 8 rejects them for current output tests.

Currentness is explicit state, not a comparison between `BuildSnapshotId`
values. A later snapshot-replacement task extends this state so a newer
snapshot supersedes an older one. Until then, Task 8 may expose crate-local
`register_current_snapshot` and `mark_obsolete` operations for tests and
downstream adapters, but it must not infer currentness from hashes or consult a
missing driver.

Obsolete outputs may remain readable through storage while retained and may be
used as validated cache input by the cache adapter. They cannot be published as
current results after their snapshot is marked obsolete.

Task 8 must not validate cache reuse. It rejects parent handles whose snapshot
differs from the publish request. Task 10 may later consume a validated
`mizar-cache` hit and rehydrate a new sealed output in the current snapshot;
only that current-snapshot handle may then be supplied to the publisher as a
parent.

Retained stale-snapshot origins follow the same rule. They are internal-only
inputs until a later cache adapter validates them and creates a current-snapshot
handle. The publisher must not promote a retained stale output into a current
or package result by origin label alone.

## Canonical Hashing

The publisher derives two hashes before sealing:

- `content_hash`: semantic hash of the canonical payload bytes plus parent
  output content hashes read from already sealed parent handles/storage
  metadata and named input hashes;
- `side_table_hash`: hash of source-map, diagnostic, explanation, and
  documentation side-table records.

Hash inputs are domain-separated and length-delimited. Collection-valued inputs
are sorted by stable keys before hashing. The publisher must not include
`PhaseOutputId`, storage slot ids, storage generations, memory addresses,
worker ids, task ids, wall-clock time, temporary filenames, retain counters, or
cache lookup timing in either hash.

Diagnostic wording, explanation previews, and development-only side tables are
side-table inputs by default, not semantic content inputs. A future phase may
declare a side table as semantic only through a spec-owned phase contract.

The publisher does not construct `mizar-cache` `CacheKey`s or dependency
fingerprints. It records producer-declared named input hashes and parent output
relationships so the later cache adapter can consume validated
`mizar-cache` results without reimplementing cache ownership.

Producer-supplied parent hash fields are not authority. Parent content hashes
must come from sealed parent handles after the publisher has validated their
snapshot and storage metadata.

## Side Tables

Side tables attached through the publisher are stable records stored beside the
sealed output:

- source-map references and source-range summaries;
- diagnostic draft identifiers or stable diagnostic records;
- explanation references;
- documentation attachment identifiers.

Real `mizar-diagnostics` integration is an `external_dependency_gap`. The
publisher may store diagnostic-shaped side-table records, but it must not
invent a diagnostics crate API, renderer, severity policy, or publication
token.

Side-table attachment is immutable after sealing. If a producer needs different
side tables, it must publish a different output whose side-table hash reflects
that change.

## No Partial IR Exposure

The publisher rejects:

- missing or empty phase, work unit, or output kind labels;
- missing canonical payload bytes;
- payloads whose runtime output kind does not match the pending slot;
- parent outputs from incompatible snapshots;
- obsolete-snapshot publication as current output;
- retained stale-snapshot origins as package/current outputs;
- open-buffer/editor-only outputs as package/current outputs;
- attempts to expose unsealed output ids or pending slots.

Dependent tasks receive only sealed `PhaseOutputRef<T>` handles. They never
observe task-local mutable values or partial IR. Published artifacts are
created later by the projection boundary and must not contain raw
`SurfaceAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`, `VcIr`, `AtpProblem`, or
kernel-internal state.

## Cache And Proof Boundaries

A publisher-created handle is ordinary internal IR evidence that a phase
completed and the output was sealed. It is not proof acceptance, trusted
status, verifier policy approval, kernel acceptance, or a proof-reuse
validation result.

Cache hits are optimization data. A later cache adapter may create a sealed
handle only after `mizar-cache` validation succeeds. Incomplete, unknown,
uncacheable, incompatible, or corrupt records are misses before any
`PhaseOutputRef<T>` is reconstructed.

## Errors

| Condition | Handling |
|---|---|
| unknown snapshot | Reject as `UnknownSnapshot`. |
| obsolete snapshot used for current publication | Reject as `ObsoleteSnapshot`. |
| open-buffer/editor-only output requested as current/package output | Reject as `OpenBufferOutput`. |
| wrong phase/work-unit/output-kind/slot metadata | Reject before sealing and abandon the pending slot. |
| incompatible parent snapshot | Reject before storage handle reconstruction. |
| missing canonical payload bytes | Reject as `MissingCanonicalPayload`. |
| side-table hash mismatch or invalid side-table record | Reject before sealing. |
| storage seal error | Propagate the storage error without publishing a handle. |

All publisher errors fail closed. A failed publish attempt returns no handle and
does not make the output visible to dependent tasks, cache writers, projection,
or current publication.

## Tests

Task 8 must cover:

- deterministic content and side-table hashes for identical canonical inputs;
- parent and named-input canonicalization;
- wrong snapshot, obsolete snapshot, retained stale-origin current
  publication, and open-buffer current publication rejection;
- pending-slot metadata mismatch rejection;
- side tables retrievable from the sealed handle;
- no partial output visibility after failed publication;
- publisher-created handles do not carry proof authority, trusted status,
  cache-key authority, or dependency-fingerprint ownership.
