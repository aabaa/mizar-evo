# mizar-parser TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| grammar | [grammar.md](./grammar.md) | `src/grammar.rs` | [~] task 11/12 の最小エントリは現在 `src/lib.rs` にある |
| pratt | [pratt.md](./pratt.md) | `src/pratt.rs` | [~] 明示 fixity の最小 Pratt は現在 `src/lib.rs` にある |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [~] task 12 の最小 recovery は現在 `src/lib.rs` にある |

`mizar-parser` は構文文法を実装する: frontend 適合済みトークンを入力とし、
`mizar_syntax::SurfaceAst` と構文診断を出力する。薄い基盤層（cursor、同期、
recovery 送出、コーパスランナー）をまず作り、その後は一度に数個の生成規則
ずつ文法を成長させる。各文法タスクは `mizar-syntax` のノード語彙の増分および
コーパス拡張と対にし、残りの文法を抱え込まずに 1 タスクを自律的に実装・
テスト・コミットできるよう、意図的に小さく切ってある。

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
  `mizar-syntax` task 8 とともに決定する。
- **コーパスランナーの場所: 未解決。task 3 で解決する。** parse-only の
  コーパスケースは実トークンを必要とするため、ランナーは frontend の実 seam
  を駆動する可能性が高い（先例: `crates/mizar-frontend/tests/lexical_corpus.rs`）。
  代替案は `mizar-test` 内のランナーである。決定して記録する。

## 順序付きタスク一覧

各タスクは、単独で実装・テスト・コミットできる粒度になっている。各タスクの
後で `cargo test -p mizar-parser` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 基盤

1. **モジュール分割と lint 方針のガード。** [ ]
   - `src/lib.rs` を `pub mod grammar;`、`pub mod pratt;`、`pub mod recovery;`
     に分割し、task 11/12 のコードを挙動変更なしに移動する。`parse`、
     `ParseRequest`、`ParserToken`、`ParseOutput` は現在のパスから到達可能の
     まま保つ。
   - `mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加する。
   - テスト: 既存のパーサーテストと frontend seam テストが変更なしに通る。
   - 依存: なし。仕様: [grammar.md](./grammar.md)。

2. **パーサー基盤: cursor、期待トークン診断、同期。** [ ]
   - 有界先読み付きのトークン cursor、精密な範囲を持つ `SyntaxDiagnostic` を
     生成する期待トークン診断ヘルパー、同期集合（`;`、`end`、トップレベル
     item キーワード、EOF）、および `mizar-syntax` の builder API の上に
     構築した recovery ノード送出ヘルパーを追加する。
   - task 12 のアドホックな recovery（`end` 欠落、文字列リテラル欠落、
     回復不能入力）を、観測可能な挙動を変えずにこれらのヘルパーへ一般化する。
   - テスト: 同期が各境界種別までスキップし、スキップ範囲を記録する。期待
     トークン診断が EOF とストリーム中間で正しい第一範囲を運ぶ。
   - 依存: 1、`mizar-syntax` task 2。仕様: [recovery.md](./recovery.md)。

3. **parse-only コーパスランナー。** [ ]
   - ランナーの場所を決定する（`lexical_corpus.rs` の先例に従う frontend
     seam 統合テストか、`mizar-test` のランナーか）。決定をここに記録し、
     ハーネスの責務が変わる場合は
     [../../mizar-test/ja/harness.md](../../mizar-test/ja/harness.md) にも
     記録する。
   - `mizar-test` の discovery と `.expect.toml` 期待値を配線し、
     `tests/miz/{pass,fail}/parser/` のすべてのケースを stage `parse_only` で
     実トークン化を通して実行し、結果・診断・（あれば）スナップショット
     期待値をアサートする。
   - 現在の最小文法に対するケース（トークンストリーム、明示 fixity の中置式、
     `end` 欠落、孤立した `end`）でコーパスをシードし、初日からランナーを
     意味のあるものにする。
   - テスト: ランナーがすべてのケースを決定的に発見する。意図的に不一致に
     したサイドカーが失敗する。シードした pass / fail ケースが診断を強制する。
   - 依存: 2。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)、
     [expectation_schema.md](../../mizar-test/ja/expectation_schema.md)。

### 文法の成長

各文法タスクは同じテンプレートに従い、1 つの変更で行う。

1. 自タスクの生成規則インベントリを、所有する仕様章から
   [grammar.md](./grammar.md) の名前付き節へ転記する（英語と日本語を一緒に）。
   転記がギャップを露見させたら、所有章と
   [付録 A](../../../spec/ja/appendix_a.grammar_summary.md) をブラッシュ
   アップする。
2. 対になる `mizar-syntax` ノード増分を追加する。
3. 同期と recovery を備えた生成規則を実装する。
4. [テストコーパス方針](#テストコーパス方針)に従い、ユニットテストに加えて
   pass / fail コーパスケースと `spec_trace.toml` エントリを提供する。

4. **修飾シンボルと namespace パス。** [ ]
   - `qualified_symbol = { namespace_segment "." } user_symbol` とドット
     区切りモジュールパスの共有ヘルパー。後続の import、型ヘッド、項、引用が
     使う。パスの形だけを扱い、変数 shadowing は resolver 側に残す。
   - コーパス例外: ここではユニットテストを提供する。コーパス網羅は最初の
     消費位置（task 6 と 8）で届け、そこに明示的に列挙する。
   - 依存: 3、`mizar-syntax` task 8（パスノードの増分）。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)
     §12.7、[§A.2.5 / §A.2.8](../../../spec/ja/appendix_a.grammar_summary.md)。

5. **モジュールスケルトンとトップレベル item ディスパッチ。** [ ]
   - モジュールファイルの形と、item 境界での同期を備えたキーワードによる
     トップレベル item ディスパッチ。これにより、後続のすべてのカテゴリが
     安定したスケルトンに収まる。
   - recovery: 未知のトップレベルトークンは、スキップトークンノードを残して
     次の item キーワードまでスキップする。`;` の欠落は次の境界で診断する。
   - 依存: 3、`mizar-syntax` task 6。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

6. **import item。** [ ]
   - alias と相対 prefix（`.` / `..`）を持つ `import` item。形は frontend の
     import 事前走査 stub と整合させる。task 4 のパス形に対する繰り延べ
     コーパスケースを含む。
   - 依存: 4、5。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

7. **export と可視性の item。** [ ]
   - モジュール章に従った export の形、`open` / `inherit`、item 上の
     `public` / `private` 可視性マーカー。
   - 依存: 5。仕様:
     [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

8. **型式。** [ ]
   - 属性連鎖（`non` を含む）、radix / mode の型ヘッド、`of` / `over` 引数
     リスト、struct 修飾の属性参照。項引数は task 9 が着地するまで項エントリ
     のスタブを通す（型と項は相互再帰である）。task 4 の修飾型ヘッドに対する
     繰り延べコーパスケースを含む。
   - 依存: 4、5、`mizar-syntax` task 7。仕様:
     [03.type_system.md](../../../spec/ja/03.type_system.md)、
     [§A.3.2](../../../spec/ja/appendix_a.grammar_summary.md)。

9. **一次項。** [ ]
   - 識別子、数値、項位置の修飾シンボル、括弧付き項、適用形。task 8 の項
     エントリスタブを置き換える。
   - 依存: 8、`mizar-syntax` task 8。仕様:
     [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

10. **selector access / update とドットの役割の surface 形状。** [ ]
    - selector access / update の連鎖（`p.x`、`line.end.y`、`p.x := t`）と
      未解決ドット連鎖の表現。ドットの役割の surface 形状の決定
      （「解決済みおよび保留中の決定」を参照）を解決し、
      [grammar.md](./grammar.md)、仕様の付録、トップレベルの決定一覧に
      記録する。
    - 依存: 9、`mizar-syntax` task 8。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)、
      [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)。

11. **`qua` 修飾。** [ ]
    - selector と適用形に対する優先順位を持つ `term qua type_expression`。
    - 依存: 8、9。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

12. **演算子式（アクティブレキシコン上の Pratt）。** [ ]
    - task 11 の明示 fixity の Pratt パーサーを、`ParserInputs` の fixity
      メタデータで駆動されるユーザー prefix / infix / postfix 演算子へ一般化
      する。優先順位と結合性は
      [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)
      に従う。非結合の連鎖と宙吊り演算子を、ソースローカルな範囲で診断する。
    - 依存: 10、11、`mizar-syntax` task 8（演算子ノードの増分）。仕様:
      [pratt.md](./pratt.md)、
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

13. **原子論理式。** [ ]
    - 述語適用（記号形と識別子形）、`is` 論理式、属性論理式、等式／不等式の
      原子。
    - 依存: 12、`mizar-syntax` task 9。仕様:
      [14.formulas.md](../../../spec/ja/14.formulas.md)。

14. **結合子と量化子。** [ ]
    - 固定結合子テーブル（`not`、`&`、`or`、`implies`、`iff`）とその論理式
      レベルの優先順位（項レベルの fixity から分離したまま保つ）。`st` /
      `holds` 本体を持つ量化子 `for` / `ex`。
    - 依存: 13。仕様: [14.formulas.md](../../../spec/ja/14.formulas.md)、
      [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)。

15. **Fraenkel と集合内包の項。** [ ]
    - `{ term where … : formula }` と関連する集合内包形。区切り句が論理式を
      埋め込むため、論理式の後に置く。
    - 依存: 14、`mizar-syntax` task 8（Fraenkel ノードの増分）。仕様:
      [13.term_expression.md](../../../spec/ja/13.term_expression.md)。

16. **単純文。** [ ]
    - `reserve`、`let`、`assume`、`take`、`set`、`given` — 正当化句を運ばない
      文の形。
    - 依存: 14、`mizar-syntax` task 10。仕様:
      [15.statements.md](../../../spec/ja/15.statements.md)。

17. **正当化と引用。** [ ]
    - `by` / `from` の正当化句、引用リスト、`.{ … }` グループ引用、`.*`
      一括引用、およびコンパクトな正当化付き文（`φ by A;`）。
    - 依存: 16、`mizar-syntax` task 11（正当化ノードの増分）。仕様:
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)
      §16.5。

18. **`consider` と `reconsider`。** [ ]
    - いずれも正当化を運ぶ `consider … such that … by …` と
      `reconsider … as … by …`。
    - 依存: 17。仕様: [15.statements.md](../../../spec/ja/15.statements.md)。

19. **結論ステップと逐次的等式。** [ ]
    - `thus` / `hence`、`then` 連鎖、およびステップごとの正当化を持つ逐次的
      等式 `.=` ステップ。
    - 依存: 17。仕様: [15.statements.md](../../../spec/ja/15.statements.md)。

20. **ブロック文。** [ ]
    - `now` / `hereby` ブロックと、`end` 同期を備えた
      `per cases` / `suppose` / `case` ブロック。
    - 依存: 19。仕様: [15.statements.md](../../../spec/ja/15.statements.md)。

21. **ローカル定義と claim。** [ ]
    - `deffunc` / `defpred` のプライベートなローカル定義と `claim` 文。
    - 依存: 20。仕様: [15.statements.md](../../../spec/ja/15.statements.md)、
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)。

22. **定理と証明。** [ ]
    - `theorem` / `lemma` の item、ラベル、`proof … end` の入れ子、証明本体の
      文の配線。
    - 依存: 21、`mizar-syntax` task 11。仕様:
      [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)。

23. **definition ブロック骨格・correctness 条件・属性定義。** [ ]
    - すべての定義種別が共有する `definition … end` ブロックの形、
      correctness 条件句の形（`existence`、`uniqueness`、`coherence`、
      `consistency`、`compatibility` など、正当化付き）、および最初の具体
      種別としての `attr` 定義。
    - 依存: 22、`mizar-syntax` task 12。仕様:
      [06.attributes.md](../../../spec/ja/06.attributes.md)。

24. **述語定義。** [ ]
    - `means` 本体を持つ `pred` 定義。
    - 依存: 23。仕様: [09.predicates.md](../../../spec/ja/09.predicates.md)。

25. **ファンクタ定義。** [ ]
    - `means` / `equals` 本体を持つ `func` 定義。
    - 依存: 23。仕様: [10.functors.md](../../../spec/ja/10.functors.md)。

26. **mode 定義。** [ ]
    - `means` 本体を持つ `mode` 定義。
    - 依存: 23。仕様: [07.modes.md](../../../spec/ja/07.modes.md)。

27. **`redefine`・`synonym`・`antonym`。** [ ]
    - task 23〜26 の定義種別にまたがる再定義と記法エイリアスの形。
    - 依存: 24、25、26。仕様:
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
    - 依存: 27。仕様: [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [07.modes.md](../../../spec/ja/07.modes.md)、
      [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [10.functors.md](../../../spec/ja/10.functors.md)。

29. **構造体。** [ ]
    - `struct` 定義: フィールド、継承／`extends`、selector 宣言。
    - 依存: 28、`mizar-syntax` task 12。仕様:
      [05.structures.md](../../../spec/ja/05.structures.md)。

30. **registration と cluster。** [ ]
    - `registration … end` ブロック、existential / conditional / functorial の
      cluster の形、`reduce`、およびそれらの correctness 条件。
    - 依存: 29、`mizar-syntax` task 12。仕様:
      [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)。

31. **テンプレート。** [ ]
    - テンプレートパラメータ、task 8 の生成規則を拡張する bracket 形の型引数
      とパラメータ prefix、`nest` の形。
    - 依存: 30、`mizar-syntax` task 13。仕様:
      [18.templates.md](../../../spec/ja/18.templates.md)。

32. **algorithm ブロック・代入・宣言。** [ ]
    - `algorithm` ブロックの形、代入文、`var` / `const` 宣言、`return`。
    - 依存: 31、`mizar-syntax` task 13。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

33. **algorithm の制御フロー。** [ ]
    - `while` / `do`（`to` / `downto` を含む）、`if` / `else`、`match`、
      `break` / `continue`。
    - 依存: 32。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

34. **algorithm の検証句。** [ ]
    - `invariant` / `decreasing` / `terminating`、`assert`、`ghost`、
      `requires` / `ensures`。
    - 依存: 33。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)。

35. **注釈。** [ ]
    - 文レベル注釈、`@[...]` ライブラリ注釈、文字列リテラル注釈引数
      （string-required 位置は frontend の lexing plan がすでに網羅する）。
    - 依存: 34、`mizar-syntax` task 13。仕様:
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
      アサートする。これは `mizar-frontend` task 27 を再開するのと同じ
      トリガーである。frontend のターゲットと一緒に着地するよう調整する。
    - 依存: 36。仕様: [recovery.md](./recovery.md)、
      [../../mizar-frontend/ja/todo.md](../../mizar-frontend/ja/todo.md)
      task 27。

40. **frontend パススルーのフォロースルー。** [ ]
    - 最小 seam を超える文法の成長は `mizar-frontend` task 28 を再開する:
      各文法タスクに歩調を合わせて、frontend の recovery マーカーの
      パススルー、診断統合順序、`SurfaceAstCacheKey` の無効化の網羅を維持し、
      full grammar-recovery 契約が入った時点で frontend の
      `parsing` / `orchestration` の状態を `[x]` に切り替える。
    - 依存: 5 から始まり、36 で完了する。仕様:
      [../../mizar-frontend/ja/todo.md](../../mizar-frontend/ja/todo.md)
      task 28。

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
    - `ParserTokenKind`、`OperatorAssociativity`、`StringRequiredContext`、
      および後から増える公開 enum について、`mizar-frontend` task 25 の手続き
      と `mizar-syntax` task 14 の決定に整合する形で、`#[non_exhaustive]` 対
      意図的 exhaustive を決定する。
    - 依存: 35。仕様: すべてのモジュール仕様。

## 推奨検証

各タスクの後に実行する。

```text
cargo test -p mizar-parser
cargo test -p mizar-syntax
cargo clippy -p mizar-parser --all-targets -- -D warnings
```

frontend seam またはコーパスランナーに触れるタスクでは、次も実行する。

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
- 文法の成長は `mizar-frontend` の保留タスクのトリガー（27 fuzz、28
  grammar-recovery フォロースルー）を発火させる。recovery surface を拡大する
  ときはその TODO を確認する。
- 仕様 EBNF のブラッシュアップは各文法タスクの一部であり、独立した作業系列
  ではない。[grammar.md](./grammar.md) へ転記した生成規則インベントリが各
  タスクの有界な契約であり、修正は所有章と付録 A に、英語と日本語を一緒に
  着地させる。
