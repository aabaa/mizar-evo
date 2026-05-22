#![no_main]

use libfuzzer_sys::fuzz_target;
use mizar_lexer::{
    CommentTrivia, RawToken, SourcePreprocessDiagnostic, preprocess_source_for_lexing, scan_raw,
};

fuzz_target!(|source: &str| {
    let preprocessed = preprocess_source_for_lexing(source);

    assert_comment_spans(source, &preprocessed.comments);
    assert_diagnostic_spans(source, &preprocessed.diagnostics);

    if let Ok(raw) = scan_raw(source) {
        assert_raw_token_spans(source, &raw.tokens);
    }

    if let Ok(raw) = scan_raw(&preprocessed.lexical_text) {
        assert_raw_token_spans(&preprocessed.lexical_text, &raw.tokens);
    }
});

fn assert_comment_spans(source: &str, comments: &[CommentTrivia]) {
    for comment in comments {
        assert_valid_span(source, comment.span.start, comment.span.end);
        assert_eq!(
            &source[comment.span.start..comment.span.end],
            comment.lexeme
        );
    }
}

fn assert_diagnostic_spans(source: &str, diagnostics: &[SourcePreprocessDiagnostic]) {
    for diagnostic in diagnostics {
        assert_valid_span(source, diagnostic.span.start, diagnostic.span.end);
    }
}

fn assert_raw_token_spans(source: &str, tokens: &[RawToken]) {
    let mut cursor = 0;
    for token in tokens {
        assert_eq!(token.span.start, cursor);
        assert_valid_span(source, token.span.start, token.span.end);
        assert_eq!(&source[token.span.start..token.span.end], token.lexeme);
        cursor = token.span.end;
    }
    assert_eq!(cursor, source.len());
}

fn assert_valid_span(source: &str, start: usize, end: usize) {
    assert!(start <= end);
    assert!(end <= source.len());
    assert!(source.is_char_boundary(start));
    assert!(source.is_char_boundary(end));
}
