# Task VC-SOURCE-DIRECT-PREMISE-CANDIDATE: original-source discharge association
Canonical language: English; [Japanese pointer](../ja/VC-SOURCE-DIRECT-PREMISE-CANDIDATE.md).
Status: implemented. Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md), task 63, bounded prerequisite of task 60.
Consumers: source proof integration; Kernel checks candidates, Proof owns acceptance.
Authority: spec §§21.7.1/21.7.5; [handoff owner](../../mizar-vc/en/kernel_evidence_handoff.md#source-direct-premise-candidate).
Gap: source_drift/test_gap; original source formulas and actual direct-fact discharge records exist without their candidate association.
Dependencies: genuine Checker/Core single/three-registration inputs, original source VC set, default Discharge records and existing local handoff/wire.
Scope: one function in existing kernel_evidence_handoff accepts CoreIr, original VcSet, selected VcId and actual DischargeOutput; return existing immutable VcKernelEvidenceHandoff or an error.
Require the selected original leaf to be Open, whole source replay to match, and the complete supplied output to equal the actual default-discharge output, including its VC graph and records.
Select only the output’s actual NewlyProduced DirectLocalFact record with ExplicitInputRefs. Preserve original target, goal polarity, context, premises, symbols, substitutions and provenance through the existing source handoff.
Never project from the Discharged set or promote its marker/hash. Existing source handoff status gates remain unchanged.
Tests: actual single and all three registration direct-premise leaves, original/renamed and false-definition sources; independently encode, parse and freshly Kernel-check the genuine returned candidate.
Reject missing supported records, wrong target or differing snapshot/source/graph/records, preserved or marker-only outputs, unsupported rules, changed original goal/context/premise and Discharged input sets.
No candidate for correctness leaves or unsupported source slices; keep existing source/ATP/Proof controls unchanged.
Forbidden: new file/type/field/codec/format, fabricated source or evidence, Kernel/SAT calls in VC, acceptance/policy/status/diagnostic changes, protected spec/.miz/expectation/trace edits.
Source scope: existing VC kernel_evidence_handoff.rs and appended real-source runner/tests.rs tests; no unrelated edits or single-use helper.
Audit: bounded untrusted producer only; general PREATP, full acceptance, distinct built-in witness schema/publication, Steps 6–8 and semantic coverage remain open.
Reviews: specification, test sufficiency, code, volume/scope, consistency, scoped gates/quality; parent fmt, full Clippy -D warnings and cargo test.
Exit: genuine supported records yield the unchanged original-source candidate; unavailable or mismatched inputs fail closed without trusted credit.
