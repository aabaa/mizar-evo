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
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticNote, DiagnosticNoteKind, DiagnosticPrimaryLocation, DiagnosticSpan,
        DiagnosticSpanRole, FailureCategory, PipelinePhase as DiagnosticPhase, SpanFreshness,
    },
    registry::{DiagnosticCode, DiagnosticSeverity},
};
use mizar_frontend::{
    cache_key::{
        ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION, SOURCE_UNIT_CACHE_KEY_VERSION,
        SourceUnitCacheKey, TOKEN_STREAM_CACHE_KEY_VERSION,
    },
    lexical_env::{
        ExportedSymbolShape, LexicalEnvironmentDiagnosticCode as EnvironmentCode,
        ModuleLexicalSummary,
    },
    lexing::{LexDiagnosticCode, LexingDiagnosticKind, ScopeSkeletonDiagnosticCode},
    orchestration::{
        DiagnosticClass, DiagnosticCode as FrontendCode, DiagnosticLocation, Frontend,
        FrontendDiagnostic, FrontendOutput,
    },
    parsing::{MizarParserSeam, ParserSeam},
    preprocess::{
        ImportPrescanDiagnosticCode, PreprocessDiagnosticKind, SourcePreprocessDiagnosticCode,
    },
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
    DiskSourceLoader, Hash, PackageId, SourceInput, SourceLoadError, SourceOrigin,
    SourceOriginInput,
};

use crate::registry::{
    PhaseCacheContext, PhaseCacheIntent, PhaseDescriptor, PhaseExecutionContext, PhaseInput,
    PhaseOwner, PhaseResult, PhaseService, PhaseStatus,
};

pub(crate) fn source_input_hash(version: &mizar_session::SourceVersion) -> Hash {
    SourceUnitCacheKey {
        version: Arc::from(SOURCE_UNIT_CACHE_KEY_VERSION),
        package_id: version.package_id.clone(),
        module_path: version.module_path.clone(),
        normalized_path: version.normalized_path.clone(),
        source_hash: version.source_hash,
        edition: version.edition.clone(),
    }
    .stable_hash()
}

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
            workspace_summaries: &[],
        }
    }
}

struct DependencyLexicalProvider<'a> {
    inputs: crate::registry::SourceLoadInputs<'a>,
    artifact_roots: &'a [(mizar_session::PackageId, PathBuf)],
    workspace_summaries: &'a [(
        mizar_resolve::module_index::IndexedModuleId,
        ModuleLexicalSummary,
    )],
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
            if let ModuleIndexLocation::WorkspaceFile { .. } = &entry.location {
                let (_, summary) = unique(
                    self.workspace_summaries
                        .iter()
                        .filter(|(target, _)| target == &module),
                )
                .ok_or_else(unavailable)?;
                if module.package != current.module.package
                    || entry.package_id != module.package
                    || entry.module_path != module.path
                {
                    return Err(unavailable());
                }
                let module_id = summary.module_id.clone();
                result.summaries.push(summary.clone());
                result.imports.push(ResolvedImportEntry {
                    stub_ordinal: import.ordinal(),
                    stub_span: import.range(),
                    import: ResolvedImport { module_id },
                });
                continue;
            }
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

        let source_key = source_input_hash(version);
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

pub(crate) struct FrontendService {
    pub(crate) artifact_roots: Vec<(PackageId, PathBuf)>,
}

impl PhaseService for FrontendService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "Frontend",
            PhaseOwner::MizarFrontend,
            vec![PipelinePhase::Frontend],
            "mizar-frontend/output-publication/v1",
            "FrontendOutput",
        )
        .expect("frontend descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "frontend inputs require a live source and dependency run".to_owned(),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext<'_>) -> PhaseResult {
        let snapshot = input.snapshot;
        let Some(mut sink) = context.diagnostics else {
            return blocking();
        };
        if sink.is_sealed()
            || !sink.drafts().is_empty()
            || sink.scope().phase() != DiagnosticPhase::Frontend
            || sink.scope().source_snapshot() != snapshot
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
        let Some(inputs) = context.source_load else {
            return blocking();
        };
        if inputs.snapshot.id != snapshot
            || inputs.snapshot.workspace_root != inputs.build_plan.workspace_root
        {
            return blocking();
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return blocking();
        };
        let Some(version) = unique(inputs.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        })) else {
            return blocking();
        };
        if version.origin != SourceOrigin::Disk {
            return blocking();
        }
        let Some(package) = unique(
            inputs
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
            inputs
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
            inputs
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
        let workspace = PathBuf::from(inputs.snapshot.workspace_root.as_str());
        let package_root = workspace.join(root);
        if workspace.join(source_root) != package_root.join("src")
            || workspace.join(manifest_path) != package_root.join("mizar.pkg")
        {
            return blocking();
        }
        let source_key = source_input_hash(version);
        for indexed in &inputs.module_index.modules {
            let ModuleIndexLocation::DependencySummary {
                artifact,
                content_hash,
            } = &indexed.location
            else {
                continue;
            };
            let Some(reference) = unique(
                inputs
                    .module_index
                    .dependency_summaries
                    .iter()
                    .filter(|reference| reference.module == indexed.module),
            ) else {
                return blocking();
            };
            if reference.artifact != *artifact
                || reference.content_hash != *content_hash
                || unique(
                    inputs
                        .snapshot
                        .dependency_artifacts
                        .iter()
                        .filter(|captured| {
                            captured.artifact == *artifact && captured.content_hash == *content_hash
                        }),
                )
                .is_none()
            {
                return blocking();
            }
        }
        for reference in &inputs.module_index.dependency_summaries {
            let Some(indexed) = unique(
                inputs
                    .module_index
                    .modules
                    .iter()
                    .filter(|indexed| indexed.module == reference.module),
            ) else {
                return blocking();
            };
            if !matches!(&indexed.location,
                ModuleIndexLocation::DependencySummary { artifact, content_hash }
                if artifact == &reference.artifact && content_hash == &reference.content_hash)
                || unique(
                    inputs
                        .snapshot
                        .dependency_artifacts
                        .iter()
                        .filter(|captured| {
                            captured.artifact == reference.artifact
                                && captured.content_hash == reference.content_hash
                        }),
                )
                .is_none()
            {
                return blocking();
            }
        }
        let mut dependencies = inputs
            .module_index
            .dependency_summaries
            .iter()
            .map(|reference| reference.content_hash)
            .collect::<Vec<_>>();
        dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        if input.identities().input_hash() != source_key
            || input.identities().dependency_hashes() != dependencies
        {
            return blocking();
        }
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let Some(parent) = unique(
            input
                .parent_outputs()
                .iter()
                .map(|handle| handle.as_output_ref())
                .filter(|handle| {
                    handle.phase() == &IrPipelinePhase::new("SourceLoad")
                        && handle.work_unit() == &work_unit
                        && handle.output_kind() == &OutputKind::new("SourceUnit")
                        && handle.schema_version() == SchemaVersion::new(1)
                }),
        ) else {
            return blocking();
        };
        let parents_current = || {
            input.parent_outputs().iter().all(|handle| {
                let handle = handle.as_output_ref();
                publisher.validate_current_output(snapshot, handle).is_ok()
                    && publisher.storage().validate_handle(handle).is_ok()
            })
        };
        if !parents_current()
            || input.parent_outputs().iter().any(|handle| {
                let handle = handle.as_output_ref();
                handle.phase() == &IrPipelinePhase::new("SourceLoad") && handle != parent
            })
        {
            return blocking();
        }
        let Ok(typed) = publisher
            .storage()
            .typed_handle::<SourceUnit>(parent, &OutputKind::new("SourceUnit"))
        else {
            return blocking();
        };
        let Ok(source) = publisher.storage().get(&typed) else {
            return blocking();
        };
        let source_input = SourceInput {
            package_id: version.package_id.clone(),
            module_path: version.module_path.clone(),
            normalized_path: version.normalized_path.clone(),
            edition: version.edition.clone(),
            origin: SourceOriginInput::Disk {
                path: version.normalized_path.as_str().into(),
            },
        };
        if source.canonical_disk_bytes().is_none()
            || source.source_id != version.source_id
            || source.package_id != version.package_id
            || source.module_path != version.module_path
            || source.normalized_path != version.normalized_path
            || source.edition != version.edition
            || source.source_hash != version.source_hash
            || source.origin != SourceOrigin::Disk
        {
            return blocking();
        }
        let Some(workspace_summaries) = (|| {
            use mizar_artifact::{
                module_summary::ModuleSummaryIdentity, store::canonical_json_string,
            };
            use mizar_resolve::{
                declarations::DeclarationShellCollector, env::NamespacePath,
                module_index::IndexedModuleId, resolved_ast::ModuleId,
                symbols::SignatureProjectionExtractor,
            };
            let mut summaries = Vec::new();
            for handle in input.parent_outputs() {
                let leaf = handle.as_output_ref();
                if leaf == parent {
                    continue;
                }
                if leaf.phase() != &IrPipelinePhase::new("Frontend")
                    || leaf.output_kind() != &OutputKind::new("FrontendOutput")
                    || leaf.schema_version() != SchemaVersion::new(1)
                {
                    return None;
                }
                let typed = publisher
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        leaf,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .ok()?;
                let output = publisher.storage().get(&typed).ok()?;
                let leaf_source = &output.source;
                let leaf_version =
                    unique(inputs.snapshot.source_versions.iter().filter(|candidate| {
                        candidate.package_id == leaf_source.package_id
                            && candidate.module_path == leaf_source.module_path
                    }))?;
                let leaf_module = IndexedModuleId::new(
                    leaf_source.package_id.clone(),
                    leaf_source.module_path.clone(),
                );
                let leaf_entry = unique(
                    inputs
                        .module_index
                        .modules
                        .iter()
                        .filter(|candidate| candidate.module == leaf_module),
                )?;
                let ModuleIndexLocation::WorkspaceFile {
                    source_root: leaf_root,
                    normalized_path: leaf_path,
                    source_relative_path,
                } = &leaf_entry.location
                else {
                    return None;
                };
                let leaf_unit = IrWorkUnit::new(format!(
                    "{:?}:{:?}",
                    leaf_module.package.as_str(),
                    leaf_module.path.as_str()
                ));
                let [source_parent] = leaf.lineage().parents.as_slice() else {
                    return None;
                };
                let source_lineage = publisher.registry().output_lineage(*source_parent)?;
                let leaf_source_key = source_input_hash(leaf_version);
                if leaf_module.package != module.package
                    || leaf_module == *module
                    || leaf.work_unit() != &leaf_unit
                    || leaf_version.origin != SourceOrigin::Disk
                    || output.canonical_disk_bytes().is_none()
                    || leaf_source.source_id != leaf_version.source_id
                    || leaf_source.package_id != leaf_version.package_id
                    || leaf_source.module_path != leaf_version.module_path
                    || leaf_source.normalized_path != leaf_version.normalized_path
                    || leaf_source.edition != leaf_version.edition
                    || leaf_version.edition != package.edition
                    || leaf_source.source_hash != leaf_version.source_hash
                    || leaf_source.origin != SourceOrigin::Disk
                    || leaf_entry.package_id != leaf_module.package
                    || leaf_entry.module_path != leaf_module.path
                    || leaf_entry.edition != leaf_version.edition
                    || leaf_root != source_root
                    || leaf_path != leaf_version.normalized_path.as_str()
                    || leaf_path != &format!("src/{source_relative_path}")
                    || source_relative_path.is_empty()
                    || index_package.version != package.version
                    || source_lineage.snapshot != snapshot
                    || source_lineage.phase != IrPipelinePhase::new("SourceLoad")
                    || source_lineage.work_unit != leaf_unit
                    || source_lineage.output_kind != OutputKind::new("SourceUnit")
                    || !source_lineage.parents.is_empty()
                    || source_lineage.named_input_hashes.as_slice()
                        != [NamedInputHash {
                            name: "source".to_owned(),
                            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                            digest: leaf_source_key,
                        }]
                    || output.cache_keys.source.stable_hash() != leaf_source_key
                    || !output.diagnostics.is_empty()
                    || output.ast.as_ref().is_none_or(|ast| {
                        ast.node_views().any(|view| view.as_export_item().is_some())
                    })
                    || authenticated_import_candidates(&output, leaf_source)
                        .is_none_or(|candidates| !candidates.is_empty())
                {
                    return None;
                }
                let ast = output.ast.as_ref()?;
                let resolver_module =
                    ModuleId::new(leaf_module.package.clone(), leaf_module.path.clone());
                let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
                let collection = SignatureProjectionExtractor::new(
                    ast,
                    &shells,
                    NamespacePath::new(resolver_module.path().as_str()),
                )
                .collect(&resolver_module);
                if !collection.diagnostics().is_empty() {
                    return None;
                }
                let identity = ModuleSummaryIdentity {
                    package_id: leaf_module.package.as_str().to_owned(),
                    package_version: Some(package.version.to_string()),
                    lockfile_identity: None,
                    module_path: leaf_module.path.as_str().to_owned(),
                    language_edition: leaf_version.edition.as_str().to_owned(),
                };
                let module_id = canonical_json_string(&identity.canonical_json().ok()?);
                let shapes = collection
                    .export_frontend_lexical_contributions(&output, &identity)?
                    .into_iter()
                    .map(|contribution| {
                        ExportedSymbolShape::from_canonical_bytes(contribution.payload.as_bytes())
                    })
                    .collect::<Option<Vec<_>>>()?;
                let summary = ModuleLexicalSummary::from_exported_symbols(
                    mizar_frontend::lexical_env::ModuleId::new(module_id),
                    shapes,
                )?;
                summaries.push((leaf_module, summary));
            }
            Some(summaries)
        })() else {
            return blocking();
        };
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(package_root)),
            DependencyLexicalProvider {
                inputs,
                artifact_roots: &self.artifact_roots,
                workspace_summaries: &workspace_summaries,
            },
            MizarParserSeam,
        );
        let Ok(output) = frontend.run_loaded(source.as_ref().clone()) else {
            return blocking();
        };
        if output.ast.is_some() != output.cache_keys.ast.is_some() {
            return blocking();
        }
        if !parents_current() {
            return blocking();
        }
        let Some(drafts) =
            convert_frontend_diagnostics(&output.source, snapshot, &output.diagnostics)
        else {
            return blocking();
        };
        for draft in drafts {
            if sink.emit(draft).is_err() {
                return blocking();
            }
        }
        if !output.diagnostics.is_empty() {
            if !parents_current() {
                return blocking();
            }
            let import_batch = (|| {
                use mizar_diagnostics::sink::{DiagnosticProducerScope, DiagnosticSink};
                use mizar_resolve::{
                    imports::{ImportPathFailureClass as Failure, ImportPathResolver},
                    module_index::ModuleIndexInput,
                };
                if !output.diagnostics.iter().all(|diagnostic| {
                    diagnostic.code
                        == FrontendCode::LexicalEnvironment(EnvironmentCode::UnresolvedImport)
                        && diagnostic.class == DiagnosticClass::LexicalEnvironment
                }) || output.source != *source
                    || output.preprocessed.source_id != source.source_id
                {
                    return None;
                }
                let candidates = authenticated_import_candidates(&output, source.as_ref())?;
                if candidates.is_empty() {
                    return None;
                }
                let index = ModuleIndexInput::new(inputs.module_index);
                let current = index.resolver_module_id(module);
                let resolution = ImportPathResolver::new(index).resolve(&current, &candidates);
                if resolution.unresolved().is_empty() {
                    return None;
                }
                let registry = mizar_diagnostics::registry::DiagnosticRegistry::builtin();
                let drafts = resolution
                    .unresolved()
                    .iter()
                    .map(|candidate| {
                        let number = match candidate.class() {
                            Failure::UnknownNamespaceOrPackage => 220,
                            Failure::UnknownModule => 221,
                            Failure::RelativePathEscapesPackage => 222,
                            Failure::DuplicateAlias => 223,
                            Failure::AliasRootConflict => 224,
                            _ => return None,
                        };
                        let alias_range = candidate
                            .alias_range()
                            .or(candidate.branch_member_range())
                            .unwrap_or(candidate.range());
                        let primary = match number {
                            220 | 222 => candidate.branch_base_range().unwrap_or(candidate.range()),
                            221 => candidate.branch_member_range().unwrap_or(candidate.range()),
                            _ => alias_range,
                        };
                        let mut secondary = match number {
                            220 | 222 => candidate.branch_member_range(),
                            _ => candidate.branch_base_range(),
                        }
                        .into_iter()
                        .collect::<Vec<_>>();
                        let mut details = vec![
                            (
                                "import.source_module",
                                DiagnosticDetailValue::List(vec![
                                    DiagnosticDetailValue::String(
                                        current.package().as_str().to_owned(),
                                    ),
                                    DiagnosticDetailValue::String(
                                        current.path().as_str().to_owned(),
                                    ),
                                ]),
                            ),
                            (
                                "import.ordinal",
                                DiagnosticDetailValue::Integer(
                                    i64::try_from(candidate.ordinal()).ok()?,
                                ),
                            ),
                            (
                                "import.path",
                                DiagnosticDetailValue::String(candidate.spelling().to_owned()),
                            ),
                        ];
                        if let Some(member) = candidate.branch_member_range() {
                            details.push((
                                "import.branch_member",
                                DiagnosticDetailValue::Source(member),
                            ));
                        }
                        if number == 223 {
                            secondary.retain(|range| *range != primary);
                            let alias = candidate
                                .alias()
                                .or_else(|| candidate.components().last().map(String::as_str))?;
                            let target = candidate.candidate_target()?;
                            details.push((
                                "import.alias",
                                DiagnosticDetailValue::String(alias.to_owned()),
                            ));
                            details.push((
                                "import.target",
                                DiagnosticDetailValue::List(vec![
                                    DiagnosticDetailValue::String(
                                        target.package().as_str().to_owned(),
                                    ),
                                    DiagnosticDetailValue::String(
                                        target.path().as_str().to_owned(),
                                    ),
                                ]),
                            ));
                            let mut peers = resolution
                                .unresolved()
                                .iter()
                                .filter(|peer| {
                                    peer.class() == Failure::DuplicateAlias
                                        && peer.ordinal() != candidate.ordinal()
                                        && peer.alias().or_else(|| {
                                            peer.components().last().map(String::as_str)
                                        }) == Some(alias)
                                })
                                .map(|peer| {
                                    Some((
                                        peer.ordinal(),
                                        peer.range().start,
                                        peer.range().end,
                                        peer.candidate_target()?,
                                        peer.alias_range()
                                            .or(peer.branch_member_range())
                                            .unwrap_or(peer.range()),
                                    ))
                                })
                                .collect::<Option<Vec<_>>>()?;
                            peers.sort_by(|a, b| {
                                (&a.0, &a.1, &a.2, &a.3).cmp(&(&b.0, &b.1, &b.2, &b.3))
                            });
                            secondary.extend(peers.into_iter().map(|peer| peer.4));
                        }
                        let span = |range, role| {
                            source.line_map.validate_range(range).ok()?;
                            DiagnosticSpan::from_anchor(
                                mizar_session::SourceAnchor::Range(range),
                                role,
                                None,
                                SpanFreshness::Current,
                                None,
                            )
                            .ok()
                        };
                        let primary = span(primary, DiagnosticSpanRole::Primary)?;
                        let secondary = secondary
                            .into_iter()
                            .map(|range| span(range, DiagnosticSpanRole::Secondary))
                            .collect::<Option<Vec<_>>>()?;
                        let descriptor = registry.lookup(
                            DiagnosticCode::from_parts(DiagnosticSeverity::Error, number).ok()?,
                        )?;
                        DiagnosticDraft::new(DiagnosticDraftInput {
                            source_snapshot: snapshot,
                            code: descriptor.code,
                            phase: DiagnosticPhase::Resolver,
                            category: FailureCategory::ResolveError,
                            stable_detail_key: descriptor.semantic_name.to_owned(),
                            message: descriptor.summary.to_owned(),
                            primary_location: DiagnosticPrimaryLocation::Span(primary),
                            secondary_spans: secondary,
                            notes: Vec::new(),
                            details: DiagnosticDetails::from_entries(details).ok()?,
                            fixes: Vec::new(),
                            explanation: None,
                        })
                        .ok()
                    })
                    .collect::<Option<Vec<_>>>()?;
                let mut resolver_sink = DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::Resolver,
                    snapshot,
                    "frontend.import_continuation",
                ));
                for draft in drafts {
                    resolver_sink.emit(draft).ok()?;
                }
                Some(resolver_sink.seal())
            })();
            if !parents_current() {
                return blocking();
            }
            let mut diagnostics = vec![sink.seal()];
            diagnostics.extend(import_batch);
            return PhaseResult {
                status: if output.ast.is_some() {
                    PhaseStatus::Recoverable
                } else {
                    PhaseStatus::Fatal
                },
                diagnostics,
                ..blocking()
            };
        }
        let Some(candidates) = authenticated_import_candidates(&output, source.as_ref()) else {
            return blocking();
        };
        let index = mizar_resolve::module_index::ModuleIndexInput::new(inputs.module_index);
        let resolution = mizar_resolve::imports::ImportPathResolver::new(index)
            .resolve(&index.resolver_module_id(module), &candidates);
        if !resolution.unresolved().is_empty() {
            return blocking();
        }
        let mut used = Vec::new();
        for import in resolution.resolved() {
            let target = mizar_resolve::module_index::IndexedModuleId::new(
                import.target().package().clone(),
                import.target().path().clone(),
            );
            let Some(entry) = unique(
                inputs
                    .module_index
                    .modules
                    .iter()
                    .filter(|entry| entry.module == target),
            ) else {
                return blocking();
            };
            if matches!(&entry.location, ModuleIndexLocation::WorkspaceFile { .. })
                && !workspace_summaries
                    .iter()
                    .any(|(module, _)| module == &target)
            {
                return blocking();
            }
            if matches!(&entry.location, ModuleIndexLocation::WorkspaceFile { .. })
                && !used.contains(&target)
            {
                used.push(target);
            }
        }
        if used.len() != workspace_summaries.len() {
            return blocking();
        }
        let Some(ast_key) = output.cache_keys.ast.as_ref() else {
            return blocking();
        };
        let Some(storage_bytes) = output.canonical_disk_bytes() else {
            return blocking();
        };
        let mut semantic_bytes = b"mizar-frontend/output-publication/v1\0".to_vec();
        semantic_bytes.extend_from_slice(ast_key.stable_hash().as_bytes());
        let storage_hash = Hash::from_bytes(blake3::derive_key(
            "mizar-frontend/output-storage-side-table/v1",
            &storage_bytes,
        ));
        let mut named_input_hashes = vec![
            NamedInputHash {
                name: "active-lexical-environment".to_owned(),
                domain: ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION.to_owned(),
                digest: output.cache_keys.active_lexical_environment.stable_hash(),
            },
            NamedInputHash {
                name: "tokens".to_owned(),
                domain: TOKEN_STREAM_CACHE_KEY_VERSION.to_owned(),
                digest: output.cache_keys.tokens.stable_hash(),
            },
        ];
        for (ordinal, digest) in dependencies.into_iter().enumerate() {
            named_input_hashes.push(NamedInputHash {
                name: format!("dependency.{ordinal}"),
                domain: "mizar-frontend/dependency-summary/v1".to_owned(),
                digest,
            });
        }
        let decode_input = source_input;
        let source_id = version.source_id;
        let expected_hash = version.source_hash;
        let phase = IrPipelinePhase::new("Frontend");
        let output_kind = OutputKind::new("FrontendOutput");
        if !parents_current() {
            return blocking();
        }
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
            payload: output,
            canonical_payload: Some(semantic_bytes),
            storage_payload: Some(storage_bytes),
            decode: BlobDecoder::new(move |bytes| {
                FrontendOutput::from_canonical_disk_bytes(bytes, source_id, &decode_input)
                    .filter(|output| output.source.source_hash == expected_hash)
                    .ok_or_else(|| BlobDecodeError::new("invalid frontend disk payload"))
            }),
            parents: input
                .parent_outputs()
                .iter()
                .map(|handle| handle.as_output_ref().clone())
                .collect(),
            named_input_hashes,
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "frontend-storage",
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

fn authenticated_import_candidates(
    output: &FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
    source: &SourceUnit,
) -> Option<Vec<mizar_resolve::imports::ImportPathCandidate>> {
    use mizar_resolve::imports::{ImportPathCandidate, ImportPathPrefix};
    let ast = output.ast.as_ref()?;
    output.cache_keys.ast.as_ref()?;
    if output.source != *source
        || output.preprocessed.source_id != source.source_id
        || ast.source_id != source.source_id
        || ast
            .node_views()
            .any(|view| view.as_recovery().is_some() || view.is_recovered())
    {
        return None;
    }
    let candidates = ImportPathCandidate::from_surface_ast(ast)?;
    let provisional = ImportPathCandidate::from_frontend_imports(
        &mizar_frontend::lexical_env::LexicalEnvironmentRequest {
            source_id: source.source_id,
            import_stubs: &output.preprocessed.import_stubs,
            edition: source.edition.clone(),
        },
    )?;
    if candidates.len() != provisional.len() {
        return None;
    }
    for ((candidate, prescan), stub) in candidates
        .iter()
        .zip(&provisional)
        .zip(&output.preprocessed.import_stubs)
    {
        if candidate.components() != prescan.components()
            || candidate.prefix() != prescan.prefix()
            || candidate.alias() != prescan.alias()
            || candidate.alias_range() != prescan.alias_range()
            || candidate.branch_base_range() != prescan.branch_base_range()
            || candidate.branch_member_range() != prescan.branch_member_range()
            || if candidate.branch_member_range().is_some() {
                candidate.range().start > prescan.range().start
                    || candidate.range().end < prescan.range().end
            } else {
                candidate.range() != prescan.range()
            }
        {
            return None;
        }
        for range in std::iter::once(candidate.range())
            .chain(std::iter::once(stub.span))
            .chain(std::iter::once(stub.path.span))
            .chain(stub.path.source_segments.iter().copied())
            .chain(candidate.alias_range())
        {
            source.line_map.validate_range(range).ok()?;
        }
        let prefix = match candidate.prefix() {
            ImportPathPrefix::Unprefixed => "",
            ImportPathPrefix::Current => ".",
            ImportPathPrefix::Parent => "..",
            _ => return None,
        };
        if stub.path.spelling.as_ref() != format!("{prefix}{}", candidate.components().join(".")) {
            return None;
        }
    }
    // Authenticate the whole import framing, not just path coordinates.
    for item in ast.node_views().filter_map(|view| view.as_import_item()) {
        let range = item.range();
        source.line_map.validate_range(range).ok()?;
        for view in ast
            .token_views()
            .filter(|view| range.start <= view.range().start && view.range().end <= range.end)
        {
            source.line_map.validate_range(view.range()).ok()?;
            if source
                .source_text
                .get(view.range().start..view.range().end)?
                != view.as_token()?.text.as_ref()
            {
                return None;
            }
        }
    }
    Some(candidates)
}

fn convert_frontend_diagnostics(
    source: &SourceUnit,
    snapshot: mizar_session::BuildSnapshotId,
    diagnostics: &[FrontendDiagnostic],
) -> Option<Vec<DiagnosticDraft>> {
    let registry = mizar_diagnostics::registry::DiagnosticRegistry::builtin();
    diagnostics
        .iter()
        .map(|diagnostic| {
            let class = match diagnostic.class {
                DiagnosticClass::LexicalPrecondition => "lexical_precondition",
                DiagnosticClass::CommentStructure => "comment_structure",
                DiagnosticClass::ImportPrescan => "import_prescan",
                DiagnosticClass::LexicalEnvironment => "lexical_environment",
                DiagnosticClass::ScopeSkeleton => "scope_skeleton",
                DiagnosticClass::Tokenization => "tokenization",
                DiagnosticClass::Syntax => "syntax",
                DiagnosticClass::AnnotationSyntax => "annotation_syntax",
                _ => return None,
            };
            let (number, prefix, leaf, expected_class) = match &diagnostic.code {
                FrontendCode::Preprocess(PreprocessDiagnosticKind::SourcePrecondition(code)) => {
                    match code {
                        SourcePreprocessDiagnosticCode::CarriageReturn => (
                            13,
                            "preprocess.source_precondition",
                            "carriage_return",
                            "lexical_precondition",
                        ),
                        SourcePreprocessDiagnosticCode::NonAsciiCode => (
                            14,
                            "preprocess.source_precondition",
                            "non_ascii_code",
                            "lexical_precondition",
                        ),
                        SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment => (
                            15,
                            "preprocess.source_precondition",
                            "unterminated_multi_line_comment",
                            "comment_structure",
                        ),
                        _ => return None,
                    }
                }
                FrontendCode::Preprocess(PreprocessDiagnosticKind::ImportPrescan(code)) => {
                    let (number, leaf) = match code {
                        ImportPrescanDiagnosticCode::MissingModulePath => {
                            (16, "missing_module_path")
                        }
                        ImportPrescanDiagnosticCode::EmptyModulePathComponent => {
                            (17, "empty_module_path_component")
                        }
                        ImportPrescanDiagnosticCode::MissingAlias => (18, "missing_alias"),
                        ImportPrescanDiagnosticCode::MissingSemicolon => (19, "missing_semicolon"),
                        ImportPrescanDiagnosticCode::UnexpectedToken => (20, "unexpected_token"),
                        _ => return None,
                    };
                    (number, "preprocess.import_prescan", leaf, "import_prescan")
                }
                FrontendCode::Preprocess(PreprocessDiagnosticKind::RawImportScan) => {
                    (21, "preprocess", "raw_import_scan", "import_prescan")
                }
                FrontendCode::LexicalEnvironment(code) => {
                    let (number, leaf) = match code {
                        EnvironmentCode::UnresolvedImport => (22, "unresolved_import"),
                        EnvironmentCode::MissingSummary => (23, "missing_summary"),
                        EnvironmentCode::UserSymbolImportConflict => {
                            (24, "user_symbol_import_conflict")
                        }
                        EnvironmentCode::InvalidUserSymbolSpelling => {
                            (25, "invalid_user_symbol_spelling")
                        }
                        EnvironmentCode::InvalidUserSymbolArity => {
                            (26, "invalid_user_symbol_arity")
                        }
                        EnvironmentCode::ReservedWordCollision => (27, "reserved_word_collision"),
                        EnvironmentCode::ReservedSymbolCollision => {
                            (28, "reserved_symbol_collision")
                        }
                        _ => return None,
                    };
                    (number, "lexical_environment", leaf, "lexical_environment")
                }
                FrontendCode::Lexing(LexingDiagnosticKind::RawScan) => {
                    (29, "lexing", "raw_scan", "tokenization")
                }
                FrontendCode::Lexing(LexingDiagnosticKind::ScopeSkeleton(code)) => {
                    let (number, leaf) = match code {
                        ScopeSkeletonDiagnosticCode::MalformedBinderList => {
                            (30, "malformed_binder_list")
                        }
                        ScopeSkeletonDiagnosticCode::UnsupportedBinderShape => {
                            (31, "unsupported_binder_shape")
                        }
                        ScopeSkeletonDiagnosticCode::DuplicateBindingName => {
                            (32, "duplicate_binding_name")
                        }
                        ScopeSkeletonDiagnosticCode::UnmatchedEnd => (33, "unmatched_end"),
                        ScopeSkeletonDiagnosticCode::MissingEnd => (10, "missing_end"),
                        _ => return None,
                    };
                    (number, "lexing.scope_skeleton", leaf, "scope_skeleton")
                }
                FrontendCode::Lexing(LexingDiagnosticKind::Lexer(code)) => {
                    let (number, leaf) = match code {
                        LexDiagnosticCode::NoValidTokenCandidate => {
                            (34, "no_valid_token_candidate")
                        }
                        LexDiagnosticCode::ParserContextRejectedCandidate => {
                            (35, "parser_context_rejected_candidate")
                        }
                        LexDiagnosticCode::AmbiguousUserSymbol => (36, "ambiguous_user_symbol"),
                        LexDiagnosticCode::MalformedStringLiteral => {
                            (2, "malformed_string_literal")
                        }
                        LexDiagnosticCode::UnsupportedRawToken => (37, "unsupported_raw_token"),
                        _ => return None,
                    };
                    (number, "lexing.lexer", leaf, "tokenization")
                }
                FrontendCode::Syntax(key) => {
                    let number = match key.as_ref() {
                        "unexpected_error_token" => 38,
                        "dangling_operator" => 39,
                        "non_associative_operator_chain" => 40,
                        "missing_end" => 10,
                        "missing_semicolon" => 41,
                        "missing_string_literal" => 42,
                        "malformed_import" => 43,
                        "malformed_export" => 44,
                        "malformed_visibility" => 45,
                        "malformed_type_expression" => 46,
                        "malformed_term_expression" => 47,
                        "malformed_formula_expression" => 48,
                        "malformed_justification" => 49,
                        "malformed_annotation" => 50,
                        "unexpected_top_level_token" => 51,
                        "unrecoverable_input" => 52,
                        _ => return None,
                    };
                    (number, "syntax", key.as_ref(), "syntax")
                }
                _ => return None,
            };
            if class != expected_class
                && !(expected_class == "syntax" && class == "annotation_syntax")
            {
                return None;
            }
            let DiagnosticLocation::SourceRange(primary) = &diagnostic.location else {
                return None;
            };
            source.line_map.validate_range(*primary).ok()?;
            let primary = DiagnosticSpan::from_anchor(
                mizar_session::SourceAnchor::Range(*primary),
                DiagnosticSpanRole::Primary,
                None,
                SpanFreshness::Current,
                None,
            )
            .ok()?;
            let secondary = diagnostic
                .secondary
                .iter()
                .map(|anchor| {
                    let span = DiagnosticSpan::from_anchor(
                        anchor.clone(),
                        DiagnosticSpanRole::Secondary,
                        None,
                        SpanFreshness::Current,
                        None,
                    )
                    .ok()?;
                    source.line_map.validate_range(span.range()).ok()?;
                    Some(span)
                })
                .collect::<Option<Vec<_>>>()?;
            let code = DiagnosticCode::from_parts(DiagnosticSeverity::Error, number).ok()?;
            let descriptor = registry.lookup(code)?;
            let local_code = format!("{prefix}.{leaf}");
            let details = DiagnosticDetails::from_entries([
                (
                    "frontend.class",
                    DiagnosticDetailValue::String(class.to_owned()),
                ),
                ("frontend.code", DiagnosticDetailValue::String(local_code)),
            ])
            .ok()?;
            let notes = diagnostic
                .recovery_note
                .as_ref()
                .map_or_else(Vec::new, |note| {
                    vec![DiagnosticNote::new(
                        DiagnosticNoteKind::Note,
                        note.clone(),
                        None,
                    )]
                });
            DiagnosticDraft::new(DiagnosticDraftInput {
                source_snapshot: snapshot,
                code,
                phase: DiagnosticPhase::Frontend,
                category: FailureCategory::ParseError,
                stable_detail_key: descriptor.semantic_name.to_owned(),
                message: diagnostic.message.to_string(),
                primary_location: DiagnosticPrimaryLocation::Span(primary),
                secondary_spans: secondary,
                notes,
                details,
                fixes: Vec::new(),
                explanation: None,
            })
            .ok()
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_frontend::lexing::{LexDiagnosticCode as L, ScopeSkeletonDiagnosticCode as S};
    use mizar_frontend::orchestration::{DiagnosticClass as C, DiagnosticCode as D};
    use mizar_frontend::preprocess::{
        ImportPrescanDiagnosticCode as I, SourcePreprocessDiagnosticCode as P,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, GeneratedSpanAnchor, GeneratedSpanOrigin,
        InMemorySessionIdAllocator, ModulePath, SessionIdAllocator, SourceAnchor, normalize_path,
    };
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_SOURCE: AtomicUsize = AtomicUsize::new(0);

    fn source() -> (BuildSnapshotId, SourceUnit) {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "28".repeat(32)
        ))
        .unwrap();
        let root = std::env::temp_dir().join(format!(
            "mizar-driver-frontend-converter-{}-{}",
            std::process::id(),
            NEXT_SOURCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/test.miz"), "éx\n").unwrap();
        let input = SourceInput {
            package_id: PackageId::new("test"),
            module_path: ModulePath::new("test"),
            normalized_path: normalize_path(&root, Path::new("src/test.miz")).unwrap(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: "src/test.miz".into(),
            },
        };
        let source = FrontendSourceLoader::new(DiskSourceLoader::new(&root))
            .load_source_unit(
                SourceUnitRequest { snapshot, input },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();
        fs::remove_dir_all(root).unwrap();
        (snapshot, source)
    }

    fn diagnostic(source: &SourceUnit, code: D, class: C) -> FrontendDiagnostic {
        FrontendDiagnostic {
            code,
            message: Arc::from("exact message"),
            class,
            location: DiagnosticLocation::SourceRange(mizar_session::SourceRange {
                source_id: source.source_id,
                start: 0,
                end: 2,
            }),
            secondary: Vec::new(),
            recovery_note: None,
        }
    }

    #[test]
    fn typed_frontend_categories_map_to_allocated_codes() {
        use mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode as E;
        use mizar_frontend::lexing::LexingDiagnosticKind as K;
        use mizar_frontend::preprocess::PreprocessDiagnosticKind as R;
        let (snapshot, source) = source();
        let cases = vec![
            (
                D::Preprocess(R::SourcePrecondition(P::CarriageReturn)),
                C::LexicalPrecondition,
                13,
            ),
            (
                D::Preprocess(R::SourcePrecondition(P::NonAsciiCode)),
                C::LexicalPrecondition,
                14,
            ),
            (
                D::Preprocess(R::SourcePrecondition(P::UnterminatedMultiLineComment)),
                C::CommentStructure,
                15,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::MissingModulePath)),
                C::ImportPrescan,
                16,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::EmptyModulePathComponent)),
                C::ImportPrescan,
                17,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::MissingAlias)),
                C::ImportPrescan,
                18,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::MissingSemicolon)),
                C::ImportPrescan,
                19,
            ),
            (
                D::Preprocess(R::ImportPrescan(I::UnexpectedToken)),
                C::ImportPrescan,
                20,
            ),
            (D::Preprocess(R::RawImportScan), C::ImportPrescan, 21),
            (
                D::LexicalEnvironment(E::UnresolvedImport),
                C::LexicalEnvironment,
                22,
            ),
            (
                D::LexicalEnvironment(E::MissingSummary),
                C::LexicalEnvironment,
                23,
            ),
            (
                D::LexicalEnvironment(E::UserSymbolImportConflict),
                C::LexicalEnvironment,
                24,
            ),
            (
                D::LexicalEnvironment(E::InvalidUserSymbolSpelling),
                C::LexicalEnvironment,
                25,
            ),
            (
                D::LexicalEnvironment(E::InvalidUserSymbolArity),
                C::LexicalEnvironment,
                26,
            ),
            (
                D::LexicalEnvironment(E::ReservedWordCollision),
                C::LexicalEnvironment,
                27,
            ),
            (
                D::LexicalEnvironment(E::ReservedSymbolCollision),
                C::LexicalEnvironment,
                28,
            ),
            (D::Lexing(K::RawScan), C::Tokenization, 29),
            (
                D::Lexing(K::ScopeSkeleton(S::MalformedBinderList)),
                C::ScopeSkeleton,
                30,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::UnsupportedBinderShape)),
                C::ScopeSkeleton,
                31,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::DuplicateBindingName)),
                C::ScopeSkeleton,
                32,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::UnmatchedEnd)),
                C::ScopeSkeleton,
                33,
            ),
            (
                D::Lexing(K::ScopeSkeleton(S::MissingEnd)),
                C::ScopeSkeleton,
                10,
            ),
            (
                D::Lexing(K::Lexer(L::NoValidTokenCandidate)),
                C::Tokenization,
                34,
            ),
            (
                D::Lexing(K::Lexer(L::ParserContextRejectedCandidate)),
                C::Tokenization,
                35,
            ),
            (
                D::Lexing(K::Lexer(L::AmbiguousUserSymbol)),
                C::Tokenization,
                36,
            ),
            (
                D::Lexing(K::Lexer(L::MalformedStringLiteral)),
                C::Tokenization,
                2,
            ),
            (
                D::Lexing(K::Lexer(L::UnsupportedRawToken)),
                C::Tokenization,
                37,
            ),
            (
                D::Syntax(Arc::from("unexpected_error_token")),
                C::Syntax,
                38,
            ),
            (D::Syntax(Arc::from("dangling_operator")), C::Syntax, 39),
            (
                D::Syntax(Arc::from("non_associative_operator_chain")),
                C::Syntax,
                40,
            ),
            (D::Syntax(Arc::from("missing_end")), C::Syntax, 10),
            (D::Syntax(Arc::from("missing_semicolon")), C::Syntax, 41),
            (
                D::Syntax(Arc::from("missing_string_literal")),
                C::Syntax,
                42,
            ),
            (D::Syntax(Arc::from("malformed_import")), C::Syntax, 43),
            (D::Syntax(Arc::from("malformed_export")), C::Syntax, 44),
            (D::Syntax(Arc::from("malformed_visibility")), C::Syntax, 45),
            (
                D::Syntax(Arc::from("malformed_type_expression")),
                C::Syntax,
                46,
            ),
            (
                D::Syntax(Arc::from("malformed_term_expression")),
                C::Syntax,
                47,
            ),
            (
                D::Syntax(Arc::from("malformed_formula_expression")),
                C::Syntax,
                48,
            ),
            (
                D::Syntax(Arc::from("malformed_justification")),
                C::Syntax,
                49,
            ),
            (
                D::Syntax(Arc::from("malformed_annotation")),
                C::AnnotationSyntax,
                50,
            ),
            (
                D::Syntax(Arc::from("unexpected_top_level_token")),
                C::Syntax,
                51,
            ),
            (D::Syntax(Arc::from("unrecoverable_input")), C::Syntax, 52),
        ];
        assert_eq!(cases.len(), 43);
        for (code, class, number) in cases {
            let drafts = convert_frontend_diagnostics(
                &source,
                snapshot,
                &[diagnostic(&source, code, class)],
            )
            .expect("typed category converts");
            assert_eq!(drafts.len(), 1);
            assert_eq!(drafts[0].code().number(), number);
            assert_eq!(drafts[0].phase(), DiagnosticPhase::Frontend);
            assert_eq!(drafts[0].message(), "exact message");
            assert!(drafts[0].details().entries().contains_key("frontend.class"));
            assert!(drafts[0].details().entries().contains_key("frontend.code"));
            if number == 15 {
                assert_eq!(
                    drafts[0].details().entries().get("frontend.code"),
                    Some(&DiagnosticDetailValue::String(
                        "preprocess.source_precondition.unterminated_multi_line_comment".to_owned()
                    ))
                );
            }
            if number == 17 {
                assert_eq!(
                    drafts[0].details().entries().get("frontend.code"),
                    Some(&DiagnosticDetailValue::String(
                        "preprocess.import_prescan.empty_module_path_component".to_owned()
                    ))
                );
            }
        }
    }

    #[test]
    fn converter_preserves_anchors_notes_and_rejects_invalid_batches_atomically() {
        let (snapshot, source) = source();
        let range = mizar_session::SourceRange {
            source_id: source.source_id,
            start: 2,
            end: 2,
        };
        let reason = "  生成 \"exact\"\n";
        let anchor = SourceAnchor::Generated(
            GeneratedSpanOrigin::new(
                GeneratedSpanAnchor::Point {
                    source_id: source.source_id,
                    offset: 2,
                },
                reason,
            )
            .unwrap(),
        );
        let generated_range = SourceAnchor::Generated(
            GeneratedSpanOrigin::new(GeneratedSpanAnchor::Range(range), reason).unwrap(),
        );
        let mut first = diagnostic(
            &source,
            D::Syntax(Arc::from("missing_semicolon")),
            C::Syntax,
        );
        first.secondary = vec![
            SourceAnchor::Range(range),
            SourceAnchor::Point {
                source_id: source.source_id,
                offset: 2,
            },
            generated_range.clone(),
            anchor.clone(),
            anchor.clone(),
        ];
        first.recovery_note = Some(String::new());
        let drafts = convert_frontend_diagnostics(&source, snapshot, &[first.clone()]).unwrap();
        assert_eq!(drafts[0].secondary_spans().len(), 5);
        assert_eq!(
            drafts[0].secondary_spans()[0].anchor(),
            &SourceAnchor::Range(range)
        );
        assert!(matches!(
            drafts[0].secondary_spans()[1].anchor(),
            SourceAnchor::Point { .. }
        ));
        assert_eq!(drafts[0].secondary_spans()[2].anchor(), &generated_range);
        assert_eq!(drafts[0].secondary_spans()[3].anchor(), &anchor);
        assert_eq!(drafts[0].secondary_spans()[4].anchor(), &anchor);
        assert_eq!(drafts[0].secondary_spans()[3].zero_width(), None);
        assert_eq!(drafts[0].notes()[0].message(), "");
        first.recovery_note = None;
        assert!(
            convert_frontend_diagnostics(&source, snapshot, &[first.clone()]).unwrap()[0]
                .notes()
                .is_empty()
        );
        let mut zero_primary = first.clone();
        zero_primary.location = DiagnosticLocation::SourceRange(range);
        let converted = convert_frontend_diagnostics(&source, snapshot, &[zero_primary]).unwrap();
        let DiagnosticPrimaryLocation::Span(span) = converted[0].primary_location() else {
            panic!("expected span");
        };
        assert_eq!(span.range(), range);
        assert_eq!(span.zero_width(), None);

        let mut invalid = first.clone();
        invalid.secondary = vec![SourceAnchor::Point {
            source_id: source.source_id,
            offset: 1,
        }];
        assert!(
            convert_frontend_diagnostics(&source, snapshot, &[first.clone(), invalid]).is_none()
        );
        let mut invalid = first.clone();
        invalid.location = DiagnosticLocation::SourceRange(mizar_session::SourceRange {
            source_id: source.source_id,
            start: 5,
            end: 5,
        });
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let ids = InMemorySessionIdAllocator::new();
        let _ = ids.next_source_id(snapshot).unwrap();
        let foreign = ids.next_source_id(snapshot).unwrap();
        let mut invalid = first.clone();
        invalid.secondary = vec![SourceAnchor::Generated(
            GeneratedSpanOrigin::new(
                GeneratedSpanAnchor::Range(mizar_session::SourceRange {
                    source_id: foreign,
                    start: 0,
                    end: 2,
                }),
                "foreign",
            )
            .unwrap(),
        )];
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first.clone();
        invalid.location = DiagnosticLocation::SourceRange(mizar_session::SourceRange {
            source_id: source.source_id,
            start: 3,
            end: 2,
        });
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first.clone();
        invalid.class = C::Tokenization;
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first.clone();
        invalid.code = D::Syntax(Arc::from("unknown_parser_key"));
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
        let mut invalid = first;
        invalid.code = D::SourceLoad;
        assert!(convert_frontend_diagnostics(&source, snapshot, &[invalid]).is_none());
    }
}
