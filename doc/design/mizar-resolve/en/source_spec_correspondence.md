# Source/spec correspondence audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

Status: task R-027 audit complete; task R-029 refactor scope re-run complete;
2026-07-02 roadmap synchronization overlay complete; task R-024 implementation
overlay complete.

## Scope

The original R-027 audit checked the completed non-deferred `mizar-resolve`
tasks through R-026 against the English canonical resolver design specs:
[resolved_ast.md](./resolved_ast.md), [env.md](./env.md),
[imports.md](./imports.md), [declarations.md](./declarations.md),
[names.md](./names.md), [labels.md](./labels.md),
[symbols.md](./symbols.md), [recovery.md](./recovery.md), this crate plan, and
[todo.md](./todo.md). It traces public API families and implementation-facing
behavior promises to source and tests.

The R-029 re-run covers the same public API and behavior promises after the
behavior-preserving private helper/test split. It updates moved source paths and
task correspondence through R-029 without expanding resolver behavior.

At the original close-out, task R-024 was explicitly deferred as
`external_dependency_gap` and was audited only for its deferral record and for
the absence of a resolver-owned artifact schema or reader. The 2026-07-02
roadmap synchronization records that the artifact-side blocker is now resolved
by `mizar-artifact` task 5. The R-024 implementation overlay adds the
resolver-side consumer for canonical `mizar-artifact` `ModuleSummary` values
without adding resolver-owned artifact schemas, readers, writers, hash framing,
or source loading for artifact-only dependencies. This audit does not replace
executable tests, and it does not change `doc/spec`, existing `.miz` sources,
or expectation sidecars to match implementation behavior.

## Result

- No missing implementation was found for non-deferred public APIs and behavior
  contracts promised by the resolver module specs. R-029 moved private helpers
  and tests only; the moved-source re-run found no new mismatch through R-029.
- No resolver behavior was found that requires a new `doc/spec` change or a
  rebaseline of existing `.miz` tests/expectations.
- Source behavior that remains outside the current executable corpus is already
  classified below: public resolver diagnostic codes (`R-G001`),
  parser/syntax scheme declaration exposure (`R-G006`), and broader semantic
  `.miz` runner assertions. The broad historical corpus gap remains `R-G002`;
  its current concrete remainder is refined by `R-G007`. R-G003 is resolved by
  R-024.
- Boundary checks found no parser/syntax/frontend/session/build/checker/proof/
  artifact responsibility takeover. The resolver consumes the build-side module
  index seam and syntax `SurfaceAst`; it does not own source loading, module
  discovery, parser recovery, type inference, overload winner selection,
  proof semantics, or artifact persistence.
- The Japanese companion carries the same API families, source paths, behavior
  boundaries, and follow-up classifications. Broader wording/terminology/link
  synchronization is handled by task R-028.
- R-029 moved only private helper/test modules. Public API paths and behavior
  promises are unchanged; the affected source rows below now include the
  private helper paths introduced by the refactor gate.

## Public API Correspondence

| Spec | Public API checked | Source | Test evidence | Finding |
|---|---|---|---|---|
| [resolved_ast.md](./resolved_ast.md) stable identity and node arena | `ModuleId`, `LocalSymbolId`, `FullyQualifiedName`, `SymbolId`, `BuiltinId`, `LabelOriginPath`, resolver id wrappers, `SemanticOrigin`, `ResolvedNode`, `ResolvedArena`, `ResolvedArenaBuilder`, `ResolvedAst`, and validation error surfaces | `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/resolved_ast/validation.rs` | `module_and_symbol_ids_are_deterministic_and_alias_independent`, `arena_allocates_deterministic_ids_and_validates_children`, `arena_rejects_cycles`, `resolved_ast_validates_node_keys_and_preserves_traversal_states`, `resolved_ast_rejects_stale_keys_and_mismatched_modules` | No finding |
| [resolved_ast.md](./resolved_ast.md) name/label/import/export reference tables | `NameRefTable`, `LabelRefTable`, `ResolvedImports`, name/label/import/export resolution records, ambiguity/unresolved records, deferred selector records | `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/resolved_ast/validation.rs` | `name_ref_table_round_trips_all_current_result_kinds`, `ambiguous_name_candidates_tie_break_by_range_before_local_symbol_id`, `label_ref_table_round_trips_all_current_result_kinds`, `resolved_imports_round_trip_and_project_canonical_modules`, `table_and_import_export_iteration_is_stable`, `node_reference_keys_are_stable_for_equivalent_builds` | No finding |
| [resolved_ast.md](./resolved_ast.md) deterministic debug rendering | `ResolvedAst::snapshot_text` and stable variant-name rendering for resolver snapshot baselines | `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/resolved_ast/snapshot.rs`, crate-root determinism test | `resolved_ast_snapshot_text_is_stable_and_covers_tables`, `resolved_ast_snapshot_text_covers_payload_escaping_and_non_range_anchors`, `resolver_public_seams_are_deterministic_for_equivalent_inputs` | No finding |
| [env.md](./env.md) symbol environment indexes | `SymbolEnv`, `SymbolEnvIndexes`, symbol, label, definition, overload, registration, lexical-summary, namespace, declaration-dependency, module-summary, diagnostic-anchor, and source-contribution index families | `crates/mizar-resolve/src/env.rs` | `index_families_round_trip_insertions_and_lookups`, `index_iteration_is_deterministic_for_all_families`, `contribution_tracking_covers_sources_summaries_builtins_and_invalidation`, `equivalent_construction_is_stable_and_checker_facts_are_absent` | No finding |
| [env.md](./env.md) deterministic environment debug rendering | `SymbolEnv::snapshot_text` and sorted index/contribution sections | `crates/mizar-resolve/src/env.rs`, `crates/mizar-resolve/src/env/snapshot.rs`, crate-root determinism test | `symbol_env_snapshot_text_is_stable_and_covers_index_families`, `resolver_public_seams_are_deterministic_for_equivalent_inputs` | No finding |
| crate plan / R-007 module-index seam | `ModuleIndexInput`, `resolver_module_id`, `WorkspaceStubModuleIndexProvider`, and re-exported build-side provider/index types | `crates/mizar-resolve/src/module_index.rs`; build-side contract in `doc/design/mizar-build/en/module_index.md` | `stub_provider_feeds_multi_module_fixture`, `forwarded_packages_preserve_provider_order_and_namespaces_are_canonical`, `module_identity_is_alias_independent`, `provider_errors_are_deterministic` | No finding |
| [imports.md](./imports.md) import path and alias resolution | `ImportPathCandidate`, `ResolvedImportCandidate`, `UnresolvedImportCandidate`, `ImportPathResolution`, `ImportPathResolver`, `ModuleImportCandidates`, `ImportEdgeCandidate`, `ImportPathPrefix`, `ImportPathFailureClass` | `crates/mizar-resolve/src/imports.rs` | `aliases_do_not_change_canonical_targets_or_graph_candidates`, `relative_prefixes_use_dot_separated_module_directories`, `namespace_bindings_win_over_package_local_fallback`, `duplicate_aliases_and_reserved_aliases_are_unresolved_deterministically`, `unknown_modules_are_rejected_before_graph_publication`, `unresolved_imports_do_not_abort_later_candidates` | No finding |
| [imports.md](./imports.md) semantic import graph and cycle rejection | `ImportGraphBuilder`, `ImportGraphResolution`, `ImportGraph`, `ImportGraphEdge`, `ImportCycle`, `ImportGraphBuildError` | `crates/mizar-resolve/src/imports.rs` | `acyclic_fixture_builds_expected_graph_and_dependency_first_order`, `cycle_fixture_is_rejected_deterministically`, `self_cycle_is_rejected_deterministically`, `independent_acyclic_components_use_canonical_ready_ties`, `independent_cycles_sort_by_source_provenance` | No finding |
| [declarations.md](./declarations.md) declaration shells and export projections | `DeclarationShellSet`, `DeclarationShell`, `DeclarationShellKind`, `DeclarationShellVisibility`, `ExportPathShell`, `ExportProjectionShell`, `DeclarationShellCollector` | `crates/mizar-resolve/src/declarations.rs` | `collector_records_represented_declaration_kinds_in_source_order`, `annotation_wrappers_are_transparent_for_shell_collection`, `excluded_context_body_statement_and_recovery_nodes_do_not_create_shells`, `malformed_export_projection_is_retained_without_target_validation` | No finding |
| [declarations.md](./declarations.md), [recovery.md](./recovery.md) recovered declaration policy | recovered-shell markers, transparent wrapper recovery, and shell-only retention without symbol fabrication | `crates/mizar-resolve/src/declarations.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_subtrees_are_retained_and_marked_recovered` | No finding |
| [names.md](./names.md) namespace resolution | namespace path candidates/results, partial candidates, import dependencies, namespace roots, candidate targets, and `NamespaceResolver` | `crates/mizar-resolve/src/names.rs` | `resolver_resolves_alias_roots_and_package_names_deterministically`, `longest_namespace_bindings_win_over_shorter_prefixes`, `qualified_lookup_restricts_namespace_and_visibility`, `missing_namespace_records_the_earliest_failing_segment_range`, `malformed_namespace_paths_are_unresolved_in_deterministic_order`, `stale_namespace_bindings_are_provider_errors`, `stale_empty_prefix_reserved_root_bindings_report_the_root_segment` | No finding |
| [names.md](./names.md) preliminary symbol-name resolution and internal diagnostics | name projections, built-in projections, reference candidates, `SymbolNameResolver`, `NameDiagnosticCollector`, `NameDiagnosticReport`, diagnostic roots/cascades | `crates/mizar-resolve/src/names.rs`, `crates/mizar-resolve/src/names/diagnostics.rs` | `unqualified_lookup_uses_declaration_point_shadowing_and_builtins`, `duplicate_import_aliases_drive_ambiguous_namespace_payloads_deterministically`, `unresolved_import_dependency_produces_one_primary_name_diagnostic`, `name_diagnostics_preserve_ambiguous_candidate_order`, `name_diagnostics_order_same_range_by_class_spelling_and_candidate_key`, `name_diagnostics_use_mixed_root_ordering`, `recovered_inputs_do_not_emit_name_diagnostic_roots` | No finding |
| [names.md](./names.md) dot-chain finalization | local term scopes/bindings, dot-chain candidates, `DotChainFinalizer`, namespace-vs-selector handoff, `DeferredSelector` results | `crates/mizar-resolve/src/names.rs`, `crates/mizar-resolve/src/resolved_ast.rs` | `dot_chain_uses_innermost_visible_local_binding`, `dot_chain_local_binding_defers_selector_without_namespace_lookup`, `dot_chain_without_visible_local_resolves_namespace_symbol`, `dot_chain_unresolved_namespace_uses_earliest_failed_segment`, `dot_chain_malformed_or_recovered_inputs_stay_unresolved`, `dot_chain_finalizer_orders_out_of_order_inputs` | No finding |
| [labels.md](./labels.md) label projection and citation resolution | label scopes, projections, reference candidates, diagnostics, result tables, `LabelResolver` | `crates/mizar-resolve/src/labels.rs`, `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/env.rs` | `unqualified_citation_respects_proof_block_visibility_and_confinement`, `duplicate_and_visible_nested_labels_are_internal_diagnostics`, `forward_references_to_later_theorem_labels_are_unresolved`, `forward_references_to_later_proof_step_labels_are_unresolved`, `qualified_and_lowered_grouped_item_citations_use_module_label_projections`, `ambiguous_cross_family_citations_keep_sorted_candidates`, `label_index_and_reference_table_order_are_deterministic`, `imported_local_only_labels_are_not_visible_to_citations` | No finding |
| [labels.md](./labels.md), [recovery.md](./recovery.md) recovered label policy | recovered/failed namespace references remain unresolved and recovered label projections do not emit conflict diagnostics | `crates/mizar-resolve/src/labels.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_empty_and_failed_namespace_references_are_unresolved`, `recovered_label_projections_do_not_emit_conflict_diagnostics` | No finding |
| [symbols.md](./symbols.md) declaration-symbol projection and collection | `SymbolDeclarationProjection`, `SignatureProjectionExtractor`, `SymbolCollector`, `SymbolCollectionResult`, `SymbolDiagnostic`, `SymbolDiagnosticClass`, overload policy, parser-backed signature shells | `crates/mizar-resolve/src/symbols.rs`, `crates/mizar-resolve/src/env.rs`, `crates/mizar-resolve/src/declarations.rs` | `registers_opaque_symbols_definitions_and_contribution_effects`, `duplicate_detection_marks_represented_kind_families_in_order`, `overloadable_candidates_form_groups_and_illegal_groups_get_diagnostics`, `registration_projection_populates_symbol_definition_and_registration_indexes`, `symbol_identity_includes_namespace_notation_arity_and_explicit_slot`, `parser_backed_extractor_projects_represented_signature_families` | No finding |
| [symbols.md](./symbols.md), [recovery.md](./recovery.md) recovered and context-only symbol policy | recovered projections remain local/malformed, context-only shells do not fabricate symbols, recovered diagnostics do not cascade | `crates/mizar-resolve/src/symbols.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_shells_stay_local_and_malformed_without_panicking`, `recovered_symbols_do_not_cascade_duplicate_or_overload_diagnostics`, `recovered_context_only_shells_do_not_emit_context_diagnostics`, `context_parent_visibility_and_recovery_propagate_to_child_symbols`, `context_only_shells_do_not_fabricate_symbol_identities`, `parser_backed_recovered_projection_uses_malformed_signature` | No finding |
| [module_summary_reuse.md](./module_summary_reuse.md) canonical summary reuse | `ModuleSummaryReuseRequest`, `ModuleSummaryReuse`, `ModuleSummaryReuseResult`, `ModuleSummaryReuseDiagnostic`, `ModuleSummaryReuseReason`, reader-backed and already-validated projection paths | `crates/mizar-resolve/src/module_summary_reuse.rs`, `crates/mizar-resolve/src/lib.rs`, `crates/mizar-resolve/Cargo.toml` | `summary_backed_projection_matches_source_backed_exports`, `summary_backed_symbol_surface_matches_source_collector`, `lockfile_identity_is_accepted_when_known_identity_fields_match`, `identity_and_expected_hash_mismatch_fall_back`, `unknown_symbol_visibility_fails_closed`, `unknown_label_visibility_and_target_kind_fail_closed`, `missing_dependency_summary_does_not_source_load` | No finding |
| [symbols.md](./symbols.md), [mizar-test staged model](../../mizar-test/en/staged_model.md) declaration-symbol runner | active `declaration_symbol` corpus stage, internal detail-key expectation matching without public resolver diagnostic codes, plus exact SymbolEnv-derived pass payload assertions for represented kind, visibility, and export status | `crates/mizar-test/src/runner.rs`, `tests/miz/pass/resolve/pass_resolve_declaration_symbol_smoke_001.*`, `tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.*`, `tests/miz/fail/resolve/fail_resolve_same_signature_return_conflict_001.*`, `tests/coverage/spec_trace.toml` | `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`; `cargo run -p mizar-test -- declaration-symbol --tests-root tests --manifest tests/coverage/spec_trace.toml`; `cargo test -p mizar-test` in R-023 and post-task-20 R-G007 increments; active expectation sidecars tagged `active_declaration_symbol` | No finding for the runner; broader import/name/dot-chain/label corpus assertions remain `R-G007` |
| [todo.md](./todo.md) lint, deterministic hardening, and enum policy | workspace lint opt-in, documented `allow` rationale guard, deterministic public-seam regression, public enum `#[non_exhaustive]` and owning-spec decision table | `crates/mizar-resolve/tests/lint_policy.rs`, `crates/mizar-resolve/src/lib.rs`, all public resolver enum owners | lint-policy tests; `resolver_public_seams_are_deterministic_for_equivalent_inputs`; `public_resolver_enums_are_marked_non_exhaustive_and_documented` | No finding |

## Behavior Boundary Trace

| Boundary | Audit result |
|---|---|
| Parser/syntax/frontend boundary | Resolver source consumes `SurfaceAst`, `SurfaceNodeView`, syntax recovery markers, and frontend-produced lexical/module surfaces. It does not add syntax vocabulary, parser recovery, tokenization, or frontend orchestration behavior. |
| Build/session boundary | `module_index.rs` consumes the build-side `ModuleIndexProvider` contract and preserves alias-independent `ModuleId` construction. It does not parse manifests, discover modules, load sources, or own build planning. |
| Checker/type/proof boundary | Name and dot-chain resolution records unresolved, ambiguous, overload-group, and deferred-selector states. It does not perform type-directed overload winner selection, selector type checking, cluster firing, proof checking, obligation generation, or VC production. |
| Diagnostics boundary | Resolver diagnostics remain crate-local/internal while `R-G001` is open. R-023 declaration-symbol expectations compare internal detail keys in payload metadata and keep public `diagnostic_codes` empty. |
| Artifact boundary | R-024 consumes canonical `mizar-artifact` `ModuleSummary` values through artifact-owned reader/hash validation and maps the validated public surface into resolver indexes. Source still contains no resolver-owned `ModuleSummary` schema, artifact writer, hash framing, manifest/store I/O, or source loading for artifact-only dependency modules. |
| Determinism boundary | Module-local tests plus the R-025 public-seam regression cover deterministic ids, table ordering, graph ordering, diagnostic ordering, and debug rendering. |

## Task Requirement Correspondence

| Task group | Source/test correspondence |
|---|---|
| R-001 crate scaffold and lint policy | Workspace member and lint policy are implemented in `crates/mizar-resolve/Cargo.toml` and `tests/lint_policy.rs`; lint tests cover workspace lint opt-in, warning/clippy denial baseline, documented `allow` rationale, and the R-026 enum decision guard. |
| R-002 to R-006 data shapes and debug rendering | `resolved_ast.md` and `env.md` are implemented by `src/resolved_ast.rs` and `src/env.rs`; unit tests cover ids, tables, validation, deterministic ordering, contribution tracking, checker-fact absence, and stable snapshot text. |
| R-007 module-index seam | `src/module_index.rs` wraps the build-side provider contract without build planning. Tests cover stub provider behavior, provider ordering, alias-independent identities, and deterministic provider errors. |
| R-008 to R-010 imports | `imports.md` is implemented by `src/imports.rs`; unit tests cover semantic path resolution, aliases, relative prefixes, unresolved recovery, graph construction, topological order, and cycle rejection. |
| R-011 declarations | `declarations.md` is implemented by `src/declarations.rs`; unit tests cover represented shell kinds, visibility wrappers, export projections, transparency/exclusion, recovery, and source-order determinism. |
| R-012 to R-016 names | `names.md` is implemented by `src/names.rs`; unit tests cover namespace lookup, declaration-point filtering, visibility/shadowing, unresolved/ambiguous representation, internal diagnostic ordering/cascade suppression, and dot-chain finalization without checker-owned selector validation. |
| R-017 to R-018 labels | `labels.md` is implemented by `src/labels.rs`; unit tests cover theorem/lemma and proof-step label scopes, forward-reference rejection, qualified/imported citation lookup, diagnostics, recovery, and deterministic tables. |
| R-019 to R-023 symbols and corpus runner | `symbols.md` is implemented by `src/symbols.rs`; unit tests cover opaque and parser-backed signatures, duplicates/conflicts, overload grouping, registrations, recovery, context-only shells, and deterministic diagnostics. R-023 adds active declaration-symbol pass/fail corpus seeds and traceability metadata. |
| R-024 ModuleSummary reuse | Implemented by `src/module_summary_reuse.rs` against canonical `mizar-artifact` summaries. Tests cover source-backed agreement, deterministic reuse/fallback, known-field identity validation including lockfile identities, unsupported projection fail-closed behavior, and absence of source loading for missing artifact summaries. |
| R-025 determinism suite | `src/lib.rs` contains the public-seam determinism regression over import graphs, name diagnostics, `ResolvedAst` snapshots, and `SymbolEnv` snapshots, complementing module-local determinism tests. |
| R-026 public enum policy | Module specs list every resolver-owned public enum decision; source attributes mark all listed enums `#[non_exhaustive]`; `tests/lint_policy.rs` guards source/spec drift for the spec-owned modules. |
| R-027 source/spec audit | This document records the correspondence. The audit found no unclassified blocking/high `spec_gap`, `test_gap`, `source_drift`, `source_undocumented_behavior`, `test_expectation_drift`, `boundary_violation`, or `repo_metadata_conflict`. |
| R-028 bilingual documentation sync audit | Bilingual design sync is recorded in `bilingual_documentation_synchronization.md`; no public source behavior changed. |
| R-029 module-boundary refactor gate | Private helper/test modules were split as recorded in `module_boundary_refactor.md`; this source/spec scope was re-run for moved APIs and found no public API, behavior, diagnostic, rendering, artifact, or boundary drift. |

## Follow-up Records

This audit did not add a new blocking follow-up. Existing classified records
after R-024 are:

| ID | Classification | Follow-up | Status |
|---|---|---|---|
| R-G001 | `spec_gap` refined to `external_dependency_gap` / deferred adoption | Public resolver diagnostic descriptors and eventual `mizar-diagnostics` adoption. The shared registry reserves a broad `Resolution` family, but resolver name/import/label descriptors are not adopted. | Deferred to R-030. Current resolver diagnostics stay crate-local/internal; do not add public numeric codes, aliases, or placeholder adapters until a real adoption task aligns registry/spec ownership and coverage. |
| R-G002 | `test_gap` | Historical lack of semantic resolver corpus coverage beyond lexical/parser import/export syntax. | Partially closed by R-023's active declaration-symbol smoke/fail fixtures, the post-task-20 R-G007 parser-backed signature-conflict active seed, and exact SymbolEnv-derived pass payload assertions. The remaining concrete corpus assertion work is refined by R-G007 and remains non-blocking for R-027 because unit tests cover the implemented behavior. |
| R-G003 | resolved by R-024 | Consume dependency modules from canonical `ModuleSummary` artifacts. | Completed in resolver as canonical `mizar-artifact` summary consumption without resolver-owned artifact schemas, shims, writers, hash framing, or source loading. |
| R-G006 | `external_dependency_gap` | Module-level scheme/template declaration shell once parser/syntax exposes an owning source role. | Non-blocking for represented source roles. Current resolver preserves direct template roles in owning signature payloads and does not fabricate scheme/template module symbols. |
| R-G007 | `test_gap` | Concrete remainder of R-G002 after the active signature-conflict and pass-payload increments: broader active semantic `.miz` assertions for import graph, namespace/name resolution, dot-chain, and label-reference facts from tasks R-009 to R-019. | Non-blocking for R-027 because unit tests cover the behavior and R-023 installed the declaration-symbol runner plus initial traceable active set. Grow remaining coverage in future runner assertion expansions without inventing behavior beyond `doc/spec/en`. |
