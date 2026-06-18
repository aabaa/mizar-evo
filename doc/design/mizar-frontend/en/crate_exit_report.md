# Crate Exit Report: mizar-frontend

> Canonical language: English. Japanese companion: [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the current retrospective `mizar-frontend` milestone, with
deferred parser-growth and producer-backed fallback items documented below.

Quality score: reviewed 93/100.

Score caps applied: none. No unresolved hard gate failure,
`source_undocumented_behavior`, or `test_expectation_drift` is known in
`mizar-frontend` scope. Deferred items are either parser-owned future grammar
growth or upstream-producer availability limits.

## Scope

Milestone scope:

- record autonomous crate-development evidence for the completed
  `mizar-frontend` crate;
- classify frontend-owned orchestration behavior, parser seam coverage,
  fallback diagnostic surfaces, and future parser-growth obligations;
- synchronize English and Japanese frontend design indexes with the crate plan
  and exit report;
- keep implementation, canonical specification, `.miz` tests, and expectation
  files unchanged.

Included:

- source loading projection, span bridge, preprocessing, import stubs, active
  lexical environment provider boundary, parser-assisted lexing, parser seam,
  orchestration, diagnostics, cache keys, deterministic output, lint/public API
  policy, frontend fuzz target, and Criterion baseline evidence;
- [00.crate_plan.md](./00.crate_plan.md) and this report;
- README index updates for English and Japanese design documents.

Excluded:

- changing `doc/spec/en`, `doc/spec/ja`, existing `.miz` tests, expectation
  files, source implementation, fuzz implementation, or benchmark code;
- parser-owned grammar expansion, template/fixity `.miz` seed activation,
  module resolution, semantic name/type/overload/proof behavior, cache storage,
  artifact publication, and LSP display rendering;
- adding producer-backed tests for future non-exhaustive variants that current
  upstream crates cannot yet produce.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | [00.crate_plan.md](./00.crate_plan.md) maps frontend behavior to architecture, component design, and language-spec refs. No canonical spec edit is included in this migration. |
| Test contract | Pass with deferred `.miz` rationale | `crates/mizar-frontend` Rust tests cover orchestration contracts; active parser `.miz` seeds cover current frontend-reachable parser seam behavior; non-complete-source orchestration surfaces remain Rust-tested. |
| Traceability | Pass | [source_spec_correspondence.md](./source_spec_correspondence.md) records task 1-29 source/design/test correspondence; [00.crate_plan.md](./00.crate_plan.md) reclassifies that evidence into autonomous-protocol tasks and gaps. |
| Design/source sync | Pass | Existing module specs describe the implemented source files; this migration adds the missing crate plan, exit report, and README index entries. |
| Boundary discipline | Pass | [README.md](./README.md) and [00.crate_plan.md](./00.crate_plan.md) exclude source identity ownership, lexer rules, syntax node ownership, parser grammar/recovery, semantic phases, cache storage, and LSP rendering. |
| Verification | Pass | Current branch verification results are recorded below. `mizar-test plan` exits successfully with existing planned/no-tests warnings outside `mizar-frontend` scope. |
| Residual risk | Pass | `MF-GAP-003`, `MF-GAP-004`, and `MF-GAP-007` are deferred with owners and unblock conditions; no blocking/high frontend-owned finding remains. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 9/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 93/100 |

The score keeps small deductions for the retrospective nature of the evidence,
the reliance on Rust tests for frontend-only orchestration contracts, and
deferred producer-backed fallback coverage.

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MF-GAP-003 | Many frontend-only orchestration contracts require provider seams, source-load failures, cache-key assertions, or direct fallback constructors rather than complete `.miz` source behavior. | Frontend maintenance | Add `.miz` tests only when the behavior becomes complete-source language/parser behavior; keep Rust tests for orchestration-only contracts. |
| MF-GAP-004 | Some reserved/fallback diagnostic surfaces cannot be produced by current upstream crates. | Owning lexer/session/parser/syntax tasks | Add producer-backed frontend tests when concrete upstream variants exist. |
| MF-GAP-007 | Planned template/fixity parser seeds remain deferred until parser tasks can satisfy their diagnostics through the frontend seam. | Parser/frontend integration | Activate or replace the seeds when parser grammar/fixity support becomes frontend-reachable and traceability metadata can move from planned to covered. |

Resolved during this migration:

- `MF-GAP-001` and `MF-GAP-002`: autonomous crate-plan/report evidence and README
  links were added.
- `MF-GAP-005`: task-16 correspondence evidence is incorporated into the
  protocol hard-gate record.
- `MF-GAP-006`: parser grammar growth remains parser-owned, with bounded
  frontend follow-up only for passthrough, merge-order, fuzz, and cache-key
  effects.

## Human Review Surface

The human reviewer should primarily inspect the retrospective protocol evidence:

- [00.crate_plan.md](./00.crate_plan.md)
- this report
- [README.md](./README.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
- [todo.md](./todo.md)
- [orchestration.md](./orchestration.md)
- [parsing.md](./parsing.md)
- [lexing.md](./lexing.md)
- [lexical_env.md](./lexical_env.md)
- [cache_key.md](./cache_key.md)
- `tests/coverage/spec_trace.toml`
- Japanese companions for the same frontend design files

Canonical language/test surfaces referenced by this report:

- `doc/spec/en/02.lexical_structure.md`
- `doc/spec/en/11.symbol_management.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/15.statements.md`
- `doc/spec/en/16.theorems_and_proofs.md`
- `doc/spec/en/22.error_handling_and_diagnostics.md`
- `doc/spec/en/appendix_a.grammar_summary.md`
- `tests/miz/pass/parser/pass_parser_minimal_token_stream_001.miz`
- `tests/miz/fail/parser/fail_parser_missing_definition_end_001.miz`
- `tests/miz/fail/parser/fail_parser_stray_end_001.miz`
- `tests/miz/pass/parser/pass_parser_template_arguments_001.miz`
- `tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.miz`
- active and planned parser `.expect.toml` sidecars summarized below

No source implementation, `.miz` file, `.expect.toml` file, or canonical
language specification file is changed by this migration.

## Test Expectation Summary

This migration does not change `.expect.toml` files, snapshots, `.miz` files,
source code, fuzz targets, or benchmark code.

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `tests/miz/pass/parser/pass_parser_minimal_token_stream_001.expect.toml` | Active parse-only token preservation through the frontend seam. | Pass | Parse-only/frontend seam | None expected beyond fixture-specific metadata | `tests/coverage/spec_trace.toml` parser entries |
| `tests/miz/fail/parser/fail_parser_missing_definition_end_001.expect.toml` | Active missing-`end` recovery through the frontend seam. | Fail/recover | Parse-only/frontend seam | Missing-end syntax diagnostics; frontend recovery diagnostics allowed by tags | `tests/coverage/spec_trace.toml` parser recovery entries |
| `tests/miz/fail/parser/fail_parser_stray_end_001.expect.toml` | Active stray-`end` rejection through the frontend seam. | Fail/recover | Parse-only/frontend seam | Stray-end syntax diagnostics; frontend recovery diagnostics allowed by tags | `tests/coverage/spec_trace.toml` parser recovery entries |
| `tests/miz/pass/parser/pass_parser_template_arguments_001.expect.toml` | Future parser template seed. | Planned/deferred | Parse-only | Deferred until frontend-reachable parser support exists | `spec.en.syntax.template_arguments.parser` |
| `tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.expect.toml` | Future parser template/fixity fail seed. | Planned/deferred | Parse-only | Deferred until frontend-visible diagnostics exist | `spec.en.syntax.template_arguments.parser` |

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
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: passed with 72 test cases, 57 requirements, 0 errors, and 4 warnings for existing planned requirements outside `mizar-frontend`:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- Start parser-owned grammar/fixity work that can activate currently deferred
  parse-only `.miz` seeds through the frontend seam, then open a bounded
  frontend follow-up for diagnostic passthrough, merge-order, fuzz, and
  `MIZAR_PARSER_CACHE_KEY_VERSION` effects if parser output semantics change.

Known constraints:

- Do not move parser grammar/recovery decisions into `mizar-frontend`.
- Do not add `.miz` or expectation changes merely to match current
  implementation behavior.
- Keep source/session, lexer, syntax, parser, resolver, cache-storage, and LSP
  display boundaries explicit.
- Keep English design docs and Japanese companions synchronized in the same
  change.

Open questions:

- Which parser milestone should first activate
  `spec.en.syntax.template_arguments.parser` through the frontend seam?
- Which future upstream non-exhaustive variant will first require
  producer-backed frontend fallback coverage?

Recommended reasoning setting for the next task:

- `high`, because the next useful work crosses parser/frontend boundaries,
  traceability metadata, active `.miz` seeds, diagnostics, and cache-version
  behavior. Lower to `medium` for documentation-only README or traceability
  cleanup, and raise above `high` if the task changes canonical grammar or
  semantic language behavior.
