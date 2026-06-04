# モジュール: preprocess

> 正本は英語です。英語版: [../en/preprocess.md](../en/preprocess.md)。

状態: 計画中。

## 目的

このモジュールはフロントエンドパイプラインの Step 2（前処理）を実装し、字句環境構築と字句解析が利用する `PreprocessedSource` を生成する。

`SourceUnit` に対して `mizar-lexer` のソース前処理ヘルパーを統制する。すなわち、コード領域の ASCII 検証、コメントとドキュメントコメントの分離、字句テキスト内の注釈構文保持、浅いトップレベルインポート事前走査である。統制と、`mizar-session` の `SourceRange` へのスパン橋渡しを所有するが、コメント除去やインポート走査のアルゴリズムは所有しない（それらは `mizar-lexer` にある）。また、トークン化・構文解析・インポート解決も行わない。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 2: Preprocess Source」「Comments and Doc Comments Are Source Metadata」「Import Pre-Scan Is Shallow」「Annotations Are Parser-Owned Syntax」を参照。

## 公開 API

```rust
pub struct PreprocessedSource {
    pub source_id: SourceId,
    pub lexical_text: LexicalText,
    pub lexical_hash: Hash,
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub import_stubs: Vec<ImportStub>,
    pub source_map: LexicalSourceMap,
    pub diagnostics: Vec<PreprocessDiagnostic>,
}

pub struct LexicalText {
    pub text: Arc<str>,
}

pub struct Comment {
    pub kind: CommentKind,
    pub source_range: SourceRange,
}

pub struct DocComment {
    pub source_range: SourceRange,
    pub raw_body: Arc<str>,
}

pub struct LexicalSourceMap { /* lexical-text offsets -> SourceRange */ }

pub struct ImportStub {
    pub path: ImportStubPath,
    pub alias: Option<ImportStubAlias>,
    pub span: SourceRange,
}

pub struct ImportStubPath {
    pub spelling: Arc<str>,
    pub relative: Option<ImportStubRelativePrefix>,
    pub components: Vec<Arc<str>>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

pub enum ImportStubRelativePrefix {
    Current,
    Parent,
}

pub struct ImportStubAlias {
    pub spelling: Arc<str>,
    pub span: SourceRange,
}

pub struct PreprocessDiagnostic {
    pub kind: PreprocessDiagnosticKind,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
}

pub enum PreprocessDiagnosticKind {
    SourcePrecondition(SourcePreprocessDiagnosticCode),
    ImportPrescan(ImportPrescanDiagnosticCode),
    RawImportScan,
}

pub fn preprocess(
    source: &SourceUnit,
    bridge: &mut SpanBridge,
) -> Result<PreprocessedSource, SpanBridgeError>;
```

`PreprocessedSource` は、アーキテクチャのインターフェースを反映する。`diagnostics` を追加し、Step 2 の字句前提・コメント構造の診断が出力とともに運ばれて、統制層で決定的な順序で後から統合されるようにする。

ユーザー入力に由来する回復可能な問題に対して、`preprocess` は `Ok(PreprocessedSource)` を返す。コメント構造、ASCII 領域、インポート事前走査のエラーは、中断せず、回復済みの字句テキストとともに診断として記録するので、字句解析器が引き続き実行され、さらなる問題を報告できる。`Err(SpanBridgeError)` は、写像不能なスパンや衝突するマップ登録のような、内部の座標橋渡し不変条件の失敗に限る。

## 依存関係

- 内部: `source`（`SourceUnit` を提供）、`span_bridge`（ソースの preprocess map を登録し、字句解析器の前処理マップオフセットを `mizar-session` の `SourceRange` へ変換）、`lexical_env` と `lexing`（`PreprocessedSource` を利用）。
- 外部: `mizar-lexer`（`preprocess_source_for_lexing`、`PreprocessedLexicalSource`、`SourcePreprocessMap`、`CommentTrivia`、`SourcePreprocessDiagnostic`、`scan_import_prelude`、`ImportPrelude`、`mizar_lexer::ImportStub`、`RawModuleRelativePrefix`、`ImportPrescanDiagnostic`、`SourcePreprocessDiagnosticCode`、`ImportPrescanDiagnosticCode`、`scan_raw`）、`mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`）。

## データ構造

### 字句テキストとソースマップ

`LexicalText` は、字句解析器がスキャンする、コメント除去済み・注釈保持のテキストである。`LexicalSourceMap` は、字句解析器の `SourcePreprocessMap` を `SourceUnit` の `LineMap` / `LoadingMap` とともにラップし、任意の字句テキストのバイトオフセットを `span_bridge` を通じて、第一の `SourceRange`（および除去されたコメントをまたぐ場合は隣接アンカーの合成）へ変換できるようにする。コメントが除去された位置に挿入された合成空白は、厳密なユーザー記述範囲を持たない。session の `MappedSourceRange` が primary を必要とする場合、その primary はアンカーへのフォールバックで縮退したものであり、厳密なソーステキストとして扱ってはならない。`lexical_hash` は、字句テキストとフロントエンドの前処理バージョン領域から計算され、コメントのみの編集で字句テキストが変わらない場合に、下流のトークン／AST 再利用キーになる。

### コメントとドキュメントコメント

通常の `Comment` は、整形・デバッグ用にのみ保持され、字句解析器には渡されない。`DocComment` はソース範囲と生本文を保持するので、パーサーが後でドキュメント化可能な項目に付与できる。付与はパーサーの関心事のままであり、構造化タグの解析は先送りする。いずれも、すでにソース座標へ変換された `SourceRange` を持つ。

### インポートスタブ

`ImportStub` は、`mizar-lexer` のインポート事前走査スタブを、フロントエンドが写像した対応物である。字句解析器の `RawModulePath` / `RawModuleAlias` の形を写すが、すべてのスパンはすでに session の `SourceRange` へ変換されている。生のドット区切りモジュールパス、相対 prefix（`./` と `../` の区別）、分岐インポートの分割ソース被覆は、`path.spelling`、`path.relative`、`path.components`、`path.source_segments` に含まれる。これらは解決済みインポートではなく、アクティブ字句環境を要求し、語彙読み込みが失敗したときに良い診断を出すのに十分なだけの情報を持つ。パッケージ／モジュールの存在、可視性、エクスポート検査、再エクスポート意味論は、モジュール解決へ先送りする。

`PreprocessDiagnostic` は、`SourcePreprocessDiagnostic`、`ImportPrescanDiagnostic`、およびフロントエンドローカルな生インポート事前走査の失敗を、フロントエンド側で写像した診断形式である。生の字句解析器診断構造体は入力として消費し、即座に変換する。公開診断は写像済みの session 範囲を持ち、preprocess マッピングが複合または縮退の場合は副次の `SourceAnchor` も保持する。

## アルゴリズム / ロジック

### SourceUnit の前処理

1. `SourceUnit.source_text` に対して `mizar_lexer::preprocess_source_for_lexing` を呼び、コード領域の ASCII 検証（コメント内の Unicode は許可し、文字列リテラル内の Unicode 受理はパーサー支援字句解析の契約に委ねる）、通常コメントの除去、ドキュメントコメントの保持、字句テキスト内の注釈構文保持を行い、`SourcePreprocessMap` を生成する。
2. 字句解析器の `SourcePreprocessMap` を session の `PreprocessMap` へ変換し、`SourceId` に対して可変な `SpanBridge` へ登録する。
3. 保持された各コメント・ドキュメントコメント・前処理診断のスパンを、字句／ソースオフセットから `span_bridge` を通じて `mizar-session` の `SourceRange` へ変換する。
4. 字句テキストを生スキャン（`scan_raw`）する。成功した場合は `scan_import_prelude` を実行して `ImportStub` とインポート事前走査診断を抽出し、それらのスパンを `SourceRange` へ変換する。
5. 生スキャンが失敗した場合は、字句テキスト全体（字句テキストが空ならソース先頭のゼロ長範囲）を覆うフロントエンドローカルなインポート事前走査診断を記録し、`import_stubs` を空のまま続行する。部分的な生ストリームから import を推測しない。現在の `mizar_lexer::LexError` はスパンや部分トークンのペイロードを持たないため、生スキャン失敗位置の精密化は、将来の回復可能な生スキャナー契約に委ねる。
6. コメント構造・ASCII 前提・インポート事前走査の診断を `diagnostics` に集約し、ソース順を保つ。
7. 最終的な字句テキストとフロントエンドの前処理バージョンから `lexical_hash` を計算する。
8. 前処理マップと line／loading マップから `LexicalSourceMap` を組み立て、`PreprocessedSource` を返す。

インポート事前走査は、生の字句解析器出力を消費する。生スキャン自体はインポートを解釈しない。`scan_raw` は厳密なので、前処理が回復済み字句テキストを返していても、生スキャンには失敗しうる。その失敗は、Step 2 の浅いインポート抽出だけを無効化する。Step 4 のトークン化は独立に回復適合を行い、トークンレベルの診断を報告する。

## エラー処理

Step 2 の診断は、致命的エラーとして送出せず、`PreprocessedSource.diagnostics` に運ぶ。

- コード領域の非 ASCII 文字や、その他の字句前提（`SourcePreprocessDiagnostic`）。
- 未終端ブロックコメントや、その他のコメント構造の問題。
- アクティブ字句環境の構築を妨げるインポート事前走査の失敗（`ImportPrescanDiagnostic`）。
- インポート事前走査中の生スキャン失敗。前処理が回復済み字句テキストを返し続けられるよう、粗い字句テキスト被覆を持つフロントエンドローカルな `PreprocessDiagnostic` 変種として表す。

語彙読み込みを妨げるほど深刻な事前走査の失敗は記録され、統制層が、該当インポートの字句環境拡張をスキップしつつ、ファイルの残りをトークン化するかどうかを判断できるようにする。前処理は意味的事実を主張しない。

## テスト

主要シナリオ:

- 通常コメントは `lexical_text` から除去されるが、正しい `SourceRange` を持つ `Comment` として保持される。
- ドキュメントコメントは生本文とソース範囲とともに保持され、字句テキストには渡されない。
- 注釈構文（`@latex(...)`、`@[...]`）は `lexical_text` に残る。
- 除去されたコメントをまたぐ字句範囲は、合成マッピングを生む。
- 合成空白は、厳密なユーザー記述範囲ではなく、アンカーに支えられた縮退マッピングとしてのみ表面化する。
- コメントのみの編集で `lexical_text` が変わらない場合、`lexical_hash` は安定する。
- トップレベルの `import` 形式は、生パス・任意の alias・`path.relative`・`path.source_segments`・スパンを持つ `ImportStub` を生み、`./` と `../` の相対 prefix を区別して保持する。不正なインポートは、中断せず `ImportPrescanDiagnostic` を生む。
- インポート事前走査中の厳密な生スキャン失敗は、診断と空の `import_stubs` を生み、前処理を中断しない。
- コード領域の非 ASCII 文字は字句前提診断として報告され、前処理は回復済み字句テキストを返す。
- 未終端ブロックコメントは報告され、回復される。

## 制約と前提

- このモジュールは、トークン化・構文解析・インポート解決を行わない。
- コメント除去・ASCII 検証・インポート事前走査のアルゴリズムは `mizar-lexer` に属する。このモジュールはそれらを統制し、スパン橋渡しを所有する。
- 注釈構文はパーサー所有のため、字句テキストに残る。前処理は注釈を別個のメタデータチャネルに集約しない。注釈の文字列引数内の Unicode は、パーサー支援字句解析の契約が、ASCII 前提診断より前に文字列必須スパンを識別できるようになってから受理する。それまでは、コメント外の非 ASCII は字句前提診断のままである。
- 合成空白は、厳密で第一のユーザー向けソース範囲にはならない。アンカーへのフォールバックで縮退したものは、session の `MappedSourceRange` の形を満たすためだけに許可される。
- `PreprocessedSource` の生成は、`source_hash` とフロントエンドバージョンでキー付けされる。下流のトークン化と構文再利用は `lexical_hash` を使うので、字句テキストが変わらないコメントのみの編集では、後続成果物を保持できる。
