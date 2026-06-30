# mizar-ir Cache Adapter

> Canonical language: English. Japanese companion:
> [../ja/cache_adapter.md](../ja/cache_adapter.md).

## Purpose

This document specifies the cache-adapter boundary for `mizar-ir`.

The adapter converts sealed `PhaseOutputRef<T>` values into internal cache
record payloads and rehydrates new sealed handles only from already validated
`mizar-cache` hits. It is an optimization boundary between `mizar-ir` storage
and `mizar-cache` records.

The adapter does not construct `mizar-cache` `CacheKey`s, dependency
fingerprints, dependency slices, proof-reuse validation decisions, verifier
policy compatibility decisions, proof acceptance, trusted status, or kernel
acceptance. It consumes those outcomes from `mizar-cache` and proof/status
owners. Cache hits and rehydrated handles are never proof authority.

Real scheduler lookup sessions, `mizar-driver` cache scheduling, producer
dependency-footprint export, and artifact publication tokens remain
`external_dependency_gap` items until the owning crates expose real seams.
`mizar-ir` must not add placeholder driver APIs or depend on `mizar-driver`.

## Inputs

### Encoding Input

Encoding a sealed output requires:

- a sealed `PhaseOutputRef<T>` from this `IrStorageService`;
- a supported `OutputKind` and payload schema version;
- canonical payload bytes produced by the phase serializer for that output
  kind and schema;
- side tables already attached to the sealed output;
- a cacheability decision supplied by the producer/cache boundary that says
  the dependency footprint is complete and cacheable;
- a `mizar-cache` key or record context supplied by the cache owner, not built
  by `mizar-ir`.

The adapter may reject encoding before writing any cache record. Rejection is
not a build failure by itself; the phase output remains sealed and usable by
dependent tasks. The scheduler may run without storing a cache entry.

### Rehydration Input

Rehydrating from cache requires:

- a target current `BuildSnapshotId`;
- an explicit current work unit, phase, output kind, and schema version;
- a validated cache-hit result from `mizar-cache`;
- cache record bytes whose header, key, schema, dependency footprint, policy,
  toolchain, source/dependency hashes, and proof-reuse metadata have already
  been accepted or classified by `mizar-cache`;
- current-snapshot parent handles or named input hashes supplied by the
  scheduler/producer context when the output lineage needs them;
- an `IrStorageService` and `SnapshotHandleRegistry` for the target snapshot.

The adapter must not create or return a `PhaseOutputRef<T>` before validation
has reached the "validated hit" state. A record classified as missing,
incomplete, unknown, unsupported, uncacheable, incompatible, corrupt, or
policy/proof invalid is a miss before handle reconstruction.

## Cacheability

An output is cacheable only when all of the following are true:

- the output is sealed and readable from the same storage service;
- the output belongs to package/current build work or to a validated current
  cache input being re-encoded;
- the output kind and payload schema have an adapter-owned codec;
- canonical payload bytes can be derived deterministically;
- side-table records are valid and serializable;
- the producer/cache boundary reports a complete dependency footprint for the
  phase;
- the record is not explicitly marked `uncacheable`;
- proof-related reuse metadata, if the output is proof-shaped and the record
  would be reused for proof-related work, has been validated as reusable by
  `mizar-cache`/proof owners.

Open-buffer/editor-only outputs, retained stale-snapshot outputs, partial
outputs, unsealed slots, collected handles, unsupported schema versions, and
outputs with unknown or incomplete dependency footprints are not cacheable.
They may remain valid in-memory IR, but the adapter must return a cache skip or
miss instead of manufacturing cache metadata.
Rejected, unknown, or incomplete proof-reuse metadata follows the same rule for
proof-shaped reuse: skip or miss before any handle is reconstructed.

`mizar-ir` may store a crate-local cacheability enum for adapter control flow,
but it must be a consumer-facing classification such as cacheable, skip,
miss, or incompatible. It must not duplicate `mizar-cache` dependency
fingerprint states or proof-reuse validation rules as an independent authority.

## Record Payload Shape

The adapter-owned payload is internal cache data. It may contain canonical
internal IR bytes because cache records are not published artifacts.

A cache record payload contains at most:

- output kind and payload schema version;
- payload content hash and side-table hash;
- canonical payload bytes or a `mizar-cache` blob reference supplied by the
  cache store;
- side-table records needed to reconstruct the sealed output;
- parent content-hash summaries and named input hashes needed to derive
  current-snapshot lineage;
- non-authoritative provenance such as producer phase and work-unit labels.

The record payload must not contain:

- a reusable `PhaseOutputRef<T>` from an old snapshot;
- storage slot ids, storage generations, memory addresses, retain owners, or
  other process-local storage internals as compatibility inputs;
- `mizar-cache` `CacheKey` construction logic;
- dependency fingerprint construction logic;
- proof accepted/trusted status, verifier-policy selection, kernel acceptance,
  or trusted `used_axioms` state as `mizar-ir` authority;
- artifact publication tokens or manifest transaction state.

If a payload includes raw internal IR bytes, they remain internal cache bytes.
They must not be returned through the artifact projection boundary or written
as published `*.mizir.json` artifacts.

## Validation Before Rehydration

Rehydration is a two-stage fail-closed process:

1. `mizar-cache` validates the lookup result. This includes cache key
   identity, dependency footprint completeness, schema/toolchain/policy
   compatibility, source/dependency hashes, cache record integrity, and
   proof-reuse metadata when applicable.
2. `mizar-ir` validates that the record payload can become a sealed output in
   the target snapshot. This includes output kind and schema support, payload
   hash match, side-table hash match, current-snapshot parent handle
   validation, deterministic lineage derivation, and storage sealing.

The adapter treats every non-validated state as a miss:

| State | Handling |
|---|---|
| missing record | Miss; return no handle. |
| incomplete dependency footprint | Miss before decoding payload bytes. |
| unknown schema/toolchain/policy compatibility | Miss before decoding payload bytes. |
| explicit `uncacheable` marker | Miss before decoding payload bytes. |
| incompatible key/header/dependency/proof validation | Miss before decoding payload bytes. |
| corrupt record bytes or blob hash mismatch | Miss; optionally report cache-integrity diagnostics through the cache owner. |
| unsupported output kind or schema | Miss; rerun producer if possible. |
| payload or side-table hash mismatch | Miss; do not seal. |
| parent handle missing, stale, collected, or from another snapshot | Miss; do not seal. |
| storage sealing error | Miss; rollback newly registered lineage and expose no handle. |

The adapter may log or return structured miss reasons, but miss reasons are
diagnostic/optimization data. They must not change proof acceptance, published
artifact identity, dependency-facing summaries, canonical diagnostics, or
source-level semantics.

## Rehydrated Handles

A rehydrated handle is a newly sealed `PhaseOutputRef<T>` in the target
`BuildSnapshotId`. It is not a recovered pointer to an old storage slot and is
not a stored handle copied out of the cache record.

The adapter derives current-snapshot lineage using the target snapshot, phase,
work unit, output kind, content hash, side-table hash, and validated
current-snapshot parents/named inputs. If the same current snapshot and inputs
produce an already registered/sealed output, rehydration may return the
existing handle. Across different snapshots, equal content hashes do not imply
handle identity or current publication rights.

Rehydration is allowed only as an optimization. If it fails, the scheduler
reruns the producer. A rehydrated handle carries no additional trust compared
with a producer-published handle. In particular, it must not upgrade:

- externally attested proof evidence to kernel-verified status;
- backend success to proof acceptance;
- cache lookup success to trusted status;
- dependency-fingerprint equality to proof acceptance;
- an obsolete snapshot output to a current result without validation.

## Snapshot And Freshness Boundaries

Cache records may have been produced for an older snapshot. The adapter must
not reuse old `PhaseOutputId`s or old storage handles as current results.
Instead, validation determines whether the record's payload applies to the
target `BuildSnapshotId`, and `mizar-ir` seals a current-snapshot output only
after that validation succeeds.

Obsolete snapshot outputs may be retained for LSP, diagnostics, explanation
requests, or cache writing. They cannot be published as current outputs and
cannot be rehydrated by origin label alone. The only path from stale data to a
current handle is the validated cache-hit path described above.

Open-buffer/editor-only outputs are not dependency artifacts. Cache records
for open-buffer dry-runs are out of scope until the owning scheduler and LSP
crates define a real seam. `mizar-ir` must classify that integration as an
`external_dependency_gap`, not create placeholder APIs.

## Errors And Results

The adapter exposes two successful outcomes:

- `Encoded`: a sealed output was converted into an internal cache record
  payload for the cache owner to store;
- `Rehydrated`: a validated hit was sealed into a current-snapshot
  `PhaseOutputRef<T>`.

All other outcomes are non-authoritative:

- `Skip`: the output is valid IR but should not be cached;
- `Miss`: the cache record could not be used and the producer should run;
- `Incompatible`: the adapter does not support this output kind/schema;
- `Corrupt`: cache data failed integrity checks and should be discarded by the
  cache owner.

No error path returns a partially reconstructed handle. No error path changes
published artifacts or proof status.

## Tests

Task 10 must cover:

- encoding and rehydration round-trip through a mock `mizar-cache` validation
  seam;
- no handle creation before the mock seam reports a validated hit;
- missing, incomplete dependency footprint, unknown compatibility,
  `uncacheable`, incompatible, corrupt, unsupported schema, and bad proof
  validation states all miss before handle reconstruction;
- tampered payload bytes/content hash and tampered side-table records/hash miss
  without sealing a handle or leaving registered lineage behind;
- rehydrated handles are sealed, typed, current-snapshot handles whose content
  hash, side-table hash, payload, and side tables match the original output;
- old snapshot handles are not copied out of cache records;
- cache hits and rehydrated handles do not carry proof authority, trusted
  status, `CacheKey` ownership, dependency-fingerprint ownership, or kernel
  acceptance.
