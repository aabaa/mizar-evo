# Task CHECKER273-REGISTRATION-INTAKE: registration intake producer
Canonical language: English; [Japanese pointer](../ja/CHECKER273-REGISTRATION-INTAKE.md).
Status: implemented within the bounded operand profile. Tier: full (new owner module and public producer seam).
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [runner plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: Tasks249–272; prepared MT10-AS consumer. Task274 remains blocked-reserved.
Authority: [producer row and shared contract](../../mizar-checker/en/payload_family_decomposition.md#producer-contract-shared-by-tasks-248-264-and-269-279), [Task273](../../mizar-checker/en/todo.md), Specs [07.8](../../../spec/en/07.modes.md#78-correctness-conditions), [16.6.3](../../../spec/en/16.theorems_and_proofs.md#1663-cluster-registration-obligations), [17.2–17.6](../../../spec/en/17.clusters_and_registrations.md#172-registration-blocks).
## Scope and representation
Close the source_drift between registration items and pending/initial-obligation intake, including multiple items in source order.
Reuse the authenticated four-kind source extractor and existing RegistrationValidationInput, RegistrationDatabase and InitialObligationTable.
The existing builtin-set, local equality/negated-equality attribute and identity-functor operand profile remains the supported lower-stage boundary.
Keep existing single-item and source-proof entry profiles unchanged; share extraction rather than introduce parallel syntax or term/formula representations.
New source_registration owner exposes a sealed handoff referencing original source identities, existing validations and pending obligations.
Carry kind-applicable guards, patterns and consequents (existential attributed-type witness; reduction LHS/RHS), registration order, correctness-condition site, InitialObligationId and original source anchor.
Authenticate complete original source/typed/resolver association; reject missing, duplicate/conflicting, recovered, foreign or reordered inputs transactionally.
Install and validate the same immutable handoff through TypedAst and ResolvedTypedAst; MT10-AS consumes that final handoff on existing real sources.
Only pending registrations and obligation intake are produced. No VcId, discharge, accepted status, activation, closure, rewrite result, artifact or theorem fact.
No fallback, accepted-status gate or activation path; no spec, MIZ expectation, trace status, diagnostic, proof-selection or coverage-credit changes.
## Owners and tests
API/invariants: [registration resolution](../../mizar-checker/en/registration_resolution.md#boundary); lower syntax extraction stays authenticated there.
Predecessor correctness/obligation ownership: [predicate definitions](../../mizar-checker/en/source_predicate_definition.md), Tasks259–264.
Artifacts: source_registration.rs, registration_resolution.rs, typed_ast.rs, resolved_typed_ast.rs and lib.rs; runner.rs and registration regression modules.
Existing tests: runner/tests/step5c11.rs and [existential](../../../../tests/miz/pass/clusters/pass_advanced_semantics_existential_registration_001.miz), [conditional](../../../../tests/miz/pass/clusters/pass_advanced_semantics_conditional_registration_001.miz), [functorial](../../../../tests/miz/pass/clusters/pass_advanced_semantics_functorial_registration_001.miz), [reduction](../../../../tests/miz/pass/clusters/pass_advanced_semantics_reduce_registration_001.miz) sources; preserve adjacent proof tests.
Family ledger: [registration intake](../../../../tests/coverage/registration_intake.tsv). Coverage audit and semantic deferrals remain unchanged.
## Frozen test matrix
Cap: 24 categories; these 15 categories retain the dispatch matrix. Guard/pattern/consequent means the kind-applicable source operands above.
1. Existential registration with guard, pattern and consequent: pending registration and obligation intake.
2. Conditional registration: pending registration and obligation intake.
3. Functorial registration: pending registration and obligation intake.
4. Reduction registration: pending registration and obligation intake.
5. Multiple registrations (distinct labelled items, including mixed kinds): preserve written order and obligation association.
6. Each registration kind: carry correctness-condition identity plus InitialObligationId plus source anchor.
7. Missing guard: refuse.
8. Missing pattern: refuse.
9. Missing consequent: refuse.
10. Duplicate/conflicting registration: refuse.
11. Missing correctness-condition identity: refuse.
12. Missing InitialObligationId: refuse.
13. Missing or wrong source anchor (including recovered/foreign source association): refuse.
14. Out-of-order source: refuse.
15. Any attempt to emit VcId, discharge, accepted status, activation, closure, rewrite result, artifact or theorem fact: refuse/absent.
## Review, verification and exit
Independent contract/spec/test-intent review precedes freeze; test sufficiency, implementation precision, volume and source/document consistency follow.
Run fmt, workspace all-target/all-feature Clippy with -D warnings, both lint_policy suites (including links/fragments), checker registration owners and runner registration owners.
Run short advanced-semantics CLI checks; existing proof registration owners guard shared extraction. Heavy evidence/cache/resolver obligations are unchanged.
Full cargo test follows protocol cadence. Exit requires all frozen cases, final MT10-AS consumption, nine gates and score at least90.
