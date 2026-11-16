# Task PROOF-DISCHARGE-ACCEPTANCE: by and built-in proof acceptance/publication
Canonical language: English; [Japanese pointer](../ja/PROOF-DISCHARGE-ACCEPTANCE.md).
Status: frozen for bounded M1.3 theorem-reflexivity publication. Tier: full gates.
Owners: [Proof plan](../../mizar-proof/en/00.crate_plan.md) task 23 (acceptance),
[Artifact plan](../../mizar-artifact/en/00.crate_plan.md) task 25 (witness support).
Consumers: [test plan](../../mizar-test/en/00.crate_plan.md), [driver plan](../../mizar-driver/en/00.crate_plan.md).
Authority: [§16.5](../../../spec/en/16.theorems_and_proofs.md#165-proof-justification-by),
[§21.7.1](../../../spec/en/21.source_code_annotation_and_atp.md#2171-minimum-kernel-architecture-and-de-bruijn-criterion),
[§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#2175-kernel-evidence-format),
[§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility),
and [§23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format).
Gap: source_drift/test_gap in committed readers and publication. Genuine theorem
reflexivity Kernel acceptance, validated FILE staging, Ref3.0 and main2.0 transport
already exist. Reuse those producers; no new acceptance or schema decision.
Dependencies: existing source/VC/Kernel original-obligation evidence and Artifact
atomic manifest/store boundaries. General PREATP-FORMULA-EVIDENCE, ATP evidence,
general source-proof integration and full phase-15 emission (Artifact 17) remain deferred.

Scope: publish only genuine original-source theorem-reflexivity DischargedBuiltin
formula/substitution evidence accepted by the existing Kernel. Bind the matching
accepted status to validated source-theorem FILE staging; generic Builtin and
algorithm-state staging retain publication gaps. Read committed main2.0/Ref3.0
alongside unchanged normal KernelVerified profiles, validate exact manifest/main
coverage and fresh witness bytes, and publish only the matching staged reference.
Exercise bounded genuine main/manifest assembly through existing Artifact APIs;
no native scheduler/ATP-only Driver adapter expansion. Preserve class, trusted
used_axioms, VC/context/policy/evidence identity, mandatory MISS and deterministic
discharge reuse input. Commit tokens retain past validation and caller-expected
snapshot checks only; no new currentness, source authentication or cache authority.
Backend success, flags/hashes and policy taint never supply trusted acceptance.
Forbidden: a new kernel checker/format/primitive, relabeling built-ins as ATP,
fabricated acceptance, weaker-policy reuse, lower-stage producer relocation,
protected .miz/expectation changes or development/MVM trust-policy changes.
Owner details: [selection](../../mizar-proof/en/selection.md#winner-classes),
[status](../../mizar-proof/en/status.md#source-theorem-reflexivity-status-projection),
[publication](../../mizar-proof/en/witness_store.md#publication-references),
[FILE staging](../../mizar-proof/en/witness_store.md#explicit-source-theorem-file-staging),
[witness readers](../../mizar-artifact/en/proof_witness.md#reader-and-writer-requirements),
[main profile](../../mizar-artifact/en/verified_artifact.md#explicit-built-in-enclosing-profile).
Tests (cap six grouped cases): genuine omitted-step reflexivity through captured
source/Kernel/staging/main2.0/manifest/readback/publication; matching status and
used_axioms while slice stays incomplete/MISS with no selected witness hash/cache
promotion; mismatched source/identity/policy/status and unsupported/cross-profile
FILE/main/ref binding refusal; missing/extra/changed reachability refusal;
stale expected snapshot refusal with existing retained-token limits; unchanged
normal KernelVerified, generic Builtin and algorithm-state controls. Reuse existing
source theorem witness/reflexivity and proof/artifact policy/rejection matrices
for no evidence, backend-only, missing premise, wrong goal/context/polarity/taint.
Acceptance: only the bounded genuine profile publishes distinct discharged_builtin
with a checked reachable witness; unsupported slices remain incomplete/non-trusted.
Audit: assign §§16/21/23 ownership only; no implementation coverage credit.
Reviews: specification, test sufficiency, implementation, volume/scope and consistency.
Verification: fmt; all-target/all-feature Clippy -D warnings; both lint_policy
suites; Proof/Artifact owners, affected source theorem/state and Driver committed
checks; genuine publication E2E; local-link/parity checks. Full cargo test only
on a protocol cadence trigger; freeze and implementation count as one task.
