# Crate Exit Report: mizar-lexer

> Canonical language: English. Japanese companion: [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the current retrospective `mizar-lexer` milestone, with
deferred parser/resolver integration items documented below.

Quality score: reviewed 91/100.

Score caps applied: none. The remaining `partial` traceability item is
classified as deferred and parser/resolver-owned, not as an unresolved
lexer-owned hard-gate failure. No unresolved `source_undocumented_behavior` or
`test_expectation_drift` item remains in this crate scope.

## Scope

Milestone scope:

- record autonomous crate-development evidence for the completed `mizar-lexer`
  crate;
- document the lexical fixture corpus as the current human-reviewable lexer
  test surface;
- classify design-derived traceability ids and the partial selector-shadowing
  handoff;
- keep English and Japanese design documents synchronized.

Included:

- raw scanning, source preprocessing, reserved tables, import pre-scan, lexical
  environment construction, scope skeletons, final disambiguation, diagnostics
  payloads, and Phase 7 regression/property/fuzz handoff evidence;
- [00.crate_plan.md](./00.crate_plan.md) and this report;
- README index updates for English and Japanese design documents.

Excluded:

- adding new `.miz` tests for parser/frontend-owned complete-source behavior;
- changing `doc/spec/en`, existing lexical fixtures, expectations, or source
  implementation behavior;
- resolving selector-vs-namespace semantics beyond the lexer handoff;
- module resolution, type checking, overload resolution, proof checking, and
  LSP/user-facing diagnostic rendering.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | Lexer-owned behavior traces to `doc/spec/en/02.lexical_structure.md`, `doc/spec/en/11.symbol_management.md`, `doc/spec/en/12.modules_and_namespaces.md`, `doc/spec/en/04.variables_and_constants.md`, and `doc/spec/en/16.theorems_and_proofs.md`; no canonical spec edit is included in this migration. |
| Test contract | Pass with deferred `.miz` rationale | `tests/lexical`, `tests/property`, `tests/fuzz`, and `crates/mizar-lexer` tests cover lexer-owned pre-parser contracts. `.miz` additions are deferred because current lexer tests need sub-file lexical fixtures before a full parser source file is meaningful. |
| Traceability | Pass | `tests/coverage/spec_trace.toml` validates with zero errors. Design-derived ids are classified in [00.crate_plan.md](./00.crate_plan.md) as executable implementation contracts subordinate to the canonical specification. |
| Design/source sync | Pass | Existing module design docs describe the implemented source files; this migration adds the missing crate plan, exit report, and README index entries. |
| Boundary discipline | Pass | [README.md](./README.md) and [00.crate_plan.md](./00.crate_plan.md) exclude file I/O ownership, module resolution, authoritative parsing, name/type/overload/proof semantics, and LSP/user-facing coordinate rendering. |
| Verification | Pass | Current branch verification results are recorded below. `mizar-test plan` exits successfully with four existing planned/no-tests warnings outside `mizar-lexer` scope. |
| Residual risk | Pass | `MLX-GAP-001` and `MLX-GAP-005` are deferred with owners. `MLX-GAP-004` and `MLX-GAP-006` are resolved documentation/metadata drift items, not score-cap-triggering residual source behavior. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 13/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 9/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 91/100 |

The score keeps deductions for the retrospective nature of the evidence, the
lack of lexer-owned `.miz` fixtures, design-derived traceability ids, and the
deferred selector-shadowing handoff.

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MLX-GAP-001 | Lexer-owned tests use `.src` fixtures rather than `.miz` files because several lexer phases operate before a complete parser source file is meaningful. | Parser/frontend milestone | Add `.miz` tests when complete-source lexing/parser integration can assert the same behavior without losing phase isolation. |
| MLX-GAP-005 | `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` remains `partial`. | Parser/resolver integration | Add complete-source tests and downstream implementation for authoritative selector-vs-namespace semantic resolution. |

Resolved during this migration:

- `MLX-GAP-002` and `MLX-GAP-003`: autonomous crate-plan/report evidence and
  README links were added.
- `MLX-GAP-004`: design-derived traceability ids are documented as subordinate
  implementation contracts, not canonical language authority.
- `MLX-GAP-006`: source-loading helper ownership is documented as a lexer
  boundary contract for tests and early integration, not production source
  identity ownership.

## Human Review Surface

The human reviewer should primarily inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- this report
- [README.md](./README.md)
- [raw_lexer.md](./raw_lexer.md)
- [import_prescan.md](./import_prescan.md)
- [lexical_environment.md](./lexical_environment.md)
- [scope_skeleton.md](./scope_skeleton.md)
- [disambiguator.md](./disambiguator.md)
- [test_and_implementation_plan.md](./test_and_implementation_plan.md)
- [todo.md](./todo.md)
- [../../../spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md)
- [../../../spec/en/04.variables_and_constants.md](../../../spec/en/04.variables_and_constants.md)
- [../../../spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md)
- [../../../spec/en/12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md)
- [../../../spec/en/16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md)
- `tests/coverage/spec_trace.toml`
- representative `tests/lexical/**/*.expect.toml` sidecars
- Japanese companions for the same design files

No `.miz` file is part of this migration's changed review surface. Existing
parser `.miz` tests remain parser-owned and unchanged.

## Test Expectation Summary

This migration does not change `.expect.toml` files, snapshots, lexical
fixtures, source code, or benchmark code.

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `tests/lexical/pass/*.expect.toml` | Positive lexer contract fixtures. | Pass | Lexical | Fixture-specific | `tests/coverage/spec_trace.toml` lexical entries |
| `tests/lexical/fail/*.expect.toml` | Negative and recoverable lexer contract fixtures. | Fail | Lexical | Fixture-specific | `tests/coverage/spec_trace.toml` lexical entries |
| `tests/property/*.expect.toml` | Metadata/property anchors for Phase 4/5/7 invariants. | Metadata only | Lexical/property | None unless fixture-specific | `tests/coverage/spec_trace.toml` property entries |
| `tests/fuzz/lexer_phase7_fuzz_handoff_001.expect.toml` | Fuzz handoff anchor for minimized lexer regressions. | Metadata only | Lexical/fuzz | None | Phase 7 and raw span refs |

## Verification

Commands run for this protocol handoff:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

Results:

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test`: passed.
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: passed with 72 test cases, 57 requirements, 0 errors, and 4 warnings for existing planned requirements outside `mizar-lexer`:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- Start parser/frontend integration work that consumes `mizar-lexer` token
  streams and decides whether complete-source `.miz` tests can replace or
  complement selected lexical `.src` fixtures.

Known constraints:

- Do not rebaseline lexical expectations to match implementation behavior
  without a spec-derived or explicitly approved test-intent change.
- Keep design-derived traceability ids subordinate to canonical `doc/spec/en`
  language requirements.
- Keep selector-vs-namespace semantic resolution outside `mizar-lexer`.

Open questions:

- Which parser milestone should first turn
  `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` from
  `partial` into covered complete-source behavior?

Recommended reasoning setting for the next task:

- `high`, because the next useful task crosses lexer/parser/frontend
  boundaries and may need to decide where `.miz` tests become the primary
  review surface. Lower to `medium` for a documentation-only traceability
  cleanup, and raise above `high` if the task changes canonical syntax or
  resolver semantics.
