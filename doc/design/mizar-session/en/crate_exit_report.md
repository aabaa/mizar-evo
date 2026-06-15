# Crate Exit Report: mizar-session

> Canonical language: English. Japanese companion: [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the current `mizar-session` milestone.

Quality score: reviewed 95/100.

Score caps applied: none. The package-name spelling `spec_gap` recorded as
`MS-GAP-001` has been resolved in the canonical package/module specifications
and remains outside the `mizar-session` implementation boundary.

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

- enforcing package-name spelling inside `mizar-session`;
- adding `.miz` language tests for crates that own syntax or semantics;
- scheduling, artifact publication, diagnostics aggregation, cache
  compatibility, IR storage, and proof policy.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | `MS-GAP-001` in [00.crate_plan.md](./00.crate_plan.md) records the package-name spelling `spec_gap` as resolved by aligning the canonical package/module specs and Japanese companions. |
| Test contract | Pass | Rust unit and integration tests cover the crate-owned contracts. `.miz` test-first additions are not applicable because the crate owns no language behavior. |
| Traceability | Pass | [todo.md](./todo.md) tasks 1-32 link module specs, source files, and tests; [00.crate_plan.md](./00.crate_plan.md) summarizes the task decomposition. |
| Design/source sync | Pass | Module design docs document the public API/error surface and README statuses are synchronized with the completed implementation. |
| Boundary discipline | Pass | [README.md](./README.md) and [00.crate_plan.md](./00.crate_plan.md) exclude scheduling, IR storage, diagnostics aggregation, artifact publication, and proof policy. |
| Verification | Pass | Current branch verification results are recorded below. `mizar-test -- plan` exits successfully with four existing planned/no-tests warnings outside `mizar-session` scope. |
| Residual risk | Pass | Package-name spelling enforcement remains an upstream build-plan concern; no `mizar-session` behavior depends on performing that validation locally. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 20/20 |
| Test contract and coverage | 19/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 4/5 |
| Total | 95/100 |

The reviewed score keeps small deductions for the retrospective nature of the
crate plan.

## Deferred Items

None. `MS-GAP-001` is resolved: package names are lowercase `snake_case`
(`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`) in both the English canonical specs and the
Japanese companions, and hyphen normalization is not defined.

## Human Review Surface

The human reviewer should primarily inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- this report
- [README.md](./README.md)
- [todo.md](./todo.md)
- [source.md](./source.md), especially source-loading error boundaries
- [snapshot.md](./snapshot.md), especially source identity validation boundaries
- [../../../spec/en/12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md)
- [../../../spec/en/23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md)
- Japanese companions for the same files

No `.miz` file is part of this migration's human review surface because the
task changes package naming specification text and downstream validation policy,
not executable language test intent.

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

- Tighten upstream build-plan validation so package ids supplied to
  `mizar-session` already satisfy the aligned package-name spelling:
  lowercase `snake_case` (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`) with no hyphen
  normalization.

Known constraints:

- Do not move package-name spelling enforcement into `mizar-session`; keep it in
  the upstream build-plan layer.
- Keep `mizar-session` decoupled from lexer/parser semantics; frontend owns
  lexer-span to session-coordinate bridging.

Open questions:

- None for `MS-GAP-001`; the canonical spelling is lowercase `snake_case`.

Recommended reasoning setting for the next task:

- `medium`, because the remaining work should be a bounded validator follow-up
  against already-aligned specification text. Raise to `high` if that task also
  changes parser, resolver, or package manifest semantics.
