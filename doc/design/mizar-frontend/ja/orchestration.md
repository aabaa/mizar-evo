# モジュール: orchestration

> 正本は英語です。英語版: [../en/orchestration.md](../en/orchestration.md)。

状態: 計画中。

## 目的

このモジュールは、`FrontendOutput` を生成するフェーズ 1〜3 のコーディネータ（source_and_frontend パイプラインの Step 1〜5）を実装する。`source` → `preprocess` → `lexical_env` → `lexing` → `parsing` を配線し、すべてのフェーズの診断を決定的に順序付けられた単一のリストへ統合し、統合されたフロントエンド出力を公開する。

エンドツーエンドのパイプラインを所有する唯一のモジュールである。ソース同一性、コメント除去、字句環境の組み立て、最長一致、文法、AST ノード定義は所有しない。それらは `mizar-session`、`mizar-lexer`、`mizar-syntax`、`mizar-parser` に属する。意味的な名前解決や型検査は行わない。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Frontend Pipeline」「Error Recovery」「Diagnostics」「FrontendOutput」を参照。

## 公開 API

```rust
pub struct FrontendOutput<A> {
    pub source: SourceUnit,
    pub preprocessed: PreprocessedSource,
    pub tokens: TokenStream,
    pub ast: Option<A>,
    pub diagnostics: Vec<FrontendDiagnostic>,
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

pub enum SourceLoadLocation {
    Path { path: PathBuf },
    NormalizedPath { path: NormalizedPath },
    OpenBuffer { uri: DocumentUri },
    Generated { anchor: Option<SourceAnchor> },
    Unknown,
}

pub enum DiagnosticCode {
    SourceLoad,
    Preprocess(PreprocessDiagnosticKind),
    LexicalEnvironment(LexicalEnvironmentDiagnosticCode),
    Lexing(LexingDiagnosticKind),
    Syntax(Arc<str>),
}

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

pub enum FrontendError {
    SourceLoad {
        source: SourceLoadError,
        diagnostic: FrontendDiagnostic,
    },
    SpanBridge {
        source: SpanBridgeError,
    },
    LexicalEnvironment {
        source: FrontendLexicalEnvironmentError,
    },
}
```

`FrontendOutput<A>` は、AST 型を抽象化したまま、アーキテクチャのインターフェースと一致する。`StubParserSeam` では `ast` は常に `None` であり、実 parser seam では `A` は `mizar_syntax::SurfaceAst` である。`FrontendDiagnostic` は、すべてのフェーズ固有診断（`SourcePreprocessDiagnostic`、`ImportPrescanDiagnostic`、`LexicalEnvironmentDiagnostic`、生スキャン／スコープスケルトン／字句解析器の診断を含む `LexingDiagnostic`、`SyntaxDiagnostic`）が変換される統一診断である。範囲付き診断は `DiagnosticLocation::SourceRange` を使い、`SourceId` / `LineMap` が存在する前に起きるソース読み込み失敗は、利用可能な path、正規化パス、オープンバッファ URI、生成アンカー、または `Unknown` を保持する `DiagnosticLocation::SourceLoad` を使う。`DiagnosticCode::Syntax` は、実 parser seam が有効になった後に、パーサー所有の構文診断コードキーを保持する。`StubParserSeam` では、構文診断は送出されない。

stub parser seam では、`ast = None` は期待されるプレースホルダ結果である。task 11 の実 parser seam は、回復済みトークン列に対して最小の `SurfaceAst` を返す。後続の parser recovery 作業では、構文解析が以降のフェーズに十分な構造を回復できなかった場合に `ast = None` を使うことがある。字句・前処理・構文の診断は依然として返される。

## 依存関係

- 内部: `source`、`preprocess`、`lexical_env`、`lexing`、`parsing`、`span_bridge`（一度構築され、各フェーズへ渡される）。
- 外部: 各フェーズモジュールを通じて `mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`、`NormalizedPath`、`DocumentUri`、`SessionIdAllocator`、`BuildSnapshotId`）、`mizar-lexer`、`mizar-syntax`、`mizar-parser`、および `std::path::PathBuf`。

このモジュールは crate の公開エントリポイントである。コンパイラドライバ、LSP、フォーマッター、テストが利用する。

## データ構造

### フロントエンド出力

`FrontendOutput` は、各フェーズの成果物と統合された診断を束ねる。後のフェーズ（モジュール／名前解決）が利用する単位である。後のフェーズは `ast` と `tokens` を読み、スパン・コメント・インポートスタブのために `source` / `preprocessed` を読む。各成果物は [architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Incrementality」に従って独自のキャッシュキーを持つので、コメントのみの編集は意味的出力を再利用でき、依存のエクスポート変更はトークン化を無効化する。

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
5. アクティブ字句環境とソースのエディションから `ParserInputs` を導出する。
6. 現在のパーサー字句文脈、または実 parser 契約がまだない場合はスタブ／一般文脈でトークン化する（`lexing`）。`TokenStream` を生成する。`SpanBridgeError` は `FrontendError` として伝播する。
7. 設定済みの `ParserSeam` を通じて構文解析する（`parsing`）。任意の AST を生成する。
8. 各フェーズの診断を `FrontendDiagnostic` へ変換し、上記の決定的順序で統合し、`FrontendOutput` を組み立てる。

Step 2〜5 は、回復可能な問題では中断しない。診断を記録して回復済みの成果物を前へ運ぶので、1 回の実行で、字句・トークン化・構文の診断をまとめて報告できる。

## エラー処理

`FrontendError` は、いかなる `FrontendOutput` も生成できない失敗のために予約される。主に、Step 1 のソース読み込み失敗（読み取り不能ファイル、不正 UTF-8、ルート外パス）と、内部 `SpanBridgeError` の不変条件違反である。字句環境構築からの `FrontendLexicalEnvironmentError` も、アクティブ字句環境へ安全に縮退できない場合は `FrontendError` になる。ソース読み込みエラーは、`SourceId` 割り当て、UTF-8 検証、`LineMap` 構築より前に起きることが多いため、`SourceRange` を持たなくてよい。これらは、捏造されたゼロ長ソース範囲ではなく、必ず `DiagnosticLocation::SourceLoad` で報告する。回復可能な字句前提・コメント・インポート事前走査・字句環境・スコープスケルトン・トークン化・構文の問題は、`FrontendError` ではなく、返された `FrontendOutput` 内の `FrontendDiagnostic` である。スタブの parser seam は構文診断を生成せず、`ast = None` を返す。構文診断は、実 parser seam が設定されているときだけ期待する。フロントエンド診断は、「未定義記号」や「曖昧なオーバーロード」のような意味的事実を決して主張しない。それらは後のフェーズに属する。

## テスト

主要シナリオ:

- `StubParserSeam` では、整形式のソースが source → tokens を実行し、`ast = None` かつパーサー診断なしの `FrontendOutput` を返す。
- 実 parser seam では、整形式のソースが全フェーズを実行し、`ast = Some` で診断なしの `FrontendOutput` を返す。
- 実 parser seam では、字句前提・インポート事前走査・字句環境・スコープスケルトン・トークン化・構文のエラーを持つソースが、決定的な統合順序ですべてを報告する。
- 後続の回復不能な parser recovery が、前処理とトークン化の診断を保持しつつ `ast = None` を返す。
- Step 1 の読み込み失敗が、ファイルレベルの診断を伴う `FrontendError` を返し、`FrontendOutput` を返さない。
- 診断順序が、内部スケジューリングに関係なく、繰り返し実行で同一である。
- 同じ class / start / code の診断も、完全な安定同順位決定キーで決定的に並ぶ。
- 範囲付きの統合診断が、スパン橋渡しを通じて解決された妥当な `SourceRange` を持ち、ソース読み込み失敗は、範囲を持たない `SourceLoadLocation` を持つ。

## 制約と前提

- このモジュールは統制のみを所有する。フェーズロジックは、各フェーズモジュールと上流の crate に残る。
- フロントエンドは構文を生成し、意味は生成しない。
- 返却される出力診断の統合順序は、全体で安定なキーによって決定的であり、範囲付き診断については、スパンと診断内容の同順位決定基準を持つ。
- `FrontendError` は回復不能な失敗のためのものであり、回復可能な問題は `FrontendOutput` 内の写像済み診断である。
- ソース読み込み診断はソース範囲を捏造してはならず、`DiagnosticLocation::SourceLoad` を使う。
- フロントエンド成果物は内部コンパイラデータであり、安定した公開ビルド出力ではない。
