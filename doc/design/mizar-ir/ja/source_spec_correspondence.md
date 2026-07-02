# mizar-ir Source/Spec Correspondence Audit

> 正本は英語です。英語版:
> [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

## Scope

この task-16 audit は、公開 `mizar-ir` API と module spec が約束する挙動を source
file と test へ trace する。元の対象は task 15 までの task stream: identity、storage、
publisher、cache adapter、artifact projection、snapshot replacement、
determinism/lifetime test、public enum compatibility であった。task 20 は
`dispatch_input` へ audit scope を拡張する。

この audit では crate completion を妨げる unresolved/current な `spec_gap`、
`source_drift`、`test_expectation_drift`、`boundary_violation` は見つからなかった。
残る downstream integration は `external_dependency_gap` として tag されており、
placeholder API は追加しない。

## Public API Trace

| Spec | Public API / behavior | Source | Tests | Status |
|---|---|---|---|---|
| `identity.md` | snapshot-scoped `ModuleId`、`ItemId`、`ExprId`、`VcId`、`PhaseOutputId`; `PipelinePhase`、`WorkUnit`、`OutputKind`; `SnapshotHandleRegistry` による deterministic assignment。 | `crates/mizar-ir/src/identity.rs` | `identity.rs` unit tests; `tests/determinism_lifetime.rs` | Covered。 |
| `identity.md` | parent lineage、duplicate-key rejection、unknown/incompatible snapshot rejection、lineage identity validation。 | `identity.rs`, `PhaseOutputLineage` | `identity.rs` unit tests; publisher/storage rollback tests | Covered。 |
| `storage.md` | `IrStorageService`、`PendingOutputSlot<T>`、`PhaseOutputRef<T>`、`AnyPhaseOutputRef`、schema/kind/generation metadata、typed read validation。 | `crates/mizar-ir/src/storage.rs` | `storage.rs` unit tests; `tests/determinism_lifetime.rs` | Covered。 |
| `storage.md` | resident/blob placement、custom `StoragePolicy`、decoder fail-closed behavior、side-table storage、retain/release、collect、idempotence、stale-generation rejection。 | `storage.rs` | `storage.rs` unit tests; `tests/determinism_lifetime.rs` | Covered。 |
| `publisher.md` | `PhaseOutputPublisher`、allowed work units、current/obsolete snapshot validation、slot metadata validation、parent-handle validation、deterministic content/side-table hash、partial output 非公開。 | `crates/mizar-ir/src/publisher.rs` | `publisher.rs` unit tests | Covered。 |
| `publisher.md`, internal 06 | `replace_current_snapshot` が古い snapshot を obsolete としつつ、retain された古い output は読み取り可能かつ cache encode 可能に保つ。 | `publisher.rs`, `storage.rs`, `cache_adapter.rs` | `publisher.rs`, `cache_adapter.rs`, `tests/determinism_lifetime.rs` | Covered。 |
| `cache_adapter.md` | `IrCacheAdapter` は caller-supplied `mizar-cache` key を使って seal 済み output を encode し、uncacheable/incompatible data を skip し、validated hit だけを current-snapshot handle へ rehydrate する。 | `crates/mizar-ir/src/cache_adapter.rs` | `cache_adapter.rs` unit tests | Covered。 |
| `cache_adapter.md` | missing/incomplete/unknown/uncacheable/incompatible/corrupt/tampered record と storage/publisher failure は handle exposure 前に fail-closed miss になる。 | `cache_adapter.rs` | `cache_adapter.rs` unit tests | Covered。 |
| `projection.md` | `ArtifactProjectionService` は current seal 済み output を検証し、`mizar-artifact` schema に基づく unpublished `VerifiedArtifactDraft` を作る。 | `crates/mizar-ir/src/projection.rs` | `projection.rs` unit tests | Covered。 |
| `projection.md` | raw IR/storage leakage rejection、canonical projected ordering、duplicate rejection、schema validation、external dependency gap recording。 | `projection.rs` | `projection.rs` unit tests; `tests/lint_policy.rs` | Covered。 |
| `dispatch_input.md` | IR 所有 phase input identity bundle、seal 済み parent output handle、snapshot/currentness validation、rehydrated-handle boundary、snapshot-aware dispatch request、generic dispatch input provider seam。 | `crates/mizar-ir/src/dispatch_input.rs` | `dispatch_input.rs` unit tests と driver registry/driver tests | Covered。 |
| all module specs, task 15 | `#[non_exhaustive]` による public enum forward compatibility と owning module spec 内の enum ごとの decision。 | public enums in `src/*.rs` | `tests/lint_policy.rs` | Covered。 |
| `00.crate_plan.md` | workspace membership、dependency boundary、crate ownership statement、`mizar-driver`/`mizar-diagnostics` dependency 不在、public boundary に proof/cache authority marker がないこと。 | `Cargo.toml`, `src/lib.rs`, module source | `tests/lint_policy.rs`; module unit tests | Covered。 |

## Public API Inventory

この inventory は、上の表で trace した public API surface を列挙する。input/output record
の public field は、それを含む型で cover する。

| Module | Public API surface | Spec/test correspondence |
|---|---|---|
| `lib.rs` | public module `cache_adapter`、`dispatch_input`、`identity`、`publisher`、`projection`、`storage`。 | crate boundary は `00.crate_plan.md` で specified され、`tests/lint_policy.rs` で guard する。 |
| `dispatch_input.rs` | record `PhaseInputIdentities`、`SealedParentOutputHandle`、snapshot-bound `PhaseDispatchInputBundle`、`PhaseDispatchInputRequest`; trait `PhaseDispatchInputProvider<Task>`; enum `DispatchInputError`。 | `dispatch_input.md` は ownership、canonical identity construction、snapshot/currentness validation、provider request shape、external gap、test、enum policy を specify する。`dispatch_input.rs` unit test が canonical ordering、duplicate parent rejection、bundle/scheduler snapshot mismatch、wrong/obsolete/foreign-storage parent rejection、validated rehydrated handle を cover し、driver test が registry/front-door consumption を cover する。 |
| `dispatch_input.rs` | method `PhaseInputIdentities::{without_parent_outputs, input_hash, dependency_hashes, parent_output_hashes}`、`PhaseDispatchInputRequest::{new, task, snapshot}`、`SealedParentOutputHandle::{from_current_output, from_validated_rehydrated_output, as_output_ref, into_output_ref, output, snapshot, identity_hash}`、`PhaseDispatchInputBundle::{new, without_parent_outputs, snapshot, validate_snapshot, identities, parent_outputs, into_parts}`、`DispatchInputError` display/error trait。 | task-20 mizar-ir unit test と mizar-driver registry/driver test で cover する。raw parent-output hash constructor は公開しない。 |
| `identity.rs` | id newtype `ModuleId`、`ItemId`、`ExprId`、`VcId`、`PhaseOutputId`; label `PipelinePhase`、`WorkUnit`、`OutputKind`; record `NamedInputHash`、`ModuleIdentityInput`、`ItemIdentityInput`、`ExprIdentityInput`、`VcIdentityInput`、`OutputIdentityInput`、`PhaseOutputLineage`; `SnapshotHandleRegistry`; `IdentityError`。 | `identity.md` は ownership、deterministic field、duplicate key、lineage、incompatible reuse、enum policy を specify する。`identity.rs` unit test と `tests/determinism_lifetime.rs` が deterministic assignment、duplicate/conflict error、parent lineage、incompatible snapshot、proof non-authority を cover する。 |
| `identity.rs` | method `SnapshotHandleRegistry::{new, register_snapshot, module_id, item_id, expr_id, vc_id, register_output, output_lineage}`; id `hash` accessor; `PhaseOutputLineage::{from_input, validate_identity}`; label constructor/accessor `PipelinePhase::{new, as_str}`、`WorkUnit::{new, as_str}`、`OutputKind::{new, as_str}`; `IdentityError` display/error trait。 | 同上。rollback-only helper は crate-private のままで、publisher/storage failure test を通じて exercise する。 |
| `storage.rs` | constant `DEFAULT_BLOB_SPILL_THRESHOLD`; metadata type `SchemaVersion`、`OutputSlotId`、`StorageGeneration`、`ContentBlobId`、`StoragePolicy`; record `SideTableRecord`、`IrSideTables`、`AllocateSlotInput`、`SealOutputInput<T>`、`SealBlobOutputInput<T>`、`SealCanonicalOutputInput<T>`、`CollectInput`、`CollectionSummary`; handle type `PendingOutputSlot<T>`、`AnyPhaseOutputRef`、`PhaseOutputRef<T>`; blob/error/owner type `BlobDecoder<T>`、`BlobDecodeError`、`RetainOwner`、`StoragePlacement`、`StorageError`; service `IrStorageService`。 | `storage.md` は slot lifecycle、sealing、typed handle、blob placement、side table、retention、collection、snapshot replacement boundary、error、enum policy を specify する。`storage.rs` unit test と `tests/determinism_lifetime.rs` が約束された behavior を cover する。 |
| `storage.rs` | method `IrStorageService::{new, with_policy, allocate, seal, seal_blob, seal_canonical, get, side_tables, side_tables_by_ref, validate_handle, typed_handle, retain, release, collect}`; `SchemaVersion`、`StoragePolicy`、`BlobDecoder`、`BlobDecodeError`、`RetainOwner`、`SideTableRecord`、`OutputSlotId`、`StorageGeneration`、`ContentBlobId`、`PendingOutputSlot`、`AnyPhaseOutputRef`、`PhaseOutputRef` の constructor/accessor。 | storage test が invisible pending slot、double seal、typed/erased handle validation、blob spill/decode failure、side table、retain/release/collect、stale generation、proof/trust authority 不在を cover する。`typed_handle` は erased-handle と projection/storage validation test で cover する。 |
| `publisher.rs` | enum `OutputOrigin`、`PublicationTarget`、`PublishError`; record `AllowedWorkUnit`、`PublishOutputInput<T>`; service `PhaseOutputPublisher`。 | `publisher.md` は producer context、current/obsolete validation、deterministic hashing、side table、partial IR exposure 禁止、cache/proof boundary、error、enum policy を specify する。`publisher.rs` unit test がこれらの promise を cover する。 |
| `publisher.rs` | method `PhaseOutputPublisher::{new, storage, registry, register_current_snapshot, mark_obsolete, replace_current_snapshot, validate_current_snapshot, validate_current_output, allow_work_unit, allocate, publish}` と `AllowedWorkUnit::new`。 | publisher/projection test が current output validation、obsolete/superseded rejection、current root generation check、work-unit check、publish rollback、trust/cache authority 不在を cover する。 |
| `cache_adapter.rs` | service `IrCacheAdapter`; enum `CacheAdapterCacheability`、`EncodeCacheRecordOutcome`、`CacheRehydrateOutcome<T>`、`CacheAdapterMiss`; record `EncodeCacheRecordInput<T>`、`RehydrateCacheHitInput<T>`。 | `cache_adapter.md` は cacheability、record payload shape、validation-before-rehydration、rehydrated handle、freshness、fail-closed error、enum policy を specify する。`cache_adapter.rs` unit test が各 state を cover する。 |
| `cache_adapter.rs` | method `IrCacheAdapter::{new, publisher, successful_rehydrations, encode, rehydrate}`; `CacheAdapterMiss` display/error trait。 | `successful_rehydrations` は rehydration success/miss test で cover する。`encode`/`rehydrate` test は old-handle non-copying、validated-hit-only handle exposure、tamper rejection、cache/proof non-authority を cover する。 |
| `projection.rs` | service `ArtifactProjectionService`; record `ProjectVerifiedArtifactInput`、`VerifiedArtifactDraft`; enum `ProjectionExternalDependencyGap`、`ProjectionError`。 | `projection.md` は ownership、draft model、input、projected data、leakage guard、error、external dependency gap、enum policy を specify する。`projection.rs` unit test がこれらの promise を cover する。 |
| `projection.rs` | method `ArtifactProjectionService::{new, publisher, project_module}`; `ProjectionError` display/error trait。 | projection test が current output validation、duplicate/wrong-snapshot rejection、raw leakage rejection、canonical ordering、schema validation、witness consistency delegation、external dependency gap recording を cover する。 |
| `tests/lint_policy.rs` | workspace membership、dependency boundary、crate boundary text、public enum compatibility、authority-marker absence の repository policy guard。 | `cargo test -p mizar-ir` で cover する。これは crate plan と task-15 policy に対する source-side guard である。 |

## Classified Gaps

| ID | Crate-plan class | Risk tag | Evidence | Current disposition |
|---|---|---|---|---|
| IR-G-004 | `design_drift` | `external_dependency_gap` | 以前の docs は real `mizar-driver` session と diagnostics integration が存在しないものとして扱っていた。現在 crate は存在するが、real producer、diagnostics rendering、artifact-token、cache-compatibility、proof/semantic adapter、LSP seam は external gap のままである。 | crate plan と task 20 docs で更新する。placeholder API はない。 |
| IR-G-005 | `design_drift` | `external_dependency_gap` | `mizar-diagnostics` は存在するが、side-table record と projected diagnostic は `mizar-ir` に公開された real registry/renderer seam とはまだ統合できない。 | crate plan、storage/publisher/projection spec、draft projection gap に記録済み。stub API/token はない。 |
| IR-G-006 | `design_drift` | `external_dependency_gap` | real resolver/checker/core/VC/ATP/kernel producer projection payload と publication token は公開されていない。 | projection は crate-local stable record と `mizar-artifact` schema だけを使う。producer-token placeholder はない。 |
| IR-G-007 | `test_gap` | `external_dependency_gap` | full clean/incremental/parallel driver equivalence は downstream orchestration crate を必要とする。 | deterministic identity/hash/lifetime test で crate-local に cover し、system integration へ defer する。 |
| IR-G-008 | `boundary_violation` | guarded ownership constraint | `mizar-cache` cache key、dependency fingerprint、proof-reuse validation、proof acceptance、trusted status、kernel acceptance を再実装すると ownership を破る。 | spec、test、crate dependency、authority-marker lint test で guard する。違反は観測されない。 |
| IR-G-009 | `design_drift` | resolved locally | 古い internal 06 API sketch は cache-key または snapshot-identity ownership を `mizar-ir` に割り当てているように読める。 | module spec と source では `mizar-session` ids と `mizar-cache` validated record のみを消費する形で解決済み。 |
| IR-G-010 | `design_drift` | task 20 | earlier audit 後に driver/diagnostics crate availability が変わった。 | task 20 で crate plan/TODO/source-spec/bilingual docs を更新する。 |
| IR-G-011 | `source_drift` / `boundary_violation` risk | task 20 で resolved | task 20 前は `mizar-driver` が `PhaseInputIdentities` と raw parent hash を所有していた。 | identity と generic provider seam は現在 `mizar-ir` にあり、driver は `PhaseDispatchInputBundle` を消費する。 |
| IR-G-012 | `external_dependency_gap` | task 20 | producer output、semantic/proof adapter、artifact token、cache compatibility、LSP conversion は未準備。 | validated identity と seal 済み parent handle だけを運び、欠けた seam を fabricate しない。 |

## Audit Result

Task 16 は新規 follow-up task なしで close する。残る deferred item は downstream
integration crate が所有する external dependency gap であり、`mizar-ir` source の欠落ではない。
Task 17 は、英語正本 document と日本語 companion の同期を独立に比較する。
