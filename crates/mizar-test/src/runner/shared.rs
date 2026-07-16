use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mizar_frontend::orchestration::{Frontend, FrontendDiagnostic};
use mizar_frontend::parsing::MizarParserSeam;
use mizar_frontend::source::{FrontendSourceLoader, SourceUnitRequest};
use mizar_resolve::declarations::DeclarationShellCollector;
use mizar_resolve::env::{NamespacePath, SymbolEnv};
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_resolve::symbols::{
    SignatureProjectionExtractor, SymbolCollector, SymbolDiagnostic, SymbolDiagnosticClass,
};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceInput, SourceOriginInput, normalize_path,
};
use mizar_syntax::SurfaceAst;

use crate::harness::{DiscoveryConfig, HarnessError, TestCase};
use crate::path_rules::absolute_from;

use super::ParseOnlyImportProvider;

pub(super) fn run_frontend(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<FrontendRun, String> {
    let prepared = prepare_source_package(workspace_root, case, ordinal)?;
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&prepared.package_root)),
        ParseOnlyImportProvider,
        MizarParserSeam,
    );
    let ids = InMemorySessionIdAllocator::new();
    let output = frontend
        .run(prepared.request.clone(), &ids)
        .map_err(|error| error.to_string())?;
    let ast_snapshot = output.ast.as_ref().map(|ast| ast.snapshot_text());
    Ok(FrontendRun {
        ast: output.ast,
        ast_snapshot,
        diagnostics: output.diagnostics,
    })
}

#[derive(Debug)]
pub(super) struct FrontendRun {
    pub(super) ast: Option<SurfaceAst>,
    pub(super) ast_snapshot: Option<String>,
    pub(super) diagnostics: Vec<FrontendDiagnostic>,
}

#[derive(Debug)]
pub(super) struct ResolverSymbolCollection {
    pub(super) module: ResolverModuleId,
    pub(super) env: SymbolEnv,
    pub(super) detail_keys: Vec<String>,
}

pub(super) fn resolver_symbol_collection(
    workspace_root: &Path,
    case: &TestCase,
    ast: &SurfaceAst,
) -> ResolverSymbolCollection {
    let module = resolver_module_id(workspace_root, &case.source_path);
    let namespace = NamespacePath::new(module.path().as_str());
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let projections = SignatureProjectionExtractor::new(ast, &shells, namespace).extract();
    let result = SymbolCollector::new(ast.source_id, &module, &shells, &projections).collect();

    let detail_keys = result
        .diagnostics()
        .iter()
        .map(symbol_diagnostic_detail_key)
        .collect();
    ResolverSymbolCollection {
        module,
        env: result.into_env(),
        detail_keys,
    }
}

fn prepare_source_package(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<PreparedSource, String> {
    let source_path = &case.source_path;
    let text = fs::read_to_string(source_path)
        .map_err(|error| format!("failed to read source `{}`: {error}", source_path.display()))?;
    let package_root = temp_package_root(ordinal);
    let temp_source = package_root
        .join("src")
        .join(source_path.file_name().unwrap_or_default());
    fs::create_dir_all(temp_source.parent().unwrap_or(&package_root))
        .map_err(|error| format!("failed to create corpus temp package: {error}"))?;
    fs::write(&temp_source, text)
        .map_err(|error| format!("failed to write corpus temp source: {error}"))?;
    let normalized_path = normalize_path(&package_root, &temp_source)
        .map_err(|error| format!("failed to normalize temp source path: {error}"))?;

    Ok(PreparedSource {
        package_root: package_root.clone(),
        request: SourceUnitRequest {
            snapshot: snapshot_id(ordinal),
            input: SourceInput {
                package_id: PackageId::new("mizar-test-corpus"),
                module_path: ModulePath::new(module_path(workspace_root, source_path)),
                normalized_path,
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path: temp_source },
            },
        },
    })
}

#[derive(Debug)]
struct PreparedSource {
    package_root: PathBuf,
    request: SourceUnitRequest,
}

impl Drop for PreparedSource {
    fn drop(&mut self) {
        match fs::remove_dir_all(&self.package_root) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }
}

fn temp_package_root(ordinal: usize) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "mizar-test-corpus-{}-{ordinal}-{nanos}",
        std::process::id()
    ))
}

pub(super) fn normalized_workspace_root(config: &DiscoveryConfig) -> Result<PathBuf, HarnessError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        HarnessError::Infrastructure(format!("failed to read current directory: {error}"))
    })?;
    Ok(absolute_from(&current_dir, &config.workspace_root))
}

pub(super) fn normalized_tests_root(workspace_root: &Path, config: &DiscoveryConfig) -> PathBuf {
    absolute_from(workspace_root, &config.tests_root)
}

pub(super) fn module_path(workspace_root: &Path, source_path: &Path) -> String {
    source_path
        .strip_prefix(workspace_root)
        .unwrap_or(source_path)
        .with_extension("")
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join(".")
}

fn resolver_module_id(workspace_root: &Path, source_path: &Path) -> ResolverModuleId {
    ResolverModuleId::new(
        PackageId::new("mizar-test-corpus"),
        ModulePath::new(module_path(workspace_root, source_path)),
    )
}

fn symbol_diagnostic_detail_key(diagnostic: &SymbolDiagnostic) -> String {
    match diagnostic.class() {
        SymbolDiagnosticClass::SameSignatureReturnConflict => {
            "declaration_symbol.signature.same_signature_return_conflict".to_owned()
        }
        class => format!(
            "declaration_symbol.symbol.{}",
            symbol_diagnostic_class_key(class)
        ),
    }
}

const fn symbol_diagnostic_class_key(class: SymbolDiagnosticClass) -> &'static str {
    match class {
        SymbolDiagnosticClass::MissingShell => "missing_shell",
        SymbolDiagnosticClass::ContextOnlyShell => "context_only_shell",
        SymbolDiagnosticClass::DuplicateDeclaration => "duplicate_declaration",
        SymbolDiagnosticClass::IllegalOverloadGroup => "illegal_overload_group",
        _ => "unknown",
    }
}

fn snapshot_id(ordinal: usize) -> BuildSnapshotId {
    BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{:064x}",
        ordinal + 1
    ))
    .expect("static parse-only snapshot id should be valid")
}
