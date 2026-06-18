# ソース／仕様対応監査

> 正本は英語です。英語版: [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

状態: task 29 まで完了。mizar-lexer の source-position-aware operator metadata
連携に合わせて更新済み。

## 範囲

この監査は、task 29 後の `mizar-frontend` 実装を、まず英語正本の
`doc/design/mizar-frontend/en/` 仕様に照合し、その後で日本語 companion
仕様が同じ公開 API 名、エラー／診断 variant、挙動の約束を保持している
ことを確認する。

これは軽量なソース／仕様／テスト対応表であり、release coverage gate でも、
実行可能テストの代替でもない。欠落した実装、古くなった仕様文、欠落した
テストが見つかった場合は、この監査に広範な変更を混ぜず、follow-up task
として記録する。

## 結果

- task 1-20 が約束する公開 API とエラー／診断 variant、task 21 の lint
  policy guard、task 22 の精密な生スキャン回復契約、および task 23 の
  resident-set guard、task 25 の enum 前方互換方針、task 26 の rustdoc
  summary 方針、task 28 の parser-growth follow-through、task 29 の
  real-parser frontend fuzz coverage、および mizar-lexer の source-position-aware
  operator metadata 連携について、欠落した実装は見つからなかった。
- task 2 の source 要件文は、監査前に追加済みだった open-buffer `file://`
  診断パスの decode/fallback テストを明示するよう更新した。
- task 1-29 と source-position-aware operator metadata 連携について、英語正本仕様に残る古い記述は見つからなかった。
- 日本語 companion 仕様は、API 名と挙動の約束が英語正本と一致することを
  確認した。API または挙動の drift は残っていない。
- より広い bilingual wording/terminology review は task 17 で完了し、
  [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
  に記録した。task 19 の incremental cache-key wiring、task 20 の
  parser-assisted lexing、task 21 の durable lint enforcement、task 22 の
  precise raw-scan recovery、task 23 の resident-set contract coverage、
  task 24 の reserved diagnostic surface coverage、task 25 の enum forward-compatibility、
  task 26 の rustdoc summary coverage、task 28 の parser-growth follow-through は現在完了済みである。将来の
  producer-backed tests は、将来の lexer/session/parser producer に結び付けて追加する。

## 公開 API 対応

| 仕様 | 確認した公開 API | ソース | テスト証跡 |
|---|---|---|---|
| [span_bridge.md](./span_bridge.md) | `SpanBridge`, `LexerByteSpan`, `SpanBridgeError`; `SpanBridge::{new, source_map_service, register_source, register_preprocess_map, loaded_span, loaded_mapping, lexical_span}`; crate 可視 `whole_lexical_text_mapping` | `crates/mizar-frontend/src/span_bridge.rs` | inline tests が loaded/lexical mapping、loading-map identity behavior、composite/degraded preprocess mapping、whole-lexical-text recovery mapping、不正 span、未登録、登録衝突、公開 `UnsupportedLexerPreprocessMap` 防御的 error surface を確認する。現在の `mizar-lexer` からは未対応 map producer を構築できないため、producer-backed coverage は将来の lexer metadata variant まで延期する。 |
| [source.md](./source.md) | `SourceUnit`, `SourceUnitRequest`, `SourceUnitLoader`, `SourceUnitLoader::load_source_unit`, `FrontendSourceLoader`, `FrontendSourceLoader::new`, `source_unit_from_loaded`, `register_source_unit` | `crates/mizar-frontend/src/source.rs` | inline tests が再計算しない projection、loader forwarding、BOM/CRLF loading map、identity load、open-buffer origin/version、open-buffer `file://` path decode/fallback、generated source、bridge registration、`SourceLoadError` の非再分類伝播を確認する。 |
| [preprocess.md](./preprocess.md) | `PreprocessedSource`, `LexicalText`, `LexicalText::as_str`, `Comment`, `DocComment`, `LexicalSourceMap`, `LexicalSourceMap::{lexical_span, lexical_len, is_empty}`, `ImportStub`, `ImportStubPath`, `ImportStubRelativePrefix::{Current, Parent}`, `ImportStubAlias`, `PreprocessDiagnostic`, `PreprocessDiagnosticKind`, `preprocess`, `lexical_hash`; 再エクスポート `ImportPrescanDiagnosticCode`, `SourcePreprocessDiagnosticCode`, `CommentKind` | `crates/mizar-frontend/src/preprocess.rs` | inline tests が comment/doc-comment separation、annotation preservation、import stub と relative prefix、malformed import recovery、import を保持し error-sentinel path boundary も固定する精密な回復可能 raw import scan diagnostics、mapped diagnostics、composite/degraded lexical mappings、stable lexical hash、non-ASCII code diagnostics、unterminated block comment を確認する。 |
| [lexical_env.md](./lexical_env.md) | `LexicalEnvironmentRequest`, `LexicalSummaryProvider`, `LexicalSummaryProvider::resolve_imports`, `ResolvedImports`, `ResolvedImportEntry`, `ActiveLexicalEnvironmentResult`, `FrontendLexicalEnvironmentError`, `LexicalEnvironmentDiagnostic`, `LexicalEnvironmentDiagnosticCode`, `build_active_lexical_environment`; `ActiveOperatorMetadata`、`ExportedOperatorMetadata`、`ExportedOperatorFixity`、`ExportedOperatorAssociativity`、`LocalLexicalDeclarations` を含む再エクスポートされた lexer environment 型 | `crates/mizar-frontend/src/lexical_env.rs` | inline tests が provider seam output、import deduplication、reserved tables、provider infrastructure failure、provider provenance hard failure、unresolved import、missing summary、conflict retry、non-conflict malformed summary、予約診断 code の provider-owned pass-through、不正 operator metadata を malformed summary とする validation、fingerprint stability/change behavior を確認する。`tests/lexical_env_resident_set.rs` は resident-set boundary として、直接 `ImportStub` だけに scope された provider call が 1 回であること、active-environment candidate field が summary 由来であり推移依存 symbol を含まないことを確認する。 |
| [lexing.md](./lexing.md) | `InternedText`, `TokenizeRequest`, `TokenizeRequest::{new, with_plan, with_current_module}`, `ParserLexingPlan`, `ParserLexingPlan::{uniform, new, for_lexical_text, context_at, is_uniform}`, `ParserLexingPlanContext`, `ParserLexingPlanContext::new`, `LexicalByteRange`, `LexicalByteRange::{new, contains}`, `TokenStream`, `TokenStream::{tokens, diagnostics, scope_view, local_declarations, into_parts}`, `Token`, `ScopeView`, `ScopeView::{empty, binding_overrides_symbol}`, `ScopeFrame`, `ScopedBinding`, `ScopeBlock`, `ScopeStatement`, `LexingDiagnostic`, `LexingDiagnosticKind`, `LexingDiagnosticPayload`, `LexingRejectedTokenCandidate`, `tokenize`; 再エクスポートされた lexer token/context/scope enum | `crates/mizar-frontend/src/lexing.rs` | inline tests が raw-span preservation、preprocess mapping、longest-match user symbols、scoped identifier overrides、compound reserved tokens、parser context/string behavior、Unicode/comment-marker 内容を持つ位置別 annotation string argument、planned string range の line-boundary rejection、範囲別 user-symbol kind filter、preprocess 済みコメントをまたぐ current-module local declaration collection、現在 producer を持つ payload mapping、recoverable error tokens、unsupported raw-token recovery、rejected candidates、secondary anchors、scope view contents、scope diagnostics、partial token continuation を伴う精密な回復可能 raw-scan diagnostics を確認する。`UnsupportedLexerPayload` は、現在 producer を持たない将来の non-exhaustive lexer payload variant 用 fallback mapping として文書化されている。 |
| [parsing.md](./parsing.md) | `DEFAULT_PARSER_CACHE_KEY_VERSION`, `STUB_PARSER_CACHE_KEY_VERSION`, `MIZAR_PARSER_CACHE_KEY_VERSION`, `ParseRequest`, `ParseRequest::new`, `ParserInputs`, `ParserInputs::{new, from_active_environment, from_active_environment_and_local_declarations, try_from_active_environment_and_local_declarations}`, `OperatorFixityTable`, `OperatorFixityTable::{empty, is_empty, try_from_active_environment_and_local_declarations}`, `OperatorFixityEntry`, `OperatorFixity::{Prefix, Infix, Postfix}`, `OperatorAssociativity::{Left, Right, NonAssociative}`, `StringRequiredContext::{None, PositionSensitive, UniformForTest}`, `StringRequiredContext::{parser_lex_context, parser_lexing_plan}`, `ParserCacheKeyVersion`, `ParserCacheKeyVersion::new`, `ParseOutput`, `ParseOutput::new`, `ParserSeam`, `ParserSeam::{cache_key_version, parse}`, `StubParserSeam`, `MizarParserSeam` | `crates/mizar-frontend/src/parsing.rs` | inline tests が parser inputs、active lexical environment 由来の summary-derived および local declaration 由来の prefix/postfix/infix operator fixity、parser-facing operator metadata の source-coordinate activation mapping、operator metadata を持たない symbol について resolver state を運ばないこと、string-required context mapping、位置別 plan construction、stub seam output、real parser AST handoff、token-kind adaptation、error-recovery tokens、部分的に閉じた nested block と algorithm control block を含む EOF missing-`end` recovery、quantifier `for` の除外、`match otherwise` branch matching、unrecoverable `ast = None`、string-required forwarding、明示的 Pratt fixity/associativity、activation-aware real-parser fixity forwarding、syntax diagnostic passthrough を確認する。cache-key version の利用は `cache_key`、frontend determinism tests、orchestration parser-version passthrough coverage で確認する。 |
| [cache_key.md](./cache_key.md) | `SOURCE_UNIT_CACHE_KEY_VERSION`, `PREPROCESSED_SOURCE_CACHE_KEY_VERSION`, `ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION`, `PARSER_LEXING_PLAN_CACHE_KEY_VERSION`, `TOKEN_STREAM_CACHE_KEY_VERSION`, `SURFACE_AST_CACHE_KEY_VERSION`, `FrontendCacheKeys`, `SourceUnitCacheKey`, `SourceUnitCacheKey::{from_source, stable_hash}`, `PreprocessedSourceCacheKey`, `PreprocessedSourceCacheKey::{from_source, stable_hash}`, `ActiveLexicalEnvironmentCacheKey`, `ActiveLexicalEnvironmentCacheKey::{new, stable_hash}`, `ParserLexingPlanCacheKey`, `ParserLexingPlanContextCacheKey`, `ParserLexingPlanCacheKey::{current, from_plan}`, `TokenStreamCacheKey`, `TokenStreamCacheKey::{new, stable_hash}`, `SurfaceAstCacheKey`, `SurfaceAstCacheKey::{new, stable_hash}`, `parser_inputs_hash` | `crates/mizar-frontend/src/cache_key.rs` | inline tests が source-key freshness exclusion と content identity changes、コメントのみ編集での preprocessing invalidation と token/AST reuse、同一 version の位置別 plan 内容を含む import/environment/parser-context/parser-plan による token invalidation、token-stream/parser-version/parser-input/edition/operator-fixity-activation による AST invalidation を確認する。crate-level determinism tests は comment-equivalent run と end-to-end import/dependency invalidation の `FrontendOutput.cache_keys` を確認する。 |
| [orchestration.md](./orchestration.md) | `cache_keys` を含む `FrontendOutput`, `Frontend`, `Frontend::{new, run}`, `FrontendDiagnostic`, `DiagnosticLocation`, `SourceLoadLocation`, `DiagnosticCode`, `DiagnosticClass`, `FrontendError`, `FrontendParserDiagnostic`, `FrontendParserDiagnostic::into_frontend_diagnostic` | `crates/mizar-frontend/src/orchestration.rs` | inline tests が stub/real parser coordinator output、source-position-aware local operator activation と current-module forwarding の parser inputs/cache keys への反映、nested missing-`end` recovery を含む syntax diagnostic merge order、parser-version cache-key passthrough、現在の coordinator path に対する repeated-run determinism、same-class sorting、捏造 range のない source-load diagnostic、open-buffer/generated load location、予約 source-load fallback location、予約 annotation-syntax class、`UnsupportedLexerPayload` に対する no-recovery-note conversion、span-bridge hard failure、lexical-environment hard failure、`ast = None` parser seam、valid range-backed merged diagnostics を確認する。crate-level determinism tests は `FrontendOutput.cache_keys` を確認する。 |

英語正本の各行に対応する日本語 companion は、同じ API 名、variant、挙動境界を
保持している。より広い言語表現の同期は task 17 で完了し、
[bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
に記録している。

## エラー／診断 variant 対応

task 25 では、将来 variant または予約 surface を約束する公開 frontend enum を下流 crate 向けに `#[non_exhaustive]` とする:
`SpanBridgeError`、`PreprocessDiagnosticKind`、
`LexicalEnvironmentDiagnosticCode`、`LexingDiagnosticKind`、
`LexingDiagnosticPayload`、`SourceLoadLocation`、`DiagnosticCode`、
`DiagnosticClass`、`FrontendError`。これら frontend-owned enum に対する内部 match は
引き続き exhaustive に保つ。上流 crate が所有する non-exhaustive enum には、文書化された
frontend fallback を使う。

| 対象 | variant | ソース／テスト状態 |
|---|---|---|
| `SpanBridgeError` | `SourceNotRegistered`, `PreprocessMapNotRegistered`, `ConflictingSourceRegistration`, `ConflictingPreprocessMapRegistration`, `UnsupportedLexerPreprocessMap`, `SourceMap` | `span_bridge.rs` に実装済み。未登録／衝突 variant、`SourceMap` wrapping、`UnsupportedLexerPreprocessMap` の直接構築／表示はテスト済み。`UnsupportedLexerPreprocessMap` は将来の未対応 lexer preprocess metadata に対する防御的 conversion guard のままであり、現在の `mizar-lexer` には producer がない。 |
| `PreprocessDiagnosticKind` | `SourcePrecondition`, `ImportPrescan`, `RawImportScan` | `preprocess.rs` に実装済み。source precondition、import pre-scan、raw-scan recovery テストで確認。 |
| `FrontendLexicalEnvironmentError` | `ProviderUnavailable`, `MalformedProviderProvenance`, `MalformedSummary` | `lexical_env.rs` に実装済み。provider infrastructure、provenance hard failure、malformed-summary テストで確認。 |
| `LexicalEnvironmentDiagnosticCode` | `UnresolvedImport`, `MissingSummary`, `UserSymbolImportConflict`, `InvalidUserSymbolSpelling`, `InvalidUserSymbolArity`, `ReservedWordCollision`, `ReservedSymbolCollision` | `lexical_env.rs` に実装済み。最初の 3 つは現在の frontend recovery path が送出し、直接テスト済み。最後の 4 つは、出所検証後の provider-owned pass-through 診断として直接 provider fixture でテスト済み。lexer-owned invalid spelling/arity と reserved collision は仕様どおり `MalformedSummary` hard failure のまま扱う。 |
| `LexingDiagnosticKind` | `RawScan`, `ScopeSkeleton`, `Lexer` | `lexing.rs` に実装済み。raw-scan recovery、scope-skeleton diagnostic、lexer diagnostic テストで確認。 |
| `LexingDiagnosticPayload` | `None`, `NoValidTokenCandidate`, `ParserContextRejectedCandidate`, `MalformedStringLiteral`, `UnsupportedRawToken`, `UnsupportedLexerPayload` | `lexing.rs` に実装済み。現在の lexer payload は producer-backed lexing tests で確認済み。`UnsupportedLexerPayload` は将来 payload variant の明示的 fallback であり、ユーザー向け no-recovery-note 方針は orchestration の diagnostic conversion 経由で直接テスト済み。 |
| `mizar_syntax::SyntaxDiagnosticCode` through `DiagnosticCode::Syntax` | `UnexpectedErrorToken`, `DanglingOperator`, `NonAssociativeOperatorChain`, `MissingEnd`, `MissingSemicolon`, `MissingStringLiteral`, `MalformedImport`, `MalformedExport`, `MalformedVisibility`, `MalformedTypeExpression`, `MalformedTermExpression`, `MalformedFormulaExpression`, `UnexpectedTopLevelToken`, `UnrecoverableInput`; 将来の non-exhaustive code 用 fallback key `syntax_diagnostic` | `mizar-syntax` / `mizar-parser` が所有し、`MizarParserSeam` と `FrontendParserDiagnostic` が pass-through する。frontend/parser tests は現在の各 parser code と syntax diagnostic passthrough を確認している。fallback の producer-backed coverage は、`mizar-syntax` が新しい code を追加するまで延期する。 |
| `DiagnosticLocation` / `SourceLoadLocation` | `SourceRange`, `SourceLoad`; `Path`, `NormalizedPath`, `OpenBuffer`, `Generated`, `Unknown` | `orchestration.rs` に実装済み。現在の disk、open-buffer、generated、range-backed location はテスト済み。`NormalizedPath` は将来の non-exhaustive source origin 用 fallback、`Unknown` は normalized input path を持たない将来の source-load 診断用に予約する。どちらの fallback variant も決定的順序を直接テスト済みで、producer-backed coverage はそのような source contract が追加されるまで延期する。 |
| `DiagnosticCode` / `DiagnosticClass` | `SourceLoad`, `Preprocess`, `LexicalEnvironment`, `Lexing`, `Syntax`; `SourceLoad`, `LexicalPrecondition`, `CommentStructure`, `ImportPrescan`, `LexicalEnvironment`, `ScopeSkeleton`, `Tokenization`, `Syntax`, `AnnotationSyntax` | `orchestration.rs` に実装済み。送出済みおよび予約済み frontend diagnostic は merge-order と class sorting テストで確認。`AnnotationSyntax` は producer を持たない予約 class だが、決定的順序 coverage はある。producer-backed coverage は annotation parsing が専用診断を公開するまで延期する。 |
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
| 21 | 完了 | durable lint enforcement は `crates/mizar-frontend/tests/lint_policy.rs` で固定する。この test は frontend manifest の workspace lint opt-in、共有 rustc/clippy denial baseline、将来の frontend `allow` 属性に隣接した理由があることを確認する。 |
| 22 | 完了 | 精密な生スキャン回復は `mizar_lexer::scan_raw_recoverable` で実装する。frontend の import pre-scan と tokenization は `RawScanDiagnostic` のスパンを精密に写像し、利用可能な部分的 import/token を保持し、字句テキスト全体の fallback は parser plan の内部欠陥に限る。 |
| 23 | 完了 | resident-set contract coverage は `crates/mizar-frontend/tests/lexical_env_resident_set.rs` で固定する。この test は直接 `ImportStub` に対する provider request が正確に 1 回であること、`ActiveLexicalEnvironment` が `ModuleLexicalSummary` 由来の字句的形状／出所だけを公開し、推移依存 symbol を含まないことを確認する。 |
| 24 | 完了 | 予約済み frontend diagnostic surface は、構築可能な範囲で coverage 済み: `UnsupportedLexerPreprocessMap`、provider-owned の予約 lexical-environment diagnostic code、予約 source-load fallback location、`AnnotationSyntax`、`UnsupportedLexerPayload`。producer-backed tests は、将来の non-exhaustive lexer/session/parser contract まで延期する。 |
| 25 | 完了 | 将来 variant または予約 surface を約束する公開 frontend enum は下流 crate 向けに `#[non_exhaustive]` とし、これら frontend-owned enum に対する内部 match は exhaustive に保つ。所有モジュール仕様は enum の隣に enum ごとの決定を記録している。 |
| 26 | 完了 | 公開 `mizar-frontend` module と公開 API item は、正準 design spec 由来の短い rustdoc summary を持つ。詳細な挙動の約束は引き続き `doc/design/mizar-frontend/en/` に置く。 |
| 27 | 完了 | `frontend_valid_utf8` fuzz target と Criterion frontend baseline は `fuzz/` と `crates/mizar-frontend/benches/frontend_pipeline.rs` に実装済み。task 28 で発火した real-parser fuzz follow-up は task 29 で完了した。 |
| 28 | 完了 | parser-growth follow-through は `mizar-parser`、`src/parsing.rs`、`src/orchestration.rs` に実装済み。nested block-end recovery、algorithm control-block matching、quantifier `for` exclusion、frontend recovery-node passthrough、構文診断 merge coverage、parser 出力 semantics 変更に対する `MIZAR_PARSER_CACHE_KEY_VERSION` invalidation を確認する。 |
| 29 | 完了 | `frontend_valid_utf8` fuzz target は valid UTF-8 を `MizarParserSeam`、構文診断 merge、AST が生成された場合の `SurfaceAstCacheKey` construction まで流す。parser-owned fuzz target は `mizar-parser` task 40 で完了済みである。 |

## Follow-up 記録

この監査では、予約済みまたは現在 producer を持たない diagnostic/fallback surface
の coverage 用に task 24 を追加した。task 18、task 19、task 20、task 21、
task 22、task 23、task 24、task 25、task 26、task 27、task 28、task 29 はその後完了した。
task 29 は frontend-owned の real-parser fuzz follow-up を完了した。parser-owned 側は
`mizar-parser` task 40 で完了済みである。現在予約されている fallback variant に対する
具体的 producer を将来の non-exhaustive lexer/session/parser contract が公開した場合は、
producer-backed tests を追加する。
