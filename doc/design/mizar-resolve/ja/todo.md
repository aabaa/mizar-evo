# mizar-resolve TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様は、それを引用する実装タスクより前に、専用の仕様タスクが
（英語と日本語を同じ変更で）順次導入する。準拠先は
[architecture/ja/03.module_and_symbol_resolution.md](../../architecture/ja/03.module_and_symbol_resolution.md)。
自律 crate 開発の準備は [00.crate_plan.md](./00.crate_plan.md) で追跡する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| resolved_ast | `resolved_ast.md`（task 2） | `src/resolved_ast.rs` | [x] |
| env | `env.md`（task 3） | `src/env.rs` | [x] |
| module_index | アーキテクチャ 03 Step 1 / `mizar-build` `module_index.md`（task 7） | `src/module_index.rs` | [x] |
| imports | `imports.md`（task 8） | `src/imports.rs` | [~] |
| declarations | `declarations.md`（task 11） | `src/declarations.rs` | [x] |
| names | `names.md`（task 12） | `src/names.rs` | [~] |
| labels | `labels.md`（task 17） | `src/labels.rs` | [x] |
| symbols | `symbols.md`（task 19） | `src/symbols.rs` | [~] |
| recovered 構文ポリシー | `recovery.md`（task 22） | `src/recovery.rs` helper と stage 別 call site | [x] |

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

この crate は `mizar-session`、`mizar-syntax`、および `mizar-build` の
build-side `ModuleIndexProvider` contract に依存する。frontend seam を通じて
生成される `SurfaceAst` を消費するため、意味のある入力は `mizar-parser`
task 5-7（モジュール骨格、import、export）が入って初めて存在する。以後、
解決の網羅性はパーサーの文法タスクとともに成長する。後段の ModuleSummary
再利用タスクで `mizar-artifact`（スキーマ波）への依存が加わる。
アーキテクチャ: [03.module_and_symbol_resolution.md](../../architecture/ja/03.module_and_symbol_resolution.md)
（[internal 07](../../internal/ja/07.crate_module_layout.md) によりアーキテクチャ
18 と 19 も精緻化対象）、
IR 所有権: [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

## 解決済みおよび保留中の決定

- **ドット役割の最終決定: task 16 で解決済み。** パーサーは selector と namespace の
  分離を構文的なまま残す（`mizar-parser` task 10、`mizar-syntax` task 8）。
  `mizar_resolve::names::DotChainFinalizer` が lexical local-term scope を用いて
  決定を完了し、selector validation は checker/type phase に残す。トップレベル
  （[../../todo.md](../../todo.md)「Resolved And Open Decisions」）にも記録済み。
- **暫定オーケストレーション seam: task 7 で解決済み。**
  パイプラインのオーケストレーションは `mizar-driver` が所有する
  （[internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)）。
  resolver は phase サービスであり、ドライバーではない。resolver は
  build 側の `ModuleIndexProvider` を
  `mizar_resolve::module_index::ModuleIndexInput` 経由で消費し、driver registry
  統合が入るまでのテスト・フィクスチャ用に限って resolver-local な
  `WorkspaceStubModuleIndexProvider` を保持する。
- **`mizar-diagnostics` 採用時期: task 13 で deferred。**
  `mizar-diagnostics` は目標 crate 配置に残る
  （[internal 07](../../internal/ja/07.crate_module_layout.md)、
  [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)）。
  R-013 は namespace-resolution failure を crate-local/internal record として保持し、
  R-015 は name diagnostic を crate-local/internal に保つ。R-G001 に resolver code range
  がまだないため public resolver diagnostics を出さない。後続の user-facing resolver
  diagnostic integration の前に再検討する。
- **ModuleSummary 再利用の時期: task 24 で解決済み。**
  アーキテクチャ 03 は依存モジュールをソース再読込ではなく `ModuleSummary`
  artifact として消費することを許す。最初のイテレーションはメモリ内の
  依存閉包を解決する。R-024 は canonical な `mizar-artifact` module summary を
  artifact-owned reader 経由で消費し、検証済み projection を resolver-owned
  summary contribution index へ写像する。resolver-local な artifact format は
  創作しない。
- **nested proof label shadowing の文言: task 17 で解決済み。**
  以前の task 18 の test note は「nested proof の label shadowing」を求めていたが、
  spec chapter 15 は inner-scope label shadowing を禁止する。R-017 はこの note を
  derived TODO の `design_drift` として分類し、visible label scope をまたぐ
  duplicate / conflict の拒否へ修正する。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-resolve` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 基盤

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session` と `mizar-syntax` に依存する workspace メンバー
     `mizar-resolve` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs`（workspace lint へのオプトイン、deny
     ベースライン、将来の `allow` の隣に根拠）を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: なし。仕様: アーキテクチャ 03。

2. **仕様: `resolved_ast.md`。** [x]
   - `ResolvedAst` のデータ形状仕様を執筆する（英語と日本語、コードなし）:
     `ModuleId`、`SymbolId`（安定かつ完全修飾）、ノード arena、
     `NameRefTable`、`LabelRefTable`、`ResolvedImports`、未解決/曖昧ノードの
     明示表現、recovered シェルの規則、下流の `ObligationAnchor` が消費する
     正規化された origin/provenance field。
   - 依存: 1。仕様: アーキテクチャ 03「Interface Definitions」、
     [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

3. **仕様: `env.md`。** [x]
   - `SymbolEnv` の仕様を執筆する（英語と日本語、コードなし）: 索引群
     （`SymbolIndex`、`LabelIndex`、`DefinitionIndex`、`OverloadIndex`、
     `RegistrationIndex`、`NamespaceGraph`、`DeclarationDependencyIndex`）、
     ソース単位の寄与追跡、無効化の注記。
   - 依存: 1。仕様: アーキテクチャ 03「Symbol Environment」。

4. **`resolved_ast` データ形状の実装。** [x]
   - task 2 の仕様どおりに arena、テーブル、id 不変条件を実装し、型付き
     accessor ヘルパーを加える。
   - テスト: id の決定性。arena 不変条件（有効な子 id、循環なし）。
     テーブルのラウンドトリップ。
   - 依存: 2。仕様: `resolved_ast.md`。

5. **`env` データ形状の実装。** [x]
   - task 3 の仕様どおりに `SymbolEnv` 索引群とソース単位寄与追跡を
     実装する。
   - テスト: 索引の挿入/参照ラウンドトリップ。ソース単位ごとの寄与追跡。
   - 依存: 3。仕様: `env.md`。

6. **決定的 debug レンダリング。** [x]
   - コーパス snapshot ベースライン用に `ResolvedAst` と `SymbolEnv` の
     安定した人間可読レンダリングを追加する。実行間・プラットフォーム間で
     バイト一致すること。
   - テスト: 繰り返しレンダリングで同一出力。現行の全ノード/テーブル種別を
     カバーするフィクスチャ。
   - 依存: 4、5。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)
     「Snapshot Tests」。

7. **モジュール索引の入力契約と暫定オーケストレーション seam。** [x]
   - 暫定 seam の決定を解決する: resolver が phase サービス
     （[internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)）
     として消費するパッケージ/モジュール索引入力（アーキテクチャ 03
     Step 1）を定義し、`mizar-build` module-index provider と `mizar-driver`
     レジストリが統合されるまでテスト用 workspace スタブプロバイダーを
     用意する。決定をここに記録し、トップレベルにも反映する。
   - 決定: `mizar_resolve::module_index::ModuleIndexInput` は build 側の
     `ModuleIndexProvider` contract を借用し、package、namespace、module、
     dependency-summary lookup を転送する。resolver は build 側 module identity
     を resolver の `ModuleId` に変換するが、package discovery、source loading、
     module-index 構築、dependency-summary artifact parsing は行わない。
     `WorkspaceStubModuleIndexProvider` は resolver-local なテスト基盤に限る。
   - テスト: スタブプロバイダーが複数モジュールのフィクスチャを供給する。
     モジュール識別がローカル別名に依存しない。provider error は決定的である。
   - 依存: 4。仕様: アーキテクチャ 03「Step 1」、`mizar-build` todo
     tasks 5〜6 と `module_index.md`。

### import

8. **仕様: `imports.md`。** [x]
   - import 解決の仕様を執筆する（英語と日本語、コードなし）: 2 パス契約
     （frontend の候補プレスキャン対意味論的検証）、別名と相対プレフィックスの
     規則、循環ポリシー、未解決 import の表現。
   - [imports.md](./imports.md) で完了。英語正本は
     [../en/imports.md](../en/imports.md)。public diagnostic code は R-G001 が
     解決されるまで範囲外に保つ。
   - 依存: 2。仕様: アーキテクチャ 03「Step 2」、
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

9. **import グラフ構築と循環の拒否。** [x]
   - モジュール索引上に意味論的 import グラフを構築し、決定的な cycle record で
     循環を拒否する。public/user-facing diagnostics は引き続き R-G001 に gate される。
   - `src/imports.rs` に、`ModuleIndexInput` 上の canonical graph construction を実装済み。
     R-010 は同じ module に、alias binding、relative-prefix interpretation、
     source-shaped path からの unresolved-import recovery を追加する。
   - テスト: 循環フィクスチャが決定的に拒否される。非循環フィクスチャが
     期待どおりのグラフになる。
   - 依存: 7、8、`mizar-parser` task 6。仕様: `imports.md`。

10. **import の別名、相対プレフィックス、未解決 import の回復。** [x]
    - 別名と `.`/`..` プレフィックスを正準モジュール識別へ解決する。未解決
      import は明示的に表現し、モジュールの残りの解決を続行する。
    - `src/imports.rs` に `ImportPathResolver` と source-shaped
      `ImportPathResolution` record を実装済み。alias span、branch provenance、
      normalized path component、matched namespace/package candidate、crate-local
      failure class を、public diagnostic code を創作せず保持する。`ResolvedAst`
      source-walk integration は後続の import/name task と pair して残る。
    - テスト: 別名が正準識別を変えない。`.`/`..` が dot-separated `ModulePath`
      directory を使う。namespace/package binding が package-local fallback に勝つ。
      未解決 import がモジュール解決を中断させない。duplicate alias と reserved-root
      alias が明示的な unresolved record になる。
    - 依存: 9。仕様: `imports.md`。

### 名前

11. **宣言シェル。** [x]
    - `SurfaceAst` の item からローカル宣言シェルを構築する（アーキテクチャ
      03 Step 3）: item の識別、可視性マーカー、export 射影。型付けも本体
      解決も行わない。
    - `src/declarations.rs` に source-shaped collector slice を実装し、
      [declarations.md](./declarations.md) に仕様化した。表現済み declaration-like
      item、visibility wrapper、recovered-shell state、透明 annotation wrapper、
      export projection shell を記録する。preliminary `SymbolId`、label scope、
      duplicate / illegal-declaration diagnostics、final export validation、
      kind-specific signature extraction は後続の name、label、symbol work に残る。
    - テスト: parser-produced declaration shell の include/exclude inventory、
      visibility wrapper propagation、transparent annotation wrapper、recovered subtree が
      recovered フラグ付きで保持されること、target validation なしで保持される
      export projection shell。
    - 依存: 7、`mizar-parser` task 5 と 7。仕様: アーキテクチャ 03「Step 3」。

12. **仕様: `names.md`。** [x]
    - 名前解決の仕様を執筆する（英語と日本語、コードなし）: スコープ
      モデル、namespace を symbol より先に解決する順序、可視性と
      シャドーイングの規則、曖昧性の表現、ドット連鎖最終決定の契約
      （決定は task 16 が記録する）。
    - 英語正本 [names.md](./names.md) を追加し、日本語 companion と同期した。
      type-directed overload winner selection、selector type checking、cluster firing、
      public resolver diagnostic-code allocation は R-012 の外に保つ。
    - 依存: 2。仕様: アーキテクチャ 03「Step 4」、
      [11.symbol_management.md](../../../spec/ja/11.symbol_management.md)。

13. **名前空間解決。** [x]
    - import グラフと宣言シェル上で、symbol より先に namespace セグメントを
      解決する（アーキテクチャ 03「Namespaces Resolve Before Symbols」）。
    - R-013 の namespace lookup slice を `src/names.rs` に実装した。source-shaped
      namespace path candidate を import alias、reserved namespace root、package-name
      binding、current-package fallback を通じて canonical module namespace に解決する。
      internal namespace unresolved / ambiguous record、unresolved import-alias
      dependency、deterministic ambiguous alias target payload、provider-error
      classification を保持する。final symbol、selector field、overload winner は lookup
      しない。
    - テスト: 入れ子 namespace fixture、import alias、package/current-package fallback、
      longest-prefix binding、すべての reserved root、recovered / malformed path、
      unresolved import alias、ambiguous alias、provider error、deterministic ordering、
      earliest failing segment range を保持する missing-namespace record。
    - 依存: 10、11、12。仕様: `names.md`。

14. **修飾名、可視性、シャドーイング。** [x]
    - 修飾・非修飾のシンボル参照を仕様のスコープ規則どおりに可視性と
      シャドーイング込みで解決し、結果を `SymbolId` として `NameRefTable`
      に記録する。
    - R-014 の symbol-name lookup slice を `src/names.rs` に実装した。
      preliminary `NameSymbolProjection` record、declaration-point filtering、
      current-module shadowing、qualified namespace restriction、imported public
      visibility、enabled builtin fallback、failed-namespace propagation、
      overload-group placeholder collapse を使い、checker-owned winner selection は
      行わない。
    - テスト: 修飾、current-module shadowing、declaration-point visibility、
      private dependency rejection、builtin shadowing/fallback、overload-group collapse、
      incompatible ambiguity、failed namespace、recovered / malformed final spelling、
      deterministic table order。
    - 依存: 13。仕様: `names.md`、
      [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

15. **未解決・曖昧参照の診断。** [x]
    - 未解決/曖昧な参照を決定的な候補リストを持つ明示ノードとして表現する。
      1 つの未解決根から診断を連鎖させない。
    - R-015 は `src/names.rs` の crate-local/internal `NameDiagnosticReport` record
      として実装済み。deterministic `NameDiagnosticRootId` allocation、primary /
      cascade role、unresolved import-alias root、namespace/name dependent record、
      stable symbol/namespace candidate payload、public numeric diagnostic code を伴わない
      record ordering を持つ。
    - テスト: 候補順が安定した曖昧性フィクスチャ。1 つの未解決 import が
      1 つの主診断を生む。import-root、namespace、name、symbol ambiguity の混在診断が
      deterministic ordering を保つ。
    - 依存: 14。仕様: `names.md`、
      [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。

16. **ドット連鎖の最終決定。** [x]
    - パーサーが構文的なまま残した未解決ドット連鎖を完了する: 変数スコープに
      よる selector access と namespace 区切りの分離。決定を `names.md` に
      記録し、トップレベルの保留決定をクローズする。
    - R-016 は `src/names.rs` の `LocalTermScope`、`LocalTermBinding`、
      `DotChainCandidate`、`DotChainFinalizer`、`DotChainResolution` として実装済み。
      in-scope local term は namespace head を shadow し、use-site base node を使った
      `DeferredSelector` record を生成する。それ以外は leading path を
      `NamespaceResolver`、final segment を qualified `SymbolNameResolver` で解決する。
    - テスト: 仕様 §A.2.5 の例による selector/namespace の分離。どちらの
      役割にも合わない連鎖の診断。out-of-scope local は namespace を shadow しない。
      innermost local binding が勝つ。出力順序は deterministic。
    - 依存: 14、`mizar-parser` task 10。仕様:
      [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)。

### ラベル

17. **仕様: `labels.md`。** [x]
    - ラベル解決の仕様を執筆する（英語と日本語、コードなし）: 独立した
      ラベルスコープ族、証明ブロックの入れ子、前方参照ポリシー、下流の
      `ObligationAnchor` 構築が使う正規化された label-origin path。
    - [labels.md](./labels.md) で完了。英語正本は
      [../en/labels.md](../en/labels.md)。proof validity、template instantiation、
      ATP premise selection、`ObligationAnchor` construction、public resolver
      diagnostic-code allocation は R-017 の範囲外に保つ。
    - 依存: 2。仕様: アーキテクチャ 03「Label Resolution Is Scoped
      Separately」。

18. **ラベル解決。** [x]
    - task 17 に従って文/定理ラベルを解決する。証明ブロックの入れ子を含む。
    - R-018 は `src/labels.rs` に `LabelScopePath`、`LabelProjection`、
      `LabelReferenceCandidate`、`LabelResolver`、`LabelResolutionResult`、
      crate-local/internal な `LabelDiagnostic` record を実装した。実行可能な slice は
      theorem / lemma と proof-step label projection を解決し、forward reference を拒否し、
      resolved namespace / module projection 済みの qualified / grouped item candidate を扱い、
      public diagnostic code や proof / VC semantics を創作せずに deterministic な
      `LabelIndex` / `LabelRefTable` output を投入する。
    - テスト: proof-block visibility。visible nested proof scope をまたぐ
      duplicate / conflicting label の拒否。後続ラベルへの参照の拒否。
      parser coverage が存在する範囲での simple / qualified / lowered grouped-item
      citation lookup。deterministic `LabelIndex` / `LabelRefTable` / diagnostic ordering。
    - 依存: 11、17、`mizar-parser` task 22。仕様: `labels.md`、
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)。

### シグネチャ収集

19. **仕様: `symbols.md`。** [x]
    - シグネチャ収集の仕様を執筆する（英語と日本語、コードなし）: 宣言パス
      契約（型検査なし）、種別ごとのシグネチャ形状、重複・不正宣言の
      ポリシー、整形や無関係な局所編集で安定する正規化された semantic
      origin。
    - 依存: 3。仕様: アーキテクチャ 03「Step 5」、
      [11.symbol_management.md](../../../spec/ja/11.symbol_management.md)。
    - R-019 で完了: `symbols.md` は resolver-owned な signature collection
      契約、stable symbol origin、symbol-bearing shell の分類、algorithm を含む
      kind ごとの opaque / structural payload、duplicate/conflict と overload
      policy、exported summary と lexical-summary projection、recovery、
      relation/dependency edge、R-020/R-021/R-023 への test handoff を仕様化した。

20. **収集骨格と重複検出。** [x]
    - 宣言シェルから `SymbolEnv` を構築する: 種別ごとの名前登録、重複・
      競合診断、オーバーロード候補のグループ化。シグネチャ自体はまだ
      不透明のままでよい。
    - テスト: 種別ごとの重複検出。候補グループ化。決定的な診断順。
    - 依存: 5、11、19。仕様: `symbols.md`。
    - R-020 で完了: `src/symbols.rs` は explicit な
      `DeclarationShellId` keyed projection seam、opaque symbol collection の
      `SymbolIndex` / `DefinitionIndex` / `RegistrationIndex` / `OverloadIndex` への
      登録、internal duplicate / illegal-overload diagnostic、recovered と
      context-only shell policy、contribution tracking、決定的な unit test を追加した。
      専用 lexical-summary data shape は R-021 で完了済みである。
      artifact-backed summary consumption は R-024 で canonical な `mizar-artifact`
      `ModuleSummary` consumer として実装済みである。

21. **種別ごとのシグネチャ抽出。** [x] — `mizar-parser` task 23-31 が律速。
    - 具体的なシグネチャ（struct、mode、attribute、predicate、functor、
      algorithm、theorem、registration、template、および synonym / antonym /
      redefinition などの relation declaration）を増分で抽出する: 各増分は、
      その宣言種別を生成するパーサー文法タスクの後に、独立した変更として
      着地させる。最後の対の増分が着地した時点でチェックを付ける。
    - 増分ごとのテスト: その種別のシグネチャ形状フィクスチャと
      `SymbolEnv` 参照。
    - 依存: 20。`mizar-parser` task 23-31 と対になる。仕様: `symbols.md`。
    - R-021 で完了: `SignatureProjectionExtractor` は表現済み parser-backed
      declaration shell を parser-owned opaque signature payload 付きの
      `SymbolDeclarationProjection` へ lower し、template role を owning
      declaration payload に保持し、exported lexer-visible spelling 用の
      `ModuleLexicalSummaryIndex` entry を seed する。module-level scheme
      declaration は、parser/syntax が scheme declaration shell を公開するまで
      external source-role gap として残る。

### 強化と横断フォローアップ

22. **recovered 構文ポリシー。** [x]
    - resolver の各段が recovered な `SurfaceAst` 部分木をどう扱うか
      （スキップ、シェルのみ、診断）を定義・実装する。`mizar-syntax` の
      `recovered` フラグ契約を維持する。
    - テスト: recovered 入力で解決が panic しない。recovered 領域から診断が
      連鎖しない。
    - 依存: 13。仕様: [mizar-syntax recovery.md](../../mizar-syntax/ja/recovery.md)。
    - R-022 で完了: [recovery.md](./recovery.md) を追加し、resolver-local な
      recovered-subtree 検出を集約した。name / label / symbol diagnostics は、
      degraded table/env fact を保持しつつ、recovered origin または shell からの
      dependent semantic diagnostics を抑制する。

23. **stage `declaration_symbol` のコーパスランナー。** [x]
    - `tests/miz/{pass,fail}/` のケースを stage `declaration_symbol` で
      ハーネスに接続し、`spec_trace.toml` のカバレッジ項目を付ける。
      declaration-symbol 経路の初期 spec-derived pass/fail set をシードし、
      40/60 の pass/fail 比率へ向けたより広い semantic corpus growth は
      明示的な follow-up coverage として記録する。
    - 依存: 20。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)、
      [traceability.md](../../mizar-test/ja/traceability.md)。
    - R-023 で完了: `mizar-test declaration-symbol` は
      `active_declaration_symbol` tag を持つ
      `stage = "declaration_symbol"` / `expected_phase = "resolve"` の active
      `.miz` expectation を発見し、frontend と resolver の declaration-shell、
      signature-projection、symbol-collection 経路を実行する。fail case は
      public resolver diagnostic code を創作せず、`diagnostic_payloads` /
      `stable_detail_key` の crate-local internal detail key と比較する。seed
      corpus coverage は、parser-backed declaration、visibility、theorem/lemma
      symbol の pass smoke fixture 1 件と、same-scope label uniqueness 由来の
      duplicate-theorem fail fixture 1 件を `spec_trace.toml` requirement とともに
      追加した。
    - post-task-20 R-G007 increment: 同じ active runner は、resolver-owned
      internal `SameSignatureReturnConflict` class と
      `declaration_symbol.signature.same_signature_return_conflict` detail key
      を使い、same argument-signature definition が異なる return signature を持つ
      parser-backed functor signature-conflict seed も実行する。task 9〜19 の
      より広い semantic import/name/label corpus の拡充は R-G007 test-gap
      follow-up として将来の runner assertion 拡張に記録するが、実行可能な
      declaration-symbol runner は traceable active case 3 件を持つ。

24. **ModuleSummary の再利用。** [x]
    - 依存モジュールを（schema-version を検証した）`ModuleSummary` artifact
      として消費し、ソースを再読込しない。summary が無い・非互換のときは
      ソース解決へフォールバックする。
    - テスト: summary 経由とソース経由の解決が共有フィクスチャ上で一致
      する。非互換スキーマは診断付きでフォールバックする。
    - 依存: 20、`mizar-artifact` task 5。仕様: アーキテクチャ 03「Module
      Summary」、[18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。
    - R-024 で完了: `src/module_summary_reuse.rs` は artifact-owned reader 経由で
      canonical な `mizar-artifact` `ModuleSummary` JSON を消費し、検証済みの
      exported symbol、label、lexical entry、re-export、dependency interface
      reference を `SymbolEnv` summary contribution index へ project する。
      summary が存在しない、incompatible、または unsupported な場合は deterministic な
      crate-local fallback record を生成する。resolver は artifact schema、reader、
      writer、hash framing、public diagnostic、artifact-only dependency module の
      source loading を定義しない。

25. **決定性スイート。** [x]
    - 同一入力が同一の id、テーブル、診断順、debug レンダリングを生むことの
      プロパティ的検証。frontend のスイートに倣う。
    - 依存: 21。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。
    - R-025 で完了: crate root の determinism regression を追加し、等価な
      public-seam input を二度構築して import graph resolution、name diagnostic
      order、`ResolvedAst` debug rendering、`SymbolEnv` debug rendering を比較する。
      `resolved_ast`、`env`、`imports`、`names`、`labels`、`symbols` 内の詳細な
      id / table / candidate / diagnostic ordering は、既存の module-local determinism
      test が引き続きカバーする。

26. **公開 enum の前方互換性ポリシー。** [x]
    - 各公開 enum に `mizar-frontend` task 25 の決定手続きを適用し、所有
      モジュール仕様の enum の隣に各決定を記録する。
    - 依存: 21。仕様: 全モジュール仕様。
    - R-026 で完了: `resolved_ast`、`env`、`imports`、`declarations`、`names`、
      `labels`、`symbols` の resolver-owned public enum を監査した。すべての
      public enum は `#[non_exhaustive]` のままで、exhaustive exception はない。
      所有 module spec に decision と下流 consumer の wildcard/fallback requirement を
      記録し、これら spec-owned module における将来の public-enum 追加は
      `mizar-resolve` lint test で guard する。

27. **ソース/仕様対応監査。** [x]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 26。仕様: 全モジュール仕様と本 TODO。
    - R-027 で完了: [source_spec_correspondence.md](./source_spec_correspondence.md)
      に公開 API family、behavior boundary、task requirement、follow-up の
      traceability を記録した。監査では unclassified な blocking/high
      `spec_gap`、`test_gap`、`source_drift`、`source_undocumented_behavior`、
      `test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict` は
      見つからなかった。既存の分類済み record として、R-G001 public resolver
      diagnostic code-space `spec_gap`、R-G007 が精緻化する R-G002 historical
      semantic corpus coverage `test_gap`、R-G006 parser/syntax scheme-role
      dependency は残る。post-task-20 R-G007 signature-conflict slice は symbol
      assertion increment の 1 つを閉じるが、import/name/dot-chain/label active
      assertion は未完了である。R-G003 deferred `ModuleSummary` reuse は R-024 で解消済みである。

28. **二言語ドキュメント同期監査。** [x]
    - `doc/design/mizar-resolve/en/` の各英語正本と日本語版を比較し、API
      一覧、状態、用語、リンク、挙動の約束を同期する。
    - 依存: 27。仕様: リポジトリのドキュメント方針。
    - R-028 で完了: [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
      に、英日 design-document pair の checklist を記録した。監査では public API
      family、enum policy、task state、deferred / external-dependency record、
      behavior promise、boundary statement、terminology、resolver task handoff
      wording に残る不一致は見つからなかった。`doc/spec`、`.miz`、expectation、
      source file は変更していない。

29. **module 境界リファクタリング gate。** [x]
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
    - R-029 で完了: [module_boundary_refactor.md](./module_boundary_refactor.md)
      に source-layout audit と behavior-preserving split を記録した。public module path と
      API は変更せず、inline unit test は module ごとの private `tests.rs` file へ移した。
      deterministic snapshot helper は `env/snapshot.rs` と
      `resolved_ast/snapshot.rs` へ、resolved-AST validation は
      `resolved_ast/validation.rs` へ、crate-local name diagnostic assembly は
      `names/diagnostics.rs` へ移した。移動した API について source/spec と二言語
      ドキュメント監査 scope を再実行し、新しい drift は見つからなかった。

30. **public resolver diagnostic adoption gate。** [ ]
    - 実際の user-facing producer adoption task が始まる場合に限り、resolver
      の name/import/label diagnostics を public `mizar-diagnostics`
      descriptor へ写像する。共有 registry はすでに `Resolution` family を
      reserve しているが、この task では具体的な semantic name、numeric code
      または alias、crate-local diagnostics からの migration behavior、
      corpus / expectation coverage、LSP / artifact projection boundary を定義する。
    - 依存: R-024、および user-facing resolver diagnostics を必要とする最初の
      downstream consumer。仕様:
      [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md),
      [mizar-diagnostics consumer adoption](../../mizar-diagnostics/ja/consumer_adoption_decision.md),
      [spec_coverage_audit.md](../../spec_coverage_audit.md)。
    - 禁止事項: placeholder adapter を作らない。registry/spec alignment なしに
      public code を創作しない。現在の crate-local diagnostics に合わせるためだけに
      既存 expectation sidecar を rebaseline しない。

## crate close-out

- 完了: [crate_exit_report.md](./crate_exit_report.md) に、non-deferred task completion、
  当初の R-024 deferral、R-024 follow-up implementation overlay、milestone gate、
  quality score 94/100、full verification、human-review surface、task commit、
  next-task handoff を記録した。R-030 は spec coverage audit により開かれた
  後続 integration follow-up であり、完了済みの R-001 から R-029 milestone を
  再オープンしない。

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
