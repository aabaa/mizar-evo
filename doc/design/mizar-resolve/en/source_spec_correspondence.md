# Source/spec correspondence audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

Status: task R-027 audit complete; task R-029 refactor scope re-run complete;
2026-07-02 roadmap synchronization overlay complete; task R-024 implementation
overlay complete; Task 265 R-031 ownership addendum and R-031 implementation
complete; R-032A implemented; R-032B committed at
`b3a7e79a6b60db2974e911c69bb56ff5f4609064`; Checker Task 258B5C is a
historical completed task committed as
`33ac57e96f048dc40559565f54369cac854409a7`.

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
  its current concrete remainder is refined by `R-G007`. Task 265 classified
  the exact same-signature/same-return conflict as R-G008, and R-031 has now
  resolved it. R-G003 is resolved by R-024.
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
- Checker Task 258B5C inventory found one Medium `source_drift`: pre-R-032B
  production normal-source `SurfaceAst` collection did not create proof-step
  `LabelProjection` and simple unqualified `LabelReferenceCandidate` inputs.
  The `LabelResolver` prefix behavior itself was correct, and committed
  R-032A/R-032B own the lower repair. Historical B5C closes its two confinement
  negatives; the rest of R-G007 and the Low deferred R-G001 public diagnostic
  adoption remain open.

## Public API Correspondence

| Spec | Public API checked | Source | Test evidence | Finding |
|---|---|---|---|---|
| [resolved_ast.md](./resolved_ast.md) stable identity and node arena | Existing ids/arena/AST plus implemented R-032A `SurfaceResolvedArena` and exact public error table | `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/resolved_ast/tests.rs`, and the sole `crates/mizar-resolve/tests/lint_policy.rs` R-026 owning-spec entry for `SurfaceResolvedArenaError` | Existing arena tests plus implemented R-032A complete-map, mismatch, stale, root, recovery, checked-overflow, precedence, equivalent-input determinism, and public-enum decision guards | No finding after R-032A |
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
| [labels.md](./labels.md) label projection and citation resolution | label scopes, projections, reference candidates, diagnostics, result tables, `LabelResolver` | `crates/mizar-resolve/src/labels.rs`, `crates/mizar-resolve/src/resolved_ast.rs`, `crates/mizar-resolve/src/env.rs` | existing explicit-projection label tests; committed R-032B matrix | Core behavior has no finding; R-032A/R-032B repair the structural-map/source-collector gap. |
| [labels.md](./labels.md), [recovery.md](./recovery.md) recovered label policy | recovered/failed namespace references remain unresolved and recovered label projections do not emit conflict diagnostics | `crates/mizar-resolve/src/labels.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_empty_and_failed_namespace_references_are_unresolved`, `recovered_label_projections_do_not_emit_conflict_diagnostics` | No finding |
| [symbols.md](./symbols.md) declaration-symbol projection and collection | `SymbolDeclarationProjection`, `SignatureProjectionExtractor`, `SymbolCollector`, `SymbolCollectionResult`, `SymbolDiagnostic`, `SymbolDiagnosticClass`, overload policy, parser-backed signature shells | `crates/mizar-resolve/src/symbols.rs`, `crates/mizar-resolve/src/env.rs`, `crates/mizar-resolve/src/env/snapshot.rs`, `crates/mizar-resolve/src/declarations.rs` | Existing collection/extraction tests plus R-031 `same_signature_same_return_functors_get_definition_conflict_class`, `same_return_conflict_candidates_keep_source_order_past_lexical_ordinal_ten`, `parser_backed_same_signature_same_return_functors_conflict`, `same_return_conflict_requires_the_exact_ordinary_functor_argument_key`, `mixed_return_group_keeps_one_return_conflict_in_canonical_order`, `recovered_same_return_functor_does_not_cascade_a_signature_conflict`, and `same_signature_definition_conflict_snapshot_spelling_is_stable` | No finding after R-031 |
| [symbols.md](./symbols.md), [recovery.md](./recovery.md) recovered and context-only symbol policy | recovered projections remain local/malformed, context-only shells do not fabricate symbols, recovered diagnostics do not cascade | `crates/mizar-resolve/src/symbols.rs`, `crates/mizar-resolve/src/recovery.rs` | `recovered_shells_stay_local_and_malformed_without_panicking`, `recovered_symbols_do_not_cascade_duplicate_or_overload_diagnostics`, `recovered_context_only_shells_do_not_emit_context_diagnostics`, `context_parent_visibility_and_recovery_propagate_to_child_symbols`, `context_only_shells_do_not_fabricate_symbol_identities`, `parser_backed_recovered_projection_uses_malformed_signature` | No finding |
| [module_summary_reuse.md](./module_summary_reuse.md) canonical summary reuse | `ModuleSummaryReuseRequest`, `ModuleSummaryReuse`, `ModuleSummaryReuseResult`, `ModuleSummaryReuseDiagnostic`, `ModuleSummaryReuseReason`, reader-backed and already-validated projection paths | `crates/mizar-resolve/src/module_summary_reuse.rs`, `crates/mizar-resolve/src/lib.rs`, `crates/mizar-resolve/Cargo.toml` | `summary_backed_projection_matches_source_backed_exports`, `summary_backed_symbol_surface_matches_source_collector`, `lockfile_identity_is_accepted_when_known_identity_fields_match`, `identity_and_expected_hash_mismatch_fall_back`, `unknown_symbol_visibility_fails_closed`, `unknown_label_visibility_and_target_kind_fail_closed`, `missing_dependency_summary_does_not_source_load` | No finding |
| [symbols.md](./symbols.md), [mizar-test staged model](../../mizar-test/en/staged_model.md) declaration-symbol runner | active `declaration_symbol` corpus stage, internal detail-key expectation matching without public resolver diagnostic codes, plus exact SymbolEnv-derived pass payload assertions for represented kind, visibility, and export status | `crates/mizar-test/src/runner/shared.rs`, `crates/mizar-test/src/runner/declaration_symbol.rs`, represented pass/fail sidecars including both same-signature conflict controls, `tests/coverage/spec_trace.toml` | focused R-031 resolver/mizar-test tests; plan and declaration-symbol CLI; `cargo test -p mizar-test`; active expectation sidecars tagged `active_declaration_symbol` | No finding after R-031: same-return and different-return exact internal keys are separately active; broader import/name/dot-chain/label assertions remain R-G007. |
| [todo.md](./todo.md) lint, deterministic hardening, and enum policy | workspace lint opt-in, documented `allow` rationale guard, deterministic public-seam regression, public enum `#[non_exhaustive]` and owning-spec decision table | `crates/mizar-resolve/tests/lint_policy.rs`, `crates/mizar-resolve/src/lib.rs`, all public resolver enum owners | lint-policy tests; `resolver_public_seams_are_deterministic_for_equivalent_inputs`; `public_resolver_enums_are_marked_non_exhaustive_and_documented` | No finding |

## Behavior Boundary Trace

| Boundary | Audit result |
|---|---|
| Parser/syntax/frontend boundary | Resolver source consumes `SurfaceAst`, `SurfaceNodeView`, syntax recovery markers, and frontend-produced lexical/module surfaces. It does not add syntax vocabulary, parser recovery, tokenization, or frontend orchestration behavior. |
| Build/session boundary | `module_index.rs` consumes the build-side `ModuleIndexProvider` contract and preserves alias-independent `ModuleId` construction. It does not parse manifests, discover modules, load sources, or own build planning. |
| Checker/type/proof boundary | Name and dot-chain resolution records unresolved, ambiguous, overload-group, and deferred-selector states. It does not perform type-directed overload winner selection, selector type checking, cluster firing, proof checking, obligation generation, or VC production. |
| Diagnostics boundary | Resolver diagnostics remain crate-local/internal while `R-G001` is open. R-023 declaration-symbol expectations compare internal detail keys in payload metadata and keep public `diagnostic_codes` empty. |
| Checker Task 258B5C boundary | R-032A first validates the structural Surface-to-resolved map; R-032B then collects narrowly supported proof-step/simple-reference rows. The later private `mizar-test` consumer owns `declaration_symbol.label.proof_scope_confinement`. Public checker handoff remains excluded. |
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
| R-019 to R-023 symbols and corpus runner | `symbols.md` is implemented by `src/symbols.rs`; unit tests cover opaque and parser-backed signatures, represented duplicates/conflicts, overload grouping, registrations, recovery, context-only shells, and deterministic diagnostics. R-023 adds active declaration-symbol pass/fail corpus seeds and traceability metadata. |
| R-031 same-return declaration conflict | `src/symbols.rs`, `src/env.rs`, and `src/env/snapshot.rs` implement the exact ordinary-functor classifier, distinct internal metadata, mixed-group priority, and stable snapshot spelling. Exact/near-miss/order/recovery tests plus the active same-return sidecar and sole covered trace row resolve R-G008 without changing the different-return control. |
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
| R-G007 | `test_gap` | Concrete remainder of R-G002 after the active signature-conflict/pass-payload increments and B5C's two active confinement negatives: broader semantic `.miz` assertions for import graph, namespace/name resolution, dot-chain, and other label-reference facts from tasks R-009 to R-019. | R-032A/R-032B are complete. Historical B5C adds only the inner-to-outer and sibling confinement negatives through private `mizar-test`. Keep public codes empty and checker handoff narrow. |
| R-G008 | resolved by R-031 | Chapter 19 §19.1 requires ordinary declarations with the same symbol kind, spelling, arity, and argument signature to conflict even when return signatures match. Pre-R-031 source skipped all-return-identical groups, the exact seed was deferred, and design lacked a distinct same-return class/detail key plus mixed-group priority. | Ordinary functor definitions are grouped by the exact resolver-syntactic key. Appended `SameSignatureDefinitionConflict` diagnostic/definition variants cover all-return-identical groups; existing `SameSignatureReturnConflict` wins for mixed/different-return groups. Exact unit/near-miss/order/recovery/snapshot tests and the active declaration-symbol sidecar cover the new key while preserving first shell/range, all candidate identities/order, the byte-identical different-return sidecar, and checker-owned semantic equality/selection boundaries. |

## Implemented R-032A / R-032B Correspondence

Canonical Chapter 15 §15.10 and Chapter 16 §§16.4.2/16.5.1 authorize the
following lower-only repair:

| Task and contract | Source | Required evidence | Excluded |
|---|---|---|---|
| R-032A exact `SurfaceResolvedArena` API and enum declaration from `resolved_ast.md` | `crates/mizar-resolve/src/resolved_ast.rs` | complete map; exact structure/origin/path; every typed mismatch including `ResolutionStateMismatch` and `ReferenceKeyMismatch`; checked overflow | labels, runner, fixtures, trace, semantic resolution |
| R-032A tests | `crates/mizar-resolve/src/resolved_ast/tests.rs` | exact error matrix, deterministic mapping, wildcard compatibility, no unchecked conversion/panic | other resolver test owners |
| R-032A R-026 enum decision guard | `crates/mizar-resolve/tests/lint_policy.rs` | exactly one `SurfaceResolvedArenaError` owning-spec decision entry | every other lint decision, lint behavior, or source owner |
| R-032B exact lifetime/error/origin/default-deny direct-edge contract from `labels.md` | `crates/mizar-resolve/src/labels.rs` | exact `Root` -> `CompilationUnit` -> `ItemList` -> theorem chain; only AST/arena borrows; global ordinals; `proof-step-v1`; every unlisted edge skips | callback, unmapped side channel, unsupported/recovered/semantic forms |
| R-032B tests | `crates/mizar-resolve/src/labels/tests.rs` | positive per upper/lower edge; missing/additional/wrong/relocated/wrapped upper negatives; confinement/origin; other negative mutations; mixed-list and all-other matrices | `.miz`, expectations, trace status/counts, active runner |
| R-032B R-026 enum decision guard | `crates/mizar-resolve/tests/lint_policy.rs` | exactly one `ProofLabelSourceCollectionError` owning-spec decision with `spec_name: "labels.md"` | every other lint decision, lint behavior, or source owner |

R-032A is implemented as its separate lower-prerequisite logical task. The
R-032B lint-policy correction and exact three-Rust-file implementation are
committed; the historical B5C consumer subsequently committed as
`33ac57e96f048dc40559565f54369cac854409a7`. This
bounded post-exit implementation does not change the original
milestone score.

R-032A implementation preflight classified the earlier two-Rust-file scope as
High `design_drift`: the new public enum is necessarily scanned by the
existing R-026 guard. The authority in `resolved_ast.md` plus the existing
R-026 correspondence is sufficient for a separate synchronized docs-only
scope correction and the sole lint decision entry in the later exact
three-Rust-file implementation. No test intent or semantic authority changes.

The inserted S-026 dependency is infrastructure-only. It changes no Chapter 15
or 16 requirement, `.miz` intent, expectation, trace row, or coverage credit.
It supplies only complete syntax ids required by the already frozen R-032A
structural mapping, so `doc/design/spec_coverage_audit.md` remains unchanged.
R-032A likewise changes no active `.miz` mapping, trace/backlink/status/count,
owner crate, deferred status, or coverage credit, so the audit remains a
deliberate no-op.

Fresh R-032B inventory classifies the omitted mandatory decision owner as High
`design_drift`, with no semantic `spec_gap`, `test_gap`, or test-intent
change. Later implementation owns exactly the three R-032B rows above. The
completed synchronized docs-only correction has an exact total scope of 31
design files: 16 resolver, eight checker, six `mizar-test`, and one global
ledger. It changes no source, fixture, sidecar, expectation, trace
status/count, Cargo metadata, or coverage state; therefore
`spec_coverage_audit.md` remains a deliberate no-op. The independent
specification, test/scope, and source/documentation consistency reviews
report **NO FINDINGS**, and the docs-only verification/count/hash gates PASS.
Independent final read-only quality also reports **NO FINDINGS**; all nine
hard gates PASS with no cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). At that pre-commit record, only task-only
staging/cached-diff review, commit, and post-commit invariant/fresh-inventory
gates remained pending. They subsequently completed in correction commit
`f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a` and its fresh inventory.

## R-032B implementation correspondence result

Committed source provides the frozen public collector/collection/error contract
in `labels.rs`, the focused matrix in `labels/tests.rs`, and only the
authorized `ProofLabelSourceCollectionError` R-026 decision in
`tests/lint_policy.rs`. The collector consumes the validated R-032A arena,
preserves the closed Surface edge allowlist, and emits existing
`LabelProjection` / `LabelReferenceCandidate` inputs with exact scopes,
ordinals, completion boundaries, structural origins, and `proof-step-v1`
identity. No new resolver outcome or semantic phase is introduced.

The Medium third-child and unauthorized `Default` / `From` implementation
findings and the initial High/Medium plus two fresh Medium test gaps are
fixed. Preimplementation specification and final fresh test-sufficiency,
implementation, and source/documentation reviews report **NO FINDINGS**.
Focused/crate/workspace and all count/hash/scope gates PASS. Exact consumers
remain unit tests and the historical private B5C route; checker unresolved-reference
handoff remains excluded. No fixture, expectation, sidecar, trace, active
runner, public diagnostic, Cargo metadata, or coverage status changes.
R-G007 stays open beyond the two active B5C confinement negatives.
`spec_coverage_audit.md` remains a deliberate no-op. Independent final
quality reports **NO FINDINGS**; all nine hard gates PASS with no score cap at
valid `100/100` (`20/20/15/15/10/10/5/5`). Task-only
restaging/cached-diff review, commit
`b3a7e79a6b60db2974e911c69bb56ff5f4609064`, and post-commit
invariant/fresh inventory are complete.

## Checker Task 258B5C source correspondence status

The historical B5C source privately consumed unchanged R-032A
`SurfaceResolvedArena` and R-032B `ProofLabelSourceCollector` /
`LabelResolver` APIs in `mizar-test`. The authority-derived corpus delta is
exactly two fail fixtures, two expectation sidecars, and two covered trace
rows. Exact source/test consumers are
`crates/mizar-test/src/runner/declaration_symbol.rs`,
`crates/mizar-test/src/runner/tests.rs`,
`crates/mizar-test/src/runner/tests/declaration_symbol.rs`, and
`crates/mizar-test/tests/metadata.rs`; the last updates four frozen
active-count/CLI assertions from declaration stage `5` to `7`. Resolver
production and public API remain unchanged.

Plan/pass/fail counts are `421/389` and `228/193`; active
parse/declaration/type/proof is `101/7/198/1`; warning/error counts are
`23/0`.
Public codes remain empty and the private route key is
`declaration_symbol.label.proof_scope_confinement`. This closes only the
inner-to-outer and sibling confinement negatives in R-G007. Import, name,
dot-chain, and other label-reference coverage remains open. Test,
implementation, source/documentation reviews and all verification gates are
complete. Independent final quality reports **NO FINDINGS**; all nine hard
gates PASS with no score cap at valid `100/100`. Task-only cached-diff review,
dedicated B5C commit `33ac57e96f048dc40559565f54369cac854409a7`, and
post-commit fresh inventory are complete at this historical checkpoint.

## Checker Task 263R Source/Specification Correspondence

Canonical Chapter 5 owns per-structure field/property identity. Its examples
legitimately repeat `carrier` in different structure declarations and require
inherited members to retain a root declaration plus a path/view. The exact
320-byte Task-263 probe parses without diagnostics and produces 75 Surface
nodes, ten shells, eight projections, and eight symbols, but preimplementation
`symbols.rs` grouped its four selectors by module namespace alone and emitted two
false duplicates. This is bounded `source_drift`; the matching design rule is
`design_drift`, and the absent owner-sensitive regression is `test_gap`.

The frozen repair derives only a nearest `StructureDefinition` owner for
selector conflict classification. It does not resolve selector uses, validate
inheritance, change public ids/signatures, or create checker/proof semantics.
No existing fixture, sidecar, expectation, trace row/status/count, or active
coverage changes in the documentation prerequisite or lower implementation.
`spec_coverage_audit.md` records the corrected Chapter-5 lower owner but grants
no corpus credit; Checker Task 263 remains the executable structure-intake
owner after Task 263R commits.

## Checker Task 263R Implemented Correspondence

The two-file resolver implementation now agrees with Chapter 5 §§5.2-5.5:
selector spelling is interpreted within the nearest structure declaration for
duplicate classification, while selectors inside one structure still share
one conflict domain. The exact Chapter-5-derived probe moves from two false
resolver diagnostics to zero without changing its `75/10/8/8` lower profile;
the exact same-owner control retains one deterministic diagnostic at
`47..70`. The extractor-backed tests directly exercise the declaration-shell
owner relation and reversed input order.

This closes the classified `source_drift` and canonical-derived `test_gap`.
The earlier documentation prerequisite already closed `design_drift`.
Canonical specification, existing corpus artifacts, expectations, and trace
metadata do not change, and no executable corpus credit is claimed. Chapter 5
remains partial for inheritance, constructor/selector semantics, correctness
obligations, and the future Task-263 checker consumer.

Final source/documentation consistency reports **NO FINDINGS**. All full
verification gates pass, and independent quality reports **NO FINDINGS**, all
nine hard gates PASS, and uncapped `100/100`; this result does not add corpus
or trace credit.

## Checker Task 264R Source/Specification Correspondence

Chapter 7 §7.4.1 and Parser Task 48 establish property implementations as
represented top-level declarations. The parser nodes currently disappearing
before resolver shells are `source_drift`; the missing context-only shell and
no-symbol contract is `design_drift`; the absent pass/recovery regressions are
`test_gap`. There is no blocking `spec_gap`. Reusing selector, property-clause,
redefinition, or registration identities would be a `boundary_violation`.

Task 264R corrects only lower representation. It changes no canonical spec,
existing `.miz` source, sidecar, expectation, trace metadata, checker/runner
source, or coverage status. `spec_coverage_audit.md` records the future lower
owner and keeps the Chapter-7 semantic and executable gaps open. Checker Task
248P must separately admit the new shell to source binding context before
Checker Task 264 may freeze and implement its property payload producer.

## Checker Task 264R Implemented Correspondence

The lower `source_drift` and authority-derived `test_gap` are closed: all
parser-represented property implementations now survive as context-only
resolver shells, while exact fixture profiles and the following theorem's
identity/effects remain stable. The documentation prerequisite closed the
matching `design_drift`. No canonical specification, corpus, expectation,
trace metadata, semantic coverage, or deferred Checker Task 248P/264 intent
changes.

## Resolver Task 277R1 Source/Specification Correspondence

The implemented [Task 277R1 contract](../../task_contracts/en/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md)
preserves the parser-proven `DefinitionBlockItem#53` / `TemplateParameter#31` /
`T#2` / generator `TypeHead#39` / `T#21` profile. The resolver-owned
declaration/use transport closes the classified `source_drift`; the paired
design records close `design_drift`; four resolver plus one direct fixture
regression close the Rust `test_gap`. There is no `spec_gap`.

Chapter 18 establishes template parameter scope and separately reserves bare
type-parameter sethood rejection to symbolic verification; Chapter 13 §13.4.2
requires the later Fraenkel sethood check. Therefore this task transports only
validated structure and explicitly leaves sethood, verdict, diagnostic,
formal/actual substitution, overload, checker activation, and coverage credit
outside the resolver. `spec_coverage_audit.md`, the inactive seed, sidecar,
trace, and expectations remain unchanged.

## Resolver Task 277R2 Source/Specification Correspondence

The [Task 277R2 contract](../../task_contracts/en/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md)
maps Chapter 13 §§13.4.2, 13.4.4, and 13.8.6 generator binding/capture
requirements and Chapter 18 §18.10.2's immutable F5 context to a resolver-only
binding/use identity collection. The exact `#53/#52/#49/#41/#19` owner and
binding profile plus mapper `#38/#37/#17` and condition `#48/#42/#24`,
`#44/#26` uses reconcile the classified `design_drift`. The implementation
and four resolver regressions plus one direct fixture regression resolve
`source_drift` and the Rust `test_gap`. There is no `spec_gap`.

This prerequisite records identity and source order only. It does not carry
R1 or 277B-L identities or decide template substitution, types, sethood,
evidence, diagnostics, rejection, or checker activation. The immutable seed,
expectation, inactive trace mapping, deferred `MC-G020`/`MC-G021` checker-plan
gaps, semantic credit, and
`spec_coverage_audit.md` remain unchanged, so the coverage audit has no delta.
The independent source/documentation/API integration review reports **NO
FINDINGS**.
