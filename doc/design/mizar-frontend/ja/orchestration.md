# モジュール: orchestration

> 正本は英語です。英語版: [../en/orchestration.md](../en/orchestration.md)。

状態: task 20 まで実装済み。回復不能な失敗の網羅、エンドツーエンド失敗アサーション、cache-key output wiring、parser lexing-plan wiring を含む。

## 目的

このモジュールは、`FrontendOutput` を生成するフェーズ 1〜3 のコーディネータ（source_and_frontend パイプラインの Step 1〜5）を実装する。`source` → `preprocess` → `lexical_env` → `lexing` → `parsing` を配線し、すべてのフェーズの診断を決定的に順序付けられた単一のリストへ統合し、統合されたフロントエンド出力を公開する。

エンドツーエンドのパイプラインを所有する唯一のモジュールである。ソース同一性、コメント除去、字句環境の組み立て、最長一致、文法、AST ノード定義は所有しない。それらは `mizar-session`、`mizar-lexer`、`mizar-syntax`、`mizar-parser` に属する。意味的な名前解決や型検査は行わない。

[architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) の「フロントエンドパイプライン」「エラーリカバリ」「診断」「FrontendOutput」を参照。

## 公開 API

```rust
pub struct FrontendOutput<A> {
    pub source: SourceUnit,
    pub preprocessed: PreprocessedSource,
    pub tokens: TokenStream,
    pub ast: Option<A>,
    pub diagnostics: Vec<FrontendDiagnostic>,
    pub cache_keys: FrontendCacheKeys,
}

pub struct Frontend<L, P, PS>
where
    L: SourceUnitLoader,
    P: LexicalSummaryProvider,
    PS: ParserSeam,
{ /* loader, lexical-summary provider, parser seam */ }

impl<L, P, PS> Frontend<L, P, PS>
where
    L: SourceUnitLoader,
    P: LexicalSummaryProvider,
    PS: ParserSeam,
    PS::Diagnostic: FrontendParserDiagnostic,
{
    pub fn new(loader: L, provider: P, parser: PS) -> Self;

    pub fn run(
        &self,
        request: SourceUnitRequest,
        ids: &dyn SessionIdAllocator,
    ) -> Result<FrontendOutput<PS::Ast>, FrontendError>;
}

pub struct FrontendDiagnostic {
    pub code: DiagnosticCode,
    pub message: Arc<str>,
    pub class: DiagnosticClass,
    pub location: DiagnosticLocation,
    pub secondary: Vec<SourceAnchor>,
    pub recovery_note: Option<String>,
}

pub enum DiagnosticLocation {
    SourceRange(SourceRange),
    SourceLoad(SourceLoadLocation),
}

#[non_exhaustive]
pub enum SourceLoadLocation {
    Path { path: PathBuf },
    NormalizedPath { path: NormalizedPath },
    OpenBuffer { uri: DocumentUri },
    Generated { anchor: Option<SourceAnchor> },
    Unknown,
}

#[non_exhaustive]
pub enum DiagnosticCode {
    SourceLoad,
    Preprocess(PreprocessDiagnosticKind),
    LexicalEnvironment(LexicalEnvironmentDiagnosticCode),
    Lexing(LexingDiagnosticKind),
    Syntax(Arc<str>),
}

#[non_exhaustive]
pub enum DiagnosticClass {
    SourceLoad,
    LexicalPrecondition,
    CommentStructure,
    ImportPrescan,
    LexicalEnvironment,
    ScopeSkeleton,
    Tokenization,
    Syntax,
    AnnotationSyntax,
}

#[non_exhaustive]
pub enum FrontendError {
    SourceLoad {
        source: Box<SourceLoadError>,
        diagnostic: Box<FrontendDiagnostic>,
    },
    SpanBridge {
        source: SpanBridgeError,
    },
    LexicalEnvironment {
        source: FrontendLexicalEnvironmentError,
    },
}

pub trait FrontendParserDiagnostic {
    fn into_frontend_diagnostic(self) -> Option<FrontendDiagnostic>;
}
```

`FrontendOutput<A>` は、AST 型を抽象化したまま、アーキテクチャのインターフェースと一致する。`StubParserSeam` では `ast` は常に `None` であり、実 parser seam では `A` は `mizar_syntax::SurfaceAst` である。`cache_keys` は [cache_key.md](./cache_key.md) の frontend-computed content-key bundle である。driver は cache record をどう永続化し、これらの content key を snapshot / task identity とどう合成するかを決める。`FrontendDiagnostic` は、すべてのフェーズ固有診断（`SourcePreprocessDiagnostic`、`ImportPrescanDiagnostic`、`LexicalEnvironmentDiagnostic`、生スキャン／スコープスケルトン／字句解析器の診断を含む `LexingDiagnostic`、`SyntaxDiagnostic`）が変換される統一診断である。範囲付き診断は `DiagnosticLocation::SourceRange` を使い、`SourceId` / `LineMap` が存在する前に起きるソース読み込み失敗は、利用可能な path、正規化パス、オープンバッファ URI、生成アンカー、または `Unknown` を保持する `DiagnosticLocation::SourceLoad` を使う。`DiagnosticCode::Syntax` は、実 parser seam が有効になった後に、パーサー所有の構文診断コードキーを保持する。`StubParserSeam` では、構文診断は送出されない。
`FrontendParserDiagnostic` は、設定済み parser seam の診断型を統一されたフロントエンド診断ストリームへ写像するための狭いアダプタである。`mizar_syntax::SyntaxDiagnostic` と、stub seam の unit 診断型に実装される。

`SourceLoadLocation`、`DiagnosticCode`、`DiagnosticClass`、`FrontendError` は下流 crate 向けに `#[non_exhaustive]` とし、将来の source、diagnostic、回復不能 frontend surface を外部 match を壊さずに追加できるようにする。`mizar-frontend` 内部の match は引き続き exhaustive に保つ。

stub parser seam では、`ast = None` は期待されるプレースホルダ結果である。実 parser seam は、回復済みトークン列に対して最小の `SurfaceAst` を返し、構文解析が以降のフェーズに十分な構造を回復できない場合は `ast = None` を返せる。字句・前処理・構文の診断は依然として返される。

## 依存関係

- 内部: `source`、`preprocess`、`lexical_env`、`lexing`、`parsing`、`cache_key`、`span_bridge`（一度構築され、各フェーズへ渡される）。
- 外部: 各フェーズモジュールを通じて `mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`、`NormalizedPath`、`DocumentUri`、`SessionIdAllocator`、`BuildSnapshotId`）、`mizar-lexer`、`mizar-syntax`、`mizar-parser`、および `std::path::PathBuf`。

このモジュールは crate の公開エントリポイントである。コンパイラドライバ、LSP、フォーマッター、テストが利用する。

## データ構造

### フロントエンド出力

`FrontendOutput` は、各フェーズの成果物、統合された診断、frontend content cache keys を束ねる。後のフェーズ（モジュール／名前解決）が利用する単位である。後のフェーズは `ast` と `tokens` を読み、スパン・コメント・インポートスタブのために `source` / `preprocessed` を読む。各成果物の content key は [architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) の「増分処理」に従って `cache_keys` で公開されるので、コメントのみの編集は意味的出力を再利用でき、依存のエクスポート変更はトークン化を無効化する。cache storage と scheduler task keys は、この crate の外側に残る。

### 診断統合順序

範囲付き診断は、フェーズ優先度、次に全体で安定なキーで統合され、順序は実行間で安定し、内部スケジューリングとは独立である。

1. 字句前提（Step 1〜2）。
2. コメント構造（Step 2）。
3. インポート事前走査（Step 2）。
4. 字句環境（Step 3）。
5. スコープスケルトン（Step 4 の曖昧性解消前）。
6. トークン化（Step 4）。
7. 構文および注釈構文（Step 5）。

クラス内では、範囲付き診断を、`source_id`、primary スパンの開始、primary スパンの終了、診断コードの安定キー（構文コード文字列を含む）、message、副次アンカーの安定キー、回復ノートのテキスト、最後に、決定的なフェーズ出力を収集する際に割り当てるフェーズローカルな送出序数で順序付ける。ソース読み込み診断は `FrontendError` として返り、フェーズ成果物が存在しないため、返却される `FrontendOutput` の統合順には参加しない。バッチ表示などで複数のソース読み込み失敗を並べる場合は、安定なソース読み込み位置キー、次に診断コードで順序付ける。副次の `SourceAnchor` は表示・説明用に保持し、主基準ではなく同順位決定キーに含める。後の診断が先の回復に影響されうる場合は、回復ノートを付ける。

## アルゴリズム / ロジック

### 単一ソースに対するフロントエンドの実行

1. 自身の常駐ソースマップサービスを所有する、新規で可変な `SpanBridge` を構築する。
2. `SourceUnit` を読み込む（`source`）。読み込みエラー時は、`DiagnosticLocation::SourceLoad` のファイルレベル診断を運ぶ `FrontendError` を返して停止する。
3. 読み込み済みソースマップを bridge に登録し、前処理する（`preprocess`）。preprocess map を登録し、`PreprocessedSource` と Step 2 の診断を生成する。`SpanBridgeError` は `FrontendError` として伝播する。
4. インポートスタブから `ActiveLexicalEnvironment` を構築する（`lexical_env`）。回復可能なインポート／プロバイダの問題は `LexicalEnvironmentDiagnostic` とし、`FrontendLexicalEnvironmentError` は `FrontendError` とする。
5. アクティブ字句環境とソースのエディションから `ParserInputs` を導出し、前処理済み字句テキストから位置別の `ParserLexingPlan` を導出する。
6. `TokenizeRequest::with_plan` でトークン化する（`lexing`）。`TokenStream` を生成し、その plan を保持する。`SpanBridgeError` は `FrontendError` として伝播する。
7. 設定済みの `ParserSeam` を通じて構文解析する（`parsing`）。任意の AST を生成する。
8. フェーズ成果物に対する frontend content cache-key bundle を計算する。
9. 各フェーズの診断を `FrontendDiagnostic` へ変換し、上記の決定的順序で統合し、`FrontendOutput` を組み立てる。

Step 2〜5 は、回復可能な問題では中断しない。診断を記録して回復済みの成果物を前へ運ぶので、1 回の実行で、字句・トークン化・構文の診断をまとめて報告できる。

## エラー処理

`FrontendError` は、いかなる `FrontendOutput` も生成できない失敗のために予約される。主に、Step 1 のソース読み込み失敗（読み取り不能ファイル、不正 UTF-8、ルート外パス）と、内部 `SpanBridgeError` の不変条件違反である。字句環境構築からの `FrontendLexicalEnvironmentError` も、アクティブ字句環境へ安全に縮退できない場合は `FrontendError` になる。ソース読み込みエラーは、`SourceId` 割り当て、UTF-8 検証、`LineMap` 構築より前に起きることが多いため、`SourceRange` を持たなくてよい。これらは、捏造されたゼロ長ソース範囲ではなく、必ず `DiagnosticLocation::SourceLoad` で報告する。`SourceLoadLocation::NormalizedPath` と `SourceLoadLocation::Unknown` は予約 fallback location である。前者は normalized input path を持つ将来の non-exhaustive `SourceOriginInput` variant 用、後者はそれすら持たない将来の source-load 診断用である。どちらも公開され、決定的に並ぶが、現在の runtime producer はない。回復可能な字句前提・コメント・インポート事前走査・字句環境・スコープスケルトン・トークン化・構文の問題は、`FrontendError` ではなく、返された `FrontendOutput` 内の `FrontendDiagnostic` である。スタブの parser seam は構文診断を生成せず、`ast = None` を返す。構文診断は、実 parser seam が設定されているときだけ期待する。フロントエンド診断は、「未定義記号」や「曖昧なオーバーロード」のような意味的事実を決して主張しない。それらは後のフェーズに属する。

## テスト

主要シナリオ:

- `StubParserSeam` では、整形式のソースが source → tokens を実行し、`ast = None` かつパーサー診断なしの `FrontendOutput` を返す。
- 実 parser seam では、整形式のソースが全フェーズを実行し、`ast = Some` で診断なしの `FrontendOutput` を返す。
- 実 parser seam では、字句前提・インポート事前走査・字句環境・スコープスケルトン・トークン化・構文のエラーを持つソースが、決定的な統合順序ですべてを報告する。
- 後続の回復不能な parser recovery が、前処理とトークン化の診断を保持しつつ `ast = None` を返す。
- Step 1 の読み込み失敗が、ファイルレベルの診断を伴う `FrontendError` を返し、`FrontendOutput` を返さない。
- 診断順序が、内部スケジューリングに関係なく、繰り返し実行で同一である。
- 同じ class / start / code の診断も、完全な安定同順位決定キーで決定的に並ぶ。
- 予約 source-load fallback location と予約 `AnnotationSyntax` class は、現在 producer がなくてもローカルに決定的順序を持つ。
- 範囲付きの統合診断が、スパン橋渡しを通じて解決された妥当な `SourceRange` を持ち、ソース読み込み失敗は、範囲を持たない `SourceLoadLocation` を持つ。

## 制約と前提

- このモジュールは統制のみを所有する。フェーズロジックは、各フェーズモジュールと上流の crate に残る。
- フロントエンドは構文を生成し、意味は生成しない。
- 返却される出力診断の統合順序は、全体で安定なキーによって決定的であり、範囲付き診断については、スパンと診断内容の同順位決定基準を持つ。
- `FrontendError` は回復不能な失敗のためのものであり、回復可能な問題は `FrontendOutput` 内の写像済み診断である。
- ソース読み込み診断はソース範囲を捏造してはならず、`DiagnosticLocation::SourceLoad` を使う。
- フロントエンド成果物は内部コンパイラデータであり、安定した公開ビルド出力ではない。
