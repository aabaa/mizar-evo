# mizar-syntax TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| ast | [ast.md](./ast.md) | `src/ast.rs` | [~] task 12 の最小 surface は現在 `src/lib.rs` にある |
| trivia | [trivia.md](./trivia.md) | `src/trivia.rs` | [ ] |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [~] task 12 の最小 recovery kind は現在 `src/lib.rs` にある |

`mizar-syntax` はデータ定義 crate である。`mizar-parser`、`mizar-frontend`、
および将来の resolver / LSP / formatter の消費者が共有する `SurfaceAst` の形を
所有し、構文解析ロジックも意味論も一切所有しない。構築は 2 つの波で行う。
まず表現基盤（arena、レンダリング、trivia、recovery 語彙）、次にノード語彙で
あり、後者は `mizar-parser` の文法タスクと歩調を合わせて成長する。

依存順序: `ast` 基盤 → `trivia` / `recovery` → `mizar-parser` と対になる
ノード語彙。

## crate の前提条件

この crate は `mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`）に
のみ依存する。task 11/12 の最小境界（`SurfaceAst`、`SurfaceNode`、recovery
kind、`SyntaxDiagnostic`）はすでに `mizar-parser` と
`mizar-frontend::parsing::MizarParserSeam` が消費しているため、ここでの変更は
同じ変更で `cargo test -p mizar-parser` と `cargo test -p mizar-frontend` を
成功状態に保たなければならない。

## 解決済みおよび保留中の決定

- **arena 表現: 未解決。task 2 で解決する。** 現在の同質な `SurfaceNode`
  arena ＋ `SurfaceNodeKind` enum に型付き accessor/view ヘルパーを加える形を
  維持するか、構文カテゴリごとの型付きノード構造体へ移行するか。語彙タスクで
  ノード種別が増殖する前に決定しなければならない。
- **trivia の所有権: 未解決。task 4 で解決する。** `mizar-frontend` はすでに
  コメントとドキュメントコメントを `PreprocessedSource` へ抽出している。
  `SurfaceAst` が付随 trivia を保持するか、frontend 所有の trivia を範囲で
  参照するか、attachment ヒントのみを保存するかを決める。
- **ドットの役割の surface 形状: 未解決。`mizar-parser` task 10 が所有する。**
  パーサーは resolver なしでは selector access と namespace 区切りを完全には
  分離できない（仕様 [§A.2.5](../../../spec/ja/appendix_a.grammar_summary.md)）。
  AST は未解決のドット連鎖を構文的に表現しなければならない。トップレベル
  （[../../todo.md](../../todo.md)「Resolved And Open Decisions」）にも登録
  済みで、[../../mizar-parser/ja/todo.md](../../mizar-parser/ja/todo.md) で
  管理する。
- **公開 enum の前方互換性: 未解決。consumer 前ゲートで初回解決する。**
  `mizar-syntax` が resolver / LSP の入力になる前に、`mizar-frontend`
  task 25 が確立したのと同じ enum ごとの `#[non_exhaustive]` 対 exhaustive の
  決定手続きを適用し、語彙 enum が増えるたびに再確認する。

## 順序付きタスク一覧

各タスクは、単独で実装・テスト・コミットできる粒度になっている。各タスクの後で
`cargo test -p mizar-syntax` を成功状態に保つこと（[推奨検証](#推奨検証)を参照）。

### 表現基盤

1. **モジュール分割と lint 方針のガード。** [ ]
   - `src/lib.rs` を `pub mod ast;` と `pub mod recovery;` に分割し、task 12 の
     型を挙動変更なしに移動する。`mizar-parser` と `mizar-frontend` のパスが
     有効なままになるよう、crate ルートからすべて再エクスポートする。
   - `mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加する:
     workspace lint へのオプトイン、deny ベースライン、将来の `allow` の隣に
     根拠を必須とする。
   - テスト: 既存の消費者が変更なしにコンパイルできる。lint 方針ガードが通る。
   - 依存: なし。仕様: [ast.md](./ast.md)、[recovery.md](./recovery.md)。

2. **AST arena 表現の決定と builder API。** [ ]
   - arena 表現を決定し（同質な kind-enum arena ＋ 型付き accessor view が
     既定候補）、決定と根拠を [ast.md](./ast.md) に記録する。
   - ノード id の不変条件（id は arena への密な index であり、文書化された
     構築順序に従って子が親に先行または後続する）、子の役割（child-role）の
     規約、および arena 内部を晒さずに `mizar-parser` がノードを構築するための
     builder API を定義する。
   - テスト: arena 不変条件（すべての子 id が有効、循環なし）。文書化された
     recovery の例外を除き、親の範囲が子の範囲を包含する。builder の
     round-trip。
   - 依存: 1。仕様: [ast.md](./ast.md)「Public API」。

3. **決定的なスナップショットレンダリング。** [ ]
   - [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)
     「スナップショットテスト」が要求するコーパスのスナップショット
     ベースラインのために、`SurfaceAst` の安定した人間可読テキスト
     レンダリング（kind、範囲、recovered フラグ、子をインデント表示）を
     追加する。
   - レンダリングは実行間・プラットフォーム間でバイト同一でなければならず、
     ハッシュマップの反復順序やアドレスなどの非決定性を含まない。
   - テスト: 繰り返しレンダリングで同一の出力。現在の全ノード種別を網羅する
     代表 fixture。recovery ノードが視認できる形で印付けされる。
   - 依存: 2。仕様: [ast.md](./ast.md)。スナップショット配置は
     [../../mizar-test/ja/snapshot.md](../../mizar-test/ja/snapshot.md)。

4. **trivia モデル。** [ ]
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
     task 6 / parser task 5 の最初の item-node 増分で着地させる。
   - 依存: 2、3。仕様: [trivia.md](./trivia.md)。

5. **recovery 語彙の拡張。** [ ]
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

**公開 enum 前方互換性の初期ゲート。** [ ]
- phase 3 境界で利用可能な各公開 enum（`SurfaceNodeKind`、`SurfaceTokenKind`、
  `SyntaxRecoveryKind`、`SyntaxDiagnosticCode`、および task 4 で導入される
  trivia の kind）について、`mizar-frontend` task 25 の手続きで
  `#[non_exhaustive]` 対 意図的 exhaustive を決定する。
- 各決定を所有モジュール仕様の enum の隣に記録し、parser task 5〜7 によって
  resolver / LSP の消費者が現実的になる前に属性を適用する。
- 依存: 4、5。仕様: [ast.md](./ast.md)、[trivia.md](./trivia.md)、
  [recovery.md](./recovery.md)。

### ノード語彙（`mizar-parser` の文法タスクと対）

各領域のノード種別は**増分的に**追加する: 各増分は、それを構築する
`mizar-parser` 文法タスクと同じ変更で着地し（変更の粒度はパーサー todo の
番号付けが統制する）、各増分はスナップショットレンダリングを拡張する。以下の
語彙タスクは、対になるパーサータスクの最後が着地した時点でチェックを入れる。
それを構築するパーサータスクに先行して、投機的にノード種別を追加しない。
仕様参照は [doc/spec/ja/](../../../spec/ja/00.index.md) 配下の規範的な文法章で
ある。

6. **モジュールと item のノード。** [ ] — `mizar-parser` task 5〜7 と対。
   - モジュールファイルの形、トップレベル item リストとキーワードで
     ディスパッチ可能な item 種別（parser task 5）。alias と相対 prefix を
     持つ import item（parser task 6）。export、`open` / `inherit`、可視性の
     形（parser task 7）。
   - 仕様: [12.modules_and_namespaces.md](../../../spec/ja/12.modules_and_namespaces.md)。

7. **型式のノード。** [ ] — `mizar-parser` task 8 と対。
   - 属性連鎖（`non` を含む）、radix / mode の型ヘッド、`of` / `over` 引数、
     struct 修飾の属性参照。
   - 仕様: [03.type_system.md](../../../spec/ja/03.type_system.md)、
     [§A.3.2](../../../spec/ja/appendix_a.grammar_summary.md)。

8. **項のノード。** [ ] — `mizar-parser` task 4、9〜12、15 と対。
   - 最初の増分: parser task 4 が必要とする修飾シンボル／namespace パスの
     ノード。続いて一次項（parser task 9）、未解決ドット連鎖と selector
     access / update（parser task 10、ドットの役割の surface 形状の決定を
     含む）、functional structure update、`qua`（parser task 11）、task 12 の
     `InfixExpression` を prefix / postfix 形へ一般化する演算子式ノード
     （parser task 12）、Fraenkel / 集合内包形（parser task 15）。一次項の
     網羅には `it`、選択式（`the type_expression`）、構造体コンストラクタ、
     集合列挙リテラル、適用形を含む。
   - 仕様: [13.term_expression.md](../../../spec/ja/13.term_expression.md)、
     [appendix_b.operator_precedence.md](../../../spec/ja/appendix_b.operator_precedence.md)。

9. **論理式のノード。** [ ] — `mizar-parser` task 13〜14 と対。
   - 原子述語適用、`is` 論理式、属性論理式（parser task 13）。結合子と
     量化子（`for` / `ex` / `st` / `holds`）（parser task 14）。
   - 仕様: [14.formulas.md](../../../spec/ja/14.formulas.md)。

10. **文のノード。** [ ] — `mizar-parser` task 16 と 18〜21 と対。
    - 単純文 `reserve`、`let`、`assume`、`take`、`set`、`given`（parser
      task 16）。`consider` / `reconsider`（parser task 18）。
      `thus` / `hence`、`then` 連鎖、逐次的等式 `.=`（parser task 19）。
      `now` / `hereby` と `per cases` / `suppose` ブロック（parser task 20）。
      `deffunc` / `defpred` のローカル定義と `claim`（parser task 21）。
    - 仕様: [15.statements.md](../../../spec/ja/15.statements.md)。

11. **定理・証明・正当化のノード。** [ ] — `mizar-parser` task 17 と 22 と対。
    - 正当化句（`by`、`from`）、`.{ … }` と `.*` を含む引用形（parser
      task 17）に加え、`by computation(...)` オプションノード（parser
      task 17）。`theorem` / `lemma` の item、ラベル、`proof … end` の入れ子
      （parser task 22）。
    - 仕様: [16.theorems_and_proofs.md](../../../spec/ja/16.theorems_and_proofs.md)、
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)
      §20.9.2。

12. **定義・構造体・registration のノード。** [ ] — `mizar-parser`
    task 23〜30 と対。
    - definition ブロック骨格、correctness 条件句、`attr` 定義（parser
      task 23）。`pred` / `func` / `mode` の本体（parser task 24〜26）。
      `redefine`、`synonym` / `antonym`（parser task 27）。property 句
      （parser task 28）。フィールドと継承を持つ `struct` 定義（parser
      task 29）。registration と cluster の形、`reduce`（parser task 30）。
    - 仕様: [06.attributes.md](../../../spec/ja/06.attributes.md)、
      [07.modes.md](../../../spec/ja/07.modes.md)、
      [09.predicates.md](../../../spec/ja/09.predicates.md)、
      [10.functors.md](../../../spec/ja/10.functors.md)、
      [05.structures.md](../../../spec/ja/05.structures.md)、
      [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)。

13. **テンプレート・アルゴリズム・注釈のノード。** [ ] — `mizar-parser`
    task 31〜35 と対。
    - テンプレートパラメータと bracket 形の型引数（parser task 31）。
      algorithm ブロック・代入・宣言・ghost 宣言 / 代入・snapshot・return
      （parser task 32）。processed collection loop と match 終端を含む制御
      フロー（parser task 33）。検証句（parser task 34）。
      文レベル注釈、`@[...]` ライブラリ注釈、文字列リテラル注釈引数
      （parser task 35）。
    - 仕様: [18.templates.md](../../../spec/ja/18.templates.md)、
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md)、
      [21.source_code_annotation_and_atp.md](../../../spec/ja/21.source_code_annotation_and_atp.md)。

### 横断的フォローアップ

14. **公開 enum の前方互換方針。** [ ]
    - 語彙が完成した時点で公開 enum 初期ゲートを再確認し、後続のノード語彙
      増分で追加された公開 enum について、`#[non_exhaustive]` 対 意図的
      exhaustive を決定する。
    - 最終決定を所有モジュール仕様の enum の隣に記録し、残りの属性を適用する。
    - 依存: 13。仕様: すべてのモジュール仕様。

15. **ソース／仕様の対応監査。** [ ]
    - `mizar-frontend` task 16 の監査に倣う: [ast.md](./ast.md)、
      [trivia.md](./trivia.md)、[recovery.md](./recovery.md) のすべての公開
      API と約束された挙動を実装とテストへトレースし、ギャップをフォロー
      アップタスクとして記録する。
    - 依存: 13。仕様: すべてのモジュール仕様と本 TODO。

16. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-syntax/en/` の各英語正本ドキュメントを日本語版と
      比較し、API 一覧、状態、用語、リンク、挙動の約束を同期する。
    - 依存: 15。仕様: リポジトリのドキュメント方針。

17. **rustdoc サマリ。** [ ] 保留。
    - `mizar-frontend` task 26 と同じワークスペースレベルの保留。再着手
      トリガー: フロントエンドパイプラインの外の最初の長命な消費者
      （resolver または `mizar-lsp`）が `mizar-syntax` に対するコーディングを
      始めるとき、またはワークスペースが rustdoc 方針を採用したとき —
      いずれか早い方。
    - 依存: 14。仕様: リポジトリのドキュメント方針。

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
- 語彙の成長は `mizar-parser` の文法タスクがペースを決める。それを構築する
  パーサータスクに先行して、投機的にノード種別を追加しない。
- `SurfaceAst` は内部コンパイラデータであり、安定した外部スキーマではない。
  スナップショットレンダリング（task 3）がコーパスベースラインに対する
  安定性の表面である。
