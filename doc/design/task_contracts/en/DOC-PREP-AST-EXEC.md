# Task DOC-PREP-AST-EXEC: Freeze obligations and computation trust
Canonical language: English; [Japanese pointer](../ja/DOC-PREP-AST-EXEC.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-vc/en/00.crate_plan.md).
Consumers: implementation owners named by the scoped contracts.
Dependencies: DOC-PREP-AST-TYPE.
Purpose: freeze a documentation-only implementation handoff.
Authority: [§20.9/E1–E5/T1–T8](../../../spec/en/20.algorithm_and_verification.md), [architecture 15](../../architecture/en/15.kernel_certificate_format.md#reflection-rule-and-execution-boundary); [protocol](../../autonomous_crate_development.md#task-workflow).
Scope: Freeze AST-CONTRACT, blocked ALG-MVM-EVALUATION and AST-COMPUTE ownership/dependencies, paired algorithm/policy boundaries and test designs; no execution/proof/status acceptance..
Classification: source_drift/test_gap remain owned follow-ups; repair design_drift only.
Boundary: existing producer/consumer responsibilities remain; implementation and test design are future work.
Forbidden: Rust, .miz, expectations, Cargo/toolchain/CI settings, trace or coverage achievement, formal specification and volume-ledger changes; completed contracts and historical logs are not reopened.
Owner details/tests: [owner plan](../../mizar-vc/en/00.crate_plan.md), scoped implementation contracts and [documentation lint](../../../../crates/mizar-test/tests/lint_policy.rs).
Tests: compare current source and existing test intent; specify future positive, rejection and corruption cases without creating tests or claiming coverage.
Audit: update follow-up ownership or deferral only where it changes.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, source/document consistency and EN/JA; apply the protocol full gates to this preparation scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed links and protected-scope checks.
Exit: Implementation contracts distinguish ready work from explicitly blocked dependencies. No implementation acceptance or coverage credit follows.
