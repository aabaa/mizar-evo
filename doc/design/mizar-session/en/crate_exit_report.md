# Crate Exit Report: mizar-session

> Canonical language: English. Japanese companion: [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the current `mizar-session` milestone.

Quality score: reviewed 93/100.

Score caps applied: none. The package-name spelling issue is a deferred
human-owned `spec_gap` outside the `mizar-session` implementation boundary, so
it does not cap the crate score.

## Scope

Milestone scope:

- complete the source identity, build snapshot, source-version, source-map, and
  retention contracts owned by `mizar-session`;
- synchronize English/Japanese component design docs with the implemented Rust
  API and tests;
- add autonomous crate-development evidence for the completed crate.

Included:

- `ids`, `source`, `snapshot`, `source_map`, and `retention` module contracts;
- deterministic snapshot/source-map behavior;
- source-loading error fidelity follow-ups recorded in [todo.md](./todo.md);
- bilingual design synchronization;
- protocol evidence in [00.crate_plan.md](./00.crate_plan.md) and this report.

Excluded:

- resolving package-name spelling across `doc/spec`;
- adding `.miz` language tests for crates that own syntax or semantics;
- scheduling, artifact publication, diagnostics aggregation, cache
  compatibility, IR storage, and proof policy.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass with deferred low-risk item | `MS-GAP-001` in [00.crate_plan.md](./00.crate_plan.md) records the package-name spelling `spec_gap` as human-owned and outside this crate boundary. |
| Test contract | Pass | Rust unit and integration tests cover the crate-owned contracts. `.miz` test-first additions are not applicable because the crate owns no language behavior. |
| Traceability | Pass | [todo.md](./todo.md) tasks 1-32 link module specs, source files, and tests; [00.crate_plan.md](./00.crate_plan.md) summarizes the task decomposition. |
| Design/source sync | Pass | Module design docs document the public API/error surface and README statuses are synchronized with the completed implementation. |
| Boundary discipline | Pass | [README.md](./README.md) and [00.crate_plan.md](./00.crate_plan.md) exclude scheduling, IR storage, diagnostics aggregation, artifact publication, and proof policy. |
| Verification | Pass | Current branch verification results are recorded below. `mizar-test -- plan` exits successfully with four existing planned/no-tests warnings outside `mizar-session` scope. |
| Residual risk | Pass with deferred item | The package-name spelling `spec_gap` is deferred to language/package specification work; no `mizar-session` behavior depends on choosing between the spellings today. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 19/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 4/5 |
| Total | 93/100 |

The reviewed score keeps small deductions for the deferred package-name spelling
`spec_gap` and the retrospective nature of the crate plan.

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MS-GAP-001 | `doc/spec/en/23.package_management_and_build_system.md` uses `[a-z][a-z0-9-]*`, while `doc/spec/en/12.modules_and_namespaces.md` uses `snake_case` package names. | Human language/package spec owner | Align English canonical package-name rules, sync Japanese companions, then tighten upstream build-plan validation if needed. |

## Human Review Surface

The human reviewer should primarily inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- this report
- [README.md](./README.md)
- [todo.md](./todo.md)
- [source.md](./source.md), especially source-loading error boundaries
- [snapshot.md](./snapshot.md), especially source identity validation boundaries
- Japanese companions for the same files

No `doc/spec` or `.miz` file is part of this migration's human review surface
because the task does not change language behavior or test intent.

## Test Expectation Summary

No `.expect.toml` or snapshot expectation was changed by this migration.

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| N/A | `mizar-session` owns Rust infrastructure contracts rather than `.miz` execution behavior. | N/A | N/A | N/A | N/A |

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
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: passed with 72 test cases, 57 requirements, 0 errors, and 4 warnings for existing planned requirements without tests:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- Resolve `MS-GAP-001` by aligning package-name spelling in the English canonical
  specs and Japanese companions, then decide whether upstream build-plan
  validation should accept hyphenated names, snake_case names, or both with
  normalization.

Known constraints:

- Do not change `mizar-session` package-id validation to decide the spelling gap
  before the spec is aligned.
- Keep `mizar-session` decoupled from lexer/parser semantics; frontend owns
  lexer-span to session-coordinate bridging.

Open questions:

- Which package-name spelling should be canonical for registry packages and
  import paths?

Recommended reasoning setting for the next task:

- `high`, because resolving `MS-GAP-001` touches canonical language/package
  specification, Japanese synchronization, and downstream validation policy.
  Lower to `medium` only if the next task is limited to gathering examples and
  making no spec edit.
