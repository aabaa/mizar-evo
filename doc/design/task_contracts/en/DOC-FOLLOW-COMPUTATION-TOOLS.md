# Task DOC-FOLLOW-COMPUTATION-TOOLS: replay configuration and computed tooling handoff
Canonical language: English; [Japanese pointer](../ja/DOC-FOLLOW-COMPUTATION-TOOLS.md).
Status: complete (documentation preparation only). Tier: full gates. Owner: [owner plan](../../mizar-build/en/00.crate_plan.md).
Consumers: implementation owners named by the scoped contracts.
Dependencies: None.
Purpose: freeze a documentation-only implementation handoff.
Authority: specification §§23.1,23.4–5,20.9.2; [protocol](../../autonomous_crate_development.md#task-workflow).
Scope: R02 manifest replay producer; R08 computed LSP consumer and parked refinement/minimization safeguards.
Classification: source_drift/test_gap remain owned follow-ups; repair design_drift only.
Boundary: existing producer/consumer responsibilities remain; implementation and test design are future work.
Forbidden: Rust, .miz, expectations, Cargo/toolchain/CI settings, trace or coverage achievement, formal specification and volume-ledger changes; completed contracts and historical logs are not reopened.
Owner details/tests: [owner plan](../../mizar-build/en/00.crate_plan.md), scoped implementation contracts and [documentation lint](../../../../crates/mizar-test/tests/lint_policy.rs).
Tests: compare current source and existing test intent; specify future positive, rejection and corruption cases without creating tests or claiming coverage.
Audit: update follow-up ownership or deferral only where it changes.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, source/document consistency and EN/JA; apply the protocol full gates to this preparation scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed links and protected-scope checks.
Exit: Implementation contracts distinguish ready work from explicitly blocked dependencies. No implementation acceptance or coverage credit follows.
