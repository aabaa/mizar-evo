# Task BUILD-COMPUTATION-POLICY: package computation replay configuration
Canonical language: English; [Japanese pointer](../ja/BUILD-COMPUTATION-POLICY.md).
Status: planned. Tier: full gates. Owner: mizar-build; consumers: driver, proof, cache and test.
Authority: [§23.1 and §23.4](../../../spec/en/23.package_management_and_build_system.md), [§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step).
Classification: source_drift/test_gap; planner currently rejects computation_replay as unknown.
Readiness: manifest parsing/default/validation can start; orchestration waits for authentic driver/proof consumers.
Dependencies: existing manifest loader and package plan; real snapshot/configuration and cache identity paths for integration.
Scope: implement the linked planner-owned computation policy and authentic integration with driver/Proof.
Boundary: producer/consumer and package identity rules are owned by the linked planner boundary.
Invariant: the configuration flag never proves replay acceptance.
Failure: reject nonboolean settings through the existing manifest-validation path; absent downstream transport blocks integration rather than dropping the setting.
Forbidden: replay execution/certification, trust-policy changes, invented precedence or diagnostics, toolchain/Cargo/CI changes and cross-package policy merging.
Owner details/tests: [planner boundary](../../mizar-build/en/planner.md#computation-policy-boundary).
Tests: use the linked planner matrix with authentic driver/Proof/cache integration.
Acceptance: authentic package configuration reaches Proof without loss or stale reuse; Proof-owned override/default behavior remains unchanged.
Audit: chapter23 follow-up ownership only; no runtime/coverage achievement.
Reviews: independent specification, test sufficiency, implementation, volume/scope and source/EN-JA consistency; full protocol gates.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test.
Owner indexes: [Build](../../mizar-build/en/00.crate_plan.md), [driver](../../mizar-driver/en/00.crate_plan.md), [Proof](../../mizar-proof/en/00.crate_plan.md), [cache TODO](../../mizar-cache/en/todo.md).
