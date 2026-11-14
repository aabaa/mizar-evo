# Task CHECKER-STACK-DEPTH-FIX: Default-stack test repair

Canonical language: English. [Japanese pointer](../ja/CHECKER-STACK-DEPTH-FIX.md).

## Ownership and readiness

Status: frozen for implementation; gate tier: full. Purpose: remove target tests' default-thread stack overflows without reducing assertions or coverage.
Primary owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); [backlog](../../mizar-checker/en/todo.md#checker-test-stack-depth-follow-up). Consumer: [mizar-test](../../mizar-test/en/00.crate_plan.md#responsibility).
Dependencies: existing fixtures and checker/runner implementations; ready for cause measurement, no language-design authorization.
Authority: dispatched Unit definition items 1–12; [workflow](../../autonomous_crate_development.md#task-workflow); backlog; checkpoint commits `c278a7e64`, `9467966a9`, `05d1448db`, `47c89014c`.

## Boundary and affected artifacts

Gap: test-infrastructure stack-depth defect; independently measure each target's recursion and stack-resident data before repair.
Owner decisions (2026-11-14): option (a) adds C, exposed by r2's acceptance gate after A's repair; option (b) authorizes enumeration then one repair pass in this unit for up to ten additional overflowing tests E.
Phase 0: build once with `env -u RUST_MIN_STACK cargo test --locked --no-run`; run each unfiltered at default stack. For aborting binaries, list tests; run each separately with `--exact` and its full path. Record binary outcomes, E names/files and isolated SIGABRT evidence in the run directory's enumeration.md. Known source_empty must appear; recheck held hashes. Stop before amendment/repair if E exceeds ten or enumeration sanity fails.
Targets (A/B/C, then E):
- `source_statement::tests::task258b3m2b2b1b1::task258b3m2b2b1b1_combined_ownership_hybrids_and_all_family_orders_are_atomic` — `crates/mizar-checker/src/source_statement/tests/task258b3m2b2b1b1.rs`.
- `runner::proof_verification::term_proof_tests::source_reserved::source_reserved_statement_original_capture_preserves_installation_boundary` — `crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_reserved.rs`.
- `source_statement::tests::task258b3m2b2b2b::task258b3m2b2b2b_combined_ownership_hybrids_and_all_family_orders_are_atomic` — `crates/mizar-checker/src/source_statement/tests/task258b3m2b2b2b.rs`.
- `runner::proof_verification::term_proof_tests::source_empty::source_empty_enumeration_witness_original_capture_retains_lower` — `crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_empty.rs`.
- `runner::proof_verification::term_proof_tests::source_theorem_scoped::task257b6_native_connective_composition_preserves_installed_replay` — `crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_theorem_scoped.rs`.
- `runner::proof_verification::term_proof_tests::source_theorem_scoped::task257b7_native_quantifier_composition_preserves_installed_replay` — `crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_theorem_scoped.rs`.
- `runner::proof_verification::term_proof_tests::source_theorem_scoped::task258_native_let_statements_preserve_source_intent_and_installed_replay` — `crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_theorem_scoped.rs`.
Affected artifacts: target tests and proven stack-consuming production callees; documentation only this contract pair and linked backlog entries.
Record each measured cause and mechanism in the implementation commit. Permitted: smaller fixtures/frames, heap allocation, recursion restructuring, or splitting combined cases while retaining every named test and assertion. Prefer reducing production recursion when responsible.
Production behavior, diagnostics, proof acceptance and lower-stage ownership remain unchanged. Owner invariants/API: [source statements](../../mizar-checker/en/source_statement.md); runner/test design: [consumer runner](../../mizar-test/en/00.crate_plan.md#task-10-consumer-runner-support), [tests](../../mizar-test/en/00.crate_plan.md#relevant-tests).
No specification, expectation, traceability, coverage-credit or audit change; defer semantics to owner.
Forbidden: stack-size environment/configuration masking; ignoring, deleting, renaming targets; weakened assertions/coverage; relaxed gates, lints or baselines, including `doc_volume_baseline.tsv`.
Disclose per test any body moved to a spawned thread with an explicitly sized stack in commit and final response; production/runner stack sizing remains forbidden.
Preserve held A/B/C; shared-helper exceptions require explanation. Stop on held-state mismatch, failed full gate (including an unenumerated overflow), semantic change or owner-only decision, plus protocol stops.

## Test matrix and exit

Case cap: 12; additional sufficiency cases require a specification clause or `.miz` test.

| Case | Input/check | Expected outcome |
| --- | --- | --- |
| M1 | Historical isolated A red | Default-stack overflow confirmed. |
| M2 | Historical isolated B red | Default-stack overflow confirmed. |
| M3 | Absorbed into MFINAL | Entire suite passes. |
| M4 | Absorbed into MFINAL | Every target passes within that run. |
| M5 | Review full diff; grep `RUST_MIN_STACK`, `stack_size` | No forbidden masking or target/assertion/coverage reduction. |
| M6 | Formatting, Clippy, both lint suites | All pass. |
| M7 | Historical isolated C red | Default-stack overflow confirmed. |
| M8 | Historical C gate observation; renew via MFINAL | C passes. |
| M9 | MFINAL A/B observations | Both pass in that gate. |
| ME1 | Complete Phase 0 record | Exhaustive universe, outcomes, E list/files/count, cap and sanity checks. |
| ME2 | Each E's isolated Phase 0 record | Full-path --exact process aborts with stack overflow; no red reruns. |
| MFINAL | `env -u RUST_MIN_STACK cargo test --locked` | Exit 0; every target observed passing by full name in this same run. |

Reviews: independent specification/test intent before amendment; sufficiency, precision, volume/scope, consistency; protocol quality assessment after gates.
Freeze: both `lint_policy` suites and link/fragment checks, then documentation commit before repairs.
Implementation: `cargo fmt` then `cargo fmt --check`; `cargo clippy --all-targets --all-features -- -D warnings`; both `lint_policy` suites; MFINAL. Use `CARGO_BUILD_JOBS=4`; no stack-size setting.
Filed acceptance triggers the full run, including owner/E2E checks. Exit: all cases/reviews pass, then close both backlog entries and commit; one changed task across freeze/amendments/implementation; no push.
