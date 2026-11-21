# Task AST-DIAGNOSTICS-E0206: source-bound computation scope reports
Canonical language: English; [Japanese pointer](../ja/AST-DIAGNOSTICS-E0206.md).
Status: frozen bounded adoption. Tier: full.
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md); descriptor owner: [Diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md).
Dependencies: [authenticated producer](RESOLVE-P19-COMPUTATION-SCOPE-EVIDENCE.md), [descriptor](AST-DIAGNOSTICS.md), [MBP-08](AST-MULTI-BINDER-PLAN.md).
Authority: [EN16 §16.5.1](../../../spec/en/16.theorems_and_proofs.md#1651-simple-citation), [EN22 §22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics) and [§22.7](../../../spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference).
Classification: source_drift/test_gap; the genuine reject-only producer exists, but no consumer emits its reserved E0206 descriptor.
Full tier: the producer predecessor has no diagnostic consumer; this adds observable emission and fails the no-new-consumer/diagnostic pattern criteria.

## Frozen boundary
Use the existing [Frontend diagnostic bridge](../../mizar-driver/en/frontend_adapter.md#disk-frontend-service), after clean original parsing and import validation, before output publication.
Privately collect the real declaration symbols and lower the same original; acquire the complete authenticated computation-scope evidence vector. No caller-supplied records or new public seam.
Only genuine records emit E0206 / resolve.ast_scope, Resolver phase, ResolveError category, Error severity and the existing descriptor summary/meaning key.
Each report identifies the offending source name/use, actual computation insertion and unavailable quantified declaration, with labeled current source spans and structured source details; preserve distinct repeated uses and declarations deterministically.
Validate all source identities, ranges and UTF-8 endpoints before emitting the entire batch. Recheck existing publisher/parent currentness. Nonempty reports return Recoverable without output; the scheduler retains failed-session/dependent refusal.
Unsupported, stale, recovered, mismatched or incomplete acquisition never emits E0206 or a partial batch. Existing InvalidShape, UnresolvedReference, Missing types and other errors retain their meanings; no message-text or spelling fallback.
Visible outer winners and unknown names produce no scope report. Retain all producer callee/formal/arity/type/original-source refusal controls unchanged.
Exclude checked reflection, capture/C, reconstruction/R2, certification/formation, execution, proof/acceptance and semantic/corpus/trace coverage credit; all other reflection diagnostics remain deferred.

## Artifacts and tests
Production/test scope: existing Driver frontend adapter and source-load tests; no new module, dependency, public API, specification, existing MIZ/expectation or trace edits.
Update paired Driver bridge and Diagnostics registry boundaries, owner plan backlinks, descriptor/MBP-08 mapping and the live coverage-audit deferral only where emission changes it.
Test matrix cap six families, using genuine Frontend originals and the existing real unary boolean-expr algorithm fixture:
1. Establish one scheduled source-to-report setup first; assert exact E0206 metadata, source name, use/insertion/declaration anchors, no output and failed/dependent refusal.
2. Repeated acquisition, repeated uses and different same-spelled declarations remain deterministic and distinct through aggregation/rendering.
3. Visible reserve/proof-local winners and genuinely unknown names yield no E0206; ordinary unresolved/Missing/shape outcomes remain unchanged.
4. Wrong/ambiguous/qualified callee, arity or explicit formal/type mismatch, and mixed unsupported input yield no E0206 or partial reports.
5. Recovered/foreign/stale source, parent/snapshot or symbol correspondence refuses reporting; retain producer replay tests and existing native binding/currentness controls.
6. Unicode/source relocation preserves exact current spans and readable names; no higher-stage checked/proof/execution output is created.
No synthetic positive AST, evidence or provenance. Reuse existing controls instead of duplicating unrelated lower-owner matrices.

## Exit and verification
Independent specification/test-intent/trust review before freeze; sufficiency, precision/trust, removable-only volume, source/document consistency and full-gate quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Driver source-load/adapter tests, Resolver evidence controls, Diagnostics tests and a representative scheduled end-to-end/CLI check with CARGO_BUILD_JOBS=4.
Changed diagnostic identity/currentness obligations run now; unchanged evidence-policy/acceptance and broad edit-versus-clean matrices defer to integration cadence. No lane full-test trigger, integration or push.
Exit: real source-bound E0206 reports only for the bounded producer, with complete refusal controls and no additional semantic credit.
