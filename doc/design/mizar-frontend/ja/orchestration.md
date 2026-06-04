# モジュール: orchestration

> 正本は英語です。英語版: [../en/orchestration.md](../en/orchestration.md)。

状態: planned。

## 目的

このモジュールは `FrontendOutput` を生成するフェーズ 1-3 の coordinator（source_and_frontend パイプラインの Step 1-5）を実装する。`source` → `preprocess` → `lexical_env` → `lexing` → `parsing` を配線し、すべてのフェーズの診断を決定的に順序付けられた単一のリストへ統合し、統合されたフロントエンド出力を公開する。

エンドツーエンドのパイプラインを所有する唯一のモジュールである。ソース同一性、コメント除去、字句環境組み立て、最長一致、文法、AST ノード定義は所有しない。それらは `mizar-session`、`mizar-lexer`、`mizar-syntax`、`mizar-parser` に属する。意味的な名前解決や型検査は行わない。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Frontend Pipeline」「Error Recovery」「Diagnostics」「FrontendOutput」を参照。

## 公開 API

```rust
pub struct FrontendOutput {
    pub source: SourceUnit,
    pub preprocessed: PreprocessedSource,
    pub tokens: TokenStream,
    pub ast: Option<SurfaceAst>,
    pub diagnostics: Vec<FrontendDiagnostic>,
}

pub struct Frontend<L, P, S>
where
    L: SourceUnitLoader,
    P: LexicalSummaryProvider,
    S: SourceMapService,
{ /* loader, lexical-summary provider, source-map service */ }

impl<L, P, S> Frontend<L, P, S> {
    pub fn new(loader: L, provider: P, service: Arc<S>) -> Self;

    pub fn run(
        &self,
        request: SourceUnitRequest,
        parser_inputs: ParserInputs,
        ids: &dyn SessionIdAllocator,
    ) -> Result<FrontendOutput, FrontendError>;
}

pub struct FrontendDiagnostic {
    pub code: DiagnosticCode,
    pub class: DiagnosticClass,
    pub primary: SourceRange,
    pub secondary: Vec<SourceRange>,
    pub recovery_note: Option<String>,
}

pub enum DiagnosticClass {
    LexicalPrecondition,
    CommentStructure,
    Tokenization,
    Syntax,
    AnnotationSyntax,
}
```

`FrontendOutput` はアーキテクチャのインターフェースと一致する。`FrontendDiagnostic` は、すべてのフェーズ固有診断（`SourcePreprocessDiagnostic`、`ImportPrescanDiagnostic`、`LexicalEnvironmentDiagnostic`、`LexDiagnostic`、`SyntaxDiagnostic`）が変換される統一診断であり、消費者は `SourceRange` でキー付けされた単一の順序付きリストを見る。

`ast = None` は、構文解析が以降のフェーズに十分な構造を回復できなかったことを意味する。字句・前処理・構文の診断は依然として返される。

## 依存関係

- 内部: `source`、`preprocess`、`lexical_env`、`lexing`、`parsing`、`span_bridge`（一度構築され各フェーズへ通される）。
- 外部: 各フェーズモジュールを通じて `mizar-session`（`SourceId`、`SourceRange`、`SourceMapService`、`SessionIdAllocator`、`BuildSnapshotId`）、`mizar-lexer`、`mizar-syntax`、`mizar-parser`。

このモジュールは crate の公開エントリポイントである。コンパイラドライバ、LSP、フォーマッター、テストが消費する。

## データ構造

### フロントエンド出力

`FrontendOutput` は各フェーズの成果物と統合された診断を束ねる。後のフェーズ（モジュール／名前解決）が消費する単位である。それらは `ast` と `tokens` を読み、スパン・コメント・インポートスタブのために `source` / `preprocessed` を読む。各成果物は [architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Incrementality」に従って独自のキャッシュキーを持つので、コメントのみの編集は意味的出力を再利用でき、依存のエクスポート変更はトークン化を無効化する。

### 診断統合順序

診断はフェーズ優先度、次に第一 `SourceRange` の開始で統合され、順序は実行間で安定し内部スケジューリングと独立である。

1. 字句前提（Step 1-2）。
2. コメント構造（Step 2）。
3. トークン化（Step 4）。
4. 構文および注釈構文（Step 5）。

クラス内では第一スパンの開始、次に診断コードで順序付ける。後の診断が先の回復に影響されうる場合は回復ノートを付ける。

## アルゴリズム / ロジック

### 単一ソースに対するフロントエンドの実行

1. `SourceMapService` 上に `SpanBridge` を構築する。
2. `SourceUnit` を読み込む（`source`）。読み込みエラー時はファイルレベルの診断を運ぶ `FrontendError` を返して停止する。
3. 前処理する（`preprocess`）。`PreprocessedSource` と Step 2 の診断を生成する。
4. インポートスタブから `ActiveLexicalEnvironment` を構築する（`lexical_env`）。
5. トークン化する（`lexing`）。`TokenStream` を生成する。
6. 構文解析する（`parsing`）。省略可能な `SurfaceAst` を生成する。
7. 各フェーズの診断を `FrontendDiagnostic` へ変換し、上記の決定的順序で統合し、`FrontendOutput` を組み立てる。

Step 2-5 は回復可能な問題で中断しない。診断を記録して回復済み成果物を前へ運ぶので、1 回の実行で字句・トークン化・構文の診断をまとめて報告できる。

## エラー処理

`FrontendError` は、いかなる `FrontendOutput` も生成できない失敗のために予約される。主に Step 1 のソース読み込み失敗（読み取り不能ファイル、不正 UTF-8、ルート外パス）と内部 `SpanBridgeError` 不変条件違反である。回復可能な字句・コメント・トークン化・構文の問題は `FrontendError` ではなく、返された `FrontendOutput` 内の `FrontendDiagnostic` である。フロントエンド診断は「未定義記号」や「曖昧なオーバーロード」のような意味的事実を決して主張しない。それらは後のフェーズに属する。

## テスト

主要シナリオ:

- 整形式のソースが全フェーズを実行し、`ast = Some` で診断なしの `FrontendOutput` を返す。
- 字句前提エラー・トークン化エラー・構文エラーを持つソースが、決定的統合順序で 3 つすべてを報告する。
- 構文解析の失敗が、前処理とトークン化の診断を保持しつつ `ast = None` を返す。
- Step 1 の読み込み失敗が、ファイルレベルの診断を伴う `FrontendError` を返し、`FrontendOutput` を返さない。
- 診断順序が、内部スケジューリングに関係なく繰り返し実行で同一である。
- 統合された診断が、span 橋渡しを通じて解決された妥当な `SourceRange` を持つ。

## 制約と前提

- このモジュールは統制のみを所有する。フェーズロジックは各フェーズモジュールと上流 crate に残る。
- フロントエンドは構文を生成し、意味は生成しない。
- 診断統合順序は決定的でスパンキー付けである。
- `FrontendError` は回復不能な失敗のためのものであり、回復可能な問題は `FrontendOutput` 内の診断である。
- フロントエンド成果物は内部コンパイラデータであり、安定した公開ビルド出力ではない。
