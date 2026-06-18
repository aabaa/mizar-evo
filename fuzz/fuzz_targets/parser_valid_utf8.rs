#![no_main]

use libfuzzer_sys::fuzz_target;
use mizar_frontend::lexical_env::{
    FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider, ModuleId,
    ResolvedImports, build_active_lexical_environment,
};
use mizar_frontend::lexing::{TokenizeRequest, tokenize};
use mizar_frontend::parsing::{MizarParserSeam, ParseRequest, ParserInputs, ParserSeam};
use mizar_frontend::preprocess::preprocess;
use mizar_frontend::source::{
    FrontendSourceLoader, SourceUnitLoader, SourceUnitRequest, register_source_unit,
};
use mizar_frontend::span_bridge::{LexerByteSpan, SpanBridge, SpanBridgeError};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, LspDocumentVersion,
    ModulePath, PackageId, SourceInput, SourceOriginInput, SourceRange, normalize_path,
};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

fuzz_target!(|source: &str| {
    let fixture = Fixture::get();
    let loader = FrontendSourceLoader::new(DiskSourceLoader::new(&fixture.package_root));
    let source_unit = loader
        .load_source_unit(fixture.request(source), &InMemorySessionIdAllocator::new())
        .expect("parser valid-UTF-8 fuzz input should load without hard error");

    let mut bridge = SpanBridge::new();
    register_source_unit(&mut bridge, &source_unit)
        .expect("parser fuzz source should register in span bridge");
    let preprocessed =
        preprocess(&source_unit, &mut bridge).expect("parser fuzz preprocessing should recover");
    assert_eq!(
        preprocessed.lexical_text.as_str().len(),
        preprocessed.source_map.lexical_len()
    );
    for diagnostic in &preprocessed.diagnostics {
        assert_source_range(&bridge, diagnostic.primary);
    }

    let lexical_environment = build_active_lexical_environment(
        &LexicalEnvironmentRequest {
            source_id: source_unit.source_id,
            import_stubs: &preprocessed.import_stubs,
            edition: source_unit.edition.clone(),
        },
        &EmptySummaryProvider,
    )
    .expect("empty fuzz summary provider should recover with diagnostics");
    for diagnostic in &lexical_environment.diagnostics {
        assert_source_range(&bridge, diagnostic.primary);
    }

    let parser_input_policy = ParserInputs::from_active_environment(
        source_unit.edition.clone(),
        &lexical_environment.environment,
    );
    let parser_lexing_plan = parser_input_policy
        .string_required_positions
        .parser_lexing_plan(preprocessed.lexical_text.as_str());
    let tokens = tokenize(
        TokenizeRequest::with_plan(
            &preprocessed,
            &lexical_environment.environment,
            parser_lexing_plan,
        )
        .with_current_module(ModuleId::new(source_unit.module_path.as_str())),
        &bridge,
    )
    .expect("parser fuzz tokenization should recover");
    for diagnostic in tokens.diagnostics() {
        assert_source_range(&bridge, diagnostic.primary);
    }

    let parser_inputs = ParserInputs::try_from_active_environment_and_local_declarations(
        source_unit.edition,
        &lexical_environment.environment,
        tokens.local_declarations(),
        |position| lexical_activation_source_offset(source_unit.source_id, &bridge, position),
    )
    .expect("parser fuzz activation offsets should map to source offsets");
    let parser_output = MizarParserSeam.parse(ParseRequest::new(&tokens, parser_inputs));
    for diagnostic in &parser_output.diagnostics {
        assert_source_range(&bridge, diagnostic.primary);
    }
    if let Some(ast) = &parser_output.ast {
        let _ = ast.snapshot_text();
        for token_id in ast.token_nodes() {
            let node = ast
                .node(*token_id)
                .expect("parser fuzz token node id should be valid");
            assert_source_range(&bridge, node.range);
        }
    }
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmptySummaryProvider;

impl LexicalSummaryProvider for EmptySummaryProvider {
    fn resolve_imports(
        &self,
        _request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        Ok(ResolvedImports {
            imports: Vec::new(),
            summaries: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
struct Fixture {
    package_root: PathBuf,
    normalized_source: mizar_session::NormalizedPath,
    source_uri: String,
}

impl Fixture {
    fn get() -> &'static Self {
        static FIXTURE: OnceLock<Fixture> = OnceLock::new();
        FIXTURE.get_or_init(Self::new)
    }

    fn new() -> Self {
        let package_root = std::env::temp_dir().join("mizar-parser-fuzz-fixture");
        let source_path = package_root.join("src").join("fuzz.miz");
        fs::create_dir_all(
            source_path
                .parent()
                .expect("fixture source should have a parent directory"),
        )
        .expect("fuzz fixture directory should be creatable");
        fs::write(&source_path, b"").expect("fuzz fixture file should be creatable");

        let normalized_source =
            normalize_path(&package_root, &source_path).expect("fuzz fixture should normalize");
        let source_uri = format!("file://{}", source_path.display());

        Self {
            package_root,
            normalized_source,
            source_uri,
        }
    }

    fn request(&self, source: &str) -> SourceUnitRequest {
        SourceUnitRequest {
            snapshot: snapshot_id(),
            input: SourceInput {
                package_id: PackageId::new("fuzz"),
                module_path: ModulePath::new("fuzz"),
                normalized_path: self.normalized_source.clone(),
                edition: Edition::new("2026"),
                origin: SourceOriginInput::OpenBuffer {
                    uri: self.source_uri.clone(),
                    expected_version: LspDocumentVersion::from(1),
                    actual_version: LspDocumentVersion::from(1),
                    text: Arc::<str>::from(source),
                },
            },
        }
    }
}

fn snapshot_id() -> BuildSnapshotId {
    BuildSnapshotId::from_published_schema_str(
        "mizar-session-build-snapshot-v1:0000000000000000000000000000000000000000000000000000000000000002",
    )
    .expect("static fuzz snapshot id should parse")
}

fn lexical_activation_source_offset(
    source_id: mizar_session::SourceId,
    bridge: &SpanBridge,
    position: usize,
) -> Result<usize, SpanBridgeError> {
    if position == 0 {
        return Ok(0);
    }
    bridge
        .lexical_span(
            source_id,
            LexerByteSpan {
                start: position,
                end: position,
            },
        )
        .map(|mapped| mapped.primary.start)
}

fn assert_source_range(bridge: &SpanBridge, range: SourceRange) {
    bridge
        .source_map_service()
        .validate_range(range)
        .expect("fuzz diagnostic and AST ranges should stay valid");
}
