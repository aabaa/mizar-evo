# mizar-cache Source/Spec 対応監査

> 正本は英語です。英語版:
> [../en/source_spec_audit.md](../en/source_spec_audit.md)。

task 18 は public `mizar-cache` source surface audit を導入し、task 20 は
incremental fail-closed cache contract に合わせてこの audit を更新する。task
22 は private test-module split だけを source inventory に反映する。これらの
update は production behavior、public API semantics、cache lookup policy、
proof acceptance policy、artifact publication policy、downstream integration
を変更しない。まだ利用できない behavior は `mizar-cache` に stub として入れず、
`deferred` または `external_dependency_gap` として記録する。

## Scope And Method

監査対象は `crates/mizar-cache/src/lib.rs` が公開する public module、各
module の top-level public item、public data type が公開する public method、
`crates/mizar-cache/tests/` の integration / lint guard、および paired module
specification である。

- [cache_key.md](./cache_key.md)
- [dependency_fingerprint.md](./dependency_fingerprint.md)
- [cache_store.md](./cache_store.md)
- [proof_reuse.md](./proof_reuse.md)
- [cluster_db.md](./cluster_db.md)
- [integration_readiness.md](./integration_readiness.md)
- [todo.md](./todo.md)

Result: 現在の cache-owned implementation には未分類の `spec_gap`、
`test_gap`、`design_drift`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict` は見つからない。残る follow-up
work は下の gap register で明示的に分類する。

## Crate Module Exports

`src/lib.rs` は `mizar-cache` 所有の module として次だけを export する。

- `cache_key`
- `dependency_fingerprint`
- `cache_store`
- `proof_reuse`
- `cluster_db`

対応する source path は次の通り。

- `crates/mizar-cache/src/cache_key.rs`
- `crates/mizar-cache/src/cache_key/tests.rs`
- `crates/mizar-cache/src/dependency_fingerprint.rs`
- `crates/mizar-cache/src/dependency_fingerprint/tests.rs`
- `crates/mizar-cache/src/cache_store.rs`
- `crates/mizar-cache/src/cache_store/tests.rs`
- `crates/mizar-cache/src/proof_reuse.rs`
- `crates/mizar-cache/src/proof_reuse/tests.rs`
- `crates/mizar-cache/src/cluster_db.rs`
- `crates/mizar-cache/src/cluster_db/tests.rs`

Evidence: `crates/mizar-cache/tests/lint_policy.rs` は module export list、crate
file allowlist、package dependencies、proof-authority boundary term、
downstream-stub exclusion、public-enum policy、および本 audit の module coverage
を check する。`crates/mizar-cache/tests/determinism_suite.rs` は cross-module
determinism と cache-deletion semantics を cover する。
`crates/mizar-cache/tests/incremental_contract.rs` は cache key、dependency
footprint、cache-store lookup、proof-reuse validation をまたぐ task-20
architecture-22 fail-closed cache contract を cover する。

## Trust Boundary Restatement

`mizar-cache` が所有するのは optimization data である。Cache key、
dependency fingerprint、cache record、proof-reuse validation outcome、
cluster-db index は過去の output を reuse candidate にできるが、trusted proof
acceptance は owning proof/status layer が選択・project した後の
`mizar-kernel` `KernelCheckResult` だけから来る。

Cache record、externally attested evidence、backend diagnostics、backend logs、
timing metadata、cache hit/miss state、record arrival order、record write order は
kernel-verified status や trusted `used_axioms` に昇格しない。Cache deletion が
変えてよいのは performance だけである。

## Public Surface Inventory

### `cache_key`

Source path: `crates/mizar-cache/src/cache_key.rs`。

Top-level public item:

- `CACHE_KEY_SCHEMA_VERSION`, `CACHE_KEY_HASH_DOMAIN`
- `PipelinePhase`, `WorkUnit`, `SchemaVersion`, `PolicyFingerprint`
- `NamedHash`, `DependencyHash`, `DependencySliceHash`,
  `NamedSchemaVersion`, `SourceIdentity`, `DependencyArtifactAvailability`,
  `CompatibilityField`, `ProofReuseEvidenceIdentity`, `DiagnosticRefHash`
- `FootprintCompleteness`, `CacheValidationInputs`, `CacheKey`,
  `CacheKeyRequest`, `CacheKeyBuilder`, `CacheKeyBuildOutcome`,
  `CacheKeyBuildRejection`

Public inherent method:

- `PipelinePhase::new`, `PipelinePhase::as_str`
- `WorkUnit::new`, `WorkUnit::as_str`
- `SchemaVersion::new`, `SchemaVersion::as_str`
- `PolicyFingerprint::new`, `PolicyFingerprint::hash`
- `CacheKeyBuilder::new`, `CacheKeyBuilder::build`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| `CacheKeyBuilder` は `CacheKeyRequest` からの pure projection であり、scheduler、filesystem、timing、backend、cache-record state を読まない。 | `CacheKeyBuilder::build`、request validation、canonicalization、hash helper は supplied request だけを扱う。 | `cache_key_implementation_excludes_mutable_runtime_inputs`; `key_builder_is_deterministic_and_sorts_all_vectors`。 | consistent |
| すべての semantic field は domain-separated final key hash に参加し、diagnostics は明示的な diagnostic ref として渡された場合だけ参加する。 | `final_hash_for_request` と field-specific hash writer。 | `every_semantic_field_changes_final_hash`; `diagnostic_refs_participate_only_when_supplied_and_nondeterministic_inputs_are_absent`。 | consistent |
| duplicate identity conflict、unknown schema、missing proof-reuse validation field、incomplete footprint、`uncacheable` marker は fail closed する。 | validation helper と `CacheKeyBuildOutcome::{Uncacheable, NoKey}`。 | duplicate、unsupported-schema、incomplete-proof-reuse、uncacheable-marker tests。 | consistent |
| Public API は proof-authority projection term を公開しない。 | public data structure は hash、compatibility field、validation input だけを保持する。 | `cache_key_api_does_not_expose_proof_authority_terms`; crate-root boundary test。 | consistent |
| Public enum は forward-compatible である。 | すべての public enum が `#[non_exhaustive]`。 | `public_cache_enums_are_forward_compatible_and_documented`。 | guarded |

### `dependency_fingerprint`

Source path: `crates/mizar-cache/src/dependency_fingerprint.rs`。

Top-level public item:

- `DEPENDENCY_FINGERPRINT_SCHEMA_VERSION`,
  `DEPENDENCY_FINGERPRINT_HASH_DOMAIN`
- `FootprintOwner`, `FingerprintTargetKind`, `FingerprintIdentity`,
  `DependencyFingerprint`, `DependencySliceFingerprint`,
  `UnknownDependencyMarker`, `ProofReuseValidationInput`,
  `ProofReuseValidationState`, `DependencyFootprintCompleteness`,
  `DependencyFootprint`, `DependencyFootprintRequest`,
  `DependencyFootprintBuilder`, `DependencyFootprintBuildOutcome`,
  `DependencyFootprintBuildRejection`
- `RebuildTrigger`, `FingerprintChangeKind`, `DependencySlicePrecision`,
  `RebuildTriggerInput`, `RebuildTriggerDecision`,
  `RebuildTriggerSummary`, `RebuildTriggerEvaluator`

Public inherent method:

- `RebuildTriggerEvaluator::evaluate`,
  `RebuildTriggerEvaluator::evaluate_all`
- `FootprintOwner::from_module_identity`
- `FingerprintTargetKind::as_str`
- `FingerprintIdentity::from_module`
- `DependencyFingerprint::from_module_summary`,
  `DependencyFingerprint::from_registration_summary`,
  `DependencyFingerprint::module_implementation`
- `DependencySliceFingerprint::from_vc_slice`,
  `DependencySliceFingerprint::as_cache_key_slice`
- `DependencyFootprintBuilder::new`,
  `DependencyFootprintBuilder::build`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Footprint は cache-side dependency target、slice、compatibility field、unknown marker、proof-reuse validation input を canonicalize して hash する。 | `DependencyFootprintBuilder::build`、canonicalization helper、`semantic_fingerprint_hash`。 | footprint determinism、semantic-field、duplicate-conflict、unknown-marker tests。 | consistent |
| Importer-visible fingerprint と implementation-only fingerprint を分離し、non-interface edit で importer を invalidate しない。 | module / registration summary constructor と importer-visible filtering。 | `non_interface_summary_metadata_is_excluded_from_importer_visible_fingerprint`; `interface_change_invalidates_importer_visible_fingerprint`; `implementation_only_change_does_not_change_importer_visible_subset`。 | consistent |
| missing、unknown、unsupported、proof-reuse-failed、uncacheable dependency coverage は trust を与えず miss を強制する。 | completeness state、build rejection、proof-reuse state projection、trigger evaluator。 | `missing_unknown_and_uncacheable_inputs_force_miss`; `proof_reuse_validation_failures_force_miss_without_granting_trust`; `unsupported_footprint_schema_produces_no_footprint`。 | consistent |
| Rebuild-trigger evaluation は deterministic で、現在の coarse granularity では conservative である。 | `RebuildTriggerEvaluator::{evaluate, evaluate_all}`。 | trigger evaluator precedence と conservative-coarse-slice tests。 | consistent |
| Public API は proof-authority input と mutable-runtime input を除外する。 | public data は producer-owned hash と compatibility field だけを含む。 | dependency-fingerprint API / implementation boundary lint tests。 | consistent |
| Public enum は forward-compatible である。 | すべての public enum が `#[non_exhaustive]`。 | `public_cache_enums_are_forward_compatible_and_documented`。 | guarded |

### `cache_store`

Source path: `crates/mizar-cache/src/cache_store.rs`。

Top-level public item:

- `CACHE_RECORD_SCHEMA_VERSION`, `CACHE_RECORD_MAGIC`,
  `CACHE_BLOB_HASH_FAMILY`
- `CacheStoreRoot`, `CacheRecordHeader`, `CacheOutputDescriptor`,
  `CacheBlobRef`, `CacheRecord`
- `CacheLookupOutcome`, `CacheInsertOutcome`, `CacheMiss`,
  `CacheStoreError`

Public method は `CacheStoreRoot::{new, root, record_path, blob_path, lookup,
lookup_key_outcome, insert, write_blob}`、
`CacheRecord::{new_inline, new_blob}`、`CacheRecordHeader::{new_inline,
new_blob}`、`CacheBlobRef::for_output`、`CacheOutputDescriptor::{inline, blob}`
を含む。

Public inherent method:

- `CacheStoreRoot::new`, `CacheStoreRoot::root`,
  `CacheStoreRoot::record_path`, `CacheStoreRoot::blob_path`,
  `CacheStoreRoot::lookup`, `CacheStoreRoot::lookup_key_outcome`,
  `CacheStoreRoot::insert`, `CacheStoreRoot::write_blob`
- `CacheRecordHeader::new_inline`, `CacheRecordHeader::new_blob`
- `CacheBlobRef::for_output`
- `CacheOutputDescriptor::inline`, `CacheOutputDescriptor::blob`
- `CacheRecord::new_inline`, `CacheRecord::new_blob`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Lookup は no-key、uncacheable、missing、incompatible、corrupt、output-hash-mismatched record に対して fail closed する。 | `lookup`、record envelope/header validation、output/blob verification。 | no-key/uncacheable、incompatible-header、corrupt-record、output mismatch、exact-key、missing-input tests。 | consistent |
| Insert は uncacheable record を拒否し、divergent existing record を overwrite しない。 | `insert`、record encoding、create-new publication、conflict checks。 | divergent same-key、uncacheable、inline round-trip、write-order、descriptor mismatch tests。 | consistent |
| Blob record は output byte で content-addressed され、missing、corrupt、unsupported blob では fail closed する。 | `write_blob`、blob path validation、blob lookup verification。 | blob round-trip、writer convergence、divergent digest rejection、missing/corrupt/unsupported blob tests。 | consistent |
| Cache deletion は availability/performance だけを変え、proof acceptance や source semantics を変えない。 | missing record/blob では lookup が `Miss` を返し、artifact/proof status mutation は存在しない。 | `cache_store_deletion_changes_only_lookup_availability`。 | consistent |
| Public API は proof-authority projection term と reusable decision 用の mutable runtime input を除外する。 | cache store は record、key、compatibility field、output hash だけを扱う。 | cache-store API / implementation boundary lint tests。 | consistent |
| Public enum は forward-compatible である。 | すべての public enum が `#[non_exhaustive]`。 | `public_cache_enums_are_forward_compatible_and_documented`。 | guarded |

### `proof_reuse`

Source path: `crates/mizar-cache/src/proof_reuse.rs`。

Top-level public item:

- `PROOF_REUSE_SCHEMA_VERSION`
- `ProofReuseMetadataSnapshot`,
  `ProofReuseDependencyCompatibilitySnapshot`,
  `ProofReuseValidationEnvironment`, `ProofReuseValidationRequest`,
  `ProofReuseValidationOutcome`, `ProofReuseValidationHit`,
  `ProofReuseValidationMiss`, `ProofReuseMissReason`,
  `ProofReuseValidator`, `validate_proof_reuse`

Public function / method:

- `ProofReuseValidator::validate`
- `validate_proof_reuse`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Reuse は `mizar-proof` status/selection metadata を validation data として消費し、policy、deterministic winner selection、status projection、witness publication、artifact commit を所有しない。 | `StatusReuseMetadata` からの snapshot conversion。validator は supplied current/cached metadata と environment だけを比較する。 | proof-reuse downstream-stub lint tests; status metadata match/mismatch tests。 | consistent |
| Reusable winner class で、anchor、VC、local context、dependency slice、policy、evidence hash、schema、validation hash が一致する場合だけ hit できる。 | `ProofReuseValidator::validate` と class/evidence comparison helper。 | matching kernel/discharge tests、each-mismatch tests、missing-required-fields tests。 | consistent |
| unknown schema/toolchain、incomplete footprint、uncacheable marker、unavailable dependency artifact、externally attested / synthesized trusted data は miss する。 | environment fail-closed helper と non-reusable class checks。 | environment fail-closed、non-trusted/synthesized metadata、upstream completeness tests。 | consistent |
| Diagnostic ref は deterministic miss/hit metadata に限られ、validation identity や proof acceptance を変えない。 | `canonical_diagnostic_refs` と hit/miss payload。 | diagnostic-order と determinism-suite proof-reuse tests。 | consistent |
| Public API は proof authority、publication token、scheduler hook、IR adapter、trace/reduction constructor を除外する。 | public surface は validation request/outcome type だけを公開する。 | proof-reuse API / implementation boundary lint tests。 | consistent |
| Public enum は forward-compatible である。 | すべての public enum が `#[non_exhaustive]`。 | `public_cache_enums_are_forward_compatible_and_documented`。 | guarded |

### `cluster_db`

Source path: `crates/mizar-cache/src/cluster_db.rs`。

Top-level public item:

- `CLUSTER_DB_SCHEMA_VERSION`, `CLUSTER_DB_HASH_DOMAIN`
- `ClusterContributionVisibility`, `ClusterContributionStatus`,
  `ClusterContributionKind`, `ClusterOriginFootprintCompleteness`,
  `ClusterIndexEntryKind`
- `ClusterContributionOrigin`, `ClusterIndexEntry`,
  `ClusterContributionRecord`, `ClusterAggregateRow`,
  `ClusterAggregateIndexes`, `ClusterIndexSnapshot`,
  `ImportScopedViewRequest`, `ImportScopedViewKey`, `ImportScopedView`,
  `ClusterDbViewMiss`, `ClusterDbIndex`, `ClusterDbUpdateReport`,
  `ClusterDbWriteRejection`

Public method は `ClusterDbIndex::{new, origin, apply_module_update,
remove_origin, snapshot, import_scoped_view}` を含む。

Public inherent method:

- `ClusterDbIndex::new`, `ClusterDbIndex::origin`,
  `ClusterDbIndex::apply_module_update`, `ClusterDbIndex::remove_origin`,
  `ClusterDbIndex::snapshot`, `ClusterDbIndex::import_scoped_view`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| 完全で互換性のある accepted importer-visible contribution record だけを importer-visible index に書く。 | `validate_record`、origin canonicalization、accepted-only checks、index insertion helper。 | accepted-contribution、non-visible/unaccepted、incomplete-origin、unknown-schema/toolchain tests。 | consistent |
| Origin write、replacement、stale-origin removal、aggregate row、update report は deterministic で mutation-safe である。 | `apply_module_update`、`remove_origin`、`ClusterAggregateIndexes`、canonical row ordering。 | stale-origin cleanup、rebuild-report、deterministic ordering、duplicate conflict、coalescing tests。 | consistent |
| Import-scoped view は明示的な visible origin key で filter され、missing origin や incompatible request metadata では fail closed する。 | `import_scoped_view`、visible-origin-set hash、request canonicalization / validation。 | import-scoped filtering、unrelated-origin reuse、visible-origin invalidation、view fail-closed、hidden-trace tests。 | consistent |
| unaccepted、recovered、externally attested、inferred trace/reduction material は visible index に入れない。 | accepted-status / visibility validator。trace constructor や reduction selector API は存在しない。 | accepted-only、hidden trace、API boundary、implementation boundary lint tests。 | consistent |
| Public API は proof authority、publication token、scheduler hook、IR adapter、trace/reduction constructor を除外する。 | public surface は origin/index/view data structure だけを公開する。 | cluster-db API / implementation boundary lint tests。 | consistent |
| Public enum は forward-compatible である。 | すべての public enum が `#[non_exhaustive]`。 | `public_cache_enums_are_forward_compatible_and_documented`。 | guarded |

## Cross-Module Evidence

| Contract | Source/test correspondence |
|---|---|
| Crate scaffolding と dependency boundary | `Cargo.toml`、`src/lib.rs`、`tests/lint_policy.rs`。manifest、workspace、module-export、dependency、file-tree、lint-baseline tests が guard する。 |
| Cache data は optimization-only のまま | crate root boundary comment、API/implementation forbidden-term tests、module specs が proof-authority projection を除外する。 |
| Deterministic canonicalization | module unit tests と `tests/determinism_suite.rs` が cache-key ordering、record deletion/availability、proof-reuse diagnostic ordering、externally attested proof material の rejection を cover する。 |
| Incremental verification fail-closed contract | `tests/incremental_contract.rs` は complete trusted reuse path と、incomplete footprint、uncacheable marker、unknown schema/toolchain/policy、dependency artifact mismatch、proof metadata mismatch、deletion、diagnostic ordering、externally attested evidence の独立した miss を exercise する。 |
| Downstream integration は owner-gated のまま | `integration_readiness.md` と lint tests が scheduler hook、`mizar-ir` placeholder、publication-token shortcut、trace/reduction constructor を禁止する。 |
| Public enum forward compatibility | source attribute、paired module policy table、`public_cache_enums_are_forward_compatible_and_documented`。 |

## Guarded Test References

source/spec lint guard は、以下の named test が存在し、EN/JA audit から参照され続ける
ことを検証する。

- `cache_key_api_does_not_expose_proof_authority_terms`
- `cache_key_implementation_excludes_mutable_runtime_inputs`
- `key_builder_is_deterministic_and_sorts_all_vectors`
- `every_semantic_field_changes_final_hash`
- `diagnostic_refs_participate_only_when_supplied_and_nondeterministic_inputs_are_absent`
- `dependency_fingerprint_api_does_not_expose_proof_authority_terms`
- `dependency_fingerprint_implementation_excludes_mutable_runtime_inputs`
- `non_interface_summary_metadata_is_excluded_from_importer_visible_fingerprint`
- `interface_change_invalidates_importer_visible_fingerprint`
- `implementation_only_change_does_not_change_importer_visible_subset`
- `missing_unknown_and_uncacheable_inputs_force_miss`
- `proof_reuse_validation_failures_force_miss_without_granting_trust`
- `unsupported_footprint_schema_produces_no_footprint`
- `cache_store_api_does_not_expose_proof_authority_terms`
- `cache_store_implementation_keeps_boundary_terms_out_of_reuse_logic`
- `cache_store_deletion_changes_only_lookup_availability`
- `trusted_incremental_contract_requires_complete_cross_module_validation`
- `missing_or_unknown_incremental_inputs_fail_closed_before_reuse`
- `proof_reuse_requires_each_architecture_22_validation_field`
- `dependency_footprint_projects_missing_and_external_proof_metadata_to_miss`
- `cache_deletion_and_diagnostic_order_are_non_semantic`
- `externally_attested_evidence_never_becomes_trusted_reuse`
- `proof_reuse_api_does_not_expose_authority_results_or_publication_tokens`
- `proof_reuse_implementation_has_no_downstream_stub_or_timing_inputs`
- `proof_reuse_validation_is_deterministic_and_never_promotes_external_evidence`
- `cluster_db_api_does_not_expose_proof_authority_or_downstream_stubs`
- `cluster_db_implementation_has_no_downstream_stub_or_timing_inputs`
- `public_cache_enums_are_forward_compatible_and_documented`

## Gap Classification

task 20 の新規 gap ID は追加しない。この audit は task ledger の既存
`deferred` または `external_dependency_gap` ID をすべて再掲し、
`PROOFREUSE-G005` を external owner 側へ再分類し、placeholder source で
gap を修復しない。

| ID | Class | Owner / trigger | Current handling |
|---|---|---|---|
| `CACHE-G-003` | `external_dependency_gap` | `mizar-build` scheduler seam | scheduler hook や scheduling trait は追加しない。 |
| `CACHE-G-004` | `external_dependency_gap` | `mizar-ir` adapter | placeholder crate、mock adapter、rehydration API は追加しない。 |
| `CACHE-G-005` | `external_dependency_gap` | artifact/proof publication token | cache は hash と validation metadata だけを比較する。publication は artifact/proof owner に残る。 |
| `DEPFPR-G001` | `external_dependency_gap` | `mizar-build` dependency-fingerprint consumer | scheduler seam が landing するまで dependency fingerprint は local cache input に留める。 |
| `DEPFPR-G002` | `external_dependency_gap` | `mizar-ir` adapter | placeholder dependency-fingerprint adapter API は追加しない。 |
| `DEPFPR-G003` | `external_dependency_gap` | artifact committed publication token | cache は availability/hash input だけを記録する。 |
| `DEPFPR-G004` | `deferred` | producer-owned fine-grained theorem/definition/cluster/notation/mode/attribute slices | 現在の dependency fingerprint は conservative な published-summary と per-VC granularity を使う。 |
| `DEPFPR-G005` | `external_dependency_gap` | downstream proof/cache/artifact consumer | cache は proof-reuse validation identity だけを記録する。 |
| `CACHESTORE-G001` | `external_dependency_gap` | `mizar-build` scheduler seam | cache store API は local のままで work を schedule しない。 |
| `CACHESTORE-G002` | `external_dependency_gap` | `mizar-ir` adapter | record-payload adapter API は追加しない。 |
| `CACHESTORE-G003` | `external_dependency_gap` | artifact committed publication token | store は local availability と記録された domain/digest だけを check する。 |
| `CACHESTORE-G004` | `deferred` | later cluster-db index storage | record store は unaccepted registration を publish しない。 |
| `PROOFREUSE-G001` | `external_dependency_gap` | `mizar-build` scheduler seam | proof-reuse validation は local のままで schedule しない。 |
| `PROOFREUSE-G002` | `external_dependency_gap` | `mizar-ir` adapter | IR placeholder API は作らない。 |
| `PROOFREUSE-G003` | `external_dependency_gap` | artifact witness publication token | cache は selected witness hash だけを比較する。 |
| `PROOFREUSE-G004` | `external_dependency_gap` | distinct trusted `DischargedBuiltin` class 用 artifact witness schema | proof reuse は deterministic discharge hash を validate できるが、その class の trusted witness artifact は publish しない。 |
| `PROOFREUSE-G005` | `external_dependency_gap` | `mizar-build` / `mizar-artifact` scheduler と publication integration | task 20 は crate-owned cache lookup と proof-reuse validation contract を cover する。cross-crate clean/incremental equivalence は scheduler と artifact publication integration に残る。 |
| `CLUSTERDB-G001` | `external_dependency_gap` | checker/artifact accepted-contribution producer | 欠けている producer field は fabricate せず defer する。 |
| `CLUSTERDB-G002` | `deferred` | persistent cluster-db storage task | task 13 は in-memory origin と aggregate index を実装済み。durable `cluster-db/` file は deferred。 |
| `CLUSTERDB-G003` | `deferred` | persistent import-scoped view storage task | task 14 は in-memory view と invalidation を実装済み。durable `views/` file は deferred。 |
| `CLUSTERDB-G004` | `external_dependency_gap` | `mizar-build` scheduler integration | placeholder scheduler API は追加しない。 |
| `CLUSTERDB-G005` | `external_dependency_gap` | `mizar-ir` adapter integration | placeholder `mizar-ir` API は追加しない。 |
| `CACHE15-G001` | `external_dependency_gap` | `mizar-build` cache-aware scheduler seam | `mizar-cache` は local cache API だけを公開する。scheduler hook や scheduling trait は追加しない。 |
| `CACHE15-G002` | `external_dependency_gap` | `mizar-ir` cache adapter | `mizar-ir` は現在存在し cache-adapter validation-before-rehydration boundary を所有するが、build/driver execution を通じた end-to-end cache-record rehydration は未配線である。placeholder mock adapter や rehydration shortcut は追加しない。 |
| `CACHE15-G003` | `external_dependency_gap` | `mizar-artifact` / `mizar-proof` committed witness publication token | cache validation は hash と metadata だけを比較する。publication は artifact/proof owner に残す。 |

現在の source/spec audit では `repo_metadata_conflict` は見つからない。

## Conclusion

task 22 後の現在の public `mizar-cache` source surface は module specification
および test coverage と一致している。Cache は internal optimization owner の
ままであり、proof authority ではない。Cache record、dependency fingerprint、
proof-reuse validation metadata、backend diagnostics/logs、timing metadata、
cluster-db index は kernel-verified proof status や trusted `used_axioms` に
昇格しない。未完成 integration point は上で `deferred` または
`external_dependency_gap` として分類済みである。
