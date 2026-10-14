# Task PREATP-FORMULA-EVIDENCE: kernel-checkable built-in discharge candidates
Canonical language: English; [Japanese pointer](../ja/PREATP-FORMULA-EVIDENCE.md).
Status: planned; source integration is dependency-blocked. Tier: full gates.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md), task 60.
Consumers: [Proof plan](../../mizar-proof/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Authority: [§21.7.1](../../../spec/en/21.source_code_annotation_and_atp.md#2171-minimum-kernel-architecture-and-de-bruijn-criterion),
[§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#2175-kernel-evidence-format),
and [§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility).
Gap: source_drift/test_gap; VC task 25 transports discharge diagnostics,
not formula/substitution evidence derived from those discharge steps.
Dependencies: VC tasks 12/25/27-29 and authentic source-derived VC, context,
formula, substitution and provenance producers for the selected discharge slice.
Equality slices also require ATPK-KERNEL-EQUALITY-INSTANCES; unsupported
trace/computation classes retain their existing producer dependencies.
Readiness: candidate design can start; task 63 supplies only bounded original-source direct-premise association. General source integration remains blocked; no route starts from markers alone.

Scope: convert supported deterministic discharge steps into the existing
formula/substitution handoff for the original VC and unchanged local context.
Use only authenticated available premises/facts and explicit substitutions;
preserve target, goal polarity, source identity and complete provenance.
Retain the goal as the final goal, never as a premise. Missing or unsupported
inputs supply no candidate and retain the actual unresolved/deferred outcome.
Boundary: VC produces candidates; Kernel checks them; Proof owns acceptance,
Artifact owns publication. MVM computed/replay remains separately owned.
Forbidden: trusted status from a rule name/hash, display-text reconstruction,
invented facts/substitutions, new kernel formats/primitives, SAT in VC,
lower-stage producer relocation, protected .miz/expectation or policy changes.
Owner details: [handoff mapping](../../mizar-vc/en/kernel_evidence_handoff.md#input-mapping),
[discharge records](../../mizar-vc/en/discharge.md#evidence-and-explanations).
Tests: real supported direct-premise and propositional-tautology VCs yield
candidates accepted by the existing kernel. Cover absent premises/payloads,
unsupported traces, marker-only/preserved status, wrong VC/context/polarity,
forged provenance and stale source; renamed equivalent sources preserve behavior.
Existing [handoff regressions](../../../../crates/mizar-vc/src/kernel_evidence_handoff.rs)
are infrastructure tests, not general source-derived discharge coverage.
Acceptance: supported genuine discharge supplies bound checkable candidates;
unavailable inputs never synthesize proof evidence or trusted used_axioms.
Audit: assign §§21/23 follow-up ownership only; implementation remains deferred.
Reviews: specification, test sufficiency, implementation, volume/scope and consistency.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected source tests and local-link/parity checks.
