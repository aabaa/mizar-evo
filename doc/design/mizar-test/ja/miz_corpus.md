# Module: miz_corpus

> Canonical language: English. English canonical version: [../en/miz_corpus.md](../en/miz_corpus.md).

## 目的

この module は large `.miz` corpus を構築・維持する strategy を定義する。

corpus は implementation asset である。syntax、type behavior、cluster behavior、overload behavior、proof rejection、dependency fingerprints、deterministic artifact output を固定するために使う。

## Corpus Classes

| Class | Purpose | Ownership |
|---|---|---|
| handwritten minimal cases | focused parser/type/cluster/proof behavior | developers が review |
| migrated examples | existing Mizar-like material 由来の realistic source patterns | acceptance 前に review |
| generated `.miz` | grammar and semantic combinations の broad coverage | stored seeds 付きで generate |
| fuzz-minimized reproducers | fuzz failures 由来の permanent regression cases | minimization 後に commit |
| bug regressions | fixed bugs and soundness failures を保護 | 可能なら issue/PR に link |
| stress/integration articles | large-file and incremental rebuild behavior | stability を review |

## Growth Targets

| Stage | Target |
|---|---|
| early module development | 100-300 `.miz` files |
| evo2 alpha | 300-1,000 `.miz` files |
| evo2 beta / release candidate | 1,000-5,000 `.miz` files |
| mature ecosystem | generated/fuzz/property corpora を含む 500,000-1,000,000 LOC equivalent |

growth は shallow pass tests の大量追加より、kernel 近傍の high-signal fail/soundness tests を優先する。

## File Size Guidelines

| Purpose | Lines |
|---|---:|
| parser test | 5-30 |
| type test | 10-50 |
| cluster test | 20-80 |
| theorem test | 30-150 |
| integration test | 100-300 |
| stress test | 500-1,000 |

validation はこれらの範囲を upper-bound review gate として扱う。多くの
fail/soundness cases は意図的に小さいため、短い minimal regression は diagnostic
なしで許可する。Oversized generated `.miz` files は `tests/stress/` 配下でない限り
error とし、oversized handwritten `.miz` files は warning とする。

## Generation Policy

generated `.miz` files は次を記録する。

- generator name and version
- seed
- generation profile
- expected outcome
- minimization status
- metadata schema version

generated、fuzz、property sidecars はこの provenance を `[origin]` に記録する。
metadata-only handoff anchors は crate-local test family または harness handoff を
generator とし、`generator_version = "handoff"`、fixture の stable phase/name を seed
としてよい。`origin.expected_outcome` は harness sidecar outcome を mirror する。
すべての fuzz seeds は `origin.original_failure_category` を記録する。promoted fuzz
failures では、その original fuzz failure class が executable `failure_category` と
一致しなければならない。

generated tests は coverage を増やす、bug を再現する、または stable stress case として機能する場合にのみ commit する。bulk generated corpora は minimize または promote されるまで default fast test set の外に置いてよい。

## Review Rules

corpus additions は次を review する。

- stable expected outcome
- deterministic diagnostics and snapshots
- test execution order への hidden reliance がないこと
- fail/soundness regressions としての minimality
- clear domain placement and naming

generated sidecars は stress cases として `tests/stress/` 配下に置く場合を除き
`tests/generated/` 配下に置く。fuzz seeds は `tests/fuzz/`、property seeds は
`tests/property/` 配下に置く。Unminimized generated/fuzz/property seeds は default
`fast` profile の外に置く。metadata-only fuzz handoff seeds は `fuzz_regression`
profile を使う。Stress cases は `profiles = ["stress"]` を使い、同時に `fast` に
opt in しない。

## Tests

key scenarios:

- generated tests は stored seed metadata から reproduce できる
- minimized fuzz reproducers は original failure category を保持する
- corpus manifest は domain ごとの pass/fail ratios を数える
- stress tests は要求されない限り default fast runs から除外される

## Constraints and Assumptions

- `.miz` corpus files は long-lived compatibility inputs である。
- Fail tests は current compiler behavior に合わせて loosen してはならない。
- soundness regression case は architecture-level review なしに削除しない。

## Checker Task 263 frozen corpus increment

separate docs prerequisite後、Task 263はcanonical-derived pass source/sidecar pair
`pass_type_elaboration_structure_definition_payload_001`をexactly 1件追加できる。
sourceはfrozen 320-byte/final-LF text、SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`である。
sidecarはpass/type_elaboration/type_check、public diagnostics/payloads empty、new
Task-263 trace ref 1件である。

existing mixed mode/structure-definition failure pairと全parser/resolver structure
fixtureはbyte-identicalに保つ。docs prerequisiteはcorpus fileを追加せず、fresh
inventory後のimplementationだけがcases `425 -> 426`、pass `232 -> 233`をprojectする。

## Checker Task 263 active corpus increment

sole new pairは`pass_type_elaboration_structure_definition_payload_001.miz`とmatching
sidecarである。sourceはexact 320 bytes/16 lines、SHA-256は
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`、sidecar hashは
`d82c8d3102ea34fdb4a32792167c4b109b96b9c05265d3f04e6310278178e8ac`。
existing `.miz`/expectationはbyte-identicalで、cases `426`、pass/fail `233/193`、
active type `203`である。

## Task 257C4C0 frozen corpus increment

[C4C0 contract](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md)は
Chapter 13 nested-comprehension outer-generator capture oracle用implemented
handwritten pass pair 1件を`tests/miz/pass/types/`でownする。Pair inventoryは
`344/344`であるがinactiveであり、current parser/semantic/route/warning-error/Task277B creditは
0。Exact source bytes/hash/sidecarはcontract ownerであり、local-lookalike/builtin-set
variantは禁止。

## Task 257C4C1 explicit-import corpus repair

[C4C1 contract](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md)は
existing oracleをexact 164-byte explicit-import sourceへin-place update済みで、inactive
sidecar noteだけを更新する。Separate 140-byte module
`crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz`はseparate
definition blocksでordered `Element`/`NAT` lexical shapesをdefineする。Crate-local
testdataでordinary corpus payloadではなくsidecarなし。Corpus pairsは`344/344`のまま、
active cases/outcomes/routes/warnings-errors/capture/Task277B creditは不変。
Measured file hashesとunchanged inventoriesはcontractの[precommit completion checkpoint](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#precommit-implementation-completion-checkpoint)に一度だけrecordする。

Independent source-doc consistency/bilingual/boundary reviewは**NO FINDINGS**。
Independent final-quality reviewも**NO FINDINGS**で、canonical [precommit completion
checkpoint](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#precommit-implementation-completion-checkpoint)からlinkする。
同historical review-time checkpointにrecordしたexact staging/cached reviewも
**NO FINDINGS**。
Task-only commit、post-commit proof、accepted fresh-inventory STOPはlanguage-local
[historical postimplementation checkpoint](../../task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#historical-immediate-postimplementation-pre-closure-checkpoint)でclosed。
Inactive zero-credit corpus dispositionは不変。

## Task 257C4C7 Frozen Corpus Increment

[C4C7 contract](../../task_contracts/ja/TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md)は
`tests/miz/pass/types/`のexact new inactive pair 1件をownする。193-byte sourceは
independently specifiedなmultiple-generator/nested-capture shapeを組み合わせ、inner
mapper `[x, y]`がexact 2件のresolved outer generatorを参照しinner `z`はlocal。
Sidecarはmembership/cardinalityだけをassertし、order/execution/Core/Task277B
creditを明示的に付与しない。Completed incrementはcorpus pairを
`344/344 -> 345/345`へ変更し、existing one-capture pairをbyte-identicalに保つ。
Exact completion evidenceはlinked contractがownする。

## 2026年9月 監査1 意味論ブリッジ・オラクル増分

2026年9月の監査ウェーブで、仕様第3〜20章にわたる手書き非活性オラクル
120ペア（pass 60 / fail 60、要求103行）を章対応のpass/failサブディレクトリ
（[layout.md](./layout.md) 参照）に追加した。全ペアは Task 257C4C0 の
非活性シード規約（`active_*` タグなし・現行クレジットなし）に従う。
対応表と規約・仕様発見事項は
[semantic_bridge_corpus_map.md](../en/semantic_bridge_corpus_map.md)、
フロントエンドギャップにブロックされる29ソースの台帳は
[semantic_bridge_frontend_gaps.md](../en/semantic_bridge_frontend_gaps.md)
と `tests/coverage/audit1_frontend_gaps.tsv` が所有する（うち1件はパーサ
panicを再現するため、G5解消までコーパス全走査ツールは全 `.miz` のパース
可能性を仮定してはならない）。コーパスペアは `345/345 -> 465/465`、発見
ケースは `430 -> 550` に増加。

## 複数引数 Binder の受入テスト

[§13.10](../../../spec/ja/13.term_expression.md#1310-関数binder-) と [§20.5.3](../../../spec/ja/20.algorithm_and_verification.md#2053-反射した構文) に対するテスト設計。

以下は将来のテスト意図を固定するもので、現時点の実装成功を主張しない。段階別fixtureには実際の生成側を要し、既存テスト・期待値は変更しない。

| ID / 段階 | 必要な成功・拒否の観測 |
|---|---|
| MB1 / 構文 | 1・2・3・4引数、最弱優先順位・右入れ子、本体/全体のqua、適用・制限、カンマ区切り呼出し内の使用。空・重複・末尾カンマ・括弧なし複数頭部・型付き・分解頭部を拒否する。 |
| MB2 / 形成 | REAL×REALとNAT×REALの推論。明示した認定済み直積期待型から因子を選び、狭い因子には全入力の型証明を要する。担体不足・因子の型不整合・直積でない期待型を、別領域への切替や等式からの因子推測で救済しない。 |
| MB3 / 空領域 | 明示した空因子は空共域の場合も空関数を与え、値・環境を生成しない。他因子が空でも因子不足はエラーとし、未対応本体形を拒否する。非空領域と空共域の組合せを拒否する。 |
| MB4 / スコープ・義務 | 新UIDは外側の値・個別仮定を引き継がず、外側の型依存・自由IDを保持する。利用不能な宣言、新頭部依存の領域/型、未証明の本体/適用/値域条件を拒否し、制限で形成失敗を救済しない。 |
| MB5 / 意味 | 2引数f.[a,b]、3引数f.[[a,b],c]、4座標の組表現が各点の意味式に従い、全座標を同時更新する。カリー化関数・単項の組引数・複数頭部を区別し暗黙変換しない。 |
| MB6 / 捕捉 | 表示名と独立に引数数を照合し、元の各座標型・順序、実領域、外側/拡張文脈を空領域でも保持する。引数数不一致はマッチしない。 |
| MB7 / 再構成 | 未変更の往復は構造的に等しく、共域の明示変更にも値域証明を要する。Functionでは保存領域の等しさ、PartFuncでは包含を要求し拡大しない。順序変更・不完全/混合グループ・C不足を拒否し、同UIDで既定共域が異なる場合も区別する。 |
| MB8 / 構造・R2 | 子の順序・重複、サイズ漸化式、全頭部UIDのFVからの除去、組である座標と別々の座標の区別、決定的な内部衝突の改名、利用不能な自由宣言の拒否は§20.5.3に従う。 |
| MB9 / 認定 | 認定された組の構成/分解と関数構成により通常/反射の意味対応を示す。ライブラリ/使用箇所の証明不足や同綴りの別定義から、公理や受理を生成しない。 |
| MB10 / 後続の微分 | REAL×REAL上の(x,y) ↦ x*y+sin yについて、第1座標はy、第2座標はx+cos yを返す。他座標の各環境で全実数への更新可能性を要求し、全頭部ID・順序を保つ。将来のライブラリテストであり、偏微分APIの定義ではない。 |
