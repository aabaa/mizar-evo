# モジュール: lexical_env

> 正本は英語です。英語版: [../en/lexical_env.md](../en/lexical_env.md)。

状態: planned。

## 目的

このモジュールはフロントエンドパイプラインの Step 3（アクティブ字句環境構築）をフロントエンド近接の統制サービスとして実装する。前処理からの浅い `ImportStub` を、字句解析器が文脈依存の最長一致曖昧性解消に用いる `ActiveLexicalEnvironment` へ変える。

前処理出力、ビルドプランナー／モジュール解決器（インポートスタブを解決済みインポートと軽量なモジュール字句サマリへ変える）、および `mizar_lexer::build_lexical_environment` の間を統制する。完全なインポート解決は行わない。パッケージ／モジュールの存在、可視性、エクスポート合法性はモジュール解決に属する。

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

`ResolvedImport`、`ModuleLexicalSummary`、`ActiveLexicalEnvironment`、`LexicalEnvironmentFingerprint`、`LexicalEnvironmentError`、`ModuleId` は `mizar-lexer` から再エクスポートされる。`ResolvedImportEntry` は frontend が所有する provenance である。lexer の `ResolvedImport` を、それを生んだ `ImportStub` の source span と ordinal で包み、複数の stub が同じ module へ解決されても後続診断が正しい import へ戻れるようにする。frontend は `mizar_lexer::build_lexical_environment` へは順序付きの `ResolvedImport` 値だけを渡す。

`FrontendLexicalEnvironmentError` は frontend が所有し、provider infrastructure failure と回復不能な lexer structural error を包む。frontend が現在縮退する lexer-side error は `LexicalEnvironmentError::UserSymbolImportConflict` だけである。それ以外の lexer `LexicalEnvironmentError` variant は、不正 summary / provider contract として `FrontendLexicalEnvironmentError::MalformedSummary` になる。`LexicalSummaryProvider` は、ビルドプランナー／解決器が解決済みインポートと字句サマリを供給するための継ぎ目である。フロントエンドはそれらを構築するためにモジュール IR へ踏み込まない。

## 依存関係

- 内部: `preprocess`（`ImportStub` を提供）、`lexing`（`ActiveLexicalEnvironment` とその fingerprint を消費）。
- 外部: `mizar-lexer`（`build_lexical_environment`、`ActiveLexicalEnvironment`、`ResolvedImport`、`ModuleLexicalSummary`、`UserSymbolIndex`、`ExportRank`、`LexicalEnvironmentFingerprint`、`LexicalEnvironmentError`、`ModuleId`）、`mizar-session`（`SourceId`、`Edition`、`SourceRange`、`SourceAnchor`）、`LexicalSummaryProvider` の背後にあるビルドプラン／解決器サービス。

このモジュールは字句解析が消費し、fingerprint を通じてインクリメンタルキャッシュも消費する。

## データ構造

### アクティブ字句環境

`ActiveLexicalEnvironment`（`mizar-lexer` 所有）は予約語・予約記号テーブル、インポートされたモジュールがエクスポートするユーザー定義記号名の `UserSymbolIndex`、および `LexicalEnvironmentFingerprint` を保持する。インポート順は決定的 fingerprint と provenance のために記録し、診断のために記号の出所を記録する。同綴りのインポート済みユーザー記号衝突はフロントエンドの import-order tie-break で解決せず、provider または `mizar-lexer` が字句環境の衝突として報告する。字句解析器ローカルであり、完全なモジュール IR や意味的適用可能性ではなく字句的な形と出所を捕捉する。

### Fingerprint とキャッシュキー

`LexicalEnvironmentFingerprint` は解決済みインポート、依存字句サマリの fingerprint、インポート順を要約する。アクティブ字句環境のキャッシュキーであり、`TokenStream` キャッシュキーの構成要素でもある。これにより、ローカルファイルが不変でも依存のエクスポート変更がトークン化を正しく無効化できる。

## アルゴリズム / ロジック

### アクティブ字句環境の構築

1. `LexicalSummaryProvider` に `ImportStub` を `ResolvedImportEntry` と `ModuleLexicalSummary` へ解決させ、インポート順を記録し、診断用に元の stub span を保持する。
2. プロバイダ側の診断（未解決インポート、欠落した依存字句サマリ）を、意味的事実を作らずに集める。プロバイダは、対応する summary を持つ解決済み import だけを返す。未解決 import や summary が利用できない import は lexer 呼び出しから除外する。`mizar_lexer::build_lexical_environment` は欠落 summary を構造的エラーとして扱うためである。
3. `ResolvedImportEntry` wrapper から順序付き lexer `ResolvedImport` list を取り出し、解決済みインポートとサマリを与えて `mizar_lexer::build_lexical_environment` を呼び、`UserSymbolIndex` と `LexicalEnvironmentFingerprint` を組み立てる。
4. lexer が `UserSymbolImportConflict { spelling, earlier_import, later_import }` を返した場合、それを `LexicalEnvironmentDiagnostic` へ変換する。`later_import` に対応する `ResolvedImportEntry` を primary span にし、`earlier_import` に対応する entry があれば secondary anchor にする。後側の衝突 import entry を active import set から除外して再試行する。この bounded retry は lexer が成功するか import が無くなるまで繰り返す。各 retry は高々 1 つの import を除くので、決定的であり、元の import 数以内に必ず停止する。
5. それ以外の lexer `LexicalEnvironmentError` は回復不能な不正 summary / provider contract として扱い、`FrontendLexicalEnvironmentError::MalformedSummary` を返す。欠落 summary は lexer 呼び出し前に診断・除外されているべきなので、ここで `MissingModuleSummary` が出るなら user-facing な回復可能診断ではなく provider/frontend の不変条件違反である。
6. 環境、fingerprint、統合された診断を返す。

インポートが解決できない場合でも、解決できたインポートから環境を構築するので、ファイルの残りをトークン化できる。失敗はハード停止ではなく診断である。

## エラー処理

`LexicalEnvironmentError`（`mizar-lexer` 由来）は、衝突するサマリ fingerprint や不正なサマリデータといった構造的失敗を扱う。provider infrastructure failure は lexer 所有の enum では表現できないため、frontend は hard failure を `FrontendLexicalEnvironmentError` で包む。プロバイダ側の問題（どのモジュールにも解決しないインポート、字句サマリが利用できない依存）は `LexicalEnvironmentDiagnostic` として運び、該当 import を lexer 呼び出し前に除外するので、ファイル全体を失敗させるのではなく、より小さなアクティブ環境へ縮退できる。回復可能な lexer 側ケースは `UserSymbolImportConflict` に限定する。frontend は後側の衝突 import を診断し、分かる場合は前側 import を secondary context に追加し、後側 import を除去して環境構築を再試行する。不正な exported symbol spelling、不正 arity、reserved word / symbol collision、一貫しない重複 summary、想定外の欠落 summary は hard な malformed-summary failure である。frontend は不正な依存 summary のどの subset が利用可能かを安全に推測できないためである。インポートの合法性（可視性、字句的形を超えるエクスポートランク衝突）はモジュール解決へ先送りし、ここでは決して判断しない。

## テスト

主要シナリオ:

- インポートスタブとモジュール字句サマリが、正しい出所とインポート序数を持つ候補からなる `UserSymbolIndex` を生む。
- 未解決 import、欠落 summary、user-symbol import conflict の診断が、複数 stub が同じ module へ解決される場合でも元の `ImportStub` span を指す。
- user-symbol import conflict は後側の衝突 import を除去して決定的に retry し、元の import ごとに高々 1 回の retry で停止する。
- conflict 以外の lexer `LexicalEnvironmentError` は、任意の summary を黙って落とすのではなく `FrontendLexicalEnvironmentError::MalformedSummary` になる。
- 異なるモジュールからインポートされた同綴りユーザー記号は決定的な字句環境衝突を生み、異なる綴りで重なり合う記号は字句解析器の最長一致選択で引き続き利用できる。
- 未解決インポートは診断とともにより小さな環境へ縮退し、残りの記号は読み込まれる。
- `LexicalEnvironmentFingerprint` は依存字句サマリが変わると変化し、ローカルファイルのコメントのみが変わるときは安定である。
- 予約語と予約記号はインポートに関係なく常に存在する。

## 制約と前提

- このモジュールはインポート解決を統制するが、解決そのものは行わない。
- 完全なインポート合法性はモジュール解決に属する。ここでは字句的な形と出所のみを捕捉する。
- アクティブ字句環境はトークン境界を変えうるので、その fingerprint は `TokenStream` キャッシュキーの一部である。
- 予約テーブルは組み込みであり、インポートと独立である。
