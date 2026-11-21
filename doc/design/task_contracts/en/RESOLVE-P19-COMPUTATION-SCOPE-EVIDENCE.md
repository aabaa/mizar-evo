# Task RESOLVE-P19-COMPUTATION-SCOPE-EVIDENCE: unavailable quantified declarations
Canonical language: English; [Japanese pointer](../ja/RESOLVE-P19-COMPUTATION-SCOPE-EVIDENCE.md).
Status: bounded implementation. Tier: full.
Owner: [Resolver plan](../../mizar-resolve/en/00.crate_plan.md); coordination/consumer: [Checker plan](../../mizar-checker/en/00.crate_plan.md), AST-TYPE and later AST-DIAGNOSTICS/[MBP-08](AST-MULTI-BINDER-PLAN.md).
Purpose: produce authenticated reject-only source evidence for a quantified declaration unavailable in its one-line statement's actual computation argument.
Dependencies: existing parser originals, declaration symbols and lexical scope infrastructure; no checked-reflection prerequisite.
Authority: [EN16 §16.5.1](../../../spec/en/16.theorems_and_proofs.md#1651-simple-citation), [EN22 §22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics), [EN04 §4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing), [EN20 formals](../../../spec/en/20.algorithm_and_verification.md#2011-algorithm-definition-block).
Gap: source_drift/test_gap; justification descendants are excluded from variable reference intake and call inventories do not select algorithm/formal targets.
Full tier fails predecessor [P19 source correspondence](RESOLVE-P19-BINDER-SOURCE-CORRESPONDENCE.md) pattern criteria: new call/formal intake and public unavailable-declaration evidence boundary.

## Frozen boundary
Implement the missing bounded intake inside Resolver. Admit one unambiguous local algorithm with one explicitly declared `expr of boolean` formal and an actual formula argument in a one-line statement's `by computation` call. Match the actual call name to its declaration symbol and its formal to the physical declaration/type; spelling alone is insufficient.
Retain original source identities and anchors for the statement, computation insertion, call/callee, argument, formal/type, offending use and unavailable quantified declaration. Evidence is source-local metadata, never a stored free UID, checked AST or acceptance result.
At each supported argument use, check the lexical winner at the insertion point first. A visible same-spelled outer binding wins; it must never be replaced by the statement's quantified binding. A name absent from both visible scope and that statement's declarations remains unknown, distinct from an unavailable declaration.
Require original-AST/resolved-arena correspondence and complete symbol-collection replay before producing evidence. Records have no caller-supplied provenance; reauthentication against another original must reject stale evidence, including coherently changed source projections. Recovered/mismatched inputs and unsupported call, formal, formula or scope shapes refuse without partial evidence or spelling fallback.
Keep existing collectors and consumers unchanged. Emit no E0206 or other new diagnostic. Exclude checked reflection, capture/C, reconstruction/R2, certification/formation, execution, proof/acceptance and MBP-03–06 credit.

## Artifacts and ownership
Use existing Resolver source and test modules; inline the bounded producer rather than creating a parallel AST or generic call resolver. Checker owns later adoption, not a new typing route in this task.
Owner invariants: [lexical scopes](../../mizar-resolve/en/names.md#step-5c1-variable-scope-resolution), [original replay](../../mizar-resolve/en/symbols.md#original-source-proof-lexical-maps).
Update paired owner boundaries and MBP-02/08 mapping only where changed. Audit ownership, executable coverage and trace credit remain unchanged. Do not modify specifications, existing MIZ/expectations, protocols or AGENTS.md.

## Test matrix and exit
Use real parser/Frontend originals and collectors; [contextual-input fixture](../../../../tests/miz/pass/parser/pass_parser_computation_contextual_inputs_001.miz) is a parser-only shape precedent, not resolution/acceptance credit. Cap six test families:
1. Real bounded algorithm, explicit boolean expr formal and one-line quantified statement produce deterministic records with exact insertion/use/declaration and call/formal/type anchors.
2. Reserve/proof-local outer bindings with the same spelling win; different declarations and repeated uses retain their genuine source identities.
3. Unknown names stay distinct; no matching unavailable declaration yields no failure evidence, never an inferred scope failure.
4. Wrong/ambiguous/missing/qualified callee, mismatched arity, non-expr/non-boolean or missing formal/type refuse; no spelling-only association.
5. Wrong original/source/module/symbols, recovered trees and coherent source changes reject stale records; pristine repeated acquisition and replay pass.
6. Unsupported formula/scope/call layout refuses wholly; ordinary resolver/Checker behavior and higher-stage refusal remain unchanged.
Establish the common real call/formal/quantifier fixture and one passing setup before implementing the remaining cases. Exit: genuine source-authenticated scope failure evidence is available for later diagnostic adoption, with every control preserved.

## Reviews and verification
Independent specification/test-intent/trust review before freeze; sufficiency, precision/trust, removable-only volume, source/document consistency and final full-gate quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites, Resolver owner tests, affected call/scope public checks and a representative CLI check with CARGO_BUILD_JOBS=4. Run changed original-replay/identity obligations immediately; unchanged acceptance/policy and broad edit-versus-clean matrices defer to integration cadence.
Documentation freeze runs both lint_policy suites including links/fragments. No lane full-test trigger, integration or push; freeze and implementation count once at integration.
