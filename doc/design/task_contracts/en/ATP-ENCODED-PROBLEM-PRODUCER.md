# Task ATP-ENCODED-PROBLEM-PRODUCER: genuine encoded backend input binding
Canonical language: English; [Japanese pointer](../ja/ATP-ENCODED-PROBLEM-PRODUCER.md).
Status: implemented. Tier: full. Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md), task 34.
Consumers: ATP30, [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: immutable validated AtpProblem, implemented real TPTP FOF/SMT-LIB encoders and existing backend container/hash machinery; source tests use ATP33. Ready for encoded input only.
Authority: spec §§21.7.2/21.7.5 and §23.4; [backend owner](../../mizar-atp/en/backend.md#genuine-encoded-input-production), [TPTP](../../mizar-atp/en/tptp_encoder.md), [SMT-LIB](../../mizar-atp/en/smtlib_encoder.md).
Gap: source_gap; production has no real encoder-to-backend binding constructor. Fixture conversions are not producers.
Scope: one existing-container factory from the actual problem and explicit format; canonical metadata and conservative provenance binding per owner, using the existing constructor. No caller-supplied encoding or hash.
Source: backend.rs; additive tests in backend/tests.rs; exactly additive public-method allowance in tests/lint_policy.rs; real-source tests appended to mizar-test runner/tests.rs. Preserve all old test bodies and Runner prefix.
Existing sources: tests/miz/pass/types/pass_type_elaboration_argument_attribute_widening_001.miz and tests/miz/fail/overload/fail_advanced_semantics_overload_ambiguous_candidates_001.miz; use unchanged real step5c3/13 producer helpers and actual default Discharge.
Tests: actual both-format bytes/labels/symbol metadata and source-problem/target/polarity/profile binding; deterministic input order/replay, diagnostic neutrality, typed symbol classes and unambiguous UTF-8 metadata; coherent provenance/source/goal/profile changes invalidate bindings; unsupported encodings propagate actual errors. Real original/renamed positive/negative single/three-registration problems reach the factory.
Forbidden: fixture hashes, raw Debug semantic encoding, recovered formulas/source from backend text, new type/field/file/dependency, generic constructor/hash/encoder changes, backend reports/processes/candidate evidence/Kernel calls/accepted statuses, protected spec/.miz/expectation/trace/test-intent edits.
Audit: no semantic coverage or follow-up ownership change; existing ATP integration remains partial. Search/verifier limits, genuine candidates, backend execution, Proof/Artifact/cache and Steps 6–8 remain open.
Require independent specification/test/implementation/volume/consistency reviews; parent cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test; scoped gates/quality review.
Exit: actual encoder output is bound to its actual immutable problem and metadata without fake hashes or proof credit; unsupported inputs fail closed.
