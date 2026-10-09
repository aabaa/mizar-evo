# Task DOC-PREP-AST-TYPE: Freeze reflected typing and lowering
Canonical language: English; [Japanese pointer](../ja/DOC-PREP-AST-TYPE.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-checker/en/00.crate_plan.md).
Consumers: implementation owners named by the scoped contracts.
Dependencies: DOC-PREP-AST-FRONT.
Purpose: freeze a documentation-only implementation handoff.
Authority: [§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--), [§20.2.4/20.5.3](../../../spec/en/20.algorithm_and_verification.md); [protocol](../../autonomous_crate_development.md#task-workflow).
Scope: Freeze AST-TYPE and AST-LOWER, paired formation/immutable handoff boundaries, and set-versus-sequence iteration ownership; preserve certification and producer blockers..
Classification: source_drift/test_gap remain owned follow-ups; repair design_drift only.
Boundary: existing producer/consumer responsibilities remain; implementation and test design are future work.
Forbidden: Rust, .miz, expectations, Cargo/toolchain/CI settings, trace or coverage achievement, formal specification and volume-ledger changes; completed contracts and historical logs are not reopened.
Owner details/tests: [owner plan](../../mizar-checker/en/00.crate_plan.md), scoped implementation contracts and [documentation lint](../../../../crates/mizar-test/tests/lint_policy.rs).
Tests: compare current source and existing test intent; specify future positive, rejection and corruption cases without creating tests or claiming coverage.
Audit: update follow-up ownership or deferral only where it changes.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, source/document consistency and EN/JA; apply the protocol full gates to this preparation scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed links and protected-scope checks.
Exit: Implementation contracts distinguish ready work from explicitly blocked dependencies. No implementation acceptance or coverage credit follows.
