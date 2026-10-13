# Task DOC-RR-RANKING: premise-selection ownership preparation
Canonical language: English; [Japanese pointer](../ja/DOC-RR-RANKING.md).
Status: complete (documentation preparation only). Tier: full gates. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md); consumer: [ATP plan](../../mizar-atp/en/00.crate_plan.md).
Dependencies: existing ranking and immutable translator/handoff boundaries.
Authority: specification §§21.4.1,22.4.4,23.1; [protocol](../../autonomous_crate_development.md#single-owner-documentation-rule).
Gap: design_drift/test_gap in the ranking result's consumer ownership.
Scope: assign VC selected-input assembly within the existing ranking task; reconcile paired VC/ATP owners, task indices and audit ownership.
Owner details/tests: [VC assembly](../../mizar-vc/en/vc_ir.md#planned-pretranslation-selection-assembly).
Boundary: ATP ranks authenticated inputs; VC validates/assembles the selection; immutable translation/handoff consumers do not select or repair premises.
Forbidden: new APIs/types/adapters, implementation, specification, .miz/expectations, configuration, coverage/trace achievement, completed history and ledger changes.
Tests: owner-local budget/identity integration design only; no active tests or proof credit.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, consistency and EN/JA; full gates for this document scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed-link and protected-scope checks.
Exit: ranking and VC assembly have explicit responsibility, authentic prerequisites and rejection tests, with implementation readiness honestly separated.
