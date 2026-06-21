# Crate Exit Report: mizar-resolve

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for non-deferred tasks R-001 to R-029. Task R-024 remains
explicitly deferred as R-G003 `external_dependency_gap`.

Quality score: 94/100.

Score caps applied: none. The read-only quality review reported no
blocking/high/medium findings; its low note was contingent on parent
verification, which passed.

## Scope

Milestone scope: `mizar-resolve` tasks R-001 to R-029.

Included:

- R-001 to R-023 completed and committed task-by-task.
- R-025 to R-029 completed and committed task-by-task.
- R-023 added the initial active `declaration_symbol` corpus runner seeds and
  traceability metadata.
- R-029 completed the behavior-preserving private module/test split.

Excluded:

- R-024 summary-backed `ModuleSummary` reuse is deferred until
  `mizar-artifact` task 5 supplies the canonical schema, writer, validating
  reader, and version compatibility policy.
- Public resolver diagnostic code allocation remains R-G001 `spec_gap`; current
  resolver diagnostics stay crate-local/internal.
- Broader semantic `.miz` assertions for import/name/dot-chain/label facts
  remain R-G007 `test_gap`.

## Milestone Gates

| Milestone | Scope | Decision |
|---|---|---|
| A | R-001 to R-007 foundation/module-index seam | Passed. Crate scaffold, `ResolvedAst`, `SymbolEnv`, deterministic snapshots, and resolver-side module-index seam are committed. |
| B | R-008 to R-016 imports/names | Passed. Import graph/path resolution, declaration shells, namespace/name lookup, internal diagnostics, and dot-chain finalization are committed; public diagnostic codes remain deferred under R-G001. |
| C | R-017 to R-023 labels/symbols/corpus runner | Passed. Label resolution, signature collection, recovered syntax policy, and active `declaration_symbol` runner seeds are committed. |
| D | R-024 ModuleSummary reuse | Deferred as R-G003 `external_dependency_gap`; no resolver-owned artifact schema, reader, writer, or shim was created. |
| E | R-025 to R-029 hardening/audits/refactor | Passed. Determinism, public enum policy, source/spec audit, bilingual sync audit, module-boundary refactor, full verification, and quality review are complete. |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Passed | No unclassified blocking/high `spec_gap` remains; R-G001/R-G003/R-G006/R-G007 are classified. |
| Test contract | Passed | Existing expectations were not rebaselined. New `.miz` tests are limited to R-023 spec-derived `declaration_symbol` seeds. |
| Traceability | Passed | R-023 fixtures have expectation sidecars and `tests/coverage/spec_trace.toml` entries. |
| Design/source sync | Passed | `source_spec_correspondence.md`, `bilingual_documentation_synchronization.md`, and `module_boundary_refactor.md` are synchronized. |
| Boundary discipline | Passed | Resolver does not own parser/syntax/frontend/session/build/checker/proof/driver/artifact responsibilities. |
| Verification | Passed | Full workspace tests, full clippy, formatting, and `mizar-test plan` completed. |
| Residual risk | Passed | Residual items are deferred, external dependency, or future test-growth records. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| R-G001 | Public resolver diagnostic code range is absent from `doc/spec/en` chapter 22. | spec / diagnostics planning | Assign public resolver diagnostic ownership before user-facing resolver diagnostic integration. |
| R-G003 / R-024 | Artifact-backed `ModuleSummary` reuse depends on missing `mizar-artifact` task 5. | `mizar-artifact` | Complete tasks 1 to 5 with canonical `ModuleSummary` schema, writer, validating reader, and version compatibility policy. |
| R-G006 | Parser/syntax does not expose a module-level scheme/template declaration source role. | `mizar-parser` / `mizar-syntax` | Expose the owning source role; resolver must not fabricate module-level scheme/template symbols before then. |
| R-G007 | Broader active semantic `.miz` assertions are not yet implemented for import/name/dot-chain/label facts. | future `mizar-test` / resolver corpus work | Extend runner assertions from `doc/spec/en` without inventing behavior or rebaselining existing tests. |

## Human Review Surface

Primary human-review artifacts added or changed during `mizar-resolve`:

- `tests/miz/pass/resolve/pass_resolve_declaration_symbol_smoke_001.miz`
- `tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz`

No `doc/spec/en` or `doc/spec/ja` files changed. Existing `.miz` tests and
existing expectations were not rebaselined to match implementation.

Derived artifacts maintained by Codex:

- `doc/design/mizar-resolve/en|ja/*.md`
- `crates/mizar-resolve/**`
- new expectation sidecars and `tests/coverage/spec_trace.toml` entries for
  the R-023 active declaration-symbol seeds.

## Test Expectation Summary

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `tests/miz/pass/resolve/pass_resolve_declaration_symbol_smoke_001.miz` | Parser-backed declaration shells, visibility-bearing declarations, and theorem/lemma declarations reach symbol collection. | pass | resolve | none | `spec.en.11.symbol_management.signatures`, `spec.en.11.symbol_management.visibility`, `spec.en.12.modules.visibility.semantic`, `spec.en.16.theorems_and_proofs.labels.declaration_symbols` |
| `tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz` | Same-scope duplicate theorem labels are rejected at declaration-symbol resolution before proof checking. | fail | resolve | internal detail key `declaration_symbol.symbol.duplicate_declaration`; public diagnostic codes remain empty | `spec.en.16.theorems_and_proofs.labels.same_scope_uniqueness` |

## Verification

Commands run for close-out:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

Results:

- `cargo fmt --check`: passed.
- `cargo test`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `mizar-test plan`: passed with 0 errors, 4 warnings for already planned
  requirements without tests:
  `spec.en.algorithm.vc.assignment_loop_exits`,
  `spec.en.binding.substitution.capture_avoidance`,
  `spec.en.elaboration.choice_comprehension.lowering`,
  `spec.en.type_soundness.escape_and_guard_failures`.

## Task Commits

| Task | Commit |
|---|---|
| R-001 | `8192219` `feat: scaffold mizar-resolve crate` |
| R-002 | `3bfb0e6` `docs: specify resolved ast shape` |
| R-003 | `de157b7` `docs: specify symbol environment shape` |
| R-004 | `7e9d40d` `feat: add resolved ast data shapes` |
| R-005 | `b8da8fe` `feat: add symbol env data shapes` |
| R-006 | `c9eef80` `feat: add resolver debug snapshots` |
| R-007 | `c069ab8` `feat: add resolver module-index seam` |
| R-008 | `c0d9224` `docs: specify resolver import resolution` |
| R-009 | `1c01bca` `feat: add resolver import graph` |
| R-010 | `03fa162` `feat: resolve import path candidates` |
| R-011 | `e3dd505` `feat: collect declaration shells` |
| R-012 | `3ab02b9` `docs: specify resolver name resolution` |
| R-013 | `178aba3` `feat: resolve namespace paths` |
| R-014 | `9ae672e` `feat: resolve symbol name references` |
| R-015 | `bad8964` `feat: add internal name diagnostics` |
| R-016 | `98749bf` `feat: finalize resolver dot chains` |
| R-017 | `89b85a7` `docs: specify resolver label resolution` |
| R-018 | `cadd158` `feat: resolve theorem and proof labels` |
| R-019 | `9de66c7` `docs: specify resolver signature collection` |
| R-020 | `ed24976` `feat: add symbol collection skeleton` |
| R-021 | `363d55b` `feat: extract parser-backed signatures` |
| R-022 | `4892e5e` `feat: handle resolver recovered syntax` |
| R-023 | `0e0ee9a` `feat: add declaration-symbol corpus runner` |
| R-024 | `cf1084c` `docs: defer module summary reuse gate` |
| R-025 | `b433f32` `test: add resolver determinism regression` |
| R-026 | `d1b7e66` `docs: record resolver enum compatibility policy` |
| R-027 | `085be10` `docs: audit resolver source spec correspondence` |
| R-028 | `dcbf2a9` `docs: audit resolver bilingual documentation sync` |
| R-029 | `7011d5a` `refactor: split resolver private modules` |

## Handoff

Recommended next crate: `mizar-artifact` wave A, tasks 1 to 5.

Reasoning setting: high. Raise to xhigh only if the artifact schema appears to
conflict with `doc/spec/en`, architecture 11/18, or resolver `ModuleSummary`
expectations; lower to medium for documentation-only subtasks.

Prompt:

```text
mizar-artifact の自律 crate 開発を、doc/design/mizar-artifact/ja/todo.md の
task 1〜5（wave A）から進めてください。AGENTS.md と
doc/design/autonomous_crate_development.md に従い、英語正本と日本語 companion を
同期し、task-by-task で review-only Agent、検証、コミットまで完了してください。

目的は、mizar-resolve R-024 を解除できる canonical `ModuleSummary`
schema / writer / validating reader / version compatibility policy を
`mizar-artifact` 側で所有することです。resolver-local artifact schema や shim は
作らず、producer crates が `mizar-artifact` に依存する方向を守ってください。
```
