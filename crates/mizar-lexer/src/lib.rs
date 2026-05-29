//! Lexical analysis helpers for Mizar source text.
//!
//! The lexer exposes a small pipeline: [`scan_raw`] performs strict raw
//! scanning, [`lex`] applies reserved-word and reserved-symbol shell
//! disambiguation, and [`disambiguate`] uses the active lexical environment plus
//! parser context when callers need the full parser-facing token stream.
//!
//! Token spans are byte offsets into the string passed to the scanner.
//! File-loading callers should validate UTF-8 bytes with
//! [`load_source_text_from_bytes`] before lexer entry when they need the
//! source-loading boundary helper used by this crate's tests. That helper also
//! strips one leading UTF-8 BOM and normalizes CRLF newline pairs to LF while
//! keeping a loading map back to original input byte offsets.
//! [`preprocess_source_for_lexing`] keeps a lightweight preprocessing map so
//! callers can relate comment-stripped lexical ranges back to the loaded source.
//!
//! ## Source-text normalization
//!
//! The lexer does not perform Unicode normalization. Code-region identifiers,
//! numerals, and user-symbol spellings are ASCII-only, and non-ASCII code text
//! that reaches preprocessing is reported as malformed input rather than
//! normalized into an accepted spelling. Comment and documentation text is
//! preserved as raw Unicode trivia unless a later source-loading or
//! documentation layer decides to issue warnings.
//!
//! ## API stability
//!
//! This crate is still a `0.1` lexer boundary. Public enums are
//! `#[non_exhaustive]` so downstream callers keep wildcard arms for token,
//! diagnostic, parser-mode, scope-skeleton, source-preprocessing, import, and
//! lexical-environment categories that may grow as the parser integration
//! matures.
//!
//! Public data structs intentionally keep their fields visible for now because
//! they are parser-facing transfer objects used by tests, corpus metadata, and
//! early integration code. Those fields should still be treated as provisional
//! while the crate remains below `1.0`. New parser integration code should
//! prefer constructors and accessors on token streams, tokens, diagnostics,
//! spans, IDs, ranks, and fingerprints where they exist; visible fields remain
//! available as transfer-object escape hatches during the `0.x` line.
//!
//! ```
//! use mizar_lexer::{scan_raw, RawTokenKind};
//!
//! let source = "alpha beta";
//! let raw = scan_raw(source)?;
//!
//! assert_eq!(raw.tokens[0].kind, RawTokenKind::LexemeRun);
//! assert_eq!(&source[raw.tokens[0].span.start..raw.tokens[0].span.end], "alpha");
//! # Ok::<(), mizar_lexer::LexError>(())
//! ```
//!
//! ```
//! use mizar_lexer::{lex, TokenKind};
//!
//! let source = "alpha beta";
//! let tokens = lex(source)?;
//!
//! assert_eq!(tokens[1].kind, TokenKind::Identifier);
//! assert_eq!(&source[tokens[1].span.start..tokens[1].span.end], "beta");
//! # Ok::<(), mizar_lexer::LexError>(())
//! ```
//!
//! ```
//! use mizar_lexer::{
//!     build_lexical_environment, build_scope_skeleton, disambiguate, scan_raw,
//!     ParserLexContext, TokenKind,
//! };
//!
//! let source = "alpha:=beta";
//! let raw = scan_raw(source)?;
//! let environment = build_lexical_environment(&[], &[])?;
//! let scopes = build_scope_skeleton(&raw);
//! let stream = disambiguate(&raw, &environment, &ParserLexContext::general(), &scopes);
//!
//! assert!(stream.diagnostics.is_empty());
//! assert_eq!(
//!     stream
//!         .tokens
//!         .iter()
//!         .map(|token| token.lexeme.as_str())
//!         .collect::<Vec<_>>(),
//!     vec!["alpha", ":=", "beta"],
//! );
//! assert_eq!(stream.tokens[1].kind, TokenKind::ReservedSymbol);
//! assert_eq!(&source[stream.tokens[1].span.start..stream.tokens[1].span.end], ":=");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
mod disambiguator;
mod import_prescan;
mod lexical_environment;
mod raw_lexer;
mod scope_skeleton;
mod source;
mod tables;

pub use disambiguator::{
    LexDiagnostic, LexDiagnosticCode, ParserLexContext, ParserLexMode, Token, TokenKind,
    TokenStream, disambiguate, disambiguate_reserved_shell, lex,
};
pub use import_prescan::{
    ImportPrelude, ImportPrescanDiagnostic, ImportPrescanDiagnosticCode, ImportStub,
    RawModuleAlias, RawModulePath, RawModulePathComponent, RawModuleRelativePrefix,
    scan_import_prelude,
};
pub use lexical_environment::{
    ActiveLexicalEnvironment, ExportRank, ExportedSymbolShape, LexicalEnvironmentError,
    LexicalEnvironmentFingerprint, LexicalSummaryFingerprint, ModuleId, ModuleLexicalSummary,
    ResolvedImport, SymbolId, UserSymbolArity, UserSymbolCandidate, UserSymbolIndex,
    UserSymbolKind, UserSymbolKindSet, build_lexical_environment,
};
pub use raw_lexer::{
    LexError, RawToken, RawTokenKind, RawTokenStream, is_identifier, is_identifier_continue,
    is_identifier_start, is_layout, is_lexeme_run_char, is_numeral, is_string_literal_spelling,
    is_user_symbol_spelling, scan_raw,
};
pub use scope_skeleton::{
    BindingShapeKind, LexicalBlockKind, LexicalBlockRange, LexicalScopeFrame, LexicalStatementKind,
    LexicalStatementRange, ScopeLexView, ScopeSkeleton, ScopeSkeletonDiagnostic,
    ScopeSkeletonDiagnosticCode, ScopedBindingShape, build_scope_skeleton,
};
pub use source::{
    CommentKind, CommentTrivia, LoadedSourceText, ModuleNamingError, ModuleSourceName,
    PreprocessedLexicalSource, SourceLineIndex, SourceLoadError, SourceLoadingMap,
    SourceLoadingMapSegment, SourceLocation, SourceLocationRange, SourcePos,
    SourcePreprocessDiagnostic, SourcePreprocessDiagnosticCode, SourcePreprocessMap,
    SourcePreprocessMapSegment, SourceRange, SourceSpan, load_source_text_from_bytes,
    module_source_name_from_path, preprocess_source_for_lexing,
};
pub use tables::{
    RESERVED_SYMBOLS, RESERVED_WORDS, ReservedSymbolTable, ReservedWordTable, is_reserved_symbol,
    is_reserved_word, longest_reserved_symbol_prefix,
};

#[cfg(test)]
mod tests;
