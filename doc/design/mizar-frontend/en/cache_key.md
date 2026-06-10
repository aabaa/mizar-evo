# Module: cache_key

> Canonical language: English. Japanese companion: [../ja/cache_key.md](../ja/cache_key.md).

Status: implemented by task 19 and updated for the task-20 parser lexing plan.

## Purpose

This module exposes the frontend-owned content cache keys for the layered
outputs described in
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Incrementality".

`mizar-frontend` computes the deterministic content keys because it has the
phase inputs in hand: source identity, source hashes, lexical hashes, active
lexical-environment fingerprints, parser lexing context, and parser seam
version. The driver / artifact layer still owns cache storage, cache-hit
validation, freshness metadata, snapshot membership, and any task key that
combines these content keys with scheduler-specific identity.

## Public API

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

`FrontendOutput.cache_keys` carries one `FrontendCacheKeys` bundle. `ast` is
`None` when the configured parser seam produced no AST artifact.

## Data Structures

The key structs retain readable components and provide `stable_hash()` for
drivers that need a compact content-addressed lookup value. The hashes are
domain-separated and include the relevant per-key version string.

`SourceUnitCacheKey` follows source-version identity: package id, module path,
normalized path, source hash, and edition. Display paths, source ids, origins,
and open-buffer versions are freshness or diagnostic metadata and are excluded.

`PreprocessedSourceCacheKey` is keyed by source hash plus a frontend
preprocessing key version. Later reuse can still use `PreprocessedSource`'s
`lexical_hash` to avoid invalidating token and AST content for comment-only
edits.

`ActiveLexicalEnvironmentCacheKey` wraps the lexer-owned
`LexicalEnvironmentFingerprint`; that fingerprint already summarizes canonical
resolved imports, dependency lexical-summary fingerprints, import order, and the
active lexical shape.

`TokenStreamCacheKey` combines the lexical hash, active lexical-environment
fingerprint, current default `ParserLexContext`, and parser-assisted lexing plan
key. The task-20 plan key records the plan version, default context, and every
position-sensitive lexical byte range plus its `ParserLexContext`; changing a
string-required range or user-symbol kind filter invalidates tokenization even
when the version string is unchanged. This is a content key for the token
sequence and diagnostics, not a complete range-faithful artifact key; a driver
that reuses source-spanned tokens must compose it with source-version or
source-map identity when exact source ranges matter.

`SurfaceAstCacheKey` combines the token-stream content hash, parser seam cache
version, parser-input hash, and edition. Parser seams expose their version
through `ParserSeam::cache_key_version`. `parser_inputs_hash` includes edition,
string-required context, and explicit operator fixity entries because those
inputs can change AST shape even when the token stream is unchanged.

Stable encodings for lexer-owned non-exhaustive context enums include explicit
keys for known variants plus debug fallback text for future variants. This keeps
future lexer changes visible in cache keys until this module can add a new
intentional key version.

## Algorithm / Logic

`Frontend::run` computes cache keys after each phase has produced the needed
inputs and stores the bundle on `FrontendOutput`. The crate does not persist
cache records. A driver that caches source-spanned artifacts should compose
these content keys with its own source-version or snapshot task identity before
publishing records, because the frontend keys deliberately omit scheduler-local
ids. For comment-only edits that preserve lexical text, token and AST content
keys can stay stable even if displayed source ranges shift; source-version
composition is therefore required for caches that need exact range reuse.

## Tests

Key scenarios:

- source cache keys ignore open-buffer version and display-path freshness
  metadata but change for every content identity component;
- comment-only edits with identical lexical text change the preprocessing key
  but keep token and AST keys stable when the environment, parser context,
  parser inputs, parser version, and edition are unchanged;
- import edits and dependency lexical-environment fingerprint changes invalidate
  token keys;
- parser context / parser-assisted lexing-plan changes invalidate token keys;
- parser version, parser inputs, token stream hash, and edition changes
  invalidate AST keys;
- crate-level determinism tests verify `FrontendOutput.cache_keys` for
  comment-equivalent frontend runs and end-to-end import/dependency invalidation.
