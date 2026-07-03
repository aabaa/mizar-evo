# Source/spec 対応監査

> 正本は英語です。英語版:
> [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

状態: task R-027 audit complete; task R-029 refactor scope re-run complete;
2026-07-02 roadmap synchronization overlay complete; task R-024 implementation
overlay complete。

## 範囲

元の R-027 監査は、R-026 までの non-deferred な `mizar-resolve` task 実装を
英語正本の resolver design spec、すなわち [resolved_ast.md](./resolved_ast.md)、
[env.md](./env.md)、[imports.md](./imports.md)、
[declarations.md](./declarations.md)、[names.md](./names.md)、
[labels.md](./labels.md)、[symbols.md](./symbols.md)、
[recovery.md](./recovery.md)、crate plan、本 [todo.md](./todo.md) と照合する。
公開 API family と implementation-facing behavior promise を source と test に
trace する。

R-029 の再実行は、behavior-preserving な private helper / test split 後に同じ public
API と behavior promise を確認する。移動した source path と task 対応を R-029 まで
更新するが、resolver behavior は拡張しない。

当初の close-out 時点では、task R-024 は `external_dependency_gap` として明示的に
deferred であり、deferral record と resolver-owned artifact schema / reader が存在しないこと
だけを監査していた。2026-07-02 の roadmap synchronization は、artifact 側 blocker が
`mizar-artifact` task 5 により解消済みであることを記録する。R-024 implementation overlay は、
canonical な `mizar-artifact` `ModuleSummary` value の resolver-side consumer を追加し、
resolver-owned artifact schema、reader、writer、hash framing、artifact-only dependency の
source loading は追加しない。
この監査は executable test を置き換えない。また、実装に合わせる目的で `doc/spec`、
既存 `.miz` source、expectation sidecar を変更しない。

## 結果

- non-deferred な公開 API と behavior contract について、missing implementation は
  見つからなかった。R-029 は private helper と test だけを移動しており、moved-source
  re-run では R-029 までの新しい不一致は見つからなかった。
- 新しい `doc/spec` 変更や既存 `.miz` test / expectation の rebaseline を必要とする
  resolver behavior は見つからなかった。
- 現在の executable corpus 外に残る source behavior はすでに分類済みである:
  public resolver diagnostic code（`R-G001`）、parser/syntax の
  scheme declaration exposure（`R-G006`）、より広い semantic `.miz` runner
  assertion である。広い履歴的 corpus gap は `R-G002` として残り、現在の具体的な
  残りは `R-G007` が精緻化する。R-G003 は R-024 で解消済みである。
- boundary check では parser / syntax / frontend / session / build / checker / proof /
  artifact の責務取り込みは見つからなかった。resolver は build-side module-index seam と
  syntax `SurfaceAst` を消費するが、source loading、module discovery、parser recovery、
  type inference、overload winner selection、proof semantics、artifact persistence は
  所有しない。
- この日本語 companion は、同じ API family、source path、behavior boundary、
  follow-up classification を保持する。より広い wording / terminology / link の同期は
  task R-028 が扱う。
- R-029 は private helper / test module だけを移動した。public API path と behavior
  promise は変更していない。下の該当 source 行は refactor gate で追加された private
  helper path も含む。

## 公開 API 対応

| Spec | 確認した公開 API | Source | Test evidence | Finding |
|---|---|---|---|---|
| [resolved_ast.md](./resolved_ast.md) stable identity と node arena | `ModuleId`, `LocalSymbolId`, `FullyQualifiedName`, `SymbolId`, `BuiltinId`, `LabelOriginPath`, resolver id wrappers, `SemanticOrigin`, `ResolvedNode`, `ResolvedArena`, `ResolvedArenaBuilder`, `ResolvedAst`, validation error surface | `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/resolved_ast/validation.rs` | `module_and_symbol_ids_are_deterministic_and_alias_independent`, `arena_allocates_deterministic_ids_and_validates_children`, `arena_rejects_cycles`, `resolved_ast_validates_node_keys_and_preserves_traversal_states`, `resolved_ast_rejects_stale_keys_and_mismatched_modules` | No finding |
| [resolved_ast.md](./resolved_ast.md) name / label / import / export reference table | `NameRefTable`, `LabelRefTable`, `ResolvedImports`, name/label/import/export resolution record, ambiguity/unresolved record, deferred selector record | `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/resolved_ast/validation.rs` | `name_ref_table_round_trips_all_current_result_kinds`, `ambiguous_name_candidates_tie_break_by_range_before_local_symbol_id`, `label_ref_table_round_trips_all_current_result_kinds`, `resolved_imports_round_trip_and_project_canonical_modules`, `table_and_import_export_iteration_is_stable`, `node_reference_keys_are_stable_for_equivalent_builds` | No finding |
| [resolved_ast.md](./resolved_ast.md) deterministic debug rendering | `ResolvedAst::snapshot_text` と resolver snapshot baseline 用の stable variant-name rendering | `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/resolved_ast/snapshot.rs`, crate-root determinism test | `resolved_ast_snapshot_text_is_stable_and_covers_tables`, `resolved_ast_snapshot_text_covers_payload_escaping_and_non_range_anchors`, `resolver_public_seams_are_deterministic_for_equivalent_inputs` | No finding |
| [env.md](./env.md) symbol environment index | `SymbolEnv`, `SymbolEnvIndexes`, symbol, label, definition, overload, registration, lexical-summary, namespace, declaration-dependency, module-summary, diagnostic-anchor, source-contribution index family | `crates/mizar-resolve/src/env.rs` | `index_families_round_trip_insertions_and_lookups`, `index_iteration_is_deterministic_for_all_families`, `contribution_tracking_covers_sources_summaries_builtins_and_invalidation`, `equivalent_construction_is_stable_and_checker_facts_are_absent` | No finding |
| [env.md](./env.md) deterministic environment debug rendering | `SymbolEnv::snapshot_text` と sorted index/contribution section | `crates/mizar-resolve/src/env.rs`, `crates/mizar-resolve/src/env/snapshot.rs`, crate-root determinism test | `symbol_env_snapshot_text_is_stable_and_covers_index_families`, `resolver_public_seams_are_deterministic_for_equivalent_inputs` | No finding |
| crate plan / R-007 module-index seam | `ModuleIndexInput`, `resolver_module_id`, `WorkspaceStubModuleIndexProvider`, re-exported build-side provider/index types | `crates/mizar-resolve/src/module_index.rs`; build-side contract は `doc/design/mizar-build/en/module_index.md` | `stub_provider_feeds_multi_module_fixture`, `forwarded_packages_preserve_provider_order_and_namespaces_are_canonical`, `module_identity_is_alias_independent`, `provider_errors_are_deterministic` | No finding |
| [imports.md](./imports.md) import path と alias resolution | `ImportPathCandidate`, `ResolvedImportCandidate`, `UnresolvedImportCandidate`, `ImportPathResolution`, `ImportPathResolver`, `ModuleImportCandidates`, `ImportEdgeCandidate`, `ImportPathPrefix`, `ImportPathFailureClass` | `crates/mizar-resolve/src/imports.rs` | `aliases_do_not_change_canonical_targets_or_graph_candidates`, `relative_prefixes_use_dot_separated_module_directories`, `namespace_bindings_win_over_package_local_fallback`, `duplicate_aliases_and_reserved_aliases_are_unresolved_deterministically`, `unknown_modules_are_rejected_before_graph_publication`, `unresolved_imports_do_not_abort_later_candidates` | No finding |
| [imports.md](./imports.md) semantic import graph と cycle rejection | `ImportGraphBuilder`, `ImportGraphResolution`, `ImportGraph`, `ImportGraphEdge`, `ImportCycle`, `ImportGraphBuildError` | `crates/mizar-resolve/src/imports.rs` | `acyclic_fixture_builds_expected_graph_and_dependency_first_order`, `cycle_fixture_is_rejected_deterministically`, `self_cycle_is_rejected_deterministically`, `independent_acyclic_components_use_canonical_ready_ties`, `independent_cycles_sort_by_source_provenance` | No finding |
| [declarations.md](./declarations.md) declaration shell と export projection | `DeclarationShellSet`, `DeclarationShell`, `DeclarationShellKind`, `DeclarationShellVisibility`, `ExportPathShell`, `ExportProjectionShell`, `DeclarationShellCollector` | `crates/mizar-resolve/src/declarations.rs` | `collector_records_represented_declaration_kinds_in_source_order`, `annotation_wrappers_are_transparent_for_shell_collection`, `excluded_context_body_statement_and_recovery_nodes_do_not_create_shells`, `malformed_export_projection_is_retained_without_target_validation` | No finding |
| [declarations.md](./declarations.md), [recovery.md](./recovery.md) recovered declaration policy | recovered-shell marker、transparent wrapper recovery、symbol fabrication なしの shell-only retention | `crates/mizar-resolve/src/declarations.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_subtrees_are_retained_and_marked_recovered` | No finding |
| [names.md](./names.md) namespace resolution | namespace path candidate/result、partial candidate、import dependency、namespace root、candidate target、`NamespaceResolver` | `crates/mizar-resolve/src/names.rs` | `resolver_resolves_alias_roots_and_package_names_deterministically`, `longest_namespace_bindings_win_over_shorter_prefixes`, `qualified_lookup_restricts_namespace_and_visibility`, `missing_namespace_records_the_earliest_failing_segment_range`, `malformed_namespace_paths_are_unresolved_in_deterministic_order`, `stale_namespace_bindings_are_provider_errors`, `stale_empty_prefix_reserved_root_bindings_report_the_root_segment` | No finding |
| [names.md](./names.md) preliminary symbol-name resolution と internal diagnostics | name projection、built-in projection、reference candidate、`SymbolNameResolver`、`NameDiagnosticCollector`、`NameDiagnosticReport`、diagnostic root/cascade | `crates/mizar-resolve/src/names.rs`, `crates/mizar-resolve/src/names/diagnostics.rs` | `unqualified_lookup_uses_declaration_point_shadowing_and_builtins`, `duplicate_import_aliases_drive_ambiguous_namespace_payloads_deterministically`, `unresolved_import_dependency_produces_one_primary_name_diagnostic`, `name_diagnostics_preserve_ambiguous_candidate_order`, `name_diagnostics_order_same_range_by_class_spelling_and_candidate_key`, `name_diagnostics_use_mixed_root_ordering`, `recovered_inputs_do_not_emit_name_diagnostic_roots` | No finding |
| [names.md](./names.md) dot-chain finalization | local term scope/binding、dot-chain candidate、`DotChainFinalizer`、namespace-vs-selector handoff、`DeferredSelector` result | `crates/mizar-resolve/src/names.rs`, `crates/mizar-resolve/src/resolved_ast.rs` | `dot_chain_uses_innermost_visible_local_binding`, `dot_chain_local_binding_defers_selector_without_namespace_lookup`, `dot_chain_without_visible_local_resolves_namespace_symbol`, `dot_chain_unresolved_namespace_uses_earliest_failed_segment`, `dot_chain_malformed_or_recovered_inputs_stay_unresolved`, `dot_chain_finalizer_orders_out_of_order_inputs` | No finding |
| [labels.md](./labels.md) label projection と citation resolution | label scope、projection、reference candidate、diagnostic、result table、`LabelResolver` | `crates/mizar-resolve/src/labels.rs`, `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/env.rs` | `unqualified_citation_respects_proof_block_visibility_and_confinement`, `duplicate_and_visible_nested_labels_are_internal_diagnostics`, `forward_references_to_later_theorem_labels_are_unresolved`, `forward_references_to_later_proof_step_labels_are_unresolved`, `qualified_and_lowered_grouped_item_citations_use_module_label_projections`, `ambiguous_cross_family_citations_keep_sorted_candidates`, `label_index_and_reference_table_order_are_deterministic`, `imported_local_only_labels_are_not_visible_to_citations` | No finding |
| [labels.md](./labels.md), [recovery.md](./recovery.md) recovered label policy | recovered / failed namespace reference は unresolved に残り、recovered label projection は conflict diagnostics を出さない | `crates/mizar-resolve/src/labels.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_empty_and_failed_namespace_references_are_unresolved`, `recovered_label_projections_do_not_emit_conflict_diagnostics` | No finding |
| [symbols.md](./symbols.md) declaration-symbol projection と collection | `SymbolDeclarationProjection`, `SignatureProjectionExtractor`, `SymbolCollector`, `SymbolCollectionResult`, `SymbolDiagnostic`, `SymbolDiagnosticClass`, overload policy, parser-backed signature shell | `crates/mizar-resolve/src/symbols.rs`, `crates/mizar-resolve/src/env.rs`, `crates/mizar-resolve/src/declarations.rs` | `registers_opaque_symbols_definitions_and_contribution_effects`, `duplicate_detection_marks_represented_kind_families_in_order`, `overloadable_candidates_form_groups_and_illegal_groups_get_diagnostics`, `registration_projection_populates_symbol_definition_and_registration_indexes`, `symbol_identity_includes_namespace_notation_arity_and_explicit_slot`, `parser_backed_extractor_projects_represented_signature_families` | No finding |
| [symbols.md](./symbols.md), [recovery.md](./recovery.md) recovered / context-only symbol policy | recovered projection は local/malformed に残り、context-only shell は symbol を創作せず、recovered diagnostics は cascade しない | `crates/mizar-resolve/src/symbols.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_shells_stay_local_and_malformed_without_panicking`, `recovered_symbols_do_not_cascade_duplicate_or_overload_diagnostics`, `recovered_context_only_shells_do_not_emit_context_diagnostics`, `context_parent_visibility_and_recovery_propagate_to_child_symbols`, `context_only_shells_do_not_fabricate_symbol_identities`, `parser_backed_recovered_projection_uses_malformed_signature` | No finding |
| [module_summary_reuse.md](./module_summary_reuse.md) canonical summary reuse | `ModuleSummaryReuseRequest`, `ModuleSummaryReuse`, `ModuleSummaryReuseResult`, `ModuleSummaryReuseDiagnostic`, `ModuleSummaryReuseReason`, reader-backed / already-validated projection path | `crates/mizar-resolve/src/module_summary_reuse.rs`, `crates/mizar-resolve/src/lib.rs`, `crates/mizar-resolve/Cargo.toml` | `summary_backed_projection_matches_source_backed_exports`, `summary_backed_symbol_surface_matches_source_collector`, `lockfile_identity_is_accepted_when_known_identity_fields_match`, `identity_and_expected_hash_mismatch_fall_back`, `unknown_symbol_visibility_fails_closed`, `unknown_label_visibility_and_target_kind_fail_closed`, `missing_dependency_summary_does_not_source_load` | No finding |
| [symbols.md](./symbols.md), [mizar-test staged model](../../mizar-test/ja/staged_model.md) declaration-symbol runner | active `declaration_symbol` corpus stage、public resolver diagnostic code を作らず internal detail-key expectation を比較する経路 | `crates/mizar-test/src/runner.rs`, `tests/miz/pass/resolve/pass_resolve_declaration_symbol_smoke_001.*`, `tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.*`, `tests/miz/fail/resolve/fail_resolve_same_signature_return_conflict_001.*`, `tests/coverage/spec_trace.toml` | `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`; R-023 と post-task-20 R-G007 signature-conflict increment の `cargo test -p mizar-test`; `active_declaration_symbol` tag 付き expectation sidecar | runner に finding はない。より広い import/name/dot-chain/label corpus assertion は `R-G007` |
| [todo.md](./todo.md) lint、deterministic hardening、enum policy | workspace lint opt-in、documented `allow` rationale guard、deterministic public-seam regression、public enum `#[non_exhaustive]` と owning-spec decision table | `crates/mizar-resolve/tests/lint_policy.rs`, `crates/mizar-resolve/src/lib.rs`, all public resolver enum owners | lint-policy tests; `resolver_public_seams_are_deterministic_for_equivalent_inputs`; `public_resolver_enums_are_marked_non_exhaustive_and_documented` | No finding |

## 挙動境界 trace

| Boundary | Audit result |
|---|---|
| Parser / syntax / frontend boundary | resolver source は `SurfaceAst`、`SurfaceNodeView`、syntax recovery marker、frontend が生成した lexical/module surface を消費する。syntax vocabulary、parser recovery、tokenization、frontend orchestration behavior は追加しない。 |
| Build / session boundary | `module_index.rs` は build-side `ModuleIndexProvider` contract を消費し、alias-independent `ModuleId` construction を保つ。manifest parsing、module discovery、source loading、build planning は所有しない。 |
| Checker / type / proof boundary | name と dot-chain resolution は unresolved、ambiguous、overload-group、deferred-selector state を記録する。type-directed overload winner selection、selector type checking、cluster firing、proof checking、obligation generation、VC production は行わない。 |
| Diagnostics boundary | `R-G001` が open の間、resolver diagnostics は crate-local/internal のままである。R-023 declaration-symbol expectation は payload metadata の internal detail key を比較し、public `diagnostic_codes` を空に保つ。 |
| Artifact boundary | R-024 は canonical な `mizar-artifact` `ModuleSummary` value を artifact-owned reader / hash validation 経由で消費し、検証済み public surface を resolver index へ map する。source には resolver-owned `ModuleSummary` schema、artifact writer、hash framing、manifest/store I/O、artifact-only dependency module の source loading は存在しない。 |
| Determinism boundary | module-local tests と R-025 public-seam regression が deterministic id、table ordering、graph ordering、diagnostic ordering、debug rendering を cover する。 |

## task requirement 対応

| Task group | Source/test correspondence |
|---|---|
| R-001 crate scaffold and lint policy | workspace member と lint policy は `crates/mizar-resolve/Cargo.toml` と `tests/lint_policy.rs` に実装済み。lint tests は workspace lint opt-in、warning/clippy denial baseline、documented `allow` rationale、R-026 enum decision guard を cover する。 |
| R-002 to R-006 data shapes and debug rendering | `resolved_ast.md` と `env.md` は `src/resolved_ast.rs` と `src/env.rs` に実装済み。unit tests は id、table、validation、deterministic ordering、contribution tracking、checker-fact absence、stable snapshot text を cover する。 |
| R-007 module-index seam | `src/module_index.rs` は build-side provider contract を build planning なしで wrap する。tests は stub provider behavior、provider ordering、alias-independent identity、deterministic provider error を cover する。 |
| R-008 to R-010 imports | `imports.md` は `src/imports.rs` に実装済み。unit tests は semantic path resolution、alias、relative prefix、unresolved recovery、graph construction、topological order、cycle rejection を cover する。 |
| R-011 declarations | `declarations.md` は `src/declarations.rs` に実装済み。unit tests は represented shell kind、visibility wrapper、export projection、transparent/excluded node、recovery、source-order determinism を cover する。 |
| R-012 to R-016 names | `names.md` は `src/names.rs` に実装済み。unit tests は namespace lookup、declaration-point filtering、visibility/shadowing、unresolved/ambiguous representation、internal diagnostic ordering/cascade suppression、checker-owned selector validation なしの dot-chain finalization を cover する。 |
| R-017 to R-018 labels | `labels.md` は `src/labels.rs` に実装済み。unit tests は theorem/lemma と proof-step label scope、forward-reference rejection、qualified/imported citation lookup、diagnostics、recovery、deterministic table を cover する。 |
| R-019 to R-023 symbols and corpus runner | `symbols.md` は `src/symbols.rs` に実装済み。unit tests は opaque / parser-backed signature、duplicate/conflict、overload grouping、registration、recovery、context-only shell、deterministic diagnostics を cover する。R-023 は active declaration-symbol pass/fail corpus seed と traceability metadata を追加済み。 |
| R-024 ModuleSummary reuse | canonical な `mizar-artifact` summary に対する `src/module_summary_reuse.rs` として実装済み。test は source-backed agreement、deterministic reuse/fallback、lockfile identity を含む既知 field identity validation、unsupported projection の fail-closed behavior、missing artifact summary で source loading をしないことを cover する。 |
| R-025 determinism suite | `src/lib.rs` は import graph、name diagnostics、`ResolvedAst` snapshot、`SymbolEnv` snapshot を横断する public-seam determinism regression を持つ。module-local determinism tests を補完する。 |
| R-026 public enum policy | module specs は resolver-owned public enum decision を全て列挙する。source attributes は全 listed enum を `#[non_exhaustive]` にしている。`tests/lint_policy.rs` は spec-owned module の source/spec drift を guard する。 |
| R-027 source/spec audit | この文書が対応関係を記録する。監査では unclassified な blocking/high `spec_gap`、`test_gap`、`source_drift`、`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict` は見つからなかった。 |
| R-028 bilingual documentation sync audit | bilingual design sync は `bilingual_documentation_synchronization.md` に記録済み。public source behavior は変更していない。 |
| R-029 module-boundary refactor gate | private helper / test module split は `module_boundary_refactor.md` に記録済み。移動した API についてこの source/spec scope を再実行し、public API、behavior、diagnostic、rendering、artifact、boundary drift は見つからなかった。 |

## follow-up record

この監査は新しい blocking follow-up を追加しない。R-024 後の既存分類 record は以下である:

| ID | Classification | Follow-up | Status |
|---|---|---|---|
| R-G001 | `spec_gap` から `external_dependency_gap` / deferred adoption へ精緻化 | public resolver diagnostic descriptor と将来の `mizar-diagnostics` adoption。共有 registry は広い `Resolution` family を reserve しているが、resolver の name/import/label descriptor は未採用。 | R-030 へ deferred。現在の resolver diagnostics は crate-local/internal のままにする。real adoption task が registry/spec ownership と coverage をそろえるまで、public numeric code、alias、placeholder adapter を追加しない。 |
| R-G002 | `test_gap` | lexical/parser の import/export syntax を超える semantic resolver corpus coverage が歴史的に不足していたこと。 | R-023 の active declaration-symbol smoke/fail fixture と post-task-20 R-G007 parser-backed signature-conflict active seed により部分的に解消済み。残る具体的な corpus assertion work は R-G007 が精緻化し、implemented behavior は unit tests が cover しているため R-027 には non-blocking。 |
| R-G003 | R-024 で解消済み | canonical `ModuleSummary` artifact から dependency module を消費する経路。 | resolver-owned artifact schema、shim、writer、hash framing、source loading を追加せず、canonical な `mizar-artifact` summary consumption として完了済み。 |
| R-G006 | `external_dependency_gap` | parser/syntax が owning source role を公開した後の module-level scheme/template declaration shell。 | represented source role については non-blocking。現 resolver は direct template role を owning signature payload に保持し、scheme/template module symbol を創作しない。 |
| R-G007 | `test_gap` | active signature-conflict increment 後に残る R-G002 の具体的な残り: R-009〜R-019 の import graph、namespace/name resolution、dot-chain、label-reference fact について、より広い active semantic `.miz` assertion を追加する。 | R-027 には non-blocking。unit tests が挙動を cover し、R-023 は declaration-symbol runner と初期 traceable active set を導入済み。`doc/spec/en` を超える挙動を創作せず、将来の runner assertion 拡張で coverage を増やす。 |
