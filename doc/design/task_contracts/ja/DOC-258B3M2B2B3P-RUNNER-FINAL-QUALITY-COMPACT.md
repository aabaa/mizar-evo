# Task DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT: runner final-quality compaction

> canonical English:
> [../en/DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT.md](../en/DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT.md)。
> 本文書は同一logical taskの日本語companionである。

これはschema-2 whole-section `task_ref` migration 1件だけをauthorizeするderived
documentation-maintenance contractであり、behavior、tests、diagnostics、source、
active results、test intent、semantic/coverage creditを変更しない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B3P-RUNNER-FINAL-QUALITY-COMPACT` |
| Status | frozen prerequisite。migration未開始。 |
| Purpose | runner-side B3P final-quality H2 sections 6件をexisting paired historical ownerへのlanguage-local linksに置換する。 |
| Historical owner | registered batch `DOC-258B3M2B2B3P-REVIEW-COMPACT`がcanonical ownershipする[Task 258B3M2B2B3P](./258B3M2B2B3P.md#completion-evidence) |
| Dependencies | B3P prerequisite `285a1f11c310bb313c4c6b4feae914eb11f74754`、implementation `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`、schema-2 task-ref owner、B3B closeout `b12fd7c693f2fe3622154b5a5e6984678cd751ef` |
| Readiness | clean fresh inventoryがunique exact familyとcanonical B3P batchからのsource-file disjointnessをproveした。 |

[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index)はbatchをindexする。
historical taskはsole registered Task Index ownership済みで、task rowをduplicateしない。

## Authority And Classification

authorityはtemporary consolidation gate、[`AGENTS.md`](../../../../AGENTS.md)、
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy)、
schema-2 owner [`DOC-COMPACT-MANIFEST-TASK-REF`](./DOC-COMPACT-MANIFEST-TASK-REF.md)。

| Class | Decision |
|---|---|
| `design_drift` | completed runner final-quality checkpoints 6件がB3P historical ownerに保存済みのfactsをduplicateする。 |
| `spec_gap`, `test_gap`, `source_drift`, `source_undocumented_behavior`, `test_expectation_drift` | 導入・repairなし。historical time-local gapsはownerに保存する。 |
| `boundary_violation` | review、frozen runner、implemented harness/boundary、postcommit、API、test、audit、traceability、coverage、sequencing sectionsを保持して回避する。 |
| `repo_metadata_conflict` | なし。local origin divergenceはmeasureだけ行いrepairしない。 |

## Frozen Source-To-Owner Map

selectionは各complete H2からnext H2まででnested headingなし。plan sectionsは各8
lines、harness/module-boundary sectionsは各6 lines。baselineは6 sections/6 distinct
files/40 lines。

| Source | Lines / SHA-256 | previous / next anchor |
|---|---|---|
| `doc/design/mizar-test/en/00.crate_plan.md` — `## Checker Task 258B3M2B2B3P Final Quality Status` | 8 / `e0f021d9c51aeb3d4c364d3a6cbac9bf08cea574efe2e71ced0b51606a0764a8` | `## Checker Task 258B3M2B2B3P Documentation Review and Verification Status` / `## Checker Task 258B3M2B2B3P Implementation Closure` |
| `doc/design/mizar-test/en/harness.md` — same heading | 6 / `a2f7a4ad7ce7a98939f4c0fa935783ba446ad50159f4c80c60e5c547cbeca9be` | `## Checker Task 258B3M2B2B3P Documentation Review Status` / `## Checker Task 258B3M2B2B3P Implemented Private Harness` |
| `doc/design/mizar-test/en/module_boundary_audit.md` — same heading | 6 / same EN 6-line hash | `## Checker Task 258B3M2B2B3P Documentation Review Status` / `## Checker Task 258B3M2B2B3P Implemented Runner Boundary` |
| `doc/design/mizar-test/ja/00.crate_plan.md` — `## Checker Task 258B3M2B2B3P final quality status` | 8 / `556789484a48bdc7704d6a9127a8c25891820e43a4538d07a6b1724d11f3cf8b` | `## Checker Task 258B3M2B2B3P documentation review/verification status` / `## Checker Task 258B3M2B2B3P implementation closure` |
| `doc/design/mizar-test/ja/harness.md` — same heading | 6 / `36dca23122d41bbc843f5b85cdf77ae2116307c467eae2264e8346f62b01c882` | `## Checker Task 258B3M2B2B3P documentation review status` / `## Checker Task 258B3M2B2B3P implemented private harness` |
| `doc/design/mizar-test/ja/module_boundary_audit.md` — same heading | 6 / same JA 6-line hash | `## Checker Task 258B3M2B2B3P documentation review status` / `## Checker Task 258B3M2B2B3P implemented runner boundary` |

English replacement：

```text
Completion evidence: [central Task-258B3M2B2B3P historical contract](../../task_contracts/en/258B3M2B2B3P.md#completion-evidence).
```

Japanese replacement：

```text
Completion evidence: [central Task-258B3M2B2B3P historical contract](../../task_contracts/ja/258B3M2B2B3P.md#completion-evidence)。
```

## Source Disjointness And Frozen Ledger Delta

canonical B3P batchはchecker-tree 12 source filesをownし、later familyはlisted
`mizar-test` 6 filesだけなのでsetsはdisjointでone `task_ref`がschema-2-safe。
prerequisiteはexact 6 Markdown paths、本EN/JA pairとchecker/test EN/JA plans各one
batch Task Index rowを変更し、selected section/historical owner/ledgerは変更しない。

prerequisite commit/clean replay後、migrationはexact 9 paths、6 sources、本EN/JA
pair、`legacy_compactions.tsv`を変更する。40 linesを6 redirectsへ置換し、Git source
delta `+6/-34`（net `-28`）。ledgerはone `batch`、one `task_ref`、6 `redirect`、
4 batch-only `index` rowsを追加する。declared countsは`1/6/6/4`、canonical
11-row expanded-inventory SHA-256は
`7f2e494ebb807529320af7b3e812788bcff5e4e15dd95a9bde516fe51341f99a`。
final global cardinalitiesは`34/45/5/648/316`、1,050 physical lines。migrationが
final physical ledger hashをrecordする。

## Protected Boundaries And Exit

canonical B3P historical ownerとoriginal batch/source inventoryを変更しない。another
task row/task Task Index rowを追加しない。`doc/spec`、`.miz`、expectations、trace、
Rust/Cargo、diagnostics、behavior、tests、active results、
`spec_coverage_audit.md`、semantic/coverage credit、unlisted sectionを変更しない。
same-task/same-source checker final-quality、mixed/paragraph-only residualsを保持する。

prerequisite/migrationはseparately independent equivalence、test/schema、
implementation、source/documentation、bilingual/boundary、final-quality reviewsを
**NO FINDINGS**にし、全9 hard gatesとuncapped `>=90/100`をpassする。
verificationはsource/hash/anchor/disjointness replay、generic recursive ledger/link/
fragment lint、ledger hashes/counts、`git diff --check`、format、warnings-denied
Clippy、full workspace tests、exact staging、task-only commits、clean postcommit
origin/stash proof、fresh inventoryを含む。coverage audit impactはexplicitly none。
