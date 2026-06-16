# mizar-syntax Task 7: Parse-Only Acceptance Matrix

> Canonical language: English. Japanese companion:
> [../ja/parse_only_acceptance_matrix.md](../ja/parse_only_acceptance_matrix.md).

Status: completed for the Task 7 grammar consistency gate on 2026-06-13.

## Purpose

This note defines the parse-only acceptance matrix that must exist before
`SurfaceAst` snapshot shape is designed. It converts the Task 6 grammar audit
inputs into fixture candidates that check only syntax: whether a source shape
is accepted, rejected, or recovered by the parser, and which Appendix A grammar
rule is being exercised.

These expectations deliberately do not depend on final AST node kinds, child
roles, range layout, or snapshot text. Later AST snapshots inherit this stable
fixture set only after the corresponding node vocabulary increment documents
the tree shape.

## Result Vocabulary

| Result | Meaning |
|---|---|
| `accept` | The parser accepts the source through the named grammar rule with no syntax error expected. Semantic invalidity is out of scope. |
| `reject` | The parser must reject the source at parse phase because no Appendix A production admits the token sequence. |
| `ambiguous-preserve-surface` | The parser accepts the source while preserving a syntax-only surface form for a later phase or parser task to classify. |
| `recover` | The parser emits a syntax diagnostic and a recovered surface tree, using the recovery category named in the row. |

`ambiguous-preserve-surface` is a matrix classification, not a new
`.expect.toml` outcome. When one of these rows becomes an executable
`mizar-test` case, it maps to syntactic acceptance with `expected_outcome =
"pass"` and `expected_phase = "parse"`; the surface-preservation note guides
later AST design.

Recovery categories are intentionally coarse here: `missing_end`,
`missing_delimiter`, `missing_separator`, `malformed_annotation`,
`misplaced_item`, `unexpected_token`, and `incomplete_construct`. The concrete
`SyntaxRecoveryKind` mapping remains owned by parser recovery implementation
tasks and `recovery.md`.

## Fixture Ownership

| Owner | Use |
|---|---|
| `mizar-parser` | Small grammar-unit tests and recovery tests that can run inside the parser crate before a full corpus runner is available. |
| `mizar-test` | Executable `.miz` corpus cases under `tests/miz/{pass,fail}/parser/` with `.expect.toml` sidecars, `stage = "parse_only"`, `expected_phase = "parse"`, and `spec_refs` back to `tests/coverage/spec_trace.toml`. |
| `pure-spec` | Illustrative examples or semantic-only ambiguity notes that document the intended surface but should not enter the executable corpus until prerequisites are implemented. |

Task 8 should seed only a small executable subset. Rows marked `pure-spec` are
still part of the matrix, but they are not default-discovered fixtures until
the owning parser task can parse the surrounding construct.

## Acceptance Matrix

| Area | Case | Category | Example shape | Expected parse-only result | Target grammar rules | Owner | Appendix A trace |
|---|---|---|---|---|---|---|---|
| Module structure | `PO-MOD-P01` | positive | `import ..Core.Algebra.{Group, Ring}; export Core.Algebra; theorem T: thesis;` | `accept` | `compilation_unit`, `import_prelude`, `export_prelude`, `annotated_declaration` | `mizar-test` | A.12, A.16 |
| Module structure | `PO-MOD-N01` | negative | `theorem T: thesis; import Late.Module;` | `reject`: import statements are allowed only in `import_prelude` before declarations. | `compilation_unit`, `import_prelude` | `mizar-parser` | A.12 |
| Module structure | `PO-MOD-A01` | ambiguous | `import A.B; theorem T: thesis by A.B.C;` | `ambiguous-preserve-surface`: dot-separated module paths and qualified references remain syntax-only until resolution. | `module_path`, `qualified_reference`, `namespace_path`, `simple_justification` | `mizar-test` | A.2, A.12, A.15 |
| Module structure | `PO-MOD-R01` | recovery-required | `definition theorem T: thesis;` | `recover`: `missing_end` for the unterminated `definition_block`. | `definition_block`, `declaration` | `mizar-parser` | A.12 |
| Module structure | `PO-MOD-R02` | recovery-required | `theorem T: thesis; import Late.Module; theorem U: thesis;` | `recover`: `misplaced_item` or `unexpected_token` for an import that appears after declarations, with synchronization before the next declaration. | `compilation_unit`, `import_prelude`, `annotated_declaration` | `mizar-parser` | A.12 |
| Declarations | `PO-DECL-P01` | positive | `definition let x be set; pred P: x R x means thesis; end;` | `accept` | `definition_block`, `definition_parameter_decl`, `pred_def` | `mizar-test` | A.9, A.12 |
| Declarations | `PO-DECL-N01` | negative | `definition pred P: x in y means thesis; end;` | `reject`: built-in `in` is a reserved predicate application token, not `def_predicate_symbol`. | `pred_def`, `pred_pattern`, `def_predicate_symbol` | `mizar-parser` | A.2, A.9 |
| Declarations | `PO-DECL-N02` | negative | `definition pred P: x = y means thesis; end;` | `reject`: built-in `=` is a reserved predicate application token, not `def_predicate_symbol`. | `pred_def`, `pred_pattern`, `def_predicate_symbol` | `mizar-parser` | A.2, A.9 |
| Declarations | `PO-DECL-N03` | negative | `definition redefine pred x <> y means thesis; coherence by A; end;` | `reject`: built-in `<>` cannot be used as the predicate symbol in `redefine_pred`. | `redefine_pred`, `pred_pattern`, `def_predicate_symbol` | `mizar-parser` | A.2, A.9, A.12 |
| Declarations | `PO-DECL-A01` | ambiguous | `definition let T be type; pred P: R[T] means thesis; end;` | `ambiguous-preserve-surface`: ordinary definition and template definition share `definition ... end;` and must not force AST category yet. | `definition_block`, `definition_parameter_decl`, `template_definition` | `pure-spec` | A.12, A.18 |
| Declarations | `PO-DECL-R01` | recovery-required | `definition func F -> set equals x end;` | `recover`: `missing_separator` for the missing semicolon before `end`. | `definition_block`, `func_def`, `term_definiens` | `mizar-parser` | A.10, A.12 |
| Declarations | `PO-DECL-P02` | positive | `definition symmetry by A; commutativity proof thus thesis; end; sethood by computation(steps: 1); end;` | `accept` | `definition_block`, `property_item`, `pred_property`, `func_property`, `mode_property`, `justification` | `mizar-test` | A.7, A.9, A.10, A.12 |
| Declarations | `PO-DECL-R02` | recovery-required | `definition symmetry; reflexivity proof end irreflexivity by A; end;` | `recover`: `incomplete_construct` for missing property justifications and `missing_separator` before a following property item. | `definition_block`, `property_item`, `justification` | `mizar-test` | A.7, A.9, A.10, A.12 |
| Declarations | `PO-DECL-P03` | positive | `definition struct S[T] where field carrier -> set; property unit -> Element of carrier; end; inherit S[T] extends set where field carrier from it; coherence by A; end; end;` | `accept` | `definition_block`, `struct_def`, `field_decl`, `property_decl`, `inherit_def`, `field_redef`, `coherence_block` | `mizar-test` | A.5, A.12 |
| Declarations | `PO-DECL-R03` | recovery-required | `definition struct Empty where end; inherit Child extends Parent where coherence with C; end; end;` | `recover`: `incomplete_construct` for empty structure/explicit inheritance members and malformed inheritance coherence justification. | `definition_block`, `struct_def`, `inherit_def`, `inherit_member`, `coherence_block` | `mizar-test` | A.5, A.12 |
| Type expressions | `PO-TYPE-P01` | positive | `reserve x for non empty T of a, b;` | `accept` | `reserve_decl`, `type_expression`, `attribute_chain`, `type_args` | `mizar-test` | A.3, A.4 |
| Type expressions | `PO-TYPE-N01` | negative | `reserve x for non;` | `reject`: `non` cannot terminate `attribute_chain` without `attribute_ref` and `type_head`. | `type_expression`, `attribute_chain`, `type_head` | `mizar-parser` | A.3 |
| Type expressions | `PO-TYPE-A01` | ambiguous | `reserve x for Foo[T];` | `ambiguous-preserve-surface`: `type_head` may be radix or mode until symbol classification. | `type_head`, `radix_type`, `mode_type`, `type_args` | `pure-spec` | A.3 |
| Type expressions | `PO-TYPE-R01` | recovery-required | `reserve x for Foo[set, object;` | `recover`: `missing_delimiter` for the unclosed type argument list. | `type_args`, `type_arg_list` | `mizar-parser` | A.3, A.4 |
| Term expressions | `PO-TERM-P01` | positive | `theorem T: thesis proof set x = the non empty T with (f := y); thus thesis; end;` | `accept` | `proof`, `constant_definition`, `choice_expression`, `term_postfix`, `field_update_list` | `mizar-test` | A.13, A.15, A.16 |
| Term expressions | `PO-TERM-P02` | positive | `theorem T: thesis proof set x = (a qua R) qua S; thus thesis; end;` | `accept` | `proof`, `constant_definition`, `qua_expression`, `type_expression` | `mizar-test` | A.13, A.15, A.16 |
| Term expressions | `PO-TERM-N01` | negative | `set x = with (f := y);` | `reject`: `with (...)` is a postfix update and cannot start `term_primary`. | `term_expression`, `term_primary`, `term_postfix` | `mizar-parser` | A.13, A.15 |
| Term expressions | `PO-TERM-A01` | ambiguous | `set x = A.B.c;` | `ambiguous-preserve-surface`: dotted surface may later be selector access, namespace qualification, or active user-symbol shape. | `qualified_symbol`, `field_access`, `term_postfix`, `namespace_path` | `pure-spec` | A.2, A.3, A.13, A.15 |
| Term expressions | `PO-TERM-A02` | ambiguous | `set x = a . b;` | `ambiguous-preserve-surface`: active `.` user-symbol tokenization must remain distinguishable from selector or namespace dot handling. | `operator_expression`, `functor_application`, `term_postfix` | `pure-spec` | A.2, A.13 |
| Term expressions | `PO-TERM-R01` | recovery-required | `set x = F(a, b;` | `recover`: `missing_delimiter` for the unclosed argument list. | `inline_functor_application`, `term_list` | `mizar-parser` | A.13, A.15 |
| Formulas | `PO-FORM-P01` | positive | `theorem T: x in X & y = z & y <> w;` | `accept` | `theorem_item`, `formula`, `builtin_predicate_application`, `and_formula` | `mizar-test` | A.9, A.14, A.16 |
| Formulas | `PO-FORM-P02` | positive | `theorem T: x is divisible(2);` | `accept` | `theorem_item`, `is_assertion`, `attribute_test_chain`, `attribute_ref`, `argument_list` | `mizar-test` | A.3, A.14, A.16 |
| Formulas | `PO-FORM-N01` | negative | `theorem T: P iff Q iff R;` | `reject`: unparenthesized `iff` chaining is non-associative. | `iff_formula` | `mizar-parser` | A.14 |
| Formulas | `PO-FORM-N02` | negative | `theorem T: x is non ;` | `reject`: `non` cannot terminate `is_assertion_body` without an `attribute_ref` or `type_expression`. | `is_assertion`, `is_assertion_body`, `attribute_test_chain`, `type_expression` | `mizar-parser` | A.3, A.14 |
| Formulas | `PO-FORM-A01` | ambiguous | `theorem T: x is T;` | `ambiguous-preserve-surface`: `is_assertion` stays generic until resolution decides type versus attribute assertion. | `is_assertion`, `is_assertion_body`, `attribute_test_chain`, `type_expression` | `mizar-test` | A.3, A.14 |
| Formulas | `PO-FORM-A02` | ambiguous | `theorem T: x is non empty;` | `ambiguous-preserve-surface`: attribute-test bodies stay under generic `is_assertion` until resolution. | `is_assertion`, `attribute_test_chain`, `attribute_ref` | `mizar-test` | A.3, A.14 |
| Formulas | `PO-FORM-A03` | ambiguous | `theorem T: x is Algebra.non EmptySet;` | `ambiguous-preserve-surface`: qualified attribute/type spellings do not force parse-time classification. | `is_assertion`, `qualified_symbol`, `type_expression`, `attribute_test_chain` | `pure-spec` | A.3, A.14 |
| Formulas | `PO-FORM-R01` | recovery-required | `theorem T: for x be set holds ;` | `recover`: `incomplete_construct` for a missing formula after `holds`. | `universal_formula`, `formula` | `mizar-parser` | A.14, A.16 |
| Statements/proofs | `PO-STMT-P01` | positive | `theorem T: thesis proof assume A: thesis; thus thesis; end;` | `accept` | `theorem_item`, `proof`, `reasoning`, `assumption`, `conclusion` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-P02` | positive | `theorem T: thesis proof reconsider x as set by A; thus thesis; end;` | `accept` | `proof`, `type_changing_statement`, `type_change_list`, `simple_justification` | `mizar-test` | A.4, A.15, A.16 |
| Statements/proofs | `PO-STMT-P03` | positive | `theorem T: thesis proof thus thesis by A.{B, C}, A.*; end;` | `accept` | `conclusion`, `references`, `grouped_reference`, `bulk_reference` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-P04` | positive | `theorem T: thesis proof A: x = y by B; then thus thesis; end;` | `accept` | `proposition`, `compact_statement`, `statement`, `conclusion` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-P05` | positive | `theorem T: thesis proof then A: x = y by B .= z by C; end;` | `accept` | `statement`, `iterative_equality`, `simple_justification` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-N01` | negative | `theorem T: thesis proof then let x be set; end;` | `reject`: `then` applies to `linkable_statement`, not `generalization`. | `statement`, `linkable_statement`, `generalization` | `mizar-parser` | A.15 |
| Statements/proofs | `PO-STMT-A01` | ambiguous | `theorem T: thesis proof x = y by A; end;` | `ambiguous-preserve-surface`: compact statement versus zero-continuation iterative equality must be decided by parser design without semantic facts. | `compact_statement`, `iterative_equality`, `simple_justification` | `pure-spec` | A.15 |
| Statements/proofs | `PO-STMT-R01` | recovery-required | `theorem T: thesis proof assume A: thesis;` | `recover`: `missing_end` for an unterminated `proof`. | `proof`, `reasoning` | `mizar-parser` | A.15, A.16 |
| Annotations | `PO-ANN-P01` | positive | `@proof_hint(max_axioms: 10, solver: vampire) theorem T: thesis;` | `accept` | `annotated_declaration`, `annotation`, `proof_hint_annotation` | `mizar-test` | A.12, A.21 |
| Annotations | `PO-ANN-P02` | positive | `definition @custom(flag) theorem T: thesis; end;` | `accept` | `definition_block`, `definition_content`, `annotation`, `theorem_item` | `mizar-test` | A.12, A.16, A.21 |
| Annotations | `PO-ANN-P03` | positive | `registration @custom(flag) cluster C: non empty set; existence by A; end;` | `accept` | `registration_block`, `registration_content`, `annotation`, `registration_item` | `mizar-test` | A.17, A.21 |
| Annotations | `PO-ANN-P04` | positive | `theorem T: thesis proof @custom(flag) thus thesis; end;` | `accept` | `proof`, `annotated_statement`, `annotation`, `conclusion` | `mizar-test` | A.15, A.16, A.21 |
| Annotations | `PO-ANN-P05` | positive | `definition algorithm f() do @custom(flag) return; end; end;` | `accept` | `algorithm_def`, `annotated_algo_statement`, `annotation`, `return_stmt` | `mizar-test` | A.20, A.21 |
| Annotations | `PO-ANN-P06` | positive | `claim C do @custom(flag) theorem T: thesis; end;` | `accept`: deferred until parser task 35; task 32 rejects claim-local annotation prefixes as recovery input. | `claim_block`, `annotated_theorem_item`, `annotation`, `theorem_item` | `mizar-test` | A.20, A.21 |
| Annotations | `PO-ANN-N01` | negative | `@latex(123) theorem T: thesis;` | `reject`: fixed `@latex` annotations require a string literal argument. | `annotated_declaration`, `annotation`, `latex_annotation` | `mizar-parser` | A.12, A.21 |
| Annotations | `PO-ANN-A01` | ambiguous | `@custom(flag, 3) theorem T: thesis;` | `ambiguous-preserve-surface`: generic annotation names and contextual argument validity are semantic or registry checks. | `annotated_declaration`, `annotation`, `statement_annotation`, `generic_annotation_name`, `annotation_args` | `mizar-test` | A.12, A.21 |
| Annotations | `PO-ANN-R01` | recovery-required | `@proof_hint(max_axioms: ) theorem T: thesis;` | `recover`: `malformed_annotation` for a missing option value. | `proof_hint_annotation`, `proof_hint_option` | `mizar-parser` | A.21 |
| Registrations | `PO-REG-P01` | positive | `registration let x be set; cluster C: non empty set; existence by A; end;` | `accept` | `registration_block`, `registration_content`, `existential_registration` | `mizar-test` | A.12, A.17 |
| Registrations | `PO-REG-N01` | negative | `registration cluster C: -> non empty for set; coherence by A; end;` | `reject`: conditional registration requires non-empty antecedent adjectives before `->`. | `conditional_registration`, `antecedent_adjectives`, `adjective_list` | `mizar-parser` | A.17 |
| Registrations | `PO-REG-A01` | ambiguous | `registration cluster C: n-dimensional V -> non empty for T; coherence by A; end;` | `ambiguous-preserve-surface`: `param_prefix` split versus active attribute spelling is parser-assisted but semantic classification remains later. | `adjective`, `param_prefix`, `attribute_name` | `pure-spec` | A.2, A.17 |
| Registrations | `PO-REG-R01` | recovery-required | `registration cluster C: non empty set; existence by A;` | `recover`: `missing_end` for an unterminated `registration_block`. | `registration_block`, `registration_item` | `mizar-parser` | A.17 |
| Templates | `PO-TPL-P01` | positive | `definition let T be type; theorem Id: thesis; end;` | `accept` | `template_definition`, `template_parameter_decl`, `let_type`, `template_item` | `mizar-test` | A.18 |
| Templates | `PO-TPL-P02` | positive | `definition let P be pred(set); theorem Id: thesis; end;` | `accept` | `template_definition`, `definition_parameter_decl`, `pred_param`, `type_list` | `mizar-test` | A.18 |
| Templates | `PO-TPL-P03` | positive | `definition let F be func(set) -> set; theorem Id: thesis; end;` | `accept` | `template_definition`, `definition_parameter_decl`, `func_param`, `type_list`, `type_expression` | `mizar-test` | A.18 |
| Templates | `PO-TPL-N01` | negative | `definition let F be func(set); theorem T: thesis; end;` | `reject`: `func_param` requires `-> type_expression`. | `func_param`, `type_list` | `mizar-parser` | A.18 |
| Templates | `PO-TPL-A01` | ambiguous | `theorem T: thesis proof thus thesis by Scheme[T, x qua R], A; end;` | `ambiguous-preserve-surface`: scheme application currently overlaps with `simple_justification` plus `reference [ template_args ]`; `param_name`, `type_arg_list`, `type_arg`, and `qua_arg` must stay explicit fixture-planning inputs until the helper is removed or repurposed. | `scheme_app`, `param_name`, `simple_justification`, `reference`, `template_args`, `template_arg`, `qua_arg` | `pure-spec` | A.3, A.15, A.18 |
| Templates | `PO-TPL-R01` | recovery-required | `theorem T: thesis by P[set, x qua R;` | `recover`: `missing_delimiter` for unclosed `template_args`. | `reference`, `template_args`, `template_arg`, `qua_arg` | `mizar-parser` | A.3, A.15, A.18 |
| Algorithms | `PO-ALG-P01` | positive | `definition algorithm f(x) -> set do var y := x; return y; end; end;` | `accept`: header contracts such as `requires` are a task-34 extension. | `definition_block`, `algorithm_def`, `algorithm_body`, `var_decl`, `return_stmt` | `mizar-test` | A.12, A.20 |
| Algorithms | `PO-ALG-P02` | positive | `definition algorithm f() do if thesis do break; else continue; end; while thesis do break; end; for i = a to b step s do continue; end; for j = b downto a do break; end; for x in S processed V do continue; end; for y in S do break; end; match t do case p do break; end; case q do continue; end; otherwise break; end; end; match u do case r do continue; end; exhaustive; end; end; end;` | `accept`: parser task 33 covers control-flow statements; loop verification clauses remain task 34. | `if_stmt`, `while_stmt`, `for_range_stmt`, `for_collection_stmt`, `match_stmt`, `match_case`, `match_ending`, `break_stmt`, `continue_stmt` | `mizar-test` | A.20 |
| Algorithms | `PO-ALG-N01` | negative | `definition algorithm f() do claim C do theorem T: thesis; end; end; end;` | `reject`: `claim_block` is top-level declaration content, not `algo_statement`. | `algorithm_def`, `algo_statement`, `claim_block` | `mizar-parser` | A.12, A.20 |
| Algorithms | `PO-ALG-A01` | ambiguous | `definition algorithm f() do x.y := z; snapshot S; end; end;` | `ambiguous-preserve-surface`: dotted `lvalue` stays syntactic until selector and namespace roles are resolved. | `assignment`, `lvalue`, `snapshot_stmt` | `pure-spec` | A.2, A.20 |
| Algorithms | `PO-ALG-R01` | recovery-required | `definition algorithm f() do if thesis do return; end;` | `recover`: `missing_end` for the surrounding `algorithm_body` or `definition_block` after a nested control block. | `algorithm_def`, `if_stmt`, `algorithm_body` | `mizar-parser` | A.20 |
| Algorithms | `PO-ALG-R02` | recovery-required | `definition algorithm f() do claim C do theorem T: thesis; end; return; end; end;` | `recover`: `misplaced_item` for a top-level `claim_block` encountered inside `algo_statement_list`, with synchronization before the following algorithm statement. | `algorithm_def`, `algo_statement_list`, `algo_statement`, `claim_block` | `mizar-parser` | A.12, A.20 |

## Cross-Cutting Fixture Requirements

- Built-in predicate coverage must include accepting applications for `in`,
  `=`, and `<>`, plus rejecting `pred` or `redefine pred` attempts that use
  those built-ins as definition symbols.
- Annotation attachment coverage must appear at module level, inside
  `definition_block`, inside `registration_block`, inside proofs, inside
  algorithm bodies, and inside `claim_block` theorem lists. The matrix rows
  above select representative owners; later parser tasks should add the
  remaining attachment sites when their enclosing productions land. Claim-block
  annotation attachment remains deferred until parser task 35.
- Dot-chain coverage must include module paths, qualified symbols, qualified
  references, grouped references, bulk references, selector access/update,
  algorithm lvalues, and active `.` user-symbol cases. Parse-only expectations
  must stay syntax-only. Active `.miz` coverage for dotted algorithm lvalues is
  deferred until frontend dot-role disambiguation can carry that surface through
  the parse-only corpus; parser-unit coverage may pin the task-32 `Lvalue`
  surface earlier.
- Term boundary coverage must include `the type_expression`, parenthesized and
  unparenthesized `qua`, chained `qua`, and `with (...)` update boundaries.
- Statement coverage must include `x = y by A;`,
  `x = y by A .= z by B;`, labels, and `then` variants before statement AST
  snapshots are introduced; `PO-STMT-A01`, `PO-STMT-P04`, and `PO-STMT-P05`
  provide the row-level anchors.
- Recovery coverage must include missing `end`, missing delimiters, malformed
  annotations, misplaced `claim`, and misplaced `import`. Misplaced forms may
  also have strict `reject` rows; recovery rows are required when the parser is
  expected to continue to a following declaration or statement.

## Fixture Activation Plan

Task 8 should not try to activate every row at once. The initial seed should
prefer high-risk, low-prerequisite cases:

1. Module prelude ordering and missing `end` recovery:
   `PO-MOD-N01`, `PO-MOD-R01`, `PO-MOD-R02`.
2. Built-in predicate acceptance/rejection and generic `is_assertion`:
   `PO-FORM-P01`, `PO-FORM-A01`, `PO-DECL-N01`.
3. Dot and term/formula boundaries:
   `PO-TERM-A01`, `PO-TERM-P01`, `PO-TYPE-A01`.
4. Compact statement versus iterative equality:
   `PO-STMT-A01`, `PO-STMT-P05`.
5. Malformed annotations and misplaced `claim`:
   `PO-ANN-R01`, `PO-ALG-N01`, `PO-ALG-R02`.

For executable `mizar-test` corpus cases, expectation sidecars use:

```toml
stage = "parse_only"
expected_phase = "parse"
```

Accepting cases use `expected_outcome = "pass"` with
`diagnostic_codes = []`. Rejecting and recovery cases use
`expected_outcome = "fail"`, `failure_category = "syntax_error"`, and stable
parser-owned `rejection_reason` / `stable_detail_key` values. Snapshot
profiles stay absent until AST snapshot design lands.

## Task 8 Seed Status

Task 8 records the initial seed in
[parse_only_fixture_seed.md](./parse_only_fixture_seed.md). The seed keeps the
Fixture Activation Plan case IDs and source shapes stable, adds supplemental
rows for `qua`, `reconsider`, and the string-required annotation rejection
boundary, and stays as a checked-in manifest until the parse-only corpus runner
and owning grammar productions can execute the cases. No AST snapshot
expectation is introduced.

## Appendix A Traceability Summary

| Appendix A section | Matrix areas |
|---|---|
| A.2 Lexical Structure | Module dot roles, type parameter prefixes, term dot roles, algorithm lvalues |
| A.3 Type Expressions | Type expressions, formulas through `is_assertion`, template `qua_arg` |
| A.4 Variables and Constants | Declaration and statement bindings |
| A.5 Structures | Structure definitions, structure members, inheritance mappings |
| A.9 Predicates | Declarations and formulas, especially built-in predicate boundaries |
| A.10 Functors and Operator Declarations | Function definitions and term/operator boundaries |
| A.12 Modules and Namespaces | Module structure, declarations, annotations, algorithms inside definitions |
| A.13 Term Expressions | Term boundaries, `the`, `qua`, field access/update |
| A.14 Formulas | Formula precedence, non-associative `iff`, generic `is_assertion` |
| A.15 Statements, Proofs, and References | Proof statements, justifications, references, template arguments |
| A.16 Theorems and Correctness Blocks | Top-level theorem and proof attachment |
| A.17 Clusters and Registrations | Registration syntax and adjective boundaries |
| A.18 Templates | Template parameters, template items, scheme/helper follow-ups |
| A.20 Algorithms and Computation | Algorithm definitions, control statements, claim placement |
| A.21 Annotations | Attaching annotations and standalone diagnostic annotations |
