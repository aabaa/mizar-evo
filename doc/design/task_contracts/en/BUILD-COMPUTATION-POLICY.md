# Task BUILD-COMPUTATION-POLICY: package computation replay configuration
Canonical language: English; [Japanese pointer](../ja/BUILD-COMPUTATION-POLICY.md).
Status: Build31 metadata slice implemented; full integration remains pending. Tier: full gates. Owner: mizar-build; consumers: driver, proof, cache and test.
Authority: [§23.1 and §23.4](../../../spec/en/23.package_management_and_build_system.md), [§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step).
Classification: metadata parser/config source_drift resolved; authentic runtime/Proof/configuration identity integration remains a producer dependency.
Dependencies: existing manifest loader and package plan; real snapshot/configuration and cache identity paths for integration.
Scope: implement the linked planner-owned computation policy and authentic integration with driver/Proof.
Current slice (Build31): add computation_replay: bool only to existing VerifierConfig, false Default and existing strict boolean parser; preserve existing manifest-to-PackagePlan-to-ordered WorkspaceVerifierConfig copies. No new representation/helper/dependency or runtime policy.
Boundary: producer/consumer and package identity rules are owned by the linked planner boundary.
Invariant: the configuration flag never proves replay acceptance.
Failure: reject nonboolean settings through the existing manifest-validation path; absent downstream transport blocks integration rather than dropping the setting.
Forbidden: replay execution/certification, trust-policy changes, invented precedence or diagnostics, toolchain/Cargo/CI changes and cross-package policy merging.
Owner details/tests: [planner boundary](../../mizar-build/en/planner.md#computation-policy-boundary).
Tests: Build31 covers omission/false/true/nonboolean, opposite workspace packages/order and typed config-only change, genuine native acquisition/Driver plan metadata. Replace the prior Rust unsupported-false control with invalid nonboolean while retaining parser-delegation intent; protected spec/.miz/expectations/trace remain unchanged. Full integration uses the linked owner matrix.
Slice exit: actual parsed package values remain distinct in package plans and ordered aggregate config, with existing InvalidType for nonboolean. Snapshot/config hashing, Proof override/replay, dependency authenticity and cache invalidation remain deferred; no Accepted/Hit or whole Steps 6–8 credit.
Acceptance: authentic package configuration reaches Proof without loss or stale reuse; Proof-owned override/default behavior remains unchanged.
Audit: chapter23 follow-up ownership only; no runtime/coverage achievement.
Reviews: independent specification, test sufficiency, implementation, volume/scope and source/EN-JA consistency; full protocol gates.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test.
Owner indexes: [Build](../../mizar-build/en/00.crate_plan.md), [driver](../../mizar-driver/en/00.crate_plan.md), [Proof](../../mizar-proof/en/00.crate_plan.md), [cache TODO](../../mizar-cache/en/todo.md).
