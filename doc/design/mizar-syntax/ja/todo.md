# mizar-syntax TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| ast | [ast.md](./ast.md) | `src/ast.rs` | [~] rowan storage 境界は導入済み、語彙は拡張中 |
| trivia | [trivia.md](./trivia.md) | `src/trivia.rs` | [x] task 4 のモデルは実装済み、task 5 の item attachment fixture は着地済み |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [x] task 5 の recovery 語彙は実装済み、parser producer は段階的に追加 |

`mizar-syntax` はデータ定義 crate である。`mizar-parser`、`mizar-frontend`、
および将来の resolver / LSP / formatter の消費者が共有する `SurfaceAst` の形を
所有し、構文解析ロジックも意味論も一切所有しない。目標となる構文木バック
エンドは `rowan` である。初期の表現作業では rowan-backed な green tree を
採用し、移行中に現在の最小境界を保つために必要な互換 wrapper だけを残して
よい。構築は 2 つの波で行う。まず表現基盤（rowan storage 境界、レンダリング、
trivia、recovery 語彙）、次にノード語彙で
あり、後者は `mizar-parser` の文法タスクと歩調を合わせて成長する。

依存順序: `ast` 基盤 → `trivia` / `recovery` → 文法整合性ゲート →
`mizar-parser` と対になるノード語彙。

## crate の前提条件

この crate は `mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`）と、
immutable green-tree storage のための `rowan` に依存する。task 11/12 の
`SurfaceNode` 風 data は `SurfaceAst` 内部に private に保持し、rowan-backed 表現上の
exported compatibility type、read-only accessor、typed view、`SurfaceNode`
constructor / field を含む一時的な公開互換 API を公開する。ここに `salsa` は追加しない:
query engine は frontend / build / resolver / checker 層の責務であり、
この crate は immutable で query-friendly な構文データ境界にとどめる。
task 11/12 の最小境界（`SurfaceAst`、`SurfaceNode`、recovery kind、
`SyntaxDiagnostic`）はすでに `mizar-parser` と
`mizar-frontend::parsing::MizarParserSeam` が消費しているため、ここでの変更は
同じ変更で `cargo test -p mizar-parser` と `cargo test -p mizar-frontend` を
成功状態に保たなければならない。

## 解決済みおよび保留中の決定

- **構文木バックエンド: 解決済み。** `SurfaceAst` は rowan-backed な green tree
  を所有する。既存の task 11/12 名に対する互換 wrapper は、private に保持された
  data 上の exported type、read-only accessor、typed view、`SurfaceNode`
  constructor / field として一時的に公開するが、parser task 5〜7 は custom arena
  backend ではなく `SurfaceAstBuilder` と typed accessor 境界に対して成長させる。
- **trivia の所有権: 解決済み。** `mizar-frontend` はコメント／ドキュメント
  コメントの抽出、raw doc-comment body、字句用テキスト、preprocess map を
  所有する。`SurfaceAst` は、その frontend 所有データを `SourceRange` と
  構文的 attachment hint で参照する、syntax-owned trivia side table を持つ。
- **salsa 統合: この crate では保留、後段では必須。** `salsa` は compiler の
  query / cache 層で必須であり、`mizar-syntax` には入れない。後続の
  `salsa` query がこの crate の意味論なし境界を変えずに `SurfaceAst` を
  出力値として扱えるよう、`SurfaceAst` を immutable、決定的、共有しやすい形に
  保つ。
- **ドットの役割の surface 形状: `mizar-parser` task 10 により parser/syntax では解決済み。**
  パーサーは resolver なしでは selector access と namespace 区切りを完全には
  分離できない（仕様 [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)）。
  AST は dotted qualified-name head を qualified surface として保持し、すでに parse
  済みの term の後にある `.` を selector/update postfix として parse する。
  scope-dependent な selector-versus-namespace classification は resolver-owned のまま
  残す。トップレベル（[../../todo.md](../../todo.md)「Resolved And Open Decisions」）
  にも登録済みで、[../../mizar-parser/ja/todo.md](../../mizar-parser/ja/todo.md)
  で管理する。
- **公開 enum の前方互換性: S-017 で解決済み。** 最終 node-vocabulary audit
  は `ast`、`recovery`、`trivia` のすべての public enum を分類する。成長し得る
  vocabulary / recovery / trivia enum は `#[non_exhaustive]` のままとし、owning
  module spec に列挙した小さな grammar-marker / payload enum は、文書化済みの
  exhaustive 例外として残す。今後 public enum を追加する場合は、着地前に
  lint-policy classification のどちらか一方へ必ず追加する。

## 順序付きタスク一覧

表現基盤 task は、単独で実装・テスト・コミットできる粒度になっている。後続の
node vocabulary entry は tracking bucket である。対になる parser 増分ごとに、
個別の tested change として着地させ、bucket は最後の対になる増分が着地した時点で
check off する。各変更の後で `cargo test -p mizar-syntax` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 表現基盤

1. **モジュール分割と lint 方針のガード。** [x]
   - `src/lib.rs` を `pub mod ast;` と `pub mod recovery;` に分割し、task 12 の
     型を挙動変更なしに移動する。`mizar-parser` と `mizar-frontend` のパスが
     有効なままになるよう、crate ルートからすべて再エクスポートする。
   - `mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加する:
     workspace lint へのオプトイン、deny ベースライン、将来の `allow` の隣に
     根拠を必須とする。
   - テスト: 既存の消費者が変更なしにコンパイルできる。lint 方針ガードが通る。
   - 依存: なし。仕様: [ast.md](./ast.md)、[recovery.md](./recovery.md)。

2. **`rowan` storage 境界と builder / accessor API。** [x]
   - rowan-backed な `SurfaceAst` green-tree 表現を採用し、決定と根拠を
     [ast.md](./ast.md) に記録する。既存の `SurfaceNode` / `SurfaceNodeId` 名を
     互換のために残す場合は、それらを storage backend ではなく rowan-backed
     表現の wrapper または view として文書化する。
   - raw `SyntaxKind` / node-kind mapping、node / token role の規約、範囲の
     attachment ルール、resolver / checker の消費者から rowan 内部を隠す
     型付き accessor / view ヘルパーを定義する。ただし公開が意図的な場合は
     文書化する。
   - `mizar-parser` が使う `SurfaceAstBuilder` 境界を定義する。parser コードは
     具体的な arena へ push したり rowan の raw tree 形状に依存したりせず、
     builder / event API 経由でノードと recovery マーカーを構築する。
   - identity ルールを記録する: rowan green-node identity と密な index は内部
     キャッシュ詳細であり、安定した artifact id ではない。決定的 snapshot と
     content cache key が安定性の表面である。
   - テスト: rowan-backed tree への builder round-trip。現在の全 node / token
     kind に対する型付き accessor の網羅。文書化された recovery の例外を除き、
     親の範囲が子の範囲を包含すること。繰り返し構築が決定的 snapshot を生むこと。
   - 依存: 1。仕様: [ast.md](./ast.md)「Public API」。

3. **決定的なスナップショットレンダリング。** [x]
   - [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)
     「スナップショットテスト」が要求するコーパスのスナップショット
     ベースラインのために、`SurfaceAst` の安定した人間可読テキスト
     レンダリング（kind、範囲、recovered フラグ、子をインデント表示）を
     追加する。
   - レンダリングは実行間・プラットフォーム間でバイト同一でなければならず、
     ハッシュマップの反復順序やアドレスなどの非決定性を含まない。
   - テスト: 繰り返しレンダリングで同一の出力。現在の全ノード種別を網羅する
     代表 fixture。recovery ノードが視認できる形で印付けされる。
   - 依存: 2。仕様: [ast.md](./ast.md)。snapshot envelope / update policy は
     [../../mizar-test/ja/snapshot.md](../../mizar-test/ja/snapshot.md)。具体的な
     `SurfaceAst` body layout は [ast.md](./ast.md)「Snapshot rendering」が所有する。

4. **trivia モデル。** [x]
   - `pub mod trivia;` を追加する。`mizar-frontend::PreprocessedSource`
     （すでにコメント・ドキュメントコメント抽出を所有する）との所有権分担を
     決定・記録し、その上で trivia の attachment を定義する: ドキュメント
     コメントの付着先、スキップされたトークン範囲、formatter と LSP の
     消費者が必要とする空白依存ヒント。
   - ドキュメントコメントの attachment は構文的なものにとどめ、意味的解釈を
     持ち込まない。
   - テスト: trivia の所有権と attachment ヒント。スキップ範囲がソース範囲と
     ともに保持される。要求時に trivia を含むレンダリングが決定的である。
     「ドキュメントコメントが直後の item ノードへ付着する」具体 fixture は、
     task 9 / S-009 と parser task 5 の最初の item-node 増分で着地させる。
   - 依存: 2、3。仕様: [trivia.md](./trivia.md)。

5. **recovery 語彙の拡張。** [x]
   - `SyntaxRecoveryKind` を task 12 の最小（`ErrorToken`、`MissingEnd`、
     `MissingStringLiteral`）から、[recovery.md](./recovery.md) が約束する
     完全な語彙へ拡張する: 欠落した構文要素、スキップされたトークン、
     対応しない区切り記号、不正な注釈。
   - `recovered` フラグの契約を維持する: resolver と checker のフェーズが、
     再解析なしに recovered な部分木をスキップまたは拒否できること。
   - テスト: 各 recovery kind が正しい範囲で構築できる。recovered 部分木の
     問い合わせヘルパー。スナップショットレンダリングが各 kind を区別して
     印付けする。
   - 依存: 2。仕様: [recovery.md](./recovery.md)。

### consumer 前の互換性ゲート

**公開 enum 前方互換性の初期ゲート。** [x]
- phase 3 境界で利用可能な各公開 enum（`SyntaxKind`、`SurfaceNodeKind`、
  `SurfaceTokenKind`、`SurfaceOperatorAssociativity`、`SyntaxRecoveryKind`、
  `SyntaxDiagnosticCode`、および task 4 で導入される trivia の kind）について、
  `mizar-frontend` task 25 の手続きで `#[non_exhaustive]` 対 意図的 exhaustive を
  決定する。
- 各決定を所有モジュール仕様の enum の隣に記録し、parser task 5〜7 によって
  resolver / LSP の消費者が現実的になる前に属性を適用する。
- 結果: この初期ゲートは、現在の方針については S-017 の最終監査により置き換えられた。
  `SyntaxKind`、`SurfaceNodeKind`、`SurfaceTokenKind`、`SyntaxRecoveryKind`、
  `SyntaxDiagnosticCode`、`TriviaAttachmentTarget`、`SkippedTokenReason`、
  `WhitespaceHintKind` を `#[non_exhaustive]` として gate する。
  `MizarLanguage`、`SurfaceOperatorAssociativity`、
  `SurfaceFormulaPrefixOperator`、`SurfaceFormulaConnective`、
  `SurfaceQuantifierKind`、`SurfaceFormulaConstant`、`TriviaPlacement` は、
  文書化された意図的 exhaustive 例外である。
- 依存: 4、5。仕様: [ast.md](./ast.md)、[trivia.md](./trivia.md)、
  [recovery.md](./recovery.md)。

### 文法整合性ゲート

これらのタスクは、意図的にノード語彙トラックへ割り込む。文法監査の指摘が
解消されるか、受け入れ済み follow-up として明示的に記録されるまでは、現在の
task 12 互換 surface を超える新しい AST node kind 設計を始めない。目的は、
文法 drift を `SurfaceAst` の node kind、child role、snapshot baseline として
固定してしまうことを避けることである。

6. **正本文法の整合性監査。** [x]
   - [Appendix A](../../../spec/ja/appendix_a.grammar_summary.md) を parser 向けの
     正本文法サマリとして扱い、第 2〜21 章の章内 syntax block と照合する。
   - 未定義 nonterminal、重複定義、`compilation_unit` から到達不能な production、
     文書化された precedence parser の外にある直接左再帰、章間 drift、
     予約 token の不一致、AST 形状に影響する曖昧な境界を確認する。
   - 指摘は grammar audit note または関連仕様ファイルへ、具体的な参照とともに
     記録する。分類は、AST 設計前に修正、semantic-only issue として受容、
     owner と parser task 付きで defer、のいずれかとする。
   - 結果: [grammar_audit.md](./grammar_audit.md) に記録した。監査で見つかった
     具体的な parser-facing 修正として Appendix A と第 2 章を同期した:
     built-in predicate は到達可能になり、annotation は parser-owned な
     statement / item wrapper 経由で付着し、`claim` は algorithm statement では
     なく top-level、`is` assertion は resolution まで generic に保持し、
     `character` は定義済み、`step` / `..` の token drift は仕様レベルで正規化
     済みである。lexer table / test の同期は `mizar-lexer` トラックで完了し、
     該当する lexical coverage entry は covered になった。残る AST 前の論点と
     Task 7 fixture input は監査ノートで分類した。
   - 依存: Appendix A normalization。仕様:
     [../../../spec/ja/appendix_a.grammar_summary.md](../../../spec/ja/appendix_a.grammar_summary.md)、
     [../../../spec/ja/](../../../spec/ja/00.index.md) 配下の章内 grammar section。

7. **parse-only acceptance matrix と fixture 計画。** [x]
   - AST snapshot を設計する前に parse-only acceptance matrix を定義する:
     module structure、declaration、type expression、term expression、formula、
     statement / proof、annotation、registration、template、algorithm について、
     positive、negative、ambiguous、recovery-required の例を並べる。
   - この段階では期待値を最終 AST 形状に依存させない。期待結果は、構文上の
     accept、reject、または recovery category と、対象の grammar rule とする。
   - どの fixture が `mizar-parser` に属し、どれが `mizar-test` に属し、どれが
     純粋な仕様例かを識別する。後続の AST snapshot が安定した fixture set を
     継承できるよう、Appendix A section への traceability を記録する。
   - 結果: [parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md)
     に記録した。matrix は syntax-only outcome（`accept`、`reject`、
     `ambiguous-preserve-surface`、`recover`）を使い、ambiguous 行は executable
     corpus expectation では通常の parse acceptance に対応させる。fixture
     ownership と、各対象領域の Appendix A traceability も記録した。
   - 依存: 6。仕様:
     [../../mizar-test/ja/staged_model.md](../../mizar-test/ja/staged_model.md)、
     [../../mizar-test/ja/expectation_schema.md](../../mizar-test/ja/expectation_schema.md)。

8. **初期 parse-only grammar fixture seed。** [x]
   - 最初の小さな parse-only fixture seed を追加する。parser support がまだ
     十分でない場合は、選定済み case を変更せずに後で有効化できる checked-in
     fixture manifest / design note を追加する。
   - task 6 で特定した高リスク grammar boundary を少なくともカバーする:
     term-vs-formula 境界、dot chain、statement reachability、import prelude 形、
     文脈依存 string literal、`qua`、`the`、`reconsider`、delimiter または
     `end` 欠落周辺の recovery。
   - まだ最終 AST node snapshot は要求しない。AST snapshot は、対応する node
     vocabulary 増分が node kind、child role、range rule、recovery rendering を
     定義した後に追加する。
   - 結果: [parse_only_fixture_seed.md](./parse_only_fixture_seed.md) に記録した。
     現在の parser readiness は full grammar corpus を default discovery で実行する
     には十分でないため、Task 8 seed は checked-in fixture manifest とし、選定済み
     case ID、source shape、parse-only expectation、有効化先を安定させた。この
     seed は Task 7 の Fixture 有効化計画を優先し、`qua`、`reconsider`、
     string-required annotation rejection boundary の補助行を追加する。AST snapshot
     は追加していない。
   - 依存: 7。仕様:
     [../../../spec/ja/appendix_a.grammar_summary.md](../../../spec/ja/appendix_a.grammar_summary.md)、
     [../../mizar-test/ja/layout.md](../../mizar-test/ja/layout.md)。

### ノード語彙（`mizar-parser` の文法タスクと対）

各領域のノード種別は**増分的に**追加する: 各増分は、それを構築する
`mizar-parser` 文法タスクと同じ変更で着地し（変更の粒度はパーサー todo の
番号付けが統制する）、各増分はスナップショットレンダリングを拡張する。以下の
語彙タスクは、対になるパーサータスクの最後が着地した時点でチェックを入れる。
それを構築するパーサータスクに先行して、投機的にノード種別を追加しない。また、
task 6〜8 が文法監査と parse-only fixture 計画を作成するまでは、これらのタスクを
開始しない。各増分では、まず [ast.md](./ast.md) の語彙増分の契約を拡張し、node kind、payload、
child role、range rule、accessor、snapshot、recovery / trivia との相互作用を
記録しなければならない。仕様参照は [doc/spec/ja/](../../../spec/ja/00.index.md)
配下の規範的な文法章である。

9. **モジュール、item、shared path のノード。** [x] — `mizar-parser`
   task 4〜7 と対。
   - import parsing の前に parser task 4 が必要とする shared qualified-symbol /
     namespace-path ノード。モジュールファイルの形、トップレベル item リストと
     キーワードでディスパッチ可能な item 種別（parser task 5）。alias と相対
     prefix を持つ import item（parser task 6）。export と可視性の形（parser
     task 7）。
   - 進捗: parser task 4 により shared path-node 増分が着地した:
     `ModulePath`、`NamespacePath`、`QualifiedSymbol`、`PathSegment`、
     `RelativePrefix`、および parser helper unit coverage。parser task 5 により
     `CompilationUnit`、`ItemList`、`PlaceholderItem`、item-level skipped-token
     recovery trivia、active module-skeleton corpus coverage、最初の doc-comment-to-item
     attachment fixture が着地した。parser task 6 により `ImportItem`、
     `ImportAliasDecl`、`ModuleBranchImport`、import 固有の typed accessor と snapshot
     coverage、`MalformedImport`、active import-item corpus coverage、import-stub
     parse-only harness support が着地した。parser task 7 により `ExportItem`、
     `VisibilityMarker`、`VisibleItem`、export/visibility typed accessor と snapshot
     coverage、`MalformedExport`、`MalformedVisibility`、active export/visibility
     corpus coverage が着地した。S-009 bucket は完了した。
   - 仕様: [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

10. **型式のノード。** [x] — `mizar-parser` task 8 と対。
   - 属性連鎖（`non` を含む）、radix / mode の型ヘッド、`of` / `over` 引数、
     struct 修飾の属性参照。
   - 仕様: [03.type_system.md](../../../spec/ja/03.type_system.md)、
     [§A.3.2](../../../spec/ja/appendix_a.grammar_summary.md)。
   - 結果: `ReserveItem`、`ReserveSegment`、`TypeExpression`、
     `AttributeChain`、`AttributeRef`、`ParameterPrefix`、`TypeHead`、
     `TypeArguments`、`TermPlaceholder` の syntax vocabulary、rowan / snapshot
     coverage、typed accessor、`MalformedTypeExpression`、paired parser task 8 による
     active parse-only corpus coverage を追加した。

11. **項のノード。** [x] — `mizar-parser` task 9〜12、15 と対。
   - task 9 で導入した shared path vocabulary を消費する。続いて一次項（parser
     task 9）、syntax-only dot-role と selector access / update surface
     （parser task 10）、functional structure update、
     `qua`（parser task 11）、task 12 の `InfixExpression` を prefix / postfix
     形へ一般化する演算子式ノード（parser task 12）、Fraenkel / 集合内包形
     （parser task 15）を追加する。一次項の網羅には `it`、選択式
     （`the type_expression`）、構造体コンストラクタ、集合列挙リテラル、
     適用形を含む。
   - 進捗: parser task 9 により primary-term 増分が着地した:
     `TermExpression`、`TermReference`、`NumeralTerm`、`ItTerm`、
     `ParenthesizedTerm`、`ChoiceTerm`、`ApplicationTerm`、
     `StructureConstructor`、`FieldArgument`、`SetEnumeration`、
     `MalformedTermExpression`、`MissingTerm` recovery coverage、active parse-only
     primary-term corpus case を追加した。parser task 10 により syntax-only dot-role
     増分として `SelectorAccess`、`StructureUpdate`、`FieldUpdate`、
     selector/update recovery coverage、active parse-only selector/update corpus case を
     追加した。parser task 11 により `qua` 増分として `QuaExpression`、bracket
     `qua_arg` の `TermPlaceholder` からの移行、`MissingTypeExpression` target
     recovery coverage、active parse-only `qua` corpus case を追加した。parser task
     12 により operator-expression 増分として `PrefixExpression`、
     `PostfixExpression`、active prefix/postfix/infix Pratt grouping、
     non-associative と dangling operator 診断、active parse-only operator corpus case を
     追加した。parser task 15 により Fraenkel / set-builder 増分として
     `SetComprehension`、`ComprehensionVariableSegment`、`SetEnumeration` との
     top-level `where` による分岐、generator / type / condition 欠落と brace
     欠落の recovery、active parse-only set-comprehension corpus case、
     rowan / snapshot / typed-accessor coverage を追加した。S-011 は documented
     parse-only term surface について完了した。binder identity、sethood、
     capture、mapper typing、elaboration は後続の semantic work に残る。
   - 仕様: [13.term_expression.md](../../../spec/ja/13.term_expression.md)、
     [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)。

12. **論理式のノード。** [x] — `mizar-parser` task 13〜14 と対。
   - 原子述語適用、および resolution が後で type assertion または
     attribute assertion に分類する generic `is` assertion（parser task 13）。
     結合子と量化子（`for` / `ex` / `st` / `holds`）（parser task 14）。
   - 結果: parser task 13〜14 は実装済みである。atomic formula node、
     generic `is` assertion、formula constant、prefix/binary formula node、
     parenthesized formula、quantifier variable segment、quantified formula、
     missing-formula recovery、theorem/lemma formula host、syntax
     typed accessor、parser unit test、active parse-only pass/fail corpus
     coverage を追加した。template predicate argument は task 31 / S-016 まで
     deferred だったが現在は表現済みであり、formula を埋め込む Fraenkel /
     set-builder term は parser task 15 / S-011 で実装済みである。
   - 仕様: [14.formulas.md](../../../spec/ja/14.formulas.md)。

13. **文のノード。** [x] — `mizar-parser` task 16 と 18〜21 と対。
    - 単純文 `let`、`assume`、`take`、`set`、`given`（parser task 16）。
      top-level `reserve` は、Chapter 4 が block-local `reserve` shaped statement を禁じているため
      既存の `ReserveItem` path のままにする。`consider` / `reconsider`（parser task 18）。
      `thus` / `hence`、`then` 連鎖、逐次的等式 `.=`（parser task 19）。
      compact equality statement と zero-step iterative equality の dispatch
      境界（grammar audit G-AUD-010）。
      `now` / `hereby` と `per cases` / `suppose` ブロック（parser task 20）。
      `deffunc` / `defpred` のローカル定義（parser task 21）。
    - 結果: parser tasks 16、18、19、20、21 は実装済みである。
      `ConsiderStatement`、`ReconsiderStatement`、`ReconsiderItem` は、
      shared-type `consider` variable、condition list、必須の simple
      justification、reconsider item list、target type、task-18 recovery、
      scope-skeleton の `type_change_list` support、syntax typed accessor、
      parser unit test、active parse-only pass/fail corpus coverage を覆う。
      `ConclusionStatement`、`ThenStatement`、
      `IterativeEqualityStatement`、`IterativeEqualityStep` は、
      `thus` / `hence`、linkable な `then` statement、逐次的等式 `.=` step、
      label / `then` variant、G-AUD-010 dispatch 境界、parser unit test、
      active parse-only pass/fail corpus coverage を覆う。
      `NowStatement`、`HerebyStatement`、`CaseReasoningStatement`、
      `CaseItem`、`SupposeItem` は、`now` / `hereby` block、`per cases`
      branch block、optional explicit `per cases by` justification、homogeneous
      な `case` / `suppose` branch、`then per cases`、block-end recovery、
      parser unit test、active parse-only pass/fail corpus coverage を覆う。
      `InlineFunctorDefinition`、`InlinePredicateDefinition`、
      `TypedParameter` は local `deffunc` / `defpred` definition、
      standalone 専用 dispatch、`be` / `being` typed parameter、
      zero-argument definition、delimiter / name / type / body recovery、
      parser unit test、syntax typed accessor、active parse-only pass/fail
      corpus coverage を覆う。S-013 は完了である。
    - 仕様: [15.statements.md](../../../spec/ja/15.statements.md)。

14. **定理・証明・正当化のノード。** [x] — `mizar-parser` task 17 と 22 と対。
    - 正当化句（`by`）、`.{ … }` と `.*` を含む引用形、`let ... by references`、
      最小の明示的 compact-statement host に加え、`by computation(...)` オプションノード
      （parser task 17）。`theorem` / `lemma` の item、ラベル、`proof … end` の入れ子
      （parser task 22）。canonical
      Chapter 15 / 16 grammar は `from` を justification form として定義していないため、
      以前の `from` 記述は実装対象の構文ではなく derived-documentation drift として扱う。
      parser task 17 の citation/computation subset は完了済みである。parser task 22 は
      `TheoremItem`、`LemmaItem`、`ProofBlock`、status token preservation、
      visibility-wrapped theorem target、proof-body statement wiring、conclusion と
      compact statement host 上の statement-level proof justification、theorem/proof
      recovery、typed accessor、active parse-only pass/fail corpus coverage を追加した。
      S-014 は完了済みである。
    - 仕様: [15.statements.md](../../../spec/ja/15.statements.md)、
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)、
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)
      §20.9.2。

15. **定義・構造体・registration のノード。** [x] — `mizar-parser`
    task 23〜30 と対。
    - definition ブロック骨格、correctness 条件句、`attr` 定義（parser
      task 23）。`pred` / `func` / `mode` の本体（parser task 24〜26）。
      `redefine`、`synonym` / `antonym`（parser task 27）。property 句
      （parser task 28）。フィールドと継承を持つ `struct` 定義（parser
      task 29）。registration と cluster の形、`reduce`（parser task 30）。
      parser task 23 は最初の S-015 増分として完了済みである。
      `DefinitionBlockItem`、`DefinitionParameter`、`AttributeDefinition`、
      `AttributePattern`、`FormulaDefiniens`、`FormulaCase`、
      `CorrectnessCondition` を、typed accessor、definition block recovery、
      active parse-only pass/fail corpus coverage、attribute definition と
      correctness condition の traceability metadata とともに実装した。
      parser task 24 は predicate definition の増分として完了済みである。
      `PredicateDefinition` と raw `PredicatePattern` を、typed accessor、
      definition-local visibility、formula-definiens body、grammar-shaped
      pattern validation、active parse-only pass/fail corpus coverage、
      traceability metadata とともに実装した。parser task 25 は functor definition
      について完了済みである。`FunctorDefinition`、raw `FunctorPattern`、
      `TermDefiniens`、`TermCase` を、typed accessor、definition-local visibility、
      `means` / `equals` body、return-type と term-definiens recovery、source-level
      circumfix coverage、template placeholder preservation、active parse-only
      pass/fail corpus coverage、traceability metadata とともに実装した。parser
      task 26 は mode definition について完了済みである。`ModeDefinition`、raw
      `ModePattern`、`ModeProperty` を、typed accessor、definition-local visibility、
      syntactic な `TypeExpression` body、mode-pattern parameter preservation、
      `sethood` justification recovery、active parse-only pass/fail corpus coverage、
      traceability metadata とともに実装した。parser task 27 は redefinition と
      notation alias について完了済みである。`AttributeRedefinition`、
      `PredicateRedefinition`、`FunctorRedefinition`、`CoherenceCondition`、
      `NotationAlias`、`NotationPattern` を typed accessor、definition-local
      visibility wrapping、raw alias-pattern preservation、redefinition recovery、
      active parse-only pass/fail corpus coverage、traceability metadata とともに
      実装した。canonical grammar は mode redefinition を定義しないため、
      `redefine mode` は concrete node ではなく placeholder / recovery boundary のまま
      保持する。parser task 28 は property clause について完了済みである。
      `PropertyClause` を typed accessor、canonical predicate / functor / standalone
      `sethood` keyword coverage、general justification recovery、active parse-only
      pass/fail corpus coverage、traceability metadata とともに実装した。canonical
      property production は `transitivity` を property clause として定義しないため、
      これは未実装のまま残す。parser task 29 は structure と inheritance について
      完了済みである。`StructureDefinition`、raw `StructurePattern`、
      `StructureField`、`StructureProperty`、`InheritanceDefinition`、
      `InheritanceTarget`、`FieldRedefinition`、`PropertyRedefinition` を、typed
      accessor、definition-local visibility wrapping、structure parameter preservation、
      field initializer preservation、`extends set` を含む shorthand / explicit
      inheritance coverage、local member recovery、active parse-only pass/fail corpus
      coverage、traceability metadata とともに実装した。structure identity、selector
      fact、inheritance coverage、coherence proof obligation、type narrowing validity、
      constructor は `mizar-syntax` の外に残す。parser task 30 は registration と cluster
      について完了済みである。`RegistrationBlockItem`、`RegistrationParameter`、
      `ExistentialRegistration`、`ConditionalRegistration`、`FunctorialRegistration`、
      `ReductionRegistration` を、typed accessor、cluster / reduction item の
      definition-local visibility wrapping、registration-adjective boundary preservation、
      parameterized registered / target type preservation、restricted functorial payload parsing、
      compound reduction term preservation、registration-local parameter parsing、
      correctness-condition recovery、frontend scope-skeleton の
      registration block support、active parse-only pass/fail corpus coverage、
      traceability metadata とともに実装した。cluster closure、existence / coherence /
      reducibility proof obligation、reduced normal form、nullary functorial disambiguation は
      `mizar-syntax` の外に残す。S-015 bucket は完了済みである。
    - 仕様: [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [07.modes.md](../../../spec/ja/07.modes.md)、
      [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [10.functors.md](../../../spec/ja/10.functors.md)、
      [11.symbol_management.md](../../../spec/ja/11.symbol_management.md)、
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)、
      [05.structures.md](../../../spec/ja/05.structures.md)、
      [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)。

16. **テンプレート・アルゴリズム・注釈のノード。** [x] — `mizar-parser`
    task 31〜35 と対。
    - parser task 31 は完了: テンプレートパラメータ、pattern 側 template
      loci、call-site template arguments、reference / template functor 引数、
      active template parse-only seed と reference citation seed、`nest`
      traceability を追加した。
    - parser task 32 は完了: algorithm ブロック・代入・宣言・ghost 宣言 /
      代入・snapshot・top-level `claim` block・return、active algorithm/claim
      parse-only seed、algorithm body / ghost assignment 用 frontend scope-skeleton
      support を追加した。
    - parser task 33 は完了: if/else、else-if parser-unit coverage、while、
      任意の `step` を持つ `to` / `downto` range loop、`processed` の有無がある
      collection loop、複数 match case、justification 有無の `otherwise` /
      `exhaustive` 終端、`break`、`continue`、active control-flow parse-only seed、
      match `otherwise` block 用 frontend scope-skeleton support。
    - parser task 34 は完了: terminating algorithm、header `requires` /
      `ensures` / `decreasing`、while `invariant` / `decreasing`、range /
      collection `invariant`、`assert`、concrete verification AST vocabulary、
      active verification parse-only seed、重複または順序違反の header clause、
      禁止される `for decreasing`、misplaced loop clause、malformed `TermList`
      measure の recovery。
    - parser task 35 は完了: `@identifier` annotation-marker の lexing handoff、
      `@[...]` library annotation、固定 `@latex` / `@proof_hint` /
      `@show_thesis` / `@show_resolution` / `@suppress` 形式、string literal
      を含む generic annotation argument、standalone `@show_type` / `@eval`
      diagnostic、statement / algorithm / definition / registration /
      claim-theorem host 用 annotation wrapper、active annotation pass/fail
      parse-only seed、`malformed_annotation` diagnostic。
    - 仕様: [18.templates.md](../../../spec/ja/18.templates.md)、
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)、
      [21.source_code_annotation_and_atp.md](../../../spec/ja/21.source_code_annotation_and_atp.md)。

### 横断的フォローアップ

17. **公開 enum の前方互換方針。** [x]
    - 語彙が完成した時点で公開 enum 初期ゲートを再確認し、後続のノード語彙
      増分で追加された公開 enum について、`#[non_exhaustive]` 対 意図的
      exhaustive を決定する。
    - 最終決定を所有モジュール仕様の enum の隣に記録し、残りの属性を適用する。
    - 結果: 初期ゲート後に新しい public enum 型は追加されていなかったが、最終監査で
      現在のすべての public enum が `crates/mizar-syntax/tests/lint_policy.rs` により
      分類済みであることを確認した。owning module spec は、これらを consumer 前の
      仮置きではなく現在の API に対する最終方針として記述する。
    - 依存: 16。仕様: すべてのモジュール仕様。

18. **増分構文再利用の監査。** [ ]
    - 完成した rowan-backed 構文木について、細粒度 incremental parsing と LSP
      再利用への準備状況を監査する: stable syntax-kind numbering 方針、
      trivia / recovery の配置、範囲の attachment、node-role accessor、局所的な
      編集時の部分木 snapshot 挙動。
    - この task は `salsa` を導入しない。後続の query 層が不安定な arena id や
      parser 内部を公開せずに `SurfaceAst` を生成・キャッシュできることを確認する。
    - 依存: 16、17。仕様: [ast.md](./ast.md)、[trivia.md](./trivia.md)、
      [recovery.md](./recovery.md)。

19. **ソース／仕様の対応監査。** [ ]
    - `mizar-frontend` task 16 の監査に倣う: [ast.md](./ast.md)、
      [trivia.md](./trivia.md)、[recovery.md](./recovery.md) のすべての公開
      API と約束された挙動を実装とテストへトレースし、ギャップをフォロー
      アップタスクとして記録する。
    - 依存: 18。仕様: すべてのモジュール仕様と本 TODO。

20. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-syntax/en/` の各英語正本ドキュメントを日本語版と
      比較し、API 一覧、状態、用語、リンク、挙動の約束を同期する。
    - 依存: 19。仕様: リポジトリのドキュメント方針。

21. **rustdoc サマリ。** [ ] 保留。
    - `mizar-frontend` task 26 と同じワークスペースレベルの保留。再着手
      トリガー: フロントエンドパイプラインの外の最初の長命な消費者
      （resolver または `mizar-lsp`）が `mizar-syntax` に対するコーディングを
      始めるとき、またはワークスペースが rustdoc 方針を採用したとき —
      いずれか早い方。
    - 依存: 17。仕様: リポジトリのドキュメント方針。

## 推奨検証

各タスクの後に実行する。

```text
cargo test -p mizar-syntax
cargo fmt --check
cargo clippy -p mizar-syntax --all-targets --all-features -- -D warnings
```

共有境界を移動・拡張するタスクでは、次も実行する。

```text
cargo test -p mizar-parser
cargo test -p mizar-frontend
```

テストが通ったら、ここでタスクにチェックを入れる。

## 注記

- `mizar-syntax` は構文データの形だけを所有する。文法ロジック、名前解決、
  型付け、証明の意味論は持たない。解決済みシンボル id、推論された型、証明
  義務が `SurfaceAst` に現れることはない。
- `rowan` が構文木バックエンドである。parser と consumer のコードは
  ad hoc な arena layout ではなく、`mizar-syntax` の builder / accessor API に
  依存する。
- `salsa` は後続の query / cache 層の関心事である。syntax crate を書き直さず
  導入できるよう、ここでは純粋な phase 境界と immutable な構文 snapshot を
  保つ。
- 語彙の成長は `mizar-parser` の文法タスクがペースを決める。それを構築する
  パーサータスクに先行して、投機的にノード種別を追加しない。
- `SurfaceAst` は内部コンパイラデータであり、安定した外部スキーマではない。
  スナップショットレンダリング（task 3）がコーパスベースラインに対する
  安定性の表面である。
