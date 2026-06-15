# mizar-parser TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| grammar | [grammar.md](./grammar.md) | `src/grammar.rs` | [~] task 11/12 の最小エントリは private な task 2 cursor / event 基盤を使用 |
| pratt | [pratt.md](./pratt.md) | `src/pratt.rs` | [~] 明示 fixity の最小 Pratt は内部 `pratt` module に分割済み |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [~] task 12 の recovery と mizar-frontend task 28 の nested block-end matching は task 2 cursor / diagnostic / sync helper を使用 |

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
string-required 文脈を運ぶ。サマリ由来の fixity は、字句サマリが fixity
メタデータを公開するまで空のままである。コーパスハーネス（`mizar-test`）と
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

- template 引数: definition、formula、predicate/functor、template の各生成規則
  が parse 可能になったら、`pass_parser_template_arguments_001` と
  `fail_parser_template_arguments_chained_iff_001` を実行可能にする。
- まだ必要な受理ケース: `by` 参照付き `let` 制約、witness 付き `take`、
  条件付き definiens、Fraenkel generator、`qua` 連鎖、述語連鎖、template
  predicate/functor の use。
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
- **ドットの役割の surface 形状: 未解決。task 10 で解決する。** この決定は
  `mizar-parser`、`mizar-syntax`、将来の resolver にまたがるため、トップ
  レベル（[../../todo.md](../../todo.md)「Resolved And Open Decisions」）にも
  登録済みである。パーサーは構文が許す範囲でのみドットの役割を解決する
  （仕様 [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)「ドットの
  曖昧性解消」）: 複合予約トークンと登録済みユーザーシンボルは字句解析器が
  所有し、selector 対 namespace の区別は変数スコープに依存して resolver が
  確定する。未解決のドット連鎖を構文的に保つ `SurfaceAst` の形を、
  `mizar-syntax` task 11 / crate-plan S-011 とともに決定する。
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
     （token stream preservation、`end` 欠落、孤立した `end`）を含む。明示
     fixity の中置式は、resolver が所有する input を bypass せずに corpus
     expectation から frontend-visible fixity を供給できるようになるまで、parser
     と frontend seam の unit test coverage に残す。
   - コミット済みの template 引数 seed ケースは、task 14、23-25、31 が
     それらの formula、definition、template 形を parse できるまで active
     runner から外す。
   - テスト: active / inactive discovery は決定的である。active tag の誤用は
     harness error になる。意図的に不一致にした sidecar は失敗する。seed した
     pass / fail case は diagnostics を強制する。`parse-only` CLI は active runner
     summary を report する。
   - 依存: 2。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)、
     [expectation_schema.md](../../mizar-test/ja/expectation_schema.md)。

### resolver 前の互換性ゲート

**公開 enum 前方互換性の初期ゲート。** [ ]
- phase 3 境界にすでに存在する parser 公開 enum
  （`ParserTokenKind`、`OperatorAssociativity`、`StringRequiredContext`）について、
  `mizar-frontend` task 25 の手続きと `mizar-syntax` の初期ゲートに沿って、
  `#[non_exhaustive]` 対 意図的 exhaustive を決定する。
- 各決定を所有モジュール仕様に記録し、parser task 5〜7 が resolver / LSP の
  入力になり得る前に属性を適用する。
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
   - `qualified_symbol = { namespace_segment "." } user_symbol` とドット
     区切りモジュールパスの共有ヘルパー。後続の import、型ヘッド、項、引用が
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

5. **モジュールスケルトンとトップレベル item ディスパッチ。** [ ]
   - モジュールファイルの形と、item 境界での同期を備えたキーワードによる
     トップレベル item ディスパッチ。これにより、後続のすべてのカテゴリが
     安定したスケルトンに収まる。
   - recovery: 未知のトップレベルトークンは、スキップトークンノードを残して
     次の item キーワードまでスキップする。`;` の欠落は次の境界で診断する。
   - 依存: 3、`mizar-syntax` task 9 / S-009。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

6. **import item。** [ ]
   - alias と相対 prefix（`.` / `..`）を持つ `import` item。形は frontend の
     import 事前走査 stub と整合させる。task 4 のパス形に対する繰り延べ
     コーパスケースを含む。
   - 依存: 4、5、`mizar-syntax` task 9 / S-009。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

7. **export と可視性の item。** [ ]
   - モジュール章に従った export の形と、item 上の `public` / `private`
     可視性マーカー。
   - 依存: 5、`mizar-syntax` task 9 / S-009。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

8. **型式。** [ ]
   - 属性連鎖（`non` を含む）、radix / mode の型ヘッド、`of` / `over` 引数
     リスト、struct 修飾の属性参照。項引数は task 9 が着地するまで項エントリ
     のスタブを通す（型と項は相互再帰である）。task 4 の修飾型ヘッドに対する
     繰り延べコーパスケースを含む。
   - 依存: 4、5、`mizar-syntax` task 10 / S-010。仕様:
     [03.type_system.md](../../../spec/ja/03.type_system.md)、
     [§A.3.2](../../../spec/ja/appendix_a.grammar_summary.md)。

9. **一次項。** [ ]
   - 識別子、数値、項位置の修飾シンボル、括弧付き項、適用形。task 8 の項
     エントリスタブを置き換える。`it`、選択式（`the type_expression`）、
     名前付きフィールド引数を持つ構造体コンストラクタ、集合列挙リテラルも含む。
   - 依存: 8、`mizar-syntax` task 11 / S-011。仕様:
     [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

10. **selector access / update とドットの役割の surface 形状。** [ ]
    - selector access / update の連鎖（`p.x`、`line.end.y`、`p.x := t`）と
      functional structure update（`p with (...)`）、および未解決ドット連鎖の
      表現。ドットの役割の surface 形状の決定
      （「解決済みおよび保留中の決定」を参照）を解決し、
      [grammar.md](./grammar.md)、仕様の付録、トップレベルの決定一覧に
      記録する。
    - 依存: 9、`mizar-syntax` task 11 / S-011。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)、
      [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)。

11. **`qua` 修飾。** [ ]
    - selector と適用形に対する優先順位を持つ `term qua type_expression`。
    - 依存: 8、9、`mizar-syntax` task 11 / S-011。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

12. **演算子式（アクティブレキシコン上の Pratt）。** [ ]
    - task 11 の明示 fixity の Pratt パーサーを、`ParserInputs` の fixity
      メタデータで駆動されるユーザー prefix / infix / postfix 演算子へ一般化
      する。優先順位と結合性は
      [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)
      に従う。非結合の連鎖と宙吊り演算子を、ソースローカルな範囲で診断する。
    - 依存: 10、11、`mizar-syntax` task 11 / S-011（演算子ノードの増分）。仕様:
      [pratt.md](./pratt.md)、
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

13. **原子論理式。** [ ]
    - 述語適用（記号形と識別子形）、built-in membership / equality /
      inequality atom、および resolution が後で type assertion または
      attribute assertion に分類する generic `is_assertion` form。
    - 依存: 12、`mizar-syntax` task 12 / S-012。仕様:
      [14.formulas.md](../../../spec/ja/14.formulas.md)。

14. **結合子と量化子。** [ ]
    - 固定結合子テーブル（`not`、`&`、`or`、`implies`、`iff`）とその論理式
      レベルの優先順位（項レベルの fixity から分離したまま保つ）。`st` /
      `holds` 本体を持つ量化子 `for` / `ex`。
    - 依存: 13、`mizar-syntax` task 12 / S-012。仕様:
      [14.formulas.md](../../../spec/ja/14.formulas.md)、
      [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)。

15. **Fraenkel と集合内包の項。** [ ]
    - `{ term where … : formula }` と関連する集合内包形（条件を省略する形を
      含む）。区切り句が論理式を埋め込むため、論理式の後に置く。集合列挙
      リテラルは task 9 で扱う。
    - 依存: 14、`mizar-syntax` task 11 / S-011（Fraenkel ノードの増分）。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

16. **単純文。** [ ]
    - `reserve`、`let`、`assume`、`take`、`set`、`given` — 正当化句を運ばない
      文の形。
    - 依存: 14、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。

17. **正当化と引用。** [ ]
    - `by` / `from` の正当化句、引用リスト、`.{ … }` グループ引用、`.*`
      一括引用、およびコンパクトな正当化付き文（`φ by A;`）。
      algorithm 章の `by computation(...)` オプションも含む。
    - 依存: 16、`mizar-syntax` task 14 / S-014（正当化ノードの増分）。仕様:
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)
      §16.5、
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)
      §20.9.2。

18. **`consider` と `reconsider`。** [ ]
    - いずれも正当化を運ぶ `consider … such that … by …` と
      `reconsider … as … by …`。
    - 依存: 17、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。

19. **結論ステップと逐次的等式。** [ ]
    - `thus` / `hence`、`then` 連鎖、およびステップごとの正当化を持つ逐次的
      等式 `.=` ステップ。compact equality statement と zero-step iterative
      equality の grammar-audit 境界（`x = y by A;` と
      `x = y by A .= z by B;`）を含める。
    - 依存: 17、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。

20. **ブロック文。** [ ]
    - `now` / `hereby` ブロックと、`end` 同期を備えた
      `per cases` / `suppose` / `case` ブロック。
    - 依存: 19、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。

21. **ローカル定義。** [ ]
    - `deffunc` / `defpred` のプライベートなローカル定義。
    - 依存: 20、`mizar-syntax` task 13 / S-013。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。

22. **定理と証明。** [ ]
    - `theorem` / `lemma` の item、ラベル、`proof … end` の入れ子、証明本体の
      文の配線。
    - 依存: 21、`mizar-syntax` task 14 / S-014。仕様:
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)。

23. **definition ブロック骨格・correctness 条件・属性定義。** [ ]
    - すべての定義種別が共有する `definition … end` ブロックの形、
      correctness 条件句の形（`existence`、`uniqueness`、`coherence`、
      `consistency`、`compatibility` など、正当化付き）、および最初の具体
      種別としての `attr` 定義。
    - 依存: 22、`mizar-syntax` task 15 / S-015。仕様:
      [06.attributes.md](../../../spec/ja/06.attributes.md)。

24. **述語定義。** [ ]
    - `means` 本体を持つ `pred` 定義。
    - 依存: 23、`mizar-syntax` task 15 / S-015。仕様:
      [09.predicates.md](../../../spec/ja/09.predicates.md)。

25. **ファンクタ定義。** [ ]
    - `means` / `equals` 本体を持つ `func` 定義。
    - 依存: 23、`mizar-syntax` task 15 / S-015。仕様:
      [10.functors.md](../../../spec/ja/10.functors.md)。

26. **mode 定義。** [ ]
    - 正本の `is` 形を用いる `mode` 定義: 属性連鎖と radix 型、型パラメータ、
      任意の `sethood` property 句。
    - 依存: 23、`mizar-syntax` task 15 / S-015。仕様:
      [07.modes.md](../../../spec/ja/07.modes.md)。

27. **`redefine`・`synonym`・`antonym`。** [ ]
    - task 23〜26 の定義種別にまたがる再定義と記法エイリアスの形。
    - 依存: 24、25、26、`mizar-syntax` task 15 / S-015。仕様:
      [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [07.modes.md](../../../spec/ja/07.modes.md)、
      [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [10.functors.md](../../../spec/ja/10.functors.md)、
      [11.symbol_management.md](../../../spec/ja/11.symbol_management.md)。

28. **property 句。** [ ]
    - 定義種別にまたがる property 句（`commutativity`、`idempotence`、
      `involutiveness`、`projectivity`、`reflexivity`、`irreflexivity`、
      `symmetry`、`asymmetry`、`connectedness`、`transitivity`、`sethood`
      など、正当化付き）。
    - 依存: 27、`mizar-syntax` task 15 / S-015。仕様:
      [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [07.modes.md](../../../spec/ja/07.modes.md)、
      [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [10.functors.md](../../../spec/ja/10.functors.md)。

29. **構造体。** [ ]
    - `struct` 定義: フィールド、継承／`extends`、selector 宣言。
    - 依存: 28、`mizar-syntax` task 15 / S-015。仕様:
      [05.structures.md](../../../spec/ja/05.structures.md)。

30. **registration と cluster。** [ ]
    - `registration … end` ブロック、existential / conditional / functorial の
      cluster の形、`reduce`、およびそれらの correctness 条件。
    - 依存: 29、`mizar-syntax` task 15 / S-015。仕様:
      [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)。

31. **テンプレート。** [ ]
    - テンプレートパラメータ、task 8 の生成規則を拡張する bracket 形の型引数
      とパラメータ prefix、`nest` の形。
    - レビュー監査由来の seed ケース
      `tests/miz/pass/parser/pass_parser_template_arguments_001.*` と
      `tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.*`
      を、traceability metadata から runner 実行済みの parse-only coverage へ
      昇格させる。
    - 依存: 30、`mizar-syntax` task 16 / S-016。仕様:
      [18.templates.md](../../../spec/ja/18.templates.md)。

32. **algorithm ブロック・代入・宣言・claim。** [ ]
    - `algorithm` ブロックの形、代入文、`var` / `const` 宣言、
      `ghost var` / `ghost const`、ghost 代入、`snapshot`、top-level `claim`
      block、任意の正当化を持つ `return` 文。
    - 依存: 31、`mizar-syntax` task 16 / S-016。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

33. **algorithm の制御フロー。** [ ]
    - `while` / `do`（`to` / `downto` を含む）、`if` / `else`、`match`、
      `for ... in ... processed ...`、`otherwise` / `exhaustive` の match 終端、
      `break` / `continue`。
    - 依存: 32、`mizar-syntax` task 16 / S-016。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

34. **algorithm の検証句。** [ ]
    - ヘッダーおよび loop の検証句: `requires` / `ensures`、`decreasing`、
      `terminating`、`invariant`、`assert`、およびそれらの正当化。
    - 依存: 33、`mizar-syntax` task 16 / S-016。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

35. **注釈。** [ ]
    - 文レベル注釈、`@[...]` ライブラリ注釈、文字列リテラル注釈引数
      （string-required 位置は frontend の lexing plan がすでに網羅する）。
    - 依存: 34、`mizar-syntax` task 16 / S-016。仕様:
      [21.source_code_annotation_and_atp.md](../../../spec/ja/21.source_code_annotation_and_atp.md)。

### 強化と横断的フォローアップ

36. **recovery の統合と fail コーパスの拡張。** [ ]
    - 全カテゴリの recovery 挙動を監査する: スキップトークンノード、対応
      しない区切り記号、不正な注釈。カテゴリがまだ同期せずに中断する箇所の
      ギャップを埋める。推奨 pass / fail 比率へ向けて fail コーパスを拡張する。
    - 依存: 35。仕様: [recovery.md](./recovery.md)、
      [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

37. **`SurfaceAst` スナップショットベースライン。** [ ]
    - 代表的なコーパスケースについて、`mizar-syntax` のレンダリング
      （その task 3）を使った決定的なスナップショットベースラインを
      `tests/snapshots/` 配下に追加し、スナップショット比較をコーパス
      ランナーに配線する。
    - 依存: 3、35、`mizar-syntax` task 3。仕様:
      [../../mizar-test/ja/snapshot.md](../../mizar-test/ja/snapshot.md)。

38. **決定性プロパティテスト。** [ ]
    - 同一のトークンストリームが同一の `SurfaceAst` ノード順序・範囲・診断
      順序を生むことの crate レベル網羅。frontend の決定性スイートに倣う。
    - 依存: 35。仕様:
      [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

39. **パーサー fuzz ターゲット。** [ ]
    - 任意の UTF-8 上でトークン化と構文解析を駆動するワークスペース fuzz
      ターゲットを追加し、panic が起きず、回復可能診断のみで完了することを
      アサートする。`mizar-frontend` task 29 の real-parser fuzz follow-up は
      frontend-owned target を着地済みであり、この task は parser-owned 側を追跡する。
    - 依存: 36。仕様: [recovery.md](./recovery.md)、
      [../../mizar-frontend/ja/todo.md](../../mizar-frontend/ja/todo.md)
      task 29。

40. **frontend パススルーのフォロースルー。** [ ]
    - 現在の mizar-frontend task 28 parser-recovery surface を超える文法の成長では、
      `mizar-frontend` の新しい follow-up を開く:
      各文法タスクに歩調を合わせて、frontend の recovery マーカーの
      パススルー、診断統合順序、`SurfaceAstCacheKey` の無効化の網羅を維持する。
    - 依存: 5 から始まり、36 で完了する。仕様:
      [../../mizar-frontend/ja/todo.md](../../mizar-frontend/ja/todo.md)
      を参照。

41. **ソース／仕様の対応監査と予約語カバレッジ。** [ ]
    - [grammar.md](./grammar.md)、[pratt.md](./pratt.md)、
      [recovery.md](./recovery.md) のすべての公開 API と約束された挙動を
      実装とテストへトレースし、ギャップをフォローアップタスクとして記録する。
    - [§A.2.4](../../../spec/ja/appendix_a.grammar_summary.md) のすべての
      予約語が、少なくとも 1 つのパーサーコーパステストで消費されていること
      （または、まだ文法位置を持たない将来予約として明示的に記録されている
      こと）を検証し、暗黙に未実装のキーワードを機械的に検出する。
    - 依存: 36。仕様: すべてのモジュール仕様と本 TODO。

42. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-parser/en/` の各英語正本ドキュメントを日本語版と
      比較し、API 一覧、状態、用語、リンク、挙動の約束を同期する。
    - 依存: 41。仕様: リポジトリのドキュメント方針。

43. **公開 enum の前方互換方針。** [ ]
    - 初期の公開 enum ゲートを task 35 後に再確認し、文法成長で追加された
      後続の公開 enum について、`mizar-frontend` task 25 の手続きと
      `mizar-syntax` task 17 の最終監査に整合する形で、
      `#[non_exhaustive]` 対 意図的 exhaustive を決定する。
    - 依存: 35。仕様: すべてのモジュール仕様。

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
