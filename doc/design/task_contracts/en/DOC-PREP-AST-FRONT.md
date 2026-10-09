# Task DOC-PREP-AST-FRONT: Freeze reflected grammar and diagnostic boundaries
Canonical language: English; [Japanese pointer](../ja/DOC-PREP-AST-FRONT.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-parser/en/00.crate_plan.md).
Consumers: implementation owners named by the scoped contracts.
Dependencies: DOC-PREP-NORMAL.
Purpose: freeze a documentation-only implementation handoff.
Authority: [§2/3/13.10/20 and Appendix A](../../../spec/en/20.algorithm_and_verification.md), [§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics); [protocol](../../autonomous_crate_development.md#task-workflow).
Scope: Freeze AST-PARSE additive syntax and AST-DIAGNOSTICS reservation/adoption contracts, paired owner-local boundaries and task indices; preserve blocked legacy grammar retirement..
Classification: source_drift/test_gap remain owned follow-ups; repair design_drift only.
Boundary: existing producer/consumer responsibilities remain; implementation and test design are future work.
Forbidden: Rust, .miz, expectations, Cargo/toolchain/CI settings, trace or coverage achievement, formal specification and volume-ledger changes; completed contracts and historical logs are not reopened.
Owner details/tests: [owner plan](../../mizar-parser/en/00.crate_plan.md), scoped implementation contracts and [documentation lint](../../../../crates/mizar-test/tests/lint_policy.rs).
Tests: compare current source and existing test intent; specify future positive, rejection and corruption cases without creating tests or claiming coverage.
Audit: update follow-up ownership or deferral only where it changes.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, source/document consistency and EN/JA; apply the protocol full gates to this preparation scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed links and protected-scope checks.
Exit: Implementation contracts distinguish ready work from explicitly blocked dependencies. No implementation acceptance or coverage credit follows.
