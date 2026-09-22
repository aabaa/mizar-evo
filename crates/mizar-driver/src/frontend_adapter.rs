//! Real disk source publication and dependency lexical input.
//! See [the source service contract](../../../../doc/design/mizar-driver/en/frontend_adapter.md).
use std::{path::PathBuf, sync::Arc};

use mizar_build::{
    module_index::ModuleIndexLocation,
    planner::PackagePlanSource,
    task_graph::{PipelinePhase, WorkUnit},
};
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput, DiagnosticPrimaryLocation,
        FailureCategory, PipelinePhase as DiagnosticPhase,
    },
    registry::{DiagnosticCode, DiagnosticSeverity},
};
use mizar_frontend::{
    cache_key::{SOURCE_UNIT_CACHE_KEY_VERSION, SourceUnitCacheKey},
    source::{FrontendSourceLoader, SourceUnit, SourceUnitLoader, SourceUnitRequest},
};
use mizar_ir::{
    identity::{
        NamedInputHash, OutputKind, PipelinePhase as IrPipelinePhase, WorkUnit as IrWorkUnit,
    },
    publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
    storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion, SideTableRecord},
};
use mizar_session::{
    DiskSourceLoader, Hash, SourceInput, SourceLoadError, SourceOrigin, SourceOriginInput,
};

use crate::registry::{
    PhaseCacheContext, PhaseCacheIntent, PhaseDescriptor, PhaseExecutionContext, PhaseInput,
    PhaseOwner, PhaseResult, PhaseService, PhaseStatus,
};

impl<'a> crate::registry::SourceLoadInputs<'a> {
    /// Resolves real dependency lexical summaries under explicit caller-owned roots.
    /// Requires build-validated indexes and stubs from the captured source version.
    pub fn dependency_lexical_provider(
        self,
        artifact_roots: &'a [(mizar_session::PackageId, PathBuf)],
    ) -> impl mizar_frontend::lexical_env::LexicalSummaryProvider + 'a {
        DependencyLexicalProvider {
            inputs: self,
            artifact_roots,
        }
    }
}

struct DependencyLexicalProvider<'a> {
    inputs: crate::registry::SourceLoadInputs<'a>,
    artifact_roots: &'a [(mizar_session::PackageId, PathBuf)],
}

impl mizar_frontend::lexical_env::LexicalSummaryProvider for DependencyLexicalProvider<'_> {
    fn resolve_imports(
        &self,
        request: &mizar_frontend::lexical_env::LexicalEnvironmentRequest<'_>,
    ) -> Result<
        mizar_frontend::lexical_env::ResolvedImports,
        mizar_frontend::lexical_env::FrontendLexicalEnvironmentError,
    > {
        use mizar_frontend::lexical_env::{
            FrontendLexicalEnvironmentError, ModuleId, ResolvedImport, ResolvedImportEntry,
            ResolvedImports,
        };
        use mizar_resolve::{
            imports::{ImportPathCandidate, ImportPathResolver},
            module_index::{IndexedModuleId, ModuleIndexInput},
            module_summary_reuse::{ModuleSummaryReuse, ModuleSummaryReuseRequest},
        };
        let unavailable = || FrontendLexicalEnvironmentError::ProviderUnavailable {
            message: "dependency lexical provider binding or payload unavailable".to_owned(),
        };
        let version = unique(
            self.inputs
                .snapshot
                .source_versions
                .iter()
                .filter(|version| version.source_id == request.source_id),
        )
        .ok_or_else(unavailable)?;
        let current = unique(self.inputs.module_index.modules.iter().filter(|entry| {
            entry.module.package == version.package_id && entry.module.path == version.module_path
        }))
        .ok_or_else(unavailable)?;
        if version.edition != request.edition
            || current.edition != request.edition
            || current.package_id != version.package_id
            || current.module_path != version.module_path
        {
            return Err(unavailable());
        }
        let index = ModuleIndexInput::new(self.inputs.module_index);
        let candidates =
            ImportPathCandidate::from_frontend_imports(request).ok_or_else(unavailable)?;
        let resolution = ImportPathResolver::new(index)
            .resolve(&index.resolver_module_id(&current.module), &candidates);
        let mut result = ResolvedImports {
            imports: Vec::new(),
            summaries: Vec::new(),
            diagnostics: Vec::new(),
        };
        for import in resolution.resolved() {
            let module = IndexedModuleId::new(
                import.target().package().clone(),
                import.target().path().clone(),
            );
            let entry = unique(
                self.inputs
                    .module_index
                    .modules
                    .iter()
                    .filter(|entry| entry.module == module),
            )
            .ok_or_else(unavailable)?;
            let ModuleIndexLocation::DependencySummary {
                artifact,
                content_hash,
            } = &entry.location
            else {
                return Err(unavailable());
            };
            let reference = unique(
                self.inputs
                    .module_index
                    .dependency_summaries
                    .iter()
                    .filter(|reference| reference.module == module),
            )
            .ok_or_else(unavailable)?;
            if entry.package_id != module.package
                || entry.module_path != module.path
                || artifact != &reference.artifact
                || content_hash != &reference.content_hash
            {
                return Err(unavailable());
            }
            let (_, root) = unique(
                self.artifact_roots
                    .iter()
                    .filter(|(package, _)| package == &module.package),
            )
            .ok_or_else(unavailable)?;
            let module_id = if let Some(value) = reference.read_current_summary(root) {
                let summary = ModuleSummaryReuse::new(index)
                    .read_lexical_summary(
                        ModuleSummaryReuseRequest::new(
                            &module,
                            mizar_session::SourceAnchor::Range(import.range()),
                        ),
                        &value,
                    )
                    .ok_or_else(unavailable)?;
                let id = summary.module_id.clone();
                result.summaries.push(summary);
                id
            } else {
                ModuleId::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                ))
            };
            result.imports.push(ResolvedImportEntry {
                stub_ordinal: import.ordinal(),
                stub_span: import.range(),
                import: ResolvedImport { module_id },
            });
        }
        Ok(result)
    }
}

pub(crate) struct SourceLoadService;

impl PhaseService for SourceLoadService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceLoad",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::SourceLoad],
            "mizar-frontend/source-unit-publication/v1",
            "SourceUnit",
        )
        .expect("source-load descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "current disk source maps require a real source load".to_owned(),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext<'_>) -> PhaseResult {
        let snapshot = input.snapshot;
        let Some(mut diagnostics) = context.diagnostics else {
            return blocking();
        };
        if diagnostics.is_sealed()
            || !diagnostics.drafts().is_empty()
            || diagnostics.scope().phase() != DiagnosticPhase::SourceLoad
            || diagnostics.scope().source_snapshot() != snapshot
        {
            return blocking();
        }
        if let Some(token) = context.cancellation {
            return if token.snapshot == snapshot {
                PhaseResult {
                    status: PhaseStatus::Cancelled,
                    ..blocking()
                }
            } else {
                blocking()
            };
        }
        let Some(publisher) = context.output_publisher else {
            return blocking();
        };
        if publisher.validate_current_snapshot(snapshot).is_err() {
            return blocking();
        }
        let Some(source_inputs) = context.source_load else {
            return blocking();
        };
        if source_inputs.snapshot.id != snapshot
            || source_inputs.snapshot.workspace_root != source_inputs.build_plan.workspace_root
        {
            return blocking();
        }
        if !input.parent_outputs().is_empty() || !input.identities().dependency_hashes().is_empty()
        {
            return blocking();
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return blocking();
        };
        let Some(version) = unique(source_inputs.snapshot.source_versions.iter().filter(
            |version| version.package_id == module.package && version.module_path == module.path,
        )) else {
            return blocking();
        };
        if !matches!(version.origin, SourceOrigin::Disk) {
            return blocking();
        }
        let Some(package) = unique(
            source_inputs
                .build_plan
                .packages
                .iter()
                .filter(|package| package.package_id == module.package),
        ) else {
            return blocking();
        };
        let PackagePlanSource::Workspace {
            root,
            source_root,
            manifest_path,
        } = &package.source
        else {
            return blocking();
        };
        let Some(index_package) = unique(
            source_inputs
                .module_index
                .packages
                .iter()
                .filter(|entry| entry.package_id == module.package),
        ) else {
            return blocking();
        };
        let mizar_build::module_index::PackageIndexSource::Workspace {
            package_root: index_root,
            source_root: index_source_root,
            manifest_path: index_manifest,
        } = &index_package.source
        else {
            return blocking();
        };
        if root != index_root
            || source_root != index_source_root
            || manifest_path != index_manifest
            || package.edition != version.edition
            || index_package.edition != version.edition
            || !safe_relative(root)
            || !safe_relative(source_root)
            || !safe_relative(manifest_path)
        {
            return blocking();
        }
        let Some(entry) = unique(
            source_inputs
                .module_index
                .modules
                .iter()
                .filter(|entry| entry.module == *module),
        ) else {
            return blocking();
        };
        let ModuleIndexLocation::WorkspaceFile {
            source_root: entry_source_root,
            normalized_path,
            source_relative_path,
        } = &entry.location
        else {
            return blocking();
        };
        if entry.package_id != module.package
            || entry.module_path != module.path
            || entry.edition != version.edition
            || entry_source_root != source_root
            || normalized_path != version.normalized_path.as_str()
            || normalized_path != &format!("src/{source_relative_path}")
            || source_relative_path.is_empty()
        {
            return blocking();
        }
        let workspace = PathBuf::from(source_inputs.snapshot.workspace_root.as_str());
        let package_root = workspace.join(root);
        let source_root_path = workspace.join(source_root);
        if source_root_path != package_root.join("src")
            || workspace.join(manifest_path) != package_root.join("mizar.pkg")
        {
            return blocking();
        }

        let source_key = SourceUnitCacheKey {
            version: Arc::from(SOURCE_UNIT_CACHE_KEY_VERSION),
            package_id: version.package_id.clone(),
            module_path: version.module_path.clone(),
            normalized_path: version.normalized_path.clone(),
            source_hash: version.source_hash,
            edition: version.edition.clone(),
        }
        .stable_hash();
        if input.identities().input_hash() != source_key {
            return blocking();
        }
        let source_input = SourceInput {
            package_id: version.package_id.clone(),
            module_path: version.module_path.clone(),
            normalized_path: version.normalized_path.clone(),
            edition: version.edition.clone(),
            origin: SourceOriginInput::Disk {
                path: version.normalized_path.as_str().into(),
            },
        };
        let bound_root = match (workspace.canonicalize(), package_root.canonicalize()) {
            (Ok(workspace), Ok(root)) if root.starts_with(&workspace) => Some(root),
            (Ok(_), Ok(_)) => return blocking(),
            _ => None,
        };
        let loader = FrontendSourceLoader::new(DiskSourceLoader::new(
            bound_root.as_ref().unwrap_or(&package_root),
        ));
        let loaded = match loader.load_source_unit(
            SourceUnitRequest {
                snapshot,
                input: source_input.clone(),
            },
            source_inputs.allocator,
        ) {
            Ok(loaded) => loaded,
            Err(error) => {
                let (number, name) = match &error {
                    SourceLoadError::InvalidUtf8 { .. } => (601, "invalid_utf8"),
                    SourceLoadError::UnreadableSourceFile { .. } => (602, "unreadable_file"),
                    SourceLoadError::SourcePathOutsidePackageRoot { .. } => {
                        (603, "outside_package_root")
                    }
                    SourceLoadError::UnsupportedFileExtension { .. } => {
                        (600, "unsupported_file_extension")
                    }
                    SourceLoadError::DuplicateModulePath { .. } => (600, "duplicate_module_path"),
                    SourceLoadError::StaleLspDocumentVersion { .. } => {
                        (600, "stale_lsp_document_version")
                    }
                    SourceLoadError::UnmappedOpenBufferUri { .. } => {
                        (600, "unmapped_open_buffer_uri")
                    }
                    SourceLoadError::GeneratedSourceWithoutMetadata { .. } => {
                        (600, "generated_source_without_metadata")
                    }
                    SourceLoadError::SourceIdAllocation { .. } => (600, "source_id_allocation"),
                    SourceLoadError::UnsupportedSourceOrigin { .. } => {
                        (600, "unsupported_source_origin")
                    }
                    SourceLoadError::InvalidSourcePath { .. } => (600, "invalid_source_path"),
                    _ => return blocking(),
                };
                let detail = format!("source.{name}");
                let code = DiagnosticCode::from_parts(DiagnosticSeverity::Error, number)
                    .expect("source-load diagnostic code is valid");
                let draft = DiagnosticDraft::new(DiagnosticDraftInput {
                    source_snapshot: snapshot,
                    code,
                    phase: DiagnosticPhase::SourceLoad,
                    category: FailureCategory::SourceLoadError,
                    stable_detail_key: detail,
                    message: error.to_string(),
                    primary_location: DiagnosticPrimaryLocation::SourceLoad {
                        package_id: source_input.package_id.clone(),
                        path: source_input.normalized_path.clone(),
                    },
                    secondary_spans: Vec::new(),
                    notes: Vec::new(),
                    details: DiagnosticDetails::new(),
                    fixes: Vec::new(),
                    explanation: None,
                });
                let Ok(draft) = draft else {
                    return blocking();
                };
                if diagnostics.emit(draft).is_err() {
                    return blocking();
                }
                return PhaseResult {
                    status: PhaseStatus::Fatal,
                    diagnostics: vec![diagnostics.seal()],
                    ..blocking()
                };
            }
        };
        if bound_root.is_none()
            || loaded.package_id != version.package_id
            || loaded.module_path != version.module_path
            || loaded.normalized_path != version.normalized_path
            || loaded.edition != version.edition
            || loaded.source_hash != version.source_hash
            || loaded.origin != SourceOrigin::Disk
        {
            return blocking();
        }
        let Some(storage_bytes) = loaded.canonical_disk_bytes() else {
            return blocking();
        };
        let Some(source) =
            SourceUnit::from_canonical_disk_bytes(&storage_bytes, version.source_id, &source_input)
        else {
            return blocking();
        };
        let semantic_bytes = source_key.as_bytes().to_vec();
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-frontend/source-unit-storage-side-table/v1",
            &storage_bytes,
        ));
        let phase = IrPipelinePhase::new("SourceLoad");
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            source.package_id.as_str(),
            source.module_path.as_str()
        ));
        let output_kind = OutputKind::new("SourceUnit");
        let decode_input = source_input;
        let source_id = source.source_id;
        let expected_hash = version.source_hash;
        let handle = publisher.publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot,
                phase.clone(),
                work_unit.clone(),
                output_kind.clone(),
                SchemaVersion::new(1),
            ),
            snapshot,
            phase,
            work_unit,
            output_kind,
            schema_version: SchemaVersion::new(1),
            payload: source,
            canonical_payload: Some(semantic_bytes),
            storage_payload: Some(storage_bytes),
            decode: BlobDecoder::new(move |bytes| {
                SourceUnit::from_canonical_disk_bytes(bytes, source_id, &decode_input)
                    .filter(|source| source.source_hash == expected_hash)
                    .ok_or_else(|| BlobDecodeError::new("invalid source-unit disk payload"))
            }),
            parents: Vec::new(),
            named_input_hashes: vec![NamedInputHash {
                name: "source".to_owned(),
                domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                digest: source_key,
            }],
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "source-storage",
                    version.normalized_path.as_str(),
                    storage_hash,
                )],
                ..IrSideTables::default()
            },
            origin: OutputOrigin::PackageSource,
            target: PublicationTarget::CurrentPackage,
        });
        match handle {
            Ok(handle) => PhaseResult {
                output_refs: vec![handle.erase()],
                ..PhaseResult::complete()
            },
            Err(_) => blocking(),
        }
    }
}

fn unique<T>(mut items: impl Iterator<Item = T>) -> Option<T> {
    let item = items.next()?;
    items.next().is_none().then_some(item)
}

fn safe_relative(path: &str) -> bool {
    path == "."
        || (!path.is_empty()
            && !path.contains(['\\', ':'])
            && path
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != ".."))
}

fn blocking() -> PhaseResult {
    PhaseResult {
        status: PhaseStatus::Blocking,
        ..PhaseResult::complete()
    }
}
