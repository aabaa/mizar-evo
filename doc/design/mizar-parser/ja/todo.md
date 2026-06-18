# mizar-parser TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| grammar | [grammar.md](./grammar.md) | `src/grammar.rs` | [~] parser task 30 の registration は private な cursor / event 基盤を使用。grammar coverage は段階的に継続 |
| module grammar | [grammar.md](./grammar.md)、[recovery.md](./recovery.md) | `src/module.rs` | [~] grammar 実装は機能的には成熟しているが oversized。task 42 で挙動維持の private module split を追跡 |
| pratt | [pratt.md](./pratt.md) | `src/pratt.rs` | [~] task 12 の active prefix/postfix/infix operator に対する項 Pratt は実装済み。task 14 の固定 formula Pratt は項 fixity から分離して実装済み |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [~] parser task 30 の registration recovery と nested block-end matching は task 2 cursor / diagnostic / sync helper を使用 |

`mizar-parser` は構文文法を実装する: frontend 適合済みトークンを入力とし、
`mizar_syntax::SurfaceAst` と構文診断を出力する。薄い基盤層（cursor、同期、
構文イベント / builder 送出、recovery 送出、コーパスランナー）をまず作り、
その後は一度に数個の生成規則ずつ文法を成長させる。各文法タスクは
`mizar-syntax` のノード語彙の増分およびコーパス拡張と対にし、残りの文法を
抱え込まずに 1 タスクを自律的に実装・テスト・コミットできるよう、意図的に
小さく切ってある。

## crate の前提条件

この crate は `mizar-session` と `mizar-syntax` にのみ依存する。トークンは
`mizar-frontend` によって曖昧性解消済みで、session の `SourceRange` 付きで
到着する。パーサー支援字句解析は、事前計算された
`ParserLexingPlan` / `StringRequiredContext` 契約を通じてのみ行われる
（トップレベルで解決済み。[../../todo.md](../../todo.md)「Resolved And Open
Decisions」を参照）。`ParseRequest` は演算子 fixity テーブルと
string-required 文脈を運ぶ。task 12 では fixture lexical-summary fixity metadata を公開し、
active parse-only case が合成 parser input ではなく frontend-visible source path で
operator fixity を検証できるようにする。コーパスハーネス（`mizar-test`）と
コーパスツリー（[tests/README.md](../../../../tests/README.md)）はすでに
存在する。

`mizar-parser` は ad hoc な `SurfaceAst` arena layout に依存してはならない。
storage backend は `mizar-syntax` task 2 の責務であり、そのターゲットは
rowan-backed syntax である。parser コードは `mizar-syntax` の builder / event
境界を通して構文木を構築し、テストでも文書化された accessor のみを消費する。
また parser は `salsa` に依存しない。後続の query 層が `ParseRequest ->
ParseOutput` を純粋な query として包めるよう、global state、隠れた cache、
resolver / build-system 依存を避ける。

## テストコーパス方針

十分なコーパス網羅が、この crate の成功基準である。各文法タスクは、同じ
変更で次を提供する。

- 新しい生成規則とその recovery 挙動に対する **crate ユニットテスト**。
- `tests/miz/pass/parser/` と `tests/miz/fail/parser/` 配下の
  **コーパステスト**。5〜30 行の `.miz` ファイルに、stage `parse_only` の
  `.expect.toml` サイドカーを付け、命名規約
  `pass_parser_<topic>_NNN.miz` / `fail_parser_<topic>_NNN.miz` に従う
  （[tests/README.md](../../../../tests/README.md)、
  [staged_model.md](../../mizar-test/ja/staged_model.md)）。
- 各ケースをそれが固定する仕様節へ対応付ける `tests/coverage/spec_trace.toml`
  の **coverage エントリ**
  （[traceability.md](../../mizar-test/ja/traceability.md)）。

例外: 単独の文法位置を持たないヘルパータスク（例: task 4 の修飾シンボル）
は、自身の変更ではユニットテストを提供し、どの後続タスクがコーパス網羅を
届けるかを注記する。その後続タスクは、ヘルパー分のコーパスケースを明示的に
列挙しなければならない。

[architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)
の推奨 pass/fail 比率（全体で pass 40% / fail 60%）へ向けて成長させる:
受理されるすべての形に対して、診断付きで拒否または回復されなければならない
不正な対応物を少なくとも 1 つ用意する。recovery ケースは「クラッシュしない」
だけでなく、診断と回復後の `SurfaceAst` の形の両方をアサートする。

## レビュー監査由来の parser coverage backlog

`tests/coverage/spec_trace.toml` に記録された文法/VC レビューのフォローアップ
には、所有する文法タスクが着地した時点で実行可能にすべき parser-facing ケース
が含まれている。parse-only ランナーと該当生成規則が存在する前に、これらを
即時の coverage 義務として扱わない。

- template 引数: task 31 で完了。`pass_parser_template_arguments_001`、
  `pass_parser_template_references_001`、
  `fail_parser_template_arguments_chained_iff_001` は active parse-only case である。
- algorithm / claim basics: task 32 で完了。
  `pass_parser_algorithms_claims_001` と
  `fail_parser_algorithms_claims_recovery_001` は active parse-only case である。
- まだ必要な受理ケース: `by` 参照付き `let` 制約、witness 付き `take`、
  条件付き definiens、Fraenkel generator、`qua` 連鎖、述語連鎖。
- まだ必要な拒否ケース: 非結合演算子の連鎖、builtin/user 述語連鎖の混在、
  不完全な項始まり論理式。

## 解決済みおよび保留中の決定

- **パーサー支援字句解析の契約: トップレベルで解決済み。** パーサーは
  字句解析器と交錯しない。string-required 位置とユーザーシンボル種別
  フィルタは、事前計算された plan を通じて到着する。
- **文法の正本とブラッシュアップの手順: 解決済み。**
  [doc/spec/ja/](../../../spec/ja/00.index.md) 配下の章仕様が規範であり、
  [appendix_a.grammar_summary.md](../../../spec/ja/appendix_a.grammar_summary.md)
  は統合サマリで、現在もブラッシュアップ中である。各文法タスクは、まず
  自タスクの生成規則インベントリを所有章から [grammar.md](./grammar.md) の
  名前付き節へ転記する（英語と日本語を同じ変更で）。その節がタスクの有界な
  規範参照となる。転記または実装が EBNF のギャップ・曖昧さ・矛盾を露見させた
  場合は、後回しにせず、そのタスクの一部として所有章と付録を（英語と日本語を
  一緒に）修正する。文法のブラッシュアップは実装に先行してではなく、実装と
  並行して進める。
- **ドットの役割の surface 形状: task 10 により parser/syntax では解決済み。**
  この決定は `mizar-parser`、`mizar-syntax`、将来の resolver にまたがるため、
  トップレベル（[../../todo.md](../../todo.md)「Resolved And Open Decisions」）にも
  登録済みである。パーサーは構文が許す範囲でのみドットの役割を解決する
  （仕様 [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)「ドットの
  曖昧性解消」）: 複合予約トークンと登録済みユーザーシンボルは字句解析器が
  所有し、dotted qualified-name head は qualified surface として保持し、すでに parse
  済みの term の後の `.` は selector/update postfix syntax になる。変数スコープに
  依存する selector 対 namespace の区別は resolver-owned のまま残す。
- **コーパスランナーの場所: task 3 で解決済み。** parse-only corpus execution
  は `mizar-test` に置く。`mizar-test` は discovery、expectation sidecar、
  traceability、CLI reporting に加えて active runner も意図的に所有する。
  metadata `plan` path は payload-free のままにし、frontend seam と session
  source loading に依存するのは `parse-only` subcommand だけとする。
- **構文木 storage 依存: `mizar-syntax` task 2 に委譲。** parser の境界は、
  文法コードを raw rowan node layout に依存させず rowan-backed syntax を
  target にできる builder / event API である。`mizar-parser` に `rowan` への
  直接依存を追加しない。文法作業が不足している tree 操作を必要とする場合は、
  先に `mizar-syntax` の builder / accessor API として追加する。
- **salsa 統合: この crate では保留、後段では必須。** `salsa` は compiler の
  query / cache 層で必須であり、`mizar-parser` には入れない。後続の build /
  frontend query が文法コードを変えずに `ParseOutput` をキャッシュできるよう、
  構文解析を決定的かつ副作用なしに保つ。

## 順序付きタスク一覧

各タスクは、単独で実装・テスト・コミットできる粒度になっている。各タスクの
後で `cargo test -p mizar-parser` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 基盤

1. **モジュール分割と lint 方針のガード。** [x]
   - `src/lib.rs` を内部実装 module の `grammar`、`pratt`、`recovery` に分割し、
     task 11/12 のコードを挙動変更なしに移動する。`parse`、`ParseRequest`、
     `ParserToken`、`ParseOutput` は現在の crate-root path から到達可能のまま
     保つ。module-level parser API を意図的に公開する後続タスクまでは、これらの
     module は private に保つ。
   - `mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加し、
     workspace lint opt-in、共有 rustc / clippy baseline、parser Rust target
     files にある意図的な `allow` 属性の inline rationale を確認する。この
     タスクでは後続の parser public-enum forward-compatibility gate や rustdoc
     policy gate は追加しない。
   - テスト: 既存のパーサーテストと frontend seam テストが変更なしに通る。
   - 依存: なし。仕様: [grammar.md](./grammar.md)。

2. **パーサー基盤: cursor、構文イベント、期待トークン診断、同期。** [x]
   - 有界先読み付きのトークン cursor、精密な範囲を持つ `SyntaxDiagnostic` を
     生成する期待トークン診断ヘルパー、`mizar-syntax` builder へ供給する
     構文イベント sink、同期集合（`;`、`end`、トップレベル item キーワード、
     EOF）、および `mizar-syntax` の builder API の上に構築した recovery
     ノード送出ヘルパーを追加する。
   - 文法コードを具体的な `SurfaceAst` storage backend から独立させる: syntax
     arena への直接 push、密な node index への依存、raw rowan traversal は
     行わない。不足している構築または検査操作は、parser 文法コードが使う前に、
     文書化された `mizar-syntax` の builder / accessor API として追加する。
   - task 12 の recovery と mizar-frontend task 28 の block-stack matching
     （`end` 欠落、文字列リテラル欠落、回復不能入力、文脈付き block opener）を、
     観測可能な挙動を変えずにこれらのヘルパーへ一般化する。
   - テスト: 同期が各境界種別までスキップし、スキップ範囲を記録する。期待
     トークン診断が EOF とストリーム中間で正しい第一範囲を運ぶ。
   - 初期 top-level item 同期キーワードは `theorem`、`definition`、
     `registration`、`notation`、`scheme`、`reserve`、`begin`、`environ`、
     `vocabularies`、`constructors`、`requirements` とする。後続の item 文法タスクは、
     実際の dispatch を追加するときにこの placeholder を調整してよい。
   - 依存: 1、`mizar-syntax` task 2。仕様: [recovery.md](./recovery.md)。

3. **parse-only コーパスランナー。** [x]
   - ランナーの場所: `mizar-test` が parse-only runner を所有する。これは
     corpus discovery、expectation sidecar、traceability、CLI reporting をすでに
     所有しているためである。この parse-only execution path のために
     `mizar-frontend` と `mizar-session` へ依存するが、metadata `plan` mode は
     payload-free のままにする。
   - `mizar-test parse-only` は通常の plan を通して expectations を discover し、
     `stage = "parse_only"` / `expected_phase = "parse"` かつ `active_parse_only`
     tag を持つ active `.miz` case を実トークン化と `MizarParserSeam` で実行する。
     inactive な planned grammar seed は discovery と traceability metadata のままにする。
   - active seed coverage は、現在 frontend から到達できる parser behavior
     （token stream preservation、`end` 欠落、孤立した `end`）を含み、task 12 以降は
     fixture lexical summary から供給される source-path operator fixity も含む。
   - task 31 は、task 14 と 23-25 が必要な formula / definition host を
     提供した後、コミット済みの template 引数 seed case を active runner に
     promote する。active set には reference-citation template argument seed も
     含まれる。
   - task 32 は active algorithm/claim pass/fail case と、source-level の
     algorithm body / ghost assignment coverage に必要な frontend scope-skeleton
     support を追加した。
   - テスト: active / inactive discovery は決定的である。active tag の誤用は
     harness error になる。意図的に不一致にした sidecar は失敗する。seed した
     pass / fail case は diagnostics を強制する。`parse-only` CLI は active runner
     summary を report する。
   - 依存: 2。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)、
     [expectation_schema.md](../../mizar-test/ja/expectation_schema.md)。

### resolver 前の互換性ゲート

**公開 enum 前方互換性の初期ゲート。** [x]
- phase 3 境界にすでに存在する parser 公開 enum
  （`ParserTokenKind`、`OperatorAssociativity`、`StringRequiredContext`）について、
  `mizar-frontend` task 25 の手続きと `mizar-syntax` の初期ゲートに沿って、
  `#[non_exhaustive]` 対 意図的 exhaustive を決定する。task 12 の
  `OperatorFixity` のような後続の公開 enum も同じゲートで分類する。
- 各決定を所有モジュール仕様に記録し、parser task 5〜7 が resolver / LSP の
  入力になり得る前に属性を適用する。
- 結果: `ParserTokenKind` と `StringRequiredContext` は downstream crate 向けに
  `#[non_exhaustive]` とした。`OperatorAssociativity` と task 12 の
  `OperatorFixity` は文書化された exhaustive 例外である。
  `crates/mizar-parser/tests/lint_policy.rs` が現在の parser public enum すべての分類を
  guard する。
- 依存: 3 と `mizar-syntax` の公開 enum 初期ゲート。仕様:
  [grammar.md](./grammar.md)、[pratt.md](./pratt.md)、[recovery.md](./recovery.md)。

### 文法の成長

各文法タスクは同じテンプレートに従い、1 つの変更で行う。

1. 自タスクの生成規則インベントリを、所有する仕様章から
   [grammar.md](./grammar.md) の名前付き節へ転記する（英語と日本語を一緒に）。
   転記がギャップを露見させたら、所有章と
   [付録 A](../../../spec/ja/appendix_a.grammar_summary.md) をブラッシュ
   アップする。
2. 文書化された rowan-backed builder / accessor 境界を通じて、対になる
   `mizar-syntax` ノード増分を追加する。
3. 同期と recovery を備えた生成規則を実装する。
4. [テストコーパス方針](#テストコーパス方針)に従い、ユニットテストに加えて
   pass / fail コーパスケースと `spec_trace.toml` エントリを提供する。

依存行の `mizar-syntax task 11 / S-011` のような参照は、そのパーサータスクが
必要とする特定のノード語彙増分を意味する。syntax 側の語彙 bucket 全体の完了を
意味しない。crate-plan S-task と古い numeric syntax task reference が食い違って
見える場合は、`doc/design/mizar-syntax/ja/00.crate_plan.md` を優先する。

4. **修飾シンボルと namespace パス。** [x]
   - `qualified_symbol = { namespace_segment "." } user_symbol`、
     `qualified_constructor_name = { namespace_segment "." } constructor_name`、および
     ドット区切りモジュールパスの共有ヘルパー。後続の import、型ヘッド、項、引用が
     使う。パスの形だけを扱い、変数 shadowing は resolver 側に残す。
   - 結果: task 4 の production inventory は [grammar.md](./grammar.md) に記録済み。
     共有 helper は `ModulePath`、`NamespacePath`、`QualifiedSymbol`、
     `PathSegment`、`RelativePrefix` syntax node を送出し、unit coverage を持つ。
     corpus coverage は計画どおり消費側 task 6 と 8 に残す。
   - コーパス例外: ここではユニットテストを提供する。コーパス網羅は最初の
     消費位置（task 6 と 8）で届け、そこに明示的に列挙する。
   - 依存: 3、`mizar-syntax` task 9 / S-009（shared path-node 増分）。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)
     §12.7、[Appendix A](../../../spec/ja/appendix_a.grammar_summary.md) A.3/A.12/A.15、
     [第 2 章](../../../spec/ja/02.lexical_structure.md) §2.5.3 / §2.8。

5. **モジュールスケルトンとトップレベル item ディスパッチ。** [x]
   - モジュールファイルの形と、item 境界での同期を備えたキーワードによる
     トップレベル item ディスパッチ。これにより、後続のすべてのカテゴリが
     安定したスケルトンに収まる。
   - recovery: 未知のトップレベルトークンは、スキップトークンノードを残して
     次の item キーワードまでスキップする。`;` の欠落は次の境界で診断する。
   - 結果: task 5 の production inventory は [grammar.md](./grammar.md) に記録した。
     parser は `CompilationUnit`、`ItemList`、`PlaceholderItem` syntax node を送出し、
     item を含まない legacy token stream は空の item list で保持する。unexpected
     top-level input では `SkippedToken` recovery と skipped-range trivia を生成し、
     item semicolon 欠落を診断する。active parse-only の pass/fail corpus coverage と
     traceability も追加済み。
   - 依存: 3、`mizar-syntax` task 9 / S-009。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

6. **import item。** [x]
   - alias と相対 prefix（`.` / `..`）を持つ `import` item。形は frontend の
     import 事前走査 stub と整合させる。task 4 のパス形に対する繰り延べ
     コーパスケースを含む。
   - 結果: task 6 の production inventory は [grammar.md](./grammar.md) に記録した。
     parser は module `ItemList` 配下に `ImportItem`、`ImportAliasDecl`、
     `ModuleBranchImport` syntax node を送出し、import path と alias には共有の
     `ModulePath` / `RelativePrefix` / `PathSegment` node を使う。`import` prelude
     が開いている間だけ import を concrete item として扱い、遅れた import は
     `UnexpectedTopLevelToken` で回復する。alias / branch の不正構文は
     `MalformedImport` で診断し、active parse-only の pass/fail corpus coverage と
     traceability を追加済みである。`mizar-test` の parse-only run は import stub を
     空の syntax-only summary に解決するため、意味的な module availability なしに
     import 構文をテストできる。
   - 依存: 4、5、`mizar-syntax` task 9 / S-009。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

7. **export と可視性の item。** [x]
   - モジュール章に従った export の形と、item 上の `public` / `private`
     可視性マーカー。
   - 結果: task 7 の production inventory を [grammar.md](./grammar.md) に記録した。
     parser は `ExportItem`、`VisibilityMarker`、`VisibleItem` syntax node を送出し、
     export prelude が開いている間だけ export を concrete に扱う。遅れた export は
     `UnexpectedTopLevelToken` で回復し、不正な export path list は `MalformedExport`、
     duplicate または不正な visibility prefix は `MalformedVisibility` で診断する。
     visible wrapper 内では annotation-prefix token order を保持し、active parse-only
     pass/fail corpus coverage と traceability を追加した。
   - 依存: 5、`mizar-syntax` task 9 / S-009。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

8. **型式。** [x]
   - 属性連鎖（`non` を含む）、radix / mode の型ヘッド、`of` / `over` 引数
     リスト、struct 修飾の属性参照。項引数は task 9 が着地するまで項エントリ
     のスタブを通す（型と項は相互再帰である）。task 4 の修飾型ヘッドに対する
     繰り延べコーパスケースを含む。test は右端の attribute / type-head split、
     bracket nested type-expression argument、bracket `qua_arg` placeholder、
     malformed type argument、incoming token が split を露出する場合の局所
     parameter-prefix preservation を pin する。
   - 依存: 4、5、`mizar-syntax` task 10 / S-010。仕様:
     [03.type_system.md](../../../spec/ja/03.type_system.md)、
     [§A.3.2](../../../spec/ja/appendix_a.grammar_summary.md)。
   - 結果: concrete top-level `reserve` parsing と
     `ReserveItem` / `ReserveSegment`、syntax-only `TypeExpression`、
     `AttributeChain`、`AttributeRef`、generic `TypeHead`、`TypeArguments`、
     `TermPlaceholder` node を実装した。`MalformedTypeExpression` recovery、
     token split が露出する局所 parameter-prefix の parser unit coverage、
     `parser.type_fixtures` 経由の active parse-only pass/fail corpus coverage を
     追加した。

9. **一次項。** [x]
   - 識別子、数値、項位置の修飾シンボル、括弧付き項、適用形。task 8 の項
     エントリスタブを置き換える。`it`、選択式（`the type_expression`）、
     名前付きフィールド引数を持つ構造体コンストラクタ、集合列挙リテラルも含む。
   - 依存: 8、`mizar-syntax` task 11 / S-011。仕様:
     [13.term_expression.md](../../../spec/ja/13.term_expression.md)。
   - 結果: reserve-hosted の `of` / `over` 引数と attribute argument list から
     到達できる syntax-only `TermExpression` primary term を実装し、それらの位置に
     あった task 8 の term placeholder を置き換えた。bracket `type_arg_list` の
     `qua_arg` placeholder は task 11 のために保持した。
     `MalformedTermExpression` / `MissingTerm` / term delimiter recovery を追加し、
     parser unit test と active parse-only pass/fail corpus coverage を追加した。

10. **selector access / update とドットの役割の surface 形状。** [x]
    - selector access / selector-call の連鎖（`p.x`、`line.finish.y`、
      `M.binop(x, y)`）、functional structure update（`p with (...)`）、および
      syntax-only dot-role representation。selector-update surface vocabulary は
      導入するが、`p.x := t` のような standalone in-place assignment は後続の
      statement / algorithm host に残す。ドットの役割の surface 形状の決定
      （「解決済みおよび保留中の決定」を参照）を解決し、
      [grammar.md](./grammar.md)、仕様の付録、トップレベルの決定一覧に
      記録する。
    - 依存: 9、`mizar-syntax` task 11 / S-011。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)、
      [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)。
   - 結果: syntax-only の `SelectorAccess`、`StructureUpdate`、`FieldUpdate`
     surface を term postfix chain として実装した。selector-call argument list、
     left-associative selector nesting、functional structure update list、
     malformed selector / update syntax の `MalformedTermExpression` recovery、
     update value 欠落の `MissingTerm`、active parse-only pass/fail corpus
     coverage、traceability entry を追加した。standalone `p.x := t` は後続の
     statement / algorithm host の担当として残す。

11. **`qua` 修飾。** [x]
    - selector と適用形に対する優先順位を持つ `term qua type_expression`。
      selector / update / application 形を `qua` より先に parse し、`qua` chain を
      left-associative に畳み込む。`x qua Element of S qua Magma` の target-type
      argument binding を保持し、bracket `qua_arg` の `TermPlaceholder` stub は
      task 11 の `TermExpression` / `QuaExpression` surface に置き換える。ただし
      Appendix A のより狭い `qua_arg ::= identifier { "qua" radix_type }` 形を尊重する。
      `qua` target 欠落または malformed target は `MalformedTypeExpression` と
      `QuaExpression` 下の `MissingTypeExpression` または skipped-tail recovery を使う。
      test は selector precedence、parenthesized selector-after-`qua`、bracket `qua_arg`
      migration、left-associative chain、target recovery、active parse-only pass/fail
      traceability を含める。
    - 依存: 8、9、`mizar-syntax` task 11 / S-011。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。
   - 結果: selector/update postfix parsing の後に syntax-only `QuaExpression`
     surface を実装した。left-associative `qua` chain、nested term-argument `qua`
     binding を持つ target type-expression parsing、bracket `qua_arg` の
     `TermPlaceholder` から `TermExpression` / `QuaExpression` への移行、
     `MissingTypeExpression` または skipped target tail を伴う
     `MalformedTypeExpression` recovery、parser unit test、active parse-only
     pass/fail corpus coverage、traceability entry を追加した。

12. **演算子式（アクティブレキシコン上の Pratt）。** [x]
    - task 11 の明示 fixity の Pratt パーサーを、`ParserInputs` の fixity
      メタデータで駆動されるユーザー prefix / infix / postfix 演算子へ一般化
      する。優先順位と結合性は
      [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)
      に従う。非結合の連鎖と宙吊り演算子を、ソースローカルな範囲で診断する。
    - 依存: 10、11、`mizar-syntax` task 11 / S-011（演算子ノードの増分）。仕様:
      [pratt.md](./pratt.md)、
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。
   - 結果: parser-facing な `OperatorFixity` metadata、frontend seam 経由の
     summary-derived `ParserInputs` 転送、`qua` の前に prefix/postfix/infix
     operator を扱う module-term Pratt parsing、`PrefixExpression` /
     `PostfixExpression` syntax surface、non-associative と dangling operator
     診断、parser unit test、active parse-only pass/fail corpus coverage、
     traceability entry `spec.en.13.operator_precedence.parser` を実装した。

13. **原子論理式。** [x]
    - 述語適用（記号形と識別子形）、built-in membership / equality /
      inequality atom、および resolution が後で type assertion または
      attribute assertion に分類する generic `is_assertion` form。
    - 依存: 12、`mizar-syntax` task 12 / S-012。仕様:
      [14.formulas.md](../../../spec/ja/14.formulas.md)。
   - 結果: task 13 の `FormulaExpression`、`BuiltinPredicateApplication`、
     generic `IsAssertion`、`AttributeTestChain`、
     `PredicateApplication` / `PredicateSegment` / `PredicateHead`、
     `InlinePredicateApplication` surface、theorem/lemma `label: formula;`
     placeholder host、malformed atomic formula 用の term/type recovery、
     parser unit test、active parse-only pass/fail corpus coverage、
     traceability entry `spec.en.14.atomic_formula.parser` を実装した。
     built-in predicate は単独 atom のままであり、user/built-in predicate
     mixed chain は reject する。

14. **結合子と量化子。** [x]
    - 固定結合子テーブル（`not`、`&`、`or`、`implies`、`iff`）とその論理式
      レベルの優先順位（項レベルの fixity から分離したまま保つ）。`st` /
      `holds` 本体を持つ量化子 `for` / `ex`。
    - 依存: 13、`mizar-syntax` task 12 / S-012。仕様:
      [14.formulas.md](../../../spec/ja/14.formulas.md)、
      [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)。
   - 結果: task 14 の `PrefixFormula`、`BinaryFormula`、
     `ParenthesizedFormula`、`QuantifiedFormula`、
     `QuantifierVariableSegment`、`FormulaConstant` surface、固定 formula
     connective precedence、`iff` non-associativity 診断、formula operator と
     quantifier body 後の `MissingFormula` /
     `MalformedFormulaExpression` recovery、parser unit test、active
     parse-only pass/fail corpus coverage、traceability entry
     `spec.en.14.formula_connectives_quantifiers.parser` を実装した。template
     predicate argument は task 31 / S-016 まで deferred だったが現在は表現済みであり、
     formula を埋め込む Fraenkel / set-builder term は task 15 で実装済みである。

15. **Fraenkel と集合内包の項。** [x]
    - `{ term where … : formula }` と関連する集合内包形（条件を省略する形を
      含む）。区切り句が論理式を埋め込むため、論理式の後に置く。集合列挙
      リテラルは task 9 で扱う。
    - 依存: 14、`mizar-syntax` task 11 / S-011（Fraenkel ノードの増分）。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。
    - 結果: task 15 の `SetComprehension` と
      `ComprehensionVariableSegment` surface、`SetEnumeration` との top-level
      `where` による分岐、task 14 の formula parser を使う任意 condition
      formula、generator type recovery、condition 欠落 recovery、brace 欠落
      recovery、parser unit test、active parse-only pass/fail corpus coverage、
      `spec.en.13.set_expressions.parser` の traceability、および expression-level
      の `is set` type word を malformed `set name =` binder statement として
      報告しない scope-skeleton guard を実装した。

16. **単純文。** [x]
    - `let`、`assume`、`take`、`set`、`given` — 正当化句を運ばない文の形。
      `reserve` は、Chapter 4 が block-local `reserve` shaped statement を禁じているため
      既存 top-level `ReserveItem` path のままにし、その path を non-regression として
      覆う。
    - 依存: 14、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。
    - `StatementItem` が host する simple statement として実装した。
      `QualifiedVariableSegment`、`ConditionList`、`Proposition`、`Witness`、
      `Equating` child を持つ。task 17 により、以前の `let ... by ...`
      placeholder 境界は concrete な justification-aware 形状へ更新済み。
      正常系、複数 `set` equating、proposition label、recovery node、skipped
      tail、semicolon-boundary 同期、および top-level `ReserveItem`
      non-regression の unit test と active parse-only pass/fail corpus
      coverage を追加済み。

17. **正当化と引用。** [x]
    - `by` の正当化句、引用リスト、`.{ … }` グループ引用、`.*` 一括引用、
      `let ... by references`、および `proposition by ...;` 用の最小の明示的
      compact-statement host。algorithm 章の `by computation(...)` オプションも含む。
      canonical Chapter 15 / 16 grammar は `from` を justification form として
      定義していないため、以前の `from` 記述は実装対象の構文ではなく
      derived-documentation drift として扱う。
    - 依存: 16、`mizar-syntax` task 14 / S-014（正当化ノードの増分）。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md) §15.2.1 / §15.8、
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)
      §16.5、
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)
      §20.9.2。
    - 結果: `JustificationClause`、`ReferenceList`、`Reference`、
      `QualifiedReference`、`GroupedReference`、`GroupedReferenceItem`、
      `BulkReference`、`ComputationJustification`、`ComputationOption`、および
      最小の explicit-justification `CompactStatement` host を実装した。
      `let ... by references` は concrete になり、`let ... by computation` は
      malformed justification として recover する。local / qualified /
      grouped / bulk / computation justification、missing proof step、skipped
      malformed citation tail、task 31 以前の reference template argument recovery
      drift、および semicolon-boundary recovery を unit test と active parse-only
      pass/fail corpus coverage で確認済み。task 31 以後、citation reference
      template argument は concrete に表現される。

18. **`consider` と `reconsider`。** [x]
    - いずれも正当化を運ぶ `consider … such that … by …` と
      `reconsider … as … by …`。
    - 依存: 17、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。
    - 結果: `ConsiderStatement`、`ReconsiderStatement`、
      `ReconsiderItem` の parsing を実装した。shared-type qualified
      variable、condition list、必須の simple `by` justification、`such` /
      condition / `as` / target type / item 部品 / justification 欠落の
      recovery、syntax typed accessor、reconsider `type_change_list` 向けの
      scope-skeleton support、active parse-only pass/fail corpus coverage を
      追加した。

19. **結論ステップと逐次的等式。** [x]
    - `thus` / `hence`、`then` 連鎖、およびステップごとの正当化を持つ逐次的
      等式 `.=` ステップ。compact equality statement と zero-step iterative
      equality の grammar-audit 境界（`x = y by A;` と
      `x = y by A .= z by B;`）を含める。
    - 依存: 17、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。
    - 結果: `ConclusionStatement`、`ThenStatement`、
      `IterativeEqualityStatement`、`IterativeEqualityStep` の parser / syntax
      surface を追加した。G-AUD-010 dispatch により、top-level `.=` continuation
      が続かない `x = y by A;` は `CompactStatement` のまま保持し、続く場合だけ
      `IterativeEqualityStatement` にする。parser は `thus` / `hence`
      conclusion、linkable な `then` statement、simple `by` justification を持つ
      iterative equality step、label variant、`then consider` / `then reconsider`
      を受け付ける。task 19 完了時点では `then per cases` は task 20 の
      block-statement placeholder として残した。parser unit test、active pass/fail corpus
      fixture、traceability metadata が新しい形状と recovery case を覆う。

20. **ブロック文。** [x]
    - `now` / `hereby` ブロックと、`end` 同期を備えた
      `per cases` / `suppose` / `case` ブロック。
    - 依存: 19、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。
    - 結果: `NowStatement`、`HerebyStatement`、
      `CaseReasoningStatement`、`CaseItem`、`SupposeItem` の parsing を実装した。
      `now` の optional label、nested reasoning body、`per cases` の optional
      explicit simple `by` justification、homogeneous な `case` または
      `suppose` branch list、branch proposition と `that` condition-list
      header、linkable な `then per cases`、non-linkable な `then now` /
      `then hereby`、malformed justification tail、branch header semicolon 欠落、
      mixed branch list、skipped token、`end` 欠落の recovery を覆う。parser unit
      test、active parse-only pass/fail corpus fixture、traceability metadata が
      新しい形状と recovery case を覆う。

21. **ローカル定義。** [x]
    - `deffunc` / `defpred` のプライベートなローカル定義。
    - 依存: 20、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。
    - 結果: standalone 専用の `InlineFunctorDefinition` と
      `InlinePredicateDefinition` の解析を実装した。`be` / `being` を持つ
      `TypedParameter` list、zero-argument definition、`->` / `equals` /
      `means` delimiter recovery、name / type / body / formula 欠落の
      recovery、non-linkable な `then deffunc` / `then defpred` の拒否、
      parser unit test、active parse-only pass/fail corpus fixture、traceability
      metadata を含む。

22. **定理と証明。** [x]
    - `theorem` / `lemma` の item、ラベル、`proof … end` の入れ子、証明本体の
      文の配線。
    - 依存: 21、`mizar-syntax` task 14 / S-014。仕様:
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)。
    - 結果: `TheoremItem`、`LemmaItem`、`ProofBlock` を実装し、status token の保存、
      visibility-wrapped theorem target、theorem-level の `by` / `by computation` /
      full-proof justification tail、proof body の concrete statement wiring、conclusion
      と compact statement host 上の statement-level proof justification、label / colon /
      formula / proof-end 欠落 recovery、active parse-only pass/fail corpus fixture、
      traceability metadata を追加した。

23. **definition ブロック骨格・correctness 条件・属性定義。** [x]
    - すべての定義種別が共有する `definition … end` ブロックの形、
      correctness 条件句の形（`existence`、`uniqueness`、`coherence`、
      `consistency`、`compatibility` など、正当化付き）、および最初の具体
      種別としての `attr` 定義。
    - 依存: 22、`mizar-syntax` task 15 / S-015。仕様:
      [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)、
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)
      §20.9.2。
    - 結果: 最初の S-015 増分として `DefinitionBlockItem`、
      `DefinitionParameter`、`AttributeDefinition`、`AttributePattern`、
      `FormulaDefiniens`、`FormulaCase`、`CorrectnessCondition` の解析を
      実装した。通常の definition parameter、assumption、attr definition、
      空 / reference / computation / proof correctness condition、theorem / lemma
      content、visible theorem / lemma content は concrete である。
      template-ambiguous parameter
      （`let T be type;`）と後続の definition-family form は G-AUD-006 のもとで
      source-preserving placeholder のまま保持する。parser unit test、active
      parse-only pass/fail corpus fixture、traceability metadata が新しい形と
      recovery case を網羅する。

24. **述語定義。** [x]
    - `means` 本体を持つ `pred` 定義。
    - 依存: 23、`mizar-syntax` task 15 / S-015。仕様:
      [09.predicates.md](../../../spec/ja/09.predicates.md)。
    - 結果: definition block 内の `PredicateDefinition` と raw
      `PredicatePattern` 解析を実装した。通常の predicate definition と
      definition-local な `public` / `private` predicate definition、task-23 の
      `FormulaDefiniens` body、意味的な symbol-role split を記録しない
      ambiguous phrase pattern の grammar-shaped validation、built-in predicate-token
      rejection、imported symbolic predicate-token coverage、parser-token
      lexeme-run symbolic pattern coverage、template definition 分類なしの
      template-loci token preservation、parser unit test、active parse-only
      pass/fail corpus fixture、traceability metadata を含む。

25. **ファンクタ定義。** [x]
    - `means` / `equals` 本体を持つ `func` 定義。
    - 依存: 23、`mizar-syntax` task 15 / S-015。仕様:
      [10.functors.md](../../../spec/ja/10.functors.md)。
    - planned increment: definition block 内の `FunctorDefinition` と raw
      `FunctorPattern` parsing、definition-local な `public` / `private` functor
      visibility、return `TypeExpression` recovery、`means` 用の task-23
      `FormulaDefiniens` 再利用、`equals` 用の task-25 `TermDefiniens` /
      `TermCase` node、imported symbolic、parser-token symbolic、circumfix functor
      pattern coverage、template definition 分類なしの template-loci preservation、
      parser unit test、active parse-only pass/fail corpus fixture、traceability metadata を追加する。
    - 結果: definition block 内の `FunctorDefinition`、raw
      `FunctorPattern`、`TermDefiniens`、`TermCase` parsing を実装した。
      `public func` / `private func` は既存の `VisibleItem` wrapper を再利用し、
      `means` 本体は `FormulaDefiniens` を再利用する。`equals` 本体は term case と
      `otherwise` term を保持する。label、pattern、return type、body keyword、term
      body、term-case condition の不正形は文書化済み recovery vocabulary を使う。
      parser unit test、active parse-only pass/fail corpus fixture、source-level
      circumfix coverage、template-functor placeholder preservation、traceability
      metadata が Chapter 10 の functor definition を覆う。

26. **mode 定義。** [x]
    - 正本の `is` 形を用いる `mode` 定義: 属性連鎖と radix 型、型パラメータ、
      任意の `sethood` property 句。
    - 依存: 23、`mizar-syntax` task 15 / S-015。仕様:
      [07.modes.md](../../../spec/ja/07.modes.md)。
    - 結果: definition block 内の `ModeDefinition`、raw `ModePattern`、
      `ModeProperty` parsing を実装した。definition-local な `public mode` /
      `private mode` は `VisibleItem` を再利用する。mode body は `TypeExpression`
      を再利用し、type parameter は raw pattern order で保持し、`sethood` は必須の
      general justification を消費する。semantic な radix validation、sethood proof
      obligation、dependent-mode check、legacy な `means` mode body は parser の外に
      残す。parser unit test、active parse-only pass/fail corpus fixture、
      読みやすいコンストラクタ名によるモードのカバレッジ、traceability metadata が
      Chapter 7 の mode definition を覆う。

27. **`redefine`・`synonym`・`antonym`。** [x]
    - task 23〜26 の定義種別にまたがる再定義と記法エイリアスの形。
    - 依存: 24、25、26、`mizar-syntax` task 15 / S-015。仕様:
      [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [07.modes.md](../../../spec/ja/07.modes.md)、
      [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [10.functors.md](../../../spec/ja/10.functors.md)、
      [11.symbol_management.md](../../../spec/ja/11.symbol_management.md)。
    - 結果: `AttributeRedefinition`、`PredicateRedefinition`、
      `FunctorRedefinition`、入れ子の `CoherenceCondition`、`NotationAlias`、
      raw `NotationPattern` parsing を実装した。definition-local な `public` /
      `private` redefinition と alias は `VisibleItem` を再利用する。canonical
      grammar は mode redefinition を定義しないため、`redefine mode` は
      placeholder / recovery boundary のまま保持する。alias branch classification は
      resolver-owned のままである。parser unit test、active parse-only pass/fail
      corpus fixture、visibility coverage、redefinition recovery、unsupported-mode
      fallback preservation、traceability metadata が task-27 surface を覆う。

28. **property 句。** [x]
    - 定義種別にまたがる property 句（`commutativity`、`idempotence`、
      `involutiveness`、`projectivity`、`reflexivity`、`irreflexivity`、
      `symmetry`、`asymmetry`、`connectedness`、`transitivity`、`sethood`
      など、正当化付き）。
    - 依存: 27、`mizar-syntax` task 15 / S-015。仕様:
      [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [07.modes.md](../../../spec/ja/07.modes.md)、
      [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [10.functors.md](../../../spec/ja/10.functors.md)。
    - 結果: canonical な predicate property keyword（`symmetry`、`asymmetry`、
      `connectedness`、`reflexivity`、`irreflexivity`）、functor property keyword
      （`commutativity`、`idempotence`、`involutiveness`、`projectivity`）、および
      standalone `sethood` property item の `PropertyClause` parsing を実装した。
      `mode` definition 直後の `sethood` は task-26 の `ModeProperty` のまま保持する。
      TODO 文言に含まれる `transitivity` は canonical property production との
      design drift であり、実装していない。parser unit test、active parse-only
      pass/fail corpus fixture、recovery coverage、traceability metadata が task-28
      surface を覆う。

29. **構造体。** [x]
    - `struct` 定義: フィールド、継承／`extends`、selector 宣言。
    - 依存: 28、`mizar-syntax` task 15 / S-015。仕様:
      [05.structures.md](../../../spec/ja/05.structures.md)。
    - 結果: definition block 内の `StructureDefinition`、raw
      `StructurePattern`、`StructureField`、`StructureProperty`、
      `InheritanceDefinition`、`InheritanceTarget`、`FieldRedefinition`、
      `PropertyRedefinition` parsing を実装した。structure name は通常の識別子、
      または読みやすいハイフン区切りのコンストラクタ名に制限され、selector は識別子の
      ままにする。definition-local な
      `public struct` / `private struct` と `public inherit` / `private inherit` は
      `VisibleItem` を再利用する。parser は structure parameter、field initializer、
      shorthand inheritance、explicit `where ... end` inheritance、`extends set`、
      任意の type narrowing、任意の coherence proof を保持し、selector fact、parent
      coverage、narrowing validity、constructor semantics、proof obligation は parse
      phase の外に残す。parser unit test、active parse-only pass/fail corpus fixture、
      recovery coverage、frontend scope-skeleton の nested-block follow-through、
      traceability metadata が task-29 surface を覆う。

30. **registration と cluster。** [x]
    - `registration … end` ブロック、existential / conditional / functorial の
      cluster の形、`reduce`、およびそれらの correctness 条件。
    - 依存: 29、`mizar-syntax` task 15 / S-015。仕様:
      [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)。
    - 結果: `RegistrationBlockItem`、`RegistrationParameter`、
      `ExistentialRegistration`、`ConditionalRegistration`、
      `FunctorialRegistration`、`ReductionRegistration` parsing を実装した。
      top-level の `registration ... end;` block は registration-local な `let`
      parameter、cluster registration、reduction、missing-end recovery を所有する。
      definition-local な `public` / `private` cluster と reduction item は
      `VisibleItem` を再利用する。parser は semantic cluster fact を作らずに
      registration adjective を保持し、functorial payload は syntactically
      unambiguous な application / operator / bracket-functor surface だけを受理する。
      nullary functorial ambiguity は deferred のまま残し、correctness condition は
      syntax-level citation / proof obligation として保持する。parser unit test、
      active parse-only pass/fail corpus fixture、frontend scope-skeleton の
      registration block support、recovery coverage、traceability metadata が task-30
      surface を覆う。

31. **テンプレート。** [x]
    - テンプレートパラメータ、task 8 の生成規則を拡張する bracket 形の型引数
      とパラメータ prefix、`nest` の形。
    - レビュー監査由来の seed ケース
      `tests/miz/pass/parser/pass_parser_template_arguments_001.*` と
      `tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.*`
      を、traceability metadata から runner 実行済みの parse-only coverage へ
      昇格させる。
    - 結果: `TemplateParameter`、`TemplateLoci` / `TemplateLocus`、
      `TemplateArguments` / `TemplateArgument` surface を実装し、predicate /
      functor / reference / template functor の引数、template-shaped
      definition-block 分類、radix-only `qua` 引数 recovery、active parse-only
      seed coverage、既存 computation-option parser による `nest` traceability
      を追加した。
    - 依存: 30、`mizar-syntax` task 16 / S-016。仕様:
      [18.templates.md](../../../spec/ja/18.templates.md)。

32. **algorithm ブロック・代入・宣言・claim。** [x]
    - `algorithm` ブロックの形、代入文、`var` / `const` 宣言、
      `ghost var` / `ghost const`、ghost 代入、`snapshot`、top-level `claim`
      block、任意の正当化を持つ `return` 文。
    - 結果: `AlgorithmDefinition`、`AlgorithmParameters`、`AlgorithmBody`、
      `AlgorithmStatementList`、`VariableDeclaration`、`VariableBinding`、
      `AssignmentStatement`、`Lvalue`、`SnapshotStatement`、`ReturnStatement`、
      `ClaimBlockItem` surface を実装し、active pass/fail parse-only coverage と
      algorithm body / ghost assignment 用 frontend scope-skeleton support を
      追加した。
    - 依存: 31、`mizar-syntax` task 16 / S-016。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

33. **algorithm の制御フロー。** [x]
    - `while ... do`、range `for ... = ... (to|downto) ... [step ...]`、
      `if` / `else`、`match`、`for ... in ... [processed ...]`、
      `otherwise` / `exhaustive` の match 終端、`break` / `continue`。
    - 結果: `IfStatement`、`WhileStatement`、`ForRangeStatement`、
      `ForCollectionStatement`、`MatchStatement`、`MatchCase`、`MatchEnding`、
      `BreakStatement`、`ContinueStatement` surface を実装し、else-if chain と
      recovery の parser unit coverage を追加した。active pass/fail parse-only
      coverage は通常の if/else、while、任意の `step` を持つ `to` / `downto`
      range loop、`processed` の有無がある collection loop、複数 match case、
      justification 有無の `otherwise` / `exhaustive` ending、および jump
      statement を行使する。concrete loop verification clause は task 34 で覆う。
    - 依存: 32、`mizar-syntax` task 16 / S-016。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

34. **algorithm の検証句。** [x]
    - ヘッダーおよび loop の検証句: `requires` / `ensures`、`decreasing`、
      `terminating`、`invariant`、`assert`、および loop clause / assertion 上の
      syntax-level justification。
    - 結果: `AlgorithmTerminationClause`、`AlgorithmRequiresClause`、
      `AlgorithmEnsuresClause`、`AlgorithmDecreasingClause`、
      `LoopInvariantClause`、`LoopDecreasingClause`、`AssertStatement`、
      `TermList` surface を実装した。active pass/fail parse-only coverage は
      terminating visible algorithm、header contract、while / range / collection
      loop clause、assertion、重複 / 順序違反 header recovery、禁止される
      `for decreasing`、misplaced loop clause、空または dangling decreasing term
      list を行使する。
    - 依存: 33、`mizar-syntax` task 16 / S-016。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

35. **注釈。** [x]
    - 文レベル注釈、`@[...]` ライブラリ注釈、文字列リテラル注釈引数
      （string-required 位置は frontend の lexing plan がすでに網羅する）。
    - 結果: parser task 35 は parser-facing `@identifier` annotation marker を
      受理し、library / fixed / generic annotation 形式を concrete syntax node
      として表現し、standalone `@show_type` / `@eval` を diagnostic annotation
      node として保持し、module / definition / registration / proof/algorithm /
      claim theorem 位置に annotation wrapper を付け、active parse-only fixture
      で malformed annotation recovery を網羅する。
    - 依存: 34、`mizar-syntax` task 16 / S-016。仕様:
      [21.source_code_annotation_and_atp.md](../../../spec/ja/21.source_code_annotation_and_atp.md)。

36. **predicate redefinition label の修正。** [x]
    - 修正済みの第 9 章と Appendix A の production
      `redefine pred label: pred_pattern ...` に parser task 27 を同期する:
      `PredicatePattern` の前に必須 label と colon を消費し、label child を
      pattern の前に送出し、label 欠落には `MissingTerm` recovery を使う。
      pass/fail corpus case と parser unit test を更新し、parser grammar/recovery
      および mizar-syntax AST documentation も同期する。
    - 結果: `mizar-syntax` task 22 とともに実装済み。parser は必須の predicate
      redefinition label slot を `PredicatePattern` の前で消費・送出し、省略
      label には `MissingTerm` を挿入する。unit test と active parse-only corpus
      coverage を更新し、grammar と syntax AST documentation を labeled contract
      に同期した。
    - 依存: 27。仕様: [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [appendix_a.grammar_summary.md](../../../spec/ja/appendix_a.grammar_summary.md)。

### 強化と横断的フォローアップ

37. **recovery の統合と fail コーパスの拡張。** [x]
    - 全カテゴリの recovery 挙動を監査する: スキップトークンノード、対応
      しない区切り記号、不正な注釈。カテゴリがまだ同期せずに中断する箇所の
      ギャップを埋める。推奨 pass / fail 比率へ向けて fail コーパスを拡張する。
    - 結果: task 36 までの実装済み recovery surface を監査し、古くなっていた
      recovery status の記述を `design_drift` と分類した。top-level の unmatched
      `@[` prefix が後続 theorem host を保持せず unexpected top-level recovery へ
      落ち得る malformed-annotation host synchronization の `source_drift` を閉じた。
      recovered AST shape の parser unit coverage、active fail corpus case
      `fail_parser_recovery_consolidation_001`、traceability entry
      `spec.en.syntax.parser_recovery.annotation_sync` を追加し、本 TODO と
      [recovery.md](./recovery.md) を英語・日本語で同期した。
    - 依存: 35。仕様: [recovery.md](./recovery.md)、
      [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

38. **`SurfaceAst` スナップショットベースライン。** [x]
    - 代表的なコーパスケースについて、`mizar-syntax` のレンダリング
      （その task 3）を使った決定的なスナップショットベースラインを
      `tests/snapshots/` 配下に追加し、スナップショット比較をコーパス
      ランナーに配線する。
    - 結果: `mizar-test` に移行用 parse-only `snapshots =
      "snapshots/parser/<id>.surface_ast.snap"` sidecar support を追加し、
      diagnostics 一致後に `SurfaceAst::snapshot_text()` を byte-for-byte で比較
      するようにした。missing / unreadable / mismatch baseline、または AST がない
      snapshot request は harness failure になる。minimal token stream と
      unexpected top-level recovery の active pass/fail parser baseline、および
      traceability entry `spec.en.testing.surface_ast_snapshots` を追加した。general
      `[[snapshots]]` hash registry と update mode は将来の `mizar-test` work に残る。
    - 依存: 3、35、`mizar-syntax` task 3。仕様:
      [../../mizar-test/ja/snapshot.md](../../mizar-test/ja/snapshot.md)。

39. **決定性プロパティテスト。** [x]
    - 同一のトークンストリームが同一の `SurfaceAst` ノード順序・範囲・診断
      順序を生むことの crate レベル網羅。frontend の決定性スイートに倣う。
    - 結果: `crates/mizar-parser/tests/determinism.rs` に public API integration
      coverage を追加し、同一の module-recovery、Pratt operator、不正な
      multi-diagnostic token stream を繰り返し parse した結果を比較するようにした。
      tests は `SurfaceAst::snapshot_text()`、token node の index / text / range、
      expression-root identity、完全な `SyntaxDiagnostic` sequence を parser 呼び出し
      間で比較する。
    - 依存: 35。仕様:
      [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

40. **パーサー fuzz ターゲット。** [ ]
    - 任意の UTF-8 上でトークン化と構文解析を駆動するワークスペース fuzz
      ターゲットを追加し、panic が起きず、回復可能診断のみで完了することを
      アサートする。`mizar-frontend` task 29 の real-parser fuzz follow-up は
      frontend-owned target を着地済みであり、この task は parser-owned 側を追跡する。
    - 依存: 37。仕様: [recovery.md](./recovery.md)、
      [../../mizar-frontend/ja/todo.md](../../mizar-frontend/ja/todo.md)
      task 29。

41. **frontend パススルーのフォロースルー。** [ ]
    - 現在の mizar-frontend task 28 parser-recovery surface を超える文法の成長では、
      `mizar-frontend` の新しい follow-up を開く:
      各文法タスクに歩調を合わせて、frontend の recovery マーカーの
      パススルー、診断統合順序、`SurfaceAstCacheKey` の無効化の網羅を維持する。
    - 依存: 5 から始まり、37 で完了する。仕様:
      [../../mizar-frontend/ja/todo.md](../../mizar-frontend/ja/todo.md)
      を参照。

42. **parser module 境界のリファクタリング。** [ ]
    - oversized な `crates/mizar-parser/src/module.rs` 実装を、構文解析の挙動、
      crate root の公開 API、syntax-event 出力、診断、コーパス期待値、
      snapshot rendering を変えずに private な責務別 module へ分割する。
      候補境界は top-level/module item、definition/registration、statement/proof、
      term/formula/pattern、algorithm/annotation、集約した test helper。最終的な
      分割は現在のコード形状から選び、完了時に本 TODO へ記録する。
    - 新しい module はすべて semantic-free かつ parser-owned に保つ:
      resolver 状態を読まない、`mizar-syntax` の raw rowan traversal に依存しない、
      新しい公開 grammar API を作らない、挙動を変える cleanup を移動と混ぜない。
      後で再利用する helper を抽出する場合も、別の spec task で公開するまでは
      private のままにする。
    - テスト: parser unit test と active parse-only corpus は byte-stable に保つ。
      harness が要求する path-only な test-name 更新を除き `.expect.toml` は変更しない。
      `cargo fmt --check` と parser/syntax Clippy を green に保つ。
    - 依存: 36、37、39。仕様: [grammar.md](./grammar.md)、
      [recovery.md](./recovery.md)、本 TODO。

43. **ソース／仕様の対応監査と予約語カバレッジ。** [ ]
    - [grammar.md](./grammar.md)、[pratt.md](./pratt.md)、
      [recovery.md](./recovery.md) のすべての公開 API と約束された挙動を
      実装とテストへトレースし、ギャップをフォローアップタスクとして記録する。
    - [§A.2.4](../../../spec/ja/appendix_a.grammar_summary.md) のすべての
      予約語が、少なくとも 1 つのパーサーコーパステストで消費されていること
      （または、まだ文法位置を持たない将来予約として明示的に記録されている
      こと）を検証し、暗黙に未実装のキーワードを機械的に検出する。
    - 依存: 37、42。仕様: すべてのモジュール仕様と本 TODO。

44. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-parser/en/` の各英語正本ドキュメントを日本語版と
      比較し、API 一覧、状態、用語、リンク、挙動の約束を同期する。
    - 依存: 43。仕様: リポジトリのドキュメント方針。

45. **公開 enum の前方互換方針。** [ ]
    - 初期の公開 enum ゲートを task 35 後に再確認し、文法成長で追加された
      後続の公開 enum について、`mizar-frontend` task 25 の手続きと
      `mizar-syntax` task 17 の最終監査に整合する形で、
      `#[non_exhaustive]` 対 意図的 exhaustive を決定する。
    - 依存: 35、42。仕様: すべてのモジュール仕様。

## 推奨検証

各タスクの後に実行する。

```text
cargo test -p mizar-parser
cargo test -p mizar-syntax
cargo fmt --check
cargo clippy -p mizar-parser --all-targets --all-features -- -D warnings
cargo clippy -p mizar-syntax --all-targets --all-features -- -D warnings
```

parse-only ランナーが着地した後の各文法タスクでは、task 3 で選択した
parse-only コーパスランナーを所有する crate も実行し、`mizar-test` の期待値
/ discovery も検証する。既定の frontend-seam ランナーを選んだ場合は次を
意味する。

```text
cargo test -p mizar-frontend
cargo test -p mizar-test
```

テストが通ったら、ここでタスクにチェックを入れる。

## 注記

- 構文解析は意味論から自由なまま保つ: 名前解決、型推論、オーバーロード選択、
  証明義務は行わない。ドットの役割は構文が許す範囲でのみ解決し、resolver が
  仕上げる。
- パーサーは frontend 適合済みトークンのみを消費する。ソーステキストを
  再字句解析せず、任意の lexer / resolver 状態を受け取らない。
- パーサーは `mizar-syntax` の builder / event API を通して構文を送出する。
  文法コードは custom arena index や raw rowan layout に依存しない。
- `salsa` は後続の query / cache 層の関心事である。文法タスクを書き直さずに
  `ParseRequest -> ParseOutput` を query 化できるよう、決定的で副作用のない
  構文解析を保つ。
- 現在の mizar-frontend task 28 parser-recovery surface 後に文法が成長する
  場合は、fuzz coverage、recovery marker passthrough、診断 merge ordering、
  `SurfaceAstCacheKey` invalidation のための新しい `mizar-frontend`
  follow-up を開く。
- 仕様 EBNF のブラッシュアップは各文法タスクの一部であり、独立した作業系列
  ではない。[grammar.md](./grammar.md) へ転記した生成規則インベントリが各
  タスクの有界な契約であり、修正は所有章と付録 A に、英語と日本語を一緒に
  着地させる。
