# Task CHECKER-STACK-DEPTH-FIX: Default-stack test repair

Canonical language: English. [Japanese pointer](../ja/CHECKER-STACK-DEPTH-FIX.md).

## Ownership and readiness

Status: frozen for implementation; gate tier: full. Purpose: remove the causes of default-test-thread stack overflow in the two named tests without reducing their assertions or coverage.
Primary owner: [mizar-checker plan](../../mizar-checker/en/00.crate_plan.md); [mizar-checker backlog](../../mizar-checker/en/todo.md#checker-test-stack-depth-follow-up).
Consumer: [mizar-test](../../mizar-test/en/00.crate_plan.md#responsibility).
Dependencies: existing test fixtures and checker/runner implementations; ready for cause measurement, with no language-design decision authorized.
Authority: dispatched Unit definition items 1–12; [workflow](../../autonomous_crate_development.md#task-workflow); [filed backlog](../../mizar-checker/en/todo.md#checker-test-stack-depth-follow-up); checkpoint commits `c278a7e64` and `9467966a9`.

## Boundary and affected artifacts

Classified gap: test-infrastructure stack-depth defect, evidenced by the filed default-stack overflows; measure each cause independently before selecting its fix.
Named test A: `task258b3m2b2b1b1_combined_ownership_hybrids_and_all_family_orders_are_atomic`, in `crates/mizar-checker/src/source_statement/tests/task258b3m2b2b1b1.rs`.
Named test B: `source_reserved_statement_original_capture_preserves_installation_boundary`, in `crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_reserved.rs`.
Affected artifacts: these tests and the production functions proven to cause their stack consumption; this contract pair and the linked backlog entries only for documentation.
Measure recursion and stack-resident data in each test and its production callees; record the cause and chosen mechanism per test in the implementation commit body.
Permitted mechanisms: smaller stack fixtures/frames, heap allocation, recursion restructuring, or splitting combined cases while preserving both named tests and all coverage. Prefer reducing production recursion when it is the cause.
Production changes must preserve all observable behavior; semantics, diagnostics, proof acceptance, and lower-stage ownership remain with their existing owners.
Owner invariants/API: [checker source statements](../../mizar-checker/en/source_statement.md); runner/test design: [consumer runner](../../mizar-test/en/00.crate_plan.md#task-10-consumer-runner-support) and [relevant tests](../../mizar-test/en/00.crate_plan.md#relevant-tests).
No specification, fixture expectation, traceability, coverage-credit, or `spec_coverage_audit.md` change is authorized; language-design questions are deferred to the owner.
Forbidden: stack-size environment/configuration masking; ignoring, deleting, or renaming either named test; weakening assertions or coverage; relaxing gates/lints/baselines, including `doc_volume_baseline.tsv`.
If either test body is moved to a spawned thread with an explicitly sized stack, disclose that mechanism per test in the implementation commit and final response; runner/production stack sizing remains forbidden.
Stop if the final full gate fails, a cause-level fix requires semantic change, or an owner-only decision is needed, alongside protocol stop conditions.

## Test matrix and exit

Case cap: 6. Any sufficiency request beyond it must name the requiring specification clause or `.miz` test.

| Case | Input/check | Expected outcome |
| --- | --- | --- |
| M1 | Before fix, A individually with `RUST_MIN_STACK` unset | Stack-overflow abort confirmed. |
| M2 | Before fix, B individually with `RUST_MIN_STACK` unset | Stack-overflow abort confirmed. |
| M3 | Final tree: `env -u RUST_MIN_STACK cargo test --locked` | Entire suite passes. |
| M4 | Inspect M3 results for A and B | Both pass within that unfiltered run. |
| M5 | Review complete diff, including grep for `RUST_MIN_STACK` and `stack_size` | No forbidden masking, ignored/deleted/renamed tests, weakened assertions, or reduced coverage. |
| M6 | Formatting, Clippy, both lint-policy suites | All pass. |

Required reviews: independent specification/test intent before freeze, then test sufficiency, implementation precision, volume/scope, and source/documentation consistency; protocol quality assessment after gates.
Freeze verification: both `lint_policy` suites and local link/fragment checks; commit the freeze before implementation.
Implementation verification: `cargo fmt` then `cargo fmt --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test -p mizar-checker --test lint_policy`; `cargo test -p mizar-test --test lint_policy`; M3 and M4. Use `CARGO_BUILD_JOBS=4` and no stack-size environment setting.
The filed acceptance is the full-run trigger; full-suite coverage includes affected owner tests and representative end-to-end checks.
Exit: all six cases and required reviews pass; only then check off both backlog entries and commit implementation. Freeze plus implementation count as one changed task; no push.
