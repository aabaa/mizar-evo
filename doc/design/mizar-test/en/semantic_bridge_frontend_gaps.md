# Audit: Semantic-Bridge Frontend Gap Inventory (September 2026, Audit 1)

> Canonical language: English. Japanese companion:
> [../ja/semantic_bridge_frontend_gaps.md](../ja/semantic_bridge_frontend_gaps.md)
> (pointer only, per the September 2026 status-document language policy).

## Purpose

While validating the audit-1 oracle corpus
([semantic_bridge_corpus_map.md](./semantic_bridge_corpus_map.md)), every
new source was parsed through the real frontend
(`Frontend` + `MizarParserSeam`, disk loader, empty-import provider plus the
parse-only fixture symbols). 91/120 sources parse cleanly; 29 spec-correct
sources are blocked by the frontend gaps below. Each gap is a candidate
owner task for the Step-5 re-decomposition (audit 2). None of these gaps may
be closed by weakening the corpus expectations.

## Gap Inventory

| Id | Severity | Description |
|---|---|---|
| G1 | closed by 5A.2 (20 ledger rows) | Same-module local notation activation is implemented; see the [task contract](../../task_contracts/en/STEP5A2-G1-LOCAL-NOTATION.md). Mixed-row G2/G6 diagnostics and all semantic activation remain with their named owners. |
| G2 | closed by 5A.3 (3 ledger rows) | Symbolic functor/predicate declaration sites now tokenize per A.2; see the [task contract](../../task_contracts/en/STEP5A3-G2-SYMBOLIC-USER-SYMBOLS.md). Semantic activation remains with the named 5C owners. |
| G3 | closed by 5A.4 (1 ledger row) | Omitted-justification compact statements now parse beneath `[then] linkable_statement`; see the [task contract](../../task_contracts/en/STEP5A4-G3-THEN-LINKING.md). Semantic activation remains with 5C.9. |
| G4 | closed by 5A.5 (3 ledger rows) | Local `synonym`/`antonym` spellings activate after their declaring item; see the [task contract](../../task_contracts/en/STEP5A5-G4-NOTATION-ALIASES.md). Semantic activation remains with 5C.6. |
| G5 | closed by 5A.1 | Root-reachable AST validation removes the `term qua <structure type>` SurfaceAstBuilder panic; see the [task contract](../../task_contracts/en/STEP5A1-G5-QUA-STRUCTURE.md). Semantic activation remains owned by 5C. |
| G6 | closed by 5A.6 (2 ledger rows) | Argument-bearing local dependent-mode use now preserves enclosing type/formula tokens; see the [task contract](../../task_contracts/en/STEP5A6-G6-DEPENDENT-MODE-USE.md). Task 68's same-module reserve extraction and imported Tasks 79/82 remain unchanged. |
| G7 | decision frozen; implementation pending | Empty justification remains legal and proof obligations remain unchanged; the bounded parser correction is owned by the [5A.8 contract](../../task_contracts/en/STEP5A8-G7-EMPTY-JUSTIFICATIONS.md). |
| G9 | closed by 5A.7 (3 ledger rows) | Dependencies 5A.2/5A.5 make already-active spellings parse in predicate redefinitions, second definitions, and synonym original patterns; dedicated evidence is recorded in the [5A.7 contract](../../task_contracts/en/STEP5A7-G9-ACTIVE-PATTERN-SPELLINGS.md). Semantic activation remains with 5C.5/5C.6. |

G8 (inline application use) is folded into G1.

## Minimal Reproducers

G5 crash (also `fail_type_elaboration_term_qua_invalid_narrowing_001`):

```mizar
definition
  struct PBox where
    field d -> set;
  end;
end;

theorem P14T: for X being set holds (X qua PBox) = X
proof
  let X be set;
  thus (X qua PBox) = X;
end;
```

G1 (declaration parses, use fails):

```mizar
definition
  let X be set;
  func P6Def: probesix X -> set equals X;
  coherence;
end;

theorem P6T: for X being set holds probesix X = X
proof
  let X be set;
  thus probesix X = X;
end;
```

G3:

```mizar
theorem QCT: for X being set st X = X holds X = X
proof
  let X be set;
  assume X = X;
  then A2: X = X;
  hence X = X by A2;
end;
```

## Blocked-Source Ledger

Machine-readable copy:
[`tests/coverage/audit1_frontend_gaps.tsv`](../../../../tests/coverage/audit1_frontend_gaps.tsv).
These sources stay committed as inactive oracle seeds. The ledger records the
audit-time blocking gaps; closure evidence belongs to the linked Step 5A task
contracts. Corpus-wide tooling must not assume every committed `.miz` parses
until the remaining frontend gaps close.

| Source | Gaps |
|---|---|
| `tests/miz/fail/clusters/fail_proof_verification_functorial_false_coherence_001.miz` | G1 |
| `tests/miz/fail/clusters/fail_proof_verification_reduce_false_reducibility_001.miz` | G1 |
| `tests/miz/fail/overload/fail_advanced_semantics_overload_ambiguous_candidates_001.miz` | G1 |
| `tests/miz/fail/predicates/fail_type_elaboration_pred_argument_type_mismatch_001.miz` | G1 |
| `tests/miz/fail/predicates/fail_type_elaboration_pred_duplicate_same_signature_001.miz` | G9 |
| `tests/miz/fail/resolve/fail_type_elaboration_synonym_loci_mismatch_001.miz` | G4,G9 |
| `tests/miz/fail/templates/fail_type_elaboration_template_arity_mismatch_001.miz` | G1 |
| `tests/miz/fail/templates/fail_type_elaboration_template_bound_violation_001.miz` | G1 |
| `tests/miz/fail/terms/fail_type_elaboration_term_qua_invalid_narrowing_001.miz` | G5 |
| `tests/miz/fail/types/fail_type_elaboration_argument_type_mismatch_functor_001.miz` | G1 |
| `tests/miz/pass/clusters/pass_advanced_semantics_functorial_registration_001.miz` | G1 |
| `tests/miz/pass/clusters/pass_advanced_semantics_reduce_registration_001.miz` | G1 |
| `tests/miz/pass/functors/pass_proof_verification_func_equals_infix_operator_001.miz` | G1,G2 |
| `tests/miz/pass/functors/pass_proof_verification_func_means_prefix_001.miz` | G1 |
| `tests/miz/pass/functors/pass_type_elaboration_func_commutativity_property_001.miz` | G1,G2 |
| `tests/miz/pass/functors/pass_type_elaboration_func_dependent_return_type_001.miz` | G1,G6 |
| `tests/miz/pass/modes/pass_type_elaboration_mode_dependent_of_params_001.miz` | G6 |
| `tests/miz/pass/overload/pass_advanced_semantics_overload_distinct_loci_001.miz` | G1 |
| `tests/miz/pass/predicates/pass_formula_statement_pred_negated_application_001.miz` | G1 |
| `tests/miz/pass/predicates/pass_proof_verification_pred_phrase_identifier_001.miz` | G1 |
| `tests/miz/pass/predicates/pass_proof_verification_pred_symbolic_infix_001.miz` | G2 |
| `tests/miz/pass/predicates/pass_type_elaboration_pred_redefine_narrower_loci_001.miz` | G9 |
| `tests/miz/pass/resolve/pass_type_elaboration_antonym_predicate_001.miz` | G4 |
| `tests/miz/pass/resolve/pass_type_elaboration_synonym_functor_001.miz` | G4,G5 |
| `tests/miz/pass/templates/pass_type_elaboration_template_pred_param_001.miz` | G1 |
| `tests/miz/pass/templates/pass_type_elaboration_template_type_param_functor_001.miz` | G1 |
| `tests/miz/pass/theorems/pass_formula_statement_then_hence_linking_001.miz` | G3 |
| `tests/miz/pass/types/pass_type_elaboration_argument_attribute_widening_001.miz` | G1 |
| `tests/miz/pass/variables/pass_formula_statement_deffunc_defpred_local_001.miz` | G1 |

## Verification Method

Results were produced by a throwaway integration harness (not committed)
that ran each source through
`Frontend::new(FrontendSourceLoader::new(DiskSourceLoader::new(root)), <import provider>, MizarParserSeam)`
in an isolated temp package, catching panics per file, classifying
`DiagnosticCode::Syntax` versus other diagnostics, and asserting: pass and
semantic-fail sidecars parse cleanly; `parse_only` fail sidecars produce
syntax diagnostics. Two probe rounds (18 + 12 minimal sources) isolated the
gap boundaries quoted above; the reproducers in this document are copies of
those probes.
