# mizar-resolve TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。準拠先は
[architecture/ja/03.module_and_symbol_resolution.md](../../architecture/ja/03.module_and_symbol_resolution.md)。
自律 crate 開発の準備は [00.crate_plan.md](./00.crate_plan.md) で追跡する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| resolved_ast | `resolved_ast.md`（task 2） | `src/resolved_ast.rs` | [ ] |
| env | `env.md`（task 3） | `src/env.rs` | [ ] |
| imports | `imports.md`（task 8） | `src/imports.rs` | [ ] |
| names | `names.md`（task 12） | `src/names.rs` | [ ] |
| labels | `labels.md`（task 17） | `src/labels.rs` | [ ] |
| symbols | `symbols.md`（task 19） | `src/symbols.rs` | [ ] |

`mizar-resolve` はパイプライン phase 4-5 を実装する。入力は `SurfaceAst`、
出力は `ResolvedAst` と `SymbolEnv` である。名前空間、import、export、
ラベル、修飾名、シグネチャ収集を扱う最初の意味論的所有者であり、2 つの波で
構築する。まずデータ形状と import/名前解決の骨格、次に解決の網羅性を
`mizar-parser` の文法カバレッジと歩調を合わせて拡大する（パーサーがまだ
生成できないものを resolver は解決できない）。

依存順序: データ形状（`resolved_ast`、`env`）→ `imports` → `names` /
`labels` → `symbols`（シグネチャ収集）→ artifact summary の再利用。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` と `mizar-syntax` に依存する。frontend seam を
通じて生成される `SurfaceAst` を消費するため、意味のある入力は
`mizar-parser` task 5-7（モジュール骨格、import、export）が入って初めて
存在する。以後、解決の網羅性はパーサーの文法タスクとともに成長する。後段の
ModuleSummary 再利用タスクで `mizar-artifact`（スキーマ波）への依存が加わる。
アーキテクチャ: [03.module_and_symbol_resolution.md](../../architecture/ja/03.module_and_symbol_resolution.md)
（[internal 07](../../internal/ja/07.crate_module_layout.md) によりアーキテクチャ
18 と 19 も精緻化対象）、
IR 所有権: [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

## 解決済みおよび保留中の決定

- **ドット役割の最終決定: 未解決。task 16 で解決する。** パーサーは
  selector と namespace の分離を構文的なまま残す（`mizar-parser` task 10、
  `mizar-syntax` task 8）。resolver が変数スコープを用いて決定を完了する。
  トップレベル（[../../todo.md](../../todo.md)「Resolved And Open
  Decisions」）にも登録済み。
- **暫定オーケストレーション seam: 未解決。task 7 で解決する。**
  パイプラインのオーケストレーションは `mizar-driver` が所有する
  （[internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)）。
  resolver は phase サービスであり、ドライバーではない。未解決なのは、
  `mizar-build` module-index provider と `mizar-driver` の phase レジストリが
  統合されるまで resolver が消費する暫定モジュール索引入力（既定候補は
  workspace スタブプロバイダー）である。
- **`mizar-diagnostics` 採用時期: 未解決。task 13 までに決定する。**
  `mizar-diagnostics` は目標 crate 配置の一部である
  （[internal 07](../../internal/ja/07.crate_module_layout.md)、
  [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)）。
  resolver は複数ファイルにまたがる診断を持つ最初の crate である。共有
  diagnostic レコードを今導入するか、もう 1 層だけ crate ごとの診断を
  維持するかを決める。トップレベルにも登録済み。
- **ModuleSummary 再利用の時期: 未解決。task 24 で解決する。**
  アーキテクチャ 03 は依存モジュールをソース再読込ではなく `ModuleSummary`
  artifact として消費することを許す。最初のイテレーションはメモリ内の
  依存閉包を解決し、artifact 経由の経路は `mizar-artifact` の
  module-summary スキーマを先に必要とする。トップレベルにも登録済み。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-resolve` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 基盤

1. **crate の足場と lint 方針のガード。** [ ]
   - `mizar-session` と `mizar-syntax` に依存する workspace メンバー
     `mizar-resolve` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs`（workspace lint へのオプトイン、deny
     ベースライン、将来の `allow` の隣に根拠）を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: なし。仕様: アーキテクチャ 03。

2. **仕様: `resolved_ast.md`。** [ ]
   - `ResolvedAst` のデータ形状仕様を執筆する（英語と日本語、コードなし）:
     `ModuleId`、`SymbolId`（安定かつ完全修飾）、ノード arena、
     `NameRefTable`、`LabelRefTable`、`ResolvedImports`、未解決/曖昧ノードの
     明示表現、recovered シェルの規則、下流の `ObligationAnchor` が消費する
     正規化された origin/provenance field。
   - 依存: 1。仕様: アーキテクチャ 03「Interface Definitions」、
     [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

3. **仕様: `env.md`。** [ ]
   - `SymbolEnv` の仕様を執筆する（英語と日本語、コードなし）: 索引群
     （`SymbolIndex`、`DefinitionIndex`、`OverloadIndex`、
     `RegistrationIndex`、`NamespaceGraph`）、ソース単位の寄与追跡、
     無効化の注記。
   - 依存: 1。仕様: アーキテクチャ 03「Symbol Environment」。

4. **`resolved_ast` データ形状の実装。** [ ]
   - task 2 の仕様どおりに arena、テーブル、id 不変条件を実装し、型付き
     accessor ヘルパーを加える。
   - テスト: id の決定性。arena 不変条件（有効な子 id、循環なし）。
     テーブルのラウンドトリップ。
   - 依存: 2。仕様: `resolved_ast.md`。

5. **`env` データ形状の実装。** [ ]
   - task 3 の仕様どおりに `SymbolEnv` 索引群とソース単位寄与追跡を
     実装する。
   - テスト: 索引の挿入/参照ラウンドトリップ。ソース単位ごとの寄与追跡。
   - 依存: 3。仕様: `env.md`。

6. **決定的 debug レンダリング。** [ ]
   - コーパス snapshot ベースライン用に `ResolvedAst` と `SymbolEnv` の
     安定した人間可読レンダリングを追加する。実行間・プラットフォーム間で
     バイト一致すること。
   - テスト: 繰り返しレンダリングで同一出力。現行の全ノード/テーブル種別を
     カバーするフィクスチャ。
   - 依存: 4、5。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)
     「Snapshot Tests」。

7. **モジュール索引の入力契約と暫定オーケストレーション seam。** [ ]
   - 暫定 seam の決定を解決する: resolver が phase サービス
     （[internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)）
     として消費するパッケージ/モジュール索引入力（アーキテクチャ 03
     Step 1）を定義し、`mizar-build` module-index provider と `mizar-driver`
     レジストリが統合されるまでテスト用 workspace スタブプロバイダーを
     用意する。決定をここに記録し、トップレベルにも反映する。
   - テスト: スタブプロバイダーが複数モジュールのフィクスチャを供給する。
     モジュール識別がローカル別名に依存しない。
   - 依存: 4。仕様: アーキテクチャ 03「Step 1」、`mizar-build` todo
     tasks 5〜6 と `module_index.md`。

### import

8. **仕様: `imports.md`。** [ ]
   - import 解決の仕様を執筆する（英語と日本語、コードなし）: 2 パス契約
     （frontend の候補プレスキャン対意味論的検証）、別名と相対プレフィックスの
     規則、循環ポリシー、未解決 import の表現。
   - 依存: 2。仕様: アーキテクチャ 03「Step 2」、
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

9. **import グラフ構築と循環の拒否。** [ ]
   - モジュール索引上に意味論的 import グラフを構築し、決定的な診断で
     循環を拒否する。
   - テスト: 循環フィクスチャが決定的に拒否される。非循環フィクスチャが
     期待どおりのグラフになる。
   - 依存: 7、8、`mizar-parser` task 6。仕様: `imports.md`。

10. **import の別名、相対プレフィックス、未解決 import の回復。** [ ]
    - 別名と `.`/`..` プレフィックスを正準モジュール識別へ解決する。未解決
      import は明示的に表現し、モジュールの残りの解決を続行する。
    - テスト: 別名が正準識別を変えない。未解決 import がモジュール解決を
      中断させない。
    - 依存: 9。仕様: `imports.md`。

### 名前

11. **宣言シェル。** [ ]
    - `SurfaceAst` の item からローカル宣言シェルを構築する（アーキテクチャ
      03 Step 3）: item の識別、可視性マーカー、export 射影。型付けも本体
      解決も行わない。
    - テスト: パーサーが生成する item 種別ごとのシェル。recovered な部分木は
      recovered フラグ付きシェルになり、黙って捨てられない。
    - 依存: 7、`mizar-parser` task 5 と 7。仕様: アーキテクチャ 03「Step 3」。

12. **仕様: `names.md`。** [ ]
    - 名前解決の仕様を執筆する（英語と日本語、コードなし）: スコープ
      モデル、namespace を symbol より先に解決する順序、可視性と
      シャドーイングの規則、曖昧性の表現、ドット連鎖最終決定の契約
      （決定は task 16 が記録する）。
    - 依存: 2。仕様: アーキテクチャ 03「Step 4」、
      [11.symbol_management.md](../../../spec/ja/11.symbol_management.md)。

13. **名前空間解決。** [ ]
    - import グラフと宣言シェル上で、symbol より先に namespace セグメントを
      解決する（アーキテクチャ 03「Namespaces Resolve Before Symbols」）。
    - テスト: 入れ子名前空間のフィクスチャ。namespace 欠落診断が失敗
      セグメントの範囲を保持する。
    - 依存: 10、11、12。仕様: `names.md`。

14. **修飾名、可視性、シャドーイング。** [ ]
    - 修飾・非修飾のシンボル参照を仕様のスコープ規則どおりに可視性と
      シャドーイング込みで解決し、結果を `SymbolId` として `NameRefTable`
      に記録する。
    - テスト: 修飾、シャドーイング、private シンボル可視性のフィクスチャ。
    - 依存: 13。仕様: `names.md`、
      [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

15. **未解決・曖昧参照の診断。** [ ]
    - 未解決/曖昧な参照を決定的な候補リストを持つ明示ノードとして表現する。
      1 つの未解決根から診断を連鎖させない。
    - テスト: 候補順が安定した曖昧性フィクスチャ。1 つの未解決 import が
      1 つの主診断を生む。
    - 依存: 14。仕様: `names.md`、
      [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。

16. **ドット連鎖の最終決定。** [ ]
    - パーサーが構文的なまま残した未解決ドット連鎖を完了する: 変数スコープに
      よる selector access と namespace 区切りの分離。決定を `names.md` に
      記録し、トップレベルの保留決定をクローズする。
    - テスト: 仕様 §A.2.5 の例による selector/namespace の分離。どちらの
      役割にも合わない連鎖の診断。
    - 依存: 14、`mizar-parser` task 10。仕様:
      [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)。

### ラベル

17. **仕様: `labels.md`。** [ ]
    - ラベル解決の仕様を執筆する（英語と日本語、コードなし）: 独立した
      ラベルスコープ族、証明ブロックの入れ子、前方参照ポリシー、下流の
      `ObligationAnchor` 構築が使う正規化された label-origin path。
    - 依存: 2。仕様: アーキテクチャ 03「Label Resolution Is Scoped
      Separately」。

18. **ラベル解決。** [ ]
    - task 17 に従って文/定理ラベルを解決する。証明ブロックの入れ子を含む。
    - テスト: 入れ子証明をまたぐラベルのシャドーイング。後方ラベルへの
      参照の拒否。`LabelRefTable` の決定性。
    - 依存: 11、17、`mizar-parser` task 22。仕様: `labels.md`、
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)。

### シグネチャ収集

19. **仕様: `symbols.md`。** [ ]
    - シグネチャ収集の仕様を執筆する（英語と日本語、コードなし）: 宣言パス
      契約（型検査なし）、種別ごとのシグネチャ形状、重複・不正宣言の
      ポリシー、整形や無関係な局所編集で安定する正規化された semantic
      origin。
    - 依存: 3。仕様: アーキテクチャ 03「Step 5」、
      [11.symbol_management.md](../../../spec/ja/11.symbol_management.md)。

20. **収集骨格と重複検出。** [ ]
    - 宣言シェルから `SymbolEnv` を構築する: 種別ごとの名前登録、重複・
      競合診断、オーバーロード候補のグループ化。シグネチャ自体はまだ
      不透明のままでよい。
    - テスト: 種別ごとの重複検出。候補グループ化。決定的な診断順。
    - 依存: 5、11、19。仕様: `symbols.md`。

21. **種別ごとのシグネチャ抽出。** [ ] — `mizar-parser` task 23-31 が律速。
    - 具体的なシグネチャ（struct、mode、attribute、predicate、functor、
      theorem、registration、template）を増分で抽出する: 各増分は、その
      宣言種別を生成するパーサー文法タスクの後に、独立した変更として
      着地させる。最後の対の増分が着地した時点でチェックを付ける。
    - 増分ごとのテスト: その種別のシグネチャ形状フィクスチャと
      `SymbolEnv` 参照。
    - 依存: 20。`mizar-parser` task 23-31 と対になる。仕様: `symbols.md`。

### 強化と横断フォローアップ

22. **recovered 構文ポリシー。** [ ]
    - resolver の各段が recovered な `SurfaceAst` 部分木をどう扱うか
      （スキップ、シェルのみ、診断）を定義・実装する。`mizar-syntax` の
      `recovered` フラグ契約を維持する。
    - テスト: recovered 入力で解決が panic しない。recovered 領域から診断が
      連鎖しない。
    - 依存: 13。仕様: [mizar-syntax recovery.md](../../mizar-syntax/ja/recovery.md)。

23. **stage `declaration_symbol` のコーパスランナー。** [ ]
    - `tests/miz/{pass,fail}/` のケースを stage `declaration_symbol` で
      ハーネスに接続し、`spec_trace.toml` のカバレッジ項目を付ける。task
      9-20 の pass/fail ケースをシードし、40/60 の pass/fail 比率へ向けて
      拡大する。
    - 依存: 20。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)、
      [traceability.md](../../mizar-test/ja/traceability.md)。

24. **ModuleSummary の再利用。** [ ]
    - 依存モジュールを（schema-version を検証した）`ModuleSummary` artifact
      として消費し、ソースを再読込しない。summary が無い・非互換のときは
      ソース解決へフォールバックする。
    - テスト: summary 経由とソース経由の解決が共有フィクスチャ上で一致
      する。非互換スキーマは診断付きでフォールバックする。
    - 依存: 20、`mizar-artifact` task 5。仕様: アーキテクチャ 03「Module
      Summary」、[18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

25. **決定性スイート。** [ ]
    - 同一入力が同一の id、テーブル、診断順、debug レンダリングを生むことの
      プロパティ的検証。frontend のスイートに倣う。
    - 依存: 21。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

26. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の決定手続きを適用し、所有
      モジュール仕様の enum の隣に各決定を記録する。
    - 依存: 21。仕様: 全モジュール仕様。

27. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 26。仕様: 全モジュール仕様と本 TODO。

28. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-resolve/en/` の各英語正本と日本語版を比較し、API
      一覧、状態、用語、リンク、挙動の約束を同期する。
    - 依存: 27。仕様: リポジトリのドキュメント方針。

29. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 28。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-resolve
cargo clippy -p mizar-resolve --all-targets -- -D warnings
```

frontend seam、コーパス、共有境界に触れるタスクでは追加で実行する:

```text
cargo test -p mizar-syntax
cargo test -p mizar-frontend
cargo test -p mizar-test
```

obligation anchor が消費する normalized origin、label-origin、symbol-origin
field では追加で実行する:

```text
cargo test -p mizar-core
cargo test -p mizar-vc
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- resolver が所有するのは名前、スコープ、import、export、ラベル、
  シグネチャ収集のみ: 型推論、オーバーロードの勝者選択、cluster 事実、
  証明意味論は持たない。
- シンボルが解決されたら、下流の段は生の文字列ではなく必ず `SymbolId` を
  使う。未解決・曖昧なノードは診断のため明示的なまま残す。
- 解決の網羅性は `mizar-parser` の文法タスクが律速する。パーサーがまだ
  生成できない構文に対する解決を先行して作らない。
