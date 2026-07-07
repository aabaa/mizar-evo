# Crate Exit Report: mizar-checker

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete.
Quality score: 94/100.
Score caps applied: none.

Post-closeout continuation: this report remains the closeout for the original
task-34 explicit-payload checker milestone. Step 5 source-derived bridge tasks
48-80 are post-closeout continuation tasks tracked in the crate plan, TODO,
source/spec audit, and corpus traceability. Task 74 adds an AST-bounded
structural bare local-mode chain pass slice after task 73's three-edge bridge;
task 75 adds a lower-stage active-range boundary for forward local-mode reserve
heads, task 76 adds the matching boundary for forward local-structure reserve
heads, and task 77 adds the matching boundary for forward local-attribute
reserve type expressions. Task 78 adds an imported structure reserve-head
extraction-gap boundary, and task 79 adds the matching imported mode
reserve-head extraction-gap boundary; task 80 adds the matching imported
attribute reserve extraction-gap boundary. These additions do not change the
original score or claim AST-wide extraction, CoreIr/ControlFlowIr/VC, or proof
payload promotion.

## Scope

Milestone scope:

- Build the `mizar-checker` crate through the task-34 module-boundary gate.
- Own explicit-payload type checking, registration/cluster trace data layers,
  overload resolution, and final resolved typed AST assembly for phases 6-8.
- Keep source-derived semantic corpus execution, proof/artifact acceptance,
  public diagnostic code allocation, and downstream CoreIr/ControlFlowIr/VC,
  kernel, and proof integration deferred until their owning crates and payload
  seams exist.

Included:

- English/Japanese checker crate plan and module specifications.
- Checker Rust source under `crates/mizar-checker/src/`.
- Checker policy/audit tests under `crates/mizar-checker/tests/lint_policy.rs`.
- Deferred corpus traceability rows in `tests/coverage/spec_trace.toml`.

Excluded:

- Direct edits to `doc/spec`.
- Rebaselining existing `.miz` fixtures or expectations.
- Source-to-checker extraction for full AST semantic payloads.
- Proof search, `VcId` assignment, kernel replay, artifact publication, cache
  reuse, and public diagnostic-code registry allocation.

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `f96dc7c` | `docs(checker-plan): add autonomous crate plan` |
| 1 | `7e5c855` | `feat(checker-task-1): scaffold mizar-checker crate` |
| 2 | `1c5c12e` | `docs(checker-task-2): specify typed ast shape` |
| 3 | `902bfd9` | `feat(checker-task-3): implement typed ast data shapes` |
| 4 | `8443aa1` | `docs(checker-task-4): specify binding environment` |
| 5 | `c9989db` | `feat(checker-task-5): implement binding environment` |
| 6 | `234566c` | `docs(checker-task-6): specify type checker` |
| 7 | `28a679a` | `feat(checker-task-7): implement type normalization` |
| 8 | `9408028` | `feat(checker-task-8): implement declaration checking` |
| 9 | `4731ba4` | `feat(checker-task-9): implement term formula inference` |
| 10 | `fa5cb0e` | `feat(checker-task-10): implement coercion obligations` |
| 11 | `61daf6b` | `feat(checker-task-11): implement type fact queries` |
| 12 | `8860acd` | `feat(checker-task-12): add type elaboration corpus runner` |
| 13 | `358256d` | `docs(checker-task-13): specify registration resolution` |
| 14 | `43473d8` | `feat(checker-task-14): implement registration index` |
| 15 | `27a8dc9` | `docs(checker-task-15): specify cluster trace` |
| 16 | `89a4629` | `feat(checker-task-16): implement cluster trace closure` |
| 17 | `8f00d81` | `feat(checker-task-17): implement cluster saturation bounds` |
| 18 | `e760b7e` | `feat(checker-task-18): record reduction trace steps` |
| 19 | `cb9a66c` | `feat(checker-task-19): validate pending registrations` |
| 20 | `b9cf866` | `feat(checker-task-20): enforce existential gates` |
| 21 | `9f2ad8b` | `docs(checker-task-21): specify overload resolution` |
| 22 | `54c7eec` | `feat(checker-task-22): collect overload candidates` |
| 23 | `61c2fe6` | `feat(checker-task-23): expand template candidates` |
| 24 | `7a53387` | `feat(checker-task-24): filter viable overload candidates` |
| 25 | `9e1539a` | `feat(checker-task-25): build specificity graphs` |
| 26 | `39fdaae` | `feat(checker-task-26): resolve overload selections` |
| 27 | `bb69f8a` | `docs(checker-task-27): specify resolved typed ast` |
| 28 | `426ebdf` | `feat(checker-task-28): assemble resolved typed ast` |
| 29 | `7d37506` | `docs(checker-task-29): record deferred corpus obligations` |
| 30 | `4f887bf` | `test(checker-task-30): add determinism suite` |
| 31 | `ad6f5bc` | `test(checker-task-31): guard enum compatibility policy` |
| 32 | `d6ecfd7` | `test(checker-task-32): audit source spec correspondence` |
| 33 | `30a854e` | `docs(checker-task-33): audit bilingual documentation sync` |
| 34 | `7e64293` | `docs(checker-task-34): record module boundary audit` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | English canonical module specs cover implemented checker-owned behavior; no `doc/spec` change was made by this milestone. |
| Test contract | passed | Implemented behavior has task-local Rust tests, lint-policy guards, determinism coverage, or explicit deferred traceability. |
| Traceability | passed | `source_spec_audit.md`, `module_boundary_audit.md`, and `tests/coverage/spec_trace.toml` record implementation/test/deferred links. |
| Design/source sync | passed | Source/spec, bilingual, and module-boundary audit guards cover public surface, doc pairs, and source layout. |
| Boundary discipline | passed | The crate keeps explicit checker payload boundaries and does not own proof, VC, artifact publication, or parser/source extraction. |
| Verification | passed | Required broad commands and cached diff check passed. |
| Residual risk | passed with deferred items | Remaining items are classified as deferred or external dependency gaps below. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 5/5 |
| Total | 94/100 |

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | No blocking/high/medium findings after fixes. |
| Test sufficiency review | No blocking/high/medium findings. |
| Full implementation review | No blocking/high/medium findings after fixes. |
| Source/documentation consistency review | No blocking/high/medium findings after fixes. |
| Read-only crate quality review | No blocking/high/medium findings; score 94/100 with no score caps. |

Quality-review residual risks: semantic translation equivalence remains
human-reviewed rather than mechanically proven, and source-derived checker
corpus coverage remains blocked by the deferred external dependency gaps below.
The quality reviewer deducted points for the intentionally deferred real `.miz`
semantic coverage and the final cached diff check being a commit-time gate.

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MC-G002 | Real semantic `.miz` checker coverage remains blocked by source-to-checker extraction. | `mizar-test` / checker extraction follow-up. | Active semantic runner and checker-ready source payloads exist. |
| MC-G004 | Artifact producer/reuse integration is cross-crate work, not checker-local schema invention. | `mizar-artifact` plus future checker integration. | Accepted artifact summary producer/reuse path is available. |
| MC-G005 | Public diagnostic code-space is not allocated. | `mizar-diagnostics` / checker diagnostic-code task. | Shared diagnostic registry and checker allocation policy exist. |
| MC-G006 | Parser/syntax template and scheme roles are not fully checker-ready. | `mizar-parser` / `mizar-syntax` / overload extraction. | Template/scheme role payloads are available without source fabrication. |
| MC-G011 | AST-wide binding extraction, use-site scope payloads, reserve payloads, and closure payloads are missing. | Resolver/checker extraction follow-up. | Resolver emits checker-ready binding payloads. |
| MC-G014 | AST-wide type-expression and mode/radix/attribute expansion payloads are missing. | Resolver/checker extraction follow-up. | Checker-ready type-expression payload extraction exists. |
| MC-G016 | Declaration/type-site tables, reserve defaults, RHS/body payloads, and evidence queries are missing. | Checker extraction follow-up. | Source declarations and evidence are available as checker-owned payloads. |
| MC-G017 | Term/formula payload tables, built-ins, candidate signatures, source `qua`, and sethood evidence are missing. | Checker extraction follow-up. | Checker-ready term/formula and evidence payloads exist. |
| MC-G018 | Coercion request tables, inheritance graphs, cluster evidence, and proof-query results are missing. | Checker / proof integration follow-up. | Coercion evidence payloads and proof-query inputs exist. |
| MC-G019 | Statement/proof assumptions, theorem acceptance payloads, and phase-7 trace fact payloads are missing. | Checker / proof integration follow-up. | Statement/proof payload extraction and accepted facts exist. |
| MC-G020 | Source-to-checker payload extraction blocks full semantic pass fixtures. | Checker extraction follow-up. | AST-wide checker payload bridge exists. |
| MC-G021 | Registration payload extraction and accepted-status integration are missing. | Checker / artifact / proof integration follow-up. | Checker-ready registration payloads and accepted activation status exist. |
| MC-G023 | Source-derived cluster/reduction fixtures, artifact/cache integration, and real trace extraction are missing. | Checker / artifact / cache follow-up. | Source-derived trace payloads and artifact/cache integration exist. |
| MC-G025 | Proof/artifact production or import of accepted registration status is not wired. | `mizar-proof` / `mizar-artifact` / checker integration. | Accepted registration status can be produced or imported. |
| MC-G026 | Source-derived existential gate cases and artifact reuse are not wired. | Checker / artifact integration follow-up. | Accepted-status integration and source-derived gate payloads exist. |
| MC-G027 | Source-derived overload payloads, diagnostics code allocation, artifact emission/reuse, and semantic fixtures are missing. | Checker overload extraction / diagnostics / artifact follow-up. | Overload source payloads, diagnostic codes, and artifact path exist. |
| MC-G030 | `formula_statement` and `advanced_semantics` runner/tag support plus source payload extraction are missing. | `mizar-test` / checker extraction follow-up. | Active runners and checker-ready payload seams exist. |

## Human Review Surface

- Canonical English specs under `doc/design/mizar-checker/en/`.
- Japanese companions under `doc/design/mizar-checker/ja/`.
- Checker source under `crates/mizar-checker/src/`.
- Checker lint and audit guards under `crates/mizar-checker/tests/lint_policy.rs`.
- Deferred corpus traceability under `tests/coverage/spec_trace.toml`.

## Test Expectation Summary

Existing `.miz` fixtures and expectation sidecars were not rebaselined to match
implementation behavior. New checker behavior is covered by Rust tests and
lint-policy guards; unavailable semantic corpus coverage is explicitly deferred.

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy -p mizar-checker --all-targets -- -D warnings` | passed |
| `cargo test -p mizar-checker` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed |

## Next-Task Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue from the completed mizar-checker autonomous crate milestone. Start the
next crate or integration task from a clean worktree after the closeout commit.
Use the mizar-checker crate exit report, source/spec audit, and deferred MC-G
rows as inputs. Do not fabricate source-to-checker payloads, accepted
registration status, artifact schemas, or active semantic corpus runners; first
select the owning crate/task for the missing seam and follow AGENTS.md with one
task per commit.
```

Rationale: downstream work crosses semantic boundaries into source extraction,
artifact/proof acceptance, diagnostics, and later `mizar-core` / `mizar-vc` /
`mizar-kernel` / `mizar-proof` integration. Lower reasoning is acceptable only
for docs-only synchronization or narrow lint-guard maintenance; raise or keep
`xhigh` for behavior, type, trace, artifact, or proof-boundary changes.
