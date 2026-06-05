use crate::lexical_env::{ActiveLexicalEnvironment, SymbolId};
use crate::lexing::{ParserLexContext, TokenStream};
use mizar_session::Edition;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRequest<'a> {
    pub tokens: &'a TokenStream,
    pub parser_inputs: ParserInputs,
}

impl<'a> ParseRequest<'a> {
    pub fn new(tokens: &'a TokenStream, parser_inputs: ParserInputs) -> Self {
        Self {
            tokens,
            parser_inputs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserInputs {
    pub edition: Edition,
    pub operator_fixity: OperatorFixityTable,
    pub string_required_positions: StringRequiredContext,
}

impl ParserInputs {
    pub fn new(
        edition: Edition,
        operator_fixity: OperatorFixityTable,
        string_required_positions: StringRequiredContext,
    ) -> Self {
        Self {
            edition,
            operator_fixity,
            string_required_positions,
        }
    }

    pub fn from_active_environment(
        edition: Edition,
        _environment: &ActiveLexicalEnvironment,
    ) -> Self {
        Self {
            edition,
            operator_fixity: OperatorFixityTable::empty(),
            string_required_positions: StringRequiredContext::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperatorFixityTable {
    pub entries: Vec<OperatorFixityEntry>,
}

impl OperatorFixityTable {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorFixityEntry {
    pub symbol_id: SymbolId,
    pub spelling: Arc<str>,
    pub precedence: u16,
    pub associativity: OperatorAssociativity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StringRequiredContext {
    #[default]
    None,
    UniformForTest,
}

impl StringRequiredContext {
    pub fn parser_lex_context(self) -> ParserLexContext {
        match self {
            Self::None => ParserLexContext::general(),
            Self::UniformForTest => ParserLexContext::string_required(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput<A, D> {
    pub ast: Option<A>,
    pub diagnostics: Vec<D>,
}

impl<A, D> ParseOutput<A, D> {
    pub fn new(ast: Option<A>, diagnostics: Vec<D>) -> Self {
        Self { ast, diagnostics }
    }
}

pub trait ParserSeam {
    type Ast;
    type Diagnostic;

    fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StubParserSeam;

impl ParserSeam for StubParserSeam {
    type Ast = ();
    type Diagnostic = ();

    fn parse(&self, _request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic> {
        ParseOutput {
            ast: None,
            diagnostics: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseRequest, ParserInputs, ParserSeam, StringRequiredContext, StubParserSeam};
    use crate::lexical_env::{ActiveLexicalEnvironment, ExportRank, ExportedSymbolShape};
    use crate::lexical_env::{
        LexicalSummaryFingerprint, ModuleId, ModuleLexicalSummary, ResolvedImport, SymbolId,
        UserSymbolArity, UserSymbolKind,
    };
    use crate::lexing::{ParserLexContext, ScopeView, TokenStream};
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
    };

    #[test]
    fn parser_inputs_carry_edition_and_currently_empty_fixity_table() {
        let edition = Edition::new("2026");
        let environment = environment_with_imported_symbol("++");

        let inputs = ParserInputs::from_active_environment(edition.clone(), &environment);

        assert_eq!(inputs.edition, edition);
        assert!(inputs.operator_fixity.is_empty());
        assert_eq!(
            inputs.string_required_positions,
            StringRequiredContext::None
        );
        assert_eq!(
            inputs.string_required_positions.parser_lex_context(),
            ParserLexContext::general()
        );
    }

    #[test]
    fn parser_inputs_do_not_carry_resolver_state_from_active_environment() {
        let environment = environment_with_imported_symbol("++");

        let inputs = ParserInputs::from_active_environment(Edition::new("2026"), &environment);

        assert!(
            environment.user_symbol("++").is_some(),
            "fixture must include resolver-visible symbol state"
        );
        assert!(
            inputs.operator_fixity.entries.is_empty(),
            "current lexical summaries expose no fixity data to carry into ParserInputs"
        );
    }

    #[test]
    fn uniform_test_string_context_maps_to_string_required_lexer_context() {
        assert_eq!(
            StringRequiredContext::UniformForTest.parser_lex_context(),
            ParserLexContext::string_required()
        );
    }

    #[test]
    fn stub_parser_seam_returns_no_ast_and_no_diagnostics() {
        let environment = environment_without_imports();
        let inputs = ParserInputs::from_active_environment(Edition::new("2026"), &environment);
        let tokens = empty_token_stream(source_id(1));
        let seam = StubParserSeam;

        let output = seam.parse(ParseRequest::new(&tokens, inputs));

        assert_eq!(output.ast, None);
        assert!(output.diagnostics.is_empty());
    }

    fn environment_without_imports() -> ActiveLexicalEnvironment {
        mizar_lexer::build_lexical_environment(&[], &[]).unwrap()
    }

    fn environment_with_imported_symbol(spelling: &str) -> ActiveLexicalEnvironment {
        mizar_lexer::build_lexical_environment(
            &[ResolvedImport {
                module_id: ModuleId::new("fixture"),
            }],
            &[ModuleLexicalSummary {
                module_id: ModuleId::new("fixture"),
                exported_symbols: vec![ExportedSymbolShape {
                    spelling: spelling.to_owned(),
                    symbol_id: SymbolId::new("fixture.symbol"),
                    source_module: ModuleId::new("fixture"),
                    export_rank: ExportRank::new(0),
                    kind: UserSymbolKind::Functor,
                    arity: UserSymbolArity::exact(2),
                }],
                fingerprint: LexicalSummaryFingerprint::new(1),
            }],
        )
        .unwrap()
    }

    fn empty_token_stream(source_id: SourceId) -> TokenStream {
        TokenStream {
            source_id,
            parser_context: ParserLexContext::general(),
            tokens: Vec::new(),
            scope_view: ScopeView::empty(source_id),
            diagnostics: Vec::new(),
        }
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
