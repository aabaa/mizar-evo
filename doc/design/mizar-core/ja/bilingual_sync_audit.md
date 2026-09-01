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
- `crate_exit_report.md`
- `elaborator.md`
- `module_boundary_audit.md`
- `source_family_decomposition.md`
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
現在の closeout document はすべて English/Japanese companion を持つ。resolved pair
update は下に記録する。

## Pair Inventory

| File | English canonical status | Japanese companion status | Sync result |
|---|---|---|---|
| `00.crate_plan.md` | responsibility、authority order、reference、test、gap、task decomposition、exit criteria を定義する。 | localized prose と Japanese reference link で plan を mirror する。 | No drift。Task 23 は両 inventory にこの audit を追加する。 |
| `binder_normalization.md` | representation、normalization、alpha-equivalence、substitution、closure、diagnostic、test、enum policy、forbidden behavior を仕様化する。 | 同じ module spec と gap classification を mirror する。 | No drift。technical term が英語多めに残るのは意図的。 |
| `bilingual_sync_audit.md` | この paired-document inventory、言語固有の許容差分、resolved pair update、remaining classification、docs-only verification を記録する。 | 同じ audit structure と classification を mirror する。 | No drift。restart / closeout inventory のため、この row は意図的に self-referential。 |
| `control_flow.md` | `ControlFlowIr`、block、local、context、contract、ghost effect、termination、diagnostic、handoff site、determinism、enum policy、test を仕様化する。 | 同じ phase-10 design を localized prose で mirror する。 | No drift。architecture-07 ownership drift は両 file で分類済み。 |
| `core_ir.md` | `CoreIr` data shape、generated origin、obligation seed、source map、diagnostic、validation、enum policy、Task-31 pending-proof projection、gap、forbidden behaviorを仕様化する。 | 同じdata-shape、exact projection、boundary policyをmirrorする。 | No drift。Task-31 addition同期済み。 |
| `crate_exit_report.md` | closeout status、task commit、hard gate、score、deferred item、verification、handoff を記録する。 | 同じ closeout evidence と classification を mirror する。 | No drift。closeout で追加。 |
| `elaborator.md` | phase-9 input/output contract、6 lowering step、exact Task-180 adapter、diagnostic、determinism、enum policy、forbidden behaviorを仕様化する。 | 同じsix-step design、exact adapter、external/deferred classificationをmirrorする。 | No drift。Task-31 addition同期済み。 |
| `module_boundary_audit.md` | Task 24 source-layout gate、large review-risk file、closeout 前に必須 split がないこと、deferred move-only follow-up を記録する。 | 同じ audit-only decision と classification を mirror する。 | No drift。Task 24 で追加。 |
| `source_family_decomposition.md` | Task 32のCore 33-53 graph、joint algorithm producer/lowerer contract、prepared consumer 5個、gate、corruption boundary、no-credit exitを記録する。 | 同じtask/dependency authorityとforbidden boundaryをmirrorする。 | No drift。Task 32で追加。 |
| `source_spec_audit.md` | Task 31を含むpublic module/API inventory、source/spec/test/deferred correspondence、`source_undocumented_behavior` pass、CORE-AUDIT follow-up registerを記録する。 | 同じaudit structure、exact Task-180 coverage、CORE-AUDIT gap ID/classをmirrorする。 | No drift。Task 22 lint guardもsource/spec audit pairを検査する。 |
| `task_ledger.md` | current task までの restart status、review result、verification、deferred reason を記録する。 | 同じ ledger row を localized prose で mirror する。 | No drift。Closeout row と task hash backfill は stage 前にこの commit で更新する。 |
| `todo.md` | ordered task list、status legend、verification、notes を定義する。 | ordered task list と verification policy を mirror する。 | No drift。Closeout status は stage 前にこの commit で更新する。 |

## Resolved Pair Updates

| ID | Prior class | Resolution |
|---|---|---|
| CORE-BILINGUAL-G001 | `deferred` | Task 24 で解消済み: `module_boundary_audit.md` は両言語に存在し、paired-file inventory に記録済み。future edit では pair を同期し続ける。 |
| CORE-BILINGUAL-G002 | `deferred` | closeout で解消済み: `crate_exit_report.md` は両言語に存在し、paired-file inventory に記録済み。future edit では pair を同期し続ける。 |

## Remaining Classification

現在の paired file set に active bilingual-documentation gap は残っていない。

Core Task 31は変更した全pair、`00.crate_plan.md`、`core_ir.md`、
`elaborator.md`、`source_spec_audit.md`、`module_boundary_audit.md`、
`crate_exit_report.md`、`task_ledger.md`、`todo.md`を再確認した。exact adapter、
snapshot exception、remaining broad deferred ownership、forbidden scopeはEN/JAで一致する。

この bilingual audit では `spec_gap`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、
`repo_metadata_conflict`、`boundary_violation` は見つからない。Task 31はRust
source、exact Task-180 expectation sidecar、新しいexact traceability row 1件を
implementationとpaired owning documentationで変更し、それらは一致する。既存
`.miz` sourceとsemantic pass intentは不変で、older broad CoreIr trace rowは
deferredのままである。

Core Task 32は`source_family_decomposition.md` pairを追加/recheckし、plan、TODO、
source audit、module spec、ledger、cross-crate ownership noteを同期する。両言語は
33-53 task split、prepared consumer stage、Gates A1/S1、VC-owned substitution
boundary、zero coverage promotionを保持する。

Core Task 33C4C8はchanged plan、TODO、source-family、elaborator、source/spec audit、
module-boundary audit、contract pairを再確認する。両言語はsole Task33C capability、exact
owner join、max-plus-one `x,y` allocation、private canonical order、zero-semantic boundary、
Core35 deferral、unchanged Task277B/protected artifactを保持する。

Core Task 33LBはchanged plan、TODO、source-family、elaborator、source/spec auditと、
canonical [`CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB`](../../task_contracts/en/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md)
contractおよび[日本語companion](../../task_contracts/ja/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md)を
再確認する。両言語はchecker binding identity/order、checked fresh Core allocation、
standalone immutable handoff、default-deny validation、separate C4C8 boundary、zero
semantic/coverage credit、deferred Core 33--35と`MT10-CIR-TE`、unchanged Task277B/
protected artifactを保持する。

Core Task 33I259はplan、TODO、source-family、elaborator、source/spec audit、ledgerと、
canonical [`CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259`](../../task_contracts/en/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md)
contractおよび[日本語companion](../../task_contracts/ja/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md)を
recheckする。両言語はexact Task259 one-predicate cardinality、context-link identity join、
whole-symbol Core lookup、standalone 33LB composition、default-deny boundary、
multi-definition deferral、zero credit、unchanged Task277B/protected artifactを保持する。

Core Task 33I260 の implementation/verification/exact task-only commit
`f8e9fc21` は complete である。Canonical [`CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260`](../../task_contracts/en/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md)
contract と[日本語companion](../../task_contracts/ja/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md) が
profile、boundary、review、verification、protected artifact、deferral の詳細 evidence を所有する。
Final bilingual review は no findings である。

## Core Task 33I261 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md) と
[JA companion](../../task_contracts/ja/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md) は、ordinary
attributeのone-row Core association、immutable handoff/error/producer surface、exact identity/
source-order boundary、default-deny validationを同じ内容で定義する。Implementation/
verification/exact task-only commit `4c6ecafc2a9bee7a4eb6e3f27336733fc672bd57` はcomplete。両言語はzero semantic/execution/coverage
credit、production installなし、generic API/slot/installerなし、Core34--36/`GeneratedOrigin`/
`MT10-CIR-TE` deferred、Task277B not-ready/zero-creditを保持する。English canonical、exceptionなし。

## Core Task 33I262 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md) と
[JA companion](../../task_contracts/ja/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md) は、ordinary
modeのone-row Core association、immutable handoff/error/producer surface、exact identity/
source-order boundary、default-deny validationを同じ内容で定義する。Implementation/
verification complete、documentation repair後の全independent reviewはno findings。両言語は
zero semantic/execution/coverage credit、checker doc変更なし、production install/generic API/
slot/installerなし、Core34--36/`GeneratedOrigin`/`MT10-CIR-TE` deferred、Task277B
not-ready/zero-creditを保持する。English canonical、exceptionなし。

## Guard Decision

Task 23 または closeout では新しい Rust lint guard は追加しない。file-pair set は小さく、現在の
pair はすべてこの audit に列挙されている。Task 22 は最も risk が高い
source/spec audit pair について、public module coverage、public item mention、
CORE-AUDIT gap synchronization を検査する focused guard をすでに追加した。Task 24
以降の future task がより大きな documentation matrix を導入する場合には、より広い
bilingual guard を後続で追加できる。しかしここで追加すると、具体的 coverage gap
なしに planned docs-only closeout update を Rust test 変更にしてしまう。

## Verification

この docs-only task の verification:

- stage 前の `git diff --check`。
- 明示 path stage 後の `git diff --cached --check`。

## Task 33I263 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md)と
[JA companion](../../task_contracts/ja/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md)は、
structure association 2行、sole Derived→Base local dependency、standalone by-value
API、default-deny boundary、prohibited semantics、protected artifact、zero-credit
status、Task277B deferralを同期する。English canonical、exceptionなし。

## Task 33I264 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md)と
[JA companion](../../task_contracts/ja/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md)は、
singleton Task264 carrier/Core association、complete by-value checker owner、scalar
`carrier_item()` API、error precedence、default-deny oracle、exact debug、private two-test
boundary、Core34 handoff、zero-credit status、semantic exclusionを同期する。English
canonical、exceptionなし。

## Task 34I264 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264.md)と
[JA companion](../../task_contracts/ja/CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264.md)は、
authenticated `marker` target/return-member association 1件、field association exclusion、
complete by-value input、environment-first default-deny oracle、exact debug grammar、
private two-test boundary、semantic lowering禁止、
zero-credit status、IR264 CoreIR owner-model handoffを同期する。English canonical、
exceptionなし。

## Task IR264 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264.md)と
[JA companion](../../task_contracts/ja/CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264.md)は、
same private-field `CoreDefinitionOwner`、sole authenticated property initializer、
validation order/reason、ordinary-row debug compatibility、exact property-row grammar、
CoreIR validation matrix 1件、mechanical ordinary-owner migration、protected artifact、
zero-credit status、Core35/36 deferralを同期する。English canonical、exceptionなし。

## Task 34D264 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264.md)と
[JA companion](../../task_contracts/ja/CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264.md)は、
same private domain association、typed parameter/target/application/root/registry validation、
unchanged return association/v1 debug byte、Core34 ownership、derived Rust assertion拡張、
protected test-intent invariance、exact write/no-impact surface、zero-credit status、
Core35/36 deferralを同期する。English canonical、exceptionなし。

## Task 36P264 bilingual contract parity

[EN contract](../../task_contracts/en/CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264.md)と
[JA companion](../../task_contracts/ja/CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264.md)は、
no-Core33-item disposition、Core36 ownership、selector/carrier anti-alias boundary、gap
classification、protected artifact、zero-credit state、same-source lower-checker
prerequisite、Task263 exclusion、later Core33/34 owner handoffを同期する。English
canonical、exceptionなし。
