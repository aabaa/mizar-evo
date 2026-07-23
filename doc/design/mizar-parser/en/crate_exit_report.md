# Crate Exit Report: mizar-parser

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the current `mizar-parser` crate milestone.

Quality score: independently reviewed 94/100 after the closeout documentation
was synchronized.

Score caps applied: none. Before this report existed, the missing paired exit
report and stale status indexes were `design_drift`, hard gate 5 failed, and any
score was invalid and capped at 89. `PARSER-CRATE-CLOSEOUT` closes that drift.

This result closes only the parser crate milestone. It does not close global
Step 5, promote parser Task 46, create or promote Task 49, or authorize Steps 6
or 7.

## Scope

Milestone scope:

- parser Tasks 1-45 and 47-48;
- frontend-adapted token input, source-shaped `SurfaceAst` output, grammar and
  Pratt parsing, syntax recovery, deterministic parser behavior, and active
  parse-only corpus execution;
- source/spec correspondence, reserved-word coverage, bilingual
  synchronization, public-enum policy, and the Task-47/48 increments;
- the current parser, syntax, corpus, traceability, and count/hash evidence
  recorded below.

Included:

- public parser transfer objects and the syntax-only parser entry point;
- grammar surfaces represented by completed parser tasks, including the three
  canonical `reconsider_tail` forms and top-level `PropertyImplementation`;
- parser unit, determinism, lint-policy, syntax, frontend, and real parse-only
  runner evidence;
- paired English/Japanese parser design documents and this exit report.

Excluded:

- the aliased P-043-01 / P-046 concrete operator-declaration gap, deferred
  until its named frontend string-required context exists;
- the nonblocking, human-owned P-265-47D wording `spec_gap` between Chapter 8
  and the Chapters 4/15 plus Appendix-A `reconsider` list form;
- resolver, checker, semantic property/coherence decisions, proof acceptance,
  Core/CFG/VC, artifacts, or global Step-5 completion;
- future grammar growth without an authority-approved task, and Steps 6/7.

Current-milestone completion does not claim that every conceivable or future
canonical grammar production is implemented. New grammar work must enter
through fresh canonical authority.

## Hard Gates

| Protocol gate | Status | Evidence |
|---|---|---|
| 1. No blocking/high specification inconsistency | Pass | Canonical English specification, active `.miz` corpus, exact trace rows, and parser plan agree for completed Tasks 1-45 and 47-48. P-265-47D is explicitly nonblocking and human-owned. |
| 2. No undocumented source behavior | Pass | [source_spec_audit.md](./source_spec_audit.md) traces public surfaces and promised parser behavior to source and tests. No unresolved `source_undocumented_behavior` remains. |
| 3. Milestone-owned coverage or explicit deferral | Pass | All 43 parse-only requirements are covered. P-043-01/P-046 has one explicit external trigger, owner, and deferred rationale. |
| 4. Expectation integrity | Pass | Task 48 changed no existing `.miz` or expectation file. Task 47 changed one stale sidecar diagnostic only toward canonical grammar; no implementation-derived rebaseline remains. |
| 5. Design/source synchronization | Pass | Paired parser documents, crate/top indexes, Task-48 completion records, and this paired report agree with source and current oracles. |
| 6. Responsibility boundaries | Pass | Parsing remains syntax-only; no resolver, checker, proof, Core/CFG/VC, artifact, cache, or build responsibility moved into `mizar-parser`. |
| 7. Coverage-audit synchronization | Pass | Task 48 updated `doc/design/spec_coverage_audit.md` for its coverage change. This docs-only closeout changes no coverage mapping, trace status, owner, follow-up ownership, or deferred rationale, so no further audit edit is required. |
| 8. Verification | Pass | Focused crate tests, format, denied-warning workspace Clippy, full workspace tests, five CLIs, count/hash manifests, and diff checks pass. |
| 9. Residual-risk classification | Pass | Remaining parser items are deferred P-043-01/P-046 and human-owned P-265-47D. No `boundary_violation` or `repo_metadata_conflict` is present. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

The deductions reflect the deferred operator-declaration surface, the
human-owned `reconsider` wording gap, and the absence of authority for a next
parser task. They are not hard-gate failures.

## Deferred Items

| ID | Classification and reason | Owner | Unblock condition |
|---|---|---|---|
| P-043-01 / P-046 | `source_drift` / `test_gap`: concrete operator declarations and their reserved-word corpus are outside the current milestone. Both IDs name the same gap. | Deferred `mizar-parser` Task 46 | A future `mizar-frontend` string-required operator-declaration context is canonically authorized and brings Task 46 into scope. |
| P-265-47D | Nonblocking human-owned `spec_gap`: Chapter 8 writes one `reconsider_item`, while Chapters 4/15 and Appendix A specify a list. | Human specification owner | Explicitly reconcile the English canonical wording; do not infer a spec edit from current parser behavior. |

## Human Review Surface

The primary review surface is:

- [00.crate_plan.md](./00.crate_plan.md), [todo.md](./todo.md), and this report;
- [grammar.md](./grammar.md), [pratt.md](./pratt.md),
  [recovery.md](./recovery.md), and
  [source_spec_audit.md](./source_spec_audit.md);
- paired English/Japanese documents;
- `doc/spec/en/`, active parser `.miz` files and sidecars referenced by
  `tests/coverage/spec_trace.toml`, and the exact Task-47/48 corpus files;
- `crates/mizar-parser/src/`, paired `mizar-syntax` vocabulary, parser
  determinism/lint tests, and `crates/mizar-test/tests/metadata.rs`.

No source, specification, `.miz`, expectation, snapshot, or traceability file
is changed by `PARSER-CRATE-CLOSEOUT`.

## Test Expectation Summary

| Evidence group | Intent | Expected phase/outcome | Specification surface |
|---|---|---|---|
| Parser unit, determinism, and lint-policy tests | Guard AST ownership, recovery boundaries, deterministic output, and public policy. | Rust tests pass; no expectation file changes. | Completed parser task contracts and paired design docs. |
| `tests/miz/pass/parser/pass_parser_reconsider_tails_001.miz` and `tests/miz/pass/parser/pass_parser_reconsider_tails_001.expect.toml` | Task 47 additive coverage for omitted and proof-block tails without semantic acceptance. | `parse_only` pass; `diagnostic_codes = []`. | Chapters 4, 8, 15; Appendix A. |
| `tests/miz/fail/parser/fail_parser_consider_reconsider_recovery_001.miz` and `tests/miz/fail/parser/fail_parser_consider_reconsider_recovery_001.expect.toml` | Task 47 kept the `.miz` byte-identical and removed only the obsolete `malformed_justification` for canonical `reconsider x as set;`; all other recovery diagnostics and intent remain. | Existing fail sidecar remains a passing parse-only recovery contract. | Chapter 15 parser syntax and P-265-47B. |
| `tests/miz/pass/parser/pass_parser_property_implementations_001.miz` and `tests/miz/pass/parser/pass_parser_property_implementations_001.expect.toml` | Task 48 additive means/equals property coverage. | `parse_only` pass; `diagnostic_codes = []`. | Chapters 7 and 12; Appendix A. |
| `tests/miz/fail/parser/fail_parser_property_implementations_recovery_001.miz` and `tests/miz/fail/parser/fail_parser_property_implementations_recovery_001.expect.toml` | Task 48 additive malformed-parameter/body/correctness and following-item recovery coverage. No existing expectation was rebaselined. | `parse_only` fail contract with the exact 13 recorded diagnostics. | Chapters 7 and 12; Appendix A. |

## Verification

Commands:

```text
cargo test -p mizar-parser
cargo test -p mizar-syntax
cargo test -p mizar-frontend
cargo test -p mizar-test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- parse-only --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- declaration-symbol --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- type-elaboration --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -p mizar-test -- proof-verification --tests-root tests --manifest tests/coverage/spec_trace.toml
git diff --check
```

Results:

- parser: 221 unit, 3 determinism, and 14 lint-policy tests pass; syntax: 54
  unit and 8 lint-policy tests pass;
- plan: 407 cases / 369 requirements, pass/fail 222/185, warnings/errors 23/0;
  parse/declaration/type/proof coverage is 43/43, 10/5, 236/224, and 4/1;
- active parse/declaration/type/proof admissions are 99/5/188/1, all passing;
- plan/parse/declaration/type/proof stdout hashes are
  `2957a40b91a4cf64206301b4bf91d1c42ecdac2a564b70af370d2e52333ab57b`,
  `c9dcbcef79e727f31720d46532febe5a20e02a7710cf691e49d89fcfb69bccfa`,
  `210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`,
  `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625`,
  and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`;
- raw/normalized 276-test-list hashes are
  `967495e78e1068f592e64834ea3ffb9eac9c25692ea5cbd4f11006a679c66590`
  and `1be4ae09188b27a40814adc6597de4806dabb13bcac019b294154e1455072adf`;
- `mizar-test` production is 18 paths / 20,088 lines with path/content hashes
  `63e4e770b0d10872415548410d417071c1901f3ffa5aea964a81d2dbbc572ed0`
  and `7e5adca22db2b73f94f04c406f10788f2cd49ba48109bb105a3fd076c339d560`;
- parser production is 12 paths / 38,256 lines with path/content hashes
  `192f9d0b5e6534c4daab010ec51a9356e9e0fd6fb86876bd2600a75844e7566a`
  and `3728e0ac374c11b3ef0553379d2e9affcd861513e004dfee80589b47bcf2130a`;
- format, Clippy, crate/workspace tests, and diff checks pass.

The 23 plan warnings are existing cross-workspace soundness/corpus-size
warnings, not parser-closeout errors; the plan reports zero errors.

## Handoff

Next recommended work: none in `mizar-parser` under current authority. Fresh
inventory is required before any future parser task. P-043-01/P-046 may be
re-entered only when its named frontend context is authorized.

Known constraints: global Step 5 remains active outside this parser milestone;
do not infer Task 49 or promote Steps 6/7; keep parser work syntax-only and
preserve the authority order and bilingual synchronization.

Recommended reasoning setting: `xhigh` for the next cross-crate authority
inventory or canonical grammar/semantic decision; `high` for a bounded,
already-authorized parser-only implementation; `medium` for pure documentation.
