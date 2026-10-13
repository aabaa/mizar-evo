# Task DOC-FOLLOW-CONSTRUCTOR-ACTUALS: general constructor argument preparation
Canonical language: English; [Japanese pointer](../ja/DOC-FOLLOW-CONSTRUCTOR-ACTUALS.md).
Status: complete (documentation preparation only). Tier: full gates. Owner: [owner plan](../../mizar-parser/en/00.crate_plan.md).
Consumers: implementation owners named by the scoped contracts.
Dependencies: None.
Purpose: freeze a documentation-only implementation handoff.
Authority: specification §§3.3,18.2.2,18.4 and Appendix A.3; [protocol](../../autonomous_crate_development.md#task-workflow).
Scope: R03 general bracket value expressions and resolved type/value role consumers.
Classification: source_drift/test_gap remain owned follow-ups; repair design_drift only.
Boundary: existing producer/consumer responsibilities remain; implementation and test design are future work.
Forbidden: Rust, .miz, expectations, Cargo/toolchain/CI settings, trace or coverage achievement, formal specification and volume-ledger changes; completed contracts and historical logs are not reopened.
Owner details/tests: [owner plan](../../mizar-parser/en/00.crate_plan.md), scoped implementation contracts and [documentation lint](../../../../crates/mizar-test/tests/lint_policy.rs).
Tests: compare current source and existing test intent; specify future positive, rejection and corruption cases without creating tests or claiming coverage.
Audit: update follow-up ownership or deferral only where it changes.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, source/document consistency and EN/JA; apply the protocol full gates to this preparation scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed links and protected-scope checks.
Exit: Implementation contracts distinguish ready work from explicitly blocked dependencies. No implementation acceptance or coverage credit follows.
