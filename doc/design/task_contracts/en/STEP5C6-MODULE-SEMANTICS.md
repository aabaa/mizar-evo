# Task STEP5C6-MODULE-SEMANTICS: module corpus activation

Canonical language: English; [Japanese pointer](../ja/STEP5C6-MODULE-SEMANTICS.md).
Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index),
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: complete; full tier. Dependencies 5A.1, 5A.5, and 5A.7 are complete.
- Purpose: connect four non-gap Step 5C.6 corpus pairs to existing resolver owners.
- `mizar-resolve` owns import and local-name resolution; `mizar-frontend` retains
  syntax/lexical-summary ownership; `mizar-test` owns admission and fixture wiring.
- Classification: `source_drift` / executable `test_gap`; the syntax-only fixture
  provider is not a semantic module index. Missing fixture infrastructure is
  an implementation prerequisite, not a new language decision.
- Authority: [§11.3–11.4](../../../spec/en/11.symbol_management.md#113-import-behavior-and-conflict-resolution),
  [§12.3](../../../spec/en/12.modules_and_namespaces.md#123-import-statements),
  [§12.5](../../../spec/en/12.modules_and_namespaces.md#125-visibility-control-privatepublic),
  the mapped sources and expectations, [trace](../../../../tests/coverage/spec_trace.toml),
  and [activation map](../../../../tests/coverage/step5_activation_map.tsv).

## Activation and scope

All four retain `declaration_symbol`, phase `resolve`, and empty public codes;
only their sole `active_declaration_symbol` tag is added.

| Case | Result |
|---|---|
| `fail_declaration_symbol_import_duplicate_alias_001` | `modules.import.duplicate_alias` |
| `pass_declaration_symbol_branch_import_form_001` | pass |
| `fail_declaration_symbol_import_unknown_module_001` | `modules.import.unknown_module` |
| `pass_declaration_symbol_private_theorem_visibility_001` | pass |

Admission authenticates exact id, workspace-relative source/sidecar paths, stage,
phase, outcome, key, and sole tag; malformed admission must not fall back.
Reuse `SurfaceAst`, resolver import candidates, module index, symbol collection,
and name/label resolution. Add no public type or parallel term/formula/statement IR.
The [harness](../../mizar-test/en/harness.md) owns a bounded fixture index grounded in
readable files under `crates/mizar-test/tests/testdata/parser/`: add `type_fixtures.miz`
with [§9.1](../../../spec/en/09.predicates.md#91-overview-and-syntax) public `divides` notation; retain `nested_capture_fixtures.miz` unchanged.
Keep lexical summaries syntax-only; module existence must come from that file index.
Extract unrecovered import paths, aliases and branch ranges from AST, then invoke
the [import owner](../../mizar-resolve/en/imports.md). Resolve the actual private
theorem citation to the earlier same-module symbol and require private/local-only
visibility via the [symbol owner](../../mizar-resolve/en/symbols.md).

No proof checking, imported proof/interface elaboration, general workspace build,
new diagnostics, lower-stage behavior, or deeper import matrix is introduced.
The three G4/G5/G9 pairs stay inactive; Chapter 12 audit coverage changes only.
Preserve existing `.miz`, outcomes/phases/keys, trace, activation map, archive,
soundness boundaries, the certificate rejection corpus, and Task 277B state.

## Exit

Review specification/docs, tests, implementation, removable volume/scope, and
source/doc consistency. Test exact activation plus invalid admission, missing
fixtures, import provenance, and private citation/visibility failures. Run focused
tests and corpus commands, then workspace fmt, warnings-denied Clippy, and tests.
Exit with four activations and one local task commit within the approved budgets.
