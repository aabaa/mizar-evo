use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mizar_frontend::cache_key::FrontendCacheKeys;
use mizar_frontend::lexical_env::{
    ExportRank, ExportedSymbolShape, FrontendLexicalEnvironmentError, LexicalEnvironmentRequest,
    LexicalSummaryFingerprint, LexicalSummaryProvider, ModuleId, ModuleLexicalSummary,
    ResolvedImport, ResolvedImportEntry, ResolvedImports, SymbolId, UserSymbolArity,
    UserSymbolKind,
};
use mizar_frontend::orchestration::{Frontend, FrontendOutput};
use mizar_frontend::parsing::MizarParserSeam;
use mizar_frontend::source::{FrontendSourceLoader, SourceUnitRequest};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceInput, SourceOriginInput, normalize_path,
};
use std::fs;
use std::hint::black_box;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

fn frontend_pipeline(c: &mut Criterion) {
    let fixture = PackageFixture::new();
    let baseline_source = large_miz_like_source(1_024, "alpha, beta");
    let comment_edit_source = large_miz_like_source_with_comment(1_024, "alpha, beta", "edited");
    let import_edit_source = large_miz_like_source(1_024, "beta");

    fixture.write("src/baseline.miz", &baseline_source);
    fixture.write("src/comment_edit.miz", &comment_edit_source);
    fixture.write("src/import_edit.miz", &import_edit_source);

    let mut group = c.benchmark_group("frontend_pipeline_large_miz_like");

    group.throughput(Throughput::Bytes(baseline_source.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("cold_full_pipeline", baseline_source.len()),
        &fixture.request("src/baseline.miz"),
        |b, request| {
            b.iter(|| run_frontend(black_box(&fixture), black_box(request.clone())).cache_keys);
        },
    );

    group.throughput(Throughput::Bytes(comment_edit_source.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("comment_only_edit_full_pipeline", comment_edit_source.len()),
        &fixture.request("src/comment_edit.miz"),
        |b, request| {
            b.iter(|| {
                consume_cache_keys(run_frontend(
                    black_box(&fixture),
                    black_box(request.clone()),
                ))
            });
        },
    );

    group.throughput(Throughput::Bytes(import_edit_source.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("import_edit_full_pipeline", import_edit_source.len()),
        &fixture.request("src/import_edit.miz"),
        |b, request| {
            b.iter(|| {
                consume_cache_keys(run_frontend(
                    black_box(&fixture),
                    black_box(request.clone()),
                ))
            });
        },
    );

    group.finish();
}

fn run_frontend(
    fixture: &PackageFixture,
    request: SourceUnitRequest,
) -> FrontendOutput<mizar_syntax::SurfaceAst> {
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root())),
        BenchmarkSummaryProvider,
        MizarParserSeam,
    );

    frontend
        .run(request, &InMemorySessionIdAllocator::new())
        .expect("benchmark fixture should produce recovered frontend output")
}

fn consume_cache_keys(output: FrontendOutput<mizar_syntax::SurfaceAst>) -> FrontendCacheKeys {
    black_box(output.cache_keys)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BenchmarkSummaryProvider;

impl LexicalSummaryProvider for BenchmarkSummaryProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        let mut imports = Vec::new();
        for (stub_ordinal, stub) in request.import_stubs.iter().enumerate() {
            if matches!(stub.path.spelling.as_ref(), "alpha" | "beta") {
                imports.push(ResolvedImportEntry {
                    stub_ordinal,
                    stub_span: stub.span,
                    import: ResolvedImport {
                        module_id: ModuleId::new(stub.path.spelling.as_ref()),
                    },
                });
            }
        }

        Ok(ResolvedImports {
            imports,
            summaries: vec![
                summary(
                    "alpha",
                    101,
                    vec![symbol("+*+", "alpha.plus_star_plus", "alpha")],
                ),
                summary("beta", 202, vec![symbol("**", "beta.times", "beta")]),
            ],
            diagnostics: Vec::new(),
        })
    }
}

fn summary(
    module_id: &str,
    fingerprint: u64,
    exported_symbols: Vec<ExportedSymbolShape>,
) -> ModuleLexicalSummary {
    ModuleLexicalSummary {
        module_id: ModuleId::new(module_id),
        exported_symbols,
        fingerprint: LexicalSummaryFingerprint::new(fingerprint),
    }
}

fn symbol(spelling: &str, symbol_id: &str, source_module: &str) -> ExportedSymbolShape {
    ExportedSymbolShape {
        spelling: spelling.to_owned(),
        symbol_id: SymbolId::new(symbol_id),
        source_module: ModuleId::new(source_module),
        export_rank: ExportRank::new(0),
        kind: UserSymbolKind::Functor,
        arity: UserSymbolArity::exact(2),
        operator: None,
    }
}

fn large_miz_like_source(items: usize, imports: &str) -> String {
    large_miz_like_source_with_comment(items, imports, "baseline")
}

fn large_miz_like_source_with_comment(items: usize, imports: &str, comment: &str) -> String {
    let mut source = String::with_capacity(items * 112);
    source.push_str(":: frontend benchmark ");
    source.push_str(comment);
    source.push('\n');
    source.push_str("import ");
    source.push_str(imports);
    source.push_str(";\n\n");

    for index in 0..items {
        source.push_str("theorem th");
        source.push_str(&index.to_string());
        source.push_str(":\n");
        source.push_str("  for x,y being set holds x = x & y = y\n");
        source.push_str("proof\n");
        source.push_str("  thus x = x;\n");
        source.push_str("  thus y = y;\n");
        if index % 16 == 0 {
            source.push_str("  :: comment-only churn candidate\n");
        }
        source.push_str("end;\n\n");
    }

    source
}

struct PackageFixture {
    root: PathBuf,
}

impl PackageFixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "mizar-frontend-pipeline-bench-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("src")).expect("benchmark fixture root should be creatable");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("benchmark fixture directory should be creatable");
        }
        fs::write(path, text).expect("benchmark fixture source should be writable");
    }

    fn request(&self, relative: &str) -> SourceUnitRequest {
        let path = self.path(relative);
        SourceUnitRequest {
            snapshot: snapshot_id(),
            input: SourceInput {
                package_id: PackageId::new("bench"),
                module_path: ModulePath::new(module_path_from_relative(relative)),
                normalized_path: normalize_path(&self.root, &path)
                    .expect("benchmark fixture path should normalize"),
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path },
            },
        }
    }
}

impl Drop for PackageFixture {
    fn drop(&mut self) {
        match fs::remove_dir_all(&self.root) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }
}

fn snapshot_id() -> BuildSnapshotId {
    BuildSnapshotId::from_published_schema_str(
        "mizar-session-build-snapshot-v1:0000000000000000000000000000000000000000000000000000000000000001",
    )
    .expect("static benchmark snapshot id should parse")
}

fn module_path_from_relative(relative: &str) -> String {
    relative
        .strip_prefix("src/")
        .unwrap_or(relative)
        .strip_suffix(".miz")
        .unwrap_or(relative)
        .replace('/', ".")
}

criterion_group!(benches, frontend_pipeline);
criterion_main!(benches);
