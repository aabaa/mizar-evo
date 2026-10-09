# Task AST-CORPUS: Reflection executable test preparation
Canonical language: English; [Japanese pointer](../ja/AST-CORPUS.md).
Status: planned. Tier: full gates. Primary owner: [mizar-test](../../mizar-test/en/00.crate_plan.md).
Consumers: AST implementation owners and specification coverage maintenance.
Dependencies: frozen owner contracts and actual stage-specific runner inputs.
Readiness: AST-PARSE additive test-first matrix ready after contract freeze;
typed meaning/contracts/computation/replay matrices blocked on their producers.
Authority: [§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--),
[§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax),
[§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step),
[§20.13.7](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8),
and [§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics).
Gap: test_gap; source specification examples assert obligations, not success.
Scope: new spec-derived .miz/sidecar/traceability designs per frozen owner slice;
stage only real source-to-producer-to-consumer routes with authentic observations.
Cover grammar/position, immutable structure/UIDs, typed meaning/Binder/C,
verified contracts, complete public candidate rejection, artifact trust/replay
and strict-policy dependencies; link semantic matrix details to owner modules.
Existing .miz/semantic expectations stay fixed; future trace/activation changes
require a separately authorized runnable slice with authentic stage observations.
Protected exhaustive pass fixture blocks retirement; never rebaseline it here.
Forbidden: synthetic reflection observations, success from examples/history,
promoting Open requests into computation proof, phantom runner activation,
invented foundational behavior, syntax or diagnostic IDs.
Owner-local invariants/tests: [harness.md](../../mizar-test/en/harness.md#reflection-corpus-boundary).
Owner details: [runner modes](../../mizar-test/en/harness.md#runner-modes),
[parse expectations](../../mizar-test/en/expectation_schema.md#parse-only-expectations),
[coverage](../../mizar-test/en/traceability.md#coverage-status).
Tests: harness/linked owner matrices require real positive, rejection and
corruption observations. Existing [parse-only](../../../../crates/mizar-test/src/runner/tests/parse_only.rs)
and [VC runner](../../../../crates/mizar-test/src/runner/tests/proof_verification.rs)
tests are regressions, not new reflection coverage.
Audit: update only the actual activated stage; all deferred stages retain gaps.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.
