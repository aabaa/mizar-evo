use super::*;

impl<'a> crate::registry::SourceLoadInputs<'a> {
    /// Resolves a complete original cohort after Driver has filtered fresh owner handoffs.
    pub(crate) fn complete_snapshot_publication(
        self,
        graph: &mizar_build::task_graph::TaskGraph,
        run: &crate::driver::DriverSchedulerRun,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    ) -> Option<crate::events::SnapshotPublication> {
        use mizar_artifact::{
            manifest::{ManifestFileReadOptions, read_manifest_file},
            store::{
                CanonicalHashDomain, HashClass, PublishedArtifactPath,
                PublishedArtifactReadOptions, artifact_hash_domain, read_published_artifact,
            },
            verified_artifact::{
                VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact_hash_excluded_paths,
                verified_artifact_json,
            },
        };
        let captured = self.captured?;
        if captured.snapshot() != self.snapshot {
            return None;
        }
        captured.native_sources()?;
        publisher.validate_current_snapshot(self.snapshot.id).ok()?;
        let modules = self
            .module_index
            .modules
            .iter()
            .filter(|entry| matches!(entry.location, ModuleIndexLocation::WorkspaceFile { .. }))
            .collect::<Vec<_>>();
        if modules.is_empty()
            || modules.len() != self.snapshot.source_versions.len()
            || graph.tasks().iter().any(|task| {
                !run.task_states.iter().any(|state| {
                    state.task_id == task.id
                        && matches!(
                            state.state,
                            mizar_build::scheduler::TaskState::Completed
                                | mizar_build::scheduler::TaskState::CacheHit
                        )
                })
            })
        {
            return None;
        }
        let mut manifests = Vec::new();
        let mut artifacts = Vec::new();
        let mut development_artifacts = Vec::new();
        for module in modules {
            let version = unique(self.snapshot.source_versions.iter().filter(|version| {
                version.package_id == module.package_id && version.module_path == module.module_path
            }))?;
            self.native_source(version)?;
            let task = unique(graph.tasks().iter().filter(|task| {
                task.kind == mizar_build::task_graph::TaskKind::ArtifactCommit
                    && task.phases == [PipelinePhase::ArtifactCommit]
                    && task.unit
                        == (WorkUnit::Module {
                            module: module.module.clone(),
                        })
            }))?;
            let [result] = run.phase_results.get(&task.id)?.as_slice() else {
                return None;
            };
            let commit = result.artifact_commit.as_ref()?;
            let [output] = result.output_refs.as_slice() else {
                return None;
            };
            if result.status != PhaseStatus::Complete || !result.diagnostics.is_empty() {
                return None;
            }
            let package = unique(
                self.build_plan
                    .packages
                    .iter()
                    .filter(|package| package.package_id == module.package_id),
            )?;
            let PackagePlanSource::Workspace { root, .. } = &package.source else {
                return None;
            };
            let root = PathBuf::from(self.snapshot.workspace_root.as_str())
                .join(root)
                .join(&commit.manifest().artifact_root);
            let manifest = read_manifest_file(
                &root,
                ManifestFileReadOptions {
                    validate_references: true,
                    validate_witness_files: true,
                    validate_development_artifacts: true,
                    expected_manifest_hash: Some(commit.write().artifact_hash),
                },
            )
            .ok()?;
            if manifest.manifest != *commit.manifest() {
                return None;
            }
            let [entry] = manifest.manifest.modules.as_slice() else {
                return None;
            };
            let handle = publisher
                .storage()
                .typed_handle::<mizar_artifact::verified_artifact::VerifiedArtifact>(
                    output,
                    &OutputKind::new("SourceTheoremArtifactData"),
                )
                .ok()?;
            publisher
                .validate_output(self.snapshot.id, output, OutputAuthorization::Published)
                .ok()?;
            let artifact = publisher.storage().get(&handle).ok()?;
            let value = verified_artifact_json(&artifact).ok()?;
            if entry.module != artifact.module
                || artifact.module.package_id != version.package_id.as_str()
                || artifact.module.module_path != version.module_path.as_str()
                || artifact.provenance.captured_source
                    != Some((
                        version.source_id,
                        version.origin.clone(),
                        version.source_hash,
                    ))
                || artifact.source_hash != version.source_hash
                || artifact.source_file != version.normalized_path.as_str()
                || artifact.schema_version != entry.artifact_hash.schema_version
                || artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version)
                    .hash(&value, &artifact_hash_excluded_paths())
                    != entry.artifact_hash.digest
            {
                return None;
            }
            for entry in &manifest.manifest.development_artifacts {
                let (hash, class) = match (&entry.artifact_hash, &entry.diagnostic_hash) {
                    (Some(hash), None) => (hash, HashClass::Artifact),
                    (None, Some(hash)) => (hash, HashClass::Diagnostic),
                    _ => return None,
                };
                let path = PublishedArtifactPath::new(&entry.path).ok()?;
                let domain =
                    CanonicalHashDomain::new(class, &hash.schema_family, hash.schema_version);
                let read =
                    read_published_artifact(&root, &path, PublishedArtifactReadOptions::default())
                        .ok()?;
                if domain.hash(&read.value, &[]) != hash.digest {
                    return None;
                }
                development_artifacts.push(read);
            }
            if artifact
                .diagnostics
                .iter()
                .filter_map(|diagnostic| diagnostic.explanation_ref.as_ref())
                .any(|reference| {
                    !manifest
                        .manifest
                        .development_artifacts
                        .iter()
                        .any(|entry| entry.diagnostic_hash.as_ref() == Some(reference))
                })
            {
                return None;
            }
            artifacts.push(artifact);
            manifests.push(manifest);
        }
        let diagnostics = mizar_diagnostics::aggregator::BuildDiagnosticIndex::from_batches(
            self.snapshot.id,
            run.phase_results
                .values()
                .flatten()
                .flat_map(|result| result.diagnostics.iter().cloned())
                .collect(),
        )
        .ok()?;
        publisher.validate_current_snapshot(self.snapshot.id).ok()?;
        Some(crate::events::SnapshotPublication {
            captured: captured.clone(),
            diagnostics,
            manifests,
            artifacts,
            development_artifacts,
        })
    }

    /// Projects real source rows into an unpublished module draft and retains its staged witness.
    pub fn project_native_theorem_artifact_draft(
        self,
        input: &PhaseInput,
        publisher: &Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (&str, &std::path::Path),
    ) -> Result<
        Option<(
            mizar_ir::projection::VerifiedArtifactDraft,
            mizar_proof::witness_store::ProofWitnessStagedRef,
        )>,
        String,
    > {
        use mizar_artifact::{
            module_summary::{MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummary, SourceRangeSummary},
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{
                CanonicalHashDomain, CanonicalJson, HashClass, PublishedArtifactPath,
                PublishedArtifactReadOptions, artifact_hash_domain, read_published_artifact,
                write_published_artifact,
            },
            verified_artifact::{
                BuildProvenance, ExportVisibility, ExpressionMetadata, ObligationMetadata,
                ObligationStatus, StatementMetadata, VerifiedExport,
            },
        };
        use mizar_checker::{
            source_atomic_formula::{SourceAtomicFormulaKind, SourceAtomicFormulaRecovery},
            source_statement::{SourceStatementRecovery, SourceTheoremRole, SourceTheoremStatus},
            source_term::{SourcePrimaryTermKind, SourcePrimaryTermRecovery},
            source_type::{SourceTypeApplicationForm, SourceTypeHead},
            type_checker::{NormalizedTypeStatus, TypeHeadRef},
            typed_ast::{NodeRecoveryState, TypeEntryActual, TypeStatus, TypingState},
        };
        use mizar_ir::{
            dispatch_input::PhaseDispatchInputBundle,
            projection::{ArtifactProjectionService, ProjectVerifiedArtifactInput},
        };
        use mizar_proof::{
            policy::{KernelEvidenceOrigin, KernelPolicyInput},
            selection::{
                ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass, VcProofSelection,
                merge_artifact_proof_selections, select_winner,
            },
            status::{
                ArtifactStatusPublication, CurrentArtifactObligationStatus, ProjectedProofStatus,
                ProofObligationIdentity, ProofStatusProjectionInput, TrustedUsedAxiomsRef,
                project_status,
            },
        };
        let no_draft = Ok(None);
        let fail = |suffix: &str| format!("native theorem artifact draft {suffix}");
        if input.snapshot != self.snapshot.id {
            return Err(fail("snapshot mismatch"));
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err(fail("requires a module"));
        };
        if input.parent_outputs().len() != 6 {
            return Err(fail("requires six parents"));
        }
        let roles = [
            ("SignatureCollection", "SourceTheoremModuleSummary"),
            ("Elaboration", "SourceTheoremCore"),
            ("VcGenerate", "SourceTheoremVC"),
            ("VcDischarge", "SourceTheoremDischarge"),
            ("KernelCheck", "SourceTheoremCandidateCheck"),
            ("KernelCheck", "SourceTheoremCandidateWitnessFile"),
        ];
        let parents = roles.map(|(phase, kind)| {
            unique(input.parent_outputs().iter().filter(|parent| {
                parent.as_output_ref().phase() == &IrPipelinePhase::new(phase)
                    && parent.as_output_ref().output_kind() == &OutputKind::new(kind)
            }))
        });
        let [
            Some(summary_parent),
            Some(core_parent),
            Some(vc_parent),
            Some(discharge_parent),
            Some(kernel_parent),
            Some(file_parent),
        ] = parents
        else {
            return Err(fail("requires unique source roles"));
        };
        let original = PhaseInput::new(
            input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                input.identities().dependency_hashes().to_vec(),
                vec![
                    core_parent.clone(),
                    vc_parent.clone(),
                    discharge_parent.clone(),
                    kernel_parent.clone(),
                ],
            )
            .map_err(|error| error.to_string())?,
        );
        let Some((selection, ((projection, slice, witness_draft), file, reference))) = self
            .native_theorem_candidate_witness_payload(
                &original, publisher, vc_id, encoding, policies, staging.0,
            )?
        else {
            return no_draft;
        };
        let version = unique(self.snapshot.source_versions.iter().filter(|version| {
            version.package_id == module.package && version.module_path == module.path
        }))
        .ok_or_else(|| fail("requires one captured source"))?;
        let (source, _) = self
            .native_source(version)
            .ok_or_else(|| fail("capture binding failed"))?;
        self.validate_native_lock_content()?;
        self.validate_native_config_content()?;
        if input.identities().input_hash() != source_input_hash(version)
            || !input.identities().dependency_hashes().is_empty()
            || !self.snapshot.dependency_artifacts.is_empty()
            || !self.module_index.dependency_summaries.is_empty()
        {
            return no_draft;
        }
        let storage = publisher.storage();
        let work_unit = IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        ));
        let named = vec![NamedInputHash {
            name: "source".to_owned(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
            digest: source_input_hash(version),
        }];
        publisher
            .validate_current_snapshot(input.snapshot)
            .map_err(|error| error.to_string())?;
        for parent in input.parent_outputs() {
            let parent = parent.as_output_ref();
            storage
                .validate_handle(parent)
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(input.snapshot, parent, OutputAuthorization::Published)
                .map_err(|error| error.to_string())?;
            if parent.snapshot() != input.snapshot
                || parent.work_unit() != &work_unit
                || parent.schema_version() != SchemaVersion::new(1)
                || parent.lineage().named_input_hashes != named
            {
                return Err(fail("parent binding mismatch"));
            }
        }
        let file_ref = file_parent.as_output_ref();
        if file_ref.lineage().parents != vec![kernel_parent.as_output_ref().output()] {
            return Err(fail("FILE Kernel parent mismatch"));
        }
        let typed_file = storage
            .typed_handle::<CanonicalJson>(
                file_ref,
                &OutputKind::new("SourceTheoremCandidateWitnessFile"),
            )
            .map_err(|error| error.to_string())?;
        if *storage
            .get(&typed_file)
            .map_err(|error| error.to_string())?
            != file
        {
            return Err(fail("complete witness file mismatch"));
        }
        let typed_summary = storage
            .typed_handle::<ModuleSummary>(
                summary_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremModuleSummary"),
            )
            .map_err(|error| error.to_string())?;
        let stored_summary = storage
            .get(&typed_summary)
            .map_err(|error| error.to_string())?;
        let Some(summary) = self.native_theorem_module_summary_candidate(module)? else {
            return no_draft;
        };
        if *stored_summary != summary {
            return Err(fail("complete summary mismatch"));
        }
        if !summary.reexports.is_empty()
            || !summary.dependency_interfaces.is_empty()
            || summary.exported_symbols.len() > 1
        {
            return no_draft;
        }
        let Some(source_metadata) =
            self.native_theorem_source_metadata(module, "native theorem artifact draft")?
        else {
            return no_draft;
        };
        let (arena, env, inference, typed, metadata) = source_metadata.as_ref();
        let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(arena, env)?;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                arena, typed, &scope, env, &labels, &resolved, None,
            )?;
        let [(owner, owner_visibility)] = check.owners() else {
            return no_draft;
        };
        if owner.role != SourceTheoremRole::Theorem
            || owner.status != SourceTheoremStatus::Unmodified
            || owner.recovery != SourceStatementRecovery::Normal
            || !env.imports().is_empty()
            || !env.exports().is_empty()
            || !typed.diagnostics().is_empty()
            || !inference.diagnostics().is_empty()
            || !metadata.diagnostics().is_empty()
        {
            return no_draft;
        }
        let symbol = env
            .symbols()
            .get(&owner.symbol)
            .ok_or_else(|| fail("checked owner symbol missing"))?;
        if symbol.kind() != mizar_resolve::env::SymbolKind::Theorem {
            return no_draft;
        }
        let (visibility, export) = match (
            symbol.visibility(),
            symbol.export_status(),
            summary.exported_symbols.as_slice(),
        ) {
            (
                mizar_resolve::env::Visibility::Public,
                mizar_resolve::env::ExportStatus::Exported,
                [export],
            ) => ("public", Some(export)),
            (
                mizar_resolve::env::Visibility::Private,
                mizar_resolve::env::ExportStatus::LocalOnly,
                [],
            ) => ("private", None),
            _ => return no_draft,
        };
        if *owner_visibility != visibility {
            return no_draft;
        }
        if owner.symbol.module() != arena.module()
            || owner.source_range.source_id != version.source_id
            || arena.source_id() != version.source_id
            || symbol.origin().source_id() != version.source_id
            || symbol.origin().module_id() != arena.module()
            || symbol.origin().is_recovered()
            || symbol.origin().import_edge().is_some()
            || summary.source_hash != version.source_hash
            || export.is_some_and(|export| {
                export.origin_id != owner.symbol.fqn().as_str()
                    || export.fully_qualified_name != owner.symbol.fqn().as_str()
                    || export.namespace_path
                        != symbol
                            .namespace()
                            .as_str()
                            .split('.')
                            .map(str::to_owned)
                            .collect::<Vec<_>>()
                    || export.visibility != "public"
                    || export.declaration_kind != "theorem"
                    || export.proof_status.is_some()
                    || export.rendered_signature.is_empty()
                    || export.source_range.start_byte != owner.source_range.start as u64
                    || export.source_range.end_byte != owner.source_range.end as u64
            })
        {
            return Err(fail("checked owner export mismatch"));
        }
        let mizar_checker::typed_ast::TypedSiteRef::Node(owner_node) = owner.site else {
            return no_draft;
        };
        let mut formula = owner_node;
        let formula_range = loop {
            let Some(node) = typed.nodes().node(formula) else {
                return no_draft;
            };
            let Some(original) = node.resolved_node.and_then(|id| arena.arena().node(id)) else {
                return no_draft;
            };
            let mizar_session::SourceAnchor::Range(range) = node.anchor else {
                return no_draft;
            };
            let original_kind = format!("{:?}", original.kind());
            let kind = original_kind.as_str();
            if (formula == owner_node && kind != "TheoremItem")
                || (formula != owner_node
                    && !matches!(
                        kind,
                        "Proposition" | "FormulaExpression" | "QuantifiedFormula(Universal)"
                    ))
            {
                return no_draft;
            }
            if node.kind.as_str() != kind
                || node.recovery != NodeRecoveryState::Normal
                || original.origin().is_recovered()
                || original.origin().import_edge().is_some()
                || original.origin().source_id() != version.source_id
                || original.origin().module_id() != arena.module()
                || original.origin().anchor() != &node.anchor
                || range.source_id != version.source_id
                || range.start >= range.end
                || range.start < owner.source_range.start
                || range.end > owner.source_range.end
                || (formula == owner_node && range != owner.source_range)
            {
                return no_draft;
            }
            if kind == "QuantifiedFormula(Universal)" {
                break range;
            }
            let mut children = node.children.iter().copied().filter(|child| {
                typed
                    .nodes()
                    .node(*child)
                    .is_some_and(|child| !child.kind.as_str().starts_with("Token("))
            });
            let Some(child) = children.next() else {
                return no_draft;
            };
            if formula != owner_node && children.next().is_some() {
                return no_draft;
            }
            if typed
                .nodes()
                .node(child)
                .and_then(|child| child.resolved_node)
                .is_none_or(|child| !original.children().contains(&child))
            {
                return no_draft;
            }
            formula = child;
        };
        let Some(elaborated) = source
            .source_text
            .get(formula_range.start..formula_range.end)
        else {
            return no_draft;
        };
        let statements = vec![StatementMetadata {
            item: owner.symbol.fqn().as_str().to_owned(),
            source_range: SourceRangeSummary {
                start_byte: formula_range.start as u64,
                end_byte: formula_range.end as u64,
            },
            elaborated: elaborated.to_owned(),
            reserve_binders: Vec::new(),
            resolved_symbols: std::collections::BTreeMap::new(),
        }];
        let exports = export
            .into_iter()
            .map(|export| VerifiedExport {
                origin_id: export.origin_id.clone(),
                fully_qualified_name: export.fully_qualified_name.clone(),
                namespace_path: export.namespace_path.clone(),
                visibility: ExportVisibility::Public,
                export_kind: export.declaration_kind.clone(),
                source_range: export.source_range,
                rendered_signature: export.rendered_signature.clone(),
                interface_fingerprint: ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    MODULE_SUMMARY_SCHEMA_FAMILY,
                    summary.schema_version,
                    export.interface_fingerprint,
                ),
                proof_status: None,
                documentation_ref: None,
            })
            .collect();
        let Some(primary) = typed.source_term() else {
            return no_draft;
        };
        let Some(atomic) = typed.source_atomic_formula() else {
            return no_draft;
        };
        if metadata.expr_metadata().is_empty()
            || metadata.expr_metadata().len() != primary.terms().len() + atomic.formulas().len()
            || !metadata.collection_candidates().is_empty()
            || !metadata.expanded_candidates().is_empty()
            || !metadata.template_expansions().is_empty()
            || !metadata.viable_candidates().is_empty()
            || !metadata.viability_decisions().is_empty()
            || !metadata.specificity_graphs().is_empty()
            || !metadata.resolved_overloads().is_empty()
            || !metadata.inserted_coercions().is_empty()
            || !atomic.predicate_heads().is_empty()
            || !atomic.candidates().is_empty()
        {
            return no_draft;
        }
        let mut sites = std::collections::BTreeSet::new();
        for (_, term) in primary.terms().iter() {
            if term.kind() != SourcePrimaryTermKind::VariableReference
                || term.recovery() != SourcePrimaryTermRecovery::Normal
                || !sites.insert(term.site().clone())
                || metadata
                    .expr_metadata()
                    .get_by_site(term.site())
                    .is_none_or(|row| {
                        row.source_range != term.source_range() || row.final_type.is_none()
                    })
            {
                return no_draft;
            }
        }
        for (_, formula) in atomic.formulas().iter() {
            if formula.kind() != SourceAtomicFormulaKind::Equality
                || formula.recovery() != SourceAtomicFormulaRecovery::Normal
                || !sites.insert(formula.site().clone())
                || metadata
                    .expr_metadata()
                    .get_by_site(formula.site())
                    .is_none_or(|row| row.source_range != formula.source_range())
            {
                return no_draft;
            }
        }
        let mut expressions = Vec::with_capacity(sites.len());
        for (_, row) in metadata.expr_metadata().canonical_iter() {
            let range = row.source_range;
            let Some(node) = typed.nodes().node(row.typed_site.node()) else {
                return no_draft;
            };
            if !sites.remove(&row.typed_site)
                || range.source_id != version.source_id
                || node.anchor != mizar_session::SourceAnchor::Range(range)
                || node.recovery != NodeRecoveryState::Normal
                || row.overload.is_some()
                || !row.inserted_views.is_empty()
                || !row.diagnostics.is_empty()
                || range.start >= range.end
            {
                return no_draft;
            }
            let Some(surface) = source.source_text.get(range.start..range.end) else {
                return no_draft;
            };
            let inferred_type = if let Some(id) = row.final_type {
                let Some(ty) = inference.normalized_types().get(id) else {
                    return no_draft;
                };
                if ty.status != NormalizedTypeStatus::Known
                    || ty.head != TypeHeadRef::BuiltinSet
                    || !ty.args.is_empty()
                    || !ty.attributes.positive().is_empty()
                    || !ty.attributes.negative().is_empty()
                {
                    return no_draft;
                }
                Some("set".to_owned())
            } else {
                None
            };
            expressions.push(ExpressionMetadata {
                expression_id: row.expr.as_str().to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: range.start as u64,
                    end_byte: range.end as u64,
                },
                expression_kind: node.kind.as_str().to_owned(),
                rendered_surface: surface.to_owned(),
                inferred_type,
                resolved_symbol: None,
                inserted_coercions: Vec::new(),
                active_thesis: None,
                overload_resolution: None,
            });
        }
        if !sites.is_empty() {
            return no_draft;
        }
        let Some(written_types) = typed.source_type() else {
            return no_draft;
        };
        let Some(context) = typed.source_context() else {
            return no_draft;
        };
        if written_types.source_id() != version.source_id
            || written_types.module_id() != arena.module()
            || context.source_id() != version.source_id
            || context.module_id() != arena.module()
            || written_types.expressions().is_empty()
            || written_types.applications().len() != context.declarations().len()
            || !written_types.arguments().is_empty()
            || !written_types.definition_returns().is_empty()
            || !written_types.mode_rhs().is_empty()
            || !written_types.structure_members().is_empty()
        {
            return no_draft;
        }
        let mut type_sites = std::collections::BTreeSet::new();
        let mut bindings = std::collections::BTreeSet::new();
        let mut ordinals = std::collections::BTreeSet::new();
        for (root_id, root) in written_types.expressions().iter() {
            let range = root.source_range();
            let head_range = root.head_range();
            let Some(node) = typed.nodes().node(root.site().node()) else {
                return no_draft;
            };
            let Some(head) = typed.nodes().node(root.head_site().node()) else {
                return no_draft;
            };
            if root.source_id() != version.source_id
                || root.module_id() != arena.module()
                || root.form() != SourceTypeApplicationForm::Bare
                || root.head() != &SourceTypeHead::BuiltinSet
                || root.recovery() != NodeRecoveryState::Normal
                || root.site() != &mizar_checker::typed_ast::TypedSiteRef::Node(root.site().node())
                || root.head_site()
                    != &mizar_checker::typed_ast::TypedSiteRef::Node(root.head_site().node())
                || !type_sites.insert(root.site().clone())
                || node.kind.as_str() != "TypeExpression"
                || head.kind.as_str() != "TypeHead"
                || node.recovery != NodeRecoveryState::Normal
                || head.recovery != NodeRecoveryState::Normal
                || node.children != [root.head_site().node()]
                || node.anchor != mizar_session::SourceAnchor::Range(range)
                || head.anchor != mizar_session::SourceAnchor::Range(head_range)
                || range.source_id != version.source_id
                || head_range.source_id != version.source_id
                || range.start >= range.end
                || head_range.start < range.start
                || head_range.end > range.end
                || head_range.start >= head_range.end
            {
                return no_draft;
            }
            let Some(surface) = source.source_text.get(range.start..range.end) else {
                return no_draft;
            };
            if surface != root.spelling()
                || source.source_text.get(head_range.start..head_range.end)
                    != Some(root.head_spelling())
            {
                return no_draft;
            }
            let mut first: Option<usize> = None;
            let mut root_type = None;
            for (_, application) in written_types
                .applications()
                .iter()
                .filter(|(_, application)| application.root() == root_id)
            {
                let Some(declaration) = unique(
                    context
                        .declarations()
                        .iter()
                        .map(|(_, row)| row)
                        .filter(|row| {
                            row.binding == application.binding()
                                && row.source_ordinal == application.source_ordinal()
                        }),
                ) else {
                    return no_draft;
                };
                if !bindings.insert(declaration.binding)
                    || !ordinals.insert(declaration.source_ordinal)
                    || declaration.written_type_range != range
                    || node.links.context != Some(declaration.local_context)
                    || head.links.context != Some(declaration.local_context)
                {
                    return no_draft;
                }
                let Some(declared_node) = typed.nodes().node(declaration.site.node()) else {
                    return no_draft;
                };
                if declared_node.typing != TypingState::Successful {
                    return no_draft;
                }
                let Some(entry) = declared_node
                    .links
                    .type_entry
                    .and_then(|id| typed.types().get(id))
                else {
                    return no_draft;
                };
                let TypeEntryActual::Known(id) = entry.actual else {
                    return no_draft;
                };
                if entry.owner != declaration.site
                    || entry.status != TypeStatus::Known
                    || root_type.is_some_and(|previous| previous != id)
                {
                    return no_draft;
                }
                let Some(ty) = inference.normalized_types().get(id) else {
                    return no_draft;
                };
                if ty.status != NormalizedTypeStatus::Known
                    || ty.head != TypeHeadRef::BuiltinSet
                    || !ty.args.is_empty()
                    || !ty.attributes.positive().is_empty()
                    || !ty.attributes.negative().is_empty()
                {
                    return no_draft;
                }
                root_type = Some(id);
                first = Some(first.map_or(declaration.source_ordinal, |previous| {
                    previous.min(declaration.source_ordinal)
                }));
            }
            let Some(first) = first else {
                return no_draft;
            };
            expressions.push(ExpressionMetadata {
                expression_id: format!("source.theorem.type.{first}"),
                source_range: SourceRangeSummary {
                    start_byte: range.start as u64,
                    end_byte: range.end as u64,
                },
                expression_kind: node.kind.as_str().to_owned(),
                rendered_surface: surface.to_owned(),
                inferred_type: Some("set".to_owned()),
                resolved_symbol: None,
                inserted_coercions: Vec::new(),
                active_thesis: None,
                overload_resolution: None,
            });
        }
        if bindings.len() != written_types.applications().len() {
            return no_draft;
        }
        let Some(node) = typed.nodes().node(formula) else {
            return no_draft;
        };
        expressions.push(ExpressionMetadata {
            expression_id: format!("source.theorem.quantified.{}", owner.symbol.fqn().as_str()),
            source_range: SourceRangeSummary {
                start_byte: formula_range.start as u64,
                end_byte: formula_range.end as u64,
            },
            expression_kind: node.kind.as_str().to_owned(),
            rendered_surface: elaborated.to_owned(),
            inferred_type: None,
            resolved_symbol: None,
            inserted_coercions: Vec::new(),
            active_thesis: None,
            overload_resolution: None,
        });
        let vcs = storage
            .typed_handle::<mizar_vc::vc_ir::VcSet>(
                vc_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremVC"),
            )
            .map_err(|error| error.to_string())?;
        let vcs = storage.get(&vcs).map_err(|error| error.to_string())?;
        let [vc] = vcs.vcs() else {
            return no_draft;
        };
        if vc.id != vc_id
            || vc.kind != mizar_vc::vc_ir::VcKind::TheoremProofStep
            || slice.vc() != vc_id
            || slice.kind() != &vc.kind
            || slice.fingerprint().hash()
                != projection.reuse_metadata().dependency_slice_fingerprint()
        {
            return no_draft;
        }
        let range = vc
            .anchor
            .source_range
            .ok_or_else(|| fail("requires original VC source range"))?;
        if range.source_id != version.source_id
            || range.start >= range.end
            || range.start < owner.source_range.start
            || range.end > owner.source_range.end
        {
            return Err(fail("original VC source range mismatch"));
        }
        let statement = source
            .source_text
            .get(range.start..range.end)
            .ok_or_else(|| fail("original VC source range invalid"))?;
        let reuse = projection.reuse_metadata();
        let identity = ProofObligationIdentity::new(
            projection.obligation_id(),
            reuse.obligation_anchor().clone(),
            reuse.obligation_fingerprint(),
            reuse.vc_fingerprint(),
            reuse.local_context_fingerprint(),
            reuse.dependency_slice_fingerprint(),
        )
        .map_err(|error| error.to_string())?;
        let kernel = storage
            .typed_handle::<(
                mizar_kernel::checker::KernelCheckResult,
                mizar_proof::policy::PolicyDecision,
            )>(
                kernel_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremCandidateCheck"),
            )
            .map_err(|error| error.to_string())?;
        let kernel = storage.get(&kernel).map_err(|error| error.to_string())?;
        let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
        let kernel_input = KernelPolicyInput::from_kernel_result(&kernel.0, origin);
        let trusted = TrustedUsedAxiomsRef::from_kernel_result(&kernel.0, origin)
            .map_err(|error| error.to_string())?;
        let selected = selection.reuse_metadata();
        let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
            selection
                .selected_candidate_id()
                .cloned()
                .ok_or_else(|| fail("requires selected candidate"))?,
            &kernel_input,
        )
        .ok_or_else(|| fail("requires trusted candidate"))?
        .with_provenance_hash(
            selected
                .selected_candidate_provenance_hash()
                .ok_or_else(|| fail("requires candidate provenance"))?,
        );
        PublishedArtifactPath::new(version.normalized_path.as_str())
            .map_err(|_| fail("source path mismatch"))?;
        let path = PublishedArtifactPath::new(staging.0).map_err(|error| error.to_string())?;
        if path.as_str() != reference.witness_path {
            return Err(fail("witness path mismatch"));
        }
        let raw_refs = [
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                "mizar-vc/source-theorem-canonical-vc/v1",
                reuse.vc_fingerprint(),
            ),
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                "mizar-vc/source-theorem-canonical-context/v1",
                reuse.local_context_fingerprint(),
            ),
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                mizar_vc::dependency_slice::DEPENDENCY_SLICE_SCHEMA_VERSION,
                reuse.dependency_slice_fingerprint(),
            ),
            (
                ArtifactHashClass::Artifact,
                HashClass::Artifact,
                if self.build_plan.lockfile.toolchain_version.is_some() {
                    "mizar-build/parsed-lock-content/v2"
                } else {
                    "mizar-build/parsed-lock-content/v1"
                },
                self.snapshot.lockfile_hash,
            ),
            (
                ArtifactHashClass::Interface,
                HashClass::Interface,
                "mizar-build/package-config-content/v1",
                self.snapshot.verifier_config_hash,
            ),
        ]
        .map(|(class, hash_class, producer, digest)| {
            let value = CanonicalJson::Object(std::collections::BTreeMap::from([
                ("producer".to_owned(), CanonicalJson::string(producer)),
                (
                    "digest".to_owned(),
                    CanonicalJson::string(
                        digest
                            .as_bytes()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>(),
                    ),
                ),
            ]));
            let family = "mizar-driver/source-artifact-owner-fingerprint";
            let version = mizar_artifact::store::SchemaVersion::new(1, 0);
            ArtifactHashRef::new(
                class,
                family,
                version,
                CanonicalHashDomain::new(hash_class, family, version).hash(&value, &[]),
            )
        });
        let [
            vc_fingerprint,
            local_context_fingerprint,
            dependency_slice_fingerprint,
            lockfile_hash,
            verifier_config_hash,
        ] = raw_refs;
        use mizar_core::core_ir::{
            CoreFormulaKind, CoreIr, CoreItemKind, CoreItemStatus, CoreNodeRef, CoreProofNodeKind,
            CoreProofStatus, CoreSourceAnchor, CoreTermKind, ObligationSeedKind,
            ObligationSeedStatus,
        };
        let core = storage
            .typed_handle::<CoreIr>(
                core_parent.as_output_ref(),
                &OutputKind::new("SourceTheoremCore"),
            )
            .map_err(|error| error.to_string())?;
        let core = storage.get(&core).map_err(|error| error.to_string())?;
        let (Some((item_id, item)), Some((_, proof)), Some((seed_id, seed))) = (
            unique(core.items().iter()),
            unique(core.proofs().iter()),
            unique(core.obligation_seeds().iter()),
        ) else {
            return no_draft;
        };
        if core.source_id() != version.source_id
            || core.module_id() != arena.module()
            || item.symbol != owner.symbol
            || item.kind != CoreItemKind::Theorem
            || item.status != CoreItemStatus::Valid
            || item.visibility.as_str() != visibility
            || proof.item != item_id
            || proof.status != CoreProofStatus::PendingAutomaticProof
            || seed.owner != item_id
            || seed.kind != ObligationSeedKind::TheoremProof
            || seed.status != ObligationSeedStatus::Active
            || core.proof_nodes().len() != 2
            || item.source.anchor != CoreSourceAnchor::SourceRange(owner.source_range)
            || proof.source.anchor != item.source.anchor
            || core.source_map().item_sources.get(&item_id) != Some(&item.source)
        {
            return no_draft;
        }
        let Some(root) = core.proof_nodes().get(proof.root) else {
            return no_draft;
        };
        let CoreProofNodeKind::IntroduceBinder {
            binder: local,
            child,
        } = &root.kind
        else {
            return no_draft;
        };
        let Some(terminal) = core.proof_nodes().get(*child) else {
            return no_draft;
        };
        let CoreProofNodeKind::TerminalGoal {
            obligation,
            citations,
        } = &terminal.kind
        else {
            return no_draft;
        };
        let (Some(goal_id), Some(guard), Some(name)) =
            (seed.goal, local.ty_guard, local.source_name.as_deref())
        else {
            return no_draft;
        };
        if *obligation != seed_id
            || !citations.is_empty()
            || local.role.as_str() != "proof-let"
            || name.is_empty()
            || seed.context != [guard]
            || !seed.core_refs.contains(&CoreNodeRef::Item(item_id))
            || !seed.core_refs.contains(&CoreNodeRef::Formula(goal_id))
            || !seed.core_refs.contains(&CoreNodeRef::ProofNode(*child))
            || core.source_map().proof_sources.get(&proof.root) != Some(&root.source)
            || core.source_map().proof_sources.get(child) != Some(&terminal.source)
            || core.source_map().obligation_sources.get(&seed_id) != Some(&seed.source)
            || terminal.source.anchor != seed.source.anchor
            || root.source.anchor != local.source.anchor
        {
            return no_draft;
        }
        let Some(goal) = core.formulas().get(goal_id) else {
            return no_draft;
        };
        let CoreFormulaKind::Equals { left, right } = goal.kind else {
            return no_draft;
        };
        let (Some(left_term), Some(right_term)) = (core.terms().get(left), core.terms().get(right))
        else {
            return no_draft;
        };
        if left == right
            || left_term.kind != CoreTermKind::Var(local.var)
            || right_term.kind != CoreTermKind::Var(local.var)
            || core.source_map().formula_sources.get(&goal_id) != Some(&goal.source)
            || core.source_map().term_sources.get(&left) != Some(&left_term.source)
            || core.source_map().term_sources.get(&right) != Some(&right_term.source)
        {
            return no_draft;
        }
        let (
            CoreSourceAnchor::SourceRange(terminal_range),
            CoreSourceAnchor::SourceRange(goal_range),
            CoreSourceAnchor::SourceRange(left_range),
            CoreSourceAnchor::SourceRange(right_range),
            CoreSourceAnchor::SourceRange(local_range),
        ) = (
            &terminal.source.anchor,
            &goal.source.anchor,
            &left_term.source.anchor,
            &right_term.source.anchor,
            &local.source.anchor,
        )
        else {
            return no_draft;
        };
        if [
            terminal_range,
            goal_range,
            left_range,
            right_range,
            local_range,
        ]
        .iter()
        .any(|range| {
            range.source_id != version.source_id
                || range.start >= range.end
                || range.start < owner.source_range.start
                || range.end > owner.source_range.end
                || source.source_text.get(range.start..range.end).is_none()
        }) || local_range.end > terminal_range.start
            || goal_range.start < terminal_range.start
            || goal_range.end > terminal_range.end
            || left_range.start < goal_range.start
            || right_range.end > goal_range.end
            || left_range.end > right_range.start
            || goal_range == left_range
            || goal_range == right_range
            || source.source_text.get(local_range.start..local_range.end) != Some(name)
        {
            return no_draft;
        }
        let Some((_, equality)) = unique(
            atomic
                .formulas()
                .iter()
                .filter(|(_, row)| row.source_range() == *goal_range),
        ) else {
            return no_draft;
        };
        let Some(equality_row) = metadata.expr_metadata().get_by_site(equality.site()) else {
            return no_draft;
        };
        let mut thesis_ids =
            std::collections::BTreeSet::from([equality_row.expr.as_str().to_owned()]);
        for range in [left_range, right_range] {
            let Some((_, term)) = unique(
                primary
                    .terms()
                    .iter()
                    .filter(|(_, row)| row.source_range() == *range),
            ) else {
                return no_draft;
            };
            let Some(row) = metadata.expr_metadata().get_by_site(term.site()) else {
                return no_draft;
            };
            if !thesis_ids.insert(row.expr.as_str().to_owned()) {
                return no_draft;
            }
        }
        if thesis_ids.len() != 3 {
            return no_draft;
        }
        let thesis = format!("{name} = {name}");
        for expression in &mut expressions {
            if thesis_ids.remove(&expression.expression_id) {
                expression.active_thesis = Some(thesis.clone());
            }
        }
        if !thesis_ids.is_empty() {
            return no_draft;
        }
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    parent.as_output_ref(),
                    OutputAuthorization::Published,
                )
                .map_err(|error| error.to_string())?;
        }
        std::fs::create_dir(staging.1).map_err(|error| error.to_string())?;
        let cleanup = NativeArtifactStagingCleanup(
            matches!(version.origin, SourceOrigin::OpenBuffer { .. })
                .then(|| staging.1.to_path_buf()),
        );
        let domain = artifact_hash_domain(
            reference.witness_artifact_hash.schema_family.clone(),
            reference.witness_artifact_hash.schema_version,
        );
        let written = write_published_artifact(staging.1, &path, &file, &domain, &[])
            .map_err(|error| error.to_string())?;
        if written.artifact_hash != reference.witness_artifact_hash.digest {
            return Err(fail("witness file hash mismatch"));
        }
        let read = read_published_artifact(
            staging.1,
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &[],
                expected_artifact_hash: Some(reference.witness_artifact_hash.digest),
            },
        )
        .map_err(|error| error.to_string())?;
        mizar_proof::witness_store::read_source_theorem_witness_file(&read, &reference)?;
        let staged = witness_draft.stage_source_theorem_file(&read, &reference)?;
        let evidence = ProofEvidenceSet::new(
            kernel.0.target_vc_fingerprint().sort_bytes(),
            selected.encoded_problem_hash(),
            policies.1.clone(),
        )
        .with_candidates([
            candidate.with_selected_proof_witness_hash(staged.staged_payload_hash().digest)
        ]);
        let merged = merge_artifact_proof_selections(
            [VcProofSelection::new(vc_id, select_winner(&evidence))],
            [],
        )
        .map_err(|error| error.to_string())?;
        let [accepted] =
            <[_; 1]>::try_from(merged).map_err(|_| fail("requires one accepted selection"))?;
        let accepted = project_status(
            ProofStatusProjectionInput::new(accepted, policies.1.clone(), identity)
                .with_trusted_used_axioms(trusted),
        )
        .map_err(|error| error.to_string())?;
        if accepted.selected_class() != ProofWinnerClass::KernelVerified
            || accepted.projected_status() != ProjectedProofStatus::Accepted
            || accepted.artifact_publication()
                != ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
            || accepted.accepted_witness_obligation_id() != Some(reference.obligation_id.as_str())
            || !accepted.diagnostic_refs().is_empty()
            || accepted.explanation_ref().is_some()
            || staged.artifact_reference_candidate() != Some(&reference)
        {
            return Err(fail("witness-bound accepted candidate mismatch"));
        }
        let obligations = vec![ObligationMetadata {
            obligation_id: accepted.obligation_id().to_owned(),
            obligation_anchor: Some(
                accepted
                    .reuse_metadata()
                    .obligation_anchor()
                    .as_str()
                    .to_owned(),
            ),
            owner_origin_id: export.map(|export| export.origin_id.clone()),
            source_range: SourceRangeSummary {
                start_byte: range.start as u64,
                end_byte: range.end as u64,
            },
            obligation_kind: "theorem-proof-step".to_owned(),
            statement_summary: statement.to_owned(),
            obligation_fingerprint: reference.obligation_fingerprint.clone(),
            vc_fingerprint,
            local_context_fingerprint,
            dependency_slice_fingerprint,
            verifier_policy_fingerprint: reference
                .kernel_acceptance
                .verifier_policy_fingerprint
                .clone(),
            status: match accepted.artifact_publication() {
                ArtifactStatusPublication::Publishable(
                    CurrentArtifactObligationStatus::Accepted,
                ) => ObligationStatus::Accepted,
                _ => return Err(fail("accepted status unavailable")),
            },
            accepted_witness_obligation_id: accepted
                .accepted_witness_obligation_id()
                .map(str::to_owned),
            deterministic_discharge_hash: None,
            diagnostic_ref: None,
        }];
        let draft = ArtifactProjectionService::new(Arc::clone(publisher))
            .project_module(ProjectVerifiedArtifactInput {
                snapshot: input.snapshot,
                required_outputs: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
                module: summary.module,
                source_file: version.normalized_path.as_str().to_owned(),
                source_hash: version.source_hash,
                verified_at: None,
                exports,
                statements,
                expressions,
                obligations,
                proof_witnesses: vec![reference],
                diagnostics: Vec::new(),
                provenance: BuildProvenance {
                    captured_source: Some((
                        version.source_id,
                        version.origin.clone(),
                        version.source_hash,
                    )),
                    toolchain: self.snapshot.toolchain.identity().to_owned(),
                    language_edition: version.edition.as_str().to_owned(),
                    lockfile_hash,
                    verifier_config_hash,
                    dependency_artifact_hashes: Vec::new(),
                    cache_key: None,
                },
            })
            .map_err(|error| error.to_string())?;
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    parent.as_output_ref(),
                    OutputAuthorization::Published,
                )
                .map_err(|error| error.to_string())?;
        }
        cleanup.release();
        Ok(Some((draft, staged)))
    }

    /// Writes and reads back a genuine unpublished module draft, retaining its staged witness.
    pub fn write_native_theorem_artifact_draft(
        self,
        input: &PhaseInput,
        publisher: &Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (&str, &str, &std::path::Path),
    ) -> Result<
        Option<(
            mizar_ir::projection::VerifiedArtifactDraft,
            mizar_artifact::store::PublishedArtifactWrite,
            mizar_proof::witness_store::ProofWitnessStagedRef,
        )>,
        String,
    > {
        use mizar_artifact::{
            store::{
                PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
                read_published_artifact, write_published_artifact,
            },
            verified_artifact::{
                VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
                artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
            },
        };
        let Some((draft, staged)) = self.project_native_theorem_artifact_draft(
            input,
            publisher,
            vc_id,
            encoding,
            policies,
            (staging.0, staging.2),
        )?
        else {
            return Ok(None);
        };
        let cleanup = NativeArtifactStagingCleanup(
            matches!(
                draft.artifact.provenance.captured_source.as_ref(),
                Some((_, SourceOrigin::OpenBuffer { .. }, _))
            )
            .then(|| staging.2.to_path_buf()),
        );
        let path = PublishedArtifactPath::new(staging.1).map_err(|error| error.to_string())?;
        if !path.as_str().ends_with(".mizir.json") {
            return Err("native theorem artifact main requires a .mizir.json main path".to_owned());
        }
        match std::fs::symlink_metadata(staging.2.join(path.as_str())) {
            Ok(_) => {
                return Err("native theorem artifact main requires a fresh main path".to_owned());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
        let value = verified_artifact_json(&draft.artifact).map_err(|error| error.to_string())?;
        let domain = artifact_hash_domain(
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            draft.artifact.schema_version,
        );
        let excluded = artifact_hash_excluded_paths();
        let storage = publisher.storage();
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    parent.as_output_ref(),
                    OutputAuthorization::Published,
                )
                .map_err(|error| error.to_string())?;
        }
        let written = write_published_artifact(staging.2, &path, &value, &domain, &excluded)
            .map_err(|error| error.to_string())?;
        let read = read_published_artifact(
            staging.2,
            &path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                hash_excluded_paths: &excluded,
                expected_artifact_hash: Some(written.artifact_hash),
            },
        )
        .map_err(|error| error.to_string())?;
        let decoded = read_verified_artifact(
            &read.value,
            VerifiedArtifactReadOptions {
                artifact_path: Some(path.as_str()),
                expected_module: Some(&draft.artifact.module),
                expected_interface_hash: Some(draft.artifact.interface_hash),
                expected_implementation_hash: Some(draft.artifact.implementation_hash),
            },
        )
        .map_err(|error| error.to_string())?;
        if decoded != draft.artifact {
            return Err("native theorem artifact main readback mismatch".to_owned());
        }
        for parent in input.parent_outputs() {
            storage
                .validate_handle(parent.as_output_ref())
                .map_err(|error| error.to_string())?;
            publisher
                .validate_output(
                    input.snapshot,
                    parent.as_output_ref(),
                    OutputAuthorization::Published,
                )
                .map_err(|error| error.to_string())?;
        }
        cleanup.release();
        Ok(Some((draft, written, staged)))
    }

    /// Commits a genuine source module manifest and publishes its retained staged witness.
    /// Also seals the committed main as ordinary Artifact data with its six source parents;
    /// retained data does not confer fresh filesystem or proof-publication authority.
    #[expect(
        clippy::type_complexity,
        reason = "keep the existing committed values and typed artifact data together"
    )]
    pub fn commit_native_theorem_artifact_manifest(
        self,
        input: &PhaseInput,
        publisher: &Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: (
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        ),
        policies: (
            &mizar_vc::discharge::DischargePolicy,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (&str, &str, &std::path::Path),
    ) -> Result<
        Option<(
            mizar_ir::projection::VerifiedArtifactDraft,
            mizar_artifact::store::PublishedArtifactWrite,
            mizar_artifact::manifest::ManifestCommit,
            mizar_proof::witness_store::ProofWitnessPublishedRef,
            mizar_ir::storage::PhaseOutputRef<mizar_artifact::verified_artifact::VerifiedArtifact>,
        )>,
        String,
    > {
        use mizar_artifact::{
            manifest::{
                ArtifactManifest, ManifestCommitOptions, ManifestFileReadOptions,
                ManifestProofWitnessEntry, ManifestProvenance, ManifestTransaction,
                ModuleArtifactEntry, PackageIdentity, current_schema_version, read_manifest_file,
            },
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{
                PublishedArtifactPath, artifact_hash_domain, canonical_json_bytes,
                parse_canonical_json_bytes,
            },
            verified_artifact::{
                VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
                artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
            },
        };
        let Some((draft, main_write, staged)) = self.write_native_theorem_artifact_draft(
            input, publisher, vc_id, encoding, policies, staging,
        )?
        else {
            return Ok(None);
        };
        let cleanup = NativeArtifactStagingCleanup(
            matches!(
                draft.artifact.provenance.captured_source.as_ref(),
                Some((_, SourceOrigin::OpenBuffer { .. }, _))
            )
            .then(|| staging.2.to_path_buf()),
        );
        let fail = |detail| format!("native theorem artifact manifest {detail}");
        let package = self
            .build_plan
            .packages
            .iter()
            .find(|package| package.package_id.as_str() == draft.artifact.module.package_id)
            .ok_or_else(|| fail("requires a bound package"))?;
        let PackagePlanSource::Workspace {
            root: package_root, ..
        } = &package.source
        else {
            return Err(fail("requires a workspace package"));
        };
        let package_root = std::fs::canonicalize(
            PathBuf::from(self.snapshot.workspace_root.as_str()).join(package_root),
        )
        .map_err(|error| error.to_string())?;
        let root = std::fs::canonicalize(staging.2).map_err(|error| error.to_string())?;
        let relative = root
            .strip_prefix(&package_root)
            .map_err(|_| fail("root must be below package root"))?;
        if relative.as_os_str().is_empty() {
            return Err(fail("root must be below package root"));
        }
        let label = relative
            .components()
            .map(|component| {
                let std::path::Component::Normal(value) = component else {
                    return Err(fail("root has an invalid path component"));
                };
                value.to_str().ok_or_else(|| fail("root is not UTF-8"))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("/");
        let label = PublishedArtifactPath::new(label).map_err(|error| error.to_string())?;
        let guard = input
            .snapshot
            .to_published_schema_string()
            .map_err(|error| error.to_string())?;
        let storage = publisher.storage();
        let freshness = |candidate: Option<&str>| {
            candidate == Some(guard.as_str())
                && publisher.validate_current_snapshot(input.snapshot).is_ok()
                && input.parent_outputs().iter().all(|parent| {
                    let parent = parent.as_output_ref();
                    storage.validate_handle(parent).is_ok()
                        && publisher
                            .validate_output(input.snapshot, parent, OutputAuthorization::Published)
                            .is_ok()
                })
        };
        if !freshness(Some(guard.as_str())) {
            return Err(fail("currentness validation failed"));
        }
        let artifact = &draft.artifact;
        let open_buffer = matches!(
            artifact.provenance.captured_source.as_ref(),
            Some((_, SourceOrigin::OpenBuffer { .. }, _))
        );
        let entry = ModuleArtifactEntry {
            module: artifact.module.clone(),
            source_file: artifact.source_file.clone(),
            source_hash: artifact.source_hash,
            artifact_file: main_write.path.as_str().to_owned(),
            artifact_hash: ArtifactHashRef::new(
                ArtifactHashClass::Artifact,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                main_write.artifact_hash,
            ),
            interface_hash: ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                artifact.interface_hash,
            ),
            implementation_hash: ArtifactHashRef::new(
                ArtifactHashClass::Implementation,
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
                artifact.implementation_hash,
            ),
            module_summary_file: None,
            module_summary_hash: None,
            module_summary_interface_hash: None,
            registration_summary_file: None,
            registration_summary_hash: None,
            registration_interface_hash: None,
            proof_witnesses: artifact
                .proof_witnesses
                .iter()
                .map(|witness| ManifestProofWitnessEntry {
                    obligation_id: witness.obligation_id.clone(),
                    obligation_fingerprint: witness.obligation_fingerprint.clone(),
                    witness_path: witness.witness_path.clone(),
                    witness_artifact_hash: witness.witness_artifact_hash.clone(),
                })
                .collect(),
            diagnostics_hash: None,
        };
        let seed = ArtifactManifest {
            schema_version: current_schema_version(),
            package: PackageIdentity {
                package_id: artifact.module.package_id.clone(),
                package_version: artifact.module.package_version.clone(),
                lockfile_identity: artifact.module.lockfile_identity.clone(),
            },
            artifact_root: label.as_str().to_owned(),
            lockfile_hash: artifact.provenance.lockfile_hash.clone(),
            toolchain: artifact.provenance.toolchain.clone(),
            language_edition: artifact.provenance.language_edition.clone(),
            verifier_config_hash: artifact.provenance.verifier_config_hash.clone(),
            modules: Vec::new(),
            development_artifacts: Vec::new(),
            provenance: ManifestProvenance {
                generated_by: concat!("mizar-driver/", env!("CARGO_PKG_VERSION")).to_owned(),
                manifest_policy: "source-theorem-unpublished".to_owned(),
                transaction_format: "mizar-artifact/manifest-transaction-v1".to_owned(),
            },
        };
        let mut transaction = ManifestTransaction::begin(&root, seed, Some(guard.clone()))
            .map_err(|error| error.to_string())?;
        transaction
            .stage_module(entry)
            .map_err(|error| error.to_string())?;
        let commit = transaction
            .commit(ManifestCommitOptions {
                freshness_check: Some(&freshness),
            })
            .map_err(|error| error.to_string())?;
        let read = read_manifest_file(
            &root,
            ManifestFileReadOptions {
                validate_references: true,
                validate_witness_files: true,
                validate_development_artifacts: true,
                expected_manifest_hash: Some(commit.write().artifact_hash),
            },
        )
        .map_err(|error| error.to_string())?;
        if read.manifest != *commit.manifest() || read.artifact_hash != commit.write().artifact_hash
        {
            return Err(fail("readback mismatch"));
        }
        if !freshness(Some(guard.as_str())) {
            return Err(fail("currentness validation failed"));
        }
        let proof =
            mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                &commit,
                &root,
                &draft.artifact.module,
                input.snapshot.fingerprint(),
            )?;
        let published = mizar_proof::witness_store::publish_ref(&staged, &proof)
            .map_err(|error| error.to_string())?;
        if !freshness(Some(guard.as_str())) {
            return Err(fail("currentness validation failed"));
        }
        let value = verified_artifact_json(&draft.artifact).map_err(|error| error.to_string())?;
        let schema_version = draft.artifact.schema_version;
        let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, schema_version);
        let excluded = artifact_hash_excluded_paths();
        let canonical_payload = domain.hash_input(&value, &excluded).into_bytes();
        let storage_payload = canonical_json_bytes(&value);
        let identity = draft.artifact.module.clone();
        let interface_hash = draft.artifact.interface_hash;
        let implementation_hash = draft.artifact.implementation_hash;
        let artifact_hash = main_write.artifact_hash;
        let artifact_path = main_write.path.as_str().to_owned();
        let phase = IrPipelinePhase::new("ArtifactCommit");
        let output_kind = OutputKind::new("SourceTheoremArtifactData");
        let work_unit = input.parent_outputs()[0]
            .as_output_ref()
            .work_unit()
            .clone();
        if !freshness(Some(guard.as_str())) {
            return Err(fail("currentness validation failed"));
        }
        let artifact_output = publish_native_output(
            publisher,
            input.snapshot,
            (phase, work_unit, output_kind),
            |slot, phase, work_unit, output_kind| PublishOutputInput {
                slot,
                snapshot: input.snapshot,
                phase,
                work_unit,
                output_kind,
                schema_version: SchemaVersion::new(1),
                payload: draft.artifact.clone(),
                canonical_payload: Some(canonical_payload),
                storage_payload: Some(storage_payload),
                decode: BlobDecoder::new(move |bytes| {
                    let value = parse_canonical_json_bytes(bytes, &artifact_path)
                        .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                    let artifact = read_verified_artifact(
                        &value,
                        VerifiedArtifactReadOptions {
                            artifact_path: Some(&artifact_path),
                            expected_module: Some(&identity),
                            expected_interface_hash: Some(interface_hash),
                            expected_implementation_hash: Some(implementation_hash),
                        },
                    )
                    .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                    if artifact.schema_version != schema_version
                        || domain.hash(&value, &excluded) != artifact_hash
                    {
                        return Err(BlobDecodeError::new(
                            "native theorem artifact stored main hash or schema mismatch",
                        ));
                    }
                    Ok(artifact)
                }),
                parents: input
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref().clone())
                    .collect(),
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: input.identities().input_hash(),
                }],
                side_tables: IrSideTables::default(),
                origin: if open_buffer {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if open_buffer {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            },
        )
        .map_err(|error| error.to_string())?;
        if !freshness(Some(guard.as_str())) {
            return Err(fail("currentness validation failed"));
        }
        cleanup.release();
        Ok(Some((
            draft,
            main_write,
            commit,
            published,
            artifact_output,
        )))
    }
}
// Resolve the parent only: following the final entry could delete an external symlink target.
pub(crate) fn remove_native_artifact_staging(root: &std::path::Path, boundary: &std::path::Path) {
    let Some(name) = root.file_name() else { return };
    let parent = root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let (Ok(parent), Ok(boundary)) = (parent.canonicalize(), boundary.canonicalize()) else {
        return;
    };
    if !parent.starts_with(&boundary) {
        return;
    }
    let root = parent.join(name);
    let Ok(metadata) = std::fs::symlink_metadata(&root) else {
        return;
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        let _ = std::fs::remove_dir_all(root);
    } else {
        let _ = std::fs::remove_file(root);
    }
}

struct NativeArtifactStagingCleanup(Option<PathBuf>);

impl NativeArtifactStagingCleanup {
    fn release(mut self) {
        self.0 = None;
    }
}

impl Drop for NativeArtifactStagingCleanup {
    fn drop(&mut self) {
        if let Some(root) = &self.0 {
            let parent = root.parent().filter(|path| !path.as_os_str().is_empty());
            remove_native_artifact_staging(root, parent.unwrap_or(std::path::Path::new(".")));
        }
    }
}

#[cfg(test)]
mod native_artifact_staging_cleanup_tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::remove_native_artifact_staging;

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    fn root() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "mizar-driver-staging-cleanup-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn removes_owned_directory_and_preserves_sibling() {
        let boundary = root();
        let owned = boundary.join("staging");
        let sibling = boundary.join("unrelated");
        fs::create_dir(&owned).unwrap();
        fs::write(owned.join("partial"), b"owned").unwrap();
        fs::write(&sibling, b"keep").unwrap();

        drop(super::NativeArtifactStagingCleanup(Some(owned.clone())));

        assert!(!owned.exists());
        assert_eq!(fs::read(sibling).unwrap(), b"keep");
        fs::remove_dir_all(boundary).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn removes_final_symlink_without_following_external_target() {
        use std::os::unix::fs::symlink;

        let boundary = root();
        let external = root();
        let target = external.join("keep");
        let link = boundary.join("staging");
        fs::write(&target, b"external").unwrap();
        symlink(&target, &link).unwrap();

        remove_native_artifact_staging(&link, &boundary);

        assert!(fs::symlink_metadata(&link).is_err());
        assert_eq!(fs::read(target).unwrap(), b"external");
        fs::remove_dir_all(boundary).unwrap();
        fs::remove_dir_all(external).unwrap();
    }

    #[test]
    fn refuses_cleanup_when_parent_is_outside_boundary() {
        let boundary = root();
        let outside = root();
        let owned = outside.join("staging");
        fs::create_dir(&owned).unwrap();
        fs::write(owned.join("partial"), b"keep").unwrap();

        remove_native_artifact_staging(&owned, &boundary);

        assert_eq!(fs::read(owned.join("partial")).unwrap(), b"keep");
        fs::remove_dir_all(boundary).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }
}
/// One complete manifest-backed build; retained data grants no currentness authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotPublication {
    pub(crate) captured: crate::request::CapturedSnapshot,
    pub(crate) diagnostics: mizar_diagnostics::aggregator::BuildDiagnosticIndex,
    pub(crate) manifests: Vec<mizar_artifact::manifest::PublishedManifestRead>,
    pub(crate) artifacts: Vec<Arc<mizar_artifact::verified_artifact::VerifiedArtifact>>,
    pub(crate) development_artifacts: Vec<mizar_artifact::store::PublishedArtifactRead>,
}

impl SnapshotPublication {
    pub fn captured(&self) -> &crate::request::CapturedSnapshot {
        &self.captured
    }

    pub fn diagnostics(&self) -> &mizar_diagnostics::aggregator::BuildDiagnosticIndex {
        &self.diagnostics
    }

    pub fn manifests(&self) -> &[mizar_artifact::manifest::PublishedManifestRead] {
        &self.manifests
    }

    /// Original typed outputs, including expression, export and explanation metadata.
    pub fn artifacts(&self) -> &[Arc<mizar_artifact::verified_artifact::VerifiedArtifact>] {
        &self.artifacts
    }

    /// Resolved manifest development entries, including optional missing-facts payloads.
    pub fn development_artifacts(&self) -> &[mizar_artifact::store::PublishedArtifactRead] {
        &self.development_artifacts
    }
}

pub(crate) struct SourceTheoremArtifactCommitService {
    pub(crate) check: SourceTheoremCandidateCheckService,
    pub(crate) main_path: String,
    pub(crate) output_parent: String,
}

impl PhaseService for SourceTheoremArtifactCommitService {
    fn phase(&self) -> PhaseDescriptor {
        let check = self.check.phase().schema_version;
        PhaseDescriptor::new(
            "SourceTheoremArtifactCommit",
            PhaseOwner::MizarArtifact,
            vec![PipelinePhase::ArtifactCommit],
            format!(
                "mizar-artifact/source-theorem-artifact-commit/v1;candidate_check={}:{};main_path={}:{}",
                check.len(),
                check,
                self.main_path.len(),
                self.main_path,
            ),
            "SourceTheoremArtifactData",
        )
        .expect("source theorem artifact commit descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason: "source theorem artifact commit has no complete dependency footprint"
                .to_owned(),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext<'_>) -> PhaseResult {
        if context.common.snapshot != input.snapshot || context.common.work_unit != input.work_unit
        {
            return blocking();
        }
        if let Some(token) = context.cancellation {
            return if token.snapshot == input.snapshot {
                PhaseResult {
                    status: PhaseStatus::Cancelled,
                    ..blocking()
                }
            } else {
                blocking()
            };
        }
        if self.check.candidate.configuration_tags().is_none() {
            return blocking();
        }
        let Some(prefix) = &self.check.witness_path_prefix else {
            return blocking();
        };
        let Some(publisher) = context.output_publisher else {
            return blocking();
        };
        let Some(inputs) = context.source_load else {
            return blocking();
        };
        if inputs.snapshot.id != input.snapshot
            || publisher.validate_current_snapshot(input.snapshot).is_err()
            || input.parent_outputs().len() != 6
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
        let source_key = source_input_hash(version);
        if input.identities().input_hash() != source_key {
            return blocking();
        }
        let Some(vc_parent) = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("VcGenerate")
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        })) else {
            return blocking();
        };
        let Some(kernel) = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::new("KernelCheck")
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new("SourceTheoremCandidateCheck")
        })) else {
            return blocking();
        };
        let storage = publisher.storage();
        let Ok(handle) = storage.typed_handle::<mizar_vc::vc_ir::VcSet>(
            vc_parent.as_output_ref(),
            &OutputKind::new("SourceTheoremVC"),
        ) else {
            return blocking();
        };
        let Ok(vcs) = storage.get(&handle) else {
            return blocking();
        };
        let [vc] = vcs.vcs() else {
            return blocking();
        };
        if storage
            .typed_handle::<(
                mizar_kernel::checker::KernelCheckResult,
                mizar_proof::policy::PolicyDecision,
            )>(
                kernel.as_output_ref(),
                &OutputKind::new("SourceTheoremCandidateCheck"),
            )
            .is_err()
            || [vc_parent, kernel].iter().any(|parent| {
                publisher
                    .validate_output(
                        input.snapshot,
                        parent.as_output_ref(),
                        OutputAuthorization::Published,
                    )
                    .is_err()
            })
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
        let PackagePlanSource::Workspace { root, .. } = &package.source else {
            return blocking();
        };
        let Ok(package_root) = std::fs::canonicalize(
            PathBuf::from(inputs.snapshot.workspace_root.as_str()).join(root),
        ) else {
            return blocking();
        };
        let Ok(parent) = std::fs::canonicalize(package_root.join(&self.output_parent)) else {
            return blocking();
        };
        if !parent.is_dir()
            || !parent
                .strip_prefix(&package_root)
                .is_ok_and(|relative| !relative.as_os_str().is_empty())
        {
            return blocking();
        }
        let snapshot_hex = input
            .snapshot
            .fingerprint()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let source_hex = source_key
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let root = parent.join(match &version.origin {
            SourceOrigin::OpenBuffer {
                version: document_version,
            } => format!(
                "{snapshot_hex}-{source_hex}-open-{}-{document_version}",
                version
                    .source_id
                    .to_nonportable_provenance_string()
                    .replace(':', "-")
            ),
            _ => format!("{snapshot_hex}-{source_hex}"),
        });
        let kernel_hex = kernel
            .as_output_ref()
            .content_hash()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let witness_path = format!("{prefix}/{kernel_hex}.json");
        let result = inputs.commit_native_theorem_artifact_manifest(
            &input,
            &publisher,
            vc.id,
            (
                self.check.candidate.logic_profile.clone(),
                self.check.candidate.concrete_format,
            ),
            (&self.check.candidate.policy, &self.check.verifier_policy),
            (&witness_path, &self.main_path, &root),
        );
        match result {
            Ok(Some((_, _, commit, _, handle))) => PhaseResult {
                output_refs: vec![handle.erase()],
                artifact_commit: Some(commit),
                ..PhaseResult::complete()
            },
            Ok(None) | Err(_) => blocking(),
        }
    }
}

#[cfg(test)]
mod snapshot_test_support {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        },
    };

    use mizar_atp::problem::{
        ConcreteFormat, EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport,
        QuantifierPolicy, SoftTypeStrategy,
    };
    use mizar_build::{
        module_index::{
            ModuleId, StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage,
            build_module_index,
        },
        planner::{
            DependencySelection, PlanRequest, WorkspacePackage, lockfile_content_hash,
            parse_lockfile, parse_package_manifest, produce_build_plan,
            workspace_config_content_hash,
        },
        task_graph::{
            EvidenceCandidateId, ModuleDependencyOverlay, VcOrderKey, VcTaskDescriptor,
            VcTaskDescriptorId,
        },
    };
    use mizar_ir::{
        identity::{OutputKind, PipelinePhase, SnapshotHandleRegistry, WorkUnit},
        publisher::{AllowedWorkUnit, PhaseOutputPublisher},
        storage::IrStorageService,
    };
    use mizar_proof::policy::VerifierPolicy;
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo, WorkspaceRoot,
        normalize_path,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };

    use crate::{
        driver::{BuildSubmission, DriverSubmitInput},
        events::SnapshotPublication,
        registry::{PhaseRegistry, PhaseRegistryBuilder, SourceLoadInputs},
        request::{
            BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
            BuildRequestGeneration, BuildRequestOrigin, BuildSession, BuildTargets,
            DependencyInputSet, SourceInputSet, VerifierConfigInput,
        },
    };

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    pub(super) struct Fixture {
        pub(super) root: PathBuf,
        pub(super) ids: InMemorySessionIdAllocator,
        pub(super) snapshots: SnapshotRegistry<InMemorySessionIdAllocator>,
    }

    impl Fixture {
        pub(super) fn new() -> Self {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "mizar-driver-coherent-snapshot-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(root.join("alpha/src")).unwrap();
            fs::create_dir(root.join("alpha/artifacts")).unwrap();
            for (module, theorem) in [("main", "Main"), ("leaf", "Leaf")] {
                fs::write(
                    root.join(format!("alpha/src/{module}.miz")),
                    format!(
                        "theorem {theorem}: for X being set holds X = X proof let X be set; thus X = X; end;"
                    ),
                )
                .unwrap();
            }
            Self {
                root,
                ids: InMemorySessionIdAllocator::new(),
                snapshots: SnapshotRegistry::new(),
            }
        }

        pub(super) fn prepare(
            &self,
            generation: u64,
        ) -> (
            BuildSession,
            DriverSubmitInput<StaticSourceLayout>,
            Arc<PhaseOutputPublisher>,
        ) {
            self.prepare_with_open_buffers(generation, &[])
        }

        pub(super) fn prepare_with_open_buffers(
            &self,
            generation: u64,
            open_buffers: &[(&str, i64)],
        ) -> (
            BuildSession,
            DriverSubmitInput<StaticSourceLayout>,
            Arc<PhaseOutputPublisher>,
        ) {
            let workspace_root = WorkspaceRoot::new(self.root.to_string_lossy().into_owned());
            let mut input = DriverSubmitInput::new(
                PlanRequest {
                    workspace_root: workspace_root.clone(),
                    dependency_selection: DependencySelection::Normal,
                    toolchain: ToolchainInfo::new("mizar-evo-test"),
                },
                vec![WorkspacePackage {
                    member_path: "alpha".into(),
                    manifest: parse_package_manifest(
                        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
                    )
                    .unwrap(),
                }],
                parse_lockfile(
                    "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n",
                )
                .unwrap(),
                StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                    package_id: PackageId::new("alpha"),
                    files: ["main", "leaf"]
                        .map(|module| {
                            WorkspaceSourceFile::new(
                                format!("src/{module}.miz"),
                                format!("{module}.miz"),
                            )
                        })
                        .to_vec(),
                }]),
            );
            input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
            input.vc_descriptors = ["main", "leaf"]
                .map(|module| {
                    VcTaskDescriptor::new(
                        VcTaskDescriptorId::new(format!("snapshot-{module}")),
                        ModuleId::new(PackageId::new("alpha"), ModulePath::new(module)),
                        VcOrderKey::new("00000000000000000000"),
                        vec![],
                        vec![EvidenceCandidateId::new("native-reflexivity")],
                    )
                })
                .to_vec();
            let plan = produce_build_plan(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
            )
            .unwrap();
            let module_index = build_module_index(&plan, &input.source_layout, &[]).unwrap();
            let draft = BuildRequestDraft {
                lane: BuildLaneId::new(1),
                origin: BuildRequestOrigin::Batch(BatchRequest {
                    invocation: BatchInvocation::default(),
                }),
                generation: BuildRequestGeneration::new(generation),
                workspace_root,
                profile: BuildProfile::new("check"),
                targets: BuildTargets::default(),
                source_inputs: SourceInputSet::default(),
                dependency_inputs: DependencyInputSet::new(
                    vec![],
                    lockfile_content_hash(&plan.lockfile),
                    plan.toolchain.clone(),
                ),
                verifier_config: VerifierConfigInput::new(workspace_config_content_hash(
                    &plan.verifier_config,
                    &plan.build_config,
                )),
            };
            let package_root = self.root.join("alpha");
            let descriptors = ["main", "leaf"].map(|module| {
                let path = format!("src/{module}.miz");
                let loader = DiskSourceLoader::new(&package_root);
                let source_input = if let Some((_, version)) =
                    open_buffers.iter().find(|(name, _)| *name == module)
                {
                    let absolute_path = package_root.join(&path);
                    let text: Arc<str> = fs::read_to_string(&absolute_path).unwrap().into();
                    module_index
                        .native_open_buffer_input(
                            &plan.workspace_root,
                            &PackageId::new("alpha"),
                            &loader,
                            SourceOriginInput::OpenBuffer {
                                uri: format!("file://{}", absolute_path.display()),
                                expected_version: *version,
                                actual_version: *version,
                                text,
                            },
                        )
                        .unwrap()
                } else {
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new(module),
                        normalized_path: normalize_path(&package_root, Path::new(&path)).unwrap(),
                        edition: Edition::new("2025"),
                        origin: SourceOriginInput::Disk { path: path.into() },
                    }
                };
                (loader, source_input)
            });
            let session = draft
                .capture_sources(&self.ids, &self.snapshots, descriptors.to_vec())
                .unwrap();
            let publisher = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::new()),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(session.captured().snapshot().id);
            for module in ["main", "leaf"] {
                for (phase, kind) in [
                    ("SourceLoad", "SourceUnit"),
                    ("Frontend", "FrontendOutput"),
                    ("ModuleResolve", "SourceTheoremModuleCandidates"),
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ("OverloadResolution", "SourceTheoremMetadata"),
                    ("Elaboration", "SourceTheoremCore"),
                    ("AlgorithmPreparation", "SourceTheoremPreparation"),
                    ("VcGenerate", "SourceTheoremVC"),
                    ("VcDischarge", "SourceTheoremDischarge"),
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                    ("KernelCheck", "SourceTheoremCandidateCheck"),
                    ("KernelCheck", "SourceTheoremCandidateWitnessFile"),
                    ("ArtifactCommit", "SourceTheoremArtifactData"),
                ] {
                    publisher.allow_work_unit(AllowedWorkUnit::new(
                        PipelinePhase::new(phase),
                        OutputKind::new(kind),
                        WorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
                    ));
                }
            }
            (session, input, publisher)
        }

        pub(super) fn resolve(
            &self,
            submission: &BuildSubmission,
            publisher: &PhaseOutputPublisher,
        ) -> Option<SnapshotPublication> {
            SourceLoadInputs {
                snapshot: submission.session.captured().snapshot(),
                captured: Some(submission.session.captured()),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &self.ids,
            }
            .complete_snapshot_publication(
                submission.task_graph.as_ref().unwrap(),
                submission.scheduler_run.as_ref().unwrap(),
                publisher,
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    pub(super) fn registry(commit: bool) -> PhaseRegistry {
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new("snapshot-reflexivity"),
                max_steps: 64,
            },
        };
        let profile = LogicProfile::try_new(
            "snapshot-native",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::ScopedGuardPredicates,
            NativePropertySupport::Supported,
            [ConcreteFormat::Tptp].into_iter().collect(),
        )
        .unwrap();
        let verifier = VerifierPolicy::default();
        let mut builder = PhaseRegistryBuilder::new();
        builder
            .register_source_load()
            .register_frontend(vec![])
            .register_source_theorem_module_resolver()
            .register_source_theorem_signature_collection()
            .register_source_theorem_type_checking()
            .register_source_theorem_registration_resolution()
            .register_source_theorem_overload_resolution()
            .register_source_theorem_elaboration()
            .register_source_theorem_algorithm_preparation()
            .register_source_theorem_vc_generation(
                GenerationSchemaVersion::new("snapshot-generation-v1"),
                VcSchemaVersion::new("snapshot-vc-v1"),
            )
            .register_source_theorem_vc_discharge(policy.clone())
            .register_source_theorem_atp_solve(
                policy.clone(),
                profile.clone(),
                ConcreteFormat::Tptp,
            )
            .unwrap()
            .register_source_theorem_candidate_check_with_witness_files(
                policy.clone(),
                profile.clone(),
                ConcreteFormat::Tptp,
                verifier.clone(),
                "proof-witnesses/snapshot".into(),
            )
            .unwrap();
        if commit {
            builder
                .register_source_theorem_artifact_commit(
                    policy,
                    profile,
                    ConcreteFormat::Tptp,
                    verifier,
                    (
                        "proof-witnesses/snapshot".into(),
                        "main.mizir.json".into(),
                        "artifacts".into(),
                    ),
                )
                .unwrap();
        }
        builder.build().unwrap()
    }
}

#[cfg(test)]
mod snapshot_payload_tests {
    use super::snapshot_test_support::*;
    use crate::driver::CompilerDriver;
    use std::{collections::BTreeSet, fs, sync::Arc};

    fn publish(
        fixture: &Fixture,
        generation: u64,
    ) -> (
        crate::driver::BuildSubmission,
        Arc<mizar_ir::publisher::PhaseOutputPublisher>,
    ) {
        let (session, input, publisher) = fixture.prepare(generation);
        let mut driver =
            CompilerDriver::new(registry(true)).with_output_publisher(publisher.clone());
        let submission = driver
            .submit_captured(
                session,
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .unwrap();
        (submission, publisher)
    }

    #[test]
    fn genuine_multimodule_build_resolves_one_original_complete_payload() {
        let fixture = Fixture::new();
        let (submission, publisher) = publish(&fixture, 1);
        let publication = fixture.resolve(&submission, &publisher).unwrap();

        assert_eq!(publication.manifests().len(), 2);
        assert_eq!(publication.artifacts().len(), 2);
        assert_eq!(
            publication.diagnostics().snapshot(),
            submission.session.captured().snapshot().id
        );
        let manifest = &publication.manifests()[0].manifest;
        assert_eq!(manifest.modules.len(), 1);
        assert!(
            publication
                .manifests()
                .iter()
                .all(|read| read.manifest.modules.len() == 1)
        );
        assert_eq!(
            publication.development_artifacts().len(),
            publication
                .manifests()
                .iter()
                .map(|read| read.manifest.development_artifacts.len())
                .sum::<usize>()
        );
        assert!(publication.diagnostics().is_empty());
        assert!(
            publication
                .manifests()
                .iter()
                .all(|read| read.manifest.development_artifacts.is_empty())
        );
        assert!(publication.development_artifacts().is_empty());
        assert!(
            publication
                .artifacts()
                .iter()
                .all(|artifact| artifact.diagnostics.is_empty())
        );
        let versions = &publication.captured().snapshot().source_versions;
        let artifacts = publication
            .artifacts()
            .iter()
            .map(|artifact| {
                let version = versions
                    .iter()
                    .find(|version| {
                        version.package_id.as_str() == artifact.module.package_id.as_str()
                            && version.module_path.as_str() == artifact.module.module_path.as_str()
                    })
                    .unwrap();
                assert_eq!(
                    artifact.source_file.as_str(),
                    version.normalized_path.as_str()
                );
                assert_eq!(artifact.source_hash, version.source_hash);
                assert!(!artifact.expressions.is_empty());
                assert!(!artifact.exports.is_empty());
                (
                    artifact.module.package_id.clone(),
                    artifact.module.module_path.clone(),
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(artifacts.len(), 2);
        let original_outputs = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .filter(|result| result.artifact_commit.is_some())
            .flat_map(|result| &result.output_refs)
            .map(|output| {
                let handle = publisher
                    .storage()
                    .typed_handle::<mizar_artifact::verified_artifact::VerifiedArtifact>(
                        output,
                        &mizar_ir::identity::OutputKind::new("SourceTheoremArtifactData"),
                    )
                    .unwrap();
                publisher.storage().get(&handle).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(original_outputs.len(), 2);
        assert!(publication.artifacts().iter().all(|artifact| {
            original_outputs
                .iter()
                .any(|original| Arc::ptr_eq(artifact, original))
        }));
        let manifest_modules = publication
            .manifests()
            .iter()
            .flat_map(|read| &read.manifest.modules)
            .map(|entry| {
                let version = versions
                    .iter()
                    .find(|version| {
                        version.package_id.as_str() == entry.module.package_id.as_str()
                            && version.module_path.as_str() == entry.module.module_path.as_str()
                    })
                    .unwrap();
                assert_eq!(entry.source_file.as_str(), version.normalized_path.as_str());
                assert_eq!(entry.source_hash, version.source_hash);
                (
                    entry.module.package_id.clone(),
                    entry.module.module_path.clone(),
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(manifest_modules, artifacts);
    }

    #[test]
    fn missing_manifest_or_member_refuses_complete_payload() {
        let fixture = Fixture::new();
        let (submission, publisher) = publish(&fixture, 2);
        let complete = fixture.resolve(&submission, &publisher).unwrap();
        let artifact_root = fixture
            .root
            .join("alpha")
            .join(&complete.manifests()[0].manifest.artifact_root);
        let manifest_path = artifact_root.join(complete.manifests()[0].path.as_str());
        let manifest_bytes = fs::read(&manifest_path).unwrap();

        fs::remove_file(&manifest_path).unwrap();
        assert!(fixture.resolve(&submission, &publisher).is_none());
        fs::write(&manifest_path, &manifest_bytes).unwrap();
        let restored = fixture.resolve(&submission, &publisher).unwrap();
        let member_path =
            artifact_root.join(&restored.manifests()[0].manifest.modules[0].artifact_file);
        let member_bytes = fs::read(&member_path).unwrap();

        fs::remove_file(&member_path).unwrap();
        assert!(fixture.resolve(&submission, &publisher).is_none());
        fs::write(&member_path, member_bytes).unwrap();
        assert!(fixture.resolve(&submission, &publisher).is_some());
    }

    #[test]
    fn changed_manifest_hash_refuses_complete_payload() {
        let fixture = Fixture::new();
        let (submission, publisher) = publish(&fixture, 3);
        let complete = fixture.resolve(&submission, &publisher).unwrap();
        let manifest_path = fixture
            .root
            .join("alpha")
            .join(&complete.manifests()[0].manifest.artifact_root)
            .join(complete.manifests()[0].path.as_str());
        let original = fs::read_to_string(&manifest_path).unwrap();
        let mut unsupported_schema = original.clone();
        let key = original.find("\"schema_version\"").unwrap();
        let colon = original[key..].find(':').unwrap() + key;
        let start = original[colon..].find('"').unwrap() + colon + 1;
        let end = original[start..].find('"').unwrap() + start;
        unsupported_schema.replace_range(start..end, "2.0");
        fs::write(&manifest_path, unsupported_schema).unwrap();
        assert!(fixture.resolve(&submission, &publisher).is_none());
        fs::write(&manifest_path, &original).unwrap();
        assert!(fixture.resolve(&submission, &publisher).is_some());

        let artifact_root = &complete.manifests()[0].manifest.artifact_root;
        let changed_hash = original.replacen(
            &format!("\"{artifact_root}\""),
            &format!("\"x{artifact_root}\""),
            1,
        );
        assert_ne!(changed_hash, original);
        fs::write(&manifest_path, changed_hash).unwrap();
        assert!(fixture.resolve(&submission, &publisher).is_none());
        fs::write(&manifest_path, original).unwrap();
        assert!(fixture.resolve(&submission, &publisher).is_some());
    }
}

#[cfg(test)]
mod snapshot_publication_tests {
    use mizar_session::SourceOrigin;
    use std::{fs, sync::Arc};

    use crate::{
        driver::CompilerDriver,
        events::BuildEventKind,
        registry::PhaseRegistryBuilder,
        request::{
            BuildLaneId, BuildRequestGeneration, BuildSessionOutcome, BuildSessionState,
            PublicationDecision,
        },
    };

    use super::snapshot_test_support::{Fixture, registry};

    fn snapshot_events(
        driver: &CompilerDriver,
        session: mizar_session::BuildSessionId,
    ) -> Vec<Arc<crate::events::SnapshotPublication>> {
        driver
            .events(session)
            .events()
            .iter()
            .filter_map(|event| match &event.kind {
                BuildEventKind::SnapshotPublished { snapshot } => Some(Arc::clone(snapshot)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn current_complete_build_publishes_one_coherent_snapshot_event() {
        let fixture = Fixture::new();
        let (session, input, owner) = fixture.prepare(1);
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(
                session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .expect("genuine complete build submits");

        assert_eq!(
            submission.publication_decision,
            PublicationDecision::Current
        );
        let publications = snapshot_events(&driver, session.id());
        assert_eq!(publications.len(), 1);
        let published = &publications[0];
        assert_eq!(
            published.captured().snapshot().id,
            session.captured().snapshot().id
        );
        assert!(!published.manifests().is_empty());
        assert!(!published.artifacts().is_empty());
        let stream = driver.events(session.id());
        let published_event_index = stream
            .events()
            .iter()
            .position(|event| matches!(&event.kind, BuildEventKind::SnapshotPublished { .. }))
            .expect("the coherent publication event is present");
        let published_event = &stream.events()[published_event_index];
        assert_eq!(published_event.identity.session, session.id());
        assert_eq!(published_event.identity.lane, BuildLaneId::new(1));
        assert_eq!(
            published_event.identity.generation,
            BuildRequestGeneration::new(1)
        );
        assert_eq!(
            published_event.identity.snapshot,
            Some(session.captured().snapshot().id)
        );
        assert_eq!(
            published_event.identity.publication,
            submission.publication_decision
        );
        let finished_event_index = stream
            .events()
            .iter()
            .position(|event| matches!(&event.kind, BuildEventKind::SessionFinished { .. }))
            .expect("the terminal event is present");
        assert!(published_event_index < finished_event_index);
        let resolved = fixture
            .resolve(&submission, &owner)
            .expect("the same genuine completed outputs resolve coherently");
        assert_eq!(published.as_ref(), &resolved);
    }

    #[test]
    fn genuine_open_buffer_build_publishes_complete_truthful_provenance() {
        let fixture = Fixture::new();
        let unrelated = fixture.root.join("alpha/artifacts/unrelated");
        fs::write(&unrelated, b"keep").unwrap();
        let (session, input, owner) =
            fixture.prepare_with_open_buffers(1, &[("main", 7), ("leaf", 7)]);
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(
                session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .expect("genuine OpenBuffer build submits");

        assert_eq!(snapshot_events(&driver, session.id()).len(), 1);
        let publication = snapshot_events(&driver, session.id()).remove(0);
        assert!(
            fixture.resolve(&submission, &owner).is_none(),
            "ephemeral stages were discarded"
        );
        assert_eq!(fs::read(unrelated).unwrap(), b"keep");
        let versions = &session.captured().snapshot().source_versions;
        assert_eq!(publication.artifacts().len(), 2);
        for artifact in publication.artifacts() {
            let version = versions
                .iter()
                .find(|version| {
                    version.module_path.as_str() == artifact.module.module_path.as_str()
                })
                .unwrap();
            assert!(matches!(
                version.origin,
                SourceOrigin::OpenBuffer { version: 7 }
            ));
            assert_eq!(artifact.source_hash, version.source_hash);
            assert_eq!(
                artifact.provenance.captured_source,
                Some((
                    version.source_id,
                    version.origin.clone(),
                    version.source_hash
                ))
            );
        }
    }

    #[test]
    fn newer_open_buffer_capture_supersedes_older_currentness() {
        let fixture = Fixture::new();
        let (older, older_input, owner) =
            fixture.prepare_with_open_buffers(1, &[("main", 7), ("leaf", 7)]);
        let (newer, newer_input, newer_owner) =
            fixture.prepare_with_open_buffers(2, &[("main", 8), ("leaf", 8)]);
        assert_eq!(
            older.captured().snapshot().id,
            newer.captured().snapshot().id
        );
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner);
        driver
            .submit_captured(
                older.clone(),
                &fixture.ids,
                &fixture.snapshots,
                older_input,
                Default::default(),
            )
            .unwrap();
        let retained = snapshot_events(&driver, older.id()).remove(0);
        driver = driver.with_output_publisher(newer_owner);
        driver
            .submit_captured(
                newer.clone(),
                &fixture.ids,
                &fixture.snapshots,
                newer_input,
                Default::default(),
            )
            .unwrap();
        assert!(
            driver
                .validate_current_native_diagnostics(older.id(), &fixture.snapshots)
                .is_err()
        );
        assert!(
            driver
                .validate_current_native_diagnostics(newer.id(), &fixture.snapshots)
                .is_ok()
        );
        let current = snapshot_events(&driver, newer.id()).remove(0);
        for (old, new) in retained.artifacts().iter().zip(current.artifacts()) {
            assert!(matches!(
                old.provenance.captured_source,
                Some((_, SourceOrigin::OpenBuffer { version: 7 }, _))
            ));
            assert!(matches!(
                new.provenance.captured_source,
                Some((_, SourceOrigin::OpenBuffer { version: 8 }, _))
            ));
        }
    }

    #[test]
    fn transition_to_disk_publishes_distinct_disk_provenance() {
        let fixture = Fixture::new();
        let (buffer, buffer_input, owner) =
            fixture.prepare_with_open_buffers(1, &[("main", 7), ("leaf", 7)]);
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner.clone());
        driver
            .submit_captured(
                buffer.clone(),
                &fixture.ids,
                &fixture.snapshots,
                buffer_input,
                Default::default(),
            )
            .expect("genuine OpenBuffer build submits");
        let buffer_publication = snapshot_events(&driver, buffer.id()).remove(0);

        let (disk, disk_input, disk_owner) = fixture.prepare(2);
        assert_eq!(
            buffer.captured().snapshot().id,
            disk.captured().snapshot().id
        );
        driver = driver.with_output_publisher(disk_owner.clone());
        let disk_submission = driver
            .submit_captured(
                disk.clone(),
                &fixture.ids,
                &fixture.snapshots,
                disk_input,
                Default::default(),
            )
            .expect("genuine Disk build submits after the editor transition");
        let disk_publication = fixture.resolve(&disk_submission, &disk_owner).unwrap();

        assert!(
            driver
                .validate_current_native_diagnostics(buffer.id(), &fixture.snapshots)
                .is_err()
        );
        assert_eq!(snapshot_events(&driver, disk.id()).len(), 1);
        for (old, new) in buffer_publication
            .artifacts()
            .iter()
            .zip(disk_publication.artifacts())
        {
            let Some((old_id, SourceOrigin::OpenBuffer { .. }, old_hash)) =
                &old.provenance.captured_source
            else {
                panic!("the retained artifact preserves its OpenBuffer provenance")
            };
            let Some((new_id, SourceOrigin::Disk, new_hash)) = &new.provenance.captured_source
            else {
                panic!("the later artifact records its genuine Disk provenance")
            };
            assert_ne!(old_id, new_id);
            assert_eq!(*old_hash, old.source_hash);
            assert_eq!(*new_hash, new.source_hash);
        }
    }

    #[test]
    fn mixed_origins_publish_as_one_complete_honest_cohort() {
        let fixture = Fixture::new();
        let (session, input, owner) = fixture.prepare_with_open_buffers(1, &[("main", 7)]);
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner.clone());
        let _submission = driver
            .submit_captured(
                session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .expect("genuine mixed-origin build submits");
        assert_eq!(snapshot_events(&driver, session.id()).len(), 1);
        let publication = snapshot_events(&driver, session.id()).remove(0);
        assert_eq!(publication.artifacts().len(), 2);
        for artifact in publication.artifacts() {
            let Some((source_id, origin, source_hash)) = &artifact.provenance.captured_source
            else {
                panic!("every artifact carries its captured source provenance")
            };
            assert!(!source_id.to_nonportable_provenance_string().is_empty());
            assert_eq!(*source_hash, artifact.source_hash);
            match artifact.module.module_path.as_str() {
                "main" => assert!(matches!(origin, SourceOrigin::OpenBuffer { version: 7 })),
                "leaf" => assert!(matches!(origin, SourceOrigin::Disk)),
                other => panic!("unexpected artifact module {other}"),
            }
        }
    }

    #[test]
    fn changed_open_buffer_capture_cannot_publish_superseded_snapshot() {
        let fixture = Fixture::new();
        let (older, input, owner) =
            fixture.prepare_with_open_buffers(1, &[("main", 7), ("leaf", 7)]);
        fs::write(
            fixture.root.join("alpha/src/main.miz"),
            "theorem Changed: for X being set holds X = X proof let X be set; thus X = X; end;",
        )
        .unwrap();
        let (newer, _, _) = fixture.prepare_with_open_buffers(2, &[("main", 8), ("leaf", 8)]);
        assert_ne!(
            older.captured().snapshot().id,
            newer.captured().snapshot().id
        );
        owner
            .replace_current_snapshot(
                older.captured().snapshot().id,
                newer.captured().snapshot().id,
            )
            .unwrap();
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner);
        driver
            .submit_captured(
                older.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .unwrap();
        assert!(snapshot_events(&driver, older.id()).is_empty());
    }

    #[test]
    fn incomplete_open_buffer_pipeline_never_publishes_partial_cohort() {
        for missing_commit in [true, false] {
            let fixture = Fixture::new();
            if !missing_commit {
                fs::write(
                    fixture.root.join("alpha/src/leaf.miz"),
                    "theorem Leaf: invalid Mizar.",
                )
                .unwrap();
            }
            let (session, input, owner) =
                fixture.prepare_with_open_buffers(1, &[("main", 7), ("leaf", 7)]);
            let mut driver =
                CompilerDriver::new(registry(!missing_commit)).with_output_publisher(owner);
            driver
                .submit_captured(
                    session.clone(),
                    &fixture.ids,
                    &fixture.snapshots,
                    input,
                    Default::default(),
                )
                .unwrap();
            assert!(snapshot_events(&driver, session.id()).is_empty());
        }
    }

    #[test]
    fn stale_captured_build_does_not_emit_a_current_snapshot_event() {
        let fixture = Fixture::new();
        let (stale_session, input, owner) = fixture.prepare(1);
        fs::write(
            fixture.root.join("alpha/src/main.miz"),
            "theorem NewMain: for X being set holds X = X proof let X be set; thus X = X; end;",
        )
        .unwrap();
        let (newer_session, _, _) = fixture.prepare(2);
        assert_ne!(
            stale_session.captured().snapshot().id,
            newer_session.captured().snapshot().id
        );
        owner
            .replace_current_snapshot(
                stale_session.captured().snapshot().id,
                newer_session.captured().snapshot().id,
            )
            .expect("the publisher accepts the captured snapshot replacement");
        assert!(
            owner
                .validate_current_snapshot(stale_session.captured().snapshot().id)
                .is_err()
        );
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner);
        let submission = driver
            .submit_captured(
                stale_session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .expect("a stale captured request still records its build result");

        assert!(snapshot_events(&driver, stale_session.id()).is_empty());
        assert_ne!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
        );
    }

    #[test]
    fn incomplete_frontend_and_artifact_inputs_do_not_emit_snapshot_events() {
        let fixture = Fixture::new();
        let (session, input, owner) = fixture.prepare(1);
        let mut driver = CompilerDriver::new(registry(false)).with_output_publisher(owner.clone());
        let (submission, index) = driver
            .submit_captured(
                session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                crate::driver::SubmitOptions {
                    diagnostic_index: Some(crate::driver::DiagnosticIndexMode::Native),
                    ..Default::default()
                },
            )
            .map(|mut submission| {
                let index = submission.diagnostic_index.take();
                (submission, index)
            })
            .expect("partial genuine pipeline submits");
        assert!(snapshot_events(&driver, session.id()).is_empty());
        assert!(
            index.is_some(),
            "diagnostic index is a genuine partial output"
        );
        assert!(fixture.resolve(&submission, &owner).is_none());

        let fixture = Fixture::new();
        fs::write(
            fixture.root.join("alpha/src/leaf.miz"),
            "theorem Leaf: this is definitely not valid Mizar.",
        )
        .unwrap();
        let (session, input, owner) = fixture.prepare(1);
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner);
        let submission = driver
            .submit_captured(
                session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .expect("partial artifact build with a malformed module submits");
        let stream = driver.events(session.id());
        assert!(snapshot_events(&driver, session.id()).is_empty());
        assert!(
            stream
                .events()
                .iter()
                .any(|event| { matches!(&event.kind, BuildEventKind::ArtifactBoundary { .. }) })
        );
        assert_ne!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
        );

        let fixture = Fixture::new();
        let (session, input, owner) = fixture.prepare(1);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        let registry = builder.build().expect("frontend-only registry is valid");
        let mut driver = CompilerDriver::new(registry).with_output_publisher(owner);
        driver
            .submit_captured(
                session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .expect("genuine frontend-only pipeline submits");
        assert!(snapshot_events(&driver, session.id()).is_empty());
    }

    #[test]
    fn retained_snapshot_payload_stays_unchanged_after_current_replacement() {
        let fixture = Fixture::new();
        let (session, input, owner) = fixture.prepare(1);
        let mut driver = CompilerDriver::new(registry(true)).with_output_publisher(owner.clone());
        driver
            .submit_captured(
                session.clone(),
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .expect("genuine complete build submits");
        let retained = snapshot_events(&driver, session.id())
            .into_iter()
            .next()
            .expect("completed current build retains its publication");
        let before = retained.as_ref().clone();
        fs::write(
            fixture.root.join("alpha/src/main.miz"),
            "theorem NewMain: for X being set holds X = X proof let X be set; thus X = X; end;",
        )
        .unwrap();
        let (replacement, _, _) = fixture.prepare(2);
        assert_ne!(
            session.captured().snapshot().id,
            replacement.captured().snapshot().id
        );
        owner
            .replace_current_snapshot(
                session.captured().snapshot().id,
                replacement.captured().snapshot().id,
            )
            .expect("the publisher accepts a subsequent snapshot replacement");

        assert!(
            owner
                .validate_current_snapshot(session.captured().snapshot().id)
                .is_err()
        );
        assert_eq!(retained.as_ref(), &before);
        let replay = snapshot_events(&driver, session.id());
        assert!(Arc::ptr_eq(&retained, &replay[0]));
        assert_eq!(
            retained.captured().snapshot().id,
            session.captured().snapshot().id
        );
    }
}
