# Bilingual Documentation Sync Audit: mizar-core

> 正本は英語です。英語版:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

Task 23 は Task 22 source/spec audit 後の English canonical `mizar-core`
design document と Japanese companion を監査する。この task は documentation-only
であり、Rust source、public API、`.miz` fixture、expectation、traceability
metadata、behavior を変更しない。

## Scope And Method

この audit は `doc/design/mizar-core/en/` 直下の現在の全 file と、
`doc/design/mizar-core/ja/` 直下の同名 file を比較する。

現在の paired file set:

- `00.crate_plan.md`
- `binder_normalization.md`
- `bilingual_sync_audit.md`
- `control_flow.md`
- `core_ir.md`
- `elaborator.md`
- `module_boundary_audit.md`
- `source_spec_audit.md`
- `task_ledger.md`
- `todo.md`

比較は word-for-word ではなく構造と意味で行う。Japanese companion は technical
English term を保持し、説明文を日本語化してよい。許容される言語固有の差分:

- English document は `../../architecture/en/`、`../../../spec/en/`、その他
  English canonical file へ link する。
- Japanese document は `../../architecture/ja/`、`../../../spec/ja/`、存在する
  Japanese companion file へ link する。
- task ledger の prose は localized でよいが、同じ status、review、verification、
  deferred/external meaning を保持する。
- Markdown heading は localized でよいが、section intent と task/gap coverage は
  揃っていなければならない。

Result: 現在の paired file set に blocking bilingual documentation drift は見つからない。
まだ存在しない document は future task output として下に記録し、missing companion
drift として扱わない。

## Pair Inventory

| File | English canonical status | Japanese companion status | Sync result |
|---|---|---|---|
| `00.crate_plan.md` | responsibility、authority order、reference、test、gap、task decomposition、exit criteria を定義する。 | localized prose と Japanese reference link で plan を mirror する。 | No drift。Task 23 は両 inventory にこの audit を追加する。 |
| `binder_normalization.md` | representation、normalization、alpha-equivalence、substitution、closure、diagnostic、test、enum policy、forbidden behavior を仕様化する。 | 同じ module spec と gap classification を mirror する。 | No drift。technical term が英語多めに残るのは意図的。 |
| `bilingual_sync_audit.md` | この paired-document inventory、言語固有の許容差分、future docs の deferred、docs-only verification を記録する。 | 同じ audit structure と classification を mirror する。 | No drift。restart / closeout inventory のため、この row は意図的に self-referential。 |
| `control_flow.md` | `ControlFlowIr`、block、local、context、contract、ghost effect、termination、diagnostic、handoff site、determinism、enum policy、test を仕様化する。 | 同じ phase-10 design を localized prose で mirror する。 | No drift。architecture-07 ownership drift は両 file で分類済み。 |
| `core_ir.md` | `CoreIr` data shape、generated origin、obligation seed、source map、diagnostic、validation、enum policy、gap、forbidden behavior を仕様化する。 | 同じ data-shape と boundary policy を mirror する。 | No drift。 |
| `elaborator.md` | phase-9 input/output contract、6 lowering step、diagnostic、determinism、enum policy、forbidden behavior を仕様化する。 | 同じ six-step elaboration design と external/deferred classification を mirror する。 | No drift。 |
| `module_boundary_audit.md` | Task 24 source-layout gate、large review-risk file、closeout 前に必須 split がないこと、deferred move-only follow-up を記録する。 | 同じ audit-only decision と classification を mirror する。 | No drift。Task 24 で追加。 |
| `source_spec_audit.md` | public module/API inventory、source/spec/test/deferred correspondence、`source_undocumented_behavior` pass、CORE-AUDIT follow-up register を記録する。 | 同じ audit structure と CORE-AUDIT gap ID/class を mirror する。 | No drift。Task 22 lint guard も source/spec audit pair を検査する。 |
| `task_ledger.md` | current task までの restart status、review result、verification、deferred reason を記録する。 | 同じ ledger row を localized prose で mirror する。 | No drift。Task 24 row は stage 前にこの commit で更新する。 |
| `todo.md` | ordered task list、status legend、verification、notes を定義する。 | ordered task list と verification policy を mirror する。 | No drift。Task 24 status は stage 前にこの commit で更新する。 |

## Resolved Pair Updates

| ID | Prior class | Resolution |
|---|---|---|
| CORE-BILINGUAL-G001 | `deferred` | Task 24 で解消済み: `module_boundary_audit.md` は両言語に存在し、paired-file inventory に記録済み。future edit では pair を同期し続ける。 |

## Remaining Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| CORE-BILINGUAL-G002 | `deferred` | `crate_exit_report.md` は closeout output なので、まだどちらの言語にも存在しない。 | Closeout は English と Japanese companion を同じ commit で追加する。 |

この bilingual audit では `spec_gap`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、
`repo_metadata_conflict`、`boundary_violation` は見つからない。既存 `.miz` test、
expectation、Rust source、traceability row は変更しない。

## Guard Decision

Task 23 では新しい Rust lint guard は追加しない。file-pair set は小さく、現在の
pair はすべてこの audit に列挙されている。Task 22 は最も risk が高い
source/spec audit pair について、public module coverage、public item mention、
CORE-AUDIT gap synchronization を検査する focused guard をすでに追加した。Task 24
で document split が発生する、または closeout がより大きな documentation matrix を
導入する場合には、より広い bilingual guard を後続で追加できる。しかしここで追加
すると、具体的 coverage gap なしに planned docs-only synchronization task を Rust
test 変更にしてしまう。

## Verification

この docs-only task の verification:

- stage 前の `git diff --check`。
- 明示 path stage 後の `git diff --cached --check`。
