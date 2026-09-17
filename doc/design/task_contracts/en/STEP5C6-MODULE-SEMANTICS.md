# Task STEP5C6-MODULE-SEMANTICS: module corpus activation

Canonical language: English; [Japanese pointer](../ja/STEP5C6-MODULE-SEMANTICS.md).
Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index),
[mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index), [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: partial, six outcomes complete; predicate antonym deferred; full tier.
- Dependencies 5A.1, 5A.5, and 5A.7 are complete; predicate antonym stays inactive.
- `mizar-resolve` owns import, local-name and bounded alias resolution; `mizar-frontend` retains
  syntax/lexical-summary ownership; `mizar-checker` checks synonym applications; `mizar-test` owns admission and fixture wiring.
- Classification: `source_drift` / executable `test_gap`; the syntax-only fixture
  provider is not a semantic module index. Missing fixture infrastructure is
  an implementation prerequisite, not a new language decision.
- Authority: [§11.1–11.5](../../../spec/en/11.symbol_management.md#111-synonyms-and-antonyms),
  [§12.3](../../../spec/en/12.modules_and_namespaces.md#123-import-statements),
  [§12.5](../../../spec/en/12.modules_and_namespaces.md#125-visibility-control-privatepublic),
  the mapped sources and expectations, [trace](../../../../tests/coverage/spec_trace.toml),
  and [activation map](../../../../tests/coverage/step5_activation_map.tsv).
## Activation and scope

The original four retain `declaration_symbol` / `resolve` and sole `active_declaration_symbol`.
The synonym negative retains `type_elaboration` / `resolve`; the positive uses `type_check`; both have sole `active_type_elaboration` and empty codes.

| Case | Result |
|---|---|
| `fail_declaration_symbol_import_duplicate_alias_001` | `modules.import.duplicate_alias` |
| `pass_declaration_symbol_branch_import_form_001` | pass |
| `fail_declaration_symbol_import_unknown_module_001` | `modules.import.unknown_module` |
| `pass_declaration_symbol_private_theorem_visibility_001` | pass |
| `fail_type_elaboration_synonym_loci_mismatch_001` | `notation.synonym.loci_mismatch` |
| `pass_type_elaboration_synonym_functor_001` | type_check / pass |

Admission authenticates exact id, workspace-relative source/sidecar paths, stage,
phase, outcome, category, domain, spec refs and sole tag; reserved identities never fall through other stages.
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

The [symbol owner](../../mizar-resolve/en/symbols.md#recovery-and-diagnostics) specifies source-aware collection and its internal synonym-loci diagnostic.
Authenticate actual ordered loci and the unique earlier functor; only a genuine locus bijection supplies the existing resolver synonym-target relation.
The [type checker](../../mizar-checker/en/type_checker.md#source-functor-synonym-typing) applies that source permutation to both real calls and checks types; no unfolding, proof/interface acceptance, new numeric code or deeper import matrix. Update Chapter 11 only.
Preserve existing `.miz`, outcomes/phases/keys, trace, activation map, archive,
soundness boundaries, the certificate rejection corpus, and Task 277B state.

## Exit

Review specification/docs, tests, implementation, removable volume/scope, and
source/doc consistency. Test exact activation plus invalid admission, missing
fixtures, import provenance, private visibility, actual alias targets, legal reordered loci, and diagnostic replay. Run focused
tests and corpus commands, then workspace fmt, warnings-denied Clippy, and tests.
Exit with six mapped activations, unchanged antonym deferral and prior outcomes, within the same cumulative task budgets.
