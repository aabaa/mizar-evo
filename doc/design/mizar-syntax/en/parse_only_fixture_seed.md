# mizar-syntax Task 8: Initial Parse-Only Fixture Seed

> Canonical language: English. Japanese companion:
> [../ja/parse_only_fixture_seed.md](../ja/parse_only_fixture_seed.md).

Status: completed as a checked-in fixture manifest on 2026-06-13; task 3
parse-only runner active-gate follow-up completed on 2026-06-13.

## Purpose

This note fixes the initial parse-only grammar fixture seed chosen from
[parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md). The
selected cases are intentionally stable even though the current parser cannot
execute the full grammar surface yet.

`mizar-parser` currently preserves token streams, parses only the minimal
explicit-fixity expression surface, and performs block-end/string recovery
needed by earlier infrastructure tasks. It does not yet implement the module,
declaration, term, formula, statement, annotation, template, or algorithm
productions required by the Task 7 matrix. For that reason, Task 8 landed this
checked-in manifest separately from active runner execution.

The parse-only corpus runner now exists in `mizar-test`, but it executes only
sidecars tagged with `active_parse_only`. Each row below remains inactive until
the owning grammar production is available; at that point it can be activated
without changing the selected case ID, source shape, parse-only expectation, or
Appendix A traceability.

## Activation Rules

- Do not add AST snapshots for these seed cases. Snapshot profiles stay absent
  until the paired node-vocabulary increment defines node kinds, child roles,
  range rules, and recovery rendering.
- Activation means adding the source as a `.miz` file under
  `tests/miz/{pass,fail}/parser/`, adding the adjacent `.expect.toml`, and
  linking the expectation path from `tests/coverage/spec_trace.toml`. Add
  `tags = ["active_parse_only"]` only when the current frontend parser seam can
  satisfy the expectation.
- `accept` and `ambiguous-preserve-surface` rows activate as
  `expected_outcome = "pass"`, `stage = "parse_only"`, and
  `expected_phase = "parse"`.
- `reject` and `recover` rows activate as `expected_outcome = "fail"`,
  `stage = "parse_only"`, `expected_phase = "parse"`,
  `failure_category = "syntax_error"`, and parser-owned
  `rejection_reason` / `stable_detail_key` values.
- Pure-spec rows remain out of default discovery until their owning parser task
  explicitly makes the surrounding construct executable.

## Selected Case Manifest

The primary seed is the Task 7 "Fixture Activation Plan". Three supplemental
rows are included to cover the Task 8 boundary list entries for `qua`,
`reconsider`, and the string-required annotation rejection boundary without
changing the priority of the activation-plan rows.

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

## Source Shapes

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

## Deferred Activation Owners

| Boundary | Seed cases | Activation owner |
|---|---|---|
| Import prelude ordering and misplaced imports | `PO-MOD-N01`, `PO-MOD-R02` | `mizar-parser` task 5/6 corpus runner path |
| Missing `end` recovery | `PO-MOD-R01` | `mizar-parser` task 2/3 runner-backed recovery checks |
| Built-in predicate and generic `is` boundaries | `PO-FORM-P01`, `PO-FORM-A01`, `PO-DECL-N01` | `mizar-parser` tasks 13, 14, and 24 |
| Dot chains and term/type boundaries | `PO-TERM-A01`, `PO-TERM-P01`, `PO-TERM-P02`, `PO-TYPE-A01` | `mizar-parser` tasks 8-11 |
| Statement reachability and equality dispatch | `PO-STMT-A01`, `PO-STMT-P02`, `PO-STMT-P05` | `mizar-parser` tasks 18 and 19 |
| Annotation and algorithm recovery | `PO-ANN-R01`, `PO-ANN-N01`, `PO-ALG-N01`, `PO-ALG-R02` | `mizar-parser` tasks 32 and 35 |
