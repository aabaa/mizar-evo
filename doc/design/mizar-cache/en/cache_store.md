# Module: cache_store

> Canonical language: English. Japanese companion: [../ja/cache_store.md](../ja/cache_store.md).

Status: specified by task 7. Source implementation begins in tasks 8 and 9.

## Purpose

`cache_store` owns internal cache record persistence for `mizar-cache`.

A cache record is an optimization artifact. A hit may let a later build skip
work only after exact-key lookup, header compatibility checks, dependency
footprint validation, proof-reuse validation, and output hash verification all
succeed. Cache records are never proof authority and must not promote cache
records, externally attested evidence, backend diagnostics, backend logs,
timing metadata, or cache hit/miss state into kernel-verified status or
trusted `used_axioms`.

Deleting `.mizar-cache/` or any record within it must not change source-level
semantics, published package compatibility, proof acceptance, or artifact
publication. Deletion may only make later builds do more work.

## Public API

Tasks 8 and 9 should expose a small record/blob store API. Exact Rust names may
use newtypes, but the semantic decisions below must remain visible to tests
and audit code.

```rust
pub const CACHE_RECORD_SCHEMA_VERSION: &str;
pub const CACHE_RECORD_MAGIC: &[u8];

pub struct CacheStoreRoot { ... }
pub struct CacheRecordHeader { ... }
pub struct CacheRecord { ... }
pub struct CacheOutputDescriptor { ... }
pub struct CacheBlobRef { ... }

pub enum CacheLookupOutcome {
    Hit(CacheRecord),
    Miss(CacheMiss),
}

pub enum CacheInsertOutcome {
    Inserted,
    AlreadyPresent,
    RejectedUncacheable,
}
```

Lookup must return a miss for incompatible, unknown, incomplete, uncacheable,
or corrupt records by default. Explicit cache-audit modes may turn corruption
into diagnostics, but they still must not turn the record into a hit.

## Store Layout

The default root is the configured `cache_dir`, normally `.mizar-cache/`:

```text
.mizar-cache/
  records/
    <phase>/
      <cache-key-final-hash>.mcr
  blobs/
    <hash-family>/
      <digest>
  tmp/
  quarantine/
```

Implementations may shard `records/` or `blobs/` by digest prefix when the
lookup path remains a deterministic function of the exact phase and
`CacheKey.final_hash`. Directory order, file modification time, temporary file
names, writer process id, and record arrival order are never reuse inputs.

`cluster-db/`, import-scoped views, resolution traces, and diagnostic
explanation backing data are separate cache surfaces. This document specifies
phase record and blob storage only. It does not implement cluster-db indexing
or build integration.

## Record Identity

The record path is derived from:

- `CacheKey.phase`;
- `CacheKey.final_hash`;
- the cache record schema's path encoding version.

The path is not a trust boundary. After opening a candidate file, lookup must
recompute compatibility from the header and verify that the embedded key
matches the requested key. A file found at the expected path but carrying a
different key, phase, work unit, schema, or output hash is a miss and cache
integrity diagnostic candidate.

## CacheRecordHeader

`CacheRecordHeader` records every value needed to validate a candidate after
an exact-key path lookup:

| Field | Meaning |
|---|---|
| `cache_record_schema_version` | On-disk record encoding and compatibility version. Unknown versions miss. |
| `cache_key_schema_version` | Schema version of the embedded `CacheKey`. Unknown or mismatched versions miss. |
| `key` | The canonical `CacheKey` that produced this record. It is compared structurally, not only by display hash. |
| `key_hash` | Copy of `CacheKey.final_hash` for path and header diagnostics. |
| `phase` | Pipeline phase copied from the key for quick rejection and diagnostics. |
| `work_unit` | Phase-local unit copied from the key for quick rejection and diagnostics. |
| `produced_by` | Toolchain identity and compatibility fields used for reuse checks. Unknown compatibility misses. |
| `policy_fingerprint` | Verifier/proof policy fingerprint copied from the key. Incompatible policy misses. |
| `schema_versions` | Output, artifact, footprint, proof-reuse metadata, and record schemas that affect interpretation. Unknown required schemas miss. |
| `dependency_footprint_hash` | Reusable dependency footprint hash used by the output. `Complete` and `ConservativeComplete` are reusable; `IncompleteUncacheable`, missing, or unsupported footprints miss. |
| `dependencies` | Dependency artifact availability and hash checks required before reuse. Missing or changed artifacts miss. |
| `proof_reuse` | Optional proof-reuse validation metadata exported by `mizar-proof`. It is validation data only. |
| `output` | `CacheOutputDescriptor` describing inline or blob-backed output bytes and their hash. |
| `uncacheable` | Explicit marker that forces lookup and insert to miss/reject. |
| `diagnostic_refs` | Optional diagnostic-only references for explaining misses. They never affect proof acceptance. |

The header must not contain wall-clock hit/miss timing, filesystem freshness,
temporary paths, record write order, backend runtime duration, process ids, or
thread ids as compatibility inputs.

## Record Encoding

Task 8 uses one binary record envelope:

```text
magic bytes
record format version
header length
canonical UTF-8 JSON header
payload length
payload bytes
```

The JSON header is canonical: object keys are sorted, vectors are sorted by the
canonical keys documented by `cache_key.md` and `dependency_fingerprint.md`,
and duplicate identity keys are rejected before write. The payload is either:

- empty when `CacheOutputDescriptor` points at a content-addressed blob; or
- inline bytes for small outputs.

The output hash is computed over the canonical output bytes, not over the
record file path or record envelope. A key hash mismatch, payload length
mismatch, malformed canonical JSON, duplicate header key, unsupported enum
variant, missing blob, or output hash mismatch is a miss.

## Blob Store

Large outputs are stored content-addressably under:

```text
.mizar-cache/blobs/<hash-family>/<digest>
```

The hash family is explicit. The initial implementation may use the same
BLAKE3-based hash family used by cache keys and dependency fingerprints, but
readers must treat unknown hash families as misses. Blob writes are atomic:
write to `tmp/`, flush, verify the digest, and rename into the final digest
path. Concurrent writers for identical bytes must converge on the same final
file. Concurrent writers for different bytes cannot share the same digest
unless the digest check succeeds, so a mismatch is a cache integrity miss.

Blob references are internal. Published artifacts must be readable without
loading cache blobs.

## Lookup

Lookup is fail-closed:

1. Reject `CacheKeyBuildOutcome::NoKey` and
   `CacheKeyBuildOutcome::Uncacheable` as misses before disk lookup.
2. Derive the candidate path from phase and `CacheKey.final_hash`.
3. Read the candidate record, if present.
4. Decode the record envelope and canonical header.
5. Compare the embedded key structurally with the requested key.
6. Check cache record schema, cache key schema, toolchain compatibility,
   verifier policy compatibility, output schema compatibility, and
   proof-reuse metadata schema compatibility.
7. Verify reusable dependency footprint status, dependency artifact
   availability, dependency hashes, and dependency-slice fingerprints.
8. For proof/VC-related records, validate the `mizar-proof` proof-reuse
   metadata, `ObligationAnchor`, canonical VC fingerprint, local-context
   fingerprint, dependency-slice fingerprints, selected proof witness hash, or
   deterministic discharge hash as required by the key.
9. Read inline bytes or blob bytes and verify `output_hash`.
10. Return `Hit` only if every check succeeds.

Any failed check returns `Miss`. Unknown schema, unknown toolchain
compatibility, incomplete or unsupported dependency footprint, missing
dependency artifact, incomplete proof-reuse validation input, unsupported
proof evidence kind, explicit `uncacheable`, missing blob, and output hash
mismatch must not be interpreted as reusable data.

## Insert

Insert accepts only complete cacheable records:

- the key outcome is `Cacheable`;
- `uncacheable` is false;
- the dependency footprint is `Complete` or `ConservativeComplete`;
- compatibility fields are known and supported;
- proof-reuse validation metadata is present when proof/VC reuse would skip
  recomputation;
- the output hash matches the output bytes or blob bytes.

`Uncacheable` keys or records are not inserted as reusable records. An
implementation may emit diagnostic-only miss accounting elsewhere, but that
data must not live in the reusable `records/<phase>/` namespace and must never
return `Hit`.

Writes use temporary files under `tmp/`, validate the complete encoded record,
flush, and atomically rename into place. Two writers racing for the same key
must either publish byte-identical records or one writer must lose without
changing observable semantics. Divergent contents for the same validated key
are a cache integrity miss, not competing proof outcomes.

## Miss Reasons

Miss reasons are diagnostics for cache behavior. They are not proof status.

| Condition | Required outcome |
|---|---|
| missing record | `Miss(NotFound)` |
| unknown record schema or key schema | `Miss(UnknownSchema)` |
| unknown or incompatible toolchain | `Miss(UnknownToolchain)` or `Miss(IncompatibleToolchain)` |
| incomplete dependency footprint | `Miss(IncompleteFootprint)` |
| unsupported dependency footprint completeness or schema | `Miss(UnsupportedFootprint)` |
| explicit `uncacheable` marker | `Miss(Uncacheable)` |
| dependency artifact missing or hash mismatch | `Miss(DependencyUnavailable)` |
| policy incompatibility | `Miss(PolicyIncompatible)` |
| proof-reuse validation missing, mismatched, externally attested only, or unsupported | `Miss(ProofReuseInvalid)` |
| malformed envelope, malformed canonical JSON, duplicate header key, missing blob, or output hash mismatch | `Miss(CorruptRecord)` |

Verbose modes may attach these reasons to build diagnostics. Normal build
semantics proceed as if the record did not exist.

## Deletability

All cache records and blobs are deletable. A clean rebuild from source,
published artifacts, verifier policy, and dependency manifests must produce
the same externally visible results regardless of cache contents. Deleting a
record cannot:

- change accepted proof status;
- change trusted `used_axioms`;
- change artifact publication eligibility;
- change importer-visible module summaries or cluster-db views;
- change diagnostics ordering, except for optional cache-debug diagnostics.

When a record references a missing blob, lookup misses. Garbage collection may
delete unreferenced blobs and stale temporary files without publishing
artifact changes.

## Proof And Trust Boundary

`cache_store` may store proof-reuse validation metadata exported by
`mizar-proof`, accepted proof witness hashes, deterministic discharge hashes,
and dependency-slice fingerprints. These fields are validation predicates
only.

The module must not:

- create or reinterpret `KernelCheckResult`;
- mark any proof as kernel-verified;
- project trusted `used_axioms`;
- select proof winners;
- choose proof status for artifacts;
- treat externally attested evidence, backend logs, backend diagnostics, or
  cache records as trusted proof evidence.

Trusted acceptance comes only from `mizar-kernel` results consumed and
projected by the proof/status owner layers.

## Tests

Task 8 should cover at least:

- record round-trip for inline output;
- exact-key path lookup still validates embedded key and header;
- incompatible record schema, key schema, toolchain, policy, and output schema
  returning misses;
- `Complete` and `ConservativeComplete` footprints returning reusable records
  when every other compatibility check succeeds;
- `uncacheable`, incomplete footprint, unsupported footprint, and missing
  validation inputs returning misses;
- corrupted envelope/header/payload records returning misses with diagnostic
  reasons;
- output hash mismatch returning miss;
- record write order and arrival order not changing lookup results.

Task 9 should cover at least:

- blob round-trip by content digest;
- missing blob and digest mismatch returning misses;
- concurrent identical writers converging;
- divergent writers for the same digest being rejected;
- blob deletion causing a miss without changing build semantics.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `CACHESTORE-G001` | `external_dependency_gap` | `mizar-build` scheduler integration is not ready. This spec defines lookup/insert semantics only and does not add placeholder scheduling. |
| `CACHESTORE-G002` | `external_dependency_gap` | `mizar-ir` cache adapters are absent. Records may carry opaque output bytes, but no IR adapter API is created here. |
| `CACHESTORE-G003` | `external_dependency_gap` | Artifact committed publication-token integration is external. Cache records may depend on published artifact hashes only after the artifact owner exposes the token. |
| `CACHESTORE-G004` | `deferred` | `cluster-db` index storage is a later cache task. This record store spec does not make unaccepted registrations importer-visible. |

## Non-Goals

This module does not construct cache keys, compute dependency fingerprints,
validate proof-reuse policy, choose proof winners, project proof status, run
ATP, call the kernel, publish artifacts, update cluster-db views, or schedule
build work.
