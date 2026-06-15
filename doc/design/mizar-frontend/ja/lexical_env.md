# モジュール: lexical_env

> 正本は英語です。英語版: [../en/lexical_env.md](../en/lexical_env.md)。

状態: タスク 6 まで実装済み。プロバイダ seam とアクティブ字句環境の回復処理は実装済みである。

## 目的

このモジュールは、フロントエンドパイプラインの Step 3（アクティブ字句環境の構築）を、フロントエンド近接の統制サービスとして実装する。前処理からの浅い `ImportStub` を、字句解析器が文脈依存の最長一致曖昧性解消に用いる `ActiveLexicalEnvironment` へ変える。

前処理出力、ビルドプランナー／モジュール解決器（インポートスタブを、解決済みインポートと軽量なモジュール字句サマリへ変える）、および `mizar_lexer::build_lexical_environment` の間を統制する。完全なインポート解決は行わない。パッケージ／モジュールの存在、可視性、エクスポートの合法性はモジュール解決に属する。

[architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) の「ステップ 3: ActiveLexicalEnvironment の構築」「インポート事前走査は浅く行う」を参照。

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
    MalformedProviderProvenance { message: String },
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

#[non_exhaustive]
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

`ActiveLexicalEnvironment`、`ExportRank`、`ExportedOperatorAssociativity`、`ExportedOperatorFixity`、`ExportedOperatorMetadata`、`ExportedSymbolShape`、`LexicalEnvironmentError`、`LexicalEnvironmentFingerprint`、`LexicalSummaryFingerprint`、`ModuleId`、`ModuleLexicalSummary`、`ResolvedImport`、`SymbolId`、`UserSymbolArity`、`UserSymbolCandidate`、`UserSymbolIndex`、`UserSymbolKind`、`UserSymbolKindSet` は `mizar-lexer` から再エクスポートされる。依存 summary と active candidate は optional な `ExportedOperatorMetadata` を運ぶ。lexer summary validation は、prefix/postfix/infix fixity に対応する exact arity を持つ symbolic functor に対してのみこれを受け付け、frontend は検証済み metadata を後で parser input へ写す。`ResolvedImportEntry` は、フロントエンドが所有する出所情報である。字句解析器の `ResolvedImport` を、それを生んだ `ImportStub` のソーススパンと序数で包み、後続の診断が正しいインポートへ戻れるようにする。フロントエンドは `mizar_lexer::build_lexical_environment` へは、順序付きの `ResolvedImport` 値だけを渡す。プロバイダが返した解決済みインポートと診断の出所情報は、利用前に request の `ImportStub` 一覧と照合する。存在しない stub 序数、古い stub スパン、別 source のスパン、別 source を指す範囲付き secondary anchor は、回復可能なインポート診断ではなく、不正なプロバイダ契約である。

`LexicalEnvironmentDiagnosticCode` は下流 crate 向けに `#[non_exhaustive]` とし、将来の provider-owned 回復可能診断を外部 match を壊さずに追加できるようにする。`mizar-frontend` 内部の match は引き続き exhaustive に保つ。

字句解析器を呼び出す前に、フロントエンドは解決済みインポートを `ModuleId` ごとに、最初の stub の順序で正規化し、module ごとに高々 1 つの `ResolvedImport` だけを字句解析器へ渡す。現在の字句解析器の `LexicalEnvironmentError::UserSymbolImportConflict` は、フロントエンドのインポート序数ではなく module id を返すため、この正規化によって衝突契約を曖昧でなく保つ。同じ module へ解決された重複 stub は、プロバイダ診断と出所テーブルでは保持できるが、重複した有効インポートとして字句解析器へは渡さない。

`FrontendLexicalEnvironmentError` は、フロントエンドが所有し、プロバイダ基盤の障害と、回復不能な字句解析器の構造的エラーを包む。`LexicalEnvironmentError::UserSymbolImportConflict` だけが、字句解析器側の回復可能なケースである。フロントエンドはこれを診断として報告し、後側の衝突インポートを除去して再試行する。それ以外の字句解析器 `LexicalEnvironmentError` 変種は、不正なサマリ／プロバイダ契約のまま扱う。`LexicalSummaryProvider` は、ビルドプランナー／解決器が、解決済みインポートと字句サマリを供給するための継ぎ目である。フロントエンドは、それらを構築するためにモジュール IR へ踏み込まない。

## 依存関係

- 内部: `preprocess`（`ImportStub` を提供）、`lexing`（`ActiveLexicalEnvironment` とそのフィンガープリントを利用）。
- 外部: `mizar-lexer`（`build_lexical_environment`、`ActiveLexicalEnvironment`、`ResolvedImport`、`ModuleLexicalSummary`、`UserSymbolIndex`、`ExportRank`、`LexicalEnvironmentFingerprint`、`LexicalEnvironmentError`、`ModuleId`）、`mizar-session`（`SourceId`、`Edition`、`SourceRange`、`SourceAnchor`）、`LexicalSummaryProvider` の背後にあるビルドプラン／解決器サービス。

このモジュールは字句解析が利用し、フィンガープリントを通じてインクリメンタルキャッシュも利用する。

## データ構造

### アクティブ字句環境

`ActiveLexicalEnvironment`（`mizar-lexer` 所有）は、予約語・予約記号テーブル、インポートされたモジュールがエクスポートするユーザー定義記号名の `UserSymbolIndex`、および `LexicalEnvironmentFingerprint` を保持する。インポート順は、決定的なフィンガープリントと出所のために記録し、記号の出所は診断のために記録する。同綴りのインポート済みユーザー記号の衝突は、フロントエンドのインポート順による優先付けでは解決せず、プロバイダまたは `mizar-lexer` が字句環境の衝突として報告する。これは字句解析器ローカルであり、完全なモジュール IR や意味的な適用可能性ではなく、字句的な形と出所を捕捉する。

### フィンガープリントとキャッシュキー

`LexicalEnvironmentFingerprint` は、解決済みインポート、依存字句サマリのフィンガープリント、インポート順、記号の形、arity、任意の operator metadata を要約する。アクティブ字句環境のキャッシュキーであり、`TokenStream` キャッシュキーの構成要素でもある。これにより、ローカルファイルが不変でも、依存のエクスポート変更や operator fixity 変更がトークン化／構文解析を正しく無効化できる。

## アルゴリズム / ロジック

### アクティブ字句環境の構築

1. `LexicalSummaryProvider` に `ImportStub` を `ResolvedImportEntry` と `ModuleLexicalSummary` へ解決させ、インポート順を記録し、診断用に元の stub スパンを保持する。各解決済みエントリの `stub_ordinal` と `stub_span`、およびプロバイダが供給した診断の primary／secondary 出所は、診断や正規化に使う前に request と照合する。
2. プロバイダ側の診断を、意味的事実を作らずに集める。解決済みエントリを持たない import stub と、依存字句サマリが利用できない正準の解決済みインポートには、フロントエンド側でも除外診断を追加する。未解決のインポートや、サマリが利用できないインポートは、字句解析器の呼び出しから除外する。`mizar_lexer::build_lexical_environment` は、欠落したサマリを構造的エラーとして扱うためである。
3. 解決済みインポートを `ModuleId` ごとに、最初の stub の順序で正規化し、各有効 `ModuleId` から正準の `ResolvedImportEntry` への参照を保持する。それらの正準ラッパーから順序付きの字句解析器 `ResolvedImport` リストを取り出し、解決済みインポートとサマリを与えて `mizar_lexer::build_lexical_environment` を呼び、`UserSymbolIndex` と `LexicalEnvironmentFingerprint` を組み立てる。
4. 字句解析器が `UserSymbolImportConflict { spelling, earlier_import, later_import }` を返した場合、それを `LexicalEnvironmentDiagnostic` へ変換する。`later_import` に対応する正準の `ResolvedImportEntry` を primary スパンにし、`earlier_import` に対応する正準エントリを副次アンカーにする。後側の衝突 module を有効インポート集合から除外して再試行する。この有界な再試行は、字句解析器が成功するか、インポートが無くなるまで繰り返す。各再試行は高々 1 つの module を除くので、決定的であり、元の正準インポート数以内に必ず停止する。どちらかの module id が正準参照に存在しない場合は、スパンを推測せず、プロバイダ／フロントエンドの不変条件違反として扱う。
5. それ以外の字句解析器 `LexicalEnvironmentError` は、回復不能な不正サマリ／プロバイダ契約として扱い、`FrontendLexicalEnvironmentError::MalformedSummary` を返す。欠落したサマリは、字句解析器の呼び出し前に診断・除外されているはずなので、ここで `MissingModuleSummary` が出るなら、それはユーザー向けの回復可能診断ではなく、プロバイダ／フロントエンドの不変条件違反である。
6. 環境、フィンガープリント、統合された診断を返す。

インポートが解決できない場合でも、解決できたインポートから環境を構築するので、ファイルの残りをトークン化できる。失敗は、ハードな停止ではなく診断である。

## エラー処理

`LexicalEnvironmentError`（`mizar-lexer` 由来）は、衝突するサマリフィンガープリントや不正なサマリデータといった構造的失敗を扱う。プロバイダ基盤の障害は、字句解析器が所有する enum では表現できないため、フロントエンドが回復不能な失敗を `FrontendLexicalEnvironmentError` で包む。プロバイダ側の問題（どのモジュールにも解決しないインポート、字句サマリが利用できない依存）は `LexicalEnvironmentDiagnostic` として運び、該当インポートを字句解析器の呼び出し前に除外するので、ファイル全体を失敗させるのではなく、より小さなアクティブ環境へ縮退できる。解決済みインポートまたはプロバイダ診断の出所が現在の request と一致しない場合は、`FrontendLexicalEnvironmentError::MalformedProviderProvenance` として報告する。それを使うと、診断を誤ったインポートや source へ結び付けてしまうためである。字句解析器側で回復可能なケースは `UserSymbolImportConflict` に限定する。フロントエンドは、後側の衝突 module を正準の出所で診断し、前側のインポートを副次コンテキストに加え、後側の module を除去して環境構築を再試行する。provider-owned の回復可能診断は、primary と secondary の出所検証を通れば任意の `LexicalEnvironmentDiagnosticCode` を利用できる。フロントエンドは、`InvalidUserSymbolSpelling`、`InvalidUserSymbolArity`、`ReservedWordCollision`、`ReservedSymbolCollision`、不正な operator metadata を lexer の summary validation から現在は合成しない。lexer-owned の不正な依存サマリは、引き続き `FrontendLexicalEnvironmentError::MalformedSummary` になる。不正なエクスポート記号のつづり、不正な arity、不正な operator metadata、予約語／予約記号の衝突、一貫しない重複サマリ、想定外の欠落サマリは、回復不能な不正サマリ失敗である。フロントエンドは、不正な依存サマリのどの部分集合が利用可能かを安全に推測できないためである。インポートの合法性（可視性、字句的な形を超えるエクスポートランクの衝突）はモジュール解決へ先送りし、ここでは決して判断しない。

## テスト

主要シナリオ:

- インポートスタブとモジュール字句サマリが、正しい出所とインポート序数を持つ候補からなる `UserSymbolIndex` を生む。
- 未解決インポート、欠落サマリ、ユーザー記号インポート衝突の診断が、元の正準 `ImportStub` のスパンを指す。
- プロバイダが返した解決済みインポートまたは診断の出所に、存在しない stub 序数、古いスパン、別 source が含まれる場合は、ハードなプロバイダ契約違反として拒否する。
- 同じ module へ解決された重複インポートは、字句解析器の呼び出し前に正規化され、偽のユーザー記号インポート衝突を生まない。
- ユーザー記号インポート衝突は、後側の衝突インポートを除去して決定的に再試行し、元の正準インポートごとに高々 1 回の再試行で停止する。
- 衝突以外の字句解析器 `LexicalEnvironmentError` は、任意のサマリを黙って落とすのではなく、`FrontendLexicalEnvironmentError::MalformedSummary` になる。
- 異なるモジュールからインポートされた同綴りのユーザー記号は、決定的な字句環境衝突を生み、異なる綴りで重なり合う記号は、字句解析器の最長一致選択で引き続き利用できる。
- 未解決インポートは、診断とともに、より小さな環境へ縮退し、残りの記号は読み込まれる。
- provider-owned の予約診断 code は出所検証後に pass-through し、lexer-owned の不正サマリは hard failure のまま残る。
- `LexicalEnvironmentFingerprint` は、依存字句サマリが変わると変化し、ローカルファイルのコメントのみが変わるときは安定である。
- 予約語と予約記号は、インポートに関係なく常に存在する。

## 制約と前提

- このモジュールはインポート解決を統制するが、解決そのものは行わない。
- 完全なインポートの合法性はモジュール解決に属する。ここでは、字句的な形と出所のみを捕捉する。
- アクティブ字句環境はトークン境界を変えうるので、そのフィンガープリントは `TokenStream` キャッシュキーの一部である。
- 予約テーブルは組み込みであり、インポートとは独立である。
- アクティブ字句環境は、インポートされたモジュールの圧縮された `ModuleLexicalSummary` 射影のみを保持し、その定義や完全なモジュール IR は決して保持しない（フロントエンドはモジュール IR に踏み込まない——「目的」「アクティブ字句環境」を参照）。`LexicalSummaryProvider` は、これらのサマリを import closure 全体ではなく現在のファイルの解決済みインポートについて供給する。これは、常駐集合メモリモデルの「本体ではなくインターフェイスを保持する」規則の、フロントエンドのシームにおける表現である（spec [§12.6.3](../../../spec/ja/12.modules_and_namespaces.md#1263-メモリモデル)、[mizar-lexer の lexical_environment.md](../../mizar-lexer/ja/lexical_environment.md) も参照）。
