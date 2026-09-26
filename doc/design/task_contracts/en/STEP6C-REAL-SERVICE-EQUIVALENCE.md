# Task STEP6C-REAL-SERVICE-EQUIVALENCE: equivalence over real phase services
Canonical language: English; [Japanese pointer](../ja/STEP6C-REAL-SERVICE-EQUIVALENCE.md).
Status: planned. Tier: full. Owner: [build plan](../../mizar-build/en/00.crate_plan.md); consumers: [driver plan](../../mizar-driver/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: at least one real phase service after SourceLoad registered as available (Frontend is); later services extend the same gate as they land.
Authority: [architecture 22](../../architecture/en/22.incremental_verification_contract.md) (IV-002, IV-003); [incremental/parallel equivalence](../../mizar-build/en/incremental_parallel_equivalence.md).
Gap: BUILD-G-017 `external_dependency_gap`. Build task 24 and driver task 16 prove equivalence only for the implemented seams, not for real phase outputs.

## Scope

Run the clean/incremental and sequential/parallel equivalence gate over real
workspaces through every phase service registered as available, comparing
published outputs, diagnostics, and cache decisions. Close BUILD-G-017 for
those phases and keep it open, by name, for services that remain gaps.

## Forbidden

Synthetic phase outputs; comparing only event streams; weakening the existing
seam-level gate; claiming equivalence for unregistered services.

## Tests and exit

Real multi-module workspaces with an edit between runs, several worker counts,
and cache hits and misses; stale-snapshot rejection. Require independent
specification, test-sufficiency, implementation, volume/scope, and
consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: the gate holds for every registered real service, and BUILD-G-017 names
only the services still missing.
