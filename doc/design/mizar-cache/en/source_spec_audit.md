# mizar-cache Source/Spec Correspondence Audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_audit.md](../ja/source_spec_audit.md).

Task 18 introduced this audit for the public `mizar-cache` source surface, and
task 20 updates it for the incremental fail-closed cache contract. Task 22
updates the source inventory for private test-module splits only. These updates
change no production behavior, public API semantics, cache lookup policy, proof
acceptance policy, artifact publication policy, or downstream integration.
Remaining unavailable behavior is recorded as `deferred` or
`external_dependency_gap` work instead of being stubbed in `mizar-cache`.

## Scope And Method

The audit covers the public modules exported by `crates/mizar-cache/src/lib.rs`,
top-level public items in each module, public methods exposed by the public
data types, integration and lint guards in `crates/mizar-cache/tests/`, and the
paired module specs:

- [cache_key.md](./cache_key.md)
- [dependency_fingerprint.md](./dependency_fingerprint.md)
- [cache_store.md](./cache_store.md)
- [proof_reuse.md](./proof_reuse.md)
- [cluster_db.md](./cluster_db.md)
- [integration_readiness.md](./integration_readiness.md)
- [todo.md](./todo.md)

Result: no unclassified `spec_gap`, `test_gap`, `design_drift`,
`source_drift`, `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict` is observed for the current
cache-owned implementation. Existing follow-up work remains explicitly
classified in the gap register below.

## Crate Module Exports

`src/lib.rs` exports exactly these `mizar-cache` owned modules:

- `cache_key`
- `dependency_fingerprint`
- `cache_store`
- `proof_reuse`
- `cluster_db`

The corresponding source paths are:

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

Evidence: `crates/mizar-cache/tests/lint_policy.rs` checks the module export
list, crate file allowlist, package dependencies, proof-authority boundary
terms, downstream-stub exclusions, public-enum policy, and this audit's module
coverage. `crates/mizar-cache/tests/determinism_suite.rs` covers cross-module
determinism and cache-deletion semantics.
`crates/mizar-cache/tests/incremental_contract.rs` covers the task-20
architecture-22 fail-closed cache contract across cache keys, dependency
footprints, cache-store lookup, and proof-reuse validation.

## Trust Boundary Restatement

`mizar-cache` owns optimization data. Cache keys, dependency fingerprints,
cache records, proof-reuse validation outcomes, and cluster-db indexes can make
a previous output a candidate for reuse, but trusted proof acceptance comes
only from `mizar-kernel` `KernelCheckResult` values after the owning
proof/status layers select and project them.

Cache records, externally attested evidence, backend diagnostics, backend
logs, timing metadata, cache hit/miss state, record arrival order, and record
write order are not elevated to kernel-verified status or trusted
`used_axioms`. Cache deletion changes only performance.

## Public Surface Inventory

### `cache_key`

Source path: `crates/mizar-cache/src/cache_key.rs`.

Top-level public items:

- `CACHE_KEY_SCHEMA_VERSION`, `CACHE_KEY_HASH_DOMAIN`
- `PipelinePhase`, `WorkUnit`, `SchemaVersion`, `PolicyFingerprint`
- `NamedHash`, `DependencyHash`, `DependencySliceHash`,
  `NamedSchemaVersion`, `SourceIdentity`, `DependencyArtifactAvailability`,
  `CompatibilityField`, `ProofReuseEvidenceIdentity`, `DiagnosticRefHash`
- `FootprintCompleteness`, `CacheValidationInputs`, `CacheKey`,
  `CacheKeyRequest`, `CacheKeyBuilder`, `CacheKeyBuildOutcome`,
  `CacheKeyBuildRejection`

Public inherent methods:

- `PipelinePhase::new`, `PipelinePhase::as_str`
- `WorkUnit::new`, `WorkUnit::as_str`
- `SchemaVersion::new`, `SchemaVersion::as_str`
- `PolicyFingerprint::new`, `PolicyFingerprint::hash`
- `CacheKeyBuilder::new`, `CacheKeyBuilder::build`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| `CacheKeyBuilder` is a pure projection from `CacheKeyRequest` and never reads mutable scheduler, filesystem, timing, backend, or cache-record state. | `CacheKeyBuilder::build`, request validation, canonicalization, and hash helpers operate only on the supplied request. | `cache_key_implementation_excludes_mutable_runtime_inputs`; `key_builder_is_deterministic_and_sorts_all_vectors`. | consistent |
| Every semantic field participates in the domain-separated final key hash, while diagnostics are included only when supplied as explicit diagnostic refs. | `final_hash_for_request` and field-specific hash writers. | `every_semantic_field_changes_final_hash`; `diagnostic_refs_participate_only_when_supplied_and_nondeterministic_inputs_are_absent`. | consistent |
| Duplicate identity conflicts, unknown schema, missing proof-reuse validation fields, incomplete footprints, and `uncacheable` markers fail closed. | validation helpers and `CacheKeyBuildOutcome::{Uncacheable, NoKey}`. | duplicate, unsupported-schema, incomplete-proof-reuse, and uncacheable-marker tests. | consistent |
| Public API does not expose proof-authority projection terms. | public data structures store hashes, compatibility fields, and validation inputs only. | `cache_key_api_does_not_expose_proof_authority_terms`; crate-root boundary test. | consistent |
| Public enums are forward-compatible. | all public enums are `#[non_exhaustive]`. | `public_cache_enums_are_forward_compatible_and_documented`. | guarded |

### `dependency_fingerprint`

Source path: `crates/mizar-cache/src/dependency_fingerprint.rs`.

Top-level public items:

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

Public inherent methods:

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
| Footprints canonicalize and hash cache-side dependency targets, slices, compatibility fields, unknown markers, and proof-reuse validation inputs. | `DependencyFootprintBuilder::build`, canonicalization helpers, and `semantic_fingerprint_hash`. | footprint determinism, semantic-field, duplicate-conflict, and unknown-marker tests. | consistent |
| Importer-visible and implementation-only fingerprints are separated so non-interface edits do not invalidate importers. | module and registration summary constructors plus importer-visible filtering. | `non_interface_summary_metadata_is_excluded_from_importer_visible_fingerprint`; `interface_change_invalidates_importer_visible_fingerprint`; `implementation_only_change_does_not_change_importer_visible_subset`. | consistent |
| Missing, unknown, unsupported, proof-reuse-failed, or uncacheable dependency coverage forces misses without granting trust. | completeness states, build rejections, proof-reuse state projection, and trigger evaluator. | `missing_unknown_and_uncacheable_inputs_force_miss`; `proof_reuse_validation_failures_force_miss_without_granting_trust`; `unsupported_footprint_schema_produces_no_footprint`. | consistent |
| Rebuild-trigger evaluation is deterministic and conservative for coarse current granularity. | `RebuildTriggerEvaluator::{evaluate, evaluate_all}`. | trigger evaluator precedence and conservative-coarse-slice tests. | consistent |
| Public API excludes proof-authority and mutable-runtime inputs. | public data contains producer-owned hashes and compatibility fields only. | dependency-fingerprint API and implementation boundary lint tests. | consistent |
| Public enums are forward-compatible. | all public enums are `#[non_exhaustive]`. | `public_cache_enums_are_forward_compatible_and_documented`. | guarded |

### `cache_store`

Source path: `crates/mizar-cache/src/cache_store.rs`.

Top-level public items:

- `CACHE_RECORD_SCHEMA_VERSION`, `CACHE_RECORD_MAGIC`,
  `CACHE_BLOB_HASH_FAMILY`
- `CacheStoreRoot`, `CacheRecordHeader`, `CacheOutputDescriptor`,
  `CacheBlobRef`, `CacheRecord`
- `CacheLookupOutcome`, `CacheInsertOutcome`, `CacheMiss`,
  `CacheStoreError`

Public methods include `CacheStoreRoot::{new, root, record_path, blob_path,
lookup, lookup_key_outcome, insert, write_blob}`,
`CacheRecord::{new_inline, new_blob}`, `CacheRecordHeader::{new_inline,
new_blob}`, `CacheBlobRef::for_output`, and
`CacheOutputDescriptor::{inline, blob}`.

Public inherent methods:

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
| Lookup is fail-closed for no-key, uncacheable, missing, incompatible, corrupt, or output-hash-mismatched records. | `lookup`, record envelope/header validation, and output/blob verification. | no-key/uncacheable, incompatible-header, corrupt-record, output mismatch, exact-key, and missing-input tests. | consistent |
| Insert rejects uncacheable records and never overwrites a divergent existing record. | `insert`, record encoding, create-new publication, and conflict checks. | divergent same-key, uncacheable, inline round-trip, write-order, and descriptor mismatch tests. | consistent |
| Blob records are content-addressed by output bytes and fail closed on missing, corrupt, or unsupported blobs. | `write_blob`, blob path validation, and blob lookup verification. | blob round-trip, writer convergence, divergent digest rejection, and missing/corrupt/unsupported blob tests. | consistent |
| Cache deletion changes only availability/performance, not proof acceptance or source semantics. | lookup returns `Miss` for missing records or blobs; no artifact/proof status mutation exists. | `cache_store_deletion_changes_only_lookup_availability`. | consistent |
| Public API excludes proof-authority projection terms and mutable runtime inputs for reusable decisions. | cache store deals with records, keys, compatibility fields, and output hashes only. | cache-store API and implementation boundary lint tests. | consistent |
| Public enums are forward-compatible. | all public enums are `#[non_exhaustive]`. | `public_cache_enums_are_forward_compatible_and_documented`. | guarded |

### `proof_reuse`

Source path: `crates/mizar-cache/src/proof_reuse.rs`.

Top-level public items:

- `PROOF_REUSE_SCHEMA_VERSION`
- `ProofReuseMetadataSnapshot`,
  `ProofReuseDependencyCompatibilitySnapshot`,
  `ProofReuseValidationEnvironment`, `ProofReuseValidationRequest`,
  `ProofReuseValidationOutcome`, `ProofReuseValidationHit`,
  `ProofReuseValidationMiss`, `ProofReuseMissReason`,
  `ProofReuseValidator`, `validate_proof_reuse`

Public functions and methods:

- `ProofReuseValidator::validate`
- `validate_proof_reuse`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Reuse consumes `mizar-proof` status/selection metadata as validation data and does not own policy, deterministic winner selection, status projection, witness publication, or artifact commit. | snapshot conversion from `StatusReuseMetadata`; validator compares supplied current/cached metadata and environment only. | proof-reuse downstream-stub lint tests; status metadata match and mismatch tests. | consistent |
| Only reusable winner classes with matching anchor, VC, local context, dependency slice, policy, evidence hash, schema, and validation hash can hit. | `ProofReuseValidator::validate` and class/evidence comparison helpers. | matching kernel/discharge tests, each-mismatch tests, missing-required-fields tests. | consistent |
| Unknown schema/toolchain, incomplete footprints, uncacheable markers, unavailable dependency artifacts, and externally attested/synthesized trusted data miss. | environment fail-closed helpers and non-reusable class checks. | environment fail-closed, non-trusted/synthesized metadata, and upstream completeness tests. | consistent |
| Diagnostic refs are deterministic miss/hit metadata only and do not change validation identity or proof acceptance. | `canonical_diagnostic_refs` and hit/miss payloads. | diagnostic-order and determinism-suite proof-reuse tests. | consistent |
| Public API excludes proof authority, publication tokens, scheduler hooks, IR adapters, and trace/reduction constructors. | public surface exposes validation request/outcome types only. | proof-reuse API and implementation boundary lint tests. | consistent |
| Public enums are forward-compatible. | all public enums are `#[non_exhaustive]`. | `public_cache_enums_are_forward_compatible_and_documented`. | guarded |

### `cluster_db`

Source path: `crates/mizar-cache/src/cluster_db.rs`.

Top-level public items:

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

Public methods include `ClusterDbIndex::{new, origin, apply_module_update,
remove_origin, snapshot, import_scoped_view}`.

Public inherent methods:

- `ClusterDbIndex::new`, `ClusterDbIndex::origin`,
  `ClusterDbIndex::apply_module_update`, `ClusterDbIndex::remove_origin`,
  `ClusterDbIndex::snapshot`, `ClusterDbIndex::import_scoped_view`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Only accepted importer-visible contribution records with complete, compatible metadata are written to the importer-visible index. | `validate_record`, origin canonicalization, accepted-only checks, and index insertion helpers. | accepted-contribution, non-visible/unaccepted, incomplete-origin, unknown-schema/toolchain tests. | consistent |
| Origin writes, replacements, stale-origin removal, aggregate rows, and update reports are deterministic and mutation-safe. | `apply_module_update`, `remove_origin`, `ClusterAggregateIndexes`, canonical row ordering. | stale-origin cleanup, rebuild-report, deterministic ordering, duplicate conflict, and coalescing tests. | consistent |
| Import-scoped views are filtered by explicit visible origin keys and fail closed for missing origins or incompatible request metadata. | `import_scoped_view`, visible-origin-set hash, request canonicalization and validation. | import-scoped filtering, unrelated-origin reuse, visible-origin invalidation, view fail-closed, and hidden-trace tests. | consistent |
| Unaccepted, recovered, externally attested, or inferred trace/reduction material is not inserted into visible indexes. | accepted-status and visibility validators; no trace constructor or reduction selector API exists. | accepted-only, hidden trace, API boundary, and implementation boundary lint tests. | consistent |
| Public API excludes proof authority, publication tokens, scheduler hooks, IR adapters, and trace/reduction constructors. | public surface exposes origin/index/view data structures only. | cluster-db API and implementation boundary lint tests. | consistent |
| Public enums are forward-compatible. | all public enums are `#[non_exhaustive]`. | `public_cache_enums_are_forward_compatible_and_documented`. | guarded |

## Cross-Module Evidence

| Contract | Source/test correspondence |
|---|---|
| Crate scaffolding and dependency boundary | `Cargo.toml`, `src/lib.rs`, and `tests/lint_policy.rs`; guarded by manifest, workspace, module-export, dependency, file-tree, and lint-baseline tests. |
| Cache data remains optimization-only | crate root boundary comments, API/implementation forbidden-term tests, and module specs all exclude proof-authority projection. |
| Deterministic canonicalization | module unit tests plus `tests/determinism_suite.rs` cover cache-key ordering, record deletion/availability, proof-reuse diagnostic ordering, and rejection of externally attested proof material. |
| Incremental verification fail-closed contract | `tests/incremental_contract.rs` exercises a complete trusted reuse path and independent misses for incomplete footprints, uncacheable markers, unknown schema/toolchain/policy, dependency artifact mismatch, proof metadata mismatches, deletion, diagnostic ordering, and externally attested evidence. |
| Downstream integration remains owner-gated | `integration_readiness.md` and lint tests prohibit scheduler hooks, `mizar-ir` placeholders, publication-token shortcuts, and trace/reduction constructors. |
| Public enum forward compatibility | source attributes, paired module policy tables, and `public_cache_enums_are_forward_compatible_and_documented`. |

## Guarded Test References

The source/spec lint guard verifies that these named tests still exist and remain
referenced by the EN/JA audits:

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

Task 20 introduces no new gap IDs. The audit repeats every existing
`deferred` or `external_dependency_gap` ID from the task ledger, reclassifies
`PROOFREUSE-G005` toward its external owners, and does not repair gaps with
placeholder source:

| ID | Class | Owner / trigger | Current handling |
|---|---|---|---|
| `CACHE-G-003` | `external_dependency_gap` | `mizar-build` scheduler seam | no scheduler hook or scheduling trait is added. |
| `CACHE-G-004` | `external_dependency_gap` | `mizar-ir` adapter | no placeholder crate, mock adapter, or rehydration API is added. |
| `CACHE-G-005` | `external_dependency_gap` | artifact/proof publication token | cache compares hashes and validation metadata only; publication remains artifact/proof owned. |
| `DEPFPR-G001` | `external_dependency_gap` | `mizar-build` dependency-fingerprint consumer | dependency fingerprints remain local cache inputs until the scheduler seam lands. |
| `DEPFPR-G002` | `external_dependency_gap` | `mizar-ir` adapter | no placeholder dependency-fingerprint adapter API is added. |
| `DEPFPR-G003` | `external_dependency_gap` | artifact committed publication token | cache records availability/hash inputs only. |
| `DEPFPR-G004` | `deferred` | producer-owned fine-grained theorem/definition/cluster/notation/mode/attribute slices | current dependency fingerprints use conservative published-summary and per-VC granularity. |
| `DEPFPR-G005` | `external_dependency_gap` | downstream proof/cache/artifact consumers | cache records proof-reuse validation identities only. |
| `CACHESTORE-G001` | `external_dependency_gap` | `mizar-build` scheduler seam | cache store APIs remain local and do not schedule work. |
| `CACHESTORE-G002` | `external_dependency_gap` | `mizar-ir` adapter | no record-payload adapter API is added. |
| `CACHESTORE-G003` | `external_dependency_gap` | artifact committed publication token | store checks local availability plus recorded domain/digest only. |
| `CACHESTORE-G004` | `deferred` | later cluster-db index storage | record store does not publish unaccepted registrations. |
| `PROOFREUSE-G001` | `external_dependency_gap` | `mizar-build` scheduler seam | proof-reuse validation is local and not scheduled. |
| `PROOFREUSE-G002` | `external_dependency_gap` | `mizar-ir` adapter | no IR placeholder API is created. |
| `PROOFREUSE-G003` | `external_dependency_gap` | artifact witness publication token | cache compares selected witness hashes only. |
| `PROOFREUSE-G004` | `external_dependency_gap` | artifact witness schema for a distinct trusted `DischargedBuiltin` class | proof reuse may validate deterministic discharge hashes but does not publish a trusted witness artifact for that class. |
| `PROOFREUSE-G005` | `external_dependency_gap` | `mizar-build` / `mizar-artifact` scheduler and publication integration | task 20 covers the crate-owned cache lookup and proof-reuse validation contract; cross-crate clean/incremental equivalence remains gated on scheduler and artifact publication integration. |
| `CLUSTERDB-G001` | `external_dependency_gap` | checker/artifact accepted-contribution producers | missing producer fields are deferred rather than fabricated. |
| `CLUSTERDB-G002` | `deferred` | persistent cluster-db storage task | task 13 implemented in-memory origins and aggregate indexes; durable `cluster-db/` files remain deferred. |
| `CLUSTERDB-G003` | `deferred` | persistent import-scoped view storage task | task 14 implemented in-memory views and invalidation; durable `views/` files remain deferred. |
| `CLUSTERDB-G004` | `external_dependency_gap` | `mizar-build` scheduler integration | no placeholder scheduler API is added. |
| `CLUSTERDB-G005` | `external_dependency_gap` | `mizar-ir` adapter integration | no placeholder `mizar-ir` API is added. |
| `CACHE15-G001` | `external_dependency_gap` | `mizar-build` cache-aware scheduler seam | `mizar-cache` exposes local cache APIs only; no scheduler hook or scheduling trait is added. |
| `CACHE15-G002` | `external_dependency_gap` | `mizar-ir` cache adapter | `mizar-ir` now exists and owns cache-adapter validation-before-rehydration boundaries, but end-to-end cache-record rehydration through build/driver execution is not wired; no placeholder mock adapter or rehydration shortcut is added. |
| `CACHE15-G003` | `external_dependency_gap` | `mizar-artifact` / `mizar-proof` committed witness publication token | cache validation compares hashes and metadata only; publication remains artifact/proof owned. |

No `repo_metadata_conflict` is observed during the current source/spec audit.

## Conclusion

The current public `mizar-cache` source surface matches the module
specifications and test coverage after task 22. The cache remains an internal optimization owner rather than proof authority. Cache records, dependency
fingerprints, proof-reuse validation metadata, backend diagnostics/logs,
timing metadata, and cluster-db indexes do not become kernel-verified proof
status or trusted `used_axioms`. Existing incomplete integration points are
classified above as `deferred` or `external_dependency_gap`.
