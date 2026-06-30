# mizar-ir Storage

> Canonical language: English. Japanese companion:
> [../ja/storage.md](../ja/storage.md).

## Purpose

This document specifies the storage boundary for `mizar-ir`.

`mizar-ir` owns compiler-internal storage slots for sealed phase outputs and
typed `PhaseOutputRef<T>` handles. It consumes snapshot and IR-local identity
from `mizar-session` and `mizar-ir::identity`. It does not own artifact
publication, cache-key construction, dependency fingerprints, proof acceptance,
trusted status, verifier policy selection, or kernel acceptance.

Storage is an optimization and lifetime boundary. Moving an output between
resident memory and a content-addressed blob must not change source semantics,
artifact schemas, cache authority, or proof status.

## Storage Model

The storage service stores one record per phase output:

- identity metadata: `PhaseOutputId`, `BuildSnapshotId`, `PipelinePhase`,
  `WorkUnit`, `OutputKind`, content hash, side-table hash, and lineage;
- a runtime kind tag and schema version for the stored payload;
- a typed payload location: resident memory or an internal blob;
- side-table records for source-map, diagnostic, explanation, and
  documentation-attachment references;
- storage-visible lifetime state: sealed/unsealed, retain owners, protected
  roots, and collection status.

Current/obsolete publication eligibility is not owned by storage. Later
publisher and snapshot-replacement logic supplies currentness checks and
explicit protection roots; storage records owning `BuildSnapshotId` values and
retention state but does not decide which snapshot is current.

The handle returned to consumers is `PhaseOutputRef<T>`. The type parameter is
a compile-time expectation for the output payload. Runtime storage still checks
the kind tag before returning `Arc<T>` so that a handle cannot reinterpret bytes
as another IR kind.

Slot numbers, arena ids, memory addresses, worker ids, temporary filenames, and
retain counters are storage implementation details. They are not stable
identity inputs and must not appear in published artifacts.

## Slot Lifecycle And Sealing

Storage uses a two-step lifecycle:

1. A producer allocates a private pending slot for one snapshot, phase, work
   unit, and output kind.
2. The producer seals the slot exactly once with the complete payload,
   side tables, and lineage, after the publisher has validated the producer
   context.

Only sealing returns a `PhaseOutputRef<T>`. Pending slots are invisible to
other tasks, artifact projection, cache writing, and current publication.
Reading a pending slot by id is rejected as `UnsealedOutput`. Sealing a slot
twice is rejected as `AlreadySealed`. Mutating a sealed output is not supported.
Before storing the payload, storage validates that the supplied lineage still
matches the canonical `PhaseOutputId` assigned by the identity module. A cloned
or mutated lineage whose content hash, side-table hash, parents, named inputs,
or producer metadata no longer match its output id is rejected as
`InvalidLineage` and the pending slot is abandoned.

A sealed handle is immutable:

- the payload location may be resident or blob-backed, but the logical payload
  is fixed;
- side-table records are fixed and identified by their hash;
- content and side-table hashes are fixed;
- lineage is fixed;
- retain and collect state may change only the storage lifetime, not the
  output value.

If sealing fails after allocation, the pending slot is abandoned and eligible
for collection. Abandoned pending slots never become current outputs.

## Typed Handles

`PhaseOutputRef<T>` must contain enough metadata to validate reads without
consulting a producer:

- `PhaseOutputId`;
- owning `BuildSnapshotId`;
- phase, work unit, and output kind;
- content hash and side-table hash;
- storage generation or equivalent stale-handle guard;
- the runtime kind tag expected for `T`.

`get<T>(&PhaseOutputRef<T>)` succeeds only when the slot exists, is sealed, has
not been collected, belongs to the same generation recorded in the handle, and
has the expected runtime kind tag. A type mismatch is an internal API error and
must fail closed; storage must not downcast or deserialize into an unrelated IR
kind.

For blob-backed payloads, the typed read path first validates the handle
generation, runtime kind tag, schema version, and blob content hash. It then
decodes only through the schema binding registered for that exact output kind
and `T`. Schema mismatch, kind mismatch, corrupt bytes, or missing codec data
fail closed as storage errors; the read path must not infer another kind,
reinterpret bytes, or synthesize cache validation.

The handle is not proof evidence. Possessing a handle, including a handle
rehydrated from cache by a later adapter, never upgrades proof status or trusted
status.

## Placement Policy

The resident-set rule is:

- handle metadata, identity indexes, lineage, and small side-table indexes stay
  resident;
- large payload bytes may spill to content-addressed internal blobs;
- collection removes unreferenced payload storage without changing published
  artifacts or source-level semantics.

The default spill threshold is **64 KiB of canonical payload bytes**. Payloads
whose canonical byte length is greater than 64 KiB are blob-backed by default;
payloads at or below 64 KiB remain resident by default. The threshold is a
performance and memory policy, not an identity rule. Changing it must not
change `PhaseOutputId`, content hash, proof status, or artifact projection.

Task 6 implements the policy with an explicit `StoragePolicy` so tests and
future build profiles can choose a lower or higher threshold. The crate must
not expose blob paths as artifact data. Blob references are internal
content-addressed references keyed by payload hash and schema version.

## Side Tables

Side tables are stored beside the sealed output, not inside published artifact
payloads. They may include stable references for:

- source maps and source ranges;
- diagnostic drafts or diagnostic identifiers;
- explanation request references;
- documentation attachment identifiers.

Real `mizar-diagnostics` integration is an `external_dependency_gap` until that
crate exists and exposes an integration seam. Until then, `mizar-ir` stores
stable side-table records without fabricating a diagnostics crate API.

Side-table changes are hashed separately from semantic payload bytes. The later
publisher spec defines when a side table is a semantic input. Storage preserves
the hashes it is given and does not decide cache compatibility.

## Retain And Collect

Storage lifetimes are explicit. A sealed output may be retained by owners such
as:

- dependent tasks that still need input handles;
- watch/LSP semantic snapshots;
- diagnostics or explanation requests;
- cache writers;
- artifact projection tasks that are reading sealed outputs.

`retain` creates a lifetime guard or registered owner for a `PhaseOutputId`.
`release` removes that owner. Current snapshot roots, active cache writers, and
artifact projection readers are represented to storage as explicit retain
owners or caller-provided protection roots, not as hidden driver or downstream
queries. `collect` may drop only outputs that are sealed or abandoned, have no
retain owners, are outside the caller-provided protection roots, and are not
covered by active in-crate guards.

If downstream ownership state is unknown, callers must keep the affected
outputs retained or protected. Collection is fail-closed: unknown protection is
treated as still live by omission from the collectable set, and storage must
not fabricate `mizar-driver`, `mizar-diagnostics`, publisher-token, cache-token,
or artifact-token APIs to discover liveness.

For batch builds, dependent tasks release their owners as soon as their
downstream output is sealed or the task fails. For watch and LSP, old snapshot
handles may remain readable while retained by stale diagnostics, explanation
requests, or semantic snapshot readers. After release, collection may remove
the old payload and subsequent reads fail as `CollectedOutput`.

Collection is idempotent. Running it twice over the same snapshot and owner
state must not change the second summary except for zero additional drops.
Collection must not mutate sealed payload content, published artifacts,
dependency fingerprints, proof status, or cache validation results.

## Snapshot Replacement Boundary

Task 13 adds current/obsolete snapshot state. Storage prepares for that rule by
recording the owning `BuildSnapshotId` on every slot and by keeping lifetime
state separate from current-publication eligibility.

After replacement, old snapshot outputs may remain readable while retained or
may be used as validated cache input by later cache-adapter logic. They cannot
be published as current results. Storage does not infer currentness from
`BuildSnapshotId` ordering or hash comparisons.

## Error Handling

| Condition | Handling |
|---|---|
| read before seal | Reject as `UnsealedOutput`; no handle may be published. |
| double seal | Reject as `AlreadySealed`; preserve the first sealed value. |
| pending slot missing from this storage service | Reject as `UnknownSlot`. |
| unknown sealed output id | Reject as `UnknownOutput`. |
| collected slot read | Reject as `CollectedOutput`; do not recreate a handle implicitly. |
| runtime kind mismatch | Reject as `TypeMismatch`; never reinterpret bytes. |
| lineage no longer matches canonical output id | Reject as `InvalidLineage` and abandon the pending slot. |
| corrupt internal blob | Reject as `CorruptBlob` and let the caller rerun the producer or treat a cache-origin value as a miss. |
| stale generation | Reject as `StaleHandle`; do not reuse storage slots across incompatible handles. |

All storage errors fail closed. If an error originates from cache rehydration,
the later cache adapter treats it as a cache miss before exposing a handle.

## Tests

Tasks 5 and 6 must cover:

- pending outputs are invisible and cannot be read as sealed handles;
- double seal is rejected and the first sealed value remains unchanged;
- `PhaseOutputRef<T>` round-trips the expected type and rejects mismatches;
- payloads over the configured spill threshold round-trip through a
  content-addressed blob;
- collection drops exactly unretained eligible outputs;
- retained outputs survive session or snapshot replacement until release;
- collection is idempotent;
- handle possession does not imply proof authority or trusted status.
