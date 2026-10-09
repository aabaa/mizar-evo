# Task DOC-PREP-AST-ARTIFACT: Runtime export and staged corpus preparation
Canonical language: English; [Japanese pointer](../ja/DOC-PREP-AST-ARTIFACT.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-artifact/en/00.crate_plan.md).
Consumers: implementation owners named by the scoped contracts.
Dependencies: AST syntax/formation/lowering/contract and runtime/import handoff contracts.
Purpose: freeze a documentation-only implementation handoff.
Authority: latest doc/spec/en §§13.10/20.9/20.13.7/22.4.8/23; [protocol](../../autonomous_crate_development.md#task-workflow).
Scope: freeze AST-ARTIFACT/AST-CORPUS, paired owner boundaries and concise dependency indices.
Classification: source_drift/test_gap remain owned follow-ups; repair design_drift only.
Boundary: existing producer/consumer responsibilities remain; implementation and test design are future work.
Forbidden: Rust, .miz, expectations, Cargo/toolchain/CI settings, trace or coverage achievement, formal specification and volume-ledger changes; completed contracts and historical logs are not reopened.
Owner details/tests: [owner plan](../../mizar-artifact/en/00.crate_plan.md), scoped implementation contracts and [documentation lint](../../../../crates/mizar-test/tests/lint_policy.rs).
Tests: compare current source and existing test intent; specify future positive, rejection and corruption cases without creating tests or claiming coverage.
Audit: update follow-up ownership or deferral only where it changes.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, source/document consistency and EN/JA; apply the protocol full gates to this preparation scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed links and protected-scope checks.
Exit: Runtime export precedes execution/import records; authentic runner stages and unresolved producer designs remain explicit. No implementation acceptance or coverage credit follows.
