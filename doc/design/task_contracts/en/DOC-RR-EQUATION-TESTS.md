# Task DOC-RR-EQUATION-TESTS: equation and induction test preparation
Canonical language: English; [Japanese pointer](../ja/DOC-RR-EQUATION-TESTS.md).
Status: complete (documentation preparation only). Tier: full gates. Owner: [Core plan](../../mizar-core/en/00.crate_plan.md); consumers: [VC plan](../../mizar-vc/en/00.crate_plan.md), [Proof plan](../../mizar-proof/en/00.crate_plan.md).
Dependencies: existing Core54/VC56 scope, source producers and formal binding rules.
Authority: specification §§4.6,20.13.1,20.13.4,20.13.6; [protocol](../../autonomous_crate_development.md#canonical-task-contracts).
Gap: test_gap in owned finite transformation branches; spec_gap in mixed-group hypothesis-dependent E2 adoption.
Scope: move and complete owner-local equation/context test matrices; index them from the existing contracts and preserve the mixed-group adoption blocker/release condition.
Owner tests: [Core54](../../mizar-core/en/source_family_decomposition.md#core54-defining-equation-tests), [VC56](../../mizar-vc/en/source_vc_decomposition.md#vc56-induction-hypothesis-tests); [adoption gap](../../mizar-vc/en/source_vc_decomposition.md#vc56-mixed-group-admission-gap).
Boundary: Core constructs unaccepted equations; VC constructs contexts; Proof's mixed-group E2 acceptance awaits canonical authority. Construction proves no termination or promotion.
Forbidden: new semantics/grammar/proofs/API, implementation, specification, .miz/expectations, configuration, coverage/trace achievement, completed history and ledger changes.
Tests: complete-formula/context comparison and malformed-input rejection design only, with expected mixed-group adoption deferred rather than guessed.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, consistency and EN/JA; full gates for this document scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed-link and protected-scope checks.
Exit: existing cases are retained, finite branches and guard/binding errors are discriminated, and unresolved adoption cannot be treated as ready.
