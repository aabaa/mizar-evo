# Task VC-SOURCE-REGISTRATION-DISCHARGE-HANDOFF: replayed phase-12 source handoff
Canonical language: English; [Japanese pointer](../ja/VC-SOURCE-REGISTRATION-DISCHARGE-HANDOFF.md).
Status: implemented. Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Scope: VC 61; consumer ATP 30. Existing Core/source-VC/Discharge dependencies are implemented; gap: source_drift at the phase-12 handoff.
Authority: spec §§21.7.1–21.7.5; [source handoff](../../mizar-vc/en/kernel_evidence_handoff.md#source-existential-registration-proof) and [Discharge](../../mizar-vc/en/discharge.md).
Implement the handoff owner's default-Discharge NeedsAtp alternative, preserving its full-source replay, signature and original Open path.
No forced status/provenance, policy/acceptance change, reconstructed formula, new type/field/file/helper/API, or spec/.miz/expectation/trace edit.
Scope: existing VC kernel_evidence_handoff.rs and added real-source tests in mizar-test runner/tests.rs; paired handoff owners/plans/TODOs, this contract and partial audit only.
Tests use actual frontend/checker/Core/source generation and Discharge: original/renamed single and three-registration inputs, matching NeedsAtp targets, unchanged Open baselines, stale targets and coherent whole-set/provenance/status/source corruption rejection.
Require independent spec/test/implementation/volume/consistency/bounded-gate reviews; parent fmt --check, full Clippy -D warnings and cargo test.
Exit: only exact source generation or selected NeedsAtp in its exact default Discharge result yields a handoff; structured ATP/backend/proof consumers and Steps 6–8 remain open.
