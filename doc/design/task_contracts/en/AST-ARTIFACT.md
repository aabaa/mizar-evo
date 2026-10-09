# Task AST-ARTIFACT: Computation artifact provenance
Canonical language: English; [Japanese pointer](../ja/AST-ARTIFACT.md).
Status: planned. Tier: full gates. Primary owner: [mizar-artifact](../../mizar-artifact/en/00.crate_plan.md).
Consumers: build/loader, ALG-MVM-EVALUATION, AST-COMPUTE, mizar-cache and tooling.
Dependencies: verified algorithm/executable correspondence and execution
dependency producer first; authentic AST-COMPUTE records for later provenance.
Readiness: blocked on runtime producers; split runtime export production from
computation-record consumption to avoid a circular fake prerequisite.
Authority: [§20.9.1](../../../spec/en/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm),
[§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step),
and [Chapter 23](../../../spec/en/23.package_management_and_build_system.md).
Gap: source_drift/test_gap for validated computational interfaces and provenance.
Scope: bind public computable identity/signature/contracts to exact executable
content, verification, transitive execution dependencies and VM/rule versions.
Retain complete inputs/context/C, results and effective trust/replay mode;
preserve strict computed dependencies in publication/reuse metadata.
Public entries may carry validated private helpers without exporting names;
cross-package calls require validated public artifacts, never source fallback.
Computable public entries, including default-public declarations, need runtime
publication without an opt-in flag or contract-only computable publication.
Contract-only consumers preserve their opaque import boundary.
Deferred: executable serialization/build-loader protocol until its real producer
contract; ordinary expression metadata is not an executable artifact.
Freeze runtime export first for ALG-MVM-EVALUATION; computation-record
publication follows AST-COMPUTE and cannot block that earlier export stage.
Forbidden: invented executable correspondence/proofs, unverified supplied code,
missing-artifact fallback, weakened publication policy or fake kernel witnesses.
Owner-local invariants/tests: [verified_artifact.md](../../mizar-artifact/en/verified_artifact.md#computation-provenance-boundary).
Owner details: [exports](../../mizar-artifact/en/verified_artifact.md#exports),
[provenance](../../mizar-artifact/en/verified_artifact.md#provenance-envelope),
[hashes](../../mizar-artifact/en/verified_artifact.md#hash-participation),
[reads](../../mizar-artifact/en/store.md#validating-reads).
Tests: owner-local identity/visibility/publication matrix; existing
[reader/hash](../../../../crates/mizar-artifact/src/verified_artifact/tests.rs),
[store](../../../../crates/mizar-artifact/src/store/tests.rs) and
[witness](../../../../crates/mizar-artifact/src/proof_witness/tests.rs) tests
are infrastructure regressions, not executable correspondence evidence.
Audit: metadata work alone grants no MVM or accepted computation credit.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.
