use super::*;
use mizar_diagnostics::{
    explain::{
        ExplanationHandle, ExplanationHandleId, ExplanationHandleInput, ExplanationKind,
        ExplanationPayload, ExplanationPreview, ExplanationPreviewFormat, ExplanationSourceRef,
        ExplanationSubject,
    },
    failure_record::DiagnosticRecord,
    fix::{
        FixApplicability, FixCommandRef, FixEdit, FixSafety, FixSuggestion, FixSuggestionId,
        FixSuggestionInput,
    },
};
use mizar_session::{BuildSnapshotId, LoadedSource, SourceRange};

/// Encoding of the bounded native export-terminator diagnostic context.
pub const NATIVE_EXPORT_SEMICOLON_ENCODING: &str = "mizar-frontend/export-semicolon-context/v1";
const EXPORT_INSERTION_DETAIL: &str = "frontend.export_semicolon_insertion";

fn export_backing_hash(role: &str, bytes: &[u8]) -> Hash {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mizar-lsp/action-backing/v1");
    for bytes in [
        role.as_bytes(),
        NATIVE_EXPORT_SEMICOLON_ENCODING.as_bytes(),
        bytes,
    ] {
        hash.update(&(bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Hash::from_bytes(*hash.finalize().as_bytes())
}

fn export_semicolon_payload(
    insertion: SourceRange,
    snapshot: BuildSnapshotId,
    code: DiagnosticCode,
    stable_detail_key: &str,
) -> Option<(Vec<FixSuggestion>, ExplanationHandle, ExplanationPayload)> {
    use mizar_artifact::store::{CanonicalJson as J, canonical_json_string};
    let source = insertion
        .source_id
        .to_nonportable_provenance_string()
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let key = format!(
        "mizar_frontend.export_semicolon.source_{source}.at_{}",
        insertion.start
    );
    let bytes = canonical_json_string(
        &J::object([
            ("category", J::string("syntax.missing_semicolon")),
            ("site", J::string("export-before-item")),
            ("expected", J::string(";")),
            (
                "insertion_offset",
                J::integer(i64::try_from(insertion.start).ok()?),
            ),
        ])
        .ok()?,
    );
    let summary_hash = export_backing_hash("summary", bytes.as_bytes());
    let handle = ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new(format!("{key}.context")).ok()?,
        kind: ExplanationKind::DiagnosticContext,
        subject: ExplanationSubject::Diagnostic {
            code,
            stable_detail_key: stable_detail_key.to_owned(),
        },
        source: ExplanationSourceRef::CacheRecord {
            cache_key: format!("{key}.context"),
            content_hash: Some(export_backing_hash("content", bytes.as_bytes())),
        },
        required_snapshot: Some(snapshot),
        required_artifact_hash: None,
        summary_hash: Some(summary_hash),
        preview: None,
    })
    .ok()?;
    let payload = ExplanationPayload::from_preview(
        ExplanationPreview::new(ExplanationPreviewFormat::StructuredText, bytes),
        Some(summary_hash),
    );
    let mut fixes = Vec::new();
    for (role, applicability, safety, edits, command, title) in [
        (
            "insert",
            FixApplicability::MaybeIncorrect,
            FixSafety::LocalTextEdit,
            vec![FixEdit::new(insertion, ";", Some(String::new())).ok()?],
            None,
            "Insert missing export terminator",
        ),
        (
            "explain",
            FixApplicability::MachineApplicable,
            FixSafety::CommandOnly,
            Vec::new(),
            Some(FixCommandRef::new("mizar_frontend.export_semicolon.explain.v1").ok()?),
            "Explain missing export terminator",
        ),
    ] {
        let identity = format!("{key}.{role}");
        fixes.push(
            FixSuggestion::new(FixSuggestionInput {
                id: FixSuggestionId::new(identity.clone()).ok()?,
                producer_key: Some(identity),
                title: title.to_owned(),
                applicability,
                safety,
                edits,
                command,
                required_snapshot: Some(snapshot),
                required_text_hash: None,
            })
            .ok()?,
        );
    }
    Some((fixes, handle, payload))
}

fn native_export_payload(
    record: &DiagnosticRecord,
    loaded: &LoadedSource,
) -> Option<(Vec<FixSuggestion>, ExplanationHandle, ExplanationPayload)> {
    let DiagnosticDetailValue::Source(insertion) =
        record.details().entries().get(EXPORT_INSERTION_DETAIL)?
    else {
        return None;
    };
    let DiagnosticPrimaryLocation::Span(primary) = record.primary_location() else {
        return None;
    };
    let primary = primary.range();
    let descriptor =
        mizar_diagnostics::registry::DiagnosticRegistry::builtin().lookup(record.code())?;
    if record.code() != DiagnosticCode::from_parts(DiagnosticSeverity::Error, 41).ok()?
        || record.phase() != DiagnosticPhase::Frontend
        || record.category() != FailureCategory::ParseError
        || record.stable_detail_key() != descriptor.semantic_name
        || record.freshness().source_snapshot() != record.handle().snapshot
        || !record.freshness().is_current()
        || insertion.source_id != loaded.source_id
        || insertion.start != insertion.end
        || insertion.start != primary.start
        || primary.source_id != loaded.source_id
        || loaded.line_map.source() != loaded.text.as_ref()
        || loaded.line_map.source_id() != loaded.source_id
    {
        return None;
    }
    loaded.line_map.validate_range(*insertion).ok()?;
    loaded.line_map.validate_range(primary).ok()?;
    export_semicolon_payload(
        *insertion,
        record.handle().snapshot,
        record.code(),
        record.stable_detail_key(),
    )
}

/// Validates a genuine native record's exact context association against its retained source.
/// Admission of records remains the original native capture owner's responsibility.
pub fn native_export_semicolon_context(
    record: &DiagnosticRecord,
    loaded: &LoadedSource,
) -> Option<ExplanationPayload> {
    let (_, expected, payload) = native_export_payload(record, loaded)?;
    let actual = record.explanation()?;
    (actual.id() == expected.id()
        && actual.kind() == expected.kind()
        && actual.subject() == expected.subject()
        && actual.source() == expected.source()
        && actual.required_snapshot() == expected.required_snapshot()
        && actual.required_artifact_hash() == expected.required_artifact_hash()
        && actual.summary_hash() == expected.summary_hash())
    .then_some(payload)
}

/// Authenticates the exact attached read-only command independently of display text.
pub fn native_export_semicolon_command(
    record: &DiagnosticRecord,
    fix: &FixSuggestion,
    loaded: &LoadedSource,
) -> bool {
    if native_export_semicolon_context(record, loaded).is_none() {
        return false;
    }
    let Some((fixes, _, _)) = native_export_payload(record, loaded) else {
        return false;
    };
    let expected = &fixes[1];
    let matches = |candidate: &FixSuggestion| {
        candidate.id() == expected.id()
            && candidate.producer_key() == expected.producer_key()
            && candidate.applicability() == expected.applicability()
            && candidate.safety() == expected.safety()
            && candidate.edits() == expected.edits()
            && candidate.command() == expected.command()
            && candidate.required_snapshot() == expected.required_snapshot()
            && candidate.required_text_hash() == expected.required_text_hash()
    };
    matches(fix) && record.fixes().iter().any(matches)
}

pub(crate) struct FrontendService {
    pub(crate) artifact_roots: Vec<(PackageId, PathBuf)>,
}

impl FrontendService {
    pub(crate) fn analyze_loaded(
        &self,
        inputs: crate::registry::SourceLoadInputs<'_>,
        package_root: PathBuf,
        workspace_summaries: &[(
            mizar_resolve::module_index::IndexedModuleId,
            ModuleLexicalSummary,
        )],
        source: SourceUnit,
    ) -> Result<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>, String> {
        Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(package_root)),
            DependencyLexicalProvider {
                inputs,
                artifact_roots: &self.artifact_roots,
                workspace_summaries,
            },
            MizarParserSeam,
        )
        .run_loaded(source)
        .map_err(|error| format!("{error:?}"))
    }
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
        if version.origin != SourceOrigin::Disk
            && inputs
                .captured
                .is_none_or(|capture| capture.native_sources().is_none())
        {
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
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
        ));
        let Some(parent) = unique(
            input
                .parent_outputs()
                .iter()
                .map(|handle| handle.as_output_ref())
                .filter(|handle| {
                    handle.phase() == &IrPipelinePhase::from(PipelinePhase::SourceLoad)
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
                (if inputs
                    .captured
                    .is_some_and(|capture| capture.native_sources().is_some())
                {
                    publisher.validate_output(snapshot, handle, OutputAuthorization::Published)
                } else {
                    publisher.validate_output(snapshot, handle, OutputAuthorization::CurrentPackage)
                })
                .is_ok()
                    && publisher.storage().validate_handle(handle).is_ok()
            })
        };
        if !parents_current()
            || input.parent_outputs().iter().any(|handle| {
                let handle = handle.as_output_ref();
                handle.phase() == &IrPipelinePhase::from(PipelinePhase::SourceLoad)
                    && handle != parent
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
        let source_input = if inputs
            .captured
            .is_some_and(|capture| capture.native_sources().is_some())
        {
            let Some((expected, input)) = inputs.native_source(version) else {
                return blocking();
            };
            if source.as_ref() != &expected {
                return blocking();
            }
            input
        } else {
            source_input
        };
        if (match &source_input.origin {
            SourceOriginInput::OpenBuffer { .. } => {
                source.canonical_open_buffer_bytes(&source_input)
            }
            _ => source.canonical_disk_bytes(),
        })
        .is_none()
            || source.source_id != version.source_id
            || source.package_id != version.package_id
            || source.module_path != version.module_path
            || source.normalized_path != version.normalized_path
            || source.edition != version.edition
            || source.source_hash != version.source_hash
            || source.origin != version.origin
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
                if leaf.phase() != &IrPipelinePhase::from(PipelinePhase::Frontend)
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
                let leaf_unit = IrWorkUnit::new(module_work_unit_label(
                    leaf_module.package.as_str(),
                    leaf_module.path.as_str(),
                ));
                let parent_lineages = leaf
                    .lineage()
                    .parents
                    .iter()
                    .map(|parent| publisher.registry().output_lineage(*parent))
                    .collect::<Option<Vec<_>>>()?;
                let source_lineage = unique(parent_lineages.iter().filter(|parent| {
                    parent.phase == IrPipelinePhase::from(PipelinePhase::SourceLoad)
                }))?;
                let leaf_package = unique(
                    inputs
                        .build_plan
                        .packages
                        .iter()
                        .filter(|package| package.package_id == leaf_module.package),
                )?;
                let PackagePlanSource::Workspace {
                    root: target_root,
                    source_root: target_source_root,
                    manifest_path: target_manifest,
                } = &leaf_package.source
                else {
                    return None;
                };
                let leaf_index_package = unique(
                    inputs
                        .module_index
                        .packages
                        .iter()
                        .filter(|package| package.package_id == leaf_module.package),
                )?;
                let mizar_build::module_index::PackageIndexSource::Workspace {
                    package_root: indexed_root,
                    source_root: indexed_source_root,
                    manifest_path: indexed_manifest,
                } = &leaf_index_package.source
                else {
                    return None;
                };
                if target_root != indexed_root
                    || target_source_root != indexed_source_root
                    || target_manifest != indexed_manifest
                    || leaf_package.version != leaf_index_package.version
                    || leaf_package.edition != leaf_version.edition
                    || leaf_index_package.edition != leaf_version.edition
                    || !safe_relative(target_root)
                    || !safe_relative(target_source_root)
                    || !safe_relative(target_manifest)
                    || workspace.join(target_source_root) != workspace.join(target_root).join("src")
                    || workspace.join(target_manifest)
                        != workspace.join(target_root).join("mizar.pkg")
                {
                    return None;
                }
                let leaf_source_key = source_input_hash(leaf_version);
                let native_leaf = if inputs
                    .captured
                    .is_some_and(|capture| capture.native_sources().is_some())
                {
                    let (expected, raw) = inputs.native_source(leaf_version)?;
                    if leaf_source != &expected {
                        return None;
                    }
                    Some(raw)
                } else {
                    None
                };
                if leaf_module == *module
                    || leaf.work_unit() != &leaf_unit
                    || (native_leaf.is_none() && leaf_version.origin != SourceOrigin::Disk)
                    || (match native_leaf.as_ref().map(|raw| &raw.origin) {
                        Some(SourceOriginInput::OpenBuffer { .. }) => {
                            output.canonical_open_buffer_bytes(native_leaf.as_ref()?)
                        }
                        _ => output.canonical_disk_bytes(),
                    })
                    .is_none()
                    || leaf_source.source_id != leaf_version.source_id
                    || leaf_source.package_id != leaf_version.package_id
                    || leaf_source.module_path != leaf_version.module_path
                    || leaf_source.normalized_path != leaf_version.normalized_path
                    || leaf_source.edition != leaf_version.edition
                    || leaf_source.source_hash != leaf_version.source_hash
                    || leaf_source.origin != leaf_version.origin
                    || leaf_entry.package_id != leaf_module.package
                    || leaf_entry.module_path != leaf_module.path
                    || leaf_entry.edition != leaf_version.edition
                    || leaf_root != target_source_root
                    || leaf_path != leaf_version.normalized_path.as_str()
                    || leaf_path != &format!("src/{source_relative_path}")
                    || source_relative_path.is_empty()
                    || index_package.version != package.version
                    || source_lineage.snapshot != snapshot
                    || source_lineage.phase != IrPipelinePhase::from(PipelinePhase::SourceLoad)
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
                {
                    return None;
                }
                let index = mizar_resolve::module_index::ModuleIndexInput::new(inputs.module_index);
                let resolution = mizar_resolve::imports::ImportPathResolver::new(index)
                    .resolve_frontend(&output, leaf_source)?;
                if !resolution.unresolved().is_empty() {
                    return None;
                }
                let mut targets = Vec::new();
                for import in resolution.resolved() {
                    let target = IndexedModuleId::new(
                        import.target().package().clone(),
                        import.target().path().clone(),
                    );
                    let entry = unique(
                        inputs
                            .module_index
                            .modules
                            .iter()
                            .filter(|entry| entry.module == target),
                    )?;
                    match &entry.location {
                        ModuleIndexLocation::WorkspaceFile { .. } => {
                            if target == leaf_module {
                                return None;
                            }
                        }
                        ModuleIndexLocation::DependencySummary { .. } => continue,
                        _ => return None,
                    }
                    let unit = IrWorkUnit::new(module_work_unit_label(
                        target.package.as_str(),
                        target.path.as_str(),
                    ));
                    if !targets.contains(&unit) {
                        targets.push(unit);
                    }
                }
                for parent in &parent_lineages {
                    if parent.output == source_lineage.output {
                        continue;
                    }
                    if parent.snapshot != snapshot
                        || parent.phase != IrPipelinePhase::from(PipelinePhase::Frontend)
                        || parent.output_kind != OutputKind::new("FrontendOutput")
                    {
                        return None;
                    }
                    let position = targets.iter().position(|unit| unit == &parent.work_unit)?;
                    targets.remove(position);
                }
                if !targets.is_empty() {
                    return None;
                }
                let mut named_inputs = vec![
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
                for (ordinal, digest) in dependencies.iter().enumerate() {
                    named_inputs.push(NamedInputHash {
                        name: format!("dependency.{ordinal}"),
                        domain: "mizar-frontend/dependency-summary/v1".to_owned(),
                        digest: *digest,
                    });
                }
                named_inputs.sort_by(|left, right| {
                    left.name
                        .cmp(&right.name)
                        .then_with(|| left.domain.cmp(&right.domain))
                        .then_with(|| left.digest.as_bytes().cmp(right.digest.as_bytes()))
                });
                if leaf.lineage().named_input_hashes != named_inputs {
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
                    package_version: Some(leaf_package.version.to_string()),
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
        let Ok(output) = self.analyze_loaded(
            inputs,
            package_root,
            &workspace_summaries,
            source.as_ref().clone(),
        ) else {
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
                let index = ModuleIndexInput::new(inputs.module_index);
                let current = index.resolver_module_id(module);
                let resolution =
                    ImportPathResolver::new(index).resolve_frontend(&output, source.as_ref())?;
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
        let index = mizar_resolve::module_index::ModuleIndexInput::new(inputs.module_index);
        let Some(resolution) = mizar_resolve::imports::ImportPathResolver::new(index)
            .resolve_frontend(&output, source.as_ref())
        else {
            return blocking();
        };
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
        let Some(storage_bytes) = (match &source_input.origin {
            SourceOriginInput::OpenBuffer { .. } => {
                output.canonical_open_buffer_bytes(&source_input)
            }
            _ => output.canonical_disk_bytes(),
        }) else {
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
        let expected_source = output.source.clone();
        let phase = IrPipelinePhase::from(PipelinePhase::Frontend);
        let output_kind = OutputKind::new("FrontendOutput");
        if !parents_current() {
            return blocking();
        }
        let (origin, target) = source_publication_route(&version.origin);
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
                FrontendOutput::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .filter(|output| {
                        output
                            .source
                            .matches_canonical_storage(&expected_source, &decode_input)
                    })
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
            origin,
            target,
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
            let mut details = DiagnosticDetails::from_entries([
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
            let native = diagnostic
                .export_semicolon_insertion
                .filter(|insertion| {
                    number == 41
                        && prefix == "syntax"
                        && class == "syntax"
                        && insertion.source_id == source.source_id
                        && insertion.start == insertion.end
                        && insertion.start == primary.range().start
                        && source.line_map.validate_range(*insertion).is_ok()
                })
                .and_then(|insertion| {
                    let payload = export_semicolon_payload(
                        insertion,
                        snapshot,
                        code,
                        descriptor.semantic_name,
                    )?;
                    details
                        .insert(
                            EXPORT_INSERTION_DETAIL,
                            DiagnosticDetailValue::Source(insertion),
                        )
                        .ok()?;
                    Some(payload)
                });
            let (fixes, explanation) = native.map_or_else(
                || (Vec::new(), None),
                |(fixes, handle, _)| (fixes, Some(handle)),
            );
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
                fixes,
                explanation,
            })
            .ok()
        })
        .collect()
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

    struct EmptyLexicalProvider;
    impl mizar_frontend::lexical_env::LexicalSummaryProvider for EmptyLexicalProvider {
        fn resolve_imports(
            &self,
            _: &mizar_frontend::lexical_env::LexicalEnvironmentRequest<'_>,
        ) -> Result<
            mizar_frontend::lexical_env::ResolvedImports,
            mizar_frontend::lexical_env::FrontendLexicalEnvironmentError,
        > {
            Ok(mizar_frontend::lexical_env::ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: Vec::new(),
            })
        }
    }

    fn native_drafts(text: &str) -> (BuildSnapshotId, LoadedSource, Vec<DiagnosticDraft>) {
        let (snapshot, loaded) = loaded_source(text);
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(".")),
            EmptyLexicalProvider,
            MizarParserSeam,
        )
        .run_loaded(source_unit_from_loaded(
            loaded.clone(),
            "src/test.miz".into(),
        ))
        .unwrap();
        let drafts =
            convert_frontend_diagnostics(&output.source, snapshot, &output.diagnostics).unwrap();
        (snapshot, loaded, drafts)
    }

    fn native_index(
        snapshot: BuildSnapshotId,
        drafts: Vec<DiagnosticDraft>,
    ) -> mizar_diagnostics::aggregator::BuildDiagnosticIndex {
        use mizar_diagnostics::sink::{DiagnosticProducerScope, DiagnosticSink};
        let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
            DiagnosticPhase::Frontend,
            snapshot,
            "native-export-test",
        ));
        for draft in drafts {
            sink.emit(draft).unwrap();
        }
        mizar_diagnostics::aggregator::BuildDiagnosticIndex::from_batches(
            snapshot,
            vec![sink.into_batch()],
        )
        .unwrap()
    }

    #[test]
    fn native_export_payload_only_attaches_at_complete_export_before_item() {
        for text in [
            "export Std\ndefinition end;",
            "\u{feff}:: é\r\nexport Std, Other\r\ndefinition end;\r\n",
        ] {
            let (snapshot, loaded, drafts) = native_drafts(text);
            let index = native_index(snapshot, drafts);
            let record = index
                .records()
                .iter()
                .find(|record| record.explanation().is_some())
                .unwrap();
            let offset = loaded.text.find("definition").unwrap();
            let insertion = &record
                .fixes()
                .iter()
                .find(|fix| !fix.edits().is_empty())
                .unwrap()
                .edits()[0];
            assert_eq!(
                insertion.range(),
                SourceRange {
                    source_id: loaded.source_id,
                    start: offset,
                    end: offset
                }
            );
            assert_eq!(insertion.replacement(), ";");
            assert_eq!(insertion.expected_text(), Some(""));
            let DiagnosticPrimaryLocation::Span(primary) = record.primary_location() else {
                panic!("range");
            };
            assert_eq!(primary.range().start, offset);
            assert!(primary.range().end > offset);
            let payload = native_export_semicolon_context(record, &loaded).unwrap();
            assert_eq!(
                payload.text(),
                format!(
                    "{{\"category\":\"syntax.missing_semicolon\",\"expected\":\";\",\"insertion_offset\":{offset},\"site\":\"export-before-item\"}}\n"
                )
            );
            assert_eq!(payload.format(), ExplanationPreviewFormat::StructuredText);
            assert!(!payload.truncated());
            let ExplanationSourceRef::CacheRecord { content_hash, .. } =
                record.explanation().unwrap().source()
            else {
                panic!("cache");
            };
            assert_eq!(
                *content_hash,
                Some(export_backing_hash("content", payload.text().as_bytes()))
            );
            assert_eq!(
                payload.summary_hash(),
                Some(export_backing_hash("summary", payload.text().as_bytes()))
            );
            assert_ne!(*content_hash, payload.summary_hash());
            assert!(
                record
                    .fixes()
                    .iter()
                    .any(|fix| native_export_semicolon_command(record, fix, &loaded))
            );
        }
        for text in [
            "export Std",
            "export Std,\ndefinition end;",
            "export Std.\ndefinition end;",
            "export\ndefinition end;",
            "import Std\ndefinition end;",
            "theorem T lemma L;",
        ] {
            let (_, _, drafts) = native_drafts(text);
            assert!(!drafts.is_empty(), "{text}");
            assert!(
                drafts
                    .iter()
                    .all(|draft| draft.fixes().is_empty() && draft.explanation().is_none()),
                "{text}"
            );
        }
    }

    #[test]
    fn native_export_occurrences_keep_canonical_record_and_command_associations() {
        let (snapshot, loaded, mut drafts) =
            native_drafts("export Std\nexport Other\ndefinition end;");
        let forward = native_index(snapshot, drafts.clone());
        drafts.reverse();
        let reverse = native_index(snapshot, drafts);
        assert_eq!(forward, reverse);
        assert_eq!(forward.len(), 2);
        let first = &forward.records()[0];
        let second = &forward.records()[1];
        assert_ne!(
            first.explanation().unwrap().id(),
            second.explanation().unwrap().id()
        );
        let command = first
            .fixes()
            .iter()
            .find(|fix| native_export_semicolon_command(first, fix, &loaded))
            .unwrap();
        assert!(!native_export_semicolon_command(second, command, &loaded));
        assert_ne!(
            native_export_semicolon_context(first, &loaded).unwrap(),
            native_export_semicolon_context(second, &loaded).unwrap()
        );
        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot).unwrap();
        let mut foreign = loaded.clone();
        foreign.source_id = ids.next_source_id(snapshot).unwrap();
        assert!(native_export_semicolon_context(first, &foreign).is_none());
        assert!(!native_export_semicolon_command(first, command, &foreign));
    }

    #[test]
    fn native_export_invalid_backing_refuses_context_without_removing_local_edit() {
        let (snapshot, loaded, drafts) = native_drafts("export Std\ndefinition end;");
        let index = native_index(snapshot, drafts);
        let record = &index.records()[0];
        let handle = record.explanation().unwrap();
        let make = |explanation, fixes| {
            DiagnosticRecord::from_draft(
                DiagnosticDraft::new(DiagnosticDraftInput {
                    source_snapshot: snapshot,
                    code: record.code(),
                    phase: record.phase(),
                    category: record.category(),
                    stable_detail_key: record.stable_detail_key().to_owned(),
                    message: record.message().to_owned(),
                    primary_location: record.primary_location().clone(),
                    secondary_spans: record.secondary_spans().to_vec(),
                    notes: record.notes().to_vec(),
                    details: record.details().clone(),
                    fixes,
                    explanation,
                })
                .unwrap(),
                record.handle(),
                record.freshness().clone(),
                Vec::new(),
            )
            .unwrap()
        };
        for variant in 0..7 {
            let mut input = ExplanationHandleInput {
                id: handle.id().clone(),
                kind: handle.kind(),
                subject: handle.subject().clone(),
                source: handle.source().clone(),
                required_snapshot: handle.required_snapshot(),
                required_artifact_hash: None,
                summary_hash: handle.summary_hash(),
                preview: None,
            };
            match variant {
                0 => input.id = ExplanationHandleId::new("foreign").unwrap(),
                1 => input.kind = ExplanationKind::Internal,
                2 => {
                    input.source = ExplanationSourceRef::CacheRecord {
                        cache_key: "foreign".to_owned(),
                        content_hash: Some(Hash::from_bytes([1; 32])),
                    }
                }
                3 => {
                    input.source = ExplanationSourceRef::CacheRecord {
                        cache_key: match handle.source() {
                            ExplanationSourceRef::CacheRecord { cache_key, .. } => {
                                cache_key.clone()
                            }
                            _ => unreachable!(),
                        },
                        content_hash: None,
                    }
                }
                4 => input.summary_hash = Some(Hash::from_bytes([1; 32])),
                5 => input.required_snapshot = None,
                6 => input.required_artifact_hash = Some(Hash::from_bytes([1; 32])),
                _ => unreachable!(),
            }
            let invalid = make(
                Some(ExplanationHandle::new(input).unwrap()),
                record.fixes().to_vec(),
            );
            assert!(native_export_semicolon_context(&invalid, &loaded).is_none());
            assert!(invalid.fixes().iter().any(|fix| !fix.edits().is_empty()));
            assert!(
                invalid
                    .fixes()
                    .iter()
                    .all(|fix| !native_export_semicolon_command(&invalid, fix, &loaded))
            );
        }
        let no_handle = make(None, record.fixes().to_vec());
        assert!(native_export_semicolon_context(&no_handle, &loaded).is_none());
        let no_command = make(
            Some(handle.clone()),
            record
                .fixes()
                .iter()
                .filter(|fix| !fix.edits().is_empty())
                .cloned()
                .collect(),
        );
        assert!(native_export_semicolon_context(&no_command, &loaded).is_some());
        let command = record
            .fixes()
            .iter()
            .find(|fix| fix.command().is_some())
            .unwrap();
        assert!(!native_export_semicolon_command(
            &no_command,
            command,
            &loaded
        ));
    }

    fn source() -> (BuildSnapshotId, SourceUnit) {
        let (snapshot, loaded) = loaded_source("éx\n");
        (
            snapshot,
            source_unit_from_loaded(loaded, "src/test.miz".into()),
        )
    }

    fn loaded_source(text: &str) -> (BuildSnapshotId, LoadedSource) {
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
        fs::write(root.join("src/test.miz"), text).unwrap();
        let input = SourceInput {
            package_id: PackageId::new("test"),
            module_path: ModulePath::new("test"),
            normalized_path: normalize_path(&root, Path::new("src/test.miz")).unwrap(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: "src/test.miz".into(),
            },
        };
        let source = mizar_session::SourceLoader::load(
            &DiskSourceLoader::new(&root),
            snapshot,
            input,
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
            export_semicolon_insertion: None,
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
