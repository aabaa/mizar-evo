# Crate Exit Report: mizar-resolve

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for tasks R-001 to R-029 after the R-024 follow-up overlay.
Task R-024 was explicitly deferred as R-G003 `external_dependency_gap` at the
original closeout; the current overlay implements resolver-side consumption of
canonical `mizar-artifact` `ModuleSummary` values without adding resolver-owned
artifact schemas, writers, hash framing, or source loading for artifact-only
dependencies.

Quality score: 94/100.

Score caps applied: none. The read-only quality review reported no
blocking/high/medium findings; its low note was contingent on parent
verification, which passed.

## Scope

Milestone scope: `mizar-resolve` tasks R-001 to R-029.

Included:

- R-001 to R-024 completed and committed task-by-task or by the current
  follow-up implementation overlay.
- R-025 to R-029 completed and committed task-by-task.
- R-023 added the initial active `declaration_symbol` corpus runner seeds and
  traceability metadata.
- R-024 adds canonical summary-backed dependency reuse through `mizar-artifact`.
- R-029 completed the behavior-preserving private module/test split.

Still excluded:

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
| D | R-024 ModuleSummary reuse | Passed by follow-up overlay. The resolver consumes canonical `mizar-artifact` summaries, validates known identity fields, projects exported symbols/labels/lexical/re-export/dependency-interface facts into resolver indexes, and still creates no resolver-owned artifact schema, writer, hash framing, or source-loading path. |
| E | R-025 to R-029 hardening/audits/refactor | Passed. Determinism, public enum policy, source/spec audit, bilingual sync audit, module-boundary refactor, full verification, and quality review are complete. |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Passed | No unclassified blocking/high `spec_gap` remains; R-G001/R-G006/R-G007 are classified and R-G003 is resolved by R-024. |
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
| R-G003 / R-024 | Resolved by the R-024 follow-up overlay. | `mizar-resolve` | Complete: canonical `mizar-artifact` summary consumption is implemented without resolver-local artifact formats. |
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
| R-024 deferral | `cf1084c` `docs: defer module summary reuse gate` |
| R-024 implementation | current follow-up change; commit hash is recorded in the final handoff for this task |
| R-025 | `b433f32` `test: add resolver determinism regression` |
| R-026 | `d1b7e66` `docs: record resolver enum compatibility policy` |
| R-027 | `085be10` `docs: audit resolver source spec correspondence` |
| R-028 | `dcbf2a9` `docs: audit resolver bilingual documentation sync` |
| R-029 | `7011d5a` `refactor: split resolver private modules` |

## Handoff

Recommended next task: start `mizar-test` foundation cleanup task 1.

Reasoning setting: high. Raise to xhigh if the task uncovers a
`repo_metadata_conflict`, a `spec_gap`, or a runner behavior change that would
need language-behavior authority; lower to medium only for a docs-only audit.

Prompt:

```text
Start mizar-test foundation cleanup task 1 following AGENTS.md and
doc/design/autonomous_crate_development.md. Keep doc/spec, existing .miz files,
and existing expectations unchanged unless the task has explicit authority.
Classify and report any spec_gap, repo_metadata_conflict, or language behavior
change instead of repairing it automatically. Use review-only agents for spec,
test, implementation, and source/documentation consistency reviews, run the
relevant verification, and commit exactly this task.
```

## R-031 Step 5 Extension Contract

R-031 is a later independent Step 5 increment and does not rewrite the scored
R-001 through R-029 close-out. It closes only R-G008 for ordinary functor
definitions whose resolver-syntactic namespace, spelling/pattern, definition
argument context, and arity match. All-return-identical groups use appended
`SameSignatureDefinitionConflict` diagnostic and definition metadata plus the
exact `declaration_symbol.signature.same_signature_definition_conflict` runner
key. Mixed/different-return groups retain one existing
`SameSignatureReturnConflict` over every candidate and the existing runner key.
The different-return sidecar stays byte-identical; only the existing
same-return seed and its one trace row may become active/covered.

R-031 close-out requires exact and near-miss unit coverage, recovered-input
suppression, mixed-group priority, permutation-stable first shell/range and
candidate ordering, exact runner keys, paired documentation and coverage-audit
updates, full verification, one R-031 commit, and a clean worktree. It adds no
public numeric diagnostic, semantic type equivalence, overload selection,
parser/checker behavior, Task-49 reconciliation, or Step 6/7 promotion.

R-031 met this contract: all specified resolver and runner tests pass, the
existing same-return source is active with its exact new internal key, the one
trace row is covered, and the different-return sidecar remains unchanged. The
original 94/100 milestone score is not reused as a post-extension score; the
required independent read-only implementation/consistency reviews and full
verification for this extension are recorded in its task handoff.
