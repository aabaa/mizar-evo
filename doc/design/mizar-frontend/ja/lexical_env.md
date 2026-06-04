# モジュール: lexical_env

> 正本は英語です。英語版: [../en/lexical_env.md](../en/lexical_env.md)。

状態: 計画中。

## 目的

このモジュールは、フロントエンドパイプラインの Step 3（アクティブ字句環境の構築）を、フロントエンド近接の統制サービスとして実装する。前処理からの浅い `ImportStub` を、字句解析器が文脈依存の最長一致曖昧性解消に用いる `ActiveLexicalEnvironment` へ変える。

前処理出力、ビルドプランナー／モジュール解決器（インポートスタブを、解決済みインポートと軽量なモジュール字句サマリへ変える）、および `mizar_lexer::build_lexical_environment` の間を統制する。完全なインポート解決は行わない。パッケージ／モジュールの存在、可視性、エクスポートの合法性はモジュール解決に属する。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 3: Build ActiveLexicalEnvironment」「Import Pre-Scan Is Shallow」を参照。

## 公開 API

```rust
pub struct LexicalEnvironmentRequest<'a> {
    pub source_id: SourceId,
    pub import_stubs: &'a [ImportStub],
    pub edition: Edition,
}

pub trait LexicalSummaryProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError>;
}

pub struct ResolvedImports {
    pub imports: Vec<ResolvedImportEntry>,
    pub summaries: Vec<ModuleLexicalSummary>,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
}

pub struct ResolvedImportEntry {
    pub stub_ordinal: usize,
    pub stub_span: SourceRange,
    pub import: ResolvedImport,
}

pub fn build_active_lexical_environment(
    request: &LexicalEnvironmentRequest<'_>,
    provider: &dyn LexicalSummaryProvider,
) -> Result<ActiveLexicalEnvironmentResult, FrontendLexicalEnvironmentError>;

pub struct ActiveLexicalEnvironmentResult {
    pub environment: ActiveLexicalEnvironment,
    pub fingerprint: LexicalEnvironmentFingerprint,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
}

pub enum FrontendLexicalEnvironmentError {
    ProviderUnavailable { message: String },
    MalformedSummary { source: LexicalEnvironmentError },
}

pub struct LexicalEnvironmentDiagnostic {
    pub code: LexicalEnvironmentDiagnosticCode,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub import_ordinal: Option<usize>,
    pub module_id: Option<ModuleId>,
}

pub enum LexicalEnvironmentDiagnosticCode {
    UnresolvedImport,
    MissingSummary,
    UserSymbolImportConflict,
    InvalidUserSymbolSpelling,
    InvalidUserSymbolArity,
    ReservedWordCollision,
    ReservedSymbolCollision,
}
```

`ResolvedImport`、`ModuleLexicalSummary`、`ActiveLexicalEnvironment`、`LexicalEnvironmentFingerprint`、`LexicalEnvironmentError`、`ModuleId` は `mizar-lexer` から再エクスポートされる。`ResolvedImportEntry` は、フロントエンドが所有する出所情報である。字句解析器の `ResolvedImport` を、それを生んだ `ImportStub` のソーススパンと序数で包み、後続の診断が正しいインポートへ戻れるようにする。フロントエンドは `mizar_lexer::build_lexical_environment` へは、順序付きの `ResolvedImport` 値だけを渡す。

字句解析器を呼び出す前に、フロントエンドは解決済みインポートを `ModuleId` ごとに、最初の stub の順序で正規化し、module ごとに高々 1 つの `ResolvedImport` だけを字句解析器へ渡す。現在の字句解析器の `LexicalEnvironmentError::UserSymbolImportConflict` は、フロントエンドのインポート序数ではなく module id を返すため、この正規化によって衝突契約を曖昧でなく保つ。同じ module へ解決された重複 stub は、プロバイダ診断と出所テーブルでは保持できるが、重複した有効インポートとして字句解析器へは渡さない。

`FrontendLexicalEnvironmentError` は、フロントエンドが所有し、プロバイダ基盤の障害と、回復不能な字句解析器の構造的エラーを包む。フロントエンドが現在縮退して扱う字句解析器側のエラーは、`LexicalEnvironmentError::UserSymbolImportConflict` だけである。それ以外の字句解析器 `LexicalEnvironmentError` 変種は、不正なサマリ／プロバイダ契約として `FrontendLexicalEnvironmentError::MalformedSummary` になる。`LexicalSummaryProvider` は、ビルドプランナー／解決器が、解決済みインポートと字句サマリを供給するための継ぎ目である。フロントエンドは、それらを構築するためにモジュール IR へ踏み込まない。

## 依存関係

- 内部: `preprocess`（`ImportStub` を提供）、`lexing`（`ActiveLexicalEnvironment` とそのフィンガープリントを利用）。
- 外部: `mizar-lexer`（`build_lexical_environment`、`ActiveLexicalEnvironment`、`ResolvedImport`、`ModuleLexicalSummary`、`UserSymbolIndex`、`ExportRank`、`LexicalEnvironmentFingerprint`、`LexicalEnvironmentError`、`ModuleId`）、`mizar-session`（`SourceId`、`Edition`、`SourceRange`、`SourceAnchor`）、`LexicalSummaryProvider` の背後にあるビルドプラン／解決器サービス。

このモジュールは字句解析が利用し、フィンガープリントを通じてインクリメンタルキャッシュも利用する。

## データ構造

### アクティブ字句環境

`ActiveLexicalEnvironment`（`mizar-lexer` 所有）は、予約語・予約記号テーブル、インポートされたモジュールがエクスポートするユーザー定義記号名の `UserSymbolIndex`、および `LexicalEnvironmentFingerprint` を保持する。インポート順は、決定的なフィンガープリントと出所のために記録し、記号の出所は診断のために記録する。同綴りのインポート済みユーザー記号の衝突は、フロントエンドのインポート順による優先付けでは解決せず、プロバイダまたは `mizar-lexer` が字句環境の衝突として報告する。これは字句解析器ローカルであり、完全なモジュール IR や意味的な適用可能性ではなく、字句的な形と出所を捕捉する。

### フィンガープリントとキャッシュキー

`LexicalEnvironmentFingerprint` は、解決済みインポート、依存字句サマリのフィンガープリント、インポート順を要約する。アクティブ字句環境のキャッシュキーであり、`TokenStream` キャッシュキーの構成要素でもある。これにより、ローカルファイルが不変でも、依存のエクスポート変更がトークン化を正しく無効化できる。

## アルゴリズム / ロジック

### アクティブ字句環境の構築

1. `LexicalSummaryProvider` に `ImportStub` を `ResolvedImportEntry` と `ModuleLexicalSummary` へ解決させ、インポート順を記録し、診断用に元の stub スパンを保持する。
2. プロバイダ側の診断（未解決インポート、欠落した依存字句サマリ）を、意味的事実を作らずに集める。プロバイダは、対応するサマリを持つ解決済みインポートだけを返す。未解決のインポートや、サマリが利用できないインポートは、字句解析器の呼び出しから除外する。`mizar_lexer::build_lexical_environment` は、欠落したサマリを構造的エラーとして扱うためである。
3. 解決済みインポートを `ModuleId` ごとに、最初の stub の順序で正規化し、各有効 `ModuleId` から正準の `ResolvedImportEntry` への参照を保持する。それらの正準ラッパーから順序付きの字句解析器 `ResolvedImport` リストを取り出し、解決済みインポートとサマリを与えて `mizar_lexer::build_lexical_environment` を呼び、`UserSymbolIndex` と `LexicalEnvironmentFingerprint` を組み立てる。
4. 字句解析器が `UserSymbolImportConflict { spelling, earlier_import, later_import }` を返した場合、それを `LexicalEnvironmentDiagnostic` へ変換する。`later_import` に対応する正準の `ResolvedImportEntry` を primary スパンにし、`earlier_import` に対応する正準エントリがあれば副次アンカーにする。後側の衝突 module を有効インポート集合から除外して再試行する。この有界な再試行は、字句解析器が成功するか、インポートが無くなるまで繰り返す。各再試行は高々 1 つの module を除くので、決定的であり、元の正準インポート数以内に必ず停止する。どちらかの module id が正準参照に存在しない場合は、スパンを推測せず、プロバイダ／フロントエンドの不変条件違反として扱う。
5. それ以外の字句解析器 `LexicalEnvironmentError` は、回復不能な不正サマリ／プロバイダ契約として扱い、`FrontendLexicalEnvironmentError::MalformedSummary` を返す。欠落したサマリは、字句解析器の呼び出し前に診断・除外されているはずなので、ここで `MissingModuleSummary` が出るなら、それはユーザー向けの回復可能診断ではなく、プロバイダ／フロントエンドの不変条件違反である。
6. 環境、フィンガープリント、統合された診断を返す。

インポートが解決できない場合でも、解決できたインポートから環境を構築するので、ファイルの残りをトークン化できる。失敗は、ハードな停止ではなく診断である。

## エラー処理

`LexicalEnvironmentError`（`mizar-lexer` 由来）は、衝突するサマリフィンガープリントや不正なサマリデータといった構造的失敗を扱う。プロバイダ基盤の障害は、字句解析器が所有する enum では表現できないため、フロントエンドが回復不能な失敗を `FrontendLexicalEnvironmentError` で包む。プロバイダ側の問題（どのモジュールにも解決しないインポート、字句サマリが利用できない依存）は `LexicalEnvironmentDiagnostic` として運び、該当インポートを字句解析器の呼び出し前に除外するので、ファイル全体を失敗させるのではなく、より小さなアクティブ環境へ縮退できる。回復可能な字句解析器側のケースは `UserSymbolImportConflict` に限定する。フロントエンドは、後側の衝突 module を正準の出所で診断し、分かる場合は前側のインポートを副次コンテキストに加え、後側の module を除去して環境構築を再試行する。不正なエクスポート記号のつづり、不正な arity、予約語／予約記号の衝突、一貫しない重複サマリ、想定外の欠落サマリは、回復不能な不正サマリ失敗である。フロントエンドは、不正な依存サマリのどの部分集合が利用可能かを安全に推測できないためである。インポートの合法性（可視性、字句的な形を超えるエクスポートランクの衝突）はモジュール解決へ先送りし、ここでは決して判断しない。

## テスト

主要シナリオ:

- インポートスタブとモジュール字句サマリが、正しい出所とインポート序数を持つ候補からなる `UserSymbolIndex` を生む。
- 未解決インポート、欠落サマリ、ユーザー記号インポート衝突の診断が、元の正準 `ImportStub` のスパンを指す。
- 同じ module へ解決された重複インポートは、字句解析器の呼び出し前に正規化され、偽のユーザー記号インポート衝突を生まない。
- ユーザー記号インポート衝突は、後側の衝突インポートを除去して決定的に再試行し、元の正準インポートごとに高々 1 回の再試行で停止する。
- 衝突以外の字句解析器 `LexicalEnvironmentError` は、任意のサマリを黙って落とすのではなく、`FrontendLexicalEnvironmentError::MalformedSummary` になる。
- 異なるモジュールからインポートされた同綴りのユーザー記号は、決定的な字句環境衝突を生み、異なる綴りで重なり合う記号は、字句解析器の最長一致選択で引き続き利用できる。
- 未解決インポートは、診断とともに、より小さな環境へ縮退し、残りの記号は読み込まれる。
- `LexicalEnvironmentFingerprint` は、依存字句サマリが変わると変化し、ローカルファイルのコメントのみが変わるときは安定である。
- 予約語と予約記号は、インポートに関係なく常に存在する。

## 制約と前提

- このモジュールはインポート解決を統制するが、解決そのものは行わない。
- 完全なインポートの合法性はモジュール解決に属する。ここでは、字句的な形と出所のみを捕捉する。
- アクティブ字句環境はトークン境界を変えうるので、そのフィンガープリントは `TokenStream` キャッシュキーの一部である。
- 予約テーブルは組み込みであり、インポートとは独立である。
