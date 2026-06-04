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
    pub imports: Vec<ResolvedImport>,
    pub summaries: Vec<ModuleLexicalSummary>,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
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
```

`ResolvedImport`、`ModuleLexicalSummary`、`ActiveLexicalEnvironment`、`LexicalEnvironmentFingerprint`、`LexicalEnvironmentError` は `mizar-lexer` から再エクスポートされる。`FrontendLexicalEnvironmentError` は frontend が所有し、provider infrastructure failure と回復不能な lexer structural error を包む。`LexicalSummaryProvider` は、ビルドプランナー／解決器が解決済みインポートと字句サマリを供給するための継ぎ目である。フロントエンドはそれらを構築するためにモジュール IR へ踏み込まない。

## 依存関係

- 内部: `preprocess`（`ImportStub` を提供）、`lexing`（`ActiveLexicalEnvironment` とその fingerprint を消費）。
- 外部: `mizar-lexer`（`build_lexical_environment`、`ActiveLexicalEnvironment`、`ResolvedImport`、`ModuleLexicalSummary`、`UserSymbolIndex`、`ExportRank`、`LexicalEnvironmentFingerprint`、`LexicalEnvironmentError`）、`mizar-session`（`SourceId`、`Edition`）、`LexicalSummaryProvider` の背後にあるビルドプラン／解決器サービス。

このモジュールは字句解析が消費し、fingerprint を通じてインクリメンタルキャッシュも消費する。

## データ構造

### アクティブ字句環境

`ActiveLexicalEnvironment`（`mizar-lexer` 所有）は予約語・予約記号テーブル、インポートされたモジュールがエクスポートするユーザー定義記号名の `UserSymbolIndex`、および `LexicalEnvironmentFingerprint` を保持する。インポート順は決定的 fingerprint と provenance のために記録し、診断のために記号の出所を記録する。同綴りのインポート済みユーザー記号衝突はフロントエンドの import-order tie-break で解決せず、provider または `mizar-lexer` が字句環境の衝突として報告する。字句解析器ローカルであり、完全なモジュール IR や意味的適用可能性ではなく字句的な形と出所を捕捉する。

### Fingerprint とキャッシュキー

`LexicalEnvironmentFingerprint` は解決済みインポート、依存字句サマリの fingerprint、インポート順を要約する。アクティブ字句環境のキャッシュキーであり、`TokenStream` キャッシュキーの構成要素でもある。これにより、ローカルファイルが不変でも依存のエクスポート変更がトークン化を正しく無効化できる。

## アルゴリズム / ロジック

### アクティブ字句環境の構築

1. `LexicalSummaryProvider` に `ImportStub` を `ResolvedImport` と `ModuleLexicalSummary` へ解決させ、インポート順を記録する。
2. プロバイダ側の診断（未解決インポート、欠落した依存字句サマリ）を、意味的事実を作らずに集める。プロバイダは、対応する summary を持つ解決済み import だけを返す。未解決 import や summary が利用できない import は lexer 呼び出しから除外する。`mizar_lexer::build_lexical_environment` は欠落 summary を構造的エラーとして扱うためである。
3. 予約テーブル、解決済みインポート、サマリを与えて `mizar_lexer::build_lexical_environment` を呼び、`UserSymbolIndex` と `LexicalEnvironmentFingerprint` を組み立てる。
4. 決定的なユーザーシンボル import 衝突のような回復可能な import-level lexer エラーは `LexicalEnvironmentDiagnostic` へ変換し、より小さなアクティブ環境へ縮退する。安全に縮退できない provider infrastructure 失敗や不正 summary データだけを hard `FrontendLexicalEnvironmentError` として返す。
5. 環境、fingerprint、統合された診断を返す。

インポートが解決できない場合でも、解決できたインポートから環境を構築するので、ファイルの残りをトークン化できる。失敗はハード停止ではなく診断である。

## エラー処理

`LexicalEnvironmentError`（`mizar-lexer` 由来）は、衝突するサマリ fingerprint や不正なサマリデータといった構造的失敗を扱う。provider infrastructure failure は lexer 所有の enum では表現できないため、frontend は hard failure を `FrontendLexicalEnvironmentError` で包む。プロバイダ側の問題（どのモジュールにも解決しないインポート、字句サマリが利用できない依存）は `LexicalEnvironmentDiagnostic` として運び、該当 import を lexer 呼び出し前に除外するので、ファイル全体を失敗させるのではなく、より小さなアクティブ環境へ縮退できる。回復可能な lexer 側字句衝突も同様に診断として表面化し、衝突した import 記号／モジュールをアクティブ環境から除外する。インポートの合法性（可視性、字句的形を超えるエクスポートランク衝突）はモジュール解決へ先送りし、ここでは決して判断しない。

## テスト

主要シナリオ:

- インポートスタブとモジュール字句サマリが、正しい出所とインポート序数を持つ候補からなる `UserSymbolIndex` を生む。
- 異なるモジュールからインポートされた同綴りユーザー記号は決定的な字句環境衝突を生み、異なる綴りで重なり合う記号は字句解析器の最長一致選択で引き続き利用できる。
- 未解決インポートは診断とともにより小さな環境へ縮退し、残りの記号は読み込まれる。
- `LexicalEnvironmentFingerprint` は依存字句サマリが変わると変化し、ローカルファイルのコメントのみが変わるときは安定である。
- 予約語と予約記号はインポートに関係なく常に存在する。

## 制約と前提

- このモジュールはインポート解決を統制するが、解決そのものは行わない。
- 完全なインポート合法性はモジュール解決に属する。ここでは字句的な形と出所のみを捕捉する。
- アクティブ字句環境はトークン境界を変えうるので、その fingerprint は `TokenStream` キャッシュキーの一部である。
- 予約テーブルは組み込みであり、インポートと独立である。
