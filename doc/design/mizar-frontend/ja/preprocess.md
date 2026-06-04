# モジュール: preprocess

> 正本は英語です。英語版: [../en/preprocess.md](../en/preprocess.md)。

状態: planned。

## 目的

このモジュールはフロントエンドパイプラインの Step 2（前処理）を実装し、字句環境構築と字句解析が消費する `PreprocessedSource` を生成する。

`SourceUnit` に対して `mizar-lexer` のソース前処理ヘルパーを統制する。すなわちコード領域の ASCII 検証、コメントとドキュメントコメントの分離、字句テキスト内の注釈構文保持、浅いトップレベルインポート事前走査である。統制と `mizar-session` `SourceRange` への span 橋渡しを所有するが、コメント除去やインポート走査のアルゴリズムは所有しない（それらは `mizar-lexer` にある）。また、トークン化・構文解析・インポート解決も行わない。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 2: Preprocess Source」「Comments and Doc Comments Are Source Metadata」「Import Pre-Scan Is Shallow」「Annotations Are Parser-Owned Syntax」を参照。

## 公開 API

```rust
pub struct PreprocessedSource {
    pub source_id: SourceId,
    pub lexical_text: LexicalText,
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
    pub relative: bool,
    pub components: Vec<Arc<str>>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

pub struct ImportStubAlias {
    pub spelling: Arc<str>,
    pub span: SourceRange,
}

pub fn preprocess(
    source: &SourceUnit,
    bridge: &mut SpanBridge,
) -> Result<PreprocessedSource, SpanBridgeError>;
```

`PreprocessedSource` はアーキテクチャのインターフェースを反映する。`diagnostics` を追加し、Step 2 の字句前提・コメント構造診断が出力とともに運ばれ、統制層で決定的な順序で後から統合されるようにする。

ユーザー入力由来の回復可能な問題に対して、`preprocess` は `Ok(PreprocessedSource)` を返す。コメント構造、ASCII 領域、インポート事前走査のエラーは、中断せず回復済みの字句テキストとともに診断として記録するので、字句解析器が引き続き実行され、さらなる問題を報告できる。`Err(SpanBridgeError)` は、写像不能な span や衝突するマップ登録のような内部座標橋渡し不変条件の失敗に限る。

## 依存関係

- 内部: `source`（`SourceUnit` を提供）、`span_bridge`（ソースの preprocess map を登録し、字句解析器の前処理マップオフセットを `mizar-session` `SourceRange` へ変換）、`lexical_env` と `lexing`（`PreprocessedSource` を消費）。
- 外部: `mizar-lexer`（`preprocess_source_for_lexing`、`PreprocessedLexicalSource`、`SourcePreprocessMap`、`CommentTrivia`、`SourcePreprocessDiagnostic`、`scan_import_prelude`、`ImportPrelude`、`mizar_lexer::ImportStub`、`ImportPrescanDiagnostic`、`scan_raw`）、`mizar-session`（`SourceId`、`SourceRange`）。

## データ構造

### 字句テキストとソースマップ

`LexicalText` は字句解析器がスキャンする、コメント除去済み・注釈保持のテキストである。`LexicalSourceMap` は字句解析器の `SourcePreprocessMap` を `SourceUnit` の `LineMap` / `LoadingMap` とともにラップし、任意の字句テキストのバイトオフセットを `span_bridge` を通じて第一の `SourceRange`（および除去されたコメントをまたぐ場合は隣接アンカーの合成）へ変換できるようにする。コメントが除去された位置に挿入された合成空白は、第一のユーザー範囲としては報告されない。

### コメントとドキュメントコメント

通常の `Comment` は整形・デバッグ用にのみ保持され、字句解析器には渡されない。`DocComment` はソース範囲と生本文を保持するので、パーサーが後でドキュメント化可能な項目に付与できる。付与はパーサーの関心事のままであり、構造化タグの解析は先送りする。両者とも、既にソース座標へ変換された `SourceRange` を持つ。

### インポートスタブ

`ImportStub` は `mizar-lexer` のインポート事前走査 stub を frontend が写像した対応物である。lexer の `RawModulePath` / `RawModuleAlias` の形を写すが、すべての span は既に session `SourceRange` へ変換されている。生のドット区切りモジュールパスと分岐インポートの分割ソース被覆は、`path.spelling`、`path.components`、`path.source_segments` に含まれる。解決済みインポートではなく、アクティブ字句環境を要求し、語彙読み込みが失敗したときに良い診断を出すのに十分なだけである。パッケージ／モジュールの存在、可視性、エクスポート検査、再エクスポート意味論はモジュール解決へ先送りする。

## アルゴリズム / ロジック

### SourceUnit の前処理

1. `SourceUnit.source_text` に対して `mizar_lexer::preprocess_source_for_lexing` を呼び、コード領域の ASCII 検証（コメント内の Unicode は許可し、文字列リテラル内 Unicode の受理は parser-assisted lexing contract に委ねる）、通常コメントの除去、ドキュメントコメントの保持、字句テキスト内の注釈構文保持を行い、`SourcePreprocessMap` を生成する。
2. 字句解析器の `SourcePreprocessMap` を session `PreprocessMap` へ変換し、`SourceId` に対して mutable `SpanBridge` へ登録する。
3. 保持された各コメント・ドキュメントコメント・前処理診断のスパンを、字句／ソースオフセットから `span_bridge` を通じて `mizar-session` `SourceRange` へ変換する。
4. 字句テキストを生スキャン（`scan_raw`）する。成功した場合は `scan_import_prelude` を実行して `ImportStub` とインポート事前走査診断を抽出し、それらのスパンを `SourceRange` へ変換する。
5. 生スキャンが失敗した場合は、字句テキスト全体（字句テキストが空ならソース先頭のゼロ長範囲）を覆う frontend-local なインポート事前走査診断を記録し、`import_stubs` を空のまま続行する。部分的な raw stream から import を推測しない。現在の `mizar_lexer::LexError` は span や部分 token payload を持たないため、raw-scan 失敗位置の精密化は将来の recoverable raw-scanner contract に委ねる。
6. コメント構造・ASCII 前提・インポート事前走査の診断を `diagnostics` に集約し、ソース順を保つ。
7. 前処理マップと line／loading マップから `LexicalSourceMap` を組み立て、`PreprocessedSource` を返す。

インポート事前走査は生の字句解析器出力を消費する。生スキャン自体はインポートを解釈しない。`scan_raw` は strict なので、前処理が回復済み字句テキストを返していても生スキャンに失敗し得る。その失敗は Step 2 の浅い import 抽出だけを無効化する。Step 4 の tokenization は独立に回復適合を行い、トークンレベル診断を報告する。

## エラー処理

Step 2 の診断はハードエラーとして送出せず、`PreprocessedSource.diagnostics` に運ぶ。

- コード領域の非 ASCII 文字やその他の字句前提（`SourcePreprocessDiagnostic`）。
- 未終端ブロックコメントやその他のコメント構造の問題。
- アクティブ字句環境構築を妨げるインポート事前走査の失敗（`ImportPrescanDiagnostic`）。
- インポート事前走査中の raw-scan 失敗。前処理が回復済み字句テキストを返し続けられるよう、粗い字句テキスト被覆を持つ frontend-local な `PreprocessDiagnostic` variant として表す。

語彙読み込みを妨げるほど深刻な事前走査の失敗は記録され、統制層が該当インポートの字句環境拡張をスキップしつつ、ファイルの残りをトークン化するか判断できるようにする。前処理は意味的事実を主張しない。

## テスト

主要シナリオ:

- 通常コメントは `lexical_text` から除去されるが、正しい `SourceRange` を持つ `Comment` として保持される。
- ドキュメントコメントは生本文とソース範囲とともに保持され、字句テキストには渡されない。
- 注釈構文（`@latex(...)`、`@[...]`）は `lexical_text` に残る。
- 除去されたコメントをまたぐ字句範囲は合成マッピングを生む。
- トップレベル `import` 形式は生パス・任意の alias・`path.source_segments`・span を持つ `ImportStub` を生み、不正なインポートは中断せず `ImportPrescanDiagnostic` を生む。
- インポート事前走査中の strict raw-scan 失敗は、診断と空の `import_stubs` を生み、前処理を中断しない。
- コード領域の非 ASCII 文字は字句前提診断として報告され、前処理は回復済み字句テキストを返す。
- 未終端ブロックコメントは報告され回復される。

## 制約と前提

- このモジュールはトークン化・構文解析・インポート解決を行わない。
- コメント除去・ASCII 検証・インポート事前走査のアルゴリズムは `mizar-lexer` に属する。このモジュールはそれらを統制し、span 橋渡しを所有する。
- 注釈構文はパーサー所有のため字句テキストに残る。前処理は注釈を別個のメタデータチャネルに集約しない。注釈文字列引数内の Unicode は、parser-assisted lexing contract が ASCII 前提診断より前に文字列必須 span を識別できるようになってから受理する。それまでは、コメント外の非 ASCII は字句前提診断のままである。
- 合成空白は第一のユーザー向けソース範囲にならない。
- `PreprocessedSource` はインクリメンタル再利用のため `source_hash` とフロントエンドバージョンでキー付けされる。
