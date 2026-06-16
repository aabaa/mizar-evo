# mizar-syntax Task 7: parse-only acceptance matrix

> 正本は英語です。英語版:
> [../en/parse_only_acceptance_matrix.md](../en/parse_only_acceptance_matrix.md)。

状態: 2026-06-13 時点で Task 7 の文法整合性ゲートとして完了。

## 目的

このノートは、`SurfaceAst` snapshot の形を設計する前に必要な parse-only
acceptance matrix を定義する。Task 6 の文法監査 input を、構文だけを確認する
fixture 候補へ変換する。確認対象は、ソース形状が parser に受理されるか、
拒否されるか、回復されるか、およびどの Appendix A grammar rule を対象にするか
だけである。

これらの期待値は、最終的な AST node kind、child role、range layout、snapshot
text に依存しない。後続の AST snapshot は、対応する node vocabulary 増分が tree
shape を文書化した後でのみ、この安定した fixture set を継承する。

## 結果語彙

| 結果 | 意味 |
|---|---|
| `accept` | parser が、指定された grammar rule に沿って source を syntax error なしで受理する。意味的な不正は範囲外である。 |
| `reject` | Appendix A の production が token sequence を受理しないため、parser が parse phase で拒否する。 |
| `ambiguous-preserve-surface` | parser は source を受理するが、後続 phase または parser task が分類できるよう、syntax-only な surface form を保持する。 |
| `recover` | parser が構文診断と recovered surface tree を出し、行で指定した recovery category を使う。 |

`ambiguous-preserve-surface` は matrix 上の分類であり、新しい `.expect.toml`
outcome ではない。この分類の行が executable な `mizar-test` case になる場合は、
`expected_outcome = "pass"`、`expected_phase = "parse"` の構文受理へ対応し、
surface-preservation の注記は後続 AST 設計のための指針になる。

recovery category はここでは意図的に粗い:
`missing_end`、`missing_delimiter`、`missing_separator`、
`malformed_annotation`、`misplaced_item`、`unexpected_token`、
`incomplete_construct`。具体的な `SyntaxRecoveryKind` への対応は、parser recovery
実装 task と `recovery.md` が所有する。

## Fixture の所属

| 所属 | 用途 |
|---|---|
| `mizar-parser` | 完全な corpus runner が利用可能になる前に parser crate 内で実行できる、小さな grammar-unit test と recovery test。 |
| `mizar-test` | `tests/miz/{pass,fail}/parser/` 配下の executable `.miz` corpus case。`.expect.toml` sidecar、`stage = "parse_only"`、`expected_phase = "parse"`、および `tests/coverage/spec_trace.toml` への `spec_refs` を持つ。 |
| `pure-spec` | 意図した surface を文書化する説明例、または semantic-only な曖昧性メモ。前提となる実装が着地するまでは executable corpus に入れない。 |

Task 8 は全行を一度に有効化しようとしない。`pure-spec` と印付けされた行も matrix
の一部だが、所有する parser task が周辺 construct を parse できるまでは default
discovery の fixture ではない。

## Acceptance Matrix

| 領域 | Case | 分類 | 例の形 | 期待される parse-only 結果 | 対象 grammar rule | 所属 | Appendix A trace |
|---|---|---|---|---|---|---|---|
| Module structure | `PO-MOD-P01` | positive | `import ..Core.Algebra.{Group, Ring}; export Core.Algebra; theorem T: thesis;` | `accept` | `compilation_unit`, `import_prelude`, `export_prelude`, `annotated_declaration` | `mizar-test` | A.12, A.16 |
| Module structure | `PO-MOD-N01` | negative | `theorem T: thesis; import Late.Module;` | `reject`: import statement は declaration より前の `import_prelude` でのみ許される。 | `compilation_unit`, `import_prelude` | `mizar-parser` | A.12 |
| Module structure | `PO-MOD-A01` | ambiguous | `import A.B; theorem T: thesis by A.B.C;` | `ambiguous-preserve-surface`: dot 区切りの module path と qualified reference は resolution まで syntax-only のまま保持する。 | `module_path`, `qualified_reference`, `namespace_path`, `simple_justification` | `mizar-test` | A.2, A.12, A.15 |
| Module structure | `PO-MOD-R01` | recovery-required | `definition theorem T: thesis;` | `recover`: 終端していない `definition_block` に対する `missing_end`。 | `definition_block`, `declaration` | `mizar-parser` | A.12 |
| Module structure | `PO-MOD-R02` | recovery-required | `theorem T: thesis; import Late.Module; theorem U: thesis;` | `recover`: declaration 後に現れた import に対する `misplaced_item` または `unexpected_token`。次の declaration の前で同期する。 | `compilation_unit`, `import_prelude`, `annotated_declaration` | `mizar-parser` | A.12 |
| Declarations | `PO-DECL-P01` | positive | `definition let x be set; pred P: x R x means thesis; end;` | `accept` | `definition_block`, `definition_parameter_decl`, `pred_def` | `mizar-test` | A.9, A.12 |
| Declarations | `PO-DECL-N01` | negative | `definition pred P: x in y means thesis; end;` | `reject`: built-in の `in` は予約された predicate application token であり、`def_predicate_symbol` ではない。 | `pred_def`, `pred_pattern`, `def_predicate_symbol` | `mizar-parser` | A.2, A.9 |
| Declarations | `PO-DECL-N02` | negative | `definition pred P: x = y means thesis; end;` | `reject`: built-in の `=` は予約された predicate application token であり、`def_predicate_symbol` ではない。 | `pred_def`, `pred_pattern`, `def_predicate_symbol` | `mizar-parser` | A.2, A.9 |
| Declarations | `PO-DECL-N03` | negative | `definition redefine pred x <> y means thesis; coherence by A; end;` | `reject`: built-in の `<>` は `redefine_pred` の predicate symbol として使えない。 | `redefine_pred`, `pred_pattern`, `def_predicate_symbol` | `mizar-parser` | A.2, A.9, A.12 |
| Declarations | `PO-DECL-A01` | ambiguous | `definition let T be type; pred P: R[T] means thesis; end;` | `ambiguous-preserve-surface`: ordinary definition と template definition は `definition ... end;` を共有するため、まだ AST category を強制しない。 | `definition_block`, `definition_parameter_decl`, `template_definition` | `pure-spec` | A.12, A.18 |
| Declarations | `PO-DECL-R01` | recovery-required | `definition func F -> set equals x end;` | `recover`: `end` の前に欠けた semicolon に対する `missing_separator`。 | `definition_block`, `func_def`, `term_definiens` | `mizar-parser` | A.10, A.12 |
| Declarations | `PO-DECL-P02` | positive | `definition symmetry by A; commutativity proof thus thesis; end; sethood by computation(steps: 1); end;` | `accept` | `definition_block`, `property_item`, `pred_property`, `func_property`, `mode_property`, `justification` | `mizar-test` | A.7, A.9, A.10, A.12 |
| Declarations | `PO-DECL-R02` | recovery-required | `definition symmetry; reflexivity proof end irreflexivity by A; end;` | `recover`: property justification 欠落に対する `incomplete_construct` と、後続 property item の前に欠けた semicolon に対する `missing_separator`。 | `definition_block`, `property_item`, `justification` | `mizar-test` | A.7, A.9, A.10, A.12 |
| Type expressions | `PO-TYPE-P01` | positive | `reserve x for non empty T of a, b;` | `accept` | `reserve_decl`, `type_expression`, `attribute_chain`, `type_args` | `mizar-test` | A.3, A.4 |
| Type expressions | `PO-TYPE-N01` | negative | `reserve x for non;` | `reject`: `non` は `attribute_ref` と `type_head` なしに `attribute_chain` を終えられない。 | `type_expression`, `attribute_chain`, `type_head` | `mizar-parser` | A.3 |
| Type expressions | `PO-TYPE-A01` | ambiguous | `reserve x for Foo[T];` | `ambiguous-preserve-surface`: `type_head` は symbol classification まで radix か mode かを決めない。 | `type_head`, `radix_type`, `mode_type`, `type_args` | `pure-spec` | A.3 |
| Type expressions | `PO-TYPE-R01` | recovery-required | `reserve x for Foo[set, object;` | `recover`: 閉じていない type argument list に対する `missing_delimiter`。 | `type_args`, `type_arg_list` | `mizar-parser` | A.3, A.4 |
| Term expressions | `PO-TERM-P01` | positive | `theorem T: thesis proof set x = the non empty T with (f := y); thus thesis; end;` | `accept` | `proof`, `constant_definition`, `choice_expression`, `term_postfix`, `field_update_list` | `mizar-test` | A.13, A.15, A.16 |
| Term expressions | `PO-TERM-P02` | positive | `theorem T: thesis proof set x = (a qua R) qua S; thus thesis; end;` | `accept` | `proof`, `constant_definition`, `qua_expression`, `type_expression` | `mizar-test` | A.13, A.15, A.16 |
| Term expressions | `PO-TERM-N01` | negative | `set x = with (f := y);` | `reject`: `with (...)` は postfix update であり、`term_primary` を開始できない。 | `term_expression`, `term_primary`, `term_postfix` | `mizar-parser` | A.13, A.15 |
| Term expressions | `PO-TERM-A01` | ambiguous | `set x = A.B.c;` | `ambiguous-preserve-surface`: dotted surface は後で selector access、namespace qualification、active user-symbol shape のいずれにもなり得る。 | `qualified_symbol`, `field_access`, `term_postfix`, `namespace_path` | `pure-spec` | A.2, A.3, A.13, A.15 |
| Term expressions | `PO-TERM-A02` | ambiguous | `set x = a . b;` | `ambiguous-preserve-surface`: active `.` user-symbol tokenization は、selector または namespace dot handling と区別できるままにする。 | `operator_expression`, `functor_application`, `term_postfix` | `pure-spec` | A.2, A.13 |
| Term expressions | `PO-TERM-R01` | recovery-required | `set x = F(a, b;` | `recover`: 閉じていない argument list に対する `missing_delimiter`。 | `inline_functor_application`, `term_list` | `mizar-parser` | A.13, A.15 |
| Formulas | `PO-FORM-P01` | positive | `theorem T: x in X & y = z & y <> w;` | `accept` | `theorem_item`, `formula`, `builtin_predicate_application`, `and_formula` | `mizar-test` | A.9, A.14, A.16 |
| Formulas | `PO-FORM-P02` | positive | `theorem T: x is divisible(2);` | `accept` | `theorem_item`, `is_assertion`, `attribute_test_chain`, `attribute_ref`, `argument_list` | `mizar-test` | A.3, A.14, A.16 |
| Formulas | `PO-FORM-N01` | negative | `theorem T: P iff Q iff R;` | `reject`: parenthesis なしの `iff` chain は non-associative である。 | `iff_formula` | `mizar-parser` | A.14 |
| Formulas | `PO-FORM-N02` | negative | `theorem T: x is non ;` | `reject`: `non` は `attribute_ref` または `type_expression` なしに `is_assertion_body` を終えられない。 | `is_assertion`, `is_assertion_body`, `attribute_test_chain`, `type_expression` | `mizar-parser` | A.3, A.14 |
| Formulas | `PO-FORM-A01` | ambiguous | `theorem T: x is T;` | `ambiguous-preserve-surface`: `is_assertion` は resolution が type assertion か attribute assertion かを決めるまで generic のままにする。 | `is_assertion`, `is_assertion_body`, `attribute_test_chain`, `type_expression` | `mizar-test` | A.3, A.14 |
| Formulas | `PO-FORM-A02` | ambiguous | `theorem T: x is non empty;` | `ambiguous-preserve-surface`: attribute-test body は resolution まで generic `is_assertion` の下に残す。 | `is_assertion`, `attribute_test_chain`, `attribute_ref` | `mizar-test` | A.3, A.14 |
| Formulas | `PO-FORM-A03` | ambiguous | `theorem T: x is Algebra.non EmptySet;` | `ambiguous-preserve-surface`: qualified attribute / type spelling は parse-time classification を強制しない。 | `is_assertion`, `qualified_symbol`, `type_expression`, `attribute_test_chain` | `pure-spec` | A.3, A.14 |
| Formulas | `PO-FORM-R01` | recovery-required | `theorem T: for x be set holds ;` | `recover`: `holds` 後の formula 欠落に対する `incomplete_construct`。 | `universal_formula`, `formula` | `mizar-parser` | A.14, A.16 |
| Statements/proofs | `PO-STMT-P01` | positive | `theorem T: thesis proof assume A: thesis; thus thesis; end;` | `accept` | `theorem_item`, `proof`, `reasoning`, `assumption`, `conclusion` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-P02` | positive | `theorem T: thesis proof reconsider x as set by A; thus thesis; end;` | `accept` | `proof`, `type_changing_statement`, `type_change_list`, `simple_justification` | `mizar-test` | A.4, A.15, A.16 |
| Statements/proofs | `PO-STMT-P03` | positive | `theorem T: thesis proof thus thesis by A.{B, C}, A.*; end;` | `accept` | `conclusion`, `references`, `grouped_reference`, `bulk_reference` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-P04` | positive | `theorem T: thesis proof A: x = y by B; then thus thesis; end;` | `accept` | `proposition`, `compact_statement`, `statement`, `conclusion` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-P05` | positive | `theorem T: thesis proof then A: x = y by B .= z by C; end;` | `accept` | `statement`, `iterative_equality`, `simple_justification` | `mizar-test` | A.15, A.16 |
| Statements/proofs | `PO-STMT-N01` | negative | `theorem T: thesis proof then let x be set; end;` | `reject`: `then` は `linkable_statement` にのみ適用され、`generalization` には適用されない。 | `statement`, `linkable_statement`, `generalization` | `mizar-parser` | A.15 |
| Statements/proofs | `PO-STMT-A01` | ambiguous | `theorem T: thesis proof x = y by A; end;` | `ambiguous-preserve-surface`: compact statement と zero-continuation iterative equality は、semantic fact なしに parser design が決める。 | `compact_statement`, `iterative_equality`, `simple_justification` | `pure-spec` | A.15 |
| Statements/proofs | `PO-STMT-R01` | recovery-required | `theorem T: thesis proof assume A: thesis;` | `recover`: 終端していない `proof` に対する `missing_end`。 | `proof`, `reasoning` | `mizar-parser` | A.15, A.16 |
| Annotations | `PO-ANN-P01` | positive | `@proof_hint(max_axioms: 10, solver: vampire) theorem T: thesis;` | `accept` | `annotated_declaration`, `annotation`, `proof_hint_annotation` | `mizar-test` | A.12, A.21 |
| Annotations | `PO-ANN-P02` | positive | `definition @custom(flag) theorem T: thesis; end;` | `accept` | `definition_block`, `definition_content`, `annotation`, `theorem_item` | `mizar-test` | A.12, A.16, A.21 |
| Annotations | `PO-ANN-P03` | positive | `registration @custom(flag) cluster C: non empty set; existence by A; end;` | `accept` | `registration_block`, `registration_content`, `annotation`, `registration_item` | `mizar-test` | A.17, A.21 |
| Annotations | `PO-ANN-P04` | positive | `theorem T: thesis proof @custom(flag) thus thesis; end;` | `accept` | `proof`, `annotated_statement`, `annotation`, `conclusion` | `mizar-test` | A.15, A.16, A.21 |
| Annotations | `PO-ANN-P05` | positive | `definition algorithm f() do @custom(flag) return; end; end;` | `accept` | `algorithm_def`, `annotated_algo_statement`, `annotation`, `return_stmt` | `mizar-test` | A.20, A.21 |
| Annotations | `PO-ANN-P06` | positive | `claim C do @custom(flag) theorem T: thesis; end;` | `accept` | `claim_block`, `annotated_theorem_item`, `annotation`, `theorem_item` | `mizar-test` | A.20, A.21 |
| Annotations | `PO-ANN-N01` | negative | `@latex(123) theorem T: thesis;` | `reject`: 固定 `@latex` annotation には string literal argument が必要である。 | `annotated_declaration`, `annotation`, `latex_annotation` | `mizar-parser` | A.12, A.21 |
| Annotations | `PO-ANN-A01` | ambiguous | `@custom(flag, 3) theorem T: thesis;` | `ambiguous-preserve-surface`: generic annotation name と context 上の argument 妥当性は semantic または registry check である。 | `annotated_declaration`, `annotation`, `statement_annotation`, `generic_annotation_name`, `annotation_args` | `mizar-test` | A.12, A.21 |
| Annotations | `PO-ANN-R01` | recovery-required | `@proof_hint(max_axioms: ) theorem T: thesis;` | `recover`: option value 欠落に対する `malformed_annotation`。 | `proof_hint_annotation`, `proof_hint_option` | `mizar-parser` | A.21 |
| Registrations | `PO-REG-P01` | positive | `registration let x be set; cluster C: non empty set; existence by A; end;` | `accept` | `registration_block`, `registration_content`, `existential_registration` | `mizar-test` | A.12, A.17 |
| Registrations | `PO-REG-N01` | negative | `registration cluster C: -> non empty for set; coherence by A; end;` | `reject`: conditional registration では `->` の前に非空の antecedent adjectives が必要である。 | `conditional_registration`, `antecedent_adjectives`, `adjective_list` | `mizar-parser` | A.17 |
| Registrations | `PO-REG-A01` | ambiguous | `registration cluster C: n-dimensional V -> non empty for T; coherence by A; end;` | `ambiguous-preserve-surface`: `param_prefix` split と active attribute spelling の境界は parser-assisted だが、semantic classification は後段に残る。 | `adjective`, `param_prefix`, `attribute_name` | `pure-spec` | A.2, A.17 |
| Registrations | `PO-REG-R01` | recovery-required | `registration cluster C: non empty set; existence by A;` | `recover`: 終端していない `registration_block` に対する `missing_end`。 | `registration_block`, `registration_item` | `mizar-parser` | A.17 |
| Templates | `PO-TPL-P01` | positive | `definition let T be type; theorem Id: thesis; end;` | `accept` | `template_definition`, `template_parameter_decl`, `let_type`, `template_item` | `mizar-test` | A.18 |
| Templates | `PO-TPL-P02` | positive | `definition let P be pred(set); theorem Id: thesis; end;` | `accept` | `template_definition`, `definition_parameter_decl`, `pred_param`, `type_list` | `mizar-test` | A.18 |
| Templates | `PO-TPL-P03` | positive | `definition let F be func(set) -> set; theorem Id: thesis; end;` | `accept` | `template_definition`, `definition_parameter_decl`, `func_param`, `type_list`, `type_expression` | `mizar-test` | A.18 |
| Templates | `PO-TPL-N01` | negative | `definition let F be func(set); theorem T: thesis; end;` | `reject`: `func_param` には `-> type_expression` が必要である。 | `func_param`, `type_list` | `mizar-parser` | A.18 |
| Templates | `PO-TPL-A01` | ambiguous | `theorem T: thesis proof thus thesis by Scheme[T, x qua R], A; end;` | `ambiguous-preserve-surface`: scheme application は現在 `simple_justification` と `reference [ template_args ]` に重なる。`param_name`、`type_arg_list`、`type_arg`、`qua_arg` は、helper を削除または再利用するまで fixture-planning input として明示的に残す。 | `scheme_app`, `param_name`, `simple_justification`, `reference`, `template_args`, `template_arg`, `qua_arg` | `pure-spec` | A.3, A.15, A.18 |
| Templates | `PO-TPL-R01` | recovery-required | `theorem T: thesis by P[set, x qua R;` | `recover`: 閉じていない `template_args` に対する `missing_delimiter`。 | `reference`, `template_args`, `template_arg`, `qua_arg` | `mizar-parser` | A.3, A.15, A.18 |
| Algorithms | `PO-ALG-P01` | positive | `definition algorithm f(x) -> set requires thesis do var y := x; return y; end; end;` | `accept` | `definition_block`, `algorithm_def`, `algorithm_body`, `var_decl`, `return_stmt` | `mizar-test` | A.12, A.20 |
| Algorithms | `PO-ALG-N01` | negative | `definition algorithm f() do claim C do theorem T: thesis; end; end; end;` | `reject`: `claim_block` は top-level declaration content であり、`algo_statement` ではない。 | `algorithm_def`, `algo_statement`, `claim_block` | `mizar-parser` | A.12, A.20 |
| Algorithms | `PO-ALG-A01` | ambiguous | `definition algorithm f() do x.y := z; snapshot S; end; end;` | `ambiguous-preserve-surface`: dotted `lvalue` は selector と namespace の役割が解決されるまで syntactic のままにする。 | `assignment`, `lvalue`, `snapshot_stmt` | `pure-spec` | A.2, A.20 |
| Algorithms | `PO-ALG-R01` | recovery-required | `definition algorithm f() do if thesis do return; end;` | `recover`: nested control block 後の外側 `algorithm_body` または `definition_block` に対する `missing_end`。 | `algorithm_def`, `if_stmt`, `algorithm_body` | `mizar-parser` | A.20 |
| Algorithms | `PO-ALG-R02` | recovery-required | `definition algorithm f() do claim C do theorem T: thesis; end; return; end; end;` | `recover`: `algo_statement_list` 内で top-level `claim_block` に遭遇したことに対する `misplaced_item`。後続 algorithm statement の前で同期する。 | `algorithm_def`, `algo_statement_list`, `algo_statement`, `claim_block` | `mizar-parser` | A.12, A.20 |

## 横断的な fixture 要件

- built-in predicate coverage は、`in`、`=`、`<>` の受理される application と、
  それらの built-in を definition symbol として使う `pred` / `redefine pred` の
  拒否例を含める。
- annotation attachment coverage は、module level、`definition_block` 内、
  `registration_block` 内、proof 内、algorithm body 内、`claim_block` の theorem
  list 内に現れなければならない。上の matrix 行は代表 owner を選ぶだけであり、
  残りの attachment site は、それを囲む production が着地する parser task で
  追加する。
- dot-chain coverage は、module path、qualified symbol、qualified reference、
  grouped reference、bulk reference、selector access/update、algorithm lvalue、
  active `.` user-symbol case を含める。parse-only expectation は syntax-only に
  とどめる。
- term boundary coverage は、`the type_expression`、parenthesized / unparenthesized
  `qua`、chained `qua`、`with (...)` update boundary を含める。
- statement coverage は、statement AST snapshot を導入する前に、
  `x = y by A;`、`x = y by A .= z by B;`、label、`then` variant を含める。
  `PO-STMT-A01`、`PO-STMT-P04`、`PO-STMT-P05` が行レベルの anchor である。
- recovery coverage は、missing `end`、missing delimiter、malformed annotation、
  misplaced `claim`、misplaced `import` を含める。Misplaced form には strict
  `reject` 行があってもよいが、parser が後続 declaration または statement へ
  継続することを期待する場合は recovery 行を必須とする。

## Fixture 有効化計画

Task 8 は全行を一度に有効化しない。初期 seed は、高リスクで前提が少ない case を
優先する。

1. Module prelude ordering と missing `end` recovery:
   `PO-MOD-N01`, `PO-MOD-R01`, `PO-MOD-R02`。
2. Built-in predicate の受理／拒否と generic `is_assertion`:
   `PO-FORM-P01`, `PO-FORM-A01`, `PO-DECL-N01`。
3. Dot と term/formula boundary:
   `PO-TERM-A01`, `PO-TERM-P01`, `PO-TYPE-A01`。
4. Compact statement と iterative equality:
   `PO-STMT-A01`, `PO-STMT-P05`。
5. Malformed annotation と misplaced `claim`:
   `PO-ANN-R01`, `PO-ALG-N01`, `PO-ALG-R02`。

Executable な `mizar-test` corpus case では、expectation sidecar に次を使う。

```toml
stage = "parse_only"
expected_phase = "parse"
```

受理 case は `expected_outcome = "pass"` と `diagnostic_codes = []` を使う。
拒否および recovery case は `expected_outcome = "fail"`、
`failure_category = "syntax_error"`、parser が所有する安定した
`rejection_reason` / `stable_detail_key` を使う。AST snapshot design が着地するまで
snapshot profile は置かない。

## Task 8 seed status

Task 8 は初期 seed を [parse_only_fixture_seed.md](./parse_only_fixture_seed.md)
に記録する。この seed は Fixture 有効化計画の case ID と source shape を安定
させ、`qua`、`reconsider`、string-required annotation rejection boundary の補助行を
追加し、parse-only corpus runner と所有する grammar production が case を実行できる
まで checked-in manifest として保持する。AST snapshot expectation は導入しない。

## Appendix A Traceability Summary

| Appendix A section | Matrix areas |
|---|---|
| A.2 Lexical Structure | Module dot role、type parameter prefix、term dot role、algorithm lvalue |
| A.3 Type Expressions | Type expression、`is_assertion` 経由の formula、template `qua_arg` |
| A.4 Variables and Constants | Declaration と statement binding |
| A.9 Predicates | Declaration と formula、特に built-in predicate boundary |
| A.10 Functors and Operator Declarations | Function definition と term/operator boundary |
| A.12 Modules and Namespaces | Module structure、declaration、annotation、definition 内 algorithm |
| A.13 Term Expressions | Term boundary、`the`、`qua`、field access/update |
| A.14 Formulas | Formula precedence、non-associative `iff`、generic `is_assertion` |
| A.15 Statements, Proofs, and References | Proof statement、justification、reference、template argument |
| A.16 Theorems and Correctness Blocks | Top-level theorem と proof attachment |
| A.17 Clusters and Registrations | Registration syntax と adjective boundary |
| A.18 Templates | Template parameter、template item、scheme/helper follow-up |
| A.20 Algorithms and Computation | Algorithm definition、control statement、claim placement |
| A.21 Annotations | Attaching annotation と standalone diagnostic annotation |
