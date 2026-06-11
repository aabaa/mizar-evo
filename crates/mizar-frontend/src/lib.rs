//! Frontend orchestration for source loading, preprocessing, lexing, and parsing.

/// Layered content cache keys for frontend pipeline outputs.
pub mod cache_key;
/// Active lexical environment construction from import stubs and summaries.
pub mod lexical_env;
/// Tokenization and parser-assisted lexing plan support.
pub mod lexing;
/// End-to-end source-to-syntax frontend coordination.
pub mod orchestration;
/// Parser seam and parser input assembly.
pub mod parsing;
/// Source preprocessing, comment extraction, and import pre-scan support.
pub mod preprocess;
/// Source loading bridge from `mizar-session` into frontend source units.
pub mod source;
/// Span conversion bridge between lexer byte offsets and session source maps.
pub mod span_bridge;
