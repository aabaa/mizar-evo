# ソース／仕様対応監査

> 正本は英語です。英語版: [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

状態: task 20 まで完了。

## 範囲

この監査は、task 20 後の `mizar-frontend` 実装を、まず英語正本の
`doc/design/mizar-frontend/en/` 仕様に照合し、その後で日本語 companion
仕様が同じ公開 API 名、エラー／診断 variant、挙動の約束を保持している
ことを確認する。

これは軽量なソース／仕様／テスト対応表であり、release coverage gate でも、
実行可能テストの代替でもない。欠落した実装、古くなった仕様文、欠落した
テストが見つかった場合は、この監査に広範な変更を混ぜず、follow-up task
として記録する。

## 結果

- task 1-20 が約束する公開 API とエラー／診断 variant について、欠落した
  実装は見つからなかった。
- task 2 の source 要件文は、監査前に追加済みだった open-buffer `file://`
  診断パスの decode/fallback テストを明示するよう更新した。
- task 1-20 について、英語正本仕様に残る古い記述は見つからなかった。
- 日本語 companion 仕様は、API 名と挙動の約束が英語正本と一致することを
  確認した。API または挙動の drift は残っていない。
- より広い bilingual wording/terminology review は task 17 で完了し、
  [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
  に記録した。task 19 の incremental cache-key wiring と task 20 の
  parser-assisted lexing は現在完了済みである。他の延期された実装または
  coverage 作業は task 21-24 に残る。

## 公開 API 対応

| 仕様 | 確認した公開 API | ソース | テスト証跡 |
|---|---|---|---|
| [span_bridge.md](./span_bridge.md) | `SpanBridge`, `LexerByteSpan`, `SpanBridgeError`; `SpanBridge::{new, source_map_service, register_source, register_preprocess_map, loaded_span, loaded_mapping, lexical_span}`; crate 可視 `whole_lexical_text_mapping` | `crates/mizar-frontend/src/span_bridge.rs` | inline tests が loaded/lexical mapping、loading-map identity behavior、composite/degraded preprocess mapping、whole-lexical-text recovery mapping、不正 span、未登録、登録衝突を確認する。unsupported lexer preprocess metadata は防御的 guard で表現されるが、現在の `mizar-lexer` からは未対応 map variant を構築できない。 |
| [source.md](./source.md) | `SourceUnit`, `SourceUnitRequest`, `SourceUnitLoader`, `SourceUnitLoader::load_source_unit`, `FrontendSourceLoader`, `FrontendSourceLoader::new`, `source_unit_from_loaded`, `register_source_unit` | `crates/mizar-frontend/src/source.rs` | inline tests が再計算しない projection、loader forwarding、BOM/CRLF loading map、identity load、open-buffer origin/version、open-buffer `file://` path decode/fallback、generated source、bridge registration、`SourceLoadError` の非再分類伝播を確認する。 |
| [preprocess.md](./preprocess.md) | `PreprocessedSource`, `LexicalText`, `LexicalText::as_str`, `Comment`, `DocComment`, `LexicalSourceMap`, `LexicalSourceMap::{lexical_span, lexical_len, is_empty}`, `ImportStub`, `ImportStubPath`, `ImportStubRelativePrefix::{Current, Parent}`, `ImportStubAlias`, `PreprocessDiagnostic`, `PreprocessDiagnosticKind`, `preprocess`, `lexical_hash`; 再エクスポート `ImportPrescanDiagnosticCode`, `SourcePreprocessDiagnosticCode`, `CommentKind` | `crates/mizar-frontend/src/preprocess.rs` | inline tests が comment/doc-comment separation、annotation preservation、import stub と relative prefix、malformed import recovery、coarse raw import scan recovery、mapped diagnostics、composite/degraded lexical mappings、stable lexical hash、non-ASCII code diagnostics、unterminated block comment を確認する。 |
| [lexical_env.md](./lexical_env.md) | `LexicalEnvironmentRequest`, `LexicalSummaryProvider`, `LexicalSummaryProvider::resolve_imports`, `ResolvedImports`, `ResolvedImportEntry`, `ActiveLexicalEnvironmentResult`, `FrontendLexicalEnvironmentError`, `LexicalEnvironmentDiagnostic`, `LexicalEnvironmentDiagnosticCode`, `build_active_lexical_environment`; 再エクスポートされた lexer environment 型 | `crates/mizar-frontend/src/lexical_env.rs` | inline tests が provider seam output、import deduplication、reserved tables、provider infrastructure failure、provider provenance hard failure、unresolved import、missing summary、conflict retry、non-conflict malformed summary、fingerprint stability/change behavior を確認する。 |
| [lexing.md](./lexing.md) | `InternedText`, `TokenizeRequest`, `TokenizeRequest::{new, with_plan}`, `ParserLexingPlan`, `ParserLexingPlan::{uniform, new, for_lexical_text, context_at, is_uniform}`, `ParserLexingPlanContext`, `ParserLexingPlanContext::new`, `LexicalByteRange`, `LexicalByteRange::{new, contains}`, `TokenStream`, `TokenStream::{tokens, diagnostics, scope_view, into_parts}`, `Token`, `ScopeView`, `ScopeView::{empty, binding_overrides_symbol}`, `ScopeFrame`, `ScopedBinding`, `ScopeBlock`, `ScopeStatement`, `LexingDiagnostic`, `LexingDiagnosticKind`, `LexingDiagnosticPayload`, `LexingRejectedTokenCandidate`, `tokenize`; 再エクスポートされた lexer token/context/scope enum | `crates/mizar-frontend/src/lexing.rs` | inline tests が raw-span preservation、preprocess mapping、longest-match user symbols、scoped identifier overrides、compound reserved tokens、parser context/string behavior、Unicode/comment-marker 内容を持つ位置別 annotation string argument、planned string range の line-boundary rejection、範囲別 user-symbol kind filter、payload mapping、recoverable error tokens、unsupported raw-token recovery、rejected candidates、secondary anchors、scope view contents、scope diagnostics、coarse raw scan recovery を確認する。 |
| [parsing.md](./parsing.md) | `DEFAULT_PARSER_CACHE_KEY_VERSION`, `STUB_PARSER_CACHE_KEY_VERSION`, `MIZAR_PARSER_CACHE_KEY_VERSION`, `ParseRequest`, `ParseRequest::new`, `ParserInputs`, `ParserInputs::{new, from_active_environment}`, `OperatorFixityTable`, `OperatorFixityTable::{empty, is_empty}`, `OperatorFixityEntry`, `OperatorAssociativity::{Left, Right, NonAssociative}`, `StringRequiredContext::{None, PositionSensitive, UniformForTest}`, `StringRequiredContext::{parser_lex_context, parser_lexing_plan}`, `ParserCacheKeyVersion`, `ParserCacheKeyVersion::new`, `ParseOutput`, `ParseOutput::new`, `ParserSeam`, `ParserSeam::{cache_key_version, parse}`, `StubParserSeam`, `MizarParserSeam` | `crates/mizar-frontend/src/parsing.rs` | inline tests が parser inputs、resolver state 不保持、string-required context mapping、位置別 plan construction、stub seam output、real parser AST handoff、token-kind adaptation、error-recovery tokens、missing-`end` recovery、unrecoverable `ast = None`、string-required forwarding、Pratt fixity/associativity、syntax diagnostic passthrough を確認する。cache-key version の利用は `cache_key` と frontend determinism tests で確認する。 |
| [cache_key.md](./cache_key.md) | `SOURCE_UNIT_CACHE_KEY_VERSION`, `PREPROCESSED_SOURCE_CACHE_KEY_VERSION`, `ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION`, `PARSER_LEXING_PLAN_CACHE_KEY_VERSION`, `TOKEN_STREAM_CACHE_KEY_VERSION`, `SURFACE_AST_CACHE_KEY_VERSION`, `FrontendCacheKeys`, `SourceUnitCacheKey`, `SourceUnitCacheKey::{from_source, stable_hash}`, `PreprocessedSourceCacheKey`, `PreprocessedSourceCacheKey::{from_source, stable_hash}`, `ActiveLexicalEnvironmentCacheKey`, `ActiveLexicalEnvironmentCacheKey::{new, stable_hash}`, `ParserLexingPlanCacheKey`, `ParserLexingPlanContextCacheKey`, `ParserLexingPlanCacheKey::{current, from_plan}`, `TokenStreamCacheKey`, `TokenStreamCacheKey::{new, stable_hash}`, `SurfaceAstCacheKey`, `SurfaceAstCacheKey::{new, stable_hash}`, `parser_inputs_hash` | `crates/mizar-frontend/src/cache_key.rs` | inline tests が source-key freshness exclusion と content identity changes、コメントのみ編集での preprocessing invalidation と token/AST reuse、同一 version の位置別 plan 内容を含む import/environment/parser-context/parser-plan による token invalidation、token-stream/parser-version/parser-input/edition による AST invalidation を確認する。crate-level determinism tests は comment-equivalent run と end-to-end import/dependency invalidation の `FrontendOutput.cache_keys` を確認する。 |
| [orchestration.md](./orchestration.md) | `cache_keys` を含む `FrontendOutput`, `Frontend`, `Frontend::{new, run}`, `FrontendDiagnostic`, `DiagnosticLocation`, `SourceLoadLocation`, `DiagnosticCode`, `DiagnosticClass`, `FrontendError`, `FrontendParserDiagnostic`, `FrontendParserDiagnostic::into_frontend_diagnostic` | `crates/mizar-frontend/src/orchestration.rs` | inline tests が stub/real parser coordinator output、syntax diagnostic merge order、現在の coordinator path に対する repeated-run determinism、same-class sorting、捏造 range のない source-load diagnostic、open-buffer/generated load location、span-bridge hard failure、lexical-environment hard failure、`ast = None` parser seam、valid range-backed merged diagnostics を確認する。crate-level determinism tests は `FrontendOutput.cache_keys` を確認する。 |

英語正本の各行に対応する日本語 companion は、同じ API 名、variant、挙動境界を
保持している。より広い言語表現の同期は task 17 で完了し、
[bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
に記録している。

## エラー／診断 variant 対応

| 対象 | variant | ソース／テスト状態 |
|---|---|---|
| `SpanBridgeError` | `SourceNotRegistered`, `PreprocessMapNotRegistered`, `ConflictingSourceRegistration`, `ConflictingPreprocessMapRegistration`, `UnsupportedLexerPreprocessMap`, `SourceMap` | `span_bridge.rs` に実装済み。未登録／衝突 variant と `SourceMap` wrapping はテスト済み。`UnsupportedLexerPreprocessMap` は将来の未対応 lexer preprocess metadata に対する防御的 conversion guard であり、現在の `mizar-lexer` には producer がない。 |
| `PreprocessDiagnosticKind` | `SourcePrecondition`, `ImportPrescan`, `RawImportScan` | `preprocess.rs` に実装済み。source precondition、import pre-scan、raw-scan recovery テストで確認。 |
| `FrontendLexicalEnvironmentError` | `ProviderUnavailable`, `MalformedProviderProvenance`, `MalformedSummary` | `lexical_env.rs` に実装済み。provider infrastructure、provenance hard failure、malformed-summary テストで確認。 |
| `LexicalEnvironmentDiagnosticCode` | `UnresolvedImport`, `MissingSummary`, `UserSymbolImportConflict`, `InvalidUserSymbolSpelling`, `InvalidUserSymbolArity`, `ReservedWordCollision`, `ReservedSymbolCollision` | `lexical_env.rs` に実装済み。最初の 3 つは現在の frontend recovery path が送出し、直接テスト済み。最後の 4 つは現在の frontend recovery では送出しない。lexer-owned invalid spelling/arity と reserved collision は仕様どおり `MalformedSummary` hard failure のまま扱う。variant-level provider pass-through policy と coverage は task 24 に記録する。 |
| `LexingDiagnosticKind` | `RawScan`, `ScopeSkeleton`, `Lexer` | `lexing.rs` に実装済み。raw-scan recovery、scope-skeleton diagnostic、lexer diagnostic テストで確認。 |
| `LexingDiagnosticPayload` | `None`, `NoValidTokenCandidate`, `ParserContextRejectedCandidate`, `MalformedStringLiteral`, `UnsupportedRawToken`, `UnsupportedLexerPayload` | `lexing.rs` に実装済み。現在の lexer payload はテスト済み。`UnsupportedLexerPayload` は将来 payload variant の明示的 fallback。 |
| `mizar_syntax::SyntaxDiagnosticCode` through `DiagnosticCode::Syntax` | `UnexpectedErrorToken`, `DanglingOperator`, `NonAssociativeOperatorChain`, `MissingEnd`, `MissingStringLiteral`, `UnrecoverableInput` | `mizar-syntax` / `mizar-parser` が所有し、`MizarParserSeam` と `FrontendParserDiagnostic` が pass-through する。frontend/parser tests は現在の各 parser code と syntax diagnostic passthrough を確認している。 |
| `DiagnosticLocation` / `SourceLoadLocation` | `SourceRange`, `SourceLoad`; `Path`, `NormalizedPath`, `OpenBuffer`, `Generated`, `Unknown` | `orchestration.rs` に実装済み。現在の disk、open-buffer、generated、range-backed location はテスト済み。`NormalizedPath` fallback は将来の non-exhaustive source origin 用に予約され、`Unknown` は `SourceInput` が `normalized_path` を持つため現在 producer がない。これらの reserved fallback location の coverage/policy は task 24 に記録する。 |
| `DiagnosticCode` / `DiagnosticClass` | `SourceLoad`, `Preprocess`, `LexicalEnvironment`, `Lexing`, `Syntax`; `SourceLoad`, `LexicalPrecondition`, `CommentStructure`, `ImportPrescan`, `LexicalEnvironment`, `ScopeSkeleton`, `Tokenization`, `Syntax`, `AnnotationSyntax` | `orchestration.rs` に実装済み。現在送出される frontend diagnostic は merge-order と class sorting テストで確認。`AnnotationSyntax` は現在 producer を持たない予約 class であり、将来の producer/test coverage は task 24 の reserved frontend diagnostic surface coverage に残る。 |
| `FrontendError` | `SourceLoad`, `SpanBridge`, `LexicalEnvironment` | `orchestration.rs` に実装済み。hard-failure path テストで確認。 |

## タスク要件対応

| タスク | 状態 | ソース／テスト対応 |
|---|---|---|
| 1 | 完了 | `span_bridge` 公開 API と source-map behavior は `src/span_bridge.rs` に実装済み。mapping と conflict tests がある。 |
| 2 | 完了 | `source` loader bridge は `src/source.rs` に実装済み。projection、loading-map、open-buffer URI path、generated-source、registration、load-error tests がある。 |
| 3 | 完了 | comment/doc-comment preprocessing は `src/preprocess.rs` に実装済み。comment、doc body、annotation、mapping、hash、non-ASCII、unterminated-comment tests がある。 |
| 4 | 完了 | shallow import pre-scan は `src/preprocess.rs` に実装済み。import stub、relative prefix、malformed import、raw-scan failure、source-order、mapping tests がある。 |
| 5 | 完了 | provider seam と provenance API は `src/lexical_env.rs` に実装済み。provider、deduplication、diagnostic、reserved-table tests がある。 |
| 6 | 完了 | active lexical environment recovery は `src/lexical_env.rs` に実装済み。unresolved import、missing summary、conflict retry、malformed summary、fingerprint tests がある。 |
| 7 | 完了 | raw scan と scope skeleton wiring は `src/lexing.rs` に実装済み。raw span と scope-view tests がある。 |
| 8 | 完了 | context-sensitive disambiguation は `src/lexing.rs` に実装済み。user symbol、compound token、string context、token span、payload mapping tests がある。 |
| 9 | 完了 | lexer recovery passthrough は `src/lexing.rs` に実装済み。error-recovery、unsupported raw token、rejected candidate、scope diagnostic preservation tests がある。 |
| 10 | 完了 | parser input assembly と stub seam は `src/parsing.rs` に実装済み。edition/fixity/string context/no-resolver-state/stub tests がある。 |
| 11 | 完了 | real parser seam は `src/parsing.rs` に実装済み。AST handoff、token adaptation、syntax diagnostics、Pratt fixity tests がある。 |
| 12 | 完了 | parser recovery passthrough は `src/parsing.rs` に実装済み。missing-`end`、unrecoverable `ast = None`、string-required context、diagnostic passthrough tests がある。 |
| 13 | 完了 | coordinator と deterministic diagnostic merge は `src/orchestration.rs` に実装済み。stub/real parser output と merge-order tests がある。 |
| 14 | 完了 | unrecoverable failure handling は `src/orchestration.rs` に実装済み。source-load、span-bridge、lexical-environment、`ast = None`、valid range-backed diagnostic tests がある。 |
| 15 | 完了 | refactoring pass は shared whole-lexical-text mapping、source URI boundary tests、lexical-env provenance hard failures、同期済み module specs に反映されている。 |
| 16 | 完了 | 英語正本の監査とこの日本語 companion が source/spec/test 対応と follow-up 状態を記録する。予約 diagnostic/fallback surface 用の新しい task 24 も含む。 |
| 17 | 完了 | bilingual documentation synchronization audit が public API/status/terminology/link/behavior commitments の同期を記録する。 |
| 18 | 完了 | crate-level determinism property tests が provider scheduling permutation と comment-equivalent cache-key stability を確認する。 |
| 19 | 完了 | incremental cache-key wiring は `src/cache_key.rs` に実装され、`FrontendOutput.cache_keys` で公開され、`cache_key.md` に記録された。tests は source/preprocess/environment/token/AST invalidation boundaries を確認する。 |
| 20 | 完了 | parser-assisted lexing は `src/lexing.rs` / `src/parsing.rs` / `src/orchestration.rs` の事前計算済み `ParserLexingPlan` を使う。tests は Unicode/comment-marker 内容を持つ annotation string argument、単一行 range guard、範囲別 user-symbol kind filter、cache-key plan invalidation、実 source-to-token-to-parser handoff を確認する。 |

## Follow-up 記録

この監査では、予約済みまたは現在 producer を持たない diagnostic/fallback surface
の coverage 用に task 24 を追加した。task 18、task 19、task 20 はその後完了した。
残る cross-cutting item は次のとおり。

- Task 21: durable lint enforcement。
- Task 22: precise raw-scan recovery contract。
- Task 23: lexical environment の resident-set contract guard。
- Task 24: reserved frontend diagnostic surface coverage。
