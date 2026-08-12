# Task DOC-258B3M2B2B3B-DOC-REVIEW-COMPACT: B3B review-evidence compaction

> canonical English:
> [../en/DOC-258B3M2B2B3B-DOC-REVIEW-COMPACT.md](../en/DOC-258B3M2B2B3B-DOC-REVIEW-COMPACT.md)。
> 本文書は同一logical taskの日本語companionである。

これはschema-2 whole-section migration 1件だけをauthorizeするderived
documentation-maintenance contractであり、language behavior、tests、diagnostics、
source、active results、semantic/coverage creditを変更しない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B3B-DOC-REVIEW-COMPACT` |
| Status | frozen prerequisite。migration未開始。 |
| Purpose | 重複するB3B documentation-review/final-quality H2 sections 4件をpaired historical ownerへのlanguage-local linksに置換する。 |
| Primary owner | repository documentation policyとschema-2 ledger |
| Historical owner | [Task 258B3M2B2B3B](./258B3M2B2B3B.md#completion-evidence) |
| Dependencies | B3B prerequisite `080e6824d843655986079f5d5fc41abe06b0fbd6`、implementation `dbbf5f6a2b0bd58d8434fb4687f7bfad398ca4bc`、schema-2 ledger/lint、lifecycle closeout `21809fb311c4a1a97e7cf4a91bb4406e86a9f411` |
| Readiness | clean fresh inventoryがselectしたunique exact whole-section family。 |

[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index)がbatchとhistorical taskを
indexする。

## Authority And Classification

authorityはtemporary consolidation gate、[`AGENTS.md`](../../../../AGENTS.md)、
[autonomous protocol](../../autonomous_crate_development.md#migration-policy)、schema-2 owner
[`DOC-COMPACT-MANIFEST-TASK-REF`](./DOC-COMPACT-MANIFEST-TASK-REF.md)。

| Class | Decision |
|---|---|
| `design_drift` | same completed documentation-review/final-quality evidenceがlive historical ownerなしに4 plansで重複する。 |
| `spec_gap`, `test_gap`, `source_drift`, `source_undocumented_behavior`, `test_expectation_drift` | 導入・repairなし。 |
| `boundary_violation` | adjacent frozen-contract、implementation、postcommit、API、runner、audit、traceability、coverage、bilingual、sequencing sectionsを保持して回避する。 |
| `repo_metadata_conflict` | なし。current origin divergenceはordinary local task commitsでrepairしない。 |

## Frozen Source-To-Owner Map

selected sectionは各complete H2からnext H2まででnested headingなし、exact 18 physical
lines。English 2 sectionsはbyte-identical、SHA-256
`4c41c3e16f97187bf5e7d91e6e3978abdee505f9ed859fbb99e6aabe19f82c57`。
Japanese 2 sectionsもbyte-identical、SHA-256
`3a14a01efa192b5bb13d3dacc3def3bb9d68f1f5cbff98e4e8c3bfd2ecdd826b`。
baselineは4 sections/4 files/72 lines。

| Source | Legacy heading | previous / next same-or-higher anchor | Destination |
|---|---|---|---|
| `doc/design/mizar-checker/en/00.crate_plan.md` | `## Task 258B3M2B2B3B Documentation Review and Final Quality` | `## Task 258B3M2B2B3B Frozen Empty-Enumeration Witness Contract` / `## Task 258B3M2B2B3B Implementation Closure` | `task_contracts/en/258B3M2B2B3B.md#completion-evidence` |
| `doc/design/mizar-test/en/00.crate_plan.md` | 同上 | `## Checker Task 258B3M2B2B3B Runner Frozen Contract` / `## Task 258B3M2B2B3B Implementation Closure` | 同English owner |
| `doc/design/mizar-checker/ja/00.crate_plan.md` | `## Task 258B3M2B2B3B documentation review / final quality` | `## Task 258B3M2B2B3B frozen empty-enumeration witness contract` / `## Task 258B3M2B2B3B implementation closure` | `task_contracts/ja/258B3M2B2B3B.md#completion-evidence` |
| `doc/design/mizar-test/ja/00.crate_plan.md` | 同上 | `## Checker Task 258B3M2B2B3B runner frozen contract` / `## Task 258B3M2B2B3B implementation closure` | 同Japanese owner |

English replacement line：

```text
Completion evidence: [central Task-258B3M2B2B3B historical contract](../../task_contracts/en/258B3M2B2B3B.md#completion-evidence).
```

Japanese replacement line：

```text
Completion evidence: [central Task-258B3M2B2B3B historical contract](../../task_contracts/ja/258B3M2B2B3B.md#completion-evidence)。
```

## Frozen Ledger And Index Delta

prerequisiteはexact 8 Markdown paths、本EN/JA batch pair、EN/JA historical pair、
checker/test EN/JA plans各1件のtask+batch Task Index rowを変更する。source section/
ledger rowは変更しない。

dedicated prerequisite commit/clean replay後、migrationはexact 7 paths、selected 4
plan sources、本EN/JA batch pair、`legacy_compactions.tsv`を変更する。72 linesを4
redirectへ置換し、exact source delta `+4/-72`。ledgerはone `batch`、one canonical
`task`、4 `redirect`、8 `index` rowsを追加し`task_ref`なし。declared batch countsは
`1/4/4/8`、canonical 13-row expanded-inventory SHA-256は
`be67d601c91a3d00745ea982ae5aae9c6f6dd8d1eafbfed0e1573a28a38f4b73`。
final global cardinalityは`33/45/4/642/312`、1,038 physical lines。final physical ledger
hashはmigration commitへrecordする。

## Protected And Forbidden Changes

`doc/spec`、`.miz`、expectations、trace、production/test Rust、Cargo、diagnostics、
active routes/results、test intent、semantic/coverage credit、
`doc/design/spec_coverage_audit.md`、unlisted documentation sectionを変更しない。
another B3B section、B3P、paragraph/mixed-owner evidenceをmigrateしない。listed headingsは
消滅、anchorsはexact保持する。prerequisiteでledger/source変更は禁止。

## Reviews, Verification, And Exit

prerequisite/migrationはseparately independent evidence-equivalence、test-sufficiency/
schema、implementation、source/documentation、bilingual/boundary、final-quality reviewを
**NO FINDINGS**にする。全9 hard gatesをscore capなし`>=90/100`でpassする。
verificationはexact section/hash/count/anchor replay、recursive ledger/link/fragment lint、
ledger count/hash replay、`git diff --check`、format、warnings-denied workspace Clippy、
full workspace tests、exact staging、task-only commits、clean postcommit origin/stash proof、
fresh single-family inventoryを含む。

`doc/design/spec_coverage_audit.md`はdesign mapping/owner/traceability/deferral/credit
impactなしでunchanged。migration後fresh inventoryはseparately observed B3P runner
final-quality familyを再評価できるが、本contractはauthorizeしない。
