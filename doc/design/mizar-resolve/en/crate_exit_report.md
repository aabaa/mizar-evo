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
- Completed R-032A/R-032B structural-map/proof-label projection and the current
  Checker Task 258B5C active confinement cases are post-close-out logical
  tasks.

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

## Planned R-032A / R-032B / Checker Task 258B5C Lower Extension

R-032 is a post-close-out prerequisite umbrella and does not reopen or rescore
the completed R-001 through R-029 milestone. Canonical Chapter 15 §15.10 and
Chapter 16 §§16.4.2/16.5.1 authorize proof-label confinement. The core
`LabelResolver` prefix behavior is correct; the missing normal-source
`SurfaceAst` proof-step declaration/simple-unqualified reference collector is
Medium `source_drift`, while the older R-023 full-source-walk attribution is
`design_drift`. At this frozen extension record, missing active B5C coverage
was R-G007 `test_gap`; absent public codes remain the Low deferred R-G001
`spec_gap`.

The pre-S-026 record used the four-step order documentation, R-032A, R-032B,
and active B5C, each a separate commit. The later S-026 dependency overlay
plus the lint-policy corrections supersede that execution order with S-026
docs, S-026 implementation, R-032A lint-policy docs correction, R-032A
implementation, R-032B lint-policy docs correction, R-032B implementation,
and active B5C. R-032A owns exactly `resolved_ast.rs`,
`resolved_ast/tests.rs`, and the sole `tests/lint_policy.rs` R-026
owning-spec entry for `SurfaceResolvedArenaError`; it implements the exact validated
`SurfaceResolvedArena` plus named public non-exhaustive error table from
`resolved_ast.md`, including state/reference-key mismatch payloads. R-032B owns
exactly `labels.rs`, `labels/tests.rs`, and the sole
`tests/lint_policy.rs` R-026 owning-spec decision for
`ProofLabelSourceCollectionError` / `labels.md`, and
implements the exact validated collector, public error table, subtree,
completion, scope, and provenance contract from `labels.md`.

Top-level theorem roots are `[0]`, `[1]`, ... in stable module source order;
nested supported proof scopes are owner-relative. Labels become visible after
their whole labelled compact subtree, including own proof, completes.
One module-global one-based statement counter is never reset per theorem, and
the exact length-framed `proof-step-v1` grammar owns identity. R-032B's exact
`'a` impl stores only AST/arena borrows, owns namespace/contribution, validates
but does not store module, and later uses `resolved.module()`. It uses only the
R-032A map and has no callback, unmapped side channel,
fabricated id, unchecked conversion, or panic. R-032A arena origin
`[surface_id]` and R-032B richer table origins are intentionally distinct.
The R-032B Surface walk is the exhaustive default-deny direct-edge table in
`labels.md`; its upper chain is exactly `Root` -> `CompilationUnit` ->
`ItemList` -> direct theorem, and every unlisted or invalid edge yields no
row/ordinal/descent.
Tests cover the exact named-error/mismatch/overflow matrix, inner/sibling and
correctly directed earlier-theorem-to-later-theorem confinement, own-proof
pre-completion rejection, post-completion success, inclusion/exclusion,
recovery, provenance, deterministic mutations, positive-per-edge coverage,
mixed-list filtering, representative all-other actions, and missing/additional/
wrong/relocated/wrapped upper-chain negatives.

R-032A/R-032B exclude parser/frontend production, Cargo/workspace metadata, all other
resolver modules, checker/type/proof/Core/CFG/VC semantics, public diagnostic
codes, grouped/qualified/bulk/imported/definition/registration label work,
`.miz`, sidecar, trace status/count, and active runner changes. After both land
with fresh inventory, the later private `mizar-test`
`declaration_symbol` consumer may own exact key
`declaration_symbol.label.proof_scope_confinement`. Public checker
`SourceStatementReferenceHandoff` stays excluded because it rejects unresolved
references.

The historical resolver exit remains closed. S-026 is an external syntax
prerequisite, and R-032A/R-032B are complete bounded post-exit follow-ups;
none reuses or changes the historical score.

## R-032A bounded post-exit implementation result

R-032A is complete as its own lower-prerequisite logical task. It adds the
exact frozen `SurfaceResolvedArena` and `SurfaceResolvedArenaError`, the
complete focused test matrix, the sole R-026 owning-spec decision, and
synchronized live status records. It changes no label collector, runner,
fixture, sidecar, expectation, trace status/count, public diagnostic, Cargo
metadata, or checker/type/proof behavior. The historical R-001 through R-029
exit and score remain unchanged. At R-032A completion the R-032B stream became
next; its current first post-exit task is the lint-policy docs correction,
followed by R-032B implementation and the active B5C consumer.

`doc/design/spec_coverage_audit.md` is deliberately unchanged: R-032A changes
no active `.miz` mapping, traceability backlink/status/count, owner crate,
deferred status, or coverage credit.

## R-032B lint-policy frozen-scope correction (completed prerequisite record)

Fresh inventory identifies the omitted R-026 decision owner as High
`design_drift`, with no semantic `spec_gap`, `test_gap`, or test-intent
change. Later R-032B implementation is restricted to the exact three Rust
files above; `tests/lint_policy.rs` may receive only the sole
`ProofLabelSourceCollectionError` / `labels.md` decision.

The synchronized docs-only correction covers exactly 31 design files
(16 resolver, eight checker, six `mizar-test`, one global ledger) and changes
no source, specification, fixture, sidecar, expectation, trace status/count,
Cargo metadata, or historical exit score. `spec_coverage_audit.md` is a
deliberate no-op. The independent specification, test/scope, and
source/documentation consistency reviews report **NO FINDINGS**, and the
docs-only verification/count/hash gates PASS. Independent final read-only
quality also reports **NO FINDINGS**; all nine hard gates PASS with no cap at
valid `100/100` (`20/20/15/15/10/10/5/5`). At that pre-commit record, only
task-only staging/cached-diff review, commit, and post-commit
invariant/fresh-inventory gates remained pending. They subsequently completed
in correction commit `f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a` and its
fresh inventory.

## R-032B bounded post-exit implementation result

The exact R-032B source/API is committed at
`b3a7e79a6b60db2974e911c69bb56ff5f4609064`.
It uses only `labels.rs`, `labels/tests.rs`, and the sole
`tests/lint_policy.rs` `ProofLabelSourceCollectionError` / `labels.md`
decision, plus synchronized live status records. It validates the R-032A map
and emits only existing proof-step projections and simple-reference
candidates with the frozen default-deny traversal, scopes, ordinals,
completion, origins, and `proof-step-v1` identity.

The initial High/Medium plus two fresh Medium test gaps, the Medium
third-child implementation defect, and the two Medium unauthorized `Default`
/ `From` findings are fixed. Preimplementation specification and final fresh
test-sufficiency, implementation, and source/documentation reviews all report
**NO FINDINGS**. Focused collector `25/25`, labels `35/35`, resolver
`144 + 11 + 1`, formatting, workspace Clippy/test, diff, CLI, count/hash, and
exact 20-file scope gates PASS. Independent final quality reports **NO
FINDINGS**; all nine hard gates PASS with no score cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Task-only restaging/cached-diff review, commit,
and post-commit invariant/fresh inventory are complete. No active
artifact, fixture, expectation, sidecar,
trace, runner, checker handoff, semantic behavior, public diagnostic, Cargo
metadata, historical exit status, or historical score changed in R-032B.

`doc/design/spec_coverage_audit.md` is deliberately unchanged because no
active `.miz` mapping, traceability backlink/status/count, owner crate,
deferred status, or coverage credit changes.

## Checker Task 258B5C historical active post-exit status

The historical B5C implementation privately consumes the unchanged R-032A
`SurfaceResolvedArena` and R-032B `ProofLabelSourceCollector` /
`LabelResolver` APIs in `mizar-test`. It adds exactly two fail fixtures,
two expectation sidecars, and two covered trace rows.
`crates/mizar-test/tests/metadata.rs` updates four frozen active-count/CLI
assertions from declaration stage `5` to `7`. Resolver production and public
API remain unchanged. The plan is
`421/389`, pass/fail is `228/193`, active parse/declaration/type/proof is
`101/7/198/1`, and warning/error counts are `23/0`.

Public diagnostic codes remain empty; the private key is
`declaration_symbol.label.proof_scope_confinement`. This closes only the
inner-to-outer and sibling confinement negatives. R-G007 remains open for
import, name, dot-chain, and other label-reference coverage. B5C test,
implementation, source/documentation reviews and all verification gates are
complete. Independent final quality reports **NO FINDINGS**; all nine hard
gates PASS with no score cap at valid `100/100`. Task-only cached-diff review,
dedicated commit `33ac57e96f048dc40559565f54369cac854409a7`, and post-commit
fresh inventory are complete.

## Checker Task 263R Active Lower Exit Addendum

Task 263R is a later bounded post-exit maintenance task and does not revise the
historical crate-wide exit score. Its docs prerequisite is committed as
`34692ee222d5465750f061da82fe878566a1557c`. The exact two-file `symbols`
implementation and two tests close the frozen selector-owner `source_drift`
and `test_gap`; focused tests, exact probes, and the test/implementation
reviews pass with **NO FINDINGS**. Resolver tests are `146` and production is
`15/18896`. Source/documentation review and all full verification gates pass;
independent final quality is **NO FINDINGS**, nine hard gates PASS, uncapped
`100/100`. Only the separate implementation commit and clean fresh inventory
remain. No public API,
corpus, trace, runner/checker, metadata, or semantic coverage credit changes.

## Resolver Task 277R1 Post-Exit Prerequisite

This later [Task 277R1 prerequisite](../../task_contracts/en/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md)
does not revise the historical crate exit or its score. It freezes a narrow
`names`-owned structural transport and a private direct-fixture assertion for
the already inactive semantic seed. The exact four-path implementation and
five regressions now pass independent test/implementation reviews and all
focused/full verification. Checker/runner production behavior, active corpus/
trace/coverage status, diagnostics, semantic verdicts, and historical exit
evidence remain unchanged. Independent source/docs and bilingual reviews report
**NO FINDINGS**. Independent final-quality review also reports **NO FINDINGS**;
all nine hard gates pass without a score cap at valid `100/100`. Exact staging,
commit, and post-commit inventory are complete with task-only implementation commit
`b22033c38249326e366ceb9e19b1a9100da2248e` and the central contract's historical
checkpoint. This post-exit prerequisite is complete without revising the
historical crate exit or its score; Task 277B remains not ready.

## Resolver Task 277R2 Post-Exit Prerequisite

This later [Task 277R2 prerequisite](../../task_contracts/en/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md)
does not revise the historical crate exit or its score. It freezes a narrow
`names`-owned generator-variable binding/use collection and one private direct
fixture assertion for the already inactive F5 seed. The future exact five-path
implementation remains subject to fresh preflight, independent reviews, all
nine hard gates, an uncapped score of at least 90/100, full verification, exact
staging, task-only commit, and post-commit proof. Production behavior,
diagnostics, active trace/coverage state, semantic verdicts, and Task 277B
readiness remain unchanged with zero semantic credit.
