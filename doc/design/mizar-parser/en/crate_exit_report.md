# Crate Exit Report: mizar-parser

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the post-Task-46 `mizar-parser` milestone.

Quality score: independently reviewed 99/100.

Score caps applied: none at report preparation. All hard gates below pass.
P-265-47D is nonblocking and human-owned. The separately classified frontend
string-position heuristic is outside the parser target and receives no parser
credit or closure claim.

`PARSER-CRATE-POST-TASK46-CLOSEOUT` closes only the parser crate milestone. It
does not close global Step 5, infer or promote parser Task 49, or authorize
Steps 6/7.

## Scope

Milestone scope:

- parser Tasks 1-48;
- frontend-adapted token input, source-shaped `SurfaceAst` output, concrete
  grammar and Pratt parsing, syntax recovery, deterministic behavior, and the
  active parse-only corpus;
- source/spec correspondence, reserved-word coverage, bilingual
  synchronization, public-enum policy, and parser module-boundary policy;
- the current parser, syntax, corpus, traceability, and count/hash evidence.

Included:

- all completed parser grammar surfaces, including the three canonical
  `reconsider_tail` forms, top-level `PropertyImplementation`, and exact
  infix/prefix/postfix `OperatorDeclaration` forms;
- annotated and visible top-level plus definition-local operator-declaration
  placement, append-only syntax kind 193, and local recovery;
- parser unit/determinism/lint-policy, syntax, frontend, and real parse-only
  runner evidence;
- paired English/Japanese parser design documents and this report.

Excluded:

- the nonblocking human-owned P-265-47D wording `spec_gap` between Chapter 8
  and Chapters 4/15 plus Appendix A for the `reconsider` item list;
- the independently classified frontend overbroad string-position heuristic,
  which remains an external frontend `source_drift` /
  `source_undocumented_behavior` with unit `test_expectation_drift`;
- operator activation, active-functor validation, overload meaning,
  resolution, semantic precedence-range validation, and Pratt metadata
  mutation from source declarations;
- checker/proof/Core/CFG/VC/artifact behavior, global Step-5 completion,
  unapproved future grammar growth, Task 49, and Steps 6/7.

## Disagreement Classification

- `design_drift`: stale pre-Task-46 live-status text and the obsolete
  review-derived coverage backlog were closed by this paired documentation
  synchronization.
- `source_drift` / `test_gap`: the aliased parser P-043-01/P-046 operator-
  declaration gap was closed by Task 46 before this closeout.
- `spec_gap`: only P-265-47D remains in parser inventory; it is nonblocking and
  human-owned.
- `source_drift` / `source_undocumented_behavior` /
  `test_expectation_drift`: the overbroad string-position heuristic remains an
  external frontend finding and is not parser credit.
- No parser-scope `test_gap`, `boundary_violation`, or
  `repo_metadata_conflict` remains. No metadata repair is attempted.

## Hard Gates

| Protocol gate | Status | Evidence |
|---|---|---|
| 1. No blocking/high specification inconsistency | Pass | Canonical English specification, Appendix-A parser normalization, active corpus, exact trace rows, and parser design agree for Tasks 1-48. P-265-47D is nonblocking and human-owned. |
| 2. No undocumented parser source behavior | Pass | [source_spec_audit.md](./source_spec_audit.md) traces parser public surfaces and implementation behavior to specification/tests. No parser-scope `source_undocumented_behavior` remains; the known frontend heuristic is explicitly external and uncredited. |
| 3. Milestone-owned coverage or explicit deferral | Pass | Parse coverage is 44/44. All implemented parser grammar slices have active coverage or documented owner boundaries; no parser-owned deferred grammar task remains. |
| 4. Expectation integrity | Pass | Task 46 added only new pass/fail sources and sidecars. Existing `.miz` and expectations were unchanged. The earlier Task-47 sidecar repair moved toward canonical grammar rather than current implementation. |
| 5. Design/source synchronization | Pass | Paired parser/syntax/mizar-test documents, crate/global indexes, source, wrapper ownership, recovery, and current oracles agree through Task 46. |
| 6. Responsibility boundaries | Pass | Parsing remains syntax-only. No resolver, checker, proof, Core/CFG/VC, artifact, cache, or build responsibility moved into `mizar-parser`. |
| 7. Coverage-audit synchronization | Pass | Task 46 already updated `doc/design/spec_coverage_audit.md` for its coverage change. This closeout changes no specification coverage, trace status, owner, or deferred rationale, so the audit remains unchanged. |
| 8. Verification | Pass | Focused/relevant crate tests, format, denied-warning workspace Clippy, full workspace tests, five CLIs, count/hash manifests, and diff checks pass. |
| 9. Residual-risk classification | Pass | P-265-47D is human-owned/nonblocking; the frontend heuristic is external and uncredited. No parser `boundary_violation` or `repo_metadata_conflict` is present. |

## Score Breakdown

The independent read-only reviewer confirmed every hard gate. The single-point
deduction is for the nonblocking, human-owned P-265-47D wording gap.

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 20/20 |
| Traceability | 15/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 5/5 |
| Total | 99/100 |

## Remaining Items

| Item | Classification and reason | Owner / action |
|---|---|---|
| P-265-47D | Nonblocking human-owned `spec_gap`: Chapter 8 writes one `reconsider_item`, while Chapters 4/15 and Appendix A specify a list. | Human specification owner must explicitly reconcile English canonical wording; do not infer a spec edit from parser behavior. |
| Frontend string-position heuristic | External frontend `source_drift` / `source_undocumented_behavior` with unit `test_expectation_drift`: quoted text is accepted after broader punctuation than the canonical operator-declaration/string-annotation positions. | Fresh frontend authority must define a separate task. Do not fold it into parser closeout or invent an identifier. |

No nonempty successor `mizar-parser` implementation task is authorized by the
current inventory.

## Human Review Surface

- [00.crate_plan.md](./00.crate_plan.md), [todo.md](./todo.md), and this report;
- [grammar.md](./grammar.md), [pratt.md](./pratt.md),
  [recovery.md](./recovery.md), and
  [source_spec_audit.md](./source_spec_audit.md);
- paired English/Japanese parser/syntax/mizar-test Task-46 addenda;
- `doc/spec/en/10.functors.md`, `12.modules_and_namespaces.md`, Appendix A,
  and the exact Task-46 trace row and pass/fail sidecars;
- `crates/mizar-parser/src/`, the paired `mizar-syntax` kind/accessor support,
  and `crates/mizar-test/tests/metadata.rs`.

This docs-only closeout changes no specification, source, test, `.miz`,
expectation, snapshot, traceability row, coverage mapping, or oracle.

## Test Expectation Summary

| Evidence group | Intent | Expected phase/outcome |
|---|---|---|
| Parser unit/determinism/lint-policy and syntax tests | Guard AST ownership, exact slots, recovery boundaries, append-only kind identity, deterministic output, and public policy. | Rust tests pass; no closeout-time expectation changes. |
| Task-47 `reconsider` corpus | Cover omitted, explicit-`by`, and proof-block tails under canonical grammar. | Active parse-only contracts pass. |
| Task-48 property-implementation pass/fail pair | Cover exact top-level placement, means/equals bodies, correctness ordering, and bounded recovery. | Active parse-only contracts pass. |
| Task-46 operator-declaration pass/fail pair | Cover exact infix/prefix/postfix forms, associativity words, annotations/visibility/local placement, malformed slots/delimiters, and following-item preservation. | Pass sidecar has no diagnostics; fail sidecar matches six existing syntax codes. |

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

- parser: 225 unit / 3 determinism / 14 lint-policy tests; syntax: 55 unit / 8
  lint-policy tests; frontend and mizar-test relevant suites pass;
- plan: 409 cases / 370 requirements, pass/fail 223/186, warnings/errors 23/0;
  parse/declaration/type/proof coverage is 44/44, 10/5, 236/224, and 4/1;
- active parse/declaration/type/proof admissions are 101/5/188/1, all passing;
- plan/parse/declaration/type/proof stdout hashes are
  `9b1e3058bde355163b1153339250647633beef9920456615cf6661c4140a93cf`,
  `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
  `210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`,
  `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625`,
  and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`;
- raw/normalized 276-test-list hashes are
  `967495e78e1068f592e64834ea3ffb9eac9c25692ea5cbd4f11006a679c66590`
  and `1be4ae09188b27a40814adc6597de4806dabb13bcac019b294154e1455072adf`;
- `mizar-test` production is 18 paths / 20,088 lines with path/content hashes
  `63e4e770b0d10872415548410d417071c1901f3ffa5aea964a81d2dbbc572ed0`
  and `7e5adca22db2b73f94f04c406f10788f2cd49ba48109bb105a3fd076c339d560`;
- parser production is 12 paths / 38,940 lines with path/content hashes
  `192f9d0b5e6534c4daab010ec51a9356e9e0fd6fb86876bd2600a75844e7566a`
  and `6f27be7c5689cc12b6cf684736bc44b1f92acebf6ce313ce581b22a46451cb5b`;
- format, denied-warning Clippy, full workspace tests, and diff checks pass.

The 23 warnings are existing cross-workspace soundness/corpus-size warnings,
not parser-closeout errors; the plan reports zero errors.

## Handoff

Next `mizar-parser` work: none under current authority. Begin the next turn with
a fresh canonical Step-5 inventory. Do not infer Task 49 or promote Steps 6/7.
The external frontend string-position finding requires its own authority and
must not be treated as a parser follow-up by implication.

Recommended reasoning setting: `xhigh` for the next cross-crate authority
inventory or canonical grammar/semantic decision; `high` for a bounded,
already-authorized implementation; `medium` for pure documentation.
