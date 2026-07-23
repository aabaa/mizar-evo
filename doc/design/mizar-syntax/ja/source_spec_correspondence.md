# ソース／仕様対応監査

## Parser Task 46 post-exit 対応

| Contract | Sourceとtest | Boundary |
|---|---|---|
| `OperatorDeclaration`、append-only `SyntaxKind::OperatorDeclaration = 193`、対応するsurface kind、typed accessor、snapshot/raw/node/rowan support | `crates/mizar-syntax/src/ast.rs`、`ast/snapshot.rs`、syntax Task-46 unit test、parser unit test、active operator-declaration pass/fail pair | syntax-only source representation。activation、resolution、precedence semanticsを含まない |

> 正本は英語です。英語版:
> [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

状態: task 24 の AST private module split と task 25 follow-up audit の後、
S-025 まで完了。新しい syntax task ID は作らず、Parser Task 48 の post-exit
correspondence を下記に追記する。

## 範囲

この監査は、完了済みの `mizar-syntax` 実装を対応する syntax design spec
（[ast.md](./ast.md)、[trivia.md](./trivia.md)、[recovery.md](./recovery.md)、
crate plan、[todo.md](./todo.md)）へ照合し、public API と implementation-facing
な挙動の約束を source と tests に trace する。

S-025 の再監査では、task 24 の module-boundary refactor を対象に、public API
list、source/test correspondence、crate-root re-export path、snapshot stability、
parser/syntax boundary promise を確認する。

これは軽量な source/spec/test 対応表であり、release coverage gate でも実行可能
テストの代替でもない。欠落した実装、古くなった spec text、欠落した test、
または新しい owner が必要な source behavior が見つかった場合は、現在挙動に
合わせて source、expectation、documentation を隠すように変更するのではなく、
分類済み follow-up として記録しなければならない。

日本語 companion は、下記の public API 名と behavior boundary を英語正本と対応
させる。より広い wording、terminology、status、link の bilingual audit は
[bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
に別途記録する。

## 結果

- parser task 35 までの `ast.md`、`trivia.md`、`recovery.md` が約束する public
  API と behavior contract について、欠落した実装は見つからなかった。
- `doc/design/mizar-syntax`、Rust tests、parser-facing `.miz` tests、既存の
  traceability metadata、または明示的 deferred / out-of-scope note に支えられて
  いない implementation-facing public behavior は見つからなかった。
- この監査では、method-level trace review 後に `SyntaxDiagnostic` builder contract
  用の targeted Rust test を 1 つ追加した。既存の `mizar-syntax` unit tests、
  lint-policy tests、parser task 4-35 unit / corpus coverage、syntax snapshot
  baseline が、残る監査対象の約束を覆っている。
- 未解決の `spec_gap`、`test_gap`、`source_drift`、
  `source_undocumented_behavior`、`test_expectation_drift`、
  `boundary_violation`、`repo_metadata_conflict` は残っていない。S-023 再監査で
  見つかった documentation `design_drift` は下で記録し、閉じた。
- 既存の follow-up record は下記の分類のまま残る: parser fixture seed activation、
  vocabulary-only future recovery producer、dotted algorithm `Lvalue` の active
  `.miz` coverage gap、S-021 deferred rustdoc summary。
- S-023 の再監査では、task 22 の predicate redefinition label repair が実装・
  テスト済みであることを確認した。`PredicateRedefinition` は
  `PredicatePattern` の前に label または `MissingTerm` の slot を所有し、parser
  task 36 はその形を送出する。active pass/fail corpus expectation は labeled case
  と missing-label case を覆い、新しい source/spec、test、expectation、metadata
  gap は残っていない。
- 再監査では、parser task 36 / syntax task 22 を未完了として扱う古い status
  text に documentation `design_drift` が見つかった。この task は parser README、
  top-level roadmap、syntax README、historical crate-exit note、bilingual audit
  record を同期し、language behavior を変更せずにその drift を閉じた。
- S-025 の再監査では、task 24 の source split が文書どおりに実装されている
  ことを確認した。`crates/mizar-syntax/src/ast.rs` は公開 `ast` module のまま、
  private helper は `src/ast/{green,snapshot,tests}.rs` に置かれ、crate-root
  re-export、`SyntaxKind` numbering、snapshot text、parser-facing builder /
  accessor contract は変わっていない。
- S-025 の再監査では、API list、source/test、re-export path、snapshot
  stability、parser/syntax boundary、bilingual documentation、expectation、
  metadata、`design_drift`、`source_drift`、`repo_metadata_conflict` の gap は
  残っていない。

## Public API 対応

| 仕様 | 確認した public API | Source | Test evidence |
|---|---|---|---|
| [ast.md](./ast.md) storage boundary | `MizarLanguage`, `RowanSyntaxNode`, `RowanSyntaxToken`, `RowanSyntaxElement`, `SyntaxKind`, `SurfaceAst`, `SurfaceAstBuilder`, `BuilderNode`, `SurfaceBuilderNodeId`, `SurfaceNodeId`, `SurfaceNode`, `SurfaceNodeView`, crate-root re-export | `crates/mizar-syntax/src/lib.rs`, `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/green.rs` | `builder_round_trips_into_rowan_backed_tree`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `repeated_construction_produces_deterministic_green_tree_and_views`, `builder_rejects_child_ids_not_created_by_this_builder`, `builder_rejects_token_sharing_between_multiple_structural_parents` |
| [ast.md](./ast.md) syntax vocabulary | `SurfaceNodeKind`, `SurfaceTokenKind`, `SurfaceToken`, `SurfaceInfixOperator`, `SurfacePrefixOperator`, `SurfacePostfixOperator`, `SurfaceOperatorAssociativity`, `SurfaceFormulaPrefixOperator`, `SurfaceFormulaConnective`, `SurfaceFormulaBinaryOperator`, `SurfaceQuantifierKind`, `SurfaceFormulaConstant`, task 35 までのすべての `SurfaceNodeView::as_*` helper と task 22 の predicate redefinition label-slot follow-through | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `typed_accessors_cover_current_node_and_token_kinds`, `task8_typed_accessors_cover_type_expression_nodes`, `task9_typed_accessors_cover_primary_term_nodes`, `task10_typed_accessors_cover_selector_and_update_nodes`, `task11_typed_accessor_covers_qua_expression`, `task12_typed_accessors_cover_prefix_and_postfix_operator_nodes`, `task13_typed_accessors_cover_atomic_formula_nodes`, `task14_typed_accessors_cover_formula_connective_and_quantifier_nodes`, `task15_typed_accessors_cover_set_comprehension_nodes`, `task16_typed_accessors_cover_simple_statement_nodes`, `task17_typed_accessors_cover_justification_nodes`, `task18_typed_accessors_cover_consider_reconsider_nodes`, `task19_typed_accessors_cover_conclusion_then_iterative_nodes`, `task20_typed_accessors_cover_block_statement_nodes`, `task21_typed_accessors_cover_inline_definition_nodes`, `task22_typed_accessors_cover_theorem_and_proof_nodes`, `task23_typed_accessors_cover_definition_nodes`, `task24_typed_accessors_cover_predicate_definition_nodes`, `task25_typed_accessors_cover_functor_definition_nodes`, `task26_typed_accessors_cover_mode_definition_nodes`, `task27_typed_accessors_cover_redefinition_and_notation_nodes`, `task22_predicate_redefinition_missing_label_snapshot_is_distinct`, `task28_typed_accessors_cover_property_clause_nodes`, `task29_typed_accessors_cover_structure_nodes`, `task30_typed_accessors_cover_registration_nodes`, `task31_typed_accessors_cover_template_nodes`, `task32_typed_accessors_cover_algorithm_nodes`, `task33_typed_accessors_cover_algorithm_control_flow_nodes`, `task34_typed_accessors_cover_algorithm_verification_nodes`, `task35_typed_accessors_cover_annotation_nodes` |
| [ast.md](./ast.md) deterministic snapshot、range、identity、reuse | `SurfaceAst::{snapshot_text, snapshot_text_with_trivia, range_contains_child_ranges, green_node, rowan_root, with_trivia, trivia}`, `SurfaceNodeKind::syntax_kind`, `SurfaceTokenKind::syntax_kind`, `SurfaceNodeView::{id, kind, syntax_kind, range, children, child_views, is_recovered}` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/snapshot.rs`, `crates/mizar-syntax/src/ast/green.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `parent_ranges_contain_child_ranges_except_recovery_attachments`, `repeated_snapshot_rendering_is_byte_identical`, `snapshot_rendering_matches_current_vocabulary_baseline`, `snapshot_payload_names_cover_current_variants`, `snapshot_rendering_includes_trivia_when_requested`, `trivia_snapshot_rendering_is_sorted_and_byte_identical`, `trivia_snapshot_target_sorting_breaks_collisions_deterministically`, `tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap` |
| [trivia.md](./trivia.md) storage、sorting、attachment | `SurfaceTrivia`, `SurfaceTriviaBuilder`, `CommentTrivia`, `DocCommentAttachment`, `SkippedTokenRange`, `WhitespaceHint`, `TriviaNodeTarget`, `TriviaAttachmentTarget`, `TriviaPlacement`, `SkippedTokenReason`, `WhitespaceHintKind` | `crates/mizar-syntax/src/trivia.rs`, attachment validation in `crates/mizar-syntax/src/ast.rs` | `trivia_builder_preserves_ownership_and_attachment_hints`, `skipped_ranges_are_preserved_with_source_ranges`, `generated_detached_anchor_must_match_trivia_source`, `doc_comment_can_attach_to_following_placeholder_item_node`, `ast_rejects_token_node_as_trivia_node_target`, `ast_rejects_non_token_trivia_token_target`, `ast_rejects_trivia_target_with_mismatched_range`, `ast_rejects_trivia_from_another_source` |
| [recovery.md](./recovery.md) diagnostics | `SyntaxDiagnostic`, `SyntaxDiagnostic::{new, with_secondary, with_recovery_note}`, `SyntaxDiagnosticCode` | `crates/mizar-syntax/src/recovery.rs`, parser producer in `crates/mizar-parser` | `syntax_diagnostic_builder_preserves_secondary_and_recovery_note`、`recovery_kinds_are_constructible_with_documented_ranges`、task 35 までの parser pass/fail corpus cases、frontend parser-seam syntax diagnostic passthrough tests |
| [recovery.md](./recovery.md) recovery vocabulary と recovered node contract | `SyntaxRecoveryKind`, `SurfaceNodeKind::ErrorRecovery`, `SurfaceTokenKind::ErrorRecovery`, `SurfaceAstBuilder::{add_recovered_token, add_recovery}` | `crates/mizar-syntax/src/recovery.rs`, `crates/mizar-syntax/src/ast.rs` | `recovery_kinds_are_constructible_with_documented_ranges`, `recovery_snapshot_names_are_unique_and_fully_fixture_backed`, `parent_ranges_contain_child_ranges_except_recovery_attachments`, task 5 と task 16-35 の parser recovery corpus coverage |
| [todo.md](./todo.md), [ast.md](./ast.md), [trivia.md](./trivia.md), [recovery.md](./recovery.md) enum policy | workspace lint opt-in、documented `allow` rationale guard、public enum forward-compatibility decision | `crates/mizar-syntax/Cargo.toml`, `crates/mizar-syntax/tests/lint_policy.rs`, `src/ast.rs`, `src/trivia.rs`, `src/recovery.rs` の public enum | `syntax_manifest_opts_into_workspace_lints`, `workspace_lint_baseline_denies_rustc_warnings_and_clippy_all`, `syntax_allow_exceptions_are_documented_inline`, `public_forward_compatible_enums_are_marked_non_exhaustive`, `public_enum_exhaustiveness_exceptions_are_documented`, `every_public_enum_has_a_forward_compatibility_decision` |

## Method-level API 対応

| API family | 確認した public method | Source | Test evidence |
|---|---|---|---|
| `SurfaceAst` accessor と storage view | `node`, `nodes`, `root`, `token_nodes`, `expression_root`, `node_view`, `root_view`, `expression_view`, `token_views`, `token_texts`, `green_node`, `rowan_root`, `trivia`, `with_trivia`, `snapshot_text`, `snapshot_text_with_trivia`, `range_contains_child_ranges` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/green.rs`, `crates/mizar-syntax/src/ast/snapshot.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `builder_round_trips_into_rowan_backed_tree`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `repeated_construction_produces_deterministic_green_tree_and_views`, `repeated_snapshot_rendering_is_byte_identical`, `snapshot_rendering_matches_current_vocabulary_baseline`, `snapshot_rendering_includes_trivia_when_requested`, `parent_ranges_contain_child_ranges_except_recovery_attachments`, trivia target rejection tests |
| `SurfaceAstBuilder` construction API | `new`, `add_node`, `add_token`, `add_recovered_token`, `add_recovery`, `node`, `node_kind`, `node_range`, `token_node_ids`, `recovery_node_ids`, `finish` | `crates/mizar-syntax/src/ast.rs` | `builder_round_trips_into_rowan_backed_tree`, `recovery_kinds_are_constructible_with_documented_ranges`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `builder_rejects_child_ids_not_created_by_this_builder`, `builder_rejects_token_sharing_between_multiple_structural_parents`, task 8-35 の全 typed-accessor tests |
| `SurfaceNodeView` typed / structural accessor | `id`, `kind`, `syntax_kind`, `range`, `children`, `is_recovered`, `as_token`, `as_infix_expression`, `as_prefix_expression`, `as_postfix_expression`, `as_recovery`, parser task 35 までの task-specific `as_*` typed helper、`child_views` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `typed_accessors_cover_current_node_and_token_kinds`, task 8-35 の各 typed-accessor test、`recovery_kinds_are_constructible_with_documented_ranges`, `recovery_snapshot_names_are_unique_and_fully_fixture_backed` |
| 互換 node / token helper | `SurfaceNode::{new, recovered, token_text}`, `SurfaceNodeKind::syntax_kind`, `SurfaceNodeKind::is_structural`, `SurfaceToken::new`, `SurfaceTokenKind::syntax_kind`, `SurfaceNodeId::index` | `crates/mizar-syntax/src/ast.rs` | `typed_accessors_cover_current_node_and_token_kinds`, `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `snapshot_payload_names_cover_current_variants`, task 8-35 typed-accessor tests |
| `SyntaxKind` raw-kind helper | `from_raw`, `is_node_kind`, `is_token_kind` | `crates/mizar-syntax/src/ast.rs`, `crates/mizar-syntax/src/ast/tests.rs` | `surface_node_raw_kinds_round_trip_through_rowan_boundary`, `typed_accessors_cover_current_node_and_token_kinds`, `snapshot_rendering_matches_current_vocabulary_baseline` |
| `SurfaceTrivia` read API | `empty`, `source_id`, `is_empty`, `comments`, `doc_comment_attachments`, `skipped_token_ranges`, `whitespace_hints` | `crates/mizar-syntax/src/trivia.rs` | `trivia_builder_preserves_ownership_and_attachment_hints`, `skipped_ranges_are_preserved_with_source_ranges`, `snapshot_rendering_includes_trivia_when_requested`, `trivia_snapshot_rendering_is_sorted_and_byte_identical` |
| `SurfaceTriviaBuilder` construction API | `new`, `add_comment`, `add_doc_comment_attachment`, `add_skipped_token_range`, `add_whitespace_hint`, `finish`; `TriviaNodeTarget::new` | `crates/mizar-syntax/src/trivia.rs` | `trivia_builder_preserves_ownership_and_attachment_hints`, `skipped_ranges_are_preserved_with_source_ranges`, `generated_detached_anchor_must_match_trivia_source`, `doc_comment_can_attach_to_following_placeholder_item_node`, trivia target rejection tests |
| `SyntaxDiagnostic` construction API | `new`, `with_secondary`, `with_recovery_note` | `crates/mizar-syntax/src/recovery.rs` | `syntax_diagnostic_builder_preserves_secondary_and_recovery_note`, task 35 までの parser pass/fail corpus cases、frontend parser-seam syntax diagnostic passthrough tests |

## Enum と診断の対応

| Surface | Current status |
|---|---|
| `SyntaxKind`, `SurfaceNodeKind`, `SurfaceTokenKind` | `ast.rs` に実装済み、`ast.md` に文書化済み、rowan raw-kind coverage と typed-accessor coverage で guard 済み。これらの enum は downstream crate に対して forward-compatible のままにする。 |
| `MizarLanguage`, `SurfaceOperatorAssociativity`, `SurfaceFormulaPrefixOperator`, `SurfaceFormulaConnective`, `SurfaceQuantifierKind`, `SurfaceFormulaConstant`, `TriviaPlacement` | 文書化された deliberate exhaustive exception。`snapshot_payload_names_cover_current_variants` と lint-policy tests が現在の closed payload vocabulary を guard する。 |
| `TriviaAttachmentTarget`, `SkippedTokenReason`, `WhitespaceHintKind` | `trivia.rs` に実装済み、`trivia.md` に文書化済み、non-exhaustive として guard 済み。sorting、same-source validation、snapshot rendering tests が現在の variant を覆う。 |
| `SyntaxRecoveryKind`, `SyntaxDiagnosticCode` | `recovery.rs` に実装済み、`recovery.md` に文書化済み、non-exhaustive として guard 済み。constructibility、snapshot-name、syntax diagnostic、parser recovery tests が現在の vocabulary を覆う。`recovery.md` で vocabulary-only とされる recovery kind は future parser-producer work であり、`mizar-syntax` の未実装ではない。 |

## タスク要件対応

| Task group | Source/test correspondence |
|---|---|
| Tasks 1-5 representation foundation | Module split、lint policy、rowan storage、builder/accessor API、deterministic snapshot、trivia side table、recovery vocabulary は `src/lib.rs`、公開 `src/ast.rs` module と private `src/ast/{green,snapshot,tests}.rs` partition、`src/trivia.rs`、`src/recovery.rs`、`tests/lint_policy.rs` に実装済み。Unit tests は builder round trip、rowan raw kind、deterministic snapshot、range rule、trivia ownership / sorting / attachment、recovery kind constructibility、diagnostics を覆う。 |
| Tasks 6-8 grammar gates | Grammar audit、parse-only acceptance matrix、fixture seed は `grammar_audit.md`、`parse_only_acceptance_matrix.md`、`parse_only_fixture_seed.md` に文書化済み。paired parser support が存在する前に final AST snapshot を固定しないことが意図されていた。 |
| S-009 module, item, and shared path nodes | Parser tasks 4-7 は syntax node / accessor / snapshot coverage と active module / import / export / visibility parse-only corpus cases とともに実装済み。 |
| S-010 type expression nodes | Parser task 8 は syntax node / accessor / snapshot coverage、malformed type recovery、active type parse-only corpus cases とともに実装済み。 |
| S-011 term nodes | Parser tasks 9-12 と 15 は term、selector/update、`qua`、operator、set-comprehension syntax coverage と active term parse-only corpus cases とともに実装済み。 |
| S-012 formula nodes | Parser tasks 13-14 は atomic formula、generic `is`、connective、quantifier、grouping、constant、formula recovery coverage とともに実装済み。 |
| S-013 statement nodes | Parser tasks 16 と 18-21 は simple statement、`consider` / `reconsider`、conclusion / `then` / iterative equality、block reasoning、inline definition とともに実装済み。 |
| S-014 theorem, proof, and justification nodes | Parser tasks 17 と 22 は justification、citation、computation、theorem / lemma、proof-block、proof recovery、active parse-only coverage とともに実装済み。 |
| S-015 definition, structure, and registration nodes | Parser tasks 23-30 は definition block、attribute / predicate / functor / mode definition、redefinition、notation alias、property、structure、inheritance、registration、recovery、active traceable corpus coverage とともに実装済み。 |
| S-016 template, algorithm, and annotation nodes | Parser tasks 31-35 は template、algorithm、claim、control flow、verification clause、annotation、recovery、syntax typed accessor、parser unit test、active `.miz` coverage、traceability metadata とともに実装済み。 |
| S-017 enum policy | 最終 enum classification は source attribute と lint-policy tests で実装・guard 済み。 |
| S-018 incremental reuse audit | Identity、raw-kind numbering、range-attached trivia / recovery reuse、localized-edit validation、annotation accessor / raw-kind gap は文書化・テスト済み。 |
| S-019 source/spec audit | この文書が対応関係を記録する。監査では、新しい implementation、source、test、expectation、metadata gap を必要とする task は見つからなかった。 |
| S-022 predicate redefinition label AST follow-through | Parser task 36 と syntax task 22 は同じ変更で実装済み。source と tests は `PredicateRedefinition` の child order が `redefine`、`pred`、label または `MissingTerm`、`:`、`PredicatePattern`、`means`、`FormulaDefiniens`、任意の semicolon、`CoherenceCondition` であることを示す。Active pass/fail corpus expectation は labeled surface と missing-label recovery を覆う。 |
| S-023 predicate-label follow-up audit | この再監査は、predicate-label repair について残る AST、accessor、snapshot、parser/syntax contract、source/spec、bilingual documentation、expectation、metadata gap がないことを記録する。古い roadmap / status text の documentation `design_drift` はこの task で閉じた。 |
| S-024 AST module-boundary refactor | oversized な公開 `ast` module を private な `green`、`snapshot`、`tests` partition へ分割した。public API path、crate-root re-export、rowan storage semantics、`SyntaxKind` numbering、typed accessor、snapshot text、trivia validation、parser-facing builder contract は変更していない。 |
| S-025 AST refactor follow-up audit | この再監査は、task 24 の分割後に API list、source/test correspondence、re-export path、snapshot stability、parser/syntax boundary、source/spec、bilingual documentation、expectation、metadata drift が残っていないことを記録する。 |

## Follow-up 記録

この監査では新しい follow-up task を追加しなかった。既存の分類済み記録は残る:

- `MSYN-GAP-001` (`test_gap`):
  [parse_only_fixture_seed.md](./parse_only_fixture_seed.md) の inactive row は、
  owning future parser activation point に紐付いたままにする。
- `MSYN-GAP-003` (`source_drift`): vocabulary-only として文書化された recovery
  kind は `mizar-syntax` で constructible のまま維持する。将来 parser producer が
  emit し始める場合は、`recovery.md` と tests を更新しなければならない。
- `MSYN-GAP-013` (`test_gap`): dotted algorithm `Lvalue` は parser unit tests で
  覆われているが、active `.miz` coverage は、その surface を unrelated diagnostic
  なしに運べる owning frontend/parser dot-role increment を待つ。
- S-020 は
  [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
  により完了済みである。見つかったのは documentation `design_drift` のみであり、
  source/test mismatch はなかった。
- S-021 は re-entry trigger が満たされるまで rustdoc summary について明示的に
  deferred のまま残る。
- S-022 から S-025 は完了済みである。predicate-label audit と AST-refactor audit
  は新しい follow-up task を作らなかった。

## Parser Task 48 post-exit 対応

| Surface contract | Source / test evidence | 境界 |
|---|---|---|
| Top-level `PropertyImplementation`、append-only な `SyntaxKind::PropertyImplementation = 192`、対応する `SurfaceNodeKind`、`SurfaceNodeView::as_property_implementation`、snapshot / raw-kind / node-kind / rowan support | `crates/mizar-syntax/src/ast.rs`、`crates/mizar-syntax/src/ast/snapshot.rs`、Task 48 の syntax accessor / snapshot / raw-kind tests、parser Task 48 unit test と active pass / fail corpus coverage | syntax-only representation と parser construction。semantic property validation は含まない |
| `DefinitionParameter -> TypeHead -> QualifiedSymbol + optional TypeArguments` | 既存 syntax node vocabulary を nested type head に使う Parser Task 48 source と unit / pass / fail tests | qualified-name と type-argument の source shape だけを保持し、name / type resolution は `mizar-syntax` の外に残す |

この addendum は `SPEC-07-PI-PLACEMENT` の syntax 側を記録する。semantic completion
の credit は与えず、semantic Task 39 は deferred のまま残す。また、新しい syntax
milestone を作るのではなく、S-025 source/spec audit を historical crate-exit record
として保持する。
