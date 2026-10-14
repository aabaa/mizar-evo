# Task VC-LOCAL-EVIDENCE-COMPONENT-HASHES: raw local evidence component identities
Canonical language: English; [Japanese pointer](../ja/VC-LOCAL-EVIDENCE-COMPONENT-HASHES.md).
Status: implemented (bounded raw component production). Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md), task 64; consumer Proof/Artifact metadata prerequisites.
Authority: spec §21.7.5/§23.5.1; [sole handoff owner](../../mizar-vc/en/kernel_evidence_handoff.md#local-evidence-component-hashes).
Dependencies: existing canonical handoff and local v1 transport; accepted-result/policy metadata association, trusted tokens and publication remain absent.
Gap: source_drift/test_gap; genuine local component payloads have no producer-owned component digests.
Scope: VcKernelEvidenceHandoff::local_evidence_component_hashes(&self) -> Result<[Hash; 4], String> in existing kernel_evidence_handoff.rs, ordered profile/formula/substitutions/provenance.
Use the owner's exact versioned framing of existing canonical hash-input sections after successful existing local v1 encoding. No new AST, wire, hashing primitive or acceptance authority.
Preserve original whole-envelope/context hashes and wire; raw component hashes do not authenticate source/currentness or replace target, context, accepted result or policy identities.
Tests: genuine single/three source registrations, original/renamed/false definitions and snapshots; actual NeedsAtp and direct-premise handoffs, exact original bytes/inputs and independent literal frames/orders.
Verify profile and actual empty substitutions independently; source/goal/provenance/context changes affect the corresponding formula/provenance components. Unsupported local framing supplies no hashes; no fake producer/evidence/Accepted/Hit or seeded positive metadata.
Write scope: existing method and additive existing local unit tests plus appended runner source tests, no helper/type/field/file/adapter/dependency or existing spec/.miz/expectation/trace/policy changes.
Audit: component-production prerequisite only; full VC60/Proof23/Artifact25, metadata association/trust/publication/reuse and Steps6–8 remain open without semantic coverage promotion.
Reviews: independent specification/tests/code/volume/consistency, scoped gates/quality; parent fmt, full Clippy -D warnings and cargo test.
Exit: supported actual local handoffs produce exact deterministic four-component identities, unsupported inputs return Err, and old handoff/check/policy/schema behavior is unchanged.
