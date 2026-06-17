pub(crate) use crate::{
    CommentKind, ExportRank, ExportedOperatorAssociativity, ExportedOperatorFixity,
    ExportedOperatorMetadata, ExportedSymbolShape, ImportPrescanDiagnosticCode, LexDiagnosticCode,
    LexDiagnosticPayload, LexRecoveryHint, LexicalBlockKind, LexicalEnvironmentError,
    LexicalSummaryFingerprint, MalformedStringLiteralReason, ModuleId, ModuleLexicalSummary,
    ModuleNamingError, ParserLexContext, ParserLexMode, RESERVED_SYMBOLS, RESERVED_WORDS,
    RawModuleRelativePrefix, RawScanDiagnostic, RawScanDiagnosticCode, RawToken, RawTokenKind,
    RecoverableRawTokenStream, RejectedTokenCandidate, ResolvedImport, ScopeLexView,
    ScopeSkeletonDiagnosticCode, SourceLineIndex, SourceLoadError, SourceLoadingMapSegment,
    SourceLocation, SourceLocationRange, SourcePreprocessDiagnosticCode,
    SourcePreprocessDiagnosticPayload, SourcePreprocessMapSegment, SourcePreprocessRecoveryHint,
    SourceSpan, SymbolId, Token, TokenKind, UserSymbolArity, UserSymbolCandidate, UserSymbolKind,
    UserSymbolKindSet, build_lexical_environment, build_scope_skeleton,
    collect_local_lexical_declarations, disambiguate, disambiguate_with_local_declarations,
    is_identifier, is_layout, is_numeral, is_reserved_symbol, is_reserved_word,
    is_string_literal_spelling, is_user_symbol_spelling, lex, load_source_text_from_bytes,
    longest_reserved_symbol_prefix, module_source_name_from_path, preprocess_source_for_lexing,
    scan_import_prelude, scan_raw, scan_raw_recoverable,
};

pub(crate) fn token(kind: TokenKind, lexeme: &str, start: usize, end: usize) -> Token {
    Token {
        kind,
        lexeme: lexeme.to_owned(),
        span: SourceSpan { start, end },
    }
}

pub(crate) fn assert_final_token_spans_point_to_lexemes(source: &str, tokens: &[Token]) {
    for token in tokens {
        assert!(
            token.span.start <= token.span.end,
            "{source:?}: invalid span {:?} for {:?}",
            token.span,
            token
        );
        assert!(
            token.span.end <= source.len(),
            "{source:?}: out-of-bounds span {:?} for {:?}",
            token.span,
            token
        );
        assert_eq!(
            &source[token.span.start..token.span.end],
            token.lexeme,
            "{source:?}: final token span should point back to its spelling"
        );
    }
}

pub(crate) fn resolved_import(module: &str) -> ResolvedImport {
    ResolvedImport {
        module_id: module_id(module),
    }
}

pub(crate) fn summary(
    module: &str,
    fingerprint: u64,
    exported_symbols: &[ExportedSymbolShape],
) -> ModuleLexicalSummary {
    ModuleLexicalSummary {
        module_id: module_id(module),
        exported_symbols: exported_symbols.to_vec(),
        fingerprint: LexicalSummaryFingerprint(fingerprint),
    }
}

pub(crate) fn exported(
    spelling: &str,
    symbol: &str,
    source_module: &str,
    rank: u32,
) -> ExportedSymbolShape {
    exported_with_metadata(
        spelling,
        symbol,
        source_module,
        rank,
        UserSymbolKind::Functor,
        UserSymbolArity::exact(2),
    )
}

pub(crate) fn exported_with_metadata(
    spelling: &str,
    symbol: &str,
    source_module: &str,
    rank: u32,
    kind: UserSymbolKind,
    arity: UserSymbolArity,
) -> ExportedSymbolShape {
    ExportedSymbolShape {
        spelling: spelling.to_owned(),
        symbol_id: symbol_id(symbol),
        source_module: module_id(source_module),
        export_rank: ExportRank(rank),
        kind,
        arity,
        operator: None,
    }
}

pub(crate) fn exported_with_operator_metadata(
    spelling: &str,
    symbol: &str,
    source_module: &str,
    rank: u32,
    operator: ExportedOperatorMetadata,
) -> ExportedSymbolShape {
    let arity = match operator.fixity {
        ExportedOperatorFixity::Prefix | ExportedOperatorFixity::Postfix => {
            UserSymbolArity::exact(1)
        }
        ExportedOperatorFixity::Infix(_) => UserSymbolArity::exact(2),
    };
    ExportedSymbolShape {
        operator: Some(operator),
        ..exported_with_metadata(
            spelling,
            symbol,
            source_module,
            rank,
            UserSymbolKind::Functor,
            arity,
        )
    }
}

pub(crate) fn module_id(value: &str) -> ModuleId {
    ModuleId(value.to_owned())
}

pub(crate) fn symbol_id(value: &str) -> SymbolId {
    SymbolId(value.to_owned())
}

pub(crate) fn nth_index(haystack: &str, needle: &str, ordinal: usize) -> usize {
    haystack
        .match_indices(needle)
        .nth(ordinal)
        .map(|(index, _)| index)
        .expect("test source should contain requested occurrence")
}
