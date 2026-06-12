# mizar-syntax Task 8: initial parse-only fixture seed

> 正本は英語です。英語版:
> [../en/parse_only_fixture_seed.md](../en/parse_only_fixture_seed.md)。

状態: 2026-06-13 時点で checked-in fixture manifest として完了。task 3 の
parse-only runner active gate follow-up も 2026-06-13 に完了。

## 目的

このノートは、[parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md)
から選んだ初期 parse-only grammar fixture seed を固定する。現在の parser が
文法 surface 全体をまだ実行できない状態でも、選定済み case は安定したままにする。

現在の `mizar-parser` は、token stream の保持、最小限の明示 fixity expression
surface の parse、および以前の基盤 task で必要だった block-end / string recovery
だけを実装している。Task 7 matrix が要求する module、declaration、term、
formula、statement、annotation、template、algorithm production はまだ実装して
いない。そのため Task 8 では、active runner execution とは分けて、この
checked-in manifest を着地させた。

parse-only corpus runner は現在 `mizar-test` に存在するが、`active_parse_only`
tag を持つ sidecar だけを実行する。下の各行は、所有する grammar production が
利用可能になるまで inactive のままにする。その時点で、case ID、source shape、
parse-only expectation、Appendix A traceability を変更せずに有効化できる。

## 有効化ルール

- これらの seed case には AST snapshot を追加しない。対応する node vocabulary
  増分が node kind、child role、range rule、recovery rendering を定義するまで、
  snapshot profile は置かない。
- 有効化とは、source を `.miz` file として `tests/miz/{pass,fail}/parser/` に置き、
  隣接する `.expect.toml` を追加し、その expectation path を
  `tests/coverage/spec_trace.toml` から link することを指す。現在の frontend
  parser seam が expectation を満たせる場合に限り、`tags = ["active_parse_only"]`
  を追加する。
- `accept` と `ambiguous-preserve-surface` の行は、
  `expected_outcome = "pass"`、`stage = "parse_only"`、
  `expected_phase = "parse"` として有効化する。
- `reject` と `recover` の行は、`expected_outcome = "fail"`、
  `stage = "parse_only"`、`expected_phase = "parse"`、
  `failure_category = "syntax_error"`、および parser が所有する
  `rejection_reason` / `stable_detail_key` として有効化する。
- pure-spec 行は、所有する parser task が周辺 construct を明示的に実行可能にする
  まで default discovery には入れない。

## 選定 case manifest

主 seed は Task 7 の「Fixture 有効化計画」である。Task 8 の boundary list にある
`qua`、`reconsider`、string-required annotation rejection boundary を満たすため、
3 つの補助行も含める。これは activation-plan 行の優先順位を変更しない。

| Case | Seed role | Matrix result | Owner | Appendix A trace | Activation target |
|---|---|---|---|---|---|
| `PO-MOD-N01` | primary | `reject` | `mizar-parser` | A.12 | `tests/miz/fail/parser/fail_parser_task8_po_mod_n01_late_import_001.*` |
| `PO-MOD-R01` | primary | `recover` | `mizar-parser` | A.12 | `tests/miz/fail/parser/fail_parser_task8_po_mod_r01_missing_definition_end_001.*` |
| `PO-MOD-R02` | primary | `recover` | `mizar-parser` | A.12 | `tests/miz/fail/parser/fail_parser_task8_po_mod_r02_misplaced_import_001.*` |
| `PO-FORM-P01` | primary | `accept` | `mizar-test` | A.9, A.14, A.16 | `tests/miz/pass/parser/pass_parser_task8_po_form_p01_builtin_predicates_001.*` |
| `PO-FORM-A01` | primary | `ambiguous-preserve-surface` | `mizar-test` | A.3, A.14 | `tests/miz/pass/parser/pass_parser_task8_po_form_a01_generic_is_001.*` |
| `PO-DECL-N01` | primary | `reject` | `mizar-parser` | A.2, A.9 | `tests/miz/fail/parser/fail_parser_task8_po_decl_n01_builtin_pred_symbol_001.*` |
| `PO-TERM-A01` | primary | `ambiguous-preserve-surface` | `pure-spec` | A.2, A.3, A.13, A.15 | `tests/miz/pass/parser/pass_parser_task8_po_term_a01_dot_chain_001.*` |
| `PO-TERM-P01` | primary | `accept` | `mizar-test` | A.13, A.15, A.16 | `tests/miz/pass/parser/pass_parser_task8_po_term_p01_choice_update_001.*` |
| `PO-TYPE-A01` | primary | `ambiguous-preserve-surface` | `pure-spec` | A.3 | `tests/miz/pass/parser/pass_parser_task8_po_type_a01_type_head_001.*` |
| `PO-STMT-A01` | primary | `ambiguous-preserve-surface` | `pure-spec` | A.15 | `tests/miz/pass/parser/pass_parser_task8_po_stmt_a01_compact_statement_001.*` |
| `PO-STMT-P05` | primary | `accept` | `mizar-test` | A.15, A.16 | `tests/miz/pass/parser/pass_parser_task8_po_stmt_p05_iterative_equality_001.*` |
| `PO-ANN-R01` | primary | `recover` | `mizar-parser` | A.21 | `tests/miz/fail/parser/fail_parser_task8_po_ann_r01_missing_option_value_001.*` |
| `PO-ALG-N01` | primary | `reject` | `mizar-parser` | A.12, A.20 | `tests/miz/fail/parser/fail_parser_task8_po_alg_n01_claim_in_algorithm_001.*` |
| `PO-ALG-R02` | primary | `recover` | `mizar-parser` | A.12, A.20 | `tests/miz/fail/parser/fail_parser_task8_po_alg_r02_misplaced_claim_recovery_001.*` |
| `PO-TERM-P02` | supplemental | `accept` | `mizar-test` | A.13, A.15, A.16 | `tests/miz/pass/parser/pass_parser_task8_po_term_p02_qua_chain_001.*` |
| `PO-STMT-P02` | supplemental | `accept` | `mizar-test` | A.4, A.15, A.16 | `tests/miz/pass/parser/pass_parser_task8_po_stmt_p02_reconsider_001.*` |
| `PO-ANN-N01` | supplemental | `reject` | `mizar-parser` | A.12, A.21 | `tests/miz/fail/parser/fail_parser_task8_po_ann_n01_latex_requires_string_001.*` |

## Source shapes

```mizar
-- PO-MOD-N01
theorem T: thesis; import Late.Module;

-- PO-MOD-R01
definition theorem T: thesis;

-- PO-MOD-R02
theorem T: thesis; import Late.Module; theorem U: thesis;

-- PO-FORM-P01
theorem T: x in X & y = z & y <> w;

-- PO-FORM-A01
theorem T: x is T;

-- PO-DECL-N01
definition pred P: x in y means thesis; end;

-- PO-TERM-A01
set x = A.B.c;

-- PO-TERM-P01
theorem T: thesis proof set x = the non empty T with (f := y); thus thesis; end;

-- PO-TYPE-A01
reserve x for Foo[T];

-- PO-STMT-A01
theorem T: thesis proof x = y by A; end;

-- PO-STMT-P05
theorem T: thesis proof then A: x = y by B .= z by C; end;

-- PO-ANN-R01
@proof_hint(max_axioms: ) theorem T: thesis;

-- PO-ALG-N01
definition algorithm f() do claim C do theorem T: thesis; end; end; end;

-- PO-ALG-R02
definition algorithm f() do claim C do theorem T: thesis; end; return; end; end;

-- PO-TERM-P02
theorem T: thesis proof set x = (a qua R) qua S; thus thesis; end;

-- PO-STMT-P02
theorem T: thesis proof reconsider x as set by A; thus thesis; end;

-- PO-ANN-N01
@latex(123) theorem T: thesis;
```

## Deferred activation owner

| Boundary | Seed cases | Activation owner |
|---|---|---|
| Import prelude ordering と misplaced import | `PO-MOD-N01`, `PO-MOD-R02` | `mizar-parser` task 5/6 corpus runner path |
| Missing `end` recovery | `PO-MOD-R01` | `mizar-parser` task 2/3 runner-backed recovery check |
| Built-in predicate と generic `is` boundary | `PO-FORM-P01`, `PO-FORM-A01`, `PO-DECL-N01` | `mizar-parser` task 13、14、24 |
| Dot chain と term/type boundary | `PO-TERM-A01`, `PO-TERM-P01`, `PO-TERM-P02`, `PO-TYPE-A01` | `mizar-parser` task 8-11 |
| Statement reachability と equality dispatch | `PO-STMT-A01`, `PO-STMT-P02`, `PO-STMT-P05` | `mizar-parser` task 18、19 |
| Annotation と algorithm recovery | `PO-ANN-R01`, `PO-ANN-N01`, `PO-ALG-N01`, `PO-ALG-R02` | `mizar-parser` task 32、35 |
