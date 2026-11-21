# Task P24-GENERIC-STATE-BUILTIN-PUBLICATION: generic draft and state built-in witness publication
Canonical language: English; [Japanese pointer](../ja/P24-GENERIC-STATE-BUILTIN-PUBLICATION.md).
Status: frozen. Tier: full; a new public draft-to-FILE API fails the predecessor's pattern-tier public-seam criterion.
Owners: [Proof plan](../../mizar-proof/en/00.crate_plan.md), [Artifact plan](../../mizar-artifact/en/00.crate_plan.md).
Consumer: [Test plan](../../mizar-test/en/00.crate_plan.md).
Predecessor: [PROOF-DISCHARGE-ACCEPTANCE](PROOF-DISCHARGE-ACCEPTANCE.md); dependency P2 owner synchronization is complete.
Authority: [§16.5](../../../spec/en/16.theorems_and_proofs.md#165-proof-justification-by),
[§21.7.1](../../../spec/en/21.source_code_annotation_and_atp.md#2171-minimum-kernel-architecture-and-de-bruijn-criterion),
[§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#2175-kernel-evidence-format),
[§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility),
[§23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format).
Gap: derived source_drift/test_gap; generic Builtin drafts lack a publishable FILE route and committed reads exclude existing state FILEs.
Readiness: existing native theorem/state fixtures, Kernel-checked drafts, FILE schemas, Ref3/main2 and manifest APIs suffice.

Scope: witness_store.rs, proof_witness.rs, their owner documents and existing Proof/Artifact/Test tests.
Add ProofWitnessDraft::source_theorem_file_value(&self) -> Result<CanonicalJson, String> for bounded DischargedBuiltin reflexivity-checked-wire2.0 only.
Emit the existing eight-key source-theorem FILE1.0 from actual draft identity, whole payload and full metadata; reuse the explicit Artifact reference encoder for metadata.
Keep legacy seven-key payload_hash_value and generic stage hashes/RefNone unchanged. Real Artifact transport and existing stage_source_theorem_file associate the new value with its draft.
Dispatch committed witness reads by exact theorem/state FILE1.0 hash family/version; reuse their existing readers and unchanged hash domains.
Allow existing validated state FILE1.0 Ref3 candidates through publish_ref, preserving exact main/manifest coverage, fresh bytes and expected snapshot checks.
No new schema, type, field, acceptance, source authentication, currentness, cache authority, lower-stage producer relocation or language-specification change.
Nontrusted/unsupported profiles remain refused; preserve normal KernelVerified, theorem FILE, deterministic discharge reuse and incomplete/MISS boundaries.
Owner invariants: [staging/publication](../../mizar-proof/en/witness_store.md#publication-references), [reference codec](../../mizar-artifact/en/proof_witness.md#built-in-reference-profile).
Audit/trace: ownership and implementation coverage remain unchanged; no coverage credit.

Tests (cap six grouped cases, existing native fixtures; no fabricated positive acceptance):
1. Genuine generic theorem draft value equals existing source producer FILE; actual write/read/stage/main2/manifest/token/publication roundtrip; legacy hash/stage unchanged.
2. Genuine state FILE actual write/read/stage/main2/manifest/token/publication roundtrip; preserve whole metadata/provenance.
3. Matching source-theorem status binding/used axioms and unchanged state/theorem incomplete/MISS/selected-hash/reuse controls.
4. Unsupported draft/class/profile/version and coherent source/policy/payload cross-association refusals; normal codecs and theorem/state cross-binding controls.
5. Missing/extra/replaced committed witness, mismatched enclosing references and changed fresh FILE refusal; reuse predecessor reachability matrices.
6. Stale expected snapshot refusal and retained-token limits; unchanged normal KernelVerified and existing theorem FILE publication controls.
Reviews: specification/test intent before freeze; independent sufficiency, precision, volume/scope and source/docs consistency after implementation.
Verification: fmt; all-target/all-feature Clippy -D warnings; both lint_policy suites/link checks; Proof/Artifact/Kernel/VC owners; affected source theorem/state and Driver committed cases; genuine publication E2E.
Full cargo test follows cadence only; this freeze and implementation count as one task. Exit requires the six groups and reviews with no unresolved findings.
