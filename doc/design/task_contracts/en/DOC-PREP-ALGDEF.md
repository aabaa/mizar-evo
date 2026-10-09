# Task DOC-PREP-ALGDEF: Refresh active algorithm handoffs
Canonical language: English; [Japanese pointer](../ja/DOC-PREP-ALGDEF.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-core/en/00.crate_plan.md).
Consumers: resolver, VC/proof and mizar-test implementation owners.
Dependencies: None.
Purpose: freeze a documentation-only implementation handoff.
Authority: [§16.5.1](../../../spec/en/16.theorems_and_proofs.md#1651-simple-citation), [§20.2–20.13](../../../spec/en/20.algorithm_and_verification.md); [protocol](../../autonomous_crate_development.md#task-workflow).
Scope: Correct unfinished ALGDEF contracts and paired algorithm ownership in architecture 07, Core/VC decompositions, generator, resolver labels and TODOs; preserve equations, induction and imported-status ownership..
Classification: source_drift/test_gap remain owned follow-ups; repair design_drift only.
Boundary: existing producer/consumer responsibilities remain; implementation and test design are future work.
Forbidden: Rust, .miz, expectations, Cargo/toolchain/CI settings, trace or coverage achievement, formal specification and volume-ledger changes; completed contracts and historical logs are not reopened.
Owner details: [Core graph](../../mizar-core/en/source_family_decomposition.md#accepted-algorithm-coreir-task-graph), [VC graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph), [labels](../../mizar-resolve/en/labels.md#label-scope-families).
Existing source/tests: [generator](../../../../crates/mizar-vc/src/generator.rs), [return baseline](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [legacy match fixture](../../../../tests/miz/pass/parser/pass_parser_algorithm_control_flow_001.miz).
Tests: compare current source and existing test intent; specify future positive, rejection and corruption cases without creating tests or claiming coverage.
Audit: update follow-up ownership or deferral only where it changes.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, source/document consistency and EN/JA; apply the protocol full gates to this preparation scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed links and protected-scope checks.
Exit: Implementation contracts distinguish ready work from explicitly blocked dependencies. No implementation acceptance or coverage credit follows.
