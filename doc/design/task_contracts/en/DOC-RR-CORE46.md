# Task DOC-RR-CORE46: recursive contract applicability preparation
Canonical language: English; [Japanese pointer](../ja/DOC-RR-CORE46.md).
Status: complete (documentation preparation only). Tier: full gates. Owner: [Core plan](../../mizar-core/en/00.crate_plan.md); consumers: [VC plan](../../mizar-vc/en/00.crate_plan.md).
Dependencies: current formal specification and existing Core46/VC56 ownership.
Authority: specification §§20.4.1,20.13.4; [protocol](../../autonomous_crate_development.md#task-workflow).
Gap: design_drift/test_gap in the declaration check and its consumer guarantee.
Scope: correct all-member terminating applicability and future acceptance/rejection controls in the paired Core owner; index that upstream guarantee from VC56.
Boundary: Resolver supplies authentic group/declaration identities; Core46 checks applicability; VC56 consumes it. VC57 retains later proof-release ownership.
Owner details/tests: [Core46 applicability](../../mizar-core/en/source_family_decomposition.md#core46-contract-applicability).
Forbidden: implementation, .miz/expectations, formal specification, Cargo/toolchain/CI, trace/coverage achievement, completed contracts/history and volume-ledger changes.
Tests: inspect formal applicability and future source/corruption controls; no new executable tests or semantic credit.
Reviews: independent specification/documentation, test sufficiency, implementation-preparation, volume/scope, consistency and EN/JA; full gates for this document scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; changed-link and protected-scope checks.
Exit: whole-group declaration eligibility is distinguished from proof acceptance; body induction hypotheses retain their own §20.13.4 entry conditions.
