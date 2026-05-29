# Module: import_prescan

> Canonical language: English. English canonical version: [../en/import_prescan.md](../en/import_prescan.md).

## Purpose

このモジュールは、生の字句解析器(lexer)の出力から、モジュールのインポートプレリュード(import prelude)を抽出します。

インポート事前スキャン(import pre-scan)は、最終トークンの曖昧性解消(disambiguation)とパース(parsing)の前に、アクティブ字句環境(active lexical environment)を構築するために存在します。この処理は浅いものに留めます。インポートの形をした構文とソーススパンは認識しますが、モジュールの解決(resolution)、シンボルの読み込み、パッケージ可視性(visibility)の検証は行いません。

## Public API

実装済み API:

```rust
pub struct ImportPrelude {
    pub imports: Vec<ImportStub>,
    pub end: SourcePos,
    pub diagnostics: Vec<ImportPrescanDiagnostic>,
}

pub struct ImportStub {
    pub path: RawModulePath,
    pub alias: Option<RawModuleAlias>,
    pub span: SourceRange,
}

pub struct RawModulePath {
    pub spelling: String,
    pub relative: Option<RawModuleRelativePrefix>,
    pub components: Vec<RawModulePathComponent>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude;
```

責務の境界は以下のとおりです。

- 入力は `RawTokenStream`;
- 出力は生のインポート綴り(spelling)とソーススパン;
- モジュールの解決は別の層が担当する。

## Algorithm

現在の実装は、小さなトークン分割器と、回復可能(recoverable)な文(statement)パーサーの組み合わせです。

1. まず `RawTokenStream` を、インポート事前スキャン専用のトークンに変換します。レイアウト(空白類)は無視します。`LexemeRun` は、ソーススパンを保ったまま `Word`, `.`, `..`, `,`, `;`, `*`, `{`, `}`, `Other` に分割します。`NumeralLike`、注釈マーカー(annotation marker)、生のエラーは `Other` として扱います。
2. プレリュードの終了位置は、最初の非レイアウトトークンの開始位置に初期化します。空のストリームなら `0` です。
3. カーソルが語 `import` を指している間、インポート文を 1 つ読みます。`import` を消費した後、セミコロン、EOF、または不正な境界に到達するまで、カンマ区切りのモジュール別名宣言を繰り返し読みます。
4. `parse_module_path` は省略可能な相対接頭辞(`.` または `..`)を受け取り、その後に識別子の形をしたパス構成要素をドット区切りで読みます。ドットの次が `{` の場合は分岐インポート(branch import)の開始として扱い、基底パスには含めません。
5. `parse_module_alias_decls` は省略可能な `as alias` 接尾辞を読みます。別名は識別子の形でなければなりません。別名が欠けていても、パスが復元できている場合は、診断を出したうえでインポートスタブを保持します。
6. 分岐インポートは `base.{child, other}` を読み、複数の `ImportStub` に展開します。展開後の綴りはソーステキスト上で連続していないため、`source_segments` に基底スパンと分岐構成要素のスパンの両方を記録します。
7. 不正な入力では、信頼できる最小のスパンに診断を付けます。少なくとも 1 つの宣言を復元できたのにセミコロンがなければ `MissingSemicolon` を、そうでなければ `UnexpectedToken`、またはパス/別名に応じたより具体的な診断を出し、文の末尾まで回復します。
8. トップレベルのインポート文の開始ではないトークンに到達したら、そこでプレリュードのスキャンは完全に終了します。

スキャナーは、プレリュード終了後にインポートを探してはいけません。後続のインポートの形をしたテキストは、第 12 章のインポート配置規則に従い、パーサーが構文エラーとして扱います。

このアルゴリズムは、`import` や `as` がそのパーサー位置で合法なトークンかどうかを予約テーブルに問い合わせません。これはパーサーの前段の浅い処理であり、後続フェーズがモジュール解決とアクティブ字句環境の構築を行うために必要な、生のインポート形状だけを集めます。

## Accepted Syntax

事前スキャンは、第 12 章のインポート構文を認識します。

```ebnf
import_stmt       ::= "import" module_alias_decl { "," module_alias_decl } ";" ;
module_alias_decl ::= module_path [ "as" module_identifier ]
                    | module_branch_import ;
module_branch_import
                  ::= module_path ".{" module_identifier { "," module_identifier } "}" ;
module_path       ::= [ relative_prefix ] module_identifier { "." module_identifier } ;
relative_prefix   ::= "." | ".." ;
module_identifier ::= identifier ;
```

認識は綴りに基づきます。`import` と `as` は予約語の綴りであり、モジュールパスの構成要素は識別子の形をした生の字句単位(lexeme)です。

分岐インポート構文は、同じ接頭辞の配下にある複数のモジュールパスを書くための略記です。たとえば `import algebra.linear.{eigen_value, jordan};` は、`algebra.linear.eigen_value` と `algebra.linear.jordan` の生スタブに展開されます。分岐展開された綴りはソーステキスト上で必ずしも連続しないため、正確なソース範囲が必要な利用側は `source_segments` を使います。

事前スキャンは、生スキャンに句読点(punctuation)を事前分割させることを要求してはいけません。`.`、`..`、`,`、`;`、`.{`、`}` を認識するために `LexemeRun` の内部を調べて分割してよいですが、ソーススパンは保持しなければなりません。たとえば `std.algebra.group;` を覆う生ランからも、モジュールパス `std.algebra.group` と終端のセミコロンを抽出できます。

## Non-Goals

インポート事前スキャンは以下を行いません。

- 絶対/相対のモジュールパスを解決する;
- モジュールの存在を確認する;
- インポートの循環を計算する;
- エクスポートされたシンボルを読み込む;
- 別名の衝突を判定する;
- インポートしたモジュールの完全な IR を調べる;
- 通常のモジュール宣言をパースする.

## Error Handling

不正なインポートプレリュード構文では診断を出しますが、決定的に復元できる `ImportStub` は可能な限り保持します。

例:

- インポート文の後にセミコロンがない;
- カンマの前、または相対接頭辞の後にモジュールパスがない;
- `as` の後に別名がない;
- モジュールパスの構成要素が空;
- 分岐インポートリストの後に `}` がない;
- プレリュードの終端の前に予期しないトークンがある。

## Tests

テストでは以下を確認します。

- 空のプレリュード;
- 単一のインポート;
- カンマ区切りのインポート;
- 分岐インポート;
- 別名;
- `.` / `..` を使う相対インポート;
- `export`, `definition`, `registration`、定理に類する項目などでプレリュードのスキャンが終了すること;
- 不正なインポートからの回復;
- プレリュード終了後にインポートの形をしたテキストを探しに行かないこと。
