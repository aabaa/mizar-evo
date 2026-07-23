# Source/spec correspondence audit

## Parser Task 46 Post-Exit Correspondence

| Contract | Source and tests | Boundary |
|---|---|---|
| `OperatorDeclaration`; append-only `SyntaxKind::OperatorDeclaration = 193`; matching surface kind, typed accessor, snapshot/raw/node/rowan support | `crates/mizar-syntax/src/ast.rs`, `ast/snapshot.rs`, syntax Task-46 unit tests, parser unit tests, and the active operator-declaration pass/fail pair | Syntax-only source representation; no activation, resolution, or precedence semantics |

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

Status: completed through S-025, after the task-24 AST private module split and
task-25 follow-up audit; Parser Task 48 post-exit correspondence is appended
below without creating a new syntax task ID.

## Scope

This audit checks the completed `mizar-syntax` implementation against the
English canonical syntax design specs: [ast.md](./ast.md),
[trivia.md](./trivia.md), [recovery.md](./recovery.md), this crate plan, and
[todo.md](./todo.md). It traces public APIs and implementation-facing behavior
promises to source and tests.

The S-025 rerun specifically checks the task-24 module-boundary refactor:
public API lists, source/test correspondence, crate-root re-export paths,
snapshot stability, and the parser/syntax boundary promise.

This is a lightweight source/spec/test correspondence map. It is not a release
coverage gate and does not replace executable tests. If this audit finds a
missing implementation, stale spec text, missing test, or source behavior that
needs a new owner, the item must be recorded as a classified follow-up rather
than hidden by changing source, expectations, or documentation to match current
behavior.

The Japanese companion mirrors the public API names and behavior boundaries
below. The broader bilingual wording, terminology, status, and link audit is
recorded separately in
[bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md).

## Result

- No missing implementation was found for the public APIs and behavior
  contracts promised by `ast.md`, `trivia.md`, or `recovery.md` through parser
  task 35.
- No implementation-facing public behavior was found that is absent from
  `doc/design/mizar-syntax`, Rust tests, parser-facing `.miz` tests, existing
  traceability metadata, or an explicit deferred/out-of-scope note.
- This audit added one targeted Rust test for the `SyntaxDiagnostic` builder
  contract after method-level trace review. Existing `mizar-syntax` unit tests,
  lint-policy tests, parser task 4-35 unit/corpus coverage, and syntax snapshot
  baselines cover the remaining audited promises.
- No remaining unclosed `spec_gap`, `test_gap`, `source_drift`,
  `source_undocumented_behavior`, `test_expectation_drift`,
  `boundary_violation`, or `repo_metadata_conflict` was found. The S-023
  documentation `design_drift` found during the rerun is recorded and closed
  below.
- Existing follow-up records remain classified below: parser fixture seed
  activation, vocabulary-only future recovery producers, the dotted algorithm
  `Lvalue` active `.miz` coverage gap, and S-021 deferred rustdoc summaries.
- The S-023 rerun found the task-22 predicate redefinition label repair
  implemented and tested: `PredicateRedefinition` owns the label-or-`MissingTerm`
  slot before `PredicatePattern`, parser task 36 emits that shape, active
  pass/fail corpus expectations cover labeled and missing-label cases, and no
  new source/spec, test, expectation, or metadata gap remains.
- The rerun found documentation `design_drift` in status text that still
  treated parser task 36 / syntax task 22 as pending. This task closes that
  drift in the parser README, top-level roadmap, syntax README, historical
  crate-exit note, and bilingual audit records without changing language
  behavior.
- The S-025 rerun found the task-24 source split implemented as documented:
  `crates/mizar-syntax/src/ast.rs` remains the public `ast` module, private
  helpers live in `src/ast/{green,snapshot,tests}.rs`, crate-root re-exports
  are unchanged, `SyntaxKind` numbering is unchanged, snapshot text remains
  byte-stable, and parser-facing builder/accessor contracts remain source- and
  test-backed.
- No remaining API-list, source/test, re-export-path, snapshot-stability,
  parser/syntax-boundary, bilingual documentation, expectation, metadata,
  `design_drift`, `source_drift`, or `repo_metadata_conflict` gap was found by
  the S-025 rerun.

## Public API Correspondence

| Spec | Public API checked | Source | Test evidence |
|---|---|---|---|
| [ast.md](./ast.md) storage boundary | `MizarLanguage`, `RowanSyntaxNode`, `RowanSyntaxToken`, `RowanSyntaxElement`, `SyntaxKind`, `SurfaceAst`, `SurfaceAstBuilder`, `BuilderNode`, `SurfaceBuilderNodeId`, `SurfaceNodeId`, `SurfaceNode`, `SurfaceNodeView`, crate-root re-exports | `crates/mizar-syntax/src/lib.rs`, `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/green.rs` | `builder_round_trips_into_rowan_backed_tree`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `repeated_construction_produces_deterministic_green_tree_and_views`, `builder_rejects_child_ids_not_created_by_this_builder`, `builder_rejects_token_sharing_between_multiple_structural_parents` |
| [ast.md](./ast.md) syntax vocabulary | `SurfaceNodeKind`, `SurfaceTokenKind`, `SurfaceToken`, `SurfaceInfixOperator`, `SurfacePrefixOperator`, `SurfacePostfixOperator`, `SurfaceOperatorAssociativity`, `SurfaceFormulaPrefixOperator`, `SurfaceFormulaConnective`, `SurfaceFormulaBinaryOperator`, `SurfaceQuantifierKind`, `SurfaceFormulaConstant`, and all `SurfaceNodeView::as_*` helpers through task 35 plus the task-22 predicate redefinition label-slot follow-through | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `typed_accessors_cover_current_node_and_token_kinds`, `task8_typed_accessors_cover_type_expression_nodes`, `task9_typed_accessors_cover_primary_term_nodes`, `task10_typed_accessors_cover_selector_and_update_nodes`, `task11_typed_accessor_covers_qua_expression`, `task12_typed_accessors_cover_prefix_and_postfix_operator_nodes`, `task13_typed_accessors_cover_atomic_formula_nodes`, `task14_typed_accessors_cover_formula_connective_and_quantifier_nodes`, `task15_typed_accessors_cover_set_comprehension_nodes`, `task16_typed_accessors_cover_simple_statement_nodes`, `task17_typed_accessors_cover_justification_nodes`, `task18_typed_accessors_cover_consider_reconsider_nodes`, `task19_typed_accessors_cover_conclusion_then_iterative_nodes`, `task20_typed_accessors_cover_block_statement_nodes`, `task21_typed_accessors_cover_inline_definition_nodes`, `task22_typed_accessors_cover_theorem_and_proof_nodes`, `task23_typed_accessors_cover_definition_nodes`, `task24_typed_accessors_cover_predicate_definition_nodes`, `task25_typed_accessors_cover_functor_definition_nodes`, `task26_typed_accessors_cover_mode_definition_nodes`, `task27_typed_accessors_cover_redefinition_and_notation_nodes`, `task22_predicate_redefinition_missing_label_snapshot_is_distinct`, `task28_typed_accessors_cover_property_clause_nodes`, `task29_typed_accessors_cover_structure_nodes`, `task30_typed_accessors_cover_registration_nodes`, `task31_typed_accessors_cover_template_nodes`, `task32_typed_accessors_cover_algorithm_nodes`, `task33_typed_accessors_cover_algorithm_control_flow_nodes`, `task34_typed_accessors_cover_algorithm_verification_nodes`, `task35_typed_accessors_cover_annotation_nodes` |
| [ast.md](./ast.md) deterministic snapshots, ranges, identity, and reuse | `SurfaceAst::{snapshot_text, snapshot_text_with_trivia, range_contains_child_ranges, green_node, rowan_root, with_trivia, trivia}`, `SurfaceNodeKind::syntax_kind`, `SurfaceTokenKind::syntax_kind`, `SurfaceNodeView::{id, kind, syntax_kind, range, children, child_views, is_recovered}` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/snapshot.rs`, `crates/mizar-syntax/src/ast/green.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `parent_ranges_contain_child_ranges_except_recovery_attachments`, `repeated_snapshot_rendering_is_byte_identical`, `snapshot_rendering_matches_current_vocabulary_baseline`, `snapshot_payload_names_cover_current_variants`, `snapshot_rendering_includes_trivia_when_requested`, `trivia_snapshot_rendering_is_sorted_and_byte_identical`, `trivia_snapshot_target_sorting_breaks_collisions_deterministically`, `tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap` |
| [trivia.md](./trivia.md) storage, sorting, and attachment | `SurfaceTrivia`, `SurfaceTriviaBuilder`, `CommentTrivia`, `DocCommentAttachment`, `SkippedTokenRange`, `WhitespaceHint`, `TriviaNodeTarget`, `TriviaAttachmentTarget`, `TriviaPlacement`, `SkippedTokenReason`, `WhitespaceHintKind` | `crates/mizar-syntax/src/trivia.rs`, `crates/mizar-syntax/src/ast.rs` attachment validation | `trivia_builder_preserves_ownership_and_attachment_hints`, `skipped_ranges_are_preserved_with_source_ranges`, `generated_detached_anchor_must_match_trivia_source`, `doc_comment_can_attach_to_following_placeholder_item_node`, `ast_rejects_token_node_as_trivia_node_target`, `ast_rejects_non_token_trivia_token_target`, `ast_rejects_trivia_target_with_mismatched_range`, `ast_rejects_trivia_from_another_source` |
| [recovery.md](./recovery.md) diagnostics | `SyntaxDiagnostic`, `SyntaxDiagnostic::{new, with_secondary, with_recovery_note}`, `SyntaxDiagnosticCode` | `crates/mizar-syntax/src/recovery.rs`, parser producers in `crates/mizar-parser` | `syntax_diagnostic_builder_preserves_secondary_and_recovery_note`, `recovery_kinds_are_constructible_with_documented_ranges`, parser pass/fail corpus cases through task 35, frontend parser-seam syntax diagnostic passthrough tests |
| [recovery.md](./recovery.md) recovery vocabulary and recovered node contract | `SyntaxRecoveryKind`, `SurfaceNodeKind::ErrorRecovery`, `SurfaceTokenKind::ErrorRecovery`, `SurfaceAstBuilder::{add_recovered_token, add_recovery}` | `crates/mizar-syntax/src/recovery.rs`, `crates/mizar-syntax/src/ast.rs` | `recovery_kinds_are_constructible_with_documented_ranges`, `recovery_snapshot_names_are_unique_and_fully_fixture_backed`, `parent_ranges_contain_child_ranges_except_recovery_attachments`, task 5 and task 16-35 parser recovery corpus coverage |
| [todo.md](./todo.md), [ast.md](./ast.md), [trivia.md](./trivia.md), [recovery.md](./recovery.md) enum policy | Workspace lint opt-in, documented `allow` rationale guard, public enum forward-compatibility decisions | `crates/mizar-syntax/Cargo.toml`, `crates/mizar-syntax/tests/lint_policy.rs`, public enums in `src/ast.rs`, `src/trivia.rs`, `src/recovery.rs` | `syntax_manifest_opts_into_workspace_lints`, `workspace_lint_baseline_denies_rustc_warnings_and_clippy_all`, `syntax_allow_exceptions_are_documented_inline`, `public_forward_compatible_enums_are_marked_non_exhaustive`, `public_enum_exhaustiveness_exceptions_are_documented`, `every_public_enum_has_a_forward_compatibility_decision` |

## Method-Level API Correspondence

| API family | Public methods checked | Source | Test evidence |
|---|---|---|---|
| `SurfaceAst` accessors and storage views | `node`, `nodes`, `root`, `token_nodes`, `expression_root`, `node_view`, `root_view`, `expression_view`, `token_views`, `token_texts`, `green_node`, `rowan_root`, `trivia`, `with_trivia`, `snapshot_text`, `snapshot_text_with_trivia`, `range_contains_child_ranges` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/green.rs`, `crates/mizar-syntax/src/ast/snapshot.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `builder_round_trips_into_rowan_backed_tree`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `repeated_construction_produces_deterministic_green_tree_and_views`, `repeated_snapshot_rendering_is_byte_identical`, `snapshot_rendering_matches_current_vocabulary_baseline`, `snapshot_rendering_includes_trivia_when_requested`, `parent_ranges_contain_child_ranges_except_recovery_attachments`, trivia target rejection tests |
| `SurfaceAstBuilder` construction API | `new`, `add_node`, `add_token`, `add_recovered_token`, `add_recovery`, `node`, `node_kind`, `node_range`, `token_node_ids`, `recovery_node_ids`, `finish` | `crates/mizar-syntax/src/ast.rs` | `builder_round_trips_into_rowan_backed_tree`, `recovery_kinds_are_constructible_with_documented_ranges`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `builder_rejects_child_ids_not_created_by_this_builder`, `builder_rejects_token_sharing_between_multiple_structural_parents`, all task 8-35 typed-accessor tests |
| `SurfaceNodeView` typed and structural accessors | `id`, `kind`, `syntax_kind`, `range`, `children`, `is_recovered`, `as_token`, `as_infix_expression`, `as_prefix_expression`, `as_postfix_expression`, `as_recovery`, every task-specific `as_*` typed helper through parser task 35, `child_views` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `typed_accessors_cover_current_node_and_token_kinds`, every task 8-35 typed-accessor test, `recovery_kinds_are_constructible_with_documented_ranges`, `recovery_snapshot_names_are_unique_and_fully_fixture_backed` |
| Compatibility node and token helpers | `SurfaceNode::{new, recovered, token_text}`, `SurfaceNodeKind::syntax_kind`, `SurfaceNodeKind::is_structural`, `SurfaceToken::new`, `SurfaceTokenKind::syntax_kind`, `SurfaceNodeId::index` | `crates/mizar-syntax/src/ast.rs` | `typed_accessors_cover_current_node_and_token_kinds`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `snapshot_payload_names_cover_current_variants`, task 8-35 typed-accessor tests |
| `SyntaxKind` raw-kind helpers | `from_raw`, `is_node_kind`, `is_token_kind` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `typed_accessors_cover_current_node_and_token_kinds`, `snapshot_rendering_matches_current_vocabulary_baseline` |
| `SurfaceTrivia` read API | `empty`, `source_id`, `is_empty`, `comments`, `doc_comment_attachments`, `skipped_token_ranges`, `whitespace_hints` | `crates/mizar-syntax/src/trivia.rs` | `trivia_builder_preserves_ownership_and_attachment_hints`, `skipped_ranges_are_preserved_with_source_ranges`, `snapshot_rendering_includes_trivia_when_requested`, `trivia_snapshot_rendering_is_sorted_and_byte_identical` |
| `SurfaceTriviaBuilder` construction API | `new`, `add_comment`, `add_doc_comment_attachment`, `add_skipped_token_range`, `add_whitespace_hint`, `finish`; `TriviaNodeTarget::new` | `crates/mizar-syntax/src/trivia.rs` | `trivia_builder_preserves_ownership_and_attachment_hints`, `skipped_ranges_are_preserved_with_source_ranges`, `generated_detached_anchor_must_match_trivia_source`, `doc_comment_can_attach_to_following_placeholder_item_node`, trivia target rejection tests |
| `SyntaxDiagnostic` construction API | `new`, `with_secondary`, `with_recovery_note` | `crates/mizar-syntax/src/recovery.rs` | `syntax_diagnostic_builder_preserves_secondary_and_recovery_note`, parser pass/fail corpus cases through task 35, frontend parser-seam syntax diagnostic passthrough tests |

## Enum And Diagnostic Correspondence

| Surface | Current status |
|---|---|
| `SyntaxKind`, `SurfaceNodeKind`, `SurfaceTokenKind` | Implemented in `ast.rs`, documented in `ast.md`, guarded by rowan raw-kind and typed-accessor coverage. These enums remain forward-compatible for downstream crates. |
| `MizarLanguage`, `SurfaceOperatorAssociativity`, `SurfaceFormulaPrefixOperator`, `SurfaceFormulaConnective`, `SurfaceQuantifierKind`, `SurfaceFormulaConstant`, `TriviaPlacement` | Documented deliberate exhaustive exceptions. `snapshot_payload_names_cover_current_variants` and lint-policy tests guard the current closed payload vocabulary. |
| `TriviaAttachmentTarget`, `SkippedTokenReason`, `WhitespaceHintKind` | Implemented in `trivia.rs`, documented in `trivia.md`, and guarded as non-exhaustive. Sorting, same-source validation, and snapshot rendering tests cover current variants. |
| `SyntaxRecoveryKind`, `SyntaxDiagnosticCode` | Implemented in `recovery.rs`, documented in `recovery.md`, and guarded as non-exhaustive. Constructibility, snapshot-name, syntax diagnostic, and parser recovery tests cover the current vocabulary. Recovery kinds marked as vocabulary-only in `recovery.md` are future parser-producer work, not missing `mizar-syntax` implementation. |

## Task Requirement Correspondence

| Task group | Source/test correspondence |
|---|---|
| Tasks 1-5 representation foundation | Module split, lint policy, rowan storage, builder/accessor API, deterministic snapshots, trivia side tables, and recovery vocabulary are implemented in `src/lib.rs`, the public `src/ast.rs` module plus private `src/ast/{green,snapshot,tests}.rs` partitions, `src/trivia.rs`, `src/recovery.rs`, and `tests/lint_policy.rs`. Unit tests cover builder round trips, rowan raw kinds, deterministic snapshots, range rules, trivia ownership/sorting/attachment, recovery kind constructibility, and diagnostics. |
| Tasks 6-8 grammar gates | Grammar audit, parse-only acceptance matrix, and fixture seed are documented in `grammar_audit.md`, `parse_only_acceptance_matrix.md`, and `parse_only_fixture_seed.md`. They intentionally did not freeze final AST snapshots before paired parser support existed. |
| S-009 module, item, and shared path nodes | Parser tasks 4-7 are implemented with syntax node/accessor/snapshot coverage and active module/import/export/visibility parse-only corpus cases. |
| S-010 type expression nodes | Parser task 8 is implemented with syntax node/accessor/snapshot coverage, malformed type recovery, and active type parse-only corpus cases. |
| S-011 term nodes | Parser tasks 9-12 and 15 are implemented with term, selector/update, `qua`, operator, and set-comprehension syntax coverage, plus active term parse-only corpus cases. |
| S-012 formula nodes | Parser tasks 13-14 are implemented with atomic formula, generic `is`, connective, quantifier, grouping, constants, and formula recovery coverage. |
| S-013 statement nodes | Parser tasks 16 and 18-21 are implemented with simple statements, `consider`/`reconsider`, conclusion/`then`/iterative equality, block reasoning, and inline definitions. |
| S-014 theorem, proof, and justification nodes | Parser tasks 17 and 22 are implemented with justification, citation, computation, theorem/lemma, proof-block, proof recovery, and active parse-only coverage. |
| S-015 definition, structure, and registration nodes | Parser tasks 23-30 are implemented with definition blocks, attribute/predicate/functor/mode definitions, redefinitions, notation aliases, properties, structures, inheritance, registrations, recovery, and active traceable corpus coverage. |
| S-016 template, algorithm, and annotation nodes | Parser tasks 31-35 are implemented with templates, algorithms, claims, control flow, verification clauses, annotations, recovery, syntax typed accessors, parser unit tests, active `.miz` coverage, and traceability metadata. |
| S-017 enum policy | Final enum classification is implemented by source attributes and guarded by lint-policy tests. |
| S-018 incremental reuse audit | Identity, raw-kind numbering, range-attached trivia/recovery reuse, localized-edit validation, and annotation accessor/raw-kind gaps are documented and covered by tests. |
| S-019 source/spec audit | This document records the correspondence. The audit found no new implementation, source, test, expectation, or metadata gap requiring a new task. |
| S-022 predicate redefinition label AST follow-through | Parser task 36 and syntax task 22 are implemented together. Source and tests show `PredicateRedefinition` child order as `redefine`, `pred`, label-or-`MissingTerm`, `:`, `PredicatePattern`, `means`, `FormulaDefiniens`, optional semicolon, and `CoherenceCondition`. Active pass/fail corpus expectations cover the labeled surface and missing-label recovery. |
| S-023 predicate-label follow-up audit | This rerun records no remaining AST, accessor, snapshot, parser/syntax contract, source/spec, bilingual documentation, expectation, or metadata gap for the predicate-label repair. Documentation `design_drift` in stale roadmap/status text was closed in this task. |
| S-024 AST module-boundary refactor | The oversized public `ast` module was split into private `green`, `snapshot`, and `tests` partitions without changing public API paths, crate-root re-exports, rowan storage semantics, `SyntaxKind` numbering, typed accessors, snapshot text, trivia validation, or parser-facing builder contracts. |
| S-025 AST refactor follow-up audit | This rerun records no remaining API-list, source/test correspondence, re-export path, snapshot stability, parser/syntax boundary, source/spec, bilingual documentation, expectation, or metadata drift after the task-24 split. |

## Follow-up Records

This audit did not add a new follow-up task. Existing classified records remain:

- `MSYN-GAP-001` (`test_gap`): inactive rows in
  [parse_only_fixture_seed.md](./parse_only_fixture_seed.md) stay tied to their
  owning future parser activation points.
- `MSYN-GAP-003` (`source_drift`): recovery kinds documented as vocabulary-only
  remain constructible in `mizar-syntax`; future parser producers must update
  `recovery.md` and tests when they start emitting them.
- `MSYN-GAP-013` (`test_gap`): dotted algorithm `Lvalue` is covered by parser
  unit tests, while active `.miz` coverage waits for the owning frontend/parser
  dot-role increment that can carry the surface without unrelated diagnostics.
- S-020 is now closed by
  [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md);
  it found only documentation `design_drift` and no source/test mismatch.
- S-021 remains explicitly deferred for rustdoc summaries until its re-entry
  trigger is met.
- S-022 through S-025 are now closed. No new follow-up task was created by the
  predicate-label or AST-refactor audits.

## Parser Task 48 Post-Exit Correspondence

| Surface contract | Source/test evidence | Boundary |
|---|---|---|
| Top-level `PropertyImplementation`; append-only `SyntaxKind::PropertyImplementation = 192`; matching `SurfaceNodeKind`, `SurfaceNodeView::as_property_implementation`, snapshot/raw-kind/node-kind/rowan support | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/snapshot.rs`, and the Task 48 syntax accessor, snapshot, and raw-kind tests; parser Task 48 unit and active pass/fail corpus coverage | Syntax-only representation and parser construction; no semantic property validation |
| `DefinitionParameter -> TypeHead -> QualifiedSymbol + optional TypeArguments` | Parser Task 48 source and unit/pass/fail tests, using existing syntax node vocabulary for the nested type head | Qualified-name and type-argument source shape only; name/type resolution remains outside `mizar-syntax` |

This addendum records the syntax side of `SPEC-07-PI-PLACEMENT`. It gives no
semantic completion credit: semantic Task 39 remains deferred. It also
preserves the S-025 source/spec audit as the historical crate-exit record
rather than inventing a new syntax milestone.
