# Task STEP6A1-DISPATCH-PUBLICATION: scheduler dispatch publication transport

Canonical language: English; [Japanese pointer](../ja/STEP6A1-DISPATCH-PUBLICATION.md).

Status: implemented; full tier. Primary owner: mizar-driver.
Purpose: carry real IR publication resources and phase results through dispatch.
Consumers: producer phase services, later Step 6 artifact and equivalence work.
Dependencies: completed build Task 27 and IR Task 20; ready independently of S1.
Owner plan: [driver](../../mizar-driver/en/00.crate_plan.md).

## Authority and inventory

- Spec [23.4–23.5](../../../spec/en/23.package_management_and_build_system.md).
- [IV-007](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results).
- [Dispatch](../../mizar-driver/en/driver.md#scheduler-boundary),
  [registry](../../mizar-driver/en/registry.md), [publisher](../../mizar-ir/en/publisher.md).
- `source_drift`: registry execution supports publication resources, but driver
  scheduler dispatch supplies none and discards returned output refs and batches.
- Existing tests: `crates/mizar-driver/tests/{driver,registry,watch,determinism}.rs`.

## Frozen scope

- Share the existing owner-provided publisher across service calls using Arc;
  wire it through CompilerDriver without creating another publication authority.
- Preserve validated PhaseResult values by scheduler task in DriverSchedulerRun;
  synthetic scheduler outputs remain excluded from the driver result.
- Reject missing/foreign/stale publisher outputs and wrong-snapshot batches;
  any non-Complete status retains diagnostics without output handles.
- Before exposing results, require both lane/request-generation currentness
  (including same-snapshot supersession) and IR publisher snapshot/output validity;
  stale results are not retained or exposed.
- Exercise real IR sealing in transport tests; test-only services confer no
  producer readiness, artifact, proof, cache-hit, or semantic coverage credit.
- Change only driver source/tests and the corresponding paired owner sections,
  plan indexes, and top-level Step 6 sequencing entry.
- No producer adapter, serialization schema, artifact token, cache policy,
  public language diagnostic, Step 7 integration, or MVM execution is introduced.

## Dependency-ordered follow-up boundaries

1. Frontend producer codec and service; then resolver/checker/core/VC services,
   each with owner-defined canonical payloads and existing source-derived tests.
2. Artifact Task 17 for available producer projections; absent proof/witness
   outputs remain explicit Step 7 dependencies, never fabricated.
3. Build 24 / test 14 / driver 16 real clean, incremental, sequential, parallel
   equivalence; scheduler completion-order simulations alone cannot close it.
Freeze each producer task after its own input/output inventory.

## Acceptance

Independent specification, test-sufficiency, implementation, volume/scope,
and source/documentation reviews; final quality evaluation under the protocol.
Run driver tests first, then `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and
`git diff --check`. Exit: dispatch retains only validated real owner results,
with regression coverage for invalid publication and unchanged absent services.
