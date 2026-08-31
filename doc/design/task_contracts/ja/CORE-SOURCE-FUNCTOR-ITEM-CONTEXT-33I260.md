# Task CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260: Task260 functor item context

> canonical English:
> [EN contract](../en/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md)。

Status: implementation/verification complete、exact task-only commit pending。
これはユーザーが採択した Core Task 33I259 の Task-260 固有 successor である。
zero-semantic/zero-credit であり、Core 33 や `MT10-CIR-TE` を完了・有効化しない。

## Identity, authority, and decision

| Field | Contract value |
|---|---|
| Task | `CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260` |
| Primary owner | `mizar-core::elaborator`、Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md) |
| Checker dependency | 既存 Task-248 Profile-B `SourceBindingContextHandoff` と active Task-260 `SourceFunctorDefinitionHandoff` |
| Core dependency | 完了済み `CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB` handoff |
| Prepared consumer | complete Core 33--35 が deterministic real `CoreIr` を生成した後の future `MT10-CIR-TE` |
| User decision | definition-family 汎用化を行わず、推奨された Task-260 固有の二行 standalone handoff を採択 |
| Coverage | semantic/execution credit は zero。Task277B は not-ready/zero-credit |

Authority 順は `doc/spec/en/`、既存 `.miz`、trace、expectation、design、
source のままである。Chapter 10 §§10.1--10.6 が ordinary functor
definition、Chapters 11/12 が current-module identity、visibility、source
order、enclosing definition block を定める。既存 Task-260 source/handoff は、
一つの normal Task-248 Profile-B item 内の二つの normal public functor
definition を source order で認証する。

既存 design は Task-260 と他の ready Core-33 family の順序を決めて
いなかったが、今回のユーザー採択が Task-260 と、block aggregate や二つの
独立 handoff ではない二 definition/二 Core-item 形を選択した。新しい言語
意味はなく `spec_gap` はない。Core association/test の欠落は bounded
`design_drift`/`test_gap` であり、implementation と private consumer がこの bounded
slice を language semantics の変更なしに close した。remote baseline 相違は
report-only `repo_metadata_conflict` のままとする。

## Implemented public API and ownership

`crates/mizar-core/src/elaborator.rs` に次だけを追加する。

- private field を持つ immutable `SourceFunctorCoreItemAssociation` と
  `source_item()`、`definition()`、`symbol()`、`core_item()` getter;
- source-ordered `SourceFunctorCoreItemAssociationTable` と
  `get(SourceFunctorDefinitionId)`、`iter()`、`len()`、`is_empty()`;
- complete 33LB、Task-248、Task-260、association table を by-value 保持する
  immutable `SourceFunctorCoreContextHandoff` と `source_id()`、
  `module_id()`、`context()`、`source_bindings()`、`source_context()`、
  `checker_owner()`、`items()`、non-authoritative `debug_text()`;
- precedence が `EnvironmentMismatch`、`InvalidSourceBindingContext`、
  `InvalidCheckerOwner`、`InvalidCoreContext`、`InvalidItemAssociation` の
  non-exhaustive `SourceFunctorCoreContextError`;
- 三 handoff を by value で受ける
  `SourceFunctorCoreContextProducer::build(...)`。

全入力を完全 postvalidate してから atomic に publish する。constructor、
adapter、installer、unchecked admission、compatibility layer、Core/Typed/
Resolved field は追加しない。33I259 public API は変更しない。private shared
validator は両方の exact family profile を変えず重複だけを減らす場合に限る。

## Cardinality, identity, order, and provenance

Task-248 は exact profile `1/2/2/2/2/2/0`、すなわち一つの normal
`DefinitionBlock` `SourceItemId(0)`、二つの normal ordered parameter
declaration/binding、module/definition binding context、exact owner・parent・
layer・visible binding・normal state を持つ module/definition local type
context、二 context link、zero diagnostic である。Task-260 は definitions/
parameters/guards/definientia/correctness が正確に `2/2/1/2/2` である。
definition 0 は correctness を持たない `Equals`、definition 1 は `Means`
であり、correctness 0/1 は definition 1 だけを owner として existence/
uniqueness order に並ぶ。definition 0/1 は同じ `BindingContextId(1)` を使い、
その `SourceContextLink` は同じ `SourceItemId(0)` を指す。

Core は whole `SymbolId` lookup で選ぶ二つの valid public `Functor` item
のみを持つ。dependency、diagnostic、import、generated origin、partial/
recovery はなく、各 item は一つの pending `DefinitionalItem` boundary と
pending worklist row を持つ。

association table は typed `SourceFunctorDefinitionId` 0/1 を key とする
二行で、Task-260 definition-table source order をそのまま保持する。両行の
source item は 0、symbol/Core item は別 identity である。sort/repair や
numeric reinterpretation を行わない。

各 item/source-map/worklist/boundary は outer `0..261` ではなく inner
definition range `61..118`/`121..179` と、exact checker provenance
`source-functor-core-item-v1.definition.0`/`.1` を使う。worklist order は
identity lookup 後も definition order と一致し、join には使わない。

`CoreItemStatus::Valid` は認証済み item shell のみを表す。Task-260
correctness row は二つの Pending existence/uniqueness obligation への typed
reference を保持するが、obligation row 自体は元の checker projection/
`TypedAst` initial-obligation table が owner であり、この Core handoff はその
table を保持しない。Core obligation、`Partial`、proof/acceptance を作らず、
boundary は `PendingBody` のままである。

## Default-deny oracle

source/module/environment/fingerprint、exact Task-248/260 cardinality・role・
context・order・range・origin・recovery・style・definiens・correctness、
context link/source item、二行 association、Core whole symbol/kind/
visibility/status/source/provenance/source-map/worklist/dependency/diagnostic/
generated-origin/boundary の missing/extra/duplicate/reordered/stale/
mismatch/orphan を fail closed にする。display name、spelling、FQN-only、
range-only、numeric id、shell ordinal、seed/map/worklist iteration join、sort、
repair、inference、recovery、partial publication を禁止する。

## Installation boundary and deferrals

既存 private Task-260 real-source test leaf だけが、認証された二 definition
から Core seed を作り、Core context と 33LB を作成して standalone producer
を呼ぶ。二行 shared-source-item association、item/source/boundary/worklist、
33LB/env retention、determinism、default-deny mutation を検証する。

production runner/installer、Typed/Resolved/Core field、`.miz`、expectation、
trace、active result、diagnostic、metadata count、coverage は変更しない。
Task-261--264、generic/complete Core 33、Core 34/35/36 semantics、parameter/
argument、checker obligation conversion、proof/acceptance、GeneratedOrigin、
C4C8、snapshot、`MT10-CIR-TE`、diagnostic、Task277B は deferred/zero-credit。

## Affected artifacts and audit impact

source は `crates/mizar-core/src/elaborator.rs` と既存 Task-260 private test
leaf だけ。derived docs はこの paired contract、paired Core owner docs/
audits/ledger、paired mizar-test harness/bilingual audit、central coverage
audit に限定する。checker API/docs は変更しない。central audit は
zero-credit mapping と follow-up narrowing だけを記録する。

freeze baseline は `elaborator.rs` `19986 / 741842`、SHA-256
`82971830bd539f184a69675ac502aa317be3f7ebc3ffaab118b07870444ba161`、
Task-260 test leaf `1674 / 61207`、SHA-256
`af20ef00e78656f94f2cae4c410c29d804e0b9b655c47615f36ae60bc2340fa3`。
contract trees は `111/111 -> 112/112`。private tests は正確に二つ、Core
tests は `163`、mizar-test は `634 -> 636`、metadata は `137` の予定。
protected source/expectation/trace/stash hash は英語正本の値から不変とする。

entry HEAD は `de42b58f7322128566326c8ee1d3d1e9a5fe4d77`、actual
`origin/main` は `a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c`、divergence
`0/2`。fetch/push/stash mutation/metadata repair を禁止する。

## Review, verification, and exit

Independent な source 前 spec/equivalence と bilingual/boundary review、続く
source 後の test-sufficiency、implementation、source/docs/API review は、finding-
specific repair 後にすべて no findings で完了した。Focused Task260 test、Core/
mizar-test lint、fmt、offline metadata、warnings-denied Clippy、doctest を含む
all-feature test、protected hash/count、`git diff --check` は pass した。Exact
task-only commit と clean postcommit inventory は pending であり、fresh postcommit
inventory が successor を選択する。

Exit は autonomous hard gate `9/9`、親 score `90/100` 以上、exact task-only commit
state、protected invariance、Task277B not-ready/zero-credit、fresh read-only
successor inventory を記録する。

## Completion evidence

Standalone producer と exact Task260 private consumer は complete。Final source
measurement は `elaborator.rs` `20805 / 775898`、SHA-256
`b8ca96a9ca86078b664a2f6f2581f45f820f13b9dff20ee624adbb32e04aa22e`、Task260 test
leaf `2114 / 78646`、SHA-256
`79d16c928cda605ff210166dee8d13888b33de5b0e8cb8475207558cc59a97fd` である。
Paired task-contract tree は exact `112/112`、Core library test は `163`、mizar-
test library test は `636` (`634 + 2`)、metadata test は `137` である。

Pre-source specification/equivalence review は obligation ownership、`Equals`/
`Means` correctness ownership、Task248 local-context profile の findings を発見
した。修正後の final re-review は no findings。Pre-source bilingual/boundary review
も final no findings。Post-source test-sufficiency review は status/order evidence
不足を発見し、`InvalidStatus` assertion を追加して sealed deterministic-order
test を引用した後の re-review は no findings。Implementation first/final review は
no findings。Source/documentation/API review は missing public-enum rows と invalid
JA marker/link を発見したが、修正後の Core lint `12/12`、mizar-test lint `15/15`、
re-review は no findings である。

Focused Task260 Core-context test は `2/2`、Core lint は `12/12`、mizar-test lint は
`15/15`、metadata は `137/137` pass。`cargo fmt --all -- --check`、offline Cargo
metadata、`cargo clippy --all-targets --all-features -- -D warnings`、doctest を含む
`cargo test --all-features`、`git diff --check` も pass。Protected Task260、Task259、
Task248 Profile-A、reserve、C4C7、trace hash はすべて frozen contract と一致し、
protected stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4` は不変である。

Parent hard gate は `9/9`。Valid uncapped score は `99/100`：specification `20/20`、
test contract `19/20`、traceability `15/15`、implementation `15/15`、design/source
synchronization `10/10`、boundary discipline `10/10`、verification `5/5`、handoff
`5/5`。No cap。Task261--264 owner family、generic/complete Core33 inventory、Core34/
35/36 types/terms/definition bodies、parameter/argument transport、checker obligation
conversion、proof/acceptance、`GeneratedOrigin`、C4C8 composition、snapshot、
`MT10-CIR-TE`、diagnostic、Task277B は deferred のままで、Task277B は not-ready/
zero-credit、Task260 correctness は checker-owned のままである。

Report-only `repo_metadata_conflict` は継続する。Precommit `HEAD` は
`de42b58f7322128566326c8ee1d3d1e9a5fe4d77`。Task 中の外部 remote-tracking-ref
update により、actual `origin/main` は entry 値
`a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c` (`0/2`) から同じ
`de42b58f7322128566326c8ee1d3d1e9a5fe4d77` (`0/0`) へ変化し、reflog は
`2026-08-31T16:54:05+09:00` の `update by push` を記録する。この agent は fetch、
push、stash mutation、metadata repair を行っていない。Exact task-only commit は
pending。Fresh postcommit read-only inventory が successor を選び、この contract は
選択しない。
