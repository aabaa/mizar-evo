# Task AST-TYPE: Reflected syntax formation
Canonical language: English; [Japanese pointer](../ja/AST-TYPE.md).
Status: planned. Tier: full gates. Primary owner: [mizar-checker](../../mizar-checker/en/00.crate_plan.md).
Consumers: mizar-core, mizar-vc and mizar-proof.
Dependencies: AST-PARSE syntax, authentic resolved declaration/type records,
and the pinned library identities and premise-checked proofs of §20.5.3.
Readiness: blocked on those producers; no same-spelling certification.
Authority: [§3.3](../../../spec/en/03.type_system.md#33-type-expressions),
[§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--),
[§19.4.2](../../../spec/en/19.overload_resolution.md#1942-viable-candidates),
[§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax),
and [§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics).
Gap: source_drift/test_gap for typed reflection, retained finite conditions,
ghost meaning/valuation, Binder modes and capture-specific reconstruction C.
Scope: certify the supported classification; preserve original types/attributes,
resolved definitions, complete value arguments, UIDs, guards and type views.
Check covariance/R3 represented types, Adm/InputOK, local guarded updates,
independent public valuation result types and all-input Binder formation.
Separate fresh introduction, pattern capture and captured reconstruction;
preserve free identity and include C in complete input before core handoff.
Deferred: uncertified structure updates/carriers, public environment/UID/
substitution/reconstruction APIs, generic registration and pattern annotations.
Forbidden: invented foundational definitions/proofs, mode fallback, runtime
meaning flow, acceptance from names/individual assumptions or MVM execution.
Owner-local invariants/tests: [typed_ast.md](../../mizar-checker/en/typed_ast.md#reflected-syntax-formation).
Owner details: [type ownership](../../mizar-checker/en/typed_ast.md#typetable),
[declarations](../../mizar-checker/en/binding_env.md#binder-identity),
[resolution](../../mizar-checker/en/overload_resolution.md#viability-over-recorded-facts).
Tests: use the linked owner-local matrix; evaluation/proof acceptance remains downstream.
Existing [binder/context tests](../../../../crates/mizar-checker/src/binding_env.rs) cover lower-stage identity only.
Audit: change reflection typing coverage only after a real owner runner exists.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.
