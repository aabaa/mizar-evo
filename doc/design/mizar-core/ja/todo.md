# mizar-core TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割に従う。
この crate はアーキテクチャ 06 と 16 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| core_ir | `core_ir.md`（task 2） | `src/core_ir.rs` | [ ] |
| binder_normalization | `binder_normalization.md`（task 4） | `src/binder_normalization.rs` | [ ] |
| elaborator | `elaborator.md`（task 7） | `src/elaborator.rs` | [ ] |
| control_flow | `control_flow.md`（task 14） | `src/control_flow.rs` | [ ] |

`mizar-core` はパイプライン phase 9（elaboration）と phase 10（制御フロー
準備）を実装する。入力は `ResolvedTypedAst`、出力は `CoreIr` と
`ControlFlowIr` である。elaboration は source 形状を保つ最後の境界であり、
core 表現は証明・検証・kernel 検査のために正規化され、soft type は明示的な
型述語を通じてのみ消去される。この crate が所有する binder ライブラリ
（アーキテクチャ 16）は、後に `mizar-kernel` の substitution checker が
再生する表現でもあるため、その不変条件は健全性に関わる。

依存順序: `core_ir` データ → `binder_normalization` → `elaborator`
（Step 1-6）→ `control_flow`。データと binder のタスク（2-6）は checker の
出力を必要とせず、`mizar-checker` の各波と並行して進められる。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session`、`mizar-resolve`（シンボル識別）、
`mizar-checker`（`ResolvedTypedAst`）に依存する。elaboration タスク（8-13）
は `mizar-checker` task 28 にゲートされる。データと binder の基盤はゲート
されない。アーキテクチャ:
[06.elaboration_and_core_ir.md](../../architecture/ja/06.elaboration_and_core_ir.md)、
[16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)。
crate 所有権: [internal 07](../../internal/ja/07.crate_module_layout.md)。

## 解決済みおよび保留中の決定

- **binder 表現: 未解決。task 4 で解決する。**
  [16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)
  の制約の中で具体表現（de Bruijn インデックス、locally nameless、または
  alpha 付き名前）を選び、kernel 再生への含意とともに決定を
  `binder_normalization.md` に記録する。kernel は置換を独立に再検査する
  ため、その検査が線形時間になる表現でなければならない。
- **ControlFlowIr 構築の所有権: internal 07 により解決済み。**
  `mizar-core` が `ControlFlowIr` 構築を含む制御フロー準備（phase 10）を
  所有し、`mizar-vc` はアルゴリズム VC 生成のために `ControlFlowIr` を
  消費する。アーキテクチャ 07 のモジュール一覧はこの分割より古い。
  `control_flow.md`（task 14）が境界を記録する。
- **消去ポリシー: アーキテクチャ 06 により解決済み。** soft type 注釈は
  明示的な型述語と仮定を通じてのみ消去される。`elaborator.md` が消去規則を
  ケースごとに列挙し、elaboration は証明探索も registration の活性化も
  決して行わない。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-core` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### Core IR と binder の基盤

1. **crate の足場と lint 方針のガード。** [ ]
   - `mizar-session`、`mizar-resolve`、`mizar-checker` に依存する workspace
     メンバー `mizar-core` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-checker` task 1。仕様: アーキテクチャ 06。

2. **仕様: `core_ir.md`。** [ ]
   - `CoreIr` のデータ形状仕様を執筆する（英語と日本語、コードなし）:
     `CoreItem`、core の項/論理式、安定した展開境界を持つ
     `CoreDefinitionTable`、`CoreProofTable`、`GeneratedFrom` マーカーを
     持つ `CoreSourceMap`、obligation seed の参照形状。
   - 依存: 1。仕様: アーキテクチャ 06「Interface Definitions」、
     [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

3. **`core_ir` データ形状の実装。** [ ]
   - task 2 に従って core の item/項/論理式/証明構造とソースマップを
     実装し、決定的 debug レンダリングを加える。
   - テスト: 構築のラウンドトリップ。item から到達可能なすべてのノードが
     ソースへ写像されるか `GeneratedFrom` を持つ。レンダリングの安定性。
   - 依存: 2。仕様: `core_ir.md`。

4. **仕様: `binder_normalization.md`。** [ ]
   - binder の仕様を執筆する（英語と日本語、コードなし）: 表現の決定
     （根拠と kernel 再生への含意付き）、alpha 同値、捕獲回避置換 API、
     自由変数条件、正規化規則。
   - 依存: 1。仕様:
     [16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)。

5. **binder 表現と置換。** [ ]
   - 選ばれた表現と、core の項/論理式上の捕獲回避置換を実装する。
   - テスト: シャドーイングと捕獲ケースを含む置換フィクスチャ。置換の
     合成則。
   - 依存: 3、4。仕様: `binder_normalization.md`。

6. **alpha 同値と正規化ユーティリティ。** [ ]
   - 決定的な正準形を持つ alpha 同値検査と binder 正規化を実装する。
   - テスト: プロパティテスト（同値の反射・対称・推移性。正規化の冪等性。
     正準形が等しい ⇔ alpha 同値）。
   - 依存: 5。仕様: `binder_normalization.md`。

### elaboration（phase 9）

7. **仕様: `elaborator.md`。** [ ]
   - elaboration の仕様を、実装タスクが引用する名前付き節とともに執筆する
     （英語と日本語、コードなし）: core コンテキスト準備、型/事実の
     lowering とケースごとの消去規則、項/論理式の lowering、展開境界を
     持つ定義の lowering、証明骨格の lowering、アルゴリズムシェルの
     lowering。
   - 依存: 2、4。仕様: アーキテクチャ 06「Step 1」〜「Step 6」。

8. **core コンテキストの準備。** [ ]
   - Step 1 を実装する: 正準シンボル識別、定義境界レジストリ、
     `ResolvedTypedAst` 上の elaboration コンテキスト。
   - テスト: コンテキストのフィクスチャ。生の綴りではなく必ず正準 id。
   - 依存: 3、7、`mizar-checker` task 28。仕様: `elaborator.md`
     （コンテキストの節）。

9. **型と事実の lowering。** [ ]
   - Step 2 を実装する: soft type と型事実を消去規則に従って明示的な
     型述語と仮定へ下ろす。
   - テスト: 各消去規則にフィクスチャがある。黙った消去がない（捨てられる
     注釈はすべて規則により正当化される）。
   - 依存: 8。仕様: `elaborator.md`（消去の節）。

10. **項と論理式の lowering。** [ ]
    - Step 3 を実装する: 挿入された `qua` view を含む解決済みの項と
      論理式を binder 正規化された core 形へ下ろす。
    - テスト: surface 形ごとの lowering フィクスチャ。失敗した意味論
      サイトは明示的なエラーノードのままで、決して有効な core 項に
      ならない。
    - 依存: 9。仕様: `elaborator.md`（項/論理式の節）。

11. **定義の lowering。** [ ]
    - Step 4 を実装する: 安定した展開境界を持つ定義の lowering（先行
      インライン化なし）。correctness condition の本体を含む。
    - テスト: 展開境界のフィクスチャ。定義の unfold は明示的であり、
      偶発的には起こらない。
    - 依存: 10。仕様: `elaborator.md`（定義の節）。

12. **証明骨格の lowering。** [ ]
    - Step 5 を実装する: 証明構造（`proof`/`now`/`per cases`、結論
      ステップ、引用）を thesis 追跡付きの core 証明木へ下ろす。
    - テスト: 証明形ごとの骨格フィクスチャ。thesis 遷移の記録。引用参照は
      シンボリックに保持される。
    - 依存: 11。仕様: `elaborator.md`（証明の節）。

13. **アルゴリズムシェルの lowering。** [ ]
    - Step 6 を実装する: アルゴリズム本体を core item へ下ろす（CFG は
      まだ作らない）。契約と ghost 注釈は phase 10 のために保持する。
    - テスト: シェルのフィクスチャ。ghost/実行時の区別の保持。
    - 依存: 11、`mizar-parser` task 32-34 のカバレッジ。仕様:
      `elaborator.md`（アルゴリズムの節）。

### 制御フロー準備（phase 10）

14. **仕様: `control_flow.md`。** [ ]
    - `ControlFlowIr` の仕様を執筆する（英語と日本語、コードなし）: basic
      block、ローカル束縛テーブル、契約集合、ghost 効果テーブル、停止性
      測度、core→CFG 構築契約。internal 07 による `mizar-vc` との所有権
      境界を記録する。
    - 依存: 2。仕様: アーキテクチャ 06「Step 6」、アーキテクチャ 07
      「Step 1」、[20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

15. **`ControlFlowIr` の構築。** [ ]
    - core のアルゴリズム item から制御フローグラフを構築する: block、
      辺、ローカル束縛情報。
    - テスト: 制御構文ごとの CFG フィクスチャ（`while`、`if`、`match`、
      `break`/`continue`）。決定的な block 順。
    - 依存: 13、14。仕様: `control_flow.md`。

16. **契約、ghost 効果、停止性測度。** [ ]
    - 事前条件、事後条件、assert、不変条件、ghost 効果追跡、停止性測度を
      CFG に取り付ける。
    - テスト: 取り付けのフィクスチャ。ghost 状態が実行時効果テーブルへ
      漏れない。
    - 依存: 15。仕様: `control_flow.md`。

17. **フロー診断。** [ ]
    - CFG 上の use-before-assignment と到達不能コードの診断を実装する。
    - テスト: 診断ごとの pass/fail フィクスチャ。安定した診断順。
    - 依存: 15。仕様: `control_flow.md`、
      [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。

### 強化と横断フォローアップ

18. **obligation seed の受け渡し契約。** [ ]
    - `mizar-vc` が消費する obligation seed 出力を定義・実装する（seed
      のみ。具体的な `VcId` は phase 11 が割り当てる）。定理本体、
      correctness condition、checker の initial obligation、アルゴリズム
      契約を網羅する。
    - テスト: seed カバレッジのフィクスチャ。seed が
      `CoreIr`/`ControlFlowIr` ノードとソース範囲を参照する。
    - 依存: 12、16。`mizar-vc` task 2 と 4 と調整する。仕様: `core_ir.md`
      （seed の節）、アーキテクチャ 06 の制約。

19. **snapshot ダンプとコーパス寄与。** [ ]
    - 決定的な `CoreIr`/`ControlFlowIr` レンダリングを stage
      `type_elaboration` と `proof_verification` のコーパス snapshot
      ベースラインへ接続する。
    - 依存: 12、15。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)、
      [snapshot.md](../../mizar-test/ja/snapshot.md)。

20. **決定性スイート。** [ ]
    - 同一の `ResolvedTypedAst` 入力が同一の core item、binder 番号付け、
      CFG、レンダリングを生むことのプロパティ的検証。
    - 依存: 18。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

21. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用し、所有
      モジュール仕様に決定を記録する。
    - 依存: 18。仕様: 全モジュール仕様。

22. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 21。仕様: 全モジュール仕様と本 TODO。

23. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-core/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 22。仕様: リポジトリのドキュメント方針。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-core
cargo clippy -p mizar-core --all-targets -- -D warnings
```

checker 境界やコーパスに触れるタスクでは追加で実行する:

```text
cargo test -p mizar-checker
cargo test -p mizar-test
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- `CoreIr` はバックエンド中立である: ATP エンコーディングの決定、パーサーの
  trivia、未解決名、surface 限定の詳細は持たない。
- elaboration は証明探索を決して行わず、registration を活性化しない。
  失敗した意味論サイトは明示的なエラーノードまたはスキップされた item の
  まま残る。
- binder ライブラリは健全性に関わる: kernel が置換を独立に再検査するため、
  表現を再生可能に保ち、不変条件をプロパティテストで守る。
- internal 07 により phase 10 はここに属する。`mizar-vc` は
  `ControlFlowIr` を消費し、決して変更しない。
