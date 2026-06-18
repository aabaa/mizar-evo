#![no_main]

use libfuzzer_sys::fuzz_target;
use mizar_frontend::lexical_env::{
    FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
    ResolvedImports,
};
use mizar_frontend::orchestration::{
    DiagnosticClass, DiagnosticCode, DiagnosticLocation, Frontend, FrontendError,
};
use mizar_frontend::parsing::{MIZAR_PARSER_CACHE_KEY_VERSION, MizarParserSeam};
use mizar_frontend::source::{FrontendSourceLoader, SourceUnitRequest};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, LspDocumentVersion,
    ModulePath, PackageId, SourceInput, SourceOriginInput, normalize_path,
};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

fuzz_target!(|source: &str| {
    let fixture = Fixture::get();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&fixture.package_root)),
        EmptySummaryProvider,
        MizarParserSeam,
    );

    let output = frontend.run(fixture.request(source), &InMemorySessionIdAllocator::new());
    assert_recovered_frontend_output(output);
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
        let package_root = std::env::temp_dir().join("mizar-frontend-fuzz-fixture");
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
        "mizar-session-build-snapshot-v1:0000000000000000000000000000000000000000000000000000000000000001",
    )
    .expect("static fuzz snapshot id should parse")
}

fn assert_recovered_frontend_output<A>(
    output: Result<mizar_frontend::orchestration::FrontendOutput<A>, FrontendError>,
) {
    let output = output.expect("frontend valid-UTF-8 fuzz input should recover without hard error");
    assert_eq!(output.source.source_id, output.preprocessed.source_id);
    assert_eq!(
        output.preprocessed.lexical_text.as_str().len(),
        output.preprocessed.source_map.lexical_len()
    );
    assert_eq!(output.ast.is_some(), output.cache_keys.ast.is_some());
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| matches!(diagnostic.location, DiagnosticLocation::SourceRange(_)))
    );
    assert!(output.diagnostics.iter().all(|diagnostic| {
        !matches!(diagnostic.code, DiagnosticCode::Syntax(_))
            || diagnostic.class == DiagnosticClass::Syntax
    }));

    if let Some(ast_key) = &output.cache_keys.ast {
        assert_eq!(
            ast_key.parser_version.version.as_ref(),
            MIZAR_PARSER_CACHE_KEY_VERSION
        );
        assert_eq!(
            ast_key.token_stream_hash,
            output.cache_keys.tokens.stable_hash()
        );
        let _ = ast_key.stable_hash();
    }
}
