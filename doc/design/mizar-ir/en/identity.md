# mizar-ir Identity

> Canonical language: English. Japanese companion:
> [../ja/identity.md](../ja/identity.md).

## Purpose

This document specifies the identity boundary for `mizar-ir`.

`mizar-ir` consumes `BuildSnapshotId` and session-owned source identity from
`mizar-session`. It does not construct build snapshots, source ids, cache keys,
dependency fingerprints, proof-reuse identities, or proof-trust state. Within a
single `BuildSnapshotId`, it assigns deterministic IR-local identities and
records parent/derived relationships between sealed phase outputs.

## Consumed And Owned Identities

| Identity | Owner | Stability |
|---|---|---|
| `BuildSnapshotId` | `mizar-session` | Deterministic for the exact source, dependency, lockfile, toolchain, and verifier-config state. |
| `SourceId` | `mizar-session` | Session-owned source handle accepted as non-canonical metadata; Task 3 does not retain it. |
| `ModuleId` | `mizar-ir` | Deterministic inside one `BuildSnapshotId` from package/module identity and source identity. |
| `ItemId` | `mizar-ir` | Deterministic inside one `BuildSnapshotId` from owning module and producer-declared item key. |
| `ExprId` | `mizar-ir` | Deterministic inside one `BuildSnapshotId` from owning item or module and producer-declared expression key. |
| `VcId` | `mizar-ir` | Deterministic inside one `BuildSnapshotId` from owning obligation order key; it is not a cross-edit proof-reuse identity. |
| `PhaseOutputId` | `mizar-ir` | Deterministic inside one `BuildSnapshotId` from phase, work unit, output kind, content hash, side-table hash, dependency output ids, and sorted named input hashes. |

`BuildSnapshotId` and `SourceId` are consumed session identities. `mizar-ir`
does not assign or persist them.

All IR-owned ids are snapshot-scoped unless an artifact projection maps them
into a stable published schema. Arena indices, memory addresses, slot numbers,
task ids, worker ids, filesystem temporary names, cache lookup timing, and
runtime duration are never stable identity inputs.

## Snapshot Identity Input

`BuildSnapshotId` already covers the exact build input state:

- normalized source versions and source hashes;
- dependency artifacts and their content hashes;
- lockfile hash;
- toolchain identity;
- verifier configuration hash.

`mizar-ir` must not weaken that identity. A handle created for one
`BuildSnapshotId` cannot be used as a current result for another snapshot. The
only allowed cross-snapshot reuse path is validated cache rehydration or an
explicit unchanged-input path defined by a later owning integration task. Both
paths are fail-closed and create a new current-snapshot handle only after
validation.

## Canonical ID Assignment

Each `mizar-ir` id is derived from a domain-separated canonical byte sequence:

```text
mizar-ir/<identity-family>/v1
snapshot = <BuildSnapshotId published-schema string>
canonical fields = fixed per-family producer-owned identity fields
```

The current implementation uses the fixed per-family field order listed below.
Collection-valued fields inside a family are sorted by their stable keys before
hashing.

The canonical fields for each family are:

| Family | Required fields |
|---|---|
| Module | package id, module path, source hash |
| Item | module id, item kind, normalized origin key, declaration order key |
| Expression | module id, optional item id, expression kind, producer path key |
| VC | module id, optional item id, obligation order key, canonical VC fingerprint when available |
| Phase output | phase, work unit, runtime output kind tag, content hash, side-table hash, dependency output ids, sorted named input hashes |

The registry also keeps a logical duplicate key for each family. The duplicate
key is narrower than the final id when a producer payload can legitimately
change the final hash:

- module duplicate key: package id and module path; source hash is payload;
- VC duplicate key: module id, optional item id, and obligation order key;
- phase-output duplicate key: phase, work unit, and output kind.

Registering the same logical key with a different payload in one snapshot is
rejected as a conflicting duplicate instead of silently producing a second
current identity.

Producer path keys are semantic or source-shaped keys supplied by the owning
phase. `mizar-ir` validates ordering, domains, and snapshot compatibility, but
it does not decide name resolution, type semantics, obligation anchors, proof
reuse, or proof acceptance.

`SourceId` may be accepted beside module identity inputs so later tasks can add
non-canonical source metadata plumbing, but Task 3 does not retain it in the
registry and it is intentionally not hashed into `ModuleId`: `mizar-session`
treats `SourceId` as a non-persistable session handle, while
`BuildSnapshotId`, package id, module path, and source hash provide the
deterministic identity inputs.

Collections used as identity input are sorted by their stable string or hash
keys before hashing. Duplicate identity keys with conflicting payloads are
rejected. Missing required identity fields are rejected rather than replaced
with empty defaults.

## Parent And Derived Output Relationships

The snapshot handle registry records lineage for each sealed output:

- `producer`: the phase/work-unit identity that produced the output;
- `parents`: input `PhaseOutputId`s consumed by the producer;
- `named_input_hashes`: non-output inputs declared by the producer;
- `content_hash`: semantic hash of the sealed output;
- `side_table_hash`: hash of side tables such as source maps, diagnostics,
  explanation refs, and documentation attachments.

A derived output must have the same `BuildSnapshotId` as every parent unless a
cache adapter has first validated and rehydrated that parent into the current
snapshot. Parent links round-trip through the registry and are used by storage,
publisher, cache adapter, and snapshot replacement logic. Lineage is not proof
evidence and must not be promoted to trusted status.

## Incompatible Snapshot Reuse

The registry rejects these operations:

- registering an output whose parent handle belongs to another snapshot;
- assigning an item, expression, or VC id from an owning module or item that is
  not registered in the same snapshot;
- assigning an expression or VC id from an item whose recorded owning module
  differs from the supplied module;
- publishing an obsolete snapshot output as a current result once the later
  publisher and snapshot-replacement tasks add current/obsolete state;
- assigning an IR-local id for a snapshot unknown to the registry;
- treating matching `ModuleId`, `ItemId`, `ExprId`, `VcId`, source range, arena
  id, or output hash alone as cross-snapshot validation;
- rehydrating a cache record before `mizar-cache` has validated schema,
  dependency footprint, policy compatibility, dependency artifacts, and
  proof-reuse metadata where applicable.

Cache hits are optimization data. A validated cache hit may reconstruct an
ordinary sealed handle for the current snapshot, but it does not upgrade proof
status or change the proof authority boundary.

## Snapshot Replacement

Task 13 extends this registry with snapshot replacement. When a newer snapshot
supersedes an older one, the registry marks the older snapshot as obsolete for
current publication. Existing retained handles may remain readable for stale
diagnostics, explanations, LSP requests, or validated cache input. They cannot
be reported as current results after supersession.

Snapshot replacement is explicit: the current snapshot is a registry property,
not a comparison between id values. Task 3 records only known snapshots and
lineage; current/obsolete publication checks are implemented by the later
publisher and snapshot-replacement tasks. Since `BuildSnapshotId` is a
hash-like opaque id, semantic ordering must not be inferred from it.

## Tests

Task 3 must cover:

- identical snapshot/id inputs produce identical IR-local ids;
- conflicting duplicate identity keys are rejected;
- handles from incompatible snapshots cannot be reused as current parents;
- parent/derived output lineage round-trips;
- `VcId` and other IR ids do not behave as proof-reuse authority.
