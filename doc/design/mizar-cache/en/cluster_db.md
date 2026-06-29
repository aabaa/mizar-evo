# Module: cluster_db

> Canonical language: English. Japanese companion:
> [../ja/cluster_db.md](../ja/cluster_db.md).

Status: specified by task 12. Task 13 implements origin writes, stale-origin
removal, and aggregate index rebuilds. Task 14 implements in-memory
import-scoped view materialization and invalidation over accepted aggregate
rows.

## Purpose

`cluster_db` owns the cache-side storage contract for `cluster-db/`: the
internal indexes over accepted registration, cluster, reduction, and
subsumption-DAG contributions plus import-scoped views derived from those
indexes.

The module is an optimization surface. It is not proof authority, not checker
authority, and not artifact publication authority. Deleting `cluster-db/`, any
origin record, or any import-scoped view may only force reindexing or
rechecking. It must not change source semantics, proof acceptance, trusted
`used_axioms`, interface hashes, or artifact publication status.

Only contributions already projected by their owner as accepted and
importer-visible may enter visible indexes. Parsed declarations, pending
registrations, recovered registrations, rejected proofs, open obligations,
externally attested proof material, backend diagnostics, backend logs, timing
metadata, and cache hit/miss state are never importer-visible cluster-db
inputs.

## Authorities And Inputs

This document refines:

- [spec 23.7.7](../../../spec/en/23.package_management_and_build_system.md#2377-storage-format-cluster-db-resolution-trace-diagnostic-explanation);
- [architecture 11](../../architecture/en/11.artifact_and_incremental_build.md);
- [architecture 17](../../architecture/en/17.cluster_trace_format.md);
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md);
- [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md).

`cluster_db` consumes producer-owned snapshots:

- accepted registration contribution summaries from checker/artifact owners;
- accepted cluster and reduction contribution payload summaries;
- accepted-witness or deterministic-discharge identity hashes for
  proof-backed accepted contributions;
- verifier policy fingerprints and schema/toolchain compatibility fields;
- origin identity, export visibility, dependency-facing interface hashes, and
  trace replay hashes.

Those values are validation and indexing inputs only. `mizar-cache` must not
construct an accepted contribution from raw source, raw checker diagnostics,
proof witness bytes, ATP evidence, or cache records.

## Conceptual Surface

Tasks 13 and 14 expose the source-level write/origin-tracking and
import-scoped-view data layer:

```rust
pub const CLUSTER_DB_SCHEMA_VERSION: &str;
pub const CLUSTER_DB_HASH_DOMAIN: &str;

pub struct ClusterContributionOrigin { ... }
pub struct ClusterContributionRecord { ... }
pub struct ClusterIndexEntry { ... }
pub struct ClusterAggregateRow { ... }
pub struct ClusterAggregateIndexes { ... }
pub struct ClusterIndexSnapshot { ... }
pub struct ImportScopedViewRequest { ... }
pub struct ImportScopedViewKey { ... }
pub struct ImportScopedView { ... }
pub struct ClusterDbIndex { ... }
pub struct ClusterDbUpdateReport { ... }

pub enum ClusterDbViewMiss {
    MissingRequiredIdentity { ... },
    UnsupportedSchema { ... },
    UnknownCompatibility { ... },
    MissingVisibleOrigin { ... },
}

pub enum ClusterDbWriteRejection {
    UnsupportedSchema { ... },
    MissingRequiredIdentity { ... },
    UnacceptedContribution { ... },
    NotImporterVisible { ... },
    IncompleteOrigin { ... },
    Uncacheable { ... },
    ConflictingDuplicateOrigin { ... },
    OriginKeyCollision { ... },
    UnknownCompatibility { ... },
}
```

Writes and rebuilds must fail closed. Unknown schema versions, unknown
schema/toolchain compatibility, incomplete origin footprints, missing origin
records, stale origin records, uncacheable markers, incompatible verifier
policy, or any unaccepted contribution force rejection or rebuild rather than
a visible row.

Task 14 exposes `ClusterDbIndex::import_scoped_view` as an in-memory cache-side
projection. It canonicalizes the request, sorts and deduplicates visible origin
keys, rejects missing visible origins, rejects unsupported cluster-db or
producer schema metadata, rejects unknown policy/toolchain/traversal
compatibility, rejects unknown producer schema compatibility, checks visible
origins against the active verifier policy, schema compatibility, and
toolchain compatibility, and filters the aggregate indexes by visible origin
set. It does not add scheduler hooks, durable view files, `mizar-ir` adapters,
proof status projection, lookup-outcome policy, or trace construction.

## Public Enum Policy

No exhaustive public enum exceptions are owned by this module. Every public
enum is `#[non_exhaustive]`; downstream matches must include a wildcard arm,
and new variants must fail closed rather than making unaccepted or hidden
contributions visible.

| Public enum | Forward-compatibility decision |
|---|---|
| `ClusterContributionVisibility` | `#[non_exhaustive]`; unknown visibility is not importer-visible. |
| `ClusterContributionStatus` | `#[non_exhaustive]`; unknown status is not accepted. |
| `ClusterContributionKind` | `#[non_exhaustive]`; unknown contribution kinds must not enter visible indexes. |
| `ClusterOriginFootprintCompleteness` | `#[non_exhaustive]`; unknown completeness states reject origin insertion or view reuse. |
| `ClusterIndexEntryKind` | `#[non_exhaustive]`; unknown index rows are not visible to importers. |
| `ClusterDbViewMiss` | `#[non_exhaustive]`; new miss reasons are diagnostic-only and fail closed. |
| `ClusterDbWriteRejection` | `#[non_exhaustive]`; new rejection reasons must not mutate visible indexes. |

## Store Layout

The default root is the configured `cache_dir`, normally `.mizar-cache/`:

```text
.mizar-cache/
  cluster-db/
    origins/
      <origin-key>.mcd
    graph.json
    subsumption-dag.json
    attr-index.json
    type-index.json
    reduction-index.json
    views/
      <import-view-key>.view
    tmp/
    quarantine/
```

The top-level JSON filenames mirror spec 23.7.7:

- `graph.json` contains the conditional cluster graph and struct inheritance
  graph in a compact adjacency format with origin identifiers for filtering.
- `subsumption-dag.json` contains the registration subsumption DAG skeleton,
  including preconditions that must be filtered against an import closure.
- `attr-index.json` maps generated attributes to origin-backed contributions.
- `type-index.json` maps type or mode trigger keys to origin-backed
  contributions.
- `reduction-index.json` maps resolved reduction `LHS` heads to accepted
  reduction rules, guard summaries, simplification-order metadata, and
  visibility origins.

`origins/` is the source of truth for cache-side invalidation. The aggregate
indexes are deterministic projections from origin records. `views/` stores
import-scoped materializations and may be deleted independently of origins or
aggregate indexes.

The spec-23.7.7 aggregate files remain canonical UTF-8 JSON surfaces with
top-level schema versions. Implementations may shard private `origins/` and
`views/` records by digest prefix, and may choose a private binary encoding for
those records, when the logical identity and canonical validation rules remain
unchanged. Directory order, file modification time, writer process id,
temporary file name, cache insertion order, and record arrival order are never
view or trace inputs.

## Origin Metadata

Each visible contribution has exactly one origin identity. The identity is
stable across rebuilds for the same semantic contribution and changes on
rename or deletion:

| Field | Meaning |
|---|---|
| `origin_key` | Domain-separated hash over package id, module path, stable contribution id, contribution kind, and schema family. |
| `package_id` | Owning package. |
| `module_path` | Owning source module. |
| `stable_contribution_id` | Producer-owned identity for the registration, cluster rule, reduction rule, or DAG node. |
| `label` | Source label or stable generated label. Label changes invalidate visible views even when diagnostics are the only downstream display use. |
| `contribution_kind` | `existential`, `conditional_cluster`, `functorial_cluster`, `reduction`, `struct_inheritance`, `subsumption_node`, or `subsumption_edge`. |
| `target_pattern_hash` | Canonical target/type/term pattern or DAG-node key. |
| `guard_hash` | Canonical guard and side-condition summary. |
| `declared_contribution_hash` | Canonical generated attribute, existence fact, result fact, reduction rule, inheritance edge, or DAG edge payload. |
| `accepted_visibility` | Export/import visibility projected by the owner. Private or local-only contributions are not visible to importers. |
| `accepted_status` | Owner-projected contribution status. Only `Accepted` may enter visible indexes. |
| `accepted_status_projection_hash` | Hash of the owner-projected accepted registration status used for visibility. It is not proof authority. |
| `accepted_witness_or_discharge_hash` | Owner-exported accepted witness or deterministic discharge identity for proof-backed accepted contributions. Missing identity makes that origin incomplete/uncacheable. It is validation metadata only. |
| `proof_backed` | Whether visibility depends on proof or deterministic discharge validation. If true, witness/discharge identity is mandatory. |
| `verifier_policy_fingerprint` | Policy under which the contribution became visible. Incompatible policies miss. |
| `policy_compatibility` | Verifier-policy compatibility fields. Missing, unknown, or unsupported compatibility fails closed. |
| `schema_compatibility` | Producer schema compatibility fields. Missing, unknown, or unsupported compatibility fails closed. |
| `toolchain_compatibility` | Producer toolchain compatibility fields. Missing, unknown, or unsupported compatibility fails closed. |
| `producer_schema_versions` | Schema families needed to interpret the contribution. Unknown required schemas miss. |
| `trace_replay_hashes` | ResolutionTrace replay hashes required to audit or replay the contribution. |
| `dependency_interface_hashes` | Dependency-facing module/registration interface hashes consumed by the contribution. |
| `origin_footprint_hash` | Complete footprint hash for stale-origin detection and view invalidation. |
| `footprint_completeness` | Completeness state of the origin dependency footprint. `IncompleteUncacheable` forces rejection. |
| `uncacheable` | Explicit marker that prevents insertion and reuse. |

The cache may store diagnostic references beside an origin record, but
diagnostics are not part of importer visibility and must not make a rejected
or external proof visible.

Duplicate `origin_key` records with identical payloads may be coalesced.
Duplicate origin keys with different payloads are invalid and force the
affected aggregate index and import-scoped views to miss.

## Accepted-Only Visibility

Importer-visible indexes may contain only contributions satisfying all of:

1. the producer marked the contribution accepted and importer-visible;
2. the contribution has complete origin metadata;
3. verifier policy and schema/toolchain compatibility are known and supported;
4. the contribution is not marked `uncacheable`;
5. dependency interface hashes and required trace replay hashes are available;
6. proof-backed accepted contributions carry an accepted witness hash or
   deterministic discharge hash exported by the owner;
7. stale records for the same origin have been removed before the view is used.

Unaccepted coherence, existence, compatibility, or reducibility proofs must
not seed `graph.json`, `subsumption-dag.json`, `attr-index.json`,
`type-index.json`, `reduction-index.json`, `views/`, `ResolutionTrace`,
`interface_hash`, or downstream dependency summaries.

Externally attested proof evidence is not an accepted cluster-db visibility
source. If an upstream policy records external evidence for diagnostics or
status explanation, `cluster_db` treats that material as non-visible for
importer indexes. Cache records themselves can never provide or upgrade an
accepted contribution projection.

## Aggregate Indexes

Aggregate indexes are canonical projections of accepted origin records:

- graph vertices and edges sort by graph kind, target/type key, source
  attribute or type key, generated attribute or target key, origin key, and
  contribution fingerprint;
- DAG nodes sort by `(symbol, guard_hash, origin_key)`;
- DAG edges sort by stronger node, weaker node, precondition hash, origin key,
  and contribution fingerprint;
- attribute index rows sort by attribute key, target graph key, origin key, and
  contribution fingerprint;
- type index rows sort by type or mode trigger key, origin key, and
  contribution fingerprint;
- reduction index rows sort by resolved `LHS` head, rule FQN, guard hash,
  simplification key, origin key, and contribution fingerprint.

Rows keep origin identifiers so an import-scoped view can filter without
duplicating the full graph for every import closure. The indexes must preserve
the deterministic traversal order from architecture 17: source type canonical
id, cluster origin module path, declaration source order, generated attribute
canonical id, and registration fingerprint. Cache write order and hash-map
iteration order must not affect a selected trace.

## Import-Scoped Views

An import-scoped view is a deterministic filter over aggregate indexes. Its key
includes:

- importing package id and module path;
- canonical import-closure identity;
- sorted visible origin set hash;
- verifier policy fingerprint;
- cluster-db schema version;
- producer schema versions required by the contributing origins;
- policy compatibility identity;
- producer schema compatibility identity;
- toolchain compatibility identity;
- traversal/profile settings that affect graph closure or reduction strategy.

The view contains only rows whose `origin_key` is visible from the import
closure. A view may include compact adjacency lists, filtered DAG edges,
filtered attr/type/reduction rows, and stable diagnostic references.

The task-14 implementation materializes this as `ImportScopedView` in memory.
The request is fail-closed: missing importer identity, unsupported
`cluster_db_schema_version`, missing producer schema versions, unknown or
unsupported policy/schema/toolchain/traversal compatibility, missing visible
origins, or visible origins whose verifier-policy or producer compatibility
metadata do not match the request all produce `ClusterDbViewMiss`.

Different import closures may see different views over the same aggregate
indexes. The cache must not physically duplicate origins per import closure,
and it must not infer additional cluster or reduction steps absent from a
`ResolutionTrace`.

## Invalidation

Changing any of the following invalidates every aggregate index row and
import-scoped view that can see the origin:

- registration label;
- target pattern;
- guard or side-condition summary;
- declared cluster, reduction, inheritance, or DAG contribution;
- coherence, existence, compatibility, or reducibility accepted status;
- accepted witness or deterministic discharge identity for proof-backed
  contributions;
- verifier policy fingerprint;
- origin identity;
- export visibility;
- required trace replay hash;
- dependency-facing interface hash;
- producer schema or toolchain compatibility.

Deleting or renaming a registration must remove stale origin records before
any dependent cache hit is accepted. If stale-origin removal cannot be proven,
the affected footprint is `IncompleteUncacheable` and lookup misses.

Import-set changes invalidate only the affected import-scoped views when the
aggregate origin records remain valid. Source changes inside the current
package invalidate the changed origins, the aggregate rows projected from
those origins, and views whose visible origin sets include them.

## ResolutionTrace Relationship

`cluster_db` stores rule indexes and view filters. It does not own
`ResolutionTrace` construction, kernel replay, or reduction selection.

Trace producers must record explicit cluster and reduction steps. Consumers
must not infer hidden transitive expansion, implicit cluster insertion, or
unrecorded reduction steps from cluster-db indexes. `ResolutionTrace` replay
hashes participate in dependency fingerprints and cache keys for outputs that
use derived cluster or reduction facts.

## Failure Semantics

Every failed compatibility, visibility, origin, or integrity check is a
cache miss or rebuild request. It is not proof rejection and not proof
acceptance.

The cache may emit stable diagnostics explaining:

- unknown cluster-db schema;
- unsupported producer schema;
- unknown toolchain compatibility;
- incompatible verifier policy;
- incomplete origin footprint;
- missing, stale, or duplicate-conflicting origin records;
- unaccepted or externally attested material offered as visible input;
- missing trace replay hashes;
- invalid canonical ordering or duplicate aggregate rows.

Diagnostic wording, backend logs, timing metadata, filesystem freshness, and
cache hit/miss timing do not affect view contents, selected traces, proof
selection, proof acceptance, or artifact order.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `CLUSTERDB-G001` | `external_dependency_gap` | Task 13 may consume concrete checker/artifact accepted-contribution producers if present. If the producer seam is insufficient, record the missing fields and defer rather than parsing raw source or fabricating accepted status. |
| `CLUSTERDB-G002` | `deferred` | Task 13 implements the in-memory origin-write and aggregate-index data layer. Durable `cluster-db/` file materialization remains deferred until a persistent cluster-db storage task is scheduled. |
| `CLUSTERDB-G003` | `deferred` | Task 14 implements in-memory import-scoped view materialization and invalidation tests. Durable `views/` file materialization remains deferred until a persistent cluster-db storage task is scheduled. |
| `CLUSTERDB-G004` | `external_dependency_gap` | Build scheduler integration remains owner-gated by task 15. `cluster_db` must not add placeholder scheduler APIs. |
| `CLUSTERDB-G005` | `external_dependency_gap` | IR cache adapter integration remains owner-gated by task 15. `cluster_db` must not add placeholder `mizar-ir` APIs. |

## Tests For Tasks 13 And 14

Task 13 must cover:

- accepted contribution insertion into origin records and aggregate indexes;
- accepted but non-importer-visible private/local-only contributions and
  rejected, pending, recovered, uncacheable, or externally attested
  contributions being excluded from visible indexes;
- incomplete origin metadata, incomplete origin footprints, missing dependency
  interface hashes, missing trace replay hashes, missing proof
  witness/discharge identity for proof-backed accepted origins, and unknown or
  unsupported schema/toolchain compatibility forcing misses instead of visible
  rows;
- rename and deletion removing stale origins before reuse;
- duplicate conflicting origins and cross-module origin-key collisions forcing
  misses without mutation;
- aggregate rebuilds touching only affected origins;
- deterministic aggregate ordering independent of write order, including
  same-index buckets.

Task 14 must cover:

- import-scoped view reuse across unrelated changes;
- visible-origin changes invalidating exactly affected views;
- filtering to visible origins across graph, subsumption-DAG, attribute, type,
  and reduction indexes;
- missing visible origins, unsupported cluster-db schema, unknown
  policy/schema/toolchain/traversal compatibility, mismatched verifier policy,
  missing producer schema metadata, and mismatched producer compatibility
  metadata forcing fail-closed misses;
- view contents independent of cache hit/miss timing and record arrival order;
- no hidden cluster/reduction step inference beyond explicit traces.

## Non-Goals

Tasks 12-14 do not implement scheduler integration, `mizar-ir` adapter
integration, artifact publication-token integration, proof status projection,
proof winner selection, kernel checking, or `ResolutionTrace` construction.
Task 14 also does not materialize durable `views/` files; persistent storage
remains a later cluster-db storage task.
