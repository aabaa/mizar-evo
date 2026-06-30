# mizar-ir Source/Spec Correspondence Audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

## Scope

This task-16 audit traces public `mizar-ir` APIs and promised module-spec
behavior to source files and tests. It covers the task stream through task 15:
identity, storage, publisher, cache adapter, artifact projection, snapshot
replacement, determinism/lifetime tests, and public enum compatibility.

The audit found no unresolved/current `spec_gap`, `source_drift`,
`test_expectation_drift`, or `boundary_violation` blocking crate completion.
Deferred downstream integration remains tagged as `external_dependency_gap`;
no placeholder APIs are added.

## Public API Trace

| Spec | Public API / behavior | Source | Tests | Status |
|---|---|---|---|---|
| `identity.md` | Snapshot-scoped `ModuleId`, `ItemId`, `ExprId`, `VcId`, `PhaseOutputId`; `PipelinePhase`, `WorkUnit`, `OutputKind`; deterministic assignment through `SnapshotHandleRegistry`. | `crates/mizar-ir/src/identity.rs` | `identity.rs` unit tests; `tests/determinism_lifetime.rs` | Covered. |
| `identity.md` | Parent lineage, duplicate-key rejection, unknown/incompatible snapshot rejection, and lineage identity validation. | `identity.rs`, `PhaseOutputLineage` | `identity.rs` unit tests; publisher/storage rollback tests | Covered. |
| `storage.md` | `IrStorageService`, `PendingOutputSlot<T>`, `PhaseOutputRef<T>`, `AnyPhaseOutputRef`, schema/kind/generation metadata, and typed read validation. | `crates/mizar-ir/src/storage.rs` | `storage.rs` unit tests; `tests/determinism_lifetime.rs` | Covered. |
| `storage.md` | Resident/blob placement, custom `StoragePolicy`, decoder fail-closed behavior, side-table storage, retain/release, collect, idempotence, and stale-generation rejection. | `storage.rs` | `storage.rs` unit tests; `tests/determinism_lifetime.rs` | Covered. |
| `publisher.md` | `PhaseOutputPublisher`, allowed work units, current/obsolete snapshot validation, slot metadata validation, parent-handle validation, deterministic content/side-table hashes, and no partial output exposure. | `crates/mizar-ir/src/publisher.rs` | `publisher.rs` unit tests | Covered. |
| `publisher.md`, internal 06 | `replace_current_snapshot` marks old snapshots obsolete while retained old outputs remain readable and cache-encodable. | `publisher.rs`, `storage.rs`, `cache_adapter.rs` | `publisher.rs`, `cache_adapter.rs`, `tests/determinism_lifetime.rs` | Covered. |
| `cache_adapter.md` | `IrCacheAdapter` encodes sealed outputs using caller-supplied `mizar-cache` keys, skips uncacheable/incompatible data, and rehydrates only validated hits into current-snapshot handles. | `crates/mizar-ir/src/cache_adapter.rs` | `cache_adapter.rs` unit tests | Covered. |
| `cache_adapter.md` | Fail-closed miss behavior for missing/incomplete/unknown/uncacheable/incompatible/corrupt/tampered records and storage/publisher failures before handle exposure. | `cache_adapter.rs` | `cache_adapter.rs` unit tests | Covered. |
| `projection.md` | `ArtifactProjectionService` validates current sealed outputs and builds unpublished `VerifiedArtifactDraft` values backed by `mizar-artifact` schemas. | `crates/mizar-ir/src/projection.rs` | `projection.rs` unit tests | Covered. |
| `projection.md` | Raw IR/storage leakage rejection, canonical projected ordering, duplicate rejection, schema validation, and external dependency gap recording. | `projection.rs` | `projection.rs` unit tests; `tests/lint_policy.rs` | Covered. |
| all module specs, task 15 | Public enum forward compatibility with `#[non_exhaustive]` and per-enum decisions in owning module specs. | public enums in `src/*.rs` | `tests/lint_policy.rs` | Covered. |
| `00.crate_plan.md` | Workspace membership, dependency boundary, crate ownership statement, no `mizar-driver`/`mizar-diagnostics` dependency, and no proof/cache authority markers in public boundaries. | `Cargo.toml`, `src/lib.rs`, module source | `tests/lint_policy.rs`; module unit tests | Covered. |

## Public API Inventory

This inventory names the public API surface traced by the table above. Public
fields on input/output records are covered with their containing types.

| Module | Public API surface | Spec/test correspondence |
|---|---|---|
| `lib.rs` | Public modules `cache_adapter`, `identity`, `publisher`, `projection`, and `storage`. | Crate boundary is specified in `00.crate_plan.md` and guarded by `tests/lint_policy.rs`. |
| `identity.rs` | Id newtypes `ModuleId`, `ItemId`, `ExprId`, `VcId`, `PhaseOutputId`; labels `PipelinePhase`, `WorkUnit`, `OutputKind`; records `NamedInputHash`, `ModuleIdentityInput`, `ItemIdentityInput`, `ExprIdentityInput`, `VcIdentityInput`, `OutputIdentityInput`, `PhaseOutputLineage`; `SnapshotHandleRegistry`; `IdentityError`. | `identity.md` specifies ownership, deterministic fields, duplicate keys, lineage, incompatible reuse, and enum policy. `identity.rs` unit tests and `tests/determinism_lifetime.rs` cover deterministic assignment, duplicate/conflict errors, parent lineage, incompatible snapshots, and proof non-authority. |
| `identity.rs` | Methods `SnapshotHandleRegistry::{new, register_snapshot, module_id, item_id, expr_id, vc_id, register_output, output_lineage}`; id `hash` accessors; `PhaseOutputLineage::{from_input, validate_identity}`; label constructors/accessors `PipelinePhase::{new, as_str}`, `WorkUnit::{new, as_str}`, `OutputKind::{new, as_str}`; `IdentityError` display/error traits. | Same as above; rollback-only helpers remain crate-private and are exercised through publisher/storage failure tests. |
| `storage.rs` | Constant `DEFAULT_BLOB_SPILL_THRESHOLD`; metadata types `SchemaVersion`, `OutputSlotId`, `StorageGeneration`, `ContentBlobId`, `StoragePolicy`; records `SideTableRecord`, `IrSideTables`, `AllocateSlotInput`, `SealOutputInput<T>`, `SealBlobOutputInput<T>`, `SealCanonicalOutputInput<T>`, `CollectInput`, `CollectionSummary`; handle types `PendingOutputSlot<T>`, `AnyPhaseOutputRef`, `PhaseOutputRef<T>`; blob/error/owner types `BlobDecoder<T>`, `BlobDecodeError`, `RetainOwner`, `StoragePlacement`, `StorageError`; service `IrStorageService`. | `storage.md` specifies slot lifecycle, sealing, typed handles, blob placement, side tables, retention, collection, snapshot replacement boundary, errors, and enum policy. `storage.rs` unit tests and `tests/determinism_lifetime.rs` cover the promised behavior. |
| `storage.rs` | Methods `IrStorageService::{new, with_policy, allocate, seal, seal_blob, seal_canonical, get, side_tables, side_tables_by_ref, validate_handle, typed_handle, retain, release, collect}`; constructors/accessors for `SchemaVersion`, `StoragePolicy`, `BlobDecoder`, `BlobDecodeError`, `RetainOwner`, `SideTableRecord`, `OutputSlotId`, `StorageGeneration`, `ContentBlobId`, `PendingOutputSlot`, `AnyPhaseOutputRef`, and `PhaseOutputRef`. | Covered by storage tests for invisible pending slots, double seal, typed/erased handle validation, blob spill/decode failure, side tables, retain/release/collect, stale generation, and no proof/trust authority. `typed_handle` is covered by erased-handle and projection/storage validation tests. |
| `publisher.rs` | Enums `OutputOrigin`, `PublicationTarget`, `PublishError`; records `AllowedWorkUnit`, `PublishOutputInput<T>`; service `PhaseOutputPublisher`. | `publisher.md` specifies producer context, current/obsolete validation, deterministic hashing, side tables, no partial IR exposure, cache/proof boundaries, errors, and enum policy. `publisher.rs` unit tests cover these promises. |
| `publisher.rs` | Methods `PhaseOutputPublisher::{new, storage, registry, register_current_snapshot, mark_obsolete, replace_current_snapshot, validate_current_snapshot, validate_current_output, allow_work_unit, allocate, publish}` and `AllowedWorkUnit::new`. | Publisher and projection tests cover current output validation, obsolete/superseded rejection, current root generation checks, work-unit checks, publish rollback, and no trust/cache authority. |
| `cache_adapter.rs` | Service `IrCacheAdapter`; enums `CacheAdapterCacheability`, `EncodeCacheRecordOutcome`, `CacheRehydrateOutcome<T>`, `CacheAdapterMiss`; records `EncodeCacheRecordInput<T>`, `RehydrateCacheHitInput<T>`. | `cache_adapter.md` specifies cacheability, record payload shape, validation-before-rehydration, rehydrated handles, freshness, fail-closed errors, and enum policy. `cache_adapter.rs` unit tests cover each state. |
| `cache_adapter.rs` | Methods `IrCacheAdapter::{new, publisher, successful_rehydrations, encode, rehydrate}`; `CacheAdapterMiss` display/error traits. | `successful_rehydrations` is covered by rehydration success and miss tests. `encode`/`rehydrate` tests cover old-handle non-copying, validated-hit-only handle exposure, tamper rejection, and cache/proof non-authority. |
| `projection.rs` | Service `ArtifactProjectionService`; records `ProjectVerifiedArtifactInput`, `VerifiedArtifactDraft`; enums `ProjectionExternalDependencyGap`, `ProjectionError`. | `projection.md` specifies ownership, draft model, inputs, projected data, leakage guard, errors, external dependency gaps, and enum policy. `projection.rs` unit tests cover these promises. |
| `projection.rs` | Methods `ArtifactProjectionService::{new, publisher, project_module}`; `ProjectionError` display/error traits. | Projection tests cover current output validation, duplicate and wrong-snapshot rejection, raw leakage rejection, canonical ordering, schema validation, witness consistency delegation, and external dependency gap recording. |
| `tests/lint_policy.rs` | Repository policy guard for workspace membership, dependency boundary, crate boundary text, public enum compatibility, and authority-marker absence. | Covered by `cargo test -p mizar-ir`; this is the source-side guard for crate-plan and task-15 policy. |

## Classified Gaps

| ID | Crate-plan class | Risk tag | Evidence | Current disposition |
|---|---|---|---|---|
| IR-G-004 | `design_drift` | `external_dependency_gap` | Real `mizar-driver` build sessions, driver cache scheduling, and publication tokens are absent from this checkout. | Recorded in crate plan, `lib.rs`, cache adapter spec, publisher spec, and projection spec. No placeholder APIs. |
| IR-G-005 | `design_drift` | `external_dependency_gap` | `mizar-diagnostics` is absent, so side-table records and projected diagnostics cannot integrate with a real registry/renderer. | Recorded in crate plan, storage/publisher/projection specs, and draft projection gaps. No stub crate or token. |
| IR-G-006 | `design_drift` | `external_dependency_gap` | Real resolver/checker/core/VC/ATP/kernel producer projection payloads and publication tokens are not exposed. | Projection uses crate-local stable records and `mizar-artifact` schemas only; no producer-token placeholder. |
| IR-G-007 | `test_gap` | `external_dependency_gap` | Full clean/incremental/parallel driver equivalence requires downstream orchestration crates. | Covered crate-locally by deterministic identity/hash/lifetime tests and deferred for system integration. |
| IR-G-008 | `boundary_violation` | guarded ownership constraint | Reimplementing `mizar-cache` cache keys, dependency fingerprints, proof-reuse validation, proof acceptance, trusted status, or kernel acceptance would violate ownership. | Guarded by specs, tests, crate dependencies, and authority-marker lint tests. No violation observed. |
| IR-G-009 | `design_drift` | resolved locally | Older internal 06 API sketches can be read as assigning cache-key or snapshot-identity ownership to `mizar-ir`. | Resolved in module specs and source by consuming `mizar-session` ids and `mizar-cache` validated records only. |

## Audit Result

Task 16 closes with no new follow-up task required. The remaining deferred
items are external dependency gaps owned by downstream integration crates, not
missing `mizar-ir` source. Task 17 should independently compare the English
canonical documents and Japanese companions for synchronization.
