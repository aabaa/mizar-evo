# mizar-vc TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

残るモジュール仕様は、それを引用する実装タスクより前に、専用の仕様タスクが
（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割に従う。
この crate はアーキテクチャ 07、16、18、19 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| vc_ir | `vc_ir.md`（task 2） | `src/vc_ir.rs` | [x] |
| generator | `generator.md`（task 5） | `src/generator.rs` | [~] |
| discharge | `discharge.md`（task 10） | `src/discharge.rs` | [ ] |
| dependency_slice | `dependency_slice.md`（task 13） | `src/dependency_slice.rs` | [ ] |

`mizar-vc` はパイプライン phase 11-12 を実装する。入力は `CoreIr` と
`ControlFlowIr`、出力は prover 非依存の `VcIr` であり、外部 prover の実行に
先立って決定的な pre-ATP discharge が証拠を生産する。Mizar 側の義務生成と
prover 側の翻訳の境界であり、`VcId` を割り当てる唯一の場所である。各
obligation seed は正確に 1 回だけ intake-accounted され、concrete VC cardinality
を明示的に記録する。`mizar-atp` は `NeedsAtp` 状態の正準 `VcIr` のみを受け取る。

依存順序: `vc_ir` データ → seed 取り込み → `generator`（定理、定義、
registration-style correctness、アルゴリズムの VC）→ 正規化/状態 →
`discharge` → `dependency_slice`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` と `mizar-core`（`CoreIr`、`ControlFlowIr`、
binder ライブラリ、obligation seed）に依存する。生成タスクは `mizar-core`
task 18（seed の受け渡し）にゲートされる。アーキテクチャ:
[07.vc_generation.md](../../architecture/ja/07.vc_generation.md)、
[18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。
crate 所有権: [internal 07](../../internal/ja/07.crate_module_layout.md)。

## 解決済みおよび保留中の決定

- **ControlFlowIr の所有権: internal 07 により解決済み。** `mizar-core` が
  `ControlFlowIr` を構築する（phase 10）。この crate はアルゴリズム VC の
  ためにそれを消費し、決して変更しない。
- **`VcId` の割り当て: アーキテクチャ 07 により解決済み。** `VcId` を
  割り当てるのは phase 11 のみである。seed は正確に 1 回 intake-accounted
  され、task 8 は no-VC / one-VC / expanded の明示 mapping を強制する。
- **discharge の計算上限: 未解決。task 11 で解決する。** pre-ATP discharge
  は同一のソース、依存、ツールチェーン、ポリシー、計算上限に対して決定的で
  なければならない。上限モデル（ステップ数予算、再帰深さ、数値範囲）と
  その設定面を決め、`discharge.md` に記録する。
- **discharge 証拠の検証範囲: 未解決。`mizar-proof` task 6 が所有する。**
  task 12 の discharge 証拠を kernel が再生するか、ポリシーに従う
  決定的な built-in 証拠として受理するか。この crate はどちらの場合でも
  証拠が再生可能であることを保証する。トップレベルに登録済み。
- **diagnostics レコード: `mizar-resolve` の決定に従う**
  （`mizar-diagnostics` 採用時期）。トップレベルに登録済み。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-vc` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### VC IR と seed の取り込み

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session` と `mizar-core` に依存する workspace メンバー
     `mizar-vc` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-core` task 1。仕様: アーキテクチャ 07。

2. **仕様: `vc_ir.md`。** [x]
   - `VcIr` のデータ形状仕様を執筆する（英語と日本語、コードなし）:
     `VcId`、`VcKind`、`LocalContext`、シンボリックな `PremiseRef`、goal
     論理式、`ProofHint`、VC 状態モデル（`NeedsAtp` とポリシー状態を
     含む）、seed accounting と concrete cardinality mapping の規則、
     architecture-22 の
     `ObligationAnchor` 契約。anchor 仕様には anchor-ready な局所 proof /
     program path、label role、正規化された semantic origin、source/core
     provenance を記録し、`VcId` と source range は snapshot-local に
     とどめる。task 20 は、discharge 証拠、依存スライス、決定性カバレッジが
     揃った後の編集横断 reuse 実装と regression gate を所有する。
   - 依存: 1。仕様: アーキテクチャ 07「VC IR」「VC Status」、
     [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

3. **`vc_ir` データ形状の実装。** [x]
   - task 2 に従って `VcIr`、状態、コンテキスト構造を実装し、決定的
     debug レンダリングを加える。
   - テスト: 構築のラウンドトリップ。premise 参照はシンボリックのまま。
     レンダリングの安定性。
   - 依存: 2。仕様: `vc_ir.md`。

4. **obligation seed の取り込み。** [x]
   - `mizar-core` の seed 受け渡し（定理本体、correctness condition、
     checker の initial obligation、アルゴリズム契約）を決定的な seed
     テーブルへ取り込む（アーキテクチャ 07 Step 2）。
   - テスト: seed カバレッジのフィクスチャ。重複 seed の拒否。決定的な
     順序。
   - 依存: 3、`mizar-core` task 18。仕様: `vc_ir.md`（seed の節）。

### 生成（phase 11）

5. **仕様: `generator.md`。** [x]
   - 生成の仕様を、名前付き節とともに執筆する（英語と日本語、コード
     なし）: ローカルコンテキストの構築、定理/定義の VC（Step 3）、
     利用可能な場合の explicit registration/redefinition/reduction correctness
     seed、構造化制御フロー上のアルゴリズム VC（Step 4）、制御された定義
     unfold、正規化/分類（Step 5）。
   - 依存: 2。仕様: アーキテクチャ 07「Step 3」〜「Step 5」、
     [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)、
     [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)、
     [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

6. **定理、定義、generated core、registration-style correctness の VC。** [ ]
   - 定理の証明ステップ、引用、定義の correctness condition から、明示的な
     ローカルコンテキストを保持した VC を生成する。non-emptiness、sethood、
     Fraenkel membership axiom の explicit core-seed obligation を生成する。
     checker-initial または core correctness seed が registration、redefinition、
     reduction correctness を明示的に表す場合は、それを registration-style correctness VC として保持
     する。その explicit payload が利用できない場合は、registration activation
     や proof acceptance を捏造せず external/deferred gap として分類する。
   - テスト: 義務種別ごとの VC フィクスチャ。generated core seed fixture。
     ローカルコンテキストは明示的であり、グローバル状態に暗黙に依存しない。
     利用不能な explicit registration payload は deferred として記録する。
   - 依存: 4、5。仕様: `generator.md`（定理/定義の節）。

7. **アルゴリズムの VC。** [ ]
   - 構造化制御フローに従って `ControlFlowIr` から VC を生成する: 契約、
     不変条件（導入/保存）、assert、ghost 規則、停止性測度。
   - テスト: 構文ごとの VC フィクスチャ（`while`、`if`、`match`）。
     不変条件の導入/保存の組。停止性 VC が測度を参照する。レビュー監査由来の
     algorithm fixture として、old-state assignment、field update の alias
     identity、`not C` を獲得しない `break` exit、`continue`/decreasing
     check、`downto` と `step` の range loop、ghost-only `Pick` erasure も
     含める。
   - 依存: 6、`mizar-core` task 16。仕様: `generator.md`（アルゴリズムの
     節）。

8. **正規化、分類、`VcId` の割り当て。** [ ]
   - VC を正規化・分類し（Step 5）、決定的な `VcId` を割り当てる。すべての
     seed が正確に 1 回 intake-accounted され、concrete cardinality が no VC /
     one VC / explicit expansion として表され、ほかの場所では id を割り当てない
     ことを強制する。
   - テスト: 実行をまたぐ id の決定性。seed accounting と seed-to-VC mapping の
     フィクスチャ。分類のフィクスチャ。
   - 依存: 7。仕様: `generator.md`（正規化の節）、`vc_ir.md`。

9. **状態とポリシーのモデル。** [ ]
   - VC 状態遷移（open、discharged、`NeedsAtp`、ポリシー割り当て状態）を
     実装し、ATP 行きの義務を消したり弱めたりせずに verifier ポリシーを
     VC に反映する。
   - テスト: 遷移のフィクスチャ。ポリシー状態がコンテキストを落とさない。
   - 依存: 8。仕様: `vc_ir.md`（状態の節）、アーキテクチャ 07「Status and
     Policy Are Reflected in VCs」。

### pre-ATP discharge（phase 12）

10. **仕様: `discharge.md`。** [ ]
    - pre-ATP discharge の仕様を執筆する（英語と日本語、コードなし）:
      Mizar 側で discharge する義務形（決定的または計算ベース）、計算上限
      モデル、説明可能性レコード、「ATP 行きの VC を消したり弱めたりしない」
      規則。
    - 依存: 2。仕様: アーキテクチャ 07「Step 6」「Pre-ATP Discharge Is
      Deterministic and Explainable」、
      [08.reasoning_boundary.md](../../architecture/ja/08.reasoning_boundary.md)。

11. **決定的 discharge エンジン。** [ ]
    - 対応する義務形に対して、決定済みの計算上限つき discharge を実装
      する。計算上限の決定を解決し記録する。
    - テスト: discharge されたフィクスチャがビット同一に再現する。上限
      超過ケースは誤答ではなく安定した診断を生む。
    - 依存: 9、10。仕様: `discharge.md`。

12. **discharge の証拠と説明。** [ ]
    - discharge された各 VC について再生可能な証拠（適用規則、入力、計算
      ステップ）を記録し、診断・artifact・後のポリシーに応じた kernel 側
      検証に備える。
    - テスト: 証拠のラウンドトリップ。discharge された全 VC が証拠を持つ。
    - 依存: 11。仕様: `discharge.md`（証拠の節）。

### 依存スライスとフォローアップ

13. **仕様: `dependency_slice.md`。** [ ]
    - 依存スライスの仕様を執筆する（英語と日本語、コードなし）: 各 VC が
      依存する import 済み事実・registration・定義と、スライスが
      canonical dependency-slice fingerprint、proof reuse、増分再ビルドへ
      供給される方法。依存範囲が不完全または不明な場合は、consumer が
      cache miss を強制できるよう保守的に表現することを明記する。
    - 依存: 2。仕様:
      [18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

14. **依存スライスの計算。** [ ]
    - premise、ローカルコンテキスト、trace 参照から VC ごとの依存スライスを
      決定的に計算する。
    - テスト: スライスのフィクスチャ。未使用の事実の除外。決定的な順序。
    - 依存: 8、13。仕様: `dependency_slice.md`。

15. **stage `proof_verification` のコーパスランナー。** [ ]
    - 編集前に `mizar-test` support を再評価する。active `proof_verification`
      runner と source-to-core extraction seam が存在するなら、
      `tests/miz/{pass,fail}/` のケースをハーネスに接続し、`spec_trace.toml`
      項目を付ける。生成と discharge のケースをシードする。task 7 に列挙した
      algorithm VC のレビュー監査ケースも含める。runner または extraction seam
      がまだない場合は、active fixture を捏造せず、具体的な
      external-dependency reason つきで corpus obligation を deferred 記録する。
    - 依存: 11。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)。

16. **決定性スイート。** [ ]
    - 同一入力が同一の VC 集合、id、順序、状態、スライス、discharge 証拠を
      生むことのプロパティ的検証。
    - 依存: 14。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

17. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用し、所有
      モジュール仕様に決定を記録する。
    - 依存: 14。仕様: 全モジュール仕様。

18. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 17。仕様: 全モジュール仕様と本 TODO。

19. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-vc/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 18。仕様: リポジトリのドキュメント方針。

20. **obligation anchor と編集をまたぐ再利用 identity。** [ ]
    - task 2 の `VcIr` / seed 契約に対する編集横断 reuse 実装を完成させる:
      `ObligationAnchor`、canonical VC fingerprint、local-context fingerprint、
      dependency-slice fingerprint を生成済み obligation へ接続する。
      `VcId`、`SourceRange`、`SurfaceNodeId`、task-local id は snapshot-local な
      証拠にとどめ、編集をまたぐ proof-reuse identity にはしない。
    - テスト: 既存 obligation の前に proof step を挿入すると `VcId` ordering
      は変わるが、anchor、canonical VC fingerprint、local context fingerprint、
      dependency slice fingerprint、compatible verifier policy、選択された
      proof witness hash または deterministic discharge hash が一致する場合に
      限って再利用可能になること。
    - 依存: 2、12、14、16。仕様:
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md),
      [07.vc_generation.md](../../architecture/ja/07.vc_generation.md),
      [18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

21. **architecture-22 フォローアップ監査。** [ ]
    - task 20 の anchor、fingerprint、proof-reuse identity 契約について、
      ソース/仕様対応監査と二言語ドキュメント同期監査を再実行する。
      consumer がこの契約へ依存する前に、残る architecture-22 gap を
      フォローアップタスクとして記録する。
    - 依存: 20。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。

22. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 21。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-vc
cargo clippy -p mizar-vc --all-targets -- -D warnings
```

core 境界やコーパスに触れるタスクでは追加で実行する:

```text
cargo test -p mizar-core
cargo test -p mizar-test
```

architecture-22 の reuse-identity 契約では、該当 crate が workspace に存在し、
task が実際に integration boundary に触れる場合に限り、anchor と proof metadata
の consumer も追加で実行する:

```text
cargo test -p mizar-cache
cargo test -p mizar-proof
```

どちらかの crate がまだ利用できない場合は、placeholder crate を追加せず、その
task の `external_dependency_gap` / `deferred` verification item として分類する。

テストが通ったらここでタスクにチェックを付ける。

## 備考

- `VcIr` は prover 非依存のまま: TPTP/SMT-LIB テキスト、抽象ヒントを超える
  バックエンドプロセス設定、証明書は持たない。
- phase 12 は VC を discharge したりポリシー状態を割り当てたりできるが、
  ATP 行きの VC を消したりコンテキストを弱めたりしてはならない。
  `mizar-atp` は `NeedsAtp` 状態の正準 `VcIr` のみを受け取る。
- premise 参照は ATP 翻訳がエンコーディングを選ぶまでシンボリックのまま。
- discharge の証拠は untrusted な生産物である: 信頼される受理はポリシーに
  従って `mizar-kernel`/`mizar-proof` で行われる。
