# Task P23-ATP-REJECTING-FIXTURES: Task-29 rejecting ATP corpus
Canonical language: English; [Japanese pointer](../ja/P23-ATP-REJECTING-FIXTURES.md).
Status: frozen; tier: full (previously deferred joint corpus execution, no family predecessor).
Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md#task-29-candidate-evidence-conformance-to-post-audit-kernel-contract).
Consumer: ATP-G-029; lower-stage owner: `mizar-kernel` (unchanged).
Readiness: the candidate-producer and variable-goal prerequisites are landed and owner-audited; retain both original `q(x)` seeds.
Authority: specification [§21.7.1](../../../spec/en/21.source_code_annotation_and_atp.md#2171-minimum-kernel-architecture-and-de-bruijn-criterion), [§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#2175-kernel-evidence-format); the two certificate seeds and expectations below.
Gap: test_gap; Task 29 has upstream refusal tests but lacks genuine ATP candidate execution of the named Kernel corpus.

## Scope and design
Tests only in `crates/mizar-atp/src/translator/tests.rs` or `mizar-test`, plus this contract and owner-index links.
Read the original JSON seeds and expectation files without altering them. Project their actual formulas,
manifests, polarity and source claims through VC handoff construction, ATP translation, both concrete
encoders and `build_local_candidate_inputs`; parse fresh candidate bytes and check them independently.
The original immutable proof-obligation handoff uses refutation polarity and valid source bindings.
For the copied-goal row, use an authenticated generated fact in the original handoff, then emit the
seed's unverifiable local-hypothesis claim; context id 99 must never enter the original context identity.
Premise closure follows the existing finder correspondence rules; both final goals remain `q(x)`.
Reuse existing test builders and wire/context helpers where possible; no fake backend or fixture-to-candidate byte conversion.
Producer boundary: [ATP candidate evidence](../../mizar-atp/en/backend.md#candidate-evidence-boundary).
Forbidden: production/API, language/specification, Kernel/schema/acceptance, seed/expectation/trace,
proof status, publication/cache, owner-gated scope, AGENTS/protocol and other task changes.
No spec-coverage-audit or trace credit promotion; historical crate-closeout records remain frozen.

## Test matrix and exit
Two corpus cases maximum, each through TPTP and SMT-LIB, with the associated canonical control.
1. `tests/certificates/fail/sat/fail_certificate_sat_goal_polarity_mismatch_001.cert.json`:
   preserve both premises and `q(x)`; consistency claim parses and rejects as `context_mismatch`;
   canonical refutation control rejects as `invalid_sat_refutation`.
2. `tests/certificates/fail/symbols/fail_certificate_symbols_unverifiable_local_hypothesis_001.cert.json`:
   preserve all three premise formulas and `q(x)`; local source 99 parses and rejects as
   `missing_provenance`; canonical generated-source control is accepted.
Bind candidate metadata to the encoded original problem, assert exact expectation category/detail keys,
and retain immutable handoffs. Establish the shared fixture with case 1 before expanding to case 2.
Reviews: specification/test intent, sufficiency, precision/trust, removable volume/scope, source/docs and scoped quality.
Verification: fmt; all-target/all-feature Clippy `-D warnings`; checker/test `lint_policy` including links;
ATP owner tests, both new corpus cases and existing `task_29_*` refusals; Kernel polarity/local-source regressions.
Full suite follows dispatch cadence; no executable behavior changes require a separate executable smoke check.
Exit: freeze committed first, both cases and required gates pass, tests-only implementation committed locally.
