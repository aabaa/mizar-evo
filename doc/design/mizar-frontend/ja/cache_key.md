# モジュール: cache_key

> 正本は英語です。英語版: [../en/cache_key.md](../en/cache_key.md)。

状態: task 19 で実装済み。task 20 の parser lexing plan に合わせて更新済み。

## 目的

このモジュールは、
[architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md)
「増分処理」で定義された層状の出力に対する、フロントエンド所有の内容キャッシュキーを公開する。

`mizar-frontend` は、ソース同一性、ソースハッシュ、字句ハッシュ、アクティブ字句環境 fingerprint、parser lexing context、parser seam version といった phase 入力を手元に持つため、決定的な内容キーを計算する。cache storage、cache hit 検証、freshness metadata、snapshot membership、およびこれらの内容キーを scheduler 固有の同一性と合成した task key は、引き続き driver / artifact 層が所有する。

## 公開 API

```rust
pub const SOURCE_UNIT_CACHE_KEY_VERSION: &str;
pub const PREPROCESSED_SOURCE_CACHE_KEY_VERSION: &str;
pub const ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION: &str;
pub const PARSER_LEXING_PLAN_CACHE_KEY_VERSION: &str;
pub const TOKEN_STREAM_CACHE_KEY_VERSION: &str;
pub const SURFACE_AST_CACHE_KEY_VERSION: &str;

pub struct FrontendCacheKeys {
    pub source: SourceUnitCacheKey,
    pub preprocessed: PreprocessedSourceCacheKey,
    pub active_lexical_environment: ActiveLexicalEnvironmentCacheKey,
    pub tokens: TokenStreamCacheKey,
    pub ast: Option<SurfaceAstCacheKey>,
}

pub struct SourceUnitCacheKey {
    pub version: Arc<str>,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub source_hash: Hash,
    pub edition: Edition,
}

impl SourceUnitCacheKey {
    pub fn from_source(source: &SourceUnit) -> Self;
    pub fn stable_hash(&self) -> Hash;
}

pub struct PreprocessedSourceCacheKey {
    pub version: Arc<str>,
    pub source_hash: Hash,
}

impl PreprocessedSourceCacheKey {
    pub fn from_source(source: &SourceUnit) -> Self;
    pub fn stable_hash(&self) -> Hash;
}

pub struct ActiveLexicalEnvironmentCacheKey {
    pub version: Arc<str>,
    pub fingerprint: LexicalEnvironmentFingerprint,
}

impl ActiveLexicalEnvironmentCacheKey {
    pub fn new(fingerprint: LexicalEnvironmentFingerprint) -> Self;
    pub fn stable_hash(&self) -> Hash;
}

pub struct ParserLexingPlanCacheKey {
    pub version: Arc<str>,
    pub default_context: ParserLexContext,
    pub contexts: Vec<ParserLexingPlanContextCacheKey>,
}

pub struct ParserLexingPlanContextCacheKey {
    pub range: LexicalByteRange,
    pub context: ParserLexContext,
}

impl ParserLexingPlanCacheKey {
    pub fn current() -> Self;
    pub fn from_plan(plan: &ParserLexingPlan) -> Self;
}

pub struct TokenStreamCacheKey {
    pub version: Arc<str>,
    pub lexical_hash: Hash,
    pub active_lexical_environment: LexicalEnvironmentFingerprint,
    pub parser_context: ParserLexContext,
    pub parser_lexing_plan: ParserLexingPlanCacheKey,
}

impl TokenStreamCacheKey {
    pub fn new(
        preprocessed: &PreprocessedSource,
        active_lexical_environment: LexicalEnvironmentFingerprint,
        parser_context: ParserLexContext,
        parser_lexing_plan: ParserLexingPlanCacheKey,
    ) -> Self;
    pub fn stable_hash(&self) -> Hash;
}

pub struct SurfaceAstCacheKey {
    pub version: Arc<str>,
    pub token_stream_hash: Hash,
    pub parser_version: ParserCacheKeyVersion,
    pub parser_inputs_hash: Hash,
    pub edition: Edition,
}

impl SurfaceAstCacheKey {
    pub fn new(
        token_stream_hash: Hash,
        parser_version: ParserCacheKeyVersion,
        parser_inputs: &ParserInputs,
    ) -> Self;
    pub fn stable_hash(&self) -> Hash;
}

pub fn parser_inputs_hash(inputs: &ParserInputs) -> Hash;
```

`FrontendOutput.cache_keys` は `FrontendCacheKeys` の束を 1 つ保持する。設定された parser seam が AST artifact を生成しなかった場合、`ast` は `None` である。

## データ構造

キー構造体は可読な構成要素を保持し、driver がコンパクトな content-addressed lookup 値を必要とする場合のために `stable_hash()` を提供する。ハッシュは domain-separated であり、関連する key version 文字列を含む。

`SourceUnitCacheKey` は source-version identity に従い、package id、module path、normalized path、source hash、edition を含む。表示 path、source id、origin、open-buffer version は freshness または診断 metadata であり、除外される。

`PreprocessedSourceCacheKey` は source hash と frontend preprocessing key version で決まる。後続の再利用では、`PreprocessedSource` の `lexical_hash` により、コメントのみの編集で token と AST の内容を無効化せずに済ませられる。

`ActiveLexicalEnvironmentCacheKey` は lexer 所有の `LexicalEnvironmentFingerprint` を包む。この fingerprint は、canonical resolved imports、依存字句サマリ fingerprint、import order、アクティブな字句形状をすでに要約している。

`TokenStreamCacheKey` は lexical hash、アクティブ字句環境 fingerprint、現在の default `ParserLexContext`、parser-assisted lexing plan key を組み合わせる。task 20 の plan key は、plan version、default context、位置別の各 lexical byte range とその `ParserLexContext` を記録する。string-required range や user-symbol kind filter が変わると、version string が同じでも tokenization は無効化される。これは token sequence と diagnostics の content key であり、range-faithful artifact key 全体ではない。source-spanned token を再利用する driver は、正確な source range が重要な場合、source-version または source-map identity と合成する必要がある。

`SurfaceAstCacheKey` は token-stream content hash、parser seam cache version、parser-input hash、edition を組み合わせる。Parser seam は `ParserSeam::cache_key_version` により version を公開する。`parser_inputs_hash` は、token stream が不変でも AST shape を変え得るため、edition、string-required context、明示的な operator fixity entries を含む。

lexer 所有の non-exhaustive context enum の安定 encoding は、既知 variant の明示 key と、将来 variant 用の debug fallback text を含む。これにより、将来の lexer 変更は、このモジュールが意図的な新 key version を追加するまで cache key に反映される。

## アルゴリズム / ロジック

`Frontend::run` は、各 phase が必要な入力を生成した後で cache key を計算し、束を `FrontendOutput` に保存する。この crate は cache record を永続化しない。source-spanned artifact を cache する driver は、frontend key が scheduler-local id を意図的に除外しているため、record を公開する前にこれらの内容 key を自前の source-version または snapshot task identity と合成する必要がある。lexical text を保つコメントのみの編集では、表示 source range がずれても token と AST の content key は安定し得る。そのため、正確な range reuse が必要な cache では source-version composition が必須である。

## テスト

主要シナリオ:

- source cache key が open-buffer version と表示 path の freshness metadata を無視するが、各 content identity component では変化する。
- 同一の lexical text を持つコメントのみの編集は preprocessing key を変えるが、environment、parser context、parser inputs、parser version、edition が不変であれば token key と AST key は安定する。
- import edit と依存字句環境 fingerprint の変更は token key を無効化する。
- parser context / parser-assisted lexing plan の変更は token key を無効化する。
- parser version、parser inputs、token stream hash、edition の変更は AST key を無効化する。
- crate-level determinism tests が、comment-equivalent な frontend 実行、および end-to-end import/dependency invalidation について `FrontendOutput.cache_keys` を検証する。
