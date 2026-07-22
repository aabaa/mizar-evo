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
| core_ir | `core_ir.md`（task 2） | `src/core_ir.rs` | [x] |
| binder_normalization | `binder_normalization.md`（task 4） | `src/binder_normalization.rs` | [x] |
| elaborator | `elaborator.md`（task 7） | `src/elaborator.rs` | [x] |
| control_flow | `control_flow.md`（task 14） | `src/control_flow.rs` | [x] |

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

- **binder 表現: task 4 で解決済み。** `binder_normalization.md` は bound
  variable に de Bruijn index、free / schematic / generated variable に安定
  `CoreVarId` を使う locally nameless representation を選ぶ。kernel は置換を
  独立に再検査する。選んだ表現は、明示的 freshness witness と guard side
  condition により線形 replay を保つ。
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

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session`、`mizar-resolve`、`mizar-checker` に依存する workspace
     メンバー `mizar-core` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-checker` task 1。仕様: アーキテクチャ 06。

2. **仕様: `core_ir.md`。** [x]
   - `CoreIr` のデータ形状仕様を執筆する（英語と日本語、コードなし）:
     `CoreItem`、core の項/論理式、安定した展開境界を持つ
     `CoreDefinitionTable`、`CoreProofTable`、`GeneratedFrom` マーカーを
     持つ `CoreSourceMap`、obligation seed の参照形状。`mizar-vc` が消費する
     anchor-ready な局所 proof / program path、label、正規化された semantic
     origin、source/core provenance を含める。
   - 依存: 1。仕様: アーキテクチャ 06「Interface Definitions」、
     [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

3. **`core_ir` データ形状の実装。** [x]
   - task 2 に従って core の item/項/論理式/証明構造とソースマップを
     実装し、決定的 debug レンダリングを加える。
   - テスト: 構築のラウンドトリップ。item から到達可能なすべてのノードが
     ソースへ写像されるか `GeneratedFrom` を持つ。レンダリングの安定性。
   - 依存: 2。仕様: `core_ir.md`。

4. **仕様: `binder_normalization.md`。** [x]
   - binder の仕様を執筆する（英語と日本語、コードなし）: 表現の決定
     （根拠と kernel 再生への含意付き）、alpha 同値、捕獲回避置換 API、
     自由変数条件、正規化規則。
   - 依存: 3。仕様:
     [16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)、
     `core_ir.md`。

5. **binder 表現と置換。** [x]
   - 選ばれた表現と、core の項/論理式上の捕獲回避置換を実装する。
   - テスト: シャドーイングと捕獲ケースを含む置換フィクスチャ。置換の
     合成則。置換 coverage を完了扱いする前に、レビュー監査由来の
     `defpred P(n be Nat) means n < m` shadowing ケースと、捕獲を生む不正な
     置換のリグレッションを含める。
   - 依存: 3、4。仕様: `binder_normalization.md`。

6. **alpha 同値と正規化ユーティリティ。** [x]
   - 決定的な正準形を持つ alpha 同値検査と binder 正規化を実装する。
   - テスト: プロパティテスト（同値の反射・対称・推移性。正規化の冪等性。
     正準形が等しい ⇔ alpha 同値）。
   - 依存: 5。仕様: `binder_normalization.md`。

### elaboration（phase 9）

7. **仕様: `elaborator.md`。** [x]
   - elaboration の仕様を、実装タスクが引用する名前付き節とともに執筆する
     （英語と日本語、コードなし）: core コンテキスト準備、型/事実の
     lowering とケースごとの消去規則、項/論理式の lowering、展開境界を
     持つ定義の lowering、証明骨格の lowering、アルゴリズムシェルの
     lowering。
   - 依存: 2、4。仕様: アーキテクチャ 06「Step 1」〜「Step 6」。

8. **core コンテキストの準備。** [x]
   - Step 1 を実装する: 正準シンボル識別、定義境界レジストリ、
     `ResolvedTypedAst` 上の elaboration コンテキスト。
   - テスト: コンテキストのフィクスチャ。生の綴りではなく必ず正準 id。
   - 依存: 3、7、`mizar-checker` task 28。仕様: `elaborator.md`
     （コンテキストの節）。

9. **型と事実の lowering。** [x]
   - Step 2 を実装する: soft type と型事実を消去規則に従って明示的な
     型述語と仮定へ下ろす。
   - テスト: 各消去規則にフィクスチャがある。黙った消去がない（捨てられる
     注釈はすべて規則により正当化される）。
   - 依存: 8。仕様: `elaborator.md`（消去の節）。

10. **項と論理式の lowering。** [x]
    - Step 3 を実装する: 挿入された `qua` view を含む解決済みの項と
      論理式を binder 正規化された core 形へ下ろす。
    - テスト: surface 形ごとの lowering フィクスチャ。失敗した意味論
      サイトは明示的なエラーノードのままで、決して有効な core 項に
      ならない。stable choice と comprehension のレビューケースも含める:
      stable `the T` は生成された `Apply(choice_T(...))` シンボルへ lower し、
      Fraenkel comprehension は必要な sethood evidence を保持する。
    - 依存: 9。仕様: `elaborator.md`（項/論理式の節）。

11. **定義の lowering。** [x]
    - Step 4 を実装する: 安定した展開境界を持つ定義の lowering（先行
      インライン化なし）。correctness condition の本体を含む。
    - テスト: 展開境界のフィクスチャ。定義の unfold は明示的であり、
      偶発的には起こらない。export された定義 choice は generated dependency
      として記録し、後続の明示 unfold surface が definition-owned symbol を
      再生成せず再利用できるようにする。
    - 依存: 10。仕様: `elaborator.md`（定義の節）。

12. **証明骨格の lowering。** [x]
    - Step 5 を実装する: 証明構造（`proof`/`now`/`per cases`、結論
      ステップ、引用）を thesis 追跡付きの core 証明木へ下ろす。
    - テスト: 証明形ごとの骨格フィクスチャ。thesis 遷移の記録。引用参照は
      シンボリックに保持される。invalid citation、missing / wrong-owner proof item、
      malformed error root、active path formula、terminal-goal back-reference、external
      dependency citation を覆う。定理/補題命題が自身の stable choice シンボルを所有する
      ケースも含める。
    - 依存: 11。仕様: `elaborator.md`（証明の節）。

13. **アルゴリズムシェルの lowering。** [x]
    - Step 6 を実装する: アルゴリズム本体を core item へ下ろす（CFG は
      まだ作らない）。契約と ghost 注釈は phase 10 のために保持する。
    - テスト: シェルのフィクスチャ。ghost/実行時の区別の保持。実行可能な
      algorithm 文中の `the` サイトが `Pick` 束縛へ lower されることと、
      ghost-only `Pick` サイトが後続 erasure 用に印付けされることを含める。
    - 依存: 12。`mizar-parser` task 32-34 のカバレッジはこの task では external な
      source-to-checker extraction gap のままにする。仕様: `elaborator.md`（アルゴリズムの節）。

### 制御フロー準備（phase 10）

14. **仕様: `control_flow.md`。** [x]
    - `ControlFlowIr` の仕様を執筆する（英語と日本語、コードなし）: basic
      block、ローカル束縛テーブル、契約集合、ghost 効果テーブル、停止性
      測度、core→CFG 構築契約。internal 07 による `mizar-vc` との所有権
      境界を記録する。
    - 依存: 2、13。仕様: アーキテクチャ 06「Step 6」、アーキテクチャ 07
      「Step 1」、[20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

15. **`ControlFlowIr` の構築。** [x]
    - core のアルゴリズム item から制御フローグラフを構築する: block、辺、
      ローカル束縛情報、最小 context、statement placement / source map、
      valid flow に必要な structural diagnostic。
    - テスト: straight-line flow、制御構文ごとの CFG フィクスチャ
      （`while`、`if`、`match`、`break`/`continue`）、deterministic block
      order / debug rendering、local/source-map fidelity、fallthrough、break、
      loop carry、unreachable-join context regression。
    - 依存: 13、14。仕様: `control_flow.md`。

16. **契約、ghost 効果、停止性測度。** [x]
    - 事前条件、事後条件、assert、不変条件、ghost 効果追跡、停止性測度を
      CFG に取り付ける。
    - テスト: 取り付けのフィクスチャ。ghost 状態が実行時効果テーブルへ
      漏れない。
    - 依存: 15。仕様: `control_flow.md`。

17. **フロー診断。** [x]
    - CFG 上の use-before-assignment と到達不能コードの診断を実装する。
    - テスト: 診断ごとの pass/fail フィクスチャ。安定した診断順。
    - 依存: 15。仕様: `control_flow.md`、
      [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。

### 強化と横断フォローアップ

18. **obligation seed の受け渡し契約。** [x]
    - `mizar-vc` が消費する obligation seed 出力を定義・実装する（seed
      のみ。具体的な `VcId` は phase 11 が割り当てる）。既存の定理本体、
      correctness condition、checker の initial obligation、generated /
      deferred / error traceability row、flow-derived algorithm contract、
      termination、ghost-erasure site を網羅する。seed は anchor-ready な
      局所 proof / program path、label、正規化された semantic origin、
      source/core provenance、局所 CFG site metadata を持つが、編集をまたぐ
      reuse identity は `mizar-vc` に委ねる。
    - テスト: seed カバレッジのフィクスチャ。seed が
      `CoreIr`/`ControlFlowIr` ノード、ソース範囲、局所 proof / program
      path、label、provenance を参照する。
    - 依存: 12、16。`mizar-vc` task 2 と 4 と調整する。仕様: `core_ir.md`
      （seed の節）、アーキテクチャ 06 の制約。

19. **snapshot ダンプとコーパス寄与。** [x]
    - `mizar-test` が該当 snapshot runner と source-derived payload seam を
      公開するまで、stage `type_elaboration` と `proof_verification` の決定的
      `CoreIr`/`ControlFlowIr` レンダリング用 corpus snapshot baseline を
      deferred として記録する。
    - 依存: 12、15。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)、
      [snapshot.md](../../mizar-test/ja/snapshot.md)。

20. **決定性スイート。** [x]
    - 同一の public-API core fixture が同一の core item、binder 番号付け、
      CFG、obligation-seed handoff、レンダリングを生むことの property-style 検証。
      完全な source-derived `ResolvedTypedAst` determinism は source-to-checker
      extraction ができるまで deferred。
    - 依存: 18。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

21. **公開 enum の前方互換性ポリシー。** [x]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用し、所有
      モジュール仕様に決定を記録する。現在の結果: すべての public
      `mizar-core` enum は downstream forward-compatible であり、
      `#[non_exhaustive]` を維持する。crate が所有する exhaustive exception はない。
    - 依存: 18。仕様: 全モジュール仕様。

22. **ソース/仕様対応監査。** [x]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。現在の結果:
      `source_spec_audit.md` が item-level public API group、現在の public
      surface に `source_undocumented_behavior` が残っていないこと、external/deferred
      seam の CORE-AUDIT follow-up record を記録する。
    - Tests: 英日 module section、public item mention、gap id/class sync、non-empty
      follow-up detail を lint-policy audit guard で検査する。
    - 依存: 21。仕様: 全モジュール仕様と本 TODO。

23. **二言語ドキュメント同期監査。** [x]
    - `doc/design/mizar-core/en/` の各英語正本と日本語版を比較し、内容を
      同期する。現在の結果: `bilingual_sync_audit.md` が現在の paired file set、
      言語固有の許容差分、blocking bilingual documentation drift がないことを記録する。
    - Tests: docs-only diff checks。
    - 依存: 22。仕様: リポジトリのドキュメント方針。

24. **module 境界リファクタリング gate。** [x]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
      現在の結果: `module_boundary_audit.md` は大きいが cohesive な module-owned
      source file と、closeout 前に source split を要求する current
      review-bottleneck がないことを記録する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 23。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。
    - Tests: Rust source を移動しないため docs-only diff checks。

25. **closeout report と quality review。** [x]
    - 英日 `crate_exit_report.md` pair を追加し、ledger に task commit hash を
      backfill し、closeout bilingual audit row を解消し、broad verification を実行し、
      final quality review score を記録する。
    - Tests: `cargo fmt --check`、
      `cargo clippy --all-targets --all-features -- -D warnings`、
      `cargo test`、`git diff --check`、staged `git diff --cached --check`。
    - 依存: 24。仕様:
      [autonomous_crate_development.md](../../autonomous_crate_development.md)、
      本 TODO、crate exit criteria。

### テンプレートエンコーディング監査フォローアップ(2026-07-05)

[template_encoding_audit.md](./template_encoding_audit.md) は spec 18.10 の
テンプレート FOL エンコードを監査し、所見 F1-F8 と
`tests/miz/fail/templates/` 配下の 4 seed の reject-first encoding corpus
(`spec.en.18.templates.encoding_soundness.semantic`)を記録した。F1-F6 と
F8 の spec 本文は同一変更(`cef7e109`: spec 03、05、13、17、18)で修正済み
であり、task 26 が残っていた F7 spec decision を記録する。task 26 後に
ここへ残るのは elaborator 実装と payload を伴う実行作業である。全所見は
タスクまたは記録済みの処置に対応する:

| 所見 | 処置 |
|---|---|
| F1(structure-view 崩壊) | spec 修正済み。task 27 は explicit-payload elaborator reduct-view lowering を実装済み。kernel 側再監査は [mizar-kernel task 35](../../mizar-kernel/en/todo.md) で完了済み。member 同一性の調整は [mizar-checker task 36](../../mizar-checker/en/todo.md)。source-derived runner/extraction は external のまま。 |
| F2(型実引数の inhabitation) | spec 修正済み(§17.3.4 gating 行)。checker task 43 が built-in/base-shape inhabitation 表を完了した。elaborator gating は task 28。 |
| F3(`type extends M` の object/schema 混同) | spec 修正済み(§18.10.2)。explicit-payload bounded-view lowering は task 27 で F1 とともに cover 済み。 |
| F4(functor guard、実引数シグネチャ適合) | spec 修正済み(§18.10.4、§18.9)。explicit-payload 実装は task 29 で完了 |
| F5(型パラメータの sethood) | spec 修正済み(§18.10.2 sethood 段落)。explicit-payload plumbing は task 30 で完了。source-derived extraction は external のまま。 |
| F6(テンプレート本体内の scheme 適用) | spec 修正済み(§18.10.3 の段落)。explicit substitution-composition metadata 実装は task 29 で完了 |
| F7(widening 上の推論決定性) | task 26 で spec 修正済み。実装は payload を伴う inference / elaboration 作業へ deferred |
| F8(部分 algorithm の functor 実引数) | spec 修正済み(§18.8.4)。explicit diagnostic-only rejection は task 29 で完了 |
| corpus seed(6 件) | inactive な `advanced_semantics` seed。元の 4 件の encoding seed と task 26 の F7 推論決定性 seed。runner 到着時に [mizar-checker task 48](../../mizar-checker/en/todo.md) と mizar-test の runner 作業で活性化 |

26. **Spec 決定: テンプレート引数推論の決定性(F7)。** [x]
    - widening 束上の §18.2.7 推論アルゴリズムを決定する。監査の推奨:
      省略された `[T]` は mode unfolding 後の実引数の宣言型で推論し、残る
      複数候補はインスタンスが論理的に等価でも ambiguity エラーとする。
      決定性のみの問題 — F1-F5 修正後は well-formed な instance はすべて
      健全。spec 18 §18.2.7 を(英日で)更新し、テンプレート推論と
      オーバーロード選択が単一の比較ストーリーを使うよう、overload
      tie-break 決定([mizar-checker task 37](../../mizar-checker/en/todo.md))
      と調整する。checker task 37 は Phase B の overload tie-break 決定を
      記録し、この task は Phase A の省略テンプレート引数推論規則を記録する。
      欠落した source payload を推測してはならない。
    - 受け入れ条件: §18.2.7 が比較に使う型と ambiguity 診断を命名する。
      残余候補ケースを固定する ambiguity `.miz` seed を sidecar と
      `spec_trace.toml` エントリ付きで追加する。
    - 検証: `cargo test -p mizar-test`。
    - 依存: mizar-checker task 37 は Phase B tie-break 決定を記録する。
      この task は Phase A 推論決定性の決定を記録する。参照:
      template_encoding_audit.md F7。
    - task 26 で完了: spec 18 §18.2.7 は、省略された func/pred template
      型パラメータを mode-unfolded declared argument type だけから推論すると
      明記した。widening 祖先の探索、cluster expansion、`qua` view 推論は
      行わない。相異なる declared-type candidate が残る場合、それらの closure
      が同値でも ambiguous template instantiation とする。inactive seed
      `fail_template_inference_declared_type_ambiguity_001` と
      `fail_template_inference_requires_explicit_qua_view_001`、および trace row
      `spec.en.18.templates.inference_determinism.semantic` を追加した。これは
      F7 の spec decision だけを閉じる。checker/core source semantics や
      payload bridge behavior は変更していない。

27. **reduct/view lowering(F1、F3)。** [x]
    - elaboration に reduct-view エンコードを実装する: 改名または複数経路
      の inherit 辺上の `qua` と bounded-type-parameter instantiation に
      対して `view_{D→B}` 項を emit する。attribute atom と field
      selection を flattened instance ではなく view 項に対して emit する。
      reduct term 上の明示的 exact-instance guard formula(§5.8.5)を保持し、
      source-derived extensionality emission は checker/runner bridge に残す。
      `type extends M` の object-level ストーリー(view 型の
      schema パラメータ、`T.binop` の lowering、§18.10.2)をカバーする。
      type/fact と term/formula lowering の surface(tasks 9-10)および
      直近で landed した builtin type bridge / typed-AST elaboration の
      seam に触れる。
    - 受け入れ条件: diamond 例(Ring → AddGroup/MulMonoid → Magma、field
      改名あり)が `add(R) = mul(R)` を導出せずに lower される。
      `fail_template_qua_view_attribute_leak_001` seed の拒否が lowered
      form から導出可能(一方の view 上の attribute evidence が他方の
      view 上の bound を discharge しない)。改名辺・複数経路・
      明示的 exact-instance guard preservation を Rust fixture がカバー
      する。
    - 検証: `cargo test -p mizar-core`、
      `cargo clippy -p mizar-core --all-targets -- -D warnings`;
      共有境界には `cargo test -p mizar-checker`。
    - 依存: 10、11; 同一性規則は mizar-checker task 36(member 同一性
      決定)。landing 時に mizar-kernel task 35 へ通知。参照: spec 05
      §5.8.3/§5.8.5、13 §13.8.7、18 §18.10.2;
      template_encoding_audit.md F1、F3。
    - task 27 で完了: `ReductViewSeed` / `ReductView` は checker-owned
      `QuaPathKey` と順序付きの明示的 reduct functor を保持する。
      `CoreTermSeedKind::Qua` は no-reduct identity/cluster view だけ base term を
      再利用し、explicit reduct payload は ordered `Apply` view term へ lower する。
      Rust fixture は renamed diamond view、composed/multi-path view、final view term
      上の template-bound fact / field selection、reduct term 上の exact-instance
      guard preservation、type/fact と term/formula lowering の両方における empty reduct
      payload rejection を cover する。`doc/spec`、既存 `.miz`、expectation、
      source-derived runner、fake checker payload は変更していない。

28. **テンプレート型実引数の inhabitation gating(F2)。** [x]
    - テンプレートの `type_expression` 実引数について checker-owned の
      §17.3.4 inhabitation-evidence gate result を consume する: schema
      コンテキストは各型パラメータについて `∃x. is_T(x)` を仮定してよく、
      その代わりすべての instantiation site は built-in/base-shape table を
      満たす checker result を持たなければならない。属性付き実引数は
      existential registration を必要とする。checker registration semantics は
      再実行せず、lowering 中に per-parameter の inhabitation fact を schema
      コンテキストへ emit する。
    - 受け入れ条件: `fail_template_type_actual_missing_existential_001` の
      拒否が導出可能(充足不能な属性連鎖の実引数が instantiation site で
      拒否され、`ex y st y is hollow set` 型の公理が emit されない)。
      evidence 付きの gated 実引数が正常に lower される pass fixture を
      持つ。
    - 検証: `cargo test -p mizar-core`、`cargo test -p mizar-checker`。
    - 依存: 27; mizar-checker task 43 の built-in/base-shape inhabitation 表と
      task 20 gate surface を consume する。参照: spec 07 §7.8、17 §17.3.4、
      18 §18.10.2;
      template_encoding_audit.md F2。
    - task 28 で完了: `TemplateTypeParameterInhabitationSeed` は checker が提供する
      witness binder を schema-context `∃x. is_T(x)` assumption へ lower する。
      `TemplateTypeActualGateSeed` は checker existential-gate status と registration、
      base-evidence、guard-fact、diagnostic backref を保存する。非 satisfied gate は
      core diagnostic だけを emit し、actual-side existential axiom や proof obligation は
      生成しない。Rust fixture は accepted registration/base/fact evidence、actual の
      missing-existential rejection、invalid gate payload を cover する。`doc/spec`、
      既存 `.miz`、expectation、traceability metadata、source-derived runner、fake checker
      payload は変更していない。

29. **scheme 実引数のシグネチャ適合・guard 義務・functor 実引数検証(F4、F6、F8)。** [x]
    - `defpred`/`deffunc` 実引数に対する §18.10.4/§18.9 規則を実装する:
      反変 domain / 共変 codomain の widening 検査。functor guard は
      instantiation 時に discharge される証明義務であり、公理として assert
      しない(`deffunc shrink(x be Nat) -> Integer` の偽公理を生まない)。
      外側テンプレートのパラメータを実引数としてテンプレート本体内で
      scheme を適用する §18.10.3 規則(F6)を実装する。`func(...)`
      パラメータへの部分(未 promote)algorithm 実引数を拒否する —
      FOL 関数記号を表すのは `deffunc`、テンプレート functor、promoted
      `terminating` algorithm のみ(F8)。
    - 受け入れ条件: `fail_template_func_actual_result_widening_001` の拒否
      が導出可能。guard 義務が assert された公理ではなく obligation seed
      (task 18 surface)として現れる。部分 algorithm 実引数が安定した
      診断で拒否される。入れ子の scheme 適用が substitution-lemma の再構成
      に従い外側パラメータを健全に使う。
    - task 29 で完了: `TemplateSchemeActualSeed` / `TemplateSchemeActual` は
      type、predicate、functor parameter の checker-owned scheme-actual row を
      保存する。predicate/functor row は directional widening evidence(schema
      domain から actual parameter、functor では actual result から schema codomain)
      を保持し、accepted functor row は axiom や active VC ではなく traceability として
      `Skipped` checker-initial guard seed を emit する。partial/void/unsupported actual
      は diagnostic-only であり、enclosing template parameter は新しい symbol や
      source-derived closure expansion を作らず substitution-composition metadata を
      保存する。source extraction と active corpus execution は external/deferred のまま。
    - 検証: `cargo test -p mizar-core`、`cargo test -p mizar-vc`(seed
      handoff)、`cargo test -p mizar-test`。
    - 依存: 27; obligation seed は task 18 を通じて流れる。参照: spec 18
      §18.9/§18.10.3/§18.10.4/§18.8.4; template_encoding_audit.md F4、F6、
      F8。

30. **型パラメータの sethood plumbing(F5)。** [x]
    - §18.10.2 の sethood 段落に従い、テンプレート本体内の Fraenkel
      comprehension gating を bound 継承または constraint 供給の sethood
      に keying する: 裸の型パラメータは sethood を持たないため、それを
      range とする comprehension は bound または明示的 constraint が
      sethood evidence を供給しない限り拒否される。
    - 受け入れ条件: explicit payload fixture は
      `fail_template_fraenkel_over_type_param_001` と同じ拒否を導出する
      (`para[set]` 上の Russell 型 comprehension が生じない)。bound 継承と
      constraint 供給の sethood は generated Fraenkel origin に保存され、
      通常の non-template Fraenkel evidence は変えず、malformed または duplicate
      cross-reference payload は fail closed する。source-derived extraction と
      active corpus execution は external/deferred のまま。
    - 検証: `cargo test -p mizar-core`、`cargo test -p mizar-checker`。
    - 依存: 28; mizar-checker task 43 のパラメータ化 sethood 形(SSA-013)を
      consume する。参照: spec 13 §13.4.2、18 §18.10.2;
      template_encoding_audit.md F5。

31. **Exact source-derived contradiction theorem lowering。** [ ]
    - mizar-test Tasks 266/268完了済みとして、そのchecker-owned theorem owner、checked
      contradiction formula、proof-status/skeleton、explicit terminal-goal payloadだけを
      Task 267のexact mappingで消費する。source/provenance identityを完全に持つ
      source-derived `CoreIr`/theorem-obligation dataを生成し、prepared mizar-test
      Task-10 type-elaboration snapshot consumerとpairにする。
    - Task 268はrequired explicit checker proof/status/terminal tableを提供済みで、
      dependency gapではない。本taskはexecutableであるが、payloadを再構築せず
      validationしてconsumeしなければならない。
    - selected mappingはpublic structurally `Valid` theorem item 1件、`False`
      1件、`PendingAutomaticProof` 1件、direct terminal 1件、`proof/0`のActive
      `TheoremProof` seed 1件である。generic checker-owned provenanceをrelaxせず、
      narrow transactional exact adapterとcomplete pre/postconditionで実装する。
    - deterministic item/formula/proof/obligation identityとmissing/duplicate/
      reordered/mismatched payloadのfail-closed testを要求する。core raw-syntax
      inspection、proof status/terminal goal合成、theorem acceptance/discharge、broader
      formula/proof、CFG/VC generation、expectation rebaselineは禁止する。
    - encoder testはnon-ASCII FQN byte length、empty/populated structural path、
      decimal/no-leading-zero spelling、S/TのNode-vs-Role rejectionをcoverする。
      adapter testはpreflight/generic lowering/exact enrichment/postvalidation
      failureをinjectし、全phaseで`Err`かつpartial `CoreIr`/source mapなしをassertする。
    - 依存: mizar-test Tasks 266-268。仕様: 14-16、architecture 06。

32. **Remaining source-derived `CoreIr`/`ControlFlowIr` task decomposition。** [ ]
    - checker Task 247後、残る全declaration/definition、attribute/type、term/formula、
      Task-180以外のproof、registration/activation/trace/overload、algorithm、
      hidden-local/contract、`ControlFlowIr` source familyをdocs/traceability-onlyで
      inventoryする。各familyにbounded checker-to-core producer/lowering task、
      prepared mizar-test snapshot consumer、dependency、explicit forbidden scopeを
      割り当てる。
    - CORE-AUDIT-G001/G002/G005をreconcileする。synthetic `CoreIr`/CFG、core内
      source推論、behavior実装、fixture/expectation/trace status change、coverage
      promotionは禁止する。依存: checker Task 247。

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

obligation-seed handoff と architecture-22 anchor input では追加で実行する:

```text
cargo test -p mizar-vc
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
