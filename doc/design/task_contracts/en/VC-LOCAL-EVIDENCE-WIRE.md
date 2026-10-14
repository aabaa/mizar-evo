# Task VC-LOCAL-EVIDENCE-WIRE: shared local formula evidence transport
Canonical language: English; [Japanese pointer](../ja/VC-LOCAL-EVIDENCE-WIRE.md).
Status: implemented. Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md), task 62.
Consumers: [Proof plan](../../mizar-proof/en/00.crate_plan.md), ATP 30.
Dependencies: existing immutable VC handoff, Kernel Formula v1/v2 and named equality instances; ready for local source transport only.
Authority: spec §§21.7.1/21.7.5; [handoff owner](../../mizar-vc/en/kernel_evidence_handoff.md#local-binary-transport), [Kernel grammar](../../mizar-kernel/en/formula_evidence.md#named-equality-instances), [Proof source owner](../../mizar-proof/en/status.md#source-existential-registration-proof).
Gap: source_drift; normal local wire framing is private to Proof and unavailable to independent candidate producers.
Scope: add the owner's one local binary encoder to the existing VC handoff; replace only Proof's duplicate framing, preserving its actual instance selection, parsing, checks and activation.
Source scope: VC kernel_evidence_handoff.rs, Proof status.rs; additive unit tests there and real-source tests appended to mizar-test runner/tests.rs. No new file/type/field/dependency or changes to existing test intent.
Tests: independently fixed v1/v2 wire framing; all supported local source classes and side-condition transport; unsupported profile/import/polarity/context rejection; actual original/renamed positive/negative single/three-registration source handoffs round-trip through the existing parser and fresh Kernel check, including original Open and actual NeedsAtp after Discharge. Keep old source proof/corruption tests active and unchanged.
Existing source authorities: tests/miz/pass/types/pass_type_elaboration_argument_attribute_widening_001.miz and tests/miz/fail/overload/fail_advanced_semantics_overload_ambiguous_candidates_001.miz; existing step5c3/step5c13 real-source helpers in the runner. Negative variants alter in-memory source only.
Forbidden: formula recovery from text or hashes, new grammar, selected facts/instances/substitutions, Kernel calls in VC, acceptance/policy/status changes, fabricated producers, protected spec/.miz/expectation/trace changes.
Audit: coverage and follow-up ownership unchanged; only existing transport implementation is shared. General extraction, deterministic search/policy limits, backend reports, built-in candidates and Steps 6–8 remain open.
Require independent specification, test, implementation, volume and consistency reviews; parent cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test; scoped hard gates and quality review.
Exit: local handoff bytes follow the existing Kernel grammar, Proof uses the same framing without changing its fresh acceptance behavior, and unsupported inputs fail closed.
