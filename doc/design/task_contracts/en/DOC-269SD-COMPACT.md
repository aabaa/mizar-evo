# Task DOC-269SD-COMPACT: Batch Legacy Completion-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-269SD-COMPACT.md](../ja/DOC-269SD-COMPACT.md).

This is a derived documentation-maintenance contract. It cannot introduce or
override language behavior, test intent, diagnostics, public API, or coverage
credit. It batches two completed tasks only because the user explicitly
authorized one coherent cleanup after Task 269SDT.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269SD-COMPACT` |
| Status | Implementation and verification complete; ready for the task-only commit. The immutable commit identity is post-commit evidence. |
| Purpose | Replace duplicated Task-269SDP and Task-269SDC completion evidence with one paired historical contract per task, retain every owner-local durable fact, and prevent the same contract/link drift prospectively. |
| Primary owners | Repository documentation policy and `mizar-test` lint policy |
| Consumers | Checker/test crate plans, owner-local checker/test design documents, audits, TODOs, and future autonomous-task agents |
| Dependencies | Task-269SDP implementation `2ba1ee910aea4939abc26b64a96a113e80c01306`; Task-269SDC implementation `b1c8c814655d58fff5e5445dd94132bab37965c7`; central-contract policy `f322a710`; Task-269SDT migration `ee91030f`; Task-269SDT implementation `c5389023eddf84600c5f7972b240712673e76d95` |
| Readiness | Fresh inventory at clean `c5389023` found one bounded exact-duplication family and no repository-metadata conflict. |

## Authority And Classification

The authority for this maintenance task is the repository workflow in
[`AGENTS.md`](../../../../AGENTS.md), the
[autonomous crate protocol](../../autonomous_crate_development.md#canonical-task-contracts),
and the user's explicit approval to compact the remaining duplicated
documentation as one task. The language authority for the two historical
tasks remains indexed by their new contracts; it is not reinterpreted here.

| Class | Decision |
|---|---|
| `design_drift` | Task-269SDP completion evidence is repeated in 40 Markdown files and Task-269SDC completion evidence in 42. The task-contract policy is present but recursive pair/link enforcement and an explicit batch-migration safety rule are absent. |
| `test_gap` | No repository test recursively checks paired task-contract paths and their local Markdown targets/fragments. One focused lint-policy test closes this derived-policy gap. |
| `spec_gap` | None for documentation structure. The historical Chapter-4/15 `set` disagreement remains unchanged and continues to block `z`/`q` semantics. |
| `source_drift` | None. Production Rust is out of scope. |
| `source_undocumented_behavior` | None introduced or inferred. |
| `test_expectation_drift` | None; expectations are protected. |
| `boundary_violation` | Avoided by retaining every adjacent owner-local API, invariant, runner, audit, traceability, and sequencing section. |
| `repo_metadata_conflict` | None at selection: `origin/main...HEAD` was `0/1`, the worktree was clean, and the protected stash was unchanged. |

## Frozen Migration Surface

The source family consists of exactly 82 H2 completion-status sections:

- 40 Task-269SDP sections: 20 English `Implementation Status` headings and
  20 Japanese lowercase `implementation status` headings
- 42 Task-269SDC sections: 20 English-tree, 20 Japanese-tree, and two
  English root-audit/roadmap `Implementation Status` headings

Their baseline body is 3,027 lines. Each source section is replaced in place
by a compact language-local link to the corresponding historical contract's
`Completion Evidence` section. The surrounding H2 before and after the
section, and every non-status byte in every affected file, are protected except
for the exact compact Task Index rows frozen below.
English files link the English historical contract, Japanese files link the
Japanese companion, and the two root English documents link English.

The task adds the paired historical contracts
[`269SDP`](./269SDP.md) and [`269SDC`](./269SDC.md), plus this synchronized
EN/JA migration contract. During implementation, each of the four checker/test
EN/JA Task Index tables will gain one row per contract, or three rows per
table: exactly 12 planned new rows. No historical task
plan, API, test-design,
runner, traceability, coverage-mapping, boundary, bilingual, or TODO owner
section outside the exact status sections is removed in this task.

The policy delta is limited to `AGENTS.md`, `doc/design/README.md`, and
`doc/design/autonomous_crate_development.md`. It permits a user-authorized,
separately reviewed batch legacy-evidence compaction only for a coherent
duplication family with an exact redirect map, paired EN/JA owners, preserved
owner-local facts, link validation, and no behavioral or coverage change. It
does not authorize wholesale historical rewriting during ordinary semantic
tasks.

The test delta is limited to `crates/mizar-test/tests/lint_policy.rs`. One new
integration test recursively inventories `doc/design/task_contracts/en` and
`ja`, requires identical relative Markdown paths, checks the canonical/
companion markers and reciprocal owning-crate-plan Task Index links, enforces
the exact 82 legacy redirects, and validates every repository-supported inline
relative Markdown file target and ATX-heading fragment reached from a task
contract. HTTP(S), mail, bare non-file references, reference-style links,
escaped or nested-parenthesis destinations, and Markdown inside code are
outside this lint grammar. Fragment validation uses the deterministic
repository ATX-heading slug function, including duplicate-heading suffixes;
this is not a claim of complete GitHub Markdown parsing. The test must pass for
the existing 269SDT pair and all pairs added here.

Task order is indexed by the [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index)
and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index).

## Protected And Forbidden Changes

This task must not change `doc/spec/**`, any `.miz`, fixture, sidecar,
expectation, `tests/coverage/spec_trace.toml`, trace row/status/backlink,
Cargo manifest, production Rust, public API, diagnostic, parser/resolver
output, active route, CLI result, or executable coverage credit. It must not
rewrite owner-local durable sections or infer language meaning from current
source. It must not resolve the Chapter-4/15 `set` disagreement or claim goal,
guard, fact, proof, discharge, acceptance, obligation, closure, or capture
semantics.

## Baseline And Expected Impact

At selection, `doc/design` contains 632 Markdown files and approximately
383,168 lines. The two exact repeated status families contain 82 sections and
3,027 lines. The task is expected to make a substantial net deletion while
adding six contract files forming three EN/JA pairs, three small policy deltas,
exactly 12 planned compact Task Index rows across four plan files, and one
lint-policy test.
Final counts are measured rather than projected into a required byte total.

Checker/runner library counts, production file inventories/hashes, corpus and
requirement counts, pass/fail totals, stage distribution, type coverage, trace
hash, all five CLI hashes, fixture/expectation hashes, and active results must
remain at the Task-269SDT post-commit baseline. Only the `lint_policy`
integration target test count and list hash may change, by exactly one test.

## Reviews And Verification

Before compaction, an independent specification/documentation reviewer must
verify that the contract freezes an exact, behavior-neutral redirect and
reports **NO FINDINGS**. After edits, independent test-sufficiency,
implementation/equivalence, and source/document/EN-JA/link-owner reviews must
each end **NO FINDINGS**. Parent final review requires all nine hard gates,
no score cap, and at least `90/100`.

Verification includes the focused `lint_policy` target, checker and runner
library suites, repository metadata tests, `cargo fmt --all --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, all
five CLIs, protected artifact/count/hash replay, exact 82-section disappearance
and redirect presence, recursive link/fragment replay, `git diff --check`, and
explicit cached name/stat/content/whitespace plus unstaged-diff review.

## Completion Evidence

- The 82 frozen status sections are absent and have been replaced by exactly 82
  language-local redirects whose files and `#completion-evidence` fragments
  resolve. The four owning Task Index tables contain exactly 12 new rows.
- The task changes 52 paths: 42 legacy owners, six paired contract files, three
  policy files, and one `mizar-test` lint-policy test file. No protected
  specification, fixture, sidecar, expectation, trace, manifest, production
  source, public API, diagnostic, or executable coverage artifact changed.
- `doc/design` contains 638 Markdown files and 381,026 lines, versus
  632 files and approximately 383,168 lines at selection. The six-file increase
  is exactly the three new EN/JA contract pairs.
- The `mizar-test` lint-policy target contains 15 tests; its raw test-list hash
  is `b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`.
  The sole added test recursively enforces pairing, exact redirects and indexes,
  reciprocal owners, and supported target/fragment resolution.
- Independent specification/documentation, test-sufficiency,
  implementation/equivalence, and source/document/EN-JA/link-owner reviews all
  ended **NO FINDINGS** after their findings were corrected.
- Focused and full lint-policy tests, checker/runner library and metadata tests,
  `cargo fmt --all --check`, workspace Clippy with warnings denied, full
  `cargo test`, all five CLIs, protected count/hash replay, and
  `git diff --check` pass. Frozen production/corpus/trace/CLI hashes and counts
  remain unchanged; the trace hash remains
  `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

## Exit And Handoff

Exit requires exactly one task-only commit, all reviews and hard gates passing,
the paired historical records retaining every shared completion fact, all
owner-local sections byte-preserved outside intentional compact indexes/policy,
clean post-commit inventory, unchanged protected stash, and no push. A commit
cannot contain its own hash; the immutable migration commit belongs in the
post-commit report.

After commit, fresh-inventory canonical authority and public APIs and select
the next dependency-ready semantic task without preassigning its ID. Parent
reasoning remains `xhigh`; bounded review agents may use `high`, and only
deterministic inventory may use a lower setting.
