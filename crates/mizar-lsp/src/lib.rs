pub mod range_mapper;

pub use range_mapper::{
    LspPosition, LspRange, RangeMapError, lsp_range_from_lexer_span, lsp_range_from_source_range,
    source_range_from_lexer_span,
};
