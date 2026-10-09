# Task AST-COMPUTE: Certified computation publication
Canonical language: English; [Japanese pointer](../ja/AST-COMPUTE.md).
Status: planned. Tier: full gates. Primary owner: [mizar-proof](../../mizar-proof/en/00.crate_plan.md).
Consumers: mizar-artifact, mizar-cache, mizar-test and tooling.
Dependencies: AST-CONTRACT verified contracts (equations when replay applies),
AST-TYPE certification,
AST-LOWER complete input and AST-ARTIFACT validated runtime interface.
Readiness: blocked on MVM/executable correspondence and proof producers;
ALG-MVM-EVALUATION owns execution and remains dependency-blocked.
Authority: [§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step),
[§20.13.7](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8),
[§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics),
and [architecture 15](../../architecture/en/15.kernel_certificate_format.md#reflection-rule-and-execution-boundary).
Gap: source_drift/test_gap for E5 binding, directed import and computation policy.
Scope: authenticate complete normal-return input/result; justify requires and
Adm/InputOK independently, instantiate complete ensures/value-equality candidates.
Apply finite T1-T8 with typed terminal UID substitution, local guards,
term/truth distinction, certified Binder construction and exact structure.
Inspect each entire candidate through types/annotations/containers; residual
AST/UID/C/operations reject it without silent ensures fragments.
Preserve internal return on import failure and distinguish actual incomplete stages.
Effective replay uses the explicit reference option over the default-false
manifest setting, for all its calls; strict release still rejects computed.
Default trust projects computed; requested replay requires all executed-call
equation/SAT evidence, never fallback; strict release retains transitive
computed dependencies through subsequent kernel checks and cache reuse.
Forbidden: universal evaluators, unrestricted unfolding, fabricated evidence,
weaker-mode cache reuse, new kernel lambda/arithmetic or producer relocation.
Owner-local invariants/tests: [policy.md](../../mizar-proof/en/policy.md#computation-publication-boundary).
Owner details: [policy](../../mizar-proof/en/policy.md#boundary-rules),
[status](../../mizar-proof/en/status.md#status-model),
[provenance](../../mizar-proof/en/witness_store.md#provenance-metadata).
Tests: use the linked owner-local matrix.
Existing [request regression](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz) supplies no execution evidence.
Audit: retain unimplemented runtime/import coverage until real execution routes.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.
