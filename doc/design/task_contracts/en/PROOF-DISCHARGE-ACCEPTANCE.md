# Task PROOF-DISCHARGE-ACCEPTANCE: by and built-in proof acceptance/publication
Canonical language: English; [Japanese pointer](../ja/PROOF-DISCHARGE-ACCEPTANCE.md).
Status: planned; source/publication integration is dependency-blocked. Tier: full gates.
Owners: [Proof plan](../../mizar-proof/en/00.crate_plan.md) task 23 (acceptance),
[Artifact plan](../../mizar-artifact/en/00.crate_plan.md) task 25 (witness support).
Consumers: [test plan](../../mizar-test/en/00.crate_plan.md), [driver plan](../../mizar-driver/en/00.crate_plan.md).
Authority: [§16.5](../../../spec/en/16.theorems_and_proofs.md#165-proof-justification-by),
[§21.7.1](../../../spec/en/21.source_code_annotation_and_atp.md#2171-minimum-kernel-architecture-and-de-bruijn-criterion),
[§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#2175-kernel-evidence-format),
[§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility),
and [§23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format).
Gap: source_drift/test_gap; trusted selection exists, but general source proof
integration and distinct DischargedBuiltin witness publication remain deferred.
Dependencies: PREATP-FORMULA-EVIDENCE for built-in candidates; ATPK-INSTANCE-FINDER
and authentic backend/candidate outputs for the ATP slice; existing kernel
checks, source VC/citation/context producers and Artifact witness/store boundaries.
Readiness: acceptance/witness design can start; each source slice waits for its
actual producers. Proof 26 owns bounded direct-premise classification; full phase-15 emission remains Artifact task 17's work.

Scope: check genuine candidate evidence for the original proof obligation before
trusted selection. Preserve accepted untainted proof-obligation binding and
classify origin as kernel_verified for ATP or discharged_builtin for built-ins.
Apply the same acceptance discipline to ordinary by and omitted justifications.
Backend success or a discharge flag/hash alone never supplies trusted status
or used_axioms. Preserve kernel rejections and policy-permitted external status;
computed/replay follows its existing separate ownership and release policy.
Artifact admits distinct discharged_builtin witnesses using the existing
formula/substitution evidence family; freeze/version any necessary status-schema
delta before code. Proof stages/exports only matching accepted results;
Artifact retains validating reads, atomic publication and manifest reachability.
Reuse metadata preserves class, VC/context/policy and evidence identity;
until actual witness support exists, retain the unsupported publication gap.
Forbidden: a new kernel checker/format/primitive, relabeling built-ins as ATP,
fabricated acceptance, weaker-policy reuse, lower-stage producer relocation,
protected .miz/expectation changes or development/MVM trust-policy changes.
Owner details: [selection](../../mizar-proof/en/selection.md#winner-classes),
[status](../../mizar-proof/en/status.md#artifact-projection),
[publication](../../mizar-proof/en/witness_store.md#publication-references),
[witness readers](../../mizar-artifact/en/proof_witness.md#reader-and-writer-requirements).
Tests: real by proof and omitted-justification tautology; no accepted evidence,
backend-only success under strict/relaxed policies, missing cited premise,
kernel rejection, wrong goal/context/polarity, policy taint and stale evidence.
Round-trip both trusted classes with real accepted evidence, used_axioms and
committed witnesses; reject forged flags, hashes, status, bytes and reachability.
Existing [selection tests](../../../../crates/mizar-proof/src/selection/tests.rs)
do not establish ordinary source proof or built-in publication coverage.
Acceptance: supported genuine proofs publish the correct distinct status and
checked witness; every unsupported/unchecked path remains non-trusted.
Audit: assign §§16/21/23 ownership only; no implementation coverage credit.
Reviews: specification, test sufficiency, implementation, volume/scope and consistency.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected source tests and local-link/parity checks.
