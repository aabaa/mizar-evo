# Task DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT: Runner Final-Quality Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT.md](../ja/DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT.md).

This derived documentation-maintenance contract authorizes one schema-2
whole-section `task_ref` migration only. It cannot change behavior, tests,
diagnostics, source, active results, test intent, or semantic/coverage credit.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT` |
| Status | Complete: prerequisite and migration committed separately; clean postcommit proof recorded. |
| Purpose | Replace six runner-side B3P final-quality H2 sections with language-local links to the existing paired historical owner. |
| Historical owner | [Task 258B3M2B2B3P](./258B3M2B2B3P.md#completion-evidence), canonically owned by registered batch `DOC-258B3M2B2B3P-REVIEW-COMPACT` |
| Dependencies | B3P prerequisite `285a1f11c310bb313c4c6b4feae914eb11f74754`, implementation `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`, schema-2 task-ref owner, and B3B closeout `b12fd7c693f2fe3622154b5a5e6984678cd751ef` |
| Readiness | Clean fresh inventory proves a unique exact family and source-file disjointness from the canonical B3P batch. |

The [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and
[runner plan](../../mizar-test/en/00.crate_plan.md#task-index) index this batch.
The historical task already has its sole registered Task Index ownership and
must not receive duplicate task rows.

## Authority And Classification

Authority is the temporary consolidation gate, [`AGENTS.md`](../../../../AGENTS.md),
the [autonomous migration policy](../../autonomous_crate_development.md#migration-policy),
and schema-2 owner [`DOC-COMPACT-MANIFEST-TASK-REF`](./DOC-COMPACT-MANIFEST-TASK-REF.md).

| Class | Decision |
|---|---|
| `design_drift` | Six completed runner final-quality checkpoints duplicate facts already retained by the B3P historical owner. |
| `spec_gap`, `test_gap`, `source_drift`, `source_undocumented_behavior`, `test_expectation_drift` | None introduced or repaired; historical time-local gaps remain preserved in the owner. |
| `boundary_violation` | Avoided: all review, frozen runner, implemented harness/boundary, postcommit, API, test, audit, traceability, coverage, and sequencing sections remain. |
| `repo_metadata_conflict` | None; local origin divergence is measured and not repaired. |

## Frozen Source-To-Owner Map

Each selection is one complete H2 through the next H2, with no nested heading.
The plan sections are 8 lines each; the harness and module-boundary sections
are 6 lines each. Baseline is 6 sections, 6 distinct files, and 40 lines.

| Source | Lines / SHA-256 | Previous / next anchor |
|---|---|---|
| `doc/design/mizar-test/en/00.crate_plan.md` — `## Checker Task 258B3M2B2B3P Final Quality Status` | 8 / `e0f021d9c51aeb3d4c364d3a6cbac9bf08cea574efe2e71ced0b51606a0764a8` | `## Checker Task 258B3M2B2B3P Documentation Review and Verification Status` / `## Checker Task 258B3M2B2B3P Implementation Closure` |
| `doc/design/mizar-test/en/harness.md` — same heading | 6 / `a2f7a4ad7ce7a98939f4c0fa935783ba446ad50159f4c80c60e5c547cbeca9be` | `## Checker Task 258B3M2B2B3P Documentation Review Status` / `## Checker Task 258B3M2B2B3P Implemented Private Harness` |
| `doc/design/mizar-test/en/module_boundary_audit.md` — same heading | 6 / same EN 6-line hash | `## Checker Task 258B3M2B2B3P Documentation Review Status` / `## Checker Task 258B3M2B2B3P Implemented Runner Boundary` |
| `doc/design/mizar-test/ja/00.crate_plan.md` — `## Checker Task 258B3M2B2B3P final quality status` | 8 / `556789484a48bdc7704d6a9127a8c25891820e43a4538d07a6b1724d11f3cf8b` | `## Checker Task 258B3M2B2B3P documentation review/verification status` / `## Checker Task 258B3M2B2B3P implementation closure` |
| `doc/design/mizar-test/ja/harness.md` — same heading | 6 / `36dca23122d41bbc843f5b85cdf77ae2116307c467eae2264e8346f62b01c882` | `## Checker Task 258B3M2B2B3P documentation review status` / `## Checker Task 258B3M2B2B3P implemented private harness` |
| `doc/design/mizar-test/ja/module_boundary_audit.md` — same heading | 6 / same JA 6-line hash | `## Checker Task 258B3M2B2B3P documentation review status` / `## Checker Task 258B3M2B2B3P implemented runner boundary` |

English replacement:

```text
Completion evidence: [central Task-258B3M2B2B3P historical contract](../../task_contracts/en/258B3M2B2B3P.md#completion-evidence).
```

Japanese replacement:

```text
Completion evidence: [central Task-258B3M2B2B3P historical contract](../../task_contracts/ja/258B3M2B2B3P.md#completion-evidence)。
```

## Source Disjointness And Frozen Ledger Delta

The canonical B3P batch owns 12 checker-tree source files. This later family
uses only the six listed `mizar-test` files, so the sets are disjoint and one
`task_ref` is schema-2-safe. The prerequisite changes exactly six Markdown
paths: this EN/JA pair plus one batch Task Index row in each checker/test EN/JA
plan. It changes no selected section, historical owner, or ledger row.

After the prerequisite commit and clean replay, migration changes exactly nine
paths: six sources, this EN/JA pair, and `legacy_compactions.tsv`. The 40 lines
become six redirects, Git source delta `+6/-34` (net `-28`). The ledger adds
one `batch`, one `task_ref`, six `redirect`, and four batch-only `index` rows.
Declared counts are `1/6/6/4`; the canonical 11-row expanded-inventory SHA-256
is `7f2e494ebb807529320af7b3e812788bcff5e4e15dd95a9bde516fe51341f99a`.
Final global cardinalities become `34/45/5/648/316` with 1,050 physical lines.
The migration records the final physical ledger hash.

## Migration Evidence

The prerequisite was committed alone as
`51785984c685bde5caa59cfb145f352ff8d3b9a2`. Clean replay confirmed HEAD,
`origin/main...HEAD = 0/6`, the unchanged protected stash, unchanged B3B ledger
SHA-256 `ff48f2627e6cd3e52be649ee893ab1bae16fe3ce97ec019c3e96ae77ccad9131`
at 1,038 lines, and focused recursive lint.

The working migration removes exactly the six headings, retains all twelve
anchors, installs six language-local redirects, and adds the canonical 11
inventory rows plus one batch row. The ledger is 1,050 lines with SHA-256
`3145d558b93f85095693b99ea4f3a09198be9b2a0332945667d29ea7d5c96eb7`
and cardinalities `34/45/5/648/316`. Its data rows remain byte-sorted and the
generic recursive ledger/link/fragment lint passes. Independent reviews,
broad verification, exact staging, commit, and clean postcommit proof remain.

Independent specification/schema/equivalence, test-sufficiency/
implementation, and source/documentation/bilingual/boundary reviews ended
**NO FINDINGS**. Recursive lint passed all 15 cases; `git diff --check`,
formatting, warnings-denied all-target/all-feature workspace Clippy, and full
workspace tests passed. Exact staging, final quality review, migration commit,
and clean postcommit proof then completed.

## Postcommit Completion Evidence

Migration commit `80af8e4dfeefdd1f06983bf1d9358774a878eb9e` contains
exactly the nine frozen paths and has prerequisite
`51785984c685bde5caa59cfb145f352ff8d3b9a2` as its parent. Clean replay
confirmed HEAD at that migration commit, `origin/main...HEAD = 0/7`, protected
stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4` unchanged, ledger SHA-256
`3145d558b93f85095693b99ea4f3a09198be9b2a0332945667d29ea7d5c96eb7`,
1,050 lines and cardinalities `34/45/5/648/316`, and focused recursive lint
passing. No protected surface or credit changed. The fresh repository-wide
inventory result is owned by the subsequent bounded wave closeout, not by this
batch contract.

## Protected Boundaries And Exit

Do not edit the canonical B3P historical owner or its original batch/source
inventory. Do not add another task row or task Task Index row. Do not edit
`doc/spec`, `.miz`, expectations, trace, Rust/Cargo, diagnostics, behavior,
tests, active results, `spec_coverage_audit.md`, semantic/coverage credit, or
any unlisted section. Same-task/same-source checker final-quality sections and
mixed or paragraph-only residuals remain intact.

Prerequisite and migration separately require independent equivalence,
test/schema, implementation, source/documentation, bilingual/boundary, and
final-quality reviews ending **NO FINDINGS**; all nine hard gates and an
uncapped `>=90/100`. Verification includes source/hash/anchor/disjointness
replay, generic recursive ledger/link/fragment lint, ledger hashes/counts,
`git diff --check`, formatting, warnings-denied Clippy, full workspace tests,
exact staging, task-only commits, clean postcommit origin/stash proof, and
fresh inventory. Coverage audit impact is explicitly none.

## Handoff

After the migration commit and clean replay, run a fresh repository inventory
and close only the currently authorized schema-2-safe wave if no other
dependency-ready family remains. Keep the parent on GPT-5.6 Sol `xhigh`
because residual classification and Phase-B authority ownership require final
semantic judgment; use Terra `high` for bounded independent reviews. Raise
review effort to `xhigh` for a disputed owner/oracle or schema boundary, and
lower it only for repeatable count/hash/link checks that cannot decide scope,
acceptance, semantics, or credit.
