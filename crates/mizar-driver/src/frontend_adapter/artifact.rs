use super::*;

impl<'a> crate::registry::SourceLoadInputs<'a> {
    /// Resolves a complete original cohort after Driver has filtered fresh owner handoffs.
    pub(crate) fn complete_snapshot_publication(
        self,
        build_request: mizar_session::BuildRequestId,
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
                VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
                artifact_hash_excluded_paths, builtin_verified_artifact_json,
                read_builtin_verified_artifact, verified_artifact_json,
            },
        };
        let captured = self.captured?;
        if captured.snapshot() != self.snapshot {
            return None;
        }
        captured.native_sources()?;
        self.validate_native_lock_content().ok()?;
        self.validate_native_config_content().ok()?;
        let lockfile_producer = if self.build_plan.lockfile.toolchain_version.is_some() {
            "mizar-build/parsed-lock-content/v2"
        } else {
            "mizar-build/parsed-lock-content/v1"
        };
        let lockfile_hash =
            mizar_artifact::registration_summary::ArtifactHashRef::source_owner_fingerprint(
                lockfile_producer,
                self.snapshot.lockfile_hash,
            )?;
        let verifier_config_hash =
            mizar_artifact::registration_summary::ArtifactHashRef::source_owner_fingerprint(
                "mizar-build/package-config-content/v1",
                self.snapshot.verifier_config_hash,
            )?;
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
        let mut artifact_roots = Vec::new();
        let mut artifacts = Vec::new();
        let mut development_artifacts = Vec::new();
        let mut source_metadata = Vec::new();
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
            if manifest.manifest != *commit.manifest()
                || manifest.manifest.lockfile_hash != lockfile_hash
                || manifest.manifest.verifier_config_hash != verifier_config_hash
                || manifest.manifest.toolchain != self.snapshot.toolchain.identity()
            {
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
            let builtin =
                artifact.schema_version == mizar_artifact::store::SchemaVersion::new(2, 0);
            let mut bound_frontend = None;
            if builtin {
                // This source view is admitted only by the complete opt-in cohort.
                if artifact.exports.len() < 2
                    || artifact.obligations.len() != artifact.exports.len()
                    || artifact.proof_witnesses.len() != artifact.exports.len()
                    || artifact.provenance.captured_source.is_some()
                    || graph.tasks().iter().any(|task| {
                        matches!(
                            task.kind,
                            mizar_build::task_graph::TaskKind::AtpSolve
                                | mizar_build::task_graph::TaskKind::BackendRun
                        )
                    })
                {
                    return None;
                }
                let discharge = unique(graph.tasks().iter().filter(|task| {
                    task.kind == mizar_build::task_graph::TaskKind::VcDischarge
                        && matches!(&task.unit, WorkUnit::Vc { module: parent, .. } if parent == &module.module)
                }))?;
                let [discharged] = run.phase_results.get(&discharge.id)?.as_slice() else {
                    return None;
                };
                let [discharge_output] = discharged.output_refs.as_slice() else {
                    return None;
                };
                if discharged.status != PhaseStatus::Complete
                    || discharge_output.output_kind() != &OutputKind::new("SourceTheoremDischarge")
                {
                    return None;
                }
                publisher
                    .validate_output(
                        self.snapshot.id,
                        discharge_output,
                        OutputAuthorization::Published,
                    )
                    .ok()?;
                publisher.storage().validate_handle(discharge_output).ok()?;
                let parent_ids = &output.lineage().parents;
                if parent_ids.len() != 5 {
                    return None;
                }
                let physical = |id| {
                    unique(
                        run.phase_results
                            .values()
                            .flatten()
                            .flat_map(|result| &result.output_refs)
                            .filter(|parent| parent.output() == id),
                    )
                };
                let mut original = Vec::new();
                for (phase, kind) in [
                    (
                        PipelinePhase::SignatureCollection,
                        "SourceTheoremModuleSummary",
                    ),
                    (PipelinePhase::Elaboration, "SourceTheoremCore"),
                    (
                        PipelinePhase::AlgorithmPreparation,
                        "SourceTheoremPreparation",
                    ),
                    (PipelinePhase::VcGenerate, "SourceTheoremVC"),
                    (
                        PipelinePhase::KernelCheck,
                        "SourceTheoremReflexivityKernelStatus",
                    ),
                ] {
                    original.push(unique(
                        parent_ids
                            .iter()
                            .filter_map(|id| physical(*id))
                            .filter(|parent| {
                                parent.phase() == &IrPipelinePhase::from(phase)
                                    && parent.output_kind() == &OutputKind::new(kind)
                            }),
                    )?);
                }
                let unit = IrWorkUnit::new(module_work_unit_label(
                    module.package_id.as_str(),
                    module.module_path.as_str(),
                ));
                let named = vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_input_hash(version),
                }];
                for parent in &original {
                    publisher.storage().validate_handle(parent).ok()?;
                    publisher
                        .validate_output(self.snapshot.id, parent, OutputAuthorization::Published)
                        .ok()?;
                    if parent.snapshot() != self.snapshot.id
                        || parent.work_unit() != &unit
                        || parent.schema_version() != SchemaVersion::new(1)
                        || parent.lineage().named_input_hashes != named
                    {
                        return None;
                    }
                }
                let (summary, core, preparation, vcs, kernel) = (
                    original[0],
                    original[1],
                    original[2],
                    original[3],
                    original[4],
                );
                if preparation.lineage().parents != [core.output()]
                    || vcs.lineage().parents.len() != 2
                    || ![core, preparation]
                        .iter()
                        .all(|parent| vcs.lineage().parents.contains(&parent.output()))
                    || kernel.lineage().parents.len() != 3
                    || ![core, preparation, vcs]
                        .iter()
                        .all(|parent| kernel.lineage().parents.contains(&parent.output()))
                {
                    return None;
                }
                let stored_core = publisher
                    .storage()
                    .get(
                        &publisher
                            .storage()
                            .typed_handle::<mizar_core::core_ir::CoreIr>(
                                core,
                                &OutputKind::new("SourceTheoremCore"),
                            )
                            .ok()?,
                    )
                    .ok()?;
                let stored_vcs = publisher
                    .storage()
                    .get(
                        &publisher
                            .storage()
                            .typed_handle::<mizar_vc::vc_ir::VcSet>(
                                vcs,
                                &OutputKind::new("SourceTheoremVC"),
                            )
                            .ok()?,
                    )
                    .ok()?;
                let checked = publisher
                    .storage()
                    .get(
                        &publisher
                            .storage()
                            .typed_handle::<(
                                mizar_core::core_ir::CoreIr,
                                mizar_vc::vc_ir::VcSet,
                                mizar_kernel::checker::KernelCheckResult,
                                mizar_proof::policy::PolicyDecision,
                                mizar_proof::status::ProofStatusProjection,
                                mizar_vc::dependency_slice::DependencySlice,
                            )>(
                                kernel,
                                &OutputKind::new("SourceTheoremReflexivityKernelStatus"),
                            )
                            .ok()?,
                    )
                    .ok()?;
                if checked.0 != *stored_core
                    || checked.1 != *stored_vcs
                    || stored_core.source_id() != version.source_id
                    || stored_core.items().len() != artifact.exports.len()
                    || stored_vcs.vcs().len() != artifact.obligations.len()
                {
                    return None;
                }
                // Follow actual producer ids, never a metadata-only source match.
                let mut ancestry = core.lineage().parents.clone();
                for (phase, kind) in [
                    (PipelinePhase::OverloadResolution, "SourceTheoremMetadata"),
                    (PipelinePhase::TypeChecking, "SourceTheoremTypedAst"),
                    (
                        PipelinePhase::SignatureCollection,
                        "SourceTheoremModuleSummary",
                    ),
                    (
                        PipelinePhase::ModuleResolve,
                        "SourceTheoremModuleCandidates",
                    ),
                    (PipelinePhase::Frontend, "FrontendOutput"),
                ] {
                    let ancestor = unique(ancestry.iter().filter_map(|id| physical(*id)).filter(
                        |parent| {
                            parent.phase() == &IrPipelinePhase::from(phase)
                                && parent.output_kind() == &OutputKind::new(kind)
                        },
                    ))?;
                    publisher.storage().validate_handle(ancestor).ok()?;
                    publisher
                        .validate_output(self.snapshot.id, ancestor, OutputAuthorization::Published)
                        .ok()?;
                    if ancestor.snapshot() != self.snapshot.id
                        || ancestor.work_unit() != &unit
                        || (phase != PipelinePhase::Frontend
                            && ancestor.lineage().named_input_hashes != named)
                        || (phase == PipelinePhase::SignatureCollection
                            && ancestor.output() != summary.output())
                    {
                        return None;
                    }
                    if phase == PipelinePhase::Frontend {
                        bound_frontend = Some(ancestor.output());
                    }
                    ancestry = ancestor.lineage().parents.clone();
                }
            }
            let value = if builtin {
                builtin_verified_artifact_json(&artifact)
            } else {
                verified_artifact_json(&artifact)
            }
            .ok()?;
            if builtin {
                let main_path = PublishedArtifactPath::new(&entry.artifact_file).ok()?;
                let domain =
                    artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
                let read = read_published_artifact(
                    &root,
                    &main_path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        hash_excluded_paths: &artifact_hash_excluded_paths(),
                        expected_artifact_hash: Some(entry.artifact_hash.digest),
                    },
                )
                .ok()?;
                let original = read_builtin_verified_artifact(
                    &read.value,
                    VerifiedArtifactReadOptions {
                        artifact_path: Some(main_path.as_str()),
                        expected_module: Some(&entry.module),
                        expected_interface_hash: Some(entry.interface_hash.digest),
                        expected_implementation_hash: Some(entry.implementation_hash.digest),
                    },
                )
                .ok()?;
                if original != *artifact {
                    return None;
                }
            }
            if entry.module != artifact.module
                || artifact.provenance.lockfile_hash != lockfile_hash
                || artifact.provenance.verifier_config_hash != verifier_config_hash
                || artifact.provenance.toolchain != self.snapshot.toolchain.identity()
                || artifact.module.package_id != version.package_id.as_str()
                || artifact.module.module_path != version.module_path.as_str()
                || (!builtin
                    && artifact.provenance.captured_source
                        != Some((
                            version.source_id,
                            version.origin.clone(),
                            version.source_hash,
                        )))
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
            let mapped_source = (|| {
                use mizar_artifact::module_summary::{
                    MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummaryReadOptions, read_module_summary,
                };
                use mizar_resolve::{
                    declarations::DeclarationShellCollector, env::NamespacePath,
                    imports::ImportPathResolver, module_index::ModuleIndexInput,
                    resolved_ast::ModuleId, symbols::SignatureProjectionExtractor,
                };
                let path = PublishedArtifactPath::new(entry.module_summary_file.as_ref()?).ok()?;
                let hash = entry.module_summary_hash.as_ref()?;
                let read = read_published_artifact(
                    &root,
                    &path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&artifact_hash_domain(
                            MODULE_SUMMARY_SCHEMA_FAMILY,
                            hash.schema_version,
                        )),
                        hash_excluded_paths: &[],
                        expected_artifact_hash: Some(hash.digest),
                    },
                )
                .ok()?;
                let summary = read_module_summary(
                    &read.value,
                    ModuleSummaryReadOptions {
                        artifact_path: Some(path.as_str()),
                        expected_module: Some(&entry.module),
                        expected_interface_hash: Some(
                            entry.module_summary_interface_hash.as_ref()?.digest,
                        ),
                    },
                )
                .ok()?;
                let unit = IrWorkUnit::new(module_work_unit_label(
                    module.package_id.as_str(),
                    module.module_path.as_str(),
                ));
                let frontend_ref = unique(
                    run.phase_results
                        .values()
                        .flatten()
                        .flat_map(|result| &result.output_refs)
                        .filter(|output| {
                            output.work_unit() == &unit
                                && output.phase() == &IrPipelinePhase::from(PipelinePhase::Frontend)
                                && output.output_kind() == &OutputKind::new("FrontendOutput")
                        }),
                )?;
                publisher
                    .validate_output(
                        self.snapshot.id,
                        frontend_ref,
                        OutputAuthorization::Published,
                    )
                    .ok()?;
                let frontend = publisher
                    .storage()
                    .get(
                        &publisher
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                &OutputKind::new("FrontendOutput"),
                            )
                            .ok()?,
                    )
                    .ok()?;
                let source = &frontend.source;
                if (builtin && bound_frontend != Some(frontend_ref.output()))
                    || source.normalized_path != version.normalized_path
                    || source.edition != version.edition
                    || source.source_id != version.source_id
                    || source.source_hash != version.source_hash
                    || summary.source_hash != version.source_hash
                    || source.package_id != version.package_id
                    || source.module_path != version.module_path
                    || source.origin != version.origin
                {
                    return None;
                }
                let module = ModuleId::new(source.package_id.clone(), source.module_path.clone());
                let ast = frontend.ast.as_ref()?;
                let shells = DeclarationShellCollector::new(ast, &module).collect();
                let collection = SignatureProjectionExtractor::new(
                    ast,
                    &shells,
                    NamespacePath::new(module.path().as_str()),
                )
                .collect(&module);
                let imports = ImportPathResolver::new(ModuleIndexInput::new(self.module_index))
                    .publish_frontend(&frontend, source, |target, expected| {
                        let unit = IrWorkUnit::new(module_work_unit_label(
                            target.package.as_str(),
                            target.path.as_str(),
                        ));
                        let output = unique(
                            run.phase_results
                                .values()
                                .flatten()
                                .flat_map(|result| &result.output_refs)
                                .filter(|output| {
                                    output.work_unit() == &unit
                                        && output.phase()
                                            == &IrPipelinePhase::from(
                                                PipelinePhase::SignatureCollection,
                                            )
                                        && output.output_kind()
                                            == &OutputKind::new("SourceTheoremModuleSummary")
                                }),
                        )?;
                        publisher
                            .validate_output(
                                self.snapshot.id,
                                output,
                                OutputAuthorization::Published,
                            )
                            .ok()?;
                        let handle = publisher
                            .storage()
                            .typed_handle::<mizar_artifact::module_summary::ModuleSummary>(
                                output,
                                &OutputKind::new("SourceTheoremModuleSummary"),
                            )
                            .ok()?;
                        let summary = publisher.storage().get(&handle).ok()?;
                        if expected.is_some_and(|hash| hash != summary.interface_hash) {
                            return None;
                        }
                        mizar_artifact::module_summary::module_summary_json(&summary).ok()
                    })?;
                let mut declarations = summary.clone();
                declarations.reexports.clear();
                declarations.dependency_interfaces.clear();
                declarations.refresh_interface_hash().ok()?;
                if imports.compose_summary(declarations)? != summary {
                    return None;
                }
                // Run the existing bounded compiler mapper on retained original
                // syntax. Missing/unsupported bindings remain explicit occurrences.
                let compiler =
                    mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
                        .ok()
                        .flatten();
                let labels = compiler.as_ref().and_then(|(arena, env)| {
                    mizar_resolve::labels::source_theorem_label_reference_candidates(
                        ast, arena, env,
                    )
                    .ok()
                    .flatten()
                });
                let bound_export = |symbol: &mizar_resolve::env::SymbolEntry| {
                    let mizar_session::SourceAnchor::Range(range) = symbol.origin().anchor() else {
                        return None;
                    };
                    if symbol.origin().is_recovered()
                        || symbol.origin().source_id() != source.source_id
                        || symbol.origin().module_id() != &module
                        || symbol.origin().import_edge().is_some()
                        || symbol.visibility() != mizar_resolve::env::Visibility::Public
                        || symbol.export_status() != mizar_resolve::env::ExportStatus::Exported
                    {
                        return None;
                    }
                    unique(artifact.exports.iter().filter(|export| {
                        export.origin_id == symbol.symbol().fqn().as_str()
                            && export.fully_qualified_name == symbol.symbol().fqn().as_str()
                            && export.source_range.start_byte == range.start as u64
                            && export.source_range.end_byte == range.end as u64
                            && export.export_kind == "theorem"
                            && export.visibility
                                == mizar_artifact::verified_artifact::ExportVisibility::Public
                    }))
                    .cloned()
                };
                let direct_by = {
                    let mut source_view;
                    let direct_by_artifact = if builtin {
                        source_view = (*artifact).clone();
                        source_view.provenance.captured_source = Some((
                            version.source_id,
                            version.origin.clone(),
                            version.source_hash,
                        ));
                        &source_view
                    } else {
                        &artifact
                    };
                    mizar_artifact::direct_by::project_frontend_direct_by(
                        &frontend,
                        direct_by_artifact,
                        manifest.artifact_hash,
                        |node| {
                            let range = ast.node_view(node)?.range();
                            let (_, env) = compiler.as_ref()?;
                            let symbol = unique(env.symbols().iter().filter(|symbol| {
                                symbol.origin().anchor()
                                    == &mizar_session::SourceAnchor::Range(range)
                                    && matches!(
                                        symbol.kind(),
                                        mizar_resolve::env::SymbolKind::Theorem
                                            | mizar_resolve::env::SymbolKind::Lemma
                                    )
                            }))?;
                            bound_export(symbol)
                        },
                        |_, node| {
                            use mizar_artifact::direct_by::{DirectByEndpoint, DirectByTarget};
                            use mizar_resolve::resolved_ast::{LabelKind, LabelResolution};
                            let mapped = (|| {
                                let (arena, env) = compiler.as_ref()?;
                                let mapped = arena.resolved_node_for(node)?;
                                let labels = labels.as_ref()?;
                                let reference = unique(
                                    labels
                                        .table()
                                        .iter()
                                        .map(|(_, entry)| entry)
                                        .filter(|entry| entry.site().node() == mapped),
                                )?;
                                let LabelResolution::Resolved(label) = reference.resolution()
                                else {
                                    return None;
                                };
                                match label.kind() {
                                    LabelKind::ProofStep => Some(DirectByTarget::NonTheorem),
                                    LabelKind::Theorem => {
                                        let symbol =
                                            unique(env.symbols().iter().filter(|symbol| {
                                                symbol.symbol().fqn().as_str()
                                                    == label.origin().as_str()
                                            }))?;
                                        Some(DirectByTarget::Theorem(Box::new(DirectByEndpoint {
                                            module: artifact.module.clone(),
                                            publication_hash: manifest.artifact_hash,
                                            export: bound_export(symbol)?,
                                        })))
                                    }
                                    _ => None,
                                }
                            })();
                            mapped.unwrap_or(DirectByTarget::Unmapped)
                        },
                    )
                };
                Some(PublishedSourceMetadata {
                    summary,
                    notation: collection.source_notation_view(&frontend),
                    imports: Some(imports),
                    direct_by,
                    identity: mizar_artifact::verified_artifact::project_frontend_identity(
                        &frontend,
                        &artifact,
                        manifest.artifact_hash,
                    ),
                })
            })();
            if builtin
                && mapped_source
                    .as_ref()
                    .and_then(|metadata| metadata.direct_by.as_ref())
                    .is_none_or(|projection| !projection.complete())
            {
                return None;
            }
            source_metadata.push(mapped_source);
            artifacts.push(artifact);
            manifests.push(manifest);
            artifact_roots.push(root);
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
            build_request,
            artifact_roots,
            lockfile_producer,
            captured: captured.clone(),
            diagnostics,
            manifests,
            artifacts,
            development_artifacts,
            source_metadata,
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
        self.project_native_theorem_artifact_draft_route(
            input,
            publisher,
            vc_id,
            Some(encoding),
            (Some(policies.0), policies.1),
            (staging.0, staging.1, false),
        )
        .and_then(|result| {
            result
                .map(|(draft, stages)| {
                    let [stage] = <[_; 1]>::try_from(stages)
                        .map_err(|_| "normal theorem projection requires one witness".to_owned())?;
                    Ok((draft, stage))
                })
                .transpose()
        })
    }

    fn project_native_theorem_artifact_draft_route(
        self,
        input: &PhaseInput,
        publisher: &Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: Option<(
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        )>,
        policies: (
            Option<&mizar_vc::discharge::DischargePolicy>,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (&str, &std::path::Path, bool),
    ) -> Result<
        Option<(
            mizar_ir::projection::VerifiedArtifactDraft,
            Vec<mizar_proof::witness_store::ProofWitnessStagedRef>,
        )>,
        String,
    > {
        use mizar_artifact::{
            module_summary::{MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummary, SourceRangeSummary},
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{
                CanonicalJson, PublishedArtifactPath, PublishedArtifactReadOptions,
                artifact_hash_domain, read_published_artifact, write_published_artifact,
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
        let builtin = staging.2;
        if input.parent_outputs().len() != if builtin { 5 } else { 6 } {
            return Err(fail(if builtin {
                "requires five parents"
            } else {
                "requires six parents"
            }));
        }
        let mut roles = vec![
            (
                PipelinePhase::SignatureCollection,
                "SourceTheoremModuleSummary",
            ),
            (PipelinePhase::Elaboration, "SourceTheoremCore"),
            (PipelinePhase::VcGenerate, "SourceTheoremVC"),
            (
                PipelinePhase::KernelCheck,
                if builtin {
                    "SourceTheoremReflexivityKernelStatus"
                } else {
                    "SourceTheoremCandidateCheck"
                },
            ),
        ];
        if builtin {
            roles.push((
                PipelinePhase::AlgorithmPreparation,
                "SourceTheoremPreparation",
            ));
        } else {
            roles.extend([
                (PipelinePhase::VcDischarge, "SourceTheoremDischarge"),
                (
                    PipelinePhase::KernelCheck,
                    "SourceTheoremCandidateWitnessFile",
                ),
            ]);
        }
        let parents = roles
            .into_iter()
            .map(|(phase, kind)| {
                unique(input.parent_outputs().iter().filter(|parent| {
                    parent.as_output_ref().phase() == &IrPipelinePhase::from(phase)
                        && parent.as_output_ref().output_kind() == &OutputKind::new(kind)
                }))
                .ok_or_else(|| fail("requires unique source roles"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (summary_parent, core_parent, vc_parent, kernel_parent) =
            (parents[0], parents[1], parents[2], parents[3]);
        let mut normal_payload = if builtin {
            None
        } else {
            let original = PhaseInput::new(
                input.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    input.snapshot,
                    input.identities().input_hash(),
                    input.identities().dependency_hashes().to_vec(),
                    vec![
                        core_parent.clone(),
                        vc_parent.clone(),
                        parents[4].clone(),
                        kernel_parent.clone(),
                    ],
                )
                .map_err(|error| error.to_string())?,
            );
            let Some(payload) = self.native_theorem_candidate_witness_payload(
                &original,
                publisher,
                vc_id,
                encoding.ok_or_else(|| fail("requires candidate encoding"))?,
                (
                    policies
                        .0
                        .ok_or_else(|| fail("requires candidate discharge policy"))?,
                    policies.1,
                ),
                staging.0,
            )?
            else {
                return no_draft;
            };
            Some(payload)
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
        let work_unit = IrWorkUnit::new(module_work_unit_label(
            module.package.as_str(),
            module.path.as_str(),
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
        if let Some((_, (_, file, _))) = &normal_payload {
            let file_ref = parents[5].as_output_ref();
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
                != *file
            {
                return Err(fail("complete witness file mismatch"));
            }
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
            || (!builtin && summary.exported_symbols.len() > 1)
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
        let sealed = if builtin {
            let core_ref = core_parent.as_output_ref();
            let preparation_ref = parents[4].as_output_ref();
            let vc_ref = vc_parent.as_output_ref();
            let kernel_ref = kernel_parent.as_output_ref();
            if preparation_ref.lineage().parents != vec![core_ref.output()]
                || vc_ref.lineage().parents.len() != 2
                || !vc_ref.lineage().parents.contains(&core_ref.output())
                || !vc_ref.lineage().parents.contains(&preparation_ref.output())
                || kernel_ref.lineage().parents.len() != 3
                || [core_ref, preparation_ref, vc_ref]
                    .iter()
                    .any(|parent| !kernel_ref.lineage().parents.contains(&parent.output()))
            {
                return Err(fail("original producer chain mismatch"));
            }
            let [metadata_id] = core_ref.lineage().parents.as_slice() else {
                return Err(fail("requires original Metadata ancestry"));
            };
            let registry = publisher.registry();
            let metadata = registry
                .output_lineage(*metadata_id)
                .ok_or_else(|| fail("requires registered Metadata ancestry"))?;
            if metadata.phase != IrPipelinePhase::from(PipelinePhase::OverloadResolution)
                || metadata.output_kind != OutputKind::new("SourceTheoremMetadata")
                || metadata.parents.len() != 2
            {
                return Err(fail("original Metadata ancestry mismatch"));
            }
            let ancestors = metadata
                .parents
                .iter()
                .map(|id| registry.output_lineage(*id))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| fail("requires original Type and Registration ancestry"))?;
            let typed = unique(ancestors.iter().filter(|ancestor| {
                ancestor.phase == IrPipelinePhase::from(PipelinePhase::TypeChecking)
                    && ancestor.output_kind == OutputKind::new("SourceTheoremTypedAst")
            }))
            .ok_or_else(|| fail("requires unique Type ancestry"))?;
            let registration = unique(ancestors.iter().filter(|ancestor| {
                ancestor.phase == IrPipelinePhase::from(PipelinePhase::RegistrationResolution)
                    && ancestor.output_kind == OutputKind::new("SourceTheoremRegistrationClosure")
            }))
            .ok_or_else(|| fail("requires unique Registration ancestry"))?;
            if [&metadata, typed, registration].iter().any(|ancestor| {
                ancestor.snapshot != input.snapshot
                    || ancestor.work_unit != work_unit
                    || ancestor.named_input_hashes != named
            }) || registration.parents != vec![typed.output]
                || typed.parents != vec![summary_parent.as_output_ref().output()]
            {
                return Err(fail("original source-check ancestry mismatch"));
            }
            let mut ancestry = summary_parent.as_output_ref().lineage().parents.clone();
            for (phase, kind, source_bound) in [
                (
                    PipelinePhase::ModuleResolve,
                    "SourceTheoremModuleCandidates",
                    true,
                ),
                (PipelinePhase::Frontend, "FrontendOutput", false),
            ] {
                let [id] = ancestry.as_slice() else {
                    return Err(fail("requires unique original source ancestry"));
                };
                let ancestor = registry
                    .output_lineage(*id)
                    .ok_or_else(|| fail("requires registered source ancestry"))?;
                if ancestor.snapshot != input.snapshot
                    || ancestor.work_unit != work_unit
                    || ancestor.phase != IrPipelinePhase::from(phase)
                    || ancestor.output_kind != OutputKind::new(kind)
                    || (source_bound && ancestor.named_input_hashes != named)
                {
                    return Err(fail("original source ancestry mismatch"));
                }
                ancestry = ancestor.parents;
            }
            let typed = storage
                .typed_handle::<(
                    mizar_core::core_ir::CoreIr,
                    mizar_vc::vc_ir::VcSet,
                    mizar_kernel::checker::KernelCheckResult,
                    mizar_proof::policy::PolicyDecision,
                    mizar_proof::status::ProofStatusProjection,
                    mizar_vc::dependency_slice::DependencySlice,
                )>(
                    kernel_ref,
                    &OutputKind::new("SourceTheoremReflexivityKernelStatus"),
                )
                .map_err(|error| error.to_string())?;
            let sealed = storage.get(&typed).map_err(|error| error.to_string())?;
            let original = PhaseInput::new(
                input.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    input.snapshot,
                    input.identities().input_hash(),
                    input.identities().dependency_hashes().to_vec(),
                    vec![core_parent.clone(), parents[4].clone()],
                )
                .map_err(|error| error.to_string())?,
            );
            let Some(vc) = sealed.1.vcs().first() else {
                return no_draft;
            };
            if vc.anchor.generation_schema_version.as_str() != "source-theorem-local-v1"
                || sealed.1.schema_version().as_str() != "vc-v1"
                || self
                    .native_theorem_vcs_with_core_preparation_parents(
                        &original,
                        publisher,
                        &vc.anchor.generation_schema_version,
                        sealed.1.schema_version(),
                    )?
                    .as_ref()
                    != Some(&sealed.1)
            {
                return Err(fail("complete original graphs or schemas mismatch"));
            }
            let actual = mizar_proof::status::check_source_theorem_reflexivity_with_status(
                arena,
                &check,
                env,
                input.snapshot,
                vc_id,
                policies.1,
            )?;
            if *sealed != actual {
                return Err(fail("complete sealed source status mismatch"));
            }
            Some(sealed)
        } else {
            None
        };
        let mut checked_statuses = Vec::new();
        if let Some(sealed) = &sealed {
            if sealed.1.vcs().len() != check.owners().len() || check.owners().is_empty() {
                return Err(fail("complete owner/VC coverage mismatch"));
            }
            for vc in sealed.1.vcs() {
                let actual = mizar_proof::status::check_source_theorem_reflexivity_with_status(
                    arena,
                    &check,
                    env,
                    input.snapshot,
                    vc.id,
                    policies.1,
                )?;
                if actual.0 != sealed.0 || actual.1 != sealed.1 {
                    return Err(fail("complete owner status graphs mismatch"));
                }
                checked_statuses.push(actual);
            }
        } else if check.owners().len() != 1 {
            return no_draft;
        }
        let [lockfile_hash, verifier_config_hash] = [
            (
                if self.build_plan.lockfile.toolchain_version.is_some() {
                    "mizar-build/parsed-lock-content/v2"
                } else {
                    "mizar-build/parsed-lock-content/v1"
                },
                self.snapshot.lockfile_hash,
            ),
            (
                "mizar-build/package-config-content/v1",
                self.snapshot.verifier_config_hash,
            ),
        ]
        .map(|(producer, digest)| {
            ArtifactHashRef::source_owner_fingerprint(producer, digest)
                .expect("native owner producers have fixed supported bindings")
        });
        let mut all_exports = Vec::new();
        let mut all_statements = Vec::new();
        let mut all_expressions: Vec<ExpressionMetadata> = Vec::new();
        let mut all_obligations = Vec::new();
        let mut all_references = Vec::new();
        let mut staged_witnesses = Vec::new();
        let mut cleanup = None;
        for (owner_index, (owner, owner_visibility)) in check.owners().iter().enumerate() {
            let sealed = if builtin {
                Some(
                    unique(checked_statuses.iter().filter(|status| {
                        status.1.vc(status.4.vc()).is_some_and(|vc| {
                            let mizar_vc::vc_ir::AnchorOwner::Theorem(item) = vc.anchor.owner
                            else {
                                return false;
                            };
                            status
                                .0
                                .items()
                                .get(item)
                                .is_some_and(|item| item.symbol == owner.symbol)
                        })
                    }))
                    .ok_or_else(|| fail("requires exact owner status"))?,
                )
            } else {
                None
            };
            let vc_id = sealed.map_or(vc_id, |sealed| sealed.4.vc());
            let witness_path = if check.owners().len() == 1 {
                staging.0.to_owned()
            } else {
                format!(
                    "{}-{owner_index}.json",
                    staging.0.strip_suffix(".json").unwrap_or(staging.0)
                )
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
            let export = unique(
                summary
                    .exported_symbols
                    .iter()
                    .filter(|export| export.origin_id == owner.symbol.fqn().as_str()),
            );
            let (visibility, export) = match (symbol.visibility(), symbol.export_status(), export) {
                (
                    mizar_resolve::env::Visibility::Public,
                    mizar_resolve::env::ExportStatus::Exported,
                    Some(export),
                ) => ("public", Some(export)),
                (
                    mizar_resolve::env::Visibility::Private,
                    mizar_resolve::env::ExportStatus::LocalOnly,
                    None,
                ) if check.owners().len() == 1 && summary.exported_symbols.is_empty() => {
                    ("private", None)
                }
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
                let Some(original) = node.resolved_node.and_then(|id| arena.arena().node(id))
                else {
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
            let exports: Vec<_> = export
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
            let (selection, ((projection, slice, mut witness_draft), file, reference)) =
                if let Some(payload) = normal_payload.take() {
                    (Some(payload.0), payload.1)
                } else if check.owners().len() == 1 {
                    (
                        None,
                        mizar_proof::status::source_theorem_reflexivity_witness_file_candidate(
                            arena,
                            env,
                            self.snapshot,
                            policies.1,
                            &witness_path,
                        )?,
                    )
                } else {
                    let original = mizar_proof::status::source_theorem_reflexivity_witness_draft(
                        arena,
                        &check,
                        env,
                        self.snapshot,
                        vc_id,
                        policies.1,
                        &witness_path,
                    )?;
                    let draft = &original.2;
                    let file = draft.source_theorem_file_value()?;
                    let CanonicalJson::Object(fields) = &file else {
                        return Err(fail("requires FILE object"));
                    };
                    let Some(CanonicalJson::String(file_family)) = fields.get("schema_family")
                    else {
                        return Err(fail("requires FILE family"));
                    };
                    let reference = mizar_artifact::proof_witness::ProofWitnessRef {
                    schema_version: mizar_artifact::store::SchemaVersion::new(3, 0),
                    obligation_id: draft.obligation_id().to_owned(),
                    obligation_fingerprint: draft.obligation_fingerprint().clone(),
                    proof_status: mizar_artifact::proof_witness::ProofStatus::DischargedBuiltin,
                    evidence_kind: mizar_artifact::proof_witness::EvidenceKind::FormulaSubstitutionKernelEvidence,
                    witness_path: draft.witness_path().to_owned(),
                    witness_artifact_hash: ArtifactHashRef::new(
                        ArtifactHashClass::Artifact,
                        file_family.as_str(),
                        mizar_artifact::store::SchemaVersion::new(1, 0),
                        artifact_hash_domain(
                            file_family.as_str(),
                            mizar_artifact::store::SchemaVersion::new(1, 0),
                        ).hash(&file, &[]),
                    ),
                    kernel_acceptance: draft.kernel_acceptance().clone(),
                };
                    mizar_artifact::proof_witness::builtin_proof_witness_ref_json(&reference)
                        .map_err(|error| error.to_string())?;
                    (None, (original, file, reference))
                };
            if let Some(sealed) = &sealed {
                let raw = projection.reuse_metadata();
                let bound = sealed.4.reuse_metadata();
                let (trusted, wire) =
                mizar_proof::status::source_theorem_reflexivity_trusted_witness_metadata_with_wire(
                    arena,
                    &check,
                    env,
                    input.snapshot,
                    vc_id,
                    policies.1,
                )?;
                let evidence = mizar_proof::selection::TrustedKernelEvidence::from_policy_input(
                    &KernelPolicyInput::from_kernel_result(
                        &sealed.2,
                        KernelEvidenceOrigin::BuiltinDischarge,
                    ),
                )
                .ok_or_else(|| fail("requires trusted built-in Kernel evidence"))?;
                if slice != sealed.5
                    || projection.vc() != sealed.4.vc()
                    || projection.obligation_id() != sealed.4.obligation_id()
                    || raw.obligation_anchor() != bound.obligation_anchor()
                    || raw.obligation_fingerprint() != bound.obligation_fingerprint()
                    || raw.vc_fingerprint() != bound.vc_fingerprint()
                    || raw.local_context_fingerprint() != bound.local_context_fingerprint()
                    || raw.dependency_slice_fingerprint() != bound.dependency_slice_fingerprint()
                    || raw.policy_fingerprint() != bound.policy_fingerprint()
                    || witness_draft.payload_bytes() != wire
                    || witness_draft.kernel_acceptance() != trusted.kernel_acceptance()
                    || reference.kernel_acceptance != *trusted.kernel_acceptance()
                    || trusted.trusted_kernel_evidence() != &evidence
                {
                    return Err(fail("complete FILE and sealed evidence mismatch"));
                }
                witness_draft = mizar_proof::witness_store::ProofWitnessDraft::new(
                    witness_draft.obligation_id(),
                    witness_draft.obligation_anchor().clone(),
                    witness_draft.obligation_fingerprint().clone(),
                    &sealed.4,
                    trusted,
                    witness_draft.payload_schema().clone(),
                    witness_draft.payload_bytes().to_vec(),
                    witness_draft.witness_path(),
                    witness_draft
                        .provenance()
                        .clone()
                        .with_trusted_used_axioms_ref(
                            sealed
                                .4
                                .trusted_used_axioms()
                                .ok_or_else(|| fail("requires sealed trusted axioms"))?,
                        ),
                )
                .map_err(|error| error.to_string())?;
            }
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
                    || root.site()
                        != &mizar_checker::typed_ast::TypedSiteRef::Node(root.site().node())
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
            if sealed.as_ref().is_some_and(|sealed| sealed.1 != *vcs) {
                return Err(fail("complete sealed VC mismatch"));
            }
            let Some(vc) = vcs.vc(vc_id) else {
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
            let normal_evidence = if builtin {
                None
            } else {
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
                let selection = selection
                    .as_ref()
                    .ok_or_else(|| fail("requires normal selection"))?;
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
                Some((kernel, trusted, selected.encoded_problem_hash(), candidate))
            };
            PublishedArtifactPath::new(version.normalized_path.as_str())
                .map_err(|_| fail("source path mismatch"))?;
            let path =
                PublishedArtifactPath::new(&witness_path).map_err(|error| error.to_string())?;
            if path.as_str() != reference.witness_path {
                return Err(fail("witness path mismatch"));
            }
            let raw_refs = [
                (
                    "mizar-vc/source-theorem-canonical-vc/v1",
                    reuse.vc_fingerprint(),
                ),
                (
                    "mizar-vc/source-theorem-canonical-context/v1",
                    reuse.local_context_fingerprint(),
                ),
                (
                    mizar_vc::dependency_slice::DEPENDENCY_SLICE_SCHEMA_VERSION,
                    reuse.dependency_slice_fingerprint(),
                ),
            ]
            .map(|(producer, digest)| {
                ArtifactHashRef::source_owner_fingerprint(producer, digest)
                    .expect("native owner producers have fixed supported bindings")
            });
            let [
                vc_fingerprint,
                local_context_fingerprint,
                dependency_slice_fingerprint,
            ] = raw_refs;
            use mizar_core::core_ir::{
                CoreFormulaKind, CoreIr, CoreItemKind, CoreItemStatus, CoreNodeRef,
                CoreProofNodeKind, CoreProofStatus, CoreSourceAnchor, CoreTermKind,
                ObligationSeedKind, ObligationSeedStatus,
            };
            let core = storage
                .typed_handle::<CoreIr>(
                    core_parent.as_output_ref(),
                    &OutputKind::new("SourceTheoremCore"),
                )
                .map_err(|error| error.to_string())?;
            let core = storage.get(&core).map_err(|error| error.to_string())?;
            if sealed.as_ref().is_some_and(|sealed| sealed.0 != *core) {
                return Err(fail("complete sealed Core mismatch"));
            }
            let (Some((item_id, item)), Some((_, proof)), Some((seed_id, seed))) = (
                unique(
                    core.items()
                        .iter()
                        .filter(|(_, item)| item.symbol == owner.symbol),
                ),
                unique(core.proofs().iter().filter(|(_, proof)| {
                    core.items()
                        .get(proof.item)
                        .is_some_and(|item| item.symbol == owner.symbol)
                })),
                unique(core.obligation_seeds().iter().filter(|(_, seed)| {
                    core.items()
                        .get(seed.owner)
                        .is_some_and(|item| item.symbol == owner.symbol)
                })),
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
                || vc.anchor.owner != mizar_vc::vc_ir::AnchorOwner::Theorem(item_id)
                || core.items().len() != check.owners().len()
                || core.proofs().len() != check.owners().len()
                || core.obligation_seeds().len() != check.owners().len()
                || core.proof_nodes().len() != 2 * check.owners().len()
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
                || (!builtin && !citations.is_empty())
                || citations.len() > 1
                || local.role.as_str() != "proof-let"
                || name.is_empty()
                || !seed.context.contains(&guard)
                || seed.context.len() != 1 + citations.len()
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
            let (Some(left_term), Some(right_term)) =
                (core.terms().get(left), core.terms().get(right))
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
            if staged_witnesses.is_empty() {
                std::fs::create_dir(staging.1).map_err(|error| error.to_string())?;
                cleanup = Some(NativeArtifactStagingCleanup(
                    matches!(version.origin, SourceOrigin::OpenBuffer { .. })
                        .then(|| staging.1.to_path_buf()),
                ));
            }
            #[cfg(test)]
            snapshot_test_support::AFTER_LEGACY_ROOT_CREATION.with(|hook| {
                if let Some(hook) = hook.take() {
                    hook();
                }
            });
            let root_lock = lock_artifact_writes().map_err(|error| error.to_string())?;
            reserve_native_artifact_path(
                &std::fs::canonicalize(staging.1).map_err(|error| error.to_string())?,
                &path,
                "native theorem witness parent must stay below artifact root",
            )
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    "native theorem witness requires a fresh witness path".to_owned()
                } else {
                    error.to_string()
                }
            })?;
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
            drop(root_lock);
            let accepted = if let Some(sealed) = &sealed {
                sealed.4.clone().with_source_theorem_witness(&staged)?
            } else {
                let (kernel, trusted, encoded_problem_hash, candidate) =
                    normal_evidence.ok_or_else(|| fail("requires normal evidence"))?;
                let evidence = ProofEvidenceSet::new(
                    kernel.0.target_vc_fingerprint().sort_bytes(),
                    encoded_problem_hash,
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
                let [accepted] = <[_; 1]>::try_from(merged)
                    .map_err(|_| fail("requires one accepted selection"))?;
                project_status(
                    ProofStatusProjectionInput::new(accepted, policies.1.clone(), identity)
                        .with_trusted_used_axioms(trusted),
                )
                .map_err(|error| error.to_string())?
            };
            if accepted.selected_class()
                != if builtin {
                    ProofWinnerClass::DischargedBuiltin
                } else {
                    ProofWinnerClass::KernelVerified
                }
                || accepted.projected_status() != ProjectedProofStatus::Accepted
                || accepted.artifact_publication()
                    != ArtifactStatusPublication::Publishable(
                        CurrentArtifactObligationStatus::Accepted,
                    )
                || accepted.accepted_witness_obligation_id()
                    != Some(reference.obligation_id.as_str())
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
            all_exports.extend(exports);
            all_statements.extend(statements);
            for expression in expressions {
                if let Some(previous) = all_expressions
                    .iter_mut()
                    .find(|previous| previous.expression_id == expression.expression_id)
                {
                    let mut comparable = expression.clone();
                    comparable.active_thesis = previous.active_thesis.clone();
                    if *previous != comparable {
                        return Err(fail("complete expression metadata mismatch"));
                    }
                    if expression.active_thesis.is_some() {
                        previous.active_thesis = expression.active_thesis;
                    }
                } else {
                    all_expressions.push(expression);
                }
            }
            all_obligations.extend(obligations);
            all_references.push(reference);
            staged_witnesses.push(staged);
        }
        if all_exports.len() != summary.exported_symbols.len()
            || all_obligations.len() != check.owners().len()
            || all_references.len() != check.owners().len()
            || all_obligations
                .iter()
                .map(|row| &row.obligation_id)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != check.owners().len()
        {
            return Err(fail("complete artifact owner coverage mismatch"));
        }
        let projection_input = ProjectVerifiedArtifactInput {
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
            exports: all_exports,
            statements: if builtin { Vec::new() } else { all_statements },
            expressions: all_expressions,
            obligations: all_obligations,
            proof_witnesses: all_references,
            diagnostics: Vec::new(),
            provenance: BuildProvenance {
                captured_source: (!builtin).then(|| {
                    (
                        version.source_id,
                        version.origin.clone(),
                        version.source_hash,
                    )
                }),
                toolchain: self.snapshot.toolchain.identity().to_owned(),
                language_edition: version.edition.as_str().to_owned(),
                lockfile_hash,
                verifier_config_hash,
                dependency_artifact_hashes: Vec::new(),
                cache_key: None,
            },
        };
        let service = ArtifactProjectionService::new(Arc::clone(publisher));
        let draft = if builtin {
            service.project_builtin_module(projection_input)
        } else {
            service.project_module(projection_input)
        }
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
        cleanup
            .ok_or_else(|| fail("requires staged witnesses"))?
            .release();
        Ok(Some((draft, staged_witnesses)))
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
        self.write_native_theorem_artifact_draft_route(
            input,
            publisher,
            vc_id,
            Some(encoding),
            (Some(policies.0), policies.1),
            (staging.0, staging.1, staging.2, false),
        )
        .and_then(|result| {
            result
                .map(|(draft, write, stages)| {
                    let [stage] = <[_; 1]>::try_from(stages)
                        .map_err(|_| "normal theorem write requires one witness".to_owned())?;
                    Ok((draft, write, stage))
                })
                .transpose()
        })
    }

    fn write_native_theorem_artifact_draft_route(
        self,
        input: &PhaseInput,
        publisher: &Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: Option<(
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        )>,
        policies: (
            Option<&mizar_vc::discharge::DischargePolicy>,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (&str, &str, &std::path::Path, bool),
    ) -> Result<
        Option<(
            mizar_ir::projection::VerifiedArtifactDraft,
            mizar_artifact::store::PublishedArtifactWrite,
            Vec<mizar_proof::witness_store::ProofWitnessStagedRef>,
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
                artifact_hash_excluded_paths, builtin_verified_artifact_json,
                read_builtin_verified_artifact, read_verified_artifact, verified_artifact_json,
            },
        };
        let Some((draft, staged)) = self.project_native_theorem_artifact_draft_route(
            input,
            publisher,
            vc_id,
            encoding,
            policies,
            (staging.0, staging.2, staging.3),
        )?
        else {
            return Ok(None);
        };
        let cleanup = NativeArtifactStagingCleanup(
            (matches!(
                draft.artifact.provenance.captured_source.as_ref(),
                Some((_, SourceOrigin::OpenBuffer { .. }, _))
            ) || (staging.3
                && self.snapshot.source_versions.iter().any(|version| {
                    version.package_id.as_str() == draft.artifact.module.package_id
                        && version.module_path.as_str() == draft.artifact.module.module_path
                        && matches!(version.origin, SourceOrigin::OpenBuffer { .. })
                })))
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
        #[cfg(test)]
        snapshot_test_support::AFTER_LEGACY_MAIN_ABSENCE.with(|hook| {
            if let Some(hook) = hook.take() {
                hook();
            }
        });
        let value = if staging.3 {
            builtin_verified_artifact_json(&draft.artifact)
        } else {
            verified_artifact_json(&draft.artifact)
        }
        .map_err(|error| error.to_string())?;
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
        // The early absence check does not exclude a concurrent Q1 writer.
        // Reserve only after validation, before the store can replace the main.
        let root = std::fs::canonicalize(staging.2).map_err(|error| error.to_string())?;
        match reserve_native_artifact_path(
            &root,
            &path,
            "native theorem artifact main parent must stay below artifact root",
        ) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err("native theorem artifact main requires a fresh main path".to_owned());
            }
            Err(error) => return Err(error.to_string()),
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
        let decode = if staging.3 {
            read_builtin_verified_artifact
        } else {
            read_verified_artifact
        };
        let decoded = decode(
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
        self.commit_native_theorem_artifact_manifest_route(
            input,
            publisher,
            vc_id,
            Some(encoding),
            (Some(policies.0), policies.1),
            (staging.0, staging.1, staging.2, false),
        )
    }

    #[expect(
        clippy::type_complexity,
        reason = "retain the existing committed values"
    )]
    fn commit_native_theorem_artifact_manifest_route(
        self,
        input: &PhaseInput,
        publisher: &Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        vc_id: mizar_vc::vc_ir::VcId,
        encoding: Option<(
            mizar_atp::problem::LogicProfile,
            mizar_atp::problem::ConcreteFormat,
        )>,
        policies: (
            Option<&mizar_vc::discharge::DischargePolicy>,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (&str, &str, &std::path::Path, bool),
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
            module_summary::{MODULE_SUMMARY_SCHEMA_FAMILY, module_summary_json},
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{
                PublishedArtifactPath, artifact_hash_domain, canonical_json_bytes,
                parse_canonical_json_bytes, write_published_artifact,
            },
            verified_artifact::{
                VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
                artifact_hash_excluded_paths, builtin_verified_artifact_json,
                read_builtin_verified_artifact, read_verified_artifact, verified_artifact_json,
            },
        };
        let Some((draft, main_write, staged)) = self.write_native_theorem_artifact_draft_route(
            input, publisher, vc_id, encoding, policies, staging,
        )?
        else {
            return Ok(None);
        };
        let open_buffer = matches!(
            draft.artifact.provenance.captured_source.as_ref(),
            Some((_, SourceOrigin::OpenBuffer { .. }, _))
        ) || (staging.3
            && self.snapshot.source_versions.iter().any(|version| {
                version.package_id.as_str() == draft.artifact.module.package_id
                    && version.module_path.as_str() == draft.artifact.module.module_path
                    && matches!(version.origin, SourceOrigin::OpenBuffer { .. })
            }));
        let cleanup = NativeArtifactStagingCleanup(open_buffer.then(|| staging.2.to_path_buf()));
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
        let WorkUnit::Module { module } = &input.work_unit else {
            return Err(fail("requires a module"));
        };
        let Some(summary) = self.native_theorem_module_summary_candidate(module)? else {
            return Ok(None);
        };
        if summary.module != artifact.module || summary.source_hash != artifact.source_hash {
            return Err(fail("summary source binding mismatch"));
        }
        let summary_path = PublishedArtifactPath::new(format!("{}.summary.json", staging.1))
            .map_err(|error| error.to_string())?;
        #[cfg(test)]
        snapshot_test_support::AFTER_LEGACY_SUMMARY_FRESHNESS.with(|hook| {
            if let Some(hook) = hook.take() {
                hook();
            }
        });
        let root_lock = lock_artifact_writes().map_err(|error| error.to_string())?;
        reserve_native_artifact_path(
            &root,
            &summary_path,
            "native theorem summary parent must stay below artifact root",
        )
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "native theorem summary requires a fresh summary path".to_owned()
            } else {
                error.to_string()
            }
        })?;
        let summary_write = write_published_artifact(
            &root,
            &summary_path,
            &module_summary_json(&summary).map_err(|error| error.to_string())?,
            &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
            &[],
        )
        .map_err(|error| error.to_string())?;
        drop(root_lock);
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
            module_summary_file: Some(summary_path.as_str().to_owned()),
            module_summary_hash: Some(ArtifactHashRef::new(
                ArtifactHashClass::Artifact,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                summary.schema_version,
                summary_write.artifact_hash,
            )),
            module_summary_interface_hash: Some(ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                summary.schema_version,
                summary.interface_hash,
            )),
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
        if staged.len() != draft.artifact.proof_witnesses.len() {
            return Err(fail("complete staged witness coverage mismatch"));
        }
        let mut published = None;
        for staged in &staged {
            let reference = mizar_proof::witness_store::publish_ref(staged, &proof)
                .map_err(|error| error.to_string())?;
            if published.is_none() {
                published = Some(reference);
            }
        }
        let published = published.ok_or_else(|| fail("requires published witnesses"))?;
        if !freshness(Some(guard.as_str())) {
            return Err(fail("currentness validation failed"));
        }
        let value = if staging.3 {
            builtin_verified_artifact_json(&draft.artifact)
        } else {
            verified_artifact_json(&draft.artifact)
        }
        .map_err(|error| error.to_string())?;
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
        let phase = IrPipelinePhase::from(PipelinePhase::ArtifactCommit);
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
                    let decode =
                        if schema_version == mizar_artifact::store::SchemaVersion::new(2, 0) {
                            read_builtin_verified_artifact
                        } else {
                            read_verified_artifact
                        };
                    let artifact = decode(
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
pub(crate) const COMMENT_ARTIFACT_RESERVATION: &str = ".driver-q1-reservation";

// A shared lock also fences cleanup of ancestor roots; never unlink its stable file.
pub(crate) fn lock_artifact_writes() -> std::io::Result<std::fs::File> {
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(std::env::temp_dir().join("mizar-driver-artifact-writes.lock"))?;
    lock.lock()?;
    Ok(lock)
}

pub(crate) fn reserve_native_artifact_path(
    root: &std::path::Path,
    path: &mizar_artifact::store::PublishedArtifactPath,
    parent_error: &str,
) -> std::io::Result<()> {
    let relative = std::path::Path::new(path.as_str());
    let mut parent = root.to_path_buf();
    for component in relative
        .parent()
        .expect("validated relative path")
        .components()
    {
        parent.push(component);
        match std::fs::create_dir(&parent) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        parent = parent.canonicalize()?;
        if !parent.starts_with(root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                parent_error,
            ));
        }
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(parent.join(relative.file_name().expect("validated main path")))?;
    Ok(())
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
    let Ok(_root_lock) = lock_artifact_writes() else {
        return;
    };
    // A nested legacy staging root may contain a main or witness owned by Q1 above it.
    for ancestor in root.ancestors().skip(1) {
        match std::fs::symlink_metadata(ancestor.join(COMMENT_ARTIFACT_RESERVATION)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return,
        }
    }
    let Ok(metadata) = std::fs::symlink_metadata(&root) else {
        return;
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        let mut directories = vec![root.clone()];
        while let Some(directory) = directories.pop() {
            let Ok(entries) = std::fs::read_dir(directory) else {
                return;
            };
            for entry in entries {
                let Ok(entry) = entry else { return };
                if entry.file_name() == COMMENT_ARTIFACT_RESERVATION {
                    return;
                }
                let Ok(kind) = entry.file_type() else { return };
                if kind.is_dir() {
                    directories.push(entry.path());
                }
            }
        }
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
        let nested = owned.join("nested");
        fs::create_dir_all(&nested).unwrap();
        let marker = nested.join(super::COMMENT_ARTIFACT_RESERVATION);
        fs::write(&marker, b"reserved").unwrap();
        drop(super::NativeArtifactStagingCleanup(Some(owned)));
        assert_eq!(fs::read(marker).unwrap(), b"reserved");
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
    pub(crate) build_request: mizar_session::BuildRequestId,
    pub(crate) artifact_roots: Vec<PathBuf>,
    pub(crate) lockfile_producer: &'static str,
    pub(crate) captured: crate::request::CapturedSnapshot,
    pub(crate) diagnostics: mizar_diagnostics::aggregator::BuildDiagnosticIndex,
    pub(crate) manifests: Vec<mizar_artifact::manifest::PublishedManifestRead>,
    pub(crate) artifacts: Vec<Arc<mizar_artifact::verified_artifact::VerifiedArtifact>>,
    pub(crate) development_artifacts: Vec<mizar_artifact::store::PublishedArtifactRead>,
    pub(crate) source_metadata: Vec<Option<PublishedSourceMetadata>>,
}

/// Owner projections bound to the matching manifest module and original source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedSourceMetadata {
    summary: mizar_artifact::module_summary::ModuleSummary,
    notation: Option<mizar_resolve::symbols::SourceNotationView>,
    imports: Option<mizar_resolve::imports::PublishedImportView>,
    direct_by: Option<mizar_artifact::direct_by::DirectByProjection>,
    identity: Option<mizar_artifact::verified_artifact::PublishedIdentityProjection>,
}

impl PublishedSourceMetadata {
    pub fn summary(&self) -> &mizar_artifact::module_summary::ModuleSummary {
        &self.summary
    }
    pub fn notation(&self) -> Option<&mizar_resolve::symbols::SourceNotationView> {
        self.notation.as_ref()
    }
    pub fn imports(&self) -> Option<&mizar_resolve::imports::PublishedImportView> {
        self.imports.as_ref()
    }
    pub fn direct_by(&self) -> Option<&mizar_artifact::direct_by::DirectByProjection> {
        self.direct_by.as_ref()
    }
    pub fn identity(
        &self,
    ) -> Option<&mizar_artifact::verified_artifact::PublishedIdentityProjection> {
        self.identity.as_ref()
    }
}

impl SnapshotPublication {
    pub fn build_request(&self) -> mizar_session::BuildRequestId {
        self.build_request
    }

    /// Original roots, in manifest order; copying an artifact grants no provenance.
    pub fn artifact_roots(&self) -> &[PathBuf] {
        &self.artifact_roots
    }

    /// Exact lock content producer selected by the captured typed build plan.
    pub fn lockfile_producer(&self) -> &'static str {
        self.lockfile_producer
    }

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

    /// Same order as artifacts; unavailable products remain distinct from complete-empty views.
    pub fn source_metadata(&self) -> &[Option<PublishedSourceMetadata>] {
        &self.source_metadata
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
        SourceTheoremArtifactCommitService::execute_route(
            input,
            context,
            (Some(&self.check), &self.check.verifier_policy),
            (
                self.check.witness_path_prefix.as_deref(),
                &self.main_path,
                &self.output_parent,
            ),
        )
    }
}

pub(crate) struct SourceTheoremReflexivityArtifactCommitService {
    pub(crate) verifier_policy: mizar_proof::policy::VerifierPolicy,
    pub(crate) witness_path_prefix: String,
    pub(crate) main_path: String,
    pub(crate) output_parent: String,
}

impl PhaseService for SourceTheoremReflexivityArtifactCommitService {
    fn phase(&self) -> PhaseDescriptor {
        PhaseDescriptor::new(
            "SourceTheoremReflexivityArtifactCommit",
            PhaseOwner::MizarArtifact,
            vec![PipelinePhase::ArtifactCommit],
            format!(
                "mizar-artifact/source-theorem-reflexivity-artifact-commit/v1;verifier_policy={};witness_path_prefix={}:{};main_path={}:{}",
                self.verifier_policy.policy_fingerprint().to_lower_hex(),
                self.witness_path_prefix.len(), self.witness_path_prefix,
                self.main_path.len(), self.main_path,
            ),
            "SourceTheoremArtifactData",
        ).expect("source theorem reflexivity Artifact descriptor is fixed and valid")
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        PhaseCacheIntent::NoKey {
            reason:
                "source theorem reflexivity artifact commit has no complete dependency footprint"
                    .to_owned(),
        }
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext<'_>) -> PhaseResult {
        SourceTheoremArtifactCommitService::execute_route(
            input,
            context,
            (None, &self.verifier_policy),
            (
                Some(&self.witness_path_prefix),
                &self.main_path,
                &self.output_parent,
            ),
        )
    }
}

impl SourceTheoremArtifactCommitService {
    fn execute_route(
        input: PhaseInput,
        context: PhaseExecutionContext<'_>,
        route: (
            Option<&SourceTheoremCandidateCheckService>,
            &mizar_proof::policy::VerifierPolicy,
        ),
        staging: (Option<&str>, &str, &str),
    ) -> PhaseResult {
        let (normal, verifier_policy) = route;
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
        if normal.is_some_and(|check| check.candidate.configuration_tags().is_none()) {
            return blocking();
        }
        let Some(prefix) = staging.0 else {
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
            || input.parent_outputs().len() != if normal.is_some() { 6 } else { 5 }
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
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::VcGenerate)
                && parent.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
        })) else {
            return blocking();
        };
        let Some(kernel) = unique(input.parent_outputs().iter().filter(|parent| {
            parent.as_output_ref().phase() == &IrPipelinePhase::from(PipelinePhase::KernelCheck)
                && parent.as_output_ref().output_kind()
                    == &OutputKind::new(if normal.is_some() {
                        "SourceTheoremCandidateCheck"
                    } else {
                        "SourceTheoremReflexivityKernelStatus"
                    })
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
        let Some(vc) = vcs.vcs().first() else {
            return blocking();
        };
        if (normal.is_some() && vcs.vcs().len() != 1)
            || (normal.is_some()
                && storage
                    .typed_handle::<(
                        mizar_kernel::checker::KernelCheckResult,
                        mizar_proof::policy::PolicyDecision,
                    )>(
                        kernel.as_output_ref(),
                        &OutputKind::new("SourceTheoremCandidateCheck"),
                    )
                    .is_err())
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
        let Ok(parent) = std::fs::canonicalize(package_root.join(staging.2)) else {
            return blocking();
        };
        if !parent.is_dir()
            || !parent
                .strip_prefix(&package_root)
                .is_ok_and(|relative| !relative.as_os_str().is_empty())
        {
            return blocking();
        }
        let snapshot_hex = input.snapshot.fingerprint().to_lower_hex();
        let source_hex = source_key.to_lower_hex();
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
        let kernel_hex = kernel.as_output_ref().content_hash().to_lower_hex();
        let witness_path = format!("{prefix}/{kernel_hex}.json");
        let result = inputs.commit_native_theorem_artifact_manifest_route(
            &input,
            &publisher,
            vc.id,
            normal.map(|check| {
                (
                    check.candidate.logic_profile.clone(),
                    check.candidate.concrete_format,
                )
            }),
            (normal.map(|check| &check.candidate.policy), verifier_policy),
            (&witness_path, staging.1, &root, normal.is_none()),
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

    std::thread_local! {
        pub(super) static AFTER_LEGACY_MAIN_ABSENCE: std::cell::RefCell<Option<Box<dyn FnOnce()>>>
            = const { std::cell::RefCell::new(None) };
        pub(super) static AFTER_LEGACY_ROOT_CREATION: std::cell::RefCell<Option<Box<dyn FnOnce()>>>
            = const { std::cell::RefCell::new(None) };
        pub(super) static AFTER_LEGACY_SUMMARY_FRESHNESS: std::cell::RefCell<Option<Box<dyn FnOnce()>>>
            = const { std::cell::RefCell::new(None) };
    }

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
                submission.session.request().id,
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

    #[test]
    fn mixed_legacy_and_commented_writers_preserve_the_winning_publication() {
        use std::{cell::RefCell, rc::Rc};

        use mizar_artifact::manifest::{ManifestFileReadOptions, read_manifest_file};
        use mizar_atp::problem::*;
        use mizar_ir::{
            dispatch_input::{PhaseDispatchInputBundle, SealedParentOutputHandle},
            identity::OutputKind,
            projection::ProjectVerifiedArtifactInput,
        };
        use mizar_proof::policy::VerifierPolicy;
        use mizar_vc::{
            discharge::{ComputationLimit, DischargePolicy},
            vc_ir::{PolicyKey, VcSet},
        };

        use super::super::*;

        for (
            legacy_open,
            obsolete_legacy,
            after_root_creation,
            witness_collision,
            nested_root,
            summary_collision,
        ) in [
            (false, false, false, false, false, false),
            (true, false, false, false, false, false),
            (true, true, false, false, false, false),
            (true, true, true, false, false, false),
            (true, false, true, true, false, false),
            (true, false, false, false, true, false),
            (true, true, false, false, false, true),
        ] {
            let fixture = Rc::new(Fixture::new());
            let (session, input, publisher) = fixture.prepare_with_open_buffers(
                1,
                if legacy_open {
                    &[("main", 7), ("leaf", 7)]
                } else {
                    &[]
                },
            );
            let submission = CompilerDriver::new(registry(false))
                .with_output_publisher(publisher.clone())
                .submit_captured(
                    session,
                    &fixture.ids,
                    &fixture.snapshots,
                    input,
                    Default::default(),
                )
                .unwrap();
            let source = submission
                .session
                .captured()
                .native_sources()
                .unwrap()
                .iter()
                .find(|(_, source, _)| source.module_path.as_str() == "main")
                .unwrap();
            assert_eq!(
                matches!(source.1.origin, SourceOrigin::OpenBuffer { .. }),
                legacy_open
            );
            let inputs = crate::registry::SourceLoadInputs {
                snapshot: submission.session.captured().snapshot(),
                captured: Some(submission.session.captured()),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &fixture.ids,
            };
            let outputs = submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .filter(|output| output.work_unit().as_str() == "\"alpha\":\"main\"")
                .collect::<Vec<_>>();
            let output = |kind| {
                (*outputs
                    .iter()
                    .find(|output| output.output_kind() == &OutputKind::new(kind))
                    .unwrap())
                .clone()
            };
            let q1_fixture = Rc::new(Fixture {
                root: fixture.root.clone(),
                ids: mizar_session::InMemorySessionIdAllocator::new(),
                snapshots: mizar_session::SnapshotRegistry::new(),
            });
            let (mut retained, q1_input, q1_publisher) = q1_fixture.prepare(1);
            let q1_submission = CompilerDriver::new(registry(false))
                .with_output_publisher(q1_publisher.clone())
                .submit_captured(
                    retained.clone(),
                    &q1_fixture.ids,
                    &q1_fixture.snapshots,
                    q1_input,
                    Default::default(),
                )
                .unwrap();
            let q1_source = q1_submission
                .session
                .captured()
                .native_sources()
                .unwrap()
                .iter()
                .find(|(_, source, _)| source.module_path.as_str() == "main")
                .unwrap();
            let q1_outputs = q1_submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .filter(|output| output.work_unit().as_str() == "\"alpha\":\"main\"")
                .collect::<Vec<_>>();
            let q1_output = |kind| {
                (*q1_outputs
                    .iter()
                    .find(|output| output.output_kind() == &OutputKind::new(kind))
                    .unwrap())
                .clone()
            };
            let frontend_ref = q1_output("FrontendOutput");
            let frontend = q1_publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &frontend_ref,
                    frontend_ref.output_kind(),
                )
                .unwrap();
            let q1_inputs = crate::registry::SourceLoadInputs {
                snapshot: q1_submission.session.captured().snapshot(),
                captured: Some(q1_submission.session.captured()),
                build_plan: q1_submission.build_plan.as_ref().unwrap(),
                module_index: q1_submission.module_index.as_ref().unwrap(),
                allocator: &q1_fixture.ids,
            };
            let (_, metadata) = Frontend::new(
                FrontendSourceLoader::new(q1_source.0.clone()),
                q1_inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded_with_comment_metadata(
                q1_publisher
                    .storage()
                    .get(&frontend)
                    .unwrap()
                    .source
                    .clone(),
                &q1_source.1.text,
            )
            .unwrap();
            let role_names = [
                "SourceTheoremModuleSummary",
                "SourceTheoremCore",
                "SourceTheoremVC",
                "SourceTheoremCandidateCheck",
                "SourceTheoremDischarge",
                "SourceTheoremCandidateWitnessFile",
            ];
            let roles = role_names.map(output);
            let q1_roles = role_names.map(q1_output);
            let required = q1_roles
                .iter()
                .cloned()
                .chain([frontend_ref])
                .collect::<Vec<_>>();
            let phase = PhaseInput::new(
                mizar_build::task_graph::WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        source.1.package_id.clone(),
                        source.1.module_path.clone(),
                    ),
                },
                PhaseDispatchInputBundle::new(
                    inputs.snapshot.id,
                    super::super::source_input_hash(
                        inputs
                            .snapshot
                            .source_versions
                            .iter()
                            .find(|version| version.source_id == source.1.source_id)
                            .unwrap(),
                    ),
                    vec![],
                    roles
                        .iter()
                        .map(|output| {
                            SealedParentOutputHandle::from_published_output(
                                &publisher,
                                inputs.snapshot.id,
                                output.clone(),
                            )
                            .unwrap()
                        })
                        .collect(),
                )
                .unwrap(),
            );
            let vcs = publisher
                .storage()
                .get(
                    &publisher
                        .storage()
                        .typed_handle::<VcSet>(&roles[2], roles[2].output_kind())
                        .unwrap(),
                )
                .unwrap();
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
            let policy = DischargePolicy {
                computation_limit: ComputationLimit {
                    policy: PolicyKey::new("snapshot-reflexivity"),
                    max_steps: 64,
                },
            };
            let verifier = VerifierPolicy::default();
            let q1_phase = PhaseInput::new(
                phase.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    q1_inputs.snapshot.id,
                    super::super::source_input_hash(
                        q1_inputs
                            .snapshot
                            .source_versions
                            .iter()
                            .find(|version| version.source_id == q1_source.1.source_id)
                            .unwrap(),
                    ),
                    vec![],
                    q1_roles
                        .iter()
                        .map(|output| {
                            SealedParentOutputHandle::from_published_output(
                                &q1_publisher,
                                q1_inputs.snapshot.id,
                                output.clone(),
                            )
                            .unwrap()
                        })
                        .collect(),
                )
                .unwrap(),
            );
            let q1_vcs = q1_publisher
                .storage()
                .get(
                    &q1_publisher
                        .storage()
                        .typed_handle::<VcSet>(&q1_roles[2], q1_roles[2].output_kind())
                        .unwrap(),
                )
                .unwrap();
            let q1_staging = fixture.root.join("alpha/artifacts/q1-staging");
            let q1_witness = if summary_collision {
                "proof-witnesses/L.mizir.json.summary.json"
            } else {
                "proof-witnesses/q1.json"
            };
            let (q1_draft, _) = q1_inputs
                .project_native_theorem_artifact_draft(
                    &q1_phase,
                    &q1_publisher,
                    q1_vcs.vcs()[0].id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    (q1_witness, &q1_staging),
                )
                .unwrap()
                .unwrap();
            let root = fixture.root.join("alpha/artifacts/mixed");
            let legacy_root = if nested_root {
                fs::create_dir(&root).unwrap();
                root.join("nested")
            } else {
                root.clone()
            };
            let legacy_witness = if witness_collision {
                "proof-witnesses/shared.mizir.json"
            } else {
                "proof-witnesses/shared.json"
            };
            let legacy_main = if summary_collision {
                "proof-witnesses/L.mizir.json"
            } else {
                "shared.mizir.json"
            };
            let q1_main = if nested_root {
                "nested/shared.mizir.json"
            } else if summary_collision {
                "q1.mizir.json"
            } else if witness_collision {
                legacy_witness
            } else {
                "shared.mizir.json"
            };
            let source_id = q1_source.1.source_id;
            let legacy_snapshot = inputs.snapshot.id;
            let saved = Rc::new(RefCell::new(None));
            let hook = if summary_collision {
                &AFTER_LEGACY_SUMMARY_FRESHNESS
            } else if after_root_creation {
                &AFTER_LEGACY_ROOT_CREATION
            } else {
                &AFTER_LEGACY_MAIN_ABSENCE
            };
            hook.set(Some(Box::new({
                let fixture = Rc::clone(&q1_fixture);
                let legacy_publisher = Arc::clone(&publisher);
                let publisher = Arc::clone(&q1_publisher);
                let saved = Rc::clone(&saved);
                let root = root.clone();
                move || {
                    let artifact = &q1_draft.artifact;
                    let metadata = metadata.unwrap();
                    let mut provenance = artifact.provenance.clone();
                    provenance.lockfile_hash.digest = retained.captured().snapshot().lockfile_hash;
                    provenance.verifier_config_hash.digest =
                        retained.captured().snapshot().verifier_config_hash;
                    retained
                        .capture_commented_module(
                            &publisher,
                            &fixture.snapshots,
                            ProjectVerifiedArtifactInput {
                                snapshot: retained.captured().snapshot().id,
                                required_outputs: required,
                                module: artifact.module.clone(),
                                source_file: artifact.source_file.clone(),
                                source_hash: artifact.source_hash,
                                verified_at: artifact.verified_at.clone(),
                                exports: artifact.exports.clone(),
                                statements: artifact.statements.clone(),
                                expressions: artifact.expressions.clone(),
                                obligations: artifact.obligations.clone(),
                                proof_witnesses: artifact.proof_witnesses.clone(),
                                diagnostics: artifact.diagnostics.clone(),
                                provenance,
                            },
                            &frontend,
                            &metadata,
                            &vec![None; metadata.boundaries().len()],
                        )
                        .unwrap();
                    fs::create_dir_all(root.join("proof-witnesses")).unwrap();
                    fs::copy(q1_staging.join(q1_witness), root.join(q1_witness)).unwrap();
                    let commit = retained
                        .emit_commented_module(
                            &publisher,
                            &fixture.snapshots,
                            source_id,
                            &root,
                            q1_main,
                        )
                        .unwrap()
                        .unwrap();
                    *saved.borrow_mut() = Some((
                        commit,
                        fs::read(root.join(q1_main)).unwrap(),
                        fs::read(root.join("artifact-manifest.json")).unwrap(),
                        fs::read(root.join(q1_witness)).unwrap(),
                    ));
                    if obsolete_legacy {
                        legacy_publisher.mark_obsolete(legacy_snapshot).unwrap();
                        publisher
                            .validate_current_snapshot(retained.captured().snapshot().id)
                            .unwrap();
                    }
                }
            })));
            let refusal = inputs
                .commit_native_theorem_artifact_manifest(
                    &phase,
                    &publisher,
                    vcs.vcs()[0].id,
                    (profile, ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    (legacy_witness, legacy_main, &legacy_root),
                )
                .unwrap_err();
            let saved = saved.borrow();
            let (commit, main, manifest, witness) =
                saved.as_ref().expect("Q1 won during legacy staging");
            assert_eq!(fs::read(root.join(q1_main)).unwrap(), *main);
            assert_eq!(fs::read(root.join(q1_witness)).unwrap(), *witness);
            assert_eq!(
                fs::read(root.join("artifact-manifest.json")).unwrap(),
                *manifest
            );
            assert_eq!(
                read_manifest_file(
                    &root,
                    ManifestFileReadOptions {
                        validate_references: true,
                        validate_witness_files: true,
                        expected_manifest_hash: Some(commit.write().artifact_hash),
                        ..ManifestFileReadOptions::default()
                    },
                )
                .unwrap()
                .manifest,
                *commit.manifest(),
            );
            assert!(
                refusal.contains(if summary_collision {
                    "fresh summary path"
                } else if obsolete_legacy {
                    "obsolete"
                } else if witness_collision {
                    "fresh witness path"
                } else {
                    "fresh main path"
                }),
                "{refusal}",
            );
        }
    }

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
    fn authenticated_same_run_imports_survive_snapshot_capture() {
        use super::super::*;
        use mizar_artifact::{
            manifest::{ManifestCommitOptions, ManifestTransaction},
            module_summary::{
                MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummaryReadOptions, module_summary_json,
                read_module_summary,
            },
            store::{
                PublishedArtifactPath, artifact_hash_domain, parse_canonical_json_bytes,
                write_published_artifact,
            },
            verified_artifact::{
                VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
                artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
            },
        };
        use mizar_build::module_index::DependencyModuleSummaryRef;
        use mizar_resolve::{
            declarations::DeclarationShellCollector,
            env::NamespacePath,
            imports::ImportPathResolver,
            module_index::{IndexedModuleId, ModuleIndexInput},
            resolved_ast::ModuleId,
            symbols::SignatureProjectionExtractor,
        };

        fn seal<T: Send + Sync + 'static>(
            owner: &mizar_ir::publisher::PhaseOutputPublisher,
            snapshot: mizar_session::BuildSnapshotId,
            identity: (IrPipelinePhase, IrWorkUnit, OutputKind),
            payload: T,
            bytes: Vec<u8>,
            decode: BlobDecoder<T>,
            parents: Vec<mizar_ir::storage::AnyPhaseOutputRef>,
        ) -> mizar_ir::storage::AnyPhaseOutputRef {
            publish_native_output(
                owner,
                snapshot,
                identity,
                |slot, phase, work_unit, output_kind| PublishOutputInput {
                    slot,
                    snapshot,
                    phase,
                    work_unit,
                    output_kind,
                    schema_version: SchemaVersion::new(1),
                    payload,
                    canonical_payload: Some(bytes.clone()),
                    storage_payload: Some(bytes),
                    decode,
                    parents,
                    named_input_hashes: vec![],
                    side_tables: IrSideTables::default(),
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                },
            )
            .unwrap()
            .erase()
        }

        let fixture = Fixture::new();
        let (baseline, baseline_owner) = publish(&fixture, 1);
        let baseline = fixture.resolve(&baseline, &baseline_owner).unwrap();
        fs::write(
            fixture.root.join("alpha/src/main.miz"),
            "import alpha.leaf as First, alpha.leaf as Second;",
        )
        .unwrap();
        fs::write(fixture.root.join("alpha/src/leaf.miz"), "").unwrap();
        let (session, input, owner) = fixture.prepare(2);
        let mut submission = CompilerDriver::new(registry(false))
            .with_output_publisher(owner.clone())
            .submit_captured(
                session,
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .unwrap();
        let snapshot = submission.session.captured().snapshot().id;
        let leaf_summary_ref = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .find(|output| {
                output.work_unit().as_str() == module_work_unit_label("alpha", "leaf")
                    && output.output_kind() == &OutputKind::new("SourceTheoremModuleSummary")
            })
            .unwrap()
            .clone();
        let leaf_summary = owner
            .storage()
            .get(
                &owner
                    .storage()
                    .typed_handle::<mizar_artifact::module_summary::ModuleSummary>(
                        &leaf_summary_ref,
                        leaf_summary_ref.output_kind(),
                    )
                    .unwrap(),
            )
            .unwrap()
            .as_ref()
            .clone();
        let leaf_value = module_summary_json(&leaf_summary).unwrap();
        let leaf_module = IndexedModuleId::new(
            PackageId::new("alpha"),
            mizar_session::ModulePath::new("leaf"),
        );
        submission
            .module_index
            .as_mut()
            .unwrap()
            .dependency_summaries
            .push(DependencyModuleSummaryRef {
                module: leaf_module.clone(),
                artifact: "leaf.summary.json".into(),
                content_hash: artifact_hash_domain(
                    MODULE_SUMMARY_SCHEMA_FAMILY,
                    leaf_summary.schema_version,
                )
                .hash(&leaf_value, &[]),
            });
        let lexical = ModuleLexicalSummary::from_exported_symbols(
            mizar_frontend::lexical_env::ModuleId::new(
                mizar_artifact::store::canonical_json_string(
                    &leaf_summary.module.canonical_json().unwrap(),
                ),
            ),
            vec![],
        )
        .unwrap();
        let workspace_summaries = [(leaf_module, lexical)];
        let inputs = crate::registry::SourceLoadInputs {
            snapshot: submission.session.captured().snapshot(),
            captured: Some(submission.session.captured()),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &fixture.ids,
        };
        let version = inputs
            .snapshot
            .source_versions
            .iter()
            .find(|version| version.module_path.as_str() == "main")
            .unwrap();
        let (source, source_input) = inputs.native_source(version).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
            DependencyLexicalProvider {
                inputs,
                artifact_roots: &[],
                workspace_summaries: &workspace_summaries,
            },
            MizarParserSeam,
        )
        .run_loaded(source.clone())
        .unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "{:?}",
            frontend.diagnostics
        );
        let module = ModuleId::new(source.package_id.clone(), source.module_path.clone());
        let ast = frontend.ast.as_ref().unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collection =
            SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main"))
                .collect(&module);
        let main_identity = baseline
            .source_metadata()
            .iter()
            .flatten()
            .find(|metadata| metadata.summary().module.module_path == "main")
            .unwrap()
            .summary()
            .module
            .clone();
        let declarations = collection
            .source_notation_module_summary_candidate(&frontend, &main_identity)
            .unwrap()
            .unwrap();
        let imports = ImportPathResolver::new(ModuleIndexInput::new(inputs.module_index))
            .publish_frontend(&frontend, &source, |_, _| Some(leaf_value.clone()))
            .unwrap();
        let main_summary = imports.compose_summary(declarations).unwrap();
        let frontend_bytes = frontend.canonical_disk_bytes().unwrap();
        let decode_input = source_input.clone();
        let source_id = source.source_id;
        let frontend_ref = seal(
            &owner,
            snapshot,
            (
                IrPipelinePhase::from(PipelinePhase::Frontend),
                IrWorkUnit::new(module_work_unit_label("alpha", "main")),
                OutputKind::new("FrontendOutput"),
            ),
            frontend,
            frontend_bytes,
            BlobDecoder::new(move |bytes| {
                FrontendOutput::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .ok_or_else(|| BlobDecodeError::new("invalid fixture frontend"))
            }),
            vec![],
        );

        // Assemble the publication boundary with real owner handles. The native proof route
        // deliberately remains leaf-only; these artifacts are publication test fixtures.
        let mut published = Vec::new();
        for (artifact, manifest) in baseline.artifacts().iter().zip(baseline.manifests()) {
            let name = artifact.module.module_path.as_str();
            let summary = if name == "main" {
                main_summary.clone()
            } else {
                leaf_summary.clone()
            };
            let version = submission
                .session
                .captured()
                .snapshot()
                .source_versions
                .iter()
                .find(|version| version.module_path.as_str() == name)
                .unwrap();
            let mut artifact = artifact.as_ref().clone();
            artifact.source_hash = version.source_hash;
            artifact.provenance.captured_source = Some((
                version.source_id,
                version.origin.clone(),
                version.source_hash,
            ));
            let relative_root = format!("artifacts/import-capture-{name}");
            let root = fixture.root.join("alpha").join(&relative_root);
            fs::create_dir_all(&root).unwrap();
            let old_root = fixture
                .root
                .join("alpha")
                .join(&manifest.manifest.artifact_root);
            for witness in &artifact.proof_witnesses {
                let target = root.join(&witness.witness_path);
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::copy(old_root.join(&witness.witness_path), target).unwrap();
            }
            let mut entry = manifest.manifest.modules[0].clone();
            entry.source_hash = version.source_hash;
            artifact.refresh_hashes().unwrap();
            entry.interface_hash.digest = artifact.interface_hash;
            entry.implementation_hash.digest = artifact.implementation_hash;
            let value = verified_artifact_json(&artifact).unwrap();
            let artifact_write = write_published_artifact(
                &root,
                &PublishedArtifactPath::new(&entry.artifact_file).unwrap(),
                &value,
                &artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version),
                &artifact_hash_excluded_paths(),
            )
            .unwrap();
            entry.artifact_hash.digest = artifact_write.artifact_hash;
            let summary_bytes =
                mizar_artifact::module_summary::write_module_summary(&summary).unwrap();
            let summary_write = write_published_artifact(
                &root,
                &PublishedArtifactPath::new(entry.module_summary_file.as_ref().unwrap()).unwrap(),
                &module_summary_json(&summary).unwrap(),
                &artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version),
                &[],
            )
            .unwrap();
            entry.module_summary_hash.as_mut().unwrap().digest = summary_write.artifact_hash;
            entry.module_summary_interface_hash.as_mut().unwrap().digest = summary.interface_hash;
            let mut seed = manifest.manifest.clone();
            seed.artifact_root = relative_root;
            seed.modules.clear();
            let mut transaction = ManifestTransaction::begin(&root, seed, None).unwrap();
            transaction.stage_module(entry).unwrap();
            let commit = transaction
                .commit(ManifestCommitOptions::default())
                .unwrap();
            let summary_ref = if name == "main" {
                seal(
                    &owner,
                    snapshot,
                    (
                        IrPipelinePhase::from(PipelinePhase::SignatureCollection),
                        IrWorkUnit::new(module_work_unit_label("alpha", name)),
                        OutputKind::new("SourceTheoremModuleSummary"),
                    ),
                    summary,
                    summary_bytes,
                    BlobDecoder::new(|bytes| {
                        let value = parse_canonical_json_bytes(bytes, "summary")
                            .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                        read_module_summary(&value, ModuleSummaryReadOptions::default())
                            .map_err(|error| BlobDecodeError::new(error.to_string()))
                    }),
                    vec![frontend_ref.clone()],
                )
            } else {
                leaf_summary_ref.clone()
            };
            let bytes =
                mizar_artifact::verified_artifact::write_verified_artifact(&artifact).unwrap();
            let artifact_ref = seal(
                &owner,
                snapshot,
                (
                    IrPipelinePhase::from(PipelinePhase::ArtifactCommit),
                    IrWorkUnit::new(module_work_unit_label("alpha", name)),
                    OutputKind::new("SourceTheoremArtifactData"),
                ),
                artifact,
                bytes,
                BlobDecoder::new(|bytes| {
                    let value = parse_canonical_json_bytes(bytes, "artifact")
                        .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                    read_verified_artifact(&value, VerifiedArtifactReadOptions::default())
                        .map_err(|error| BlobDecodeError::new(error.to_string()))
                }),
                vec![summary_ref.clone()],
            );
            published.push((name.to_owned(), summary_ref, artifact_ref, commit));
        }
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_mut().unwrap();
        for state in &mut run.task_states {
            state.state = mizar_build::scheduler::TaskState::Completed;
        }
        for results in run.phase_results.values_mut() {
            for result in results {
                result.output_refs.retain(|output| {
                    output.work_unit().as_str() != module_work_unit_label("alpha", "main")
                });
            }
        }
        for (name, summary_ref, artifact_ref, commit) in published {
            let task = graph
                .tasks()
                .iter()
                .find(|task| {
                    task.kind == mizar_build::task_graph::TaskKind::ArtifactCommit
                        && matches!(&task.unit, WorkUnit::Module { module } if module.package.as_str() == "alpha" && module.path.as_str() == name)
                })
                .unwrap();
            run.phase_results.insert(
                task.id.clone(),
                vec![PhaseResult {
                    output_refs: vec![artifact_ref],
                    artifact_commit: Some(commit),
                    ..PhaseResult::complete()
                }],
            );
            if name == "main" {
                let task = graph
                    .tasks()
                    .iter()
                    .find(|task| {
                        task.phases == [PipelinePhase::Frontend]
                            && matches!(&task.unit, WorkUnit::Module { module } if module.package.as_str() == "alpha" && module.path.as_str() == "main")
                    })
                    .unwrap();
                run.phase_results.insert(
                    task.id.clone(),
                    vec![PhaseResult {
                        output_refs: vec![frontend_ref.clone(), summary_ref],
                        ..PhaseResult::complete()
                    }],
                );
            }
        }
        let publication = fixture.resolve(&submission, &owner).unwrap();
        let metadata = publication
            .source_metadata()
            .iter()
            .flatten()
            .find(|metadata| metadata.summary().module.module_path == "main")
            .unwrap();
        let view = metadata.imports().unwrap();
        assert_eq!(view.source_id(), source_id);
        assert_eq!(view.source_hash(), main_summary.source_hash);
        assert_eq!(
            view.visible_surfaces()
                .unwrap()
                .map(|(binding, _)| binding.alias().to_owned())
                .collect::<Vec<_>>(),
            ["First", "Second"]
        );
        assert_eq!(metadata.summary(), &main_summary);
        for results in submission
            .scheduler_run
            .as_mut()
            .unwrap()
            .phase_results
            .values_mut()
        {
            for result in results {
                result.output_refs.retain(|output| {
                    !(output.work_unit().as_str() == module_work_unit_label("alpha", "leaf")
                        && output.output_kind() == &OutputKind::new("SourceTheoremModuleSummary"))
                });
            }
        }
        let unavailable = fixture.resolve(&submission, &owner).unwrap();
        let index = unavailable
            .artifacts()
            .iter()
            .position(|artifact| artifact.module.module_path == "main")
            .unwrap();
        assert!(unavailable.source_metadata()[index].is_none());
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

        // Genuine source compilation supplies identity products before Doc consumes them.
        use mizar_doc::doc_build::*;
        let requests = publication
            .manifests()
            .iter()
            .map(|read| DocPublicationRequest {
                package: read.manifest.package.clone(),
                artifact_root: fixture
                    .root
                    .join("alpha")
                    .join(&read.manifest.artifact_root),
                expected_manifest_hash: read.artifact_hash,
                output_dir: fixture.root.join("identity-doc-output"),
                source_bundle: None,
                dependency_indexes: None,
                import_order: Default::default(),
            })
            .collect::<Vec<_>>();
        let snapshot = publication.captured().snapshot();
        let mut doc_request = DocBuildRequest {
            profile: Phase16Profile {
                render_docs: true,
                extract_code: false,
                fail_on_doc_warnings: false,
                fail_on_extraction_warnings: false,
                include_private_docs: false,
            },
            publications: vec![requests[0].clone()],
        };
        assert!(matches!(
            build_doc_plans(
                &fixture.snapshots,
                submission.session.request().id,
                snapshot.id,
                &snapshot.workspace_root,
                &doc_request
            ),
            Err(DocBuildError::Binding(
                "package or manifest build provenance mismatch"
            ))
        ));

        // Doc4's raw build hashes and native producer fingerprints remain distinct.
        // Select an explicit empty test root; both native dependency publications
        // and their captured sources remain byte-exact original producer outputs.
        use mizar_artifact::{
            manifest::write_manifest_file,
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
        };
        let root_path = fixture.root.join("identity-doc-root");
        fs::create_dir_all(&root_path).unwrap();
        let mut empty = publication.manifests()[0].manifest.clone();
        empty.modules.clear();
        empty.artifact_root = "identity-doc-root".into();
        empty.lockfile_hash = ArtifactHashRef::new(
            ArtifactHashClass::Artifact,
            "mizar-build/parsed-lock-content/v1",
            mizar_artifact::store::SchemaVersion::new(1, 0),
            snapshot.lockfile_hash,
        );
        empty.verifier_config_hash = ArtifactHashRef::new(
            ArtifactHashClass::Interface,
            "mizar-build/package-config-content/v1",
            mizar_artifact::store::SchemaVersion::new(1, 0),
            snapshot.verifier_config_hash,
        );
        empty.provenance.generated_by = "explicit-empty-doc-root-test-fixture".into();
        let root_hash = write_manifest_file(&root_path, &empty)
            .unwrap()
            .artifact_hash;
        let mut root_request = requests[0].clone();
        root_request.artifact_root = root_path;
        root_request.expected_manifest_hash = root_hash;
        doc_request.publications = vec![root_request];
        let plans = build_doc_plans(
            &fixture.snapshots,
            submission.session.request().id,
            snapshot.id,
            &snapshot.workspace_root,
            &doc_request,
        )
        .unwrap();
        let inventory = requests
            .iter()
            .map(|request| {
                (
                    &request.package,
                    request.expected_manifest_hash,
                    request.artifact_root.as_path(),
                )
            })
            .collect::<Vec<_>>();
        let imports = publication
            .source_metadata()
            .iter()
            .flatten()
            .map(|metadata| {
                let identity = metadata.identity().expect("genuine source identity");
                (
                    identity.artifact().module.clone(),
                    identity.publication_hash(),
                    metadata.imports().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let root = &plans[0];
        assert!(root.index.modules.is_empty());
        let inputs = authenticate_doc_imports(
            &fixture.snapshots,
            &snapshot.workspace_root,
            root,
            &inventory,
            &imports,
        )
        .unwrap();
        let identities = publication
            .source_metadata()
            .iter()
            .flatten()
            .map(|metadata| metadata.identity().unwrap())
            .collect::<Vec<_>>();
        let saved = publication
            .manifests()
            .iter()
            .zip(&requests)
            .flat_map(|(read, request)| {
                std::iter::once(request.artifact_root.join(read.path.as_str())).chain(
                    read.manifest
                        .modules
                        .iter()
                        .map(|entry| request.artifact_root.join(&entry.artifact_file)),
                )
            })
            .map(|path| {
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            })
            .collect::<Vec<_>>();
        let first = authenticate_doc_identities(
            &fixture.snapshots,
            &snapshot.workspace_root,
            &inputs,
            &inventory,
            &identities,
        )
        .unwrap();
        assert_eq!(first.identities().len(), 2);
        assert_eq!(inputs.closure_coverage(), DocImportCoverage::Complete);
        assert_eq!(
            first,
            authenticate_doc_identities(
                &fixture.snapshots,
                &snapshot.workspace_root,
                &inputs,
                &inventory,
                &identities.iter().rev().copied().collect::<Vec<_>>()
            )
            .unwrap()
        );
        assert!(
            authenticate_doc_identities(
                &fixture.snapshots,
                &snapshot.workspace_root,
                &inputs,
                &inventory,
                &identities[..1]
            )
            .is_err()
        );
        for identity in first.identities() {
            assert_eq!(
                identity.module_components(),
                [identity.artifact().module.module_path.clone()]
            );
            assert!(
                publication
                    .artifacts()
                    .iter()
                    .any(|artifact| artifact.as_ref() == identity.artifact())
            );
        }
        for (path, bytes) in saved {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
        assert!(!fixture.root.join("identity-doc-output").exists());
    }

    #[test]
    fn source_owner_commitments_keep_exact_supported_framing() {
        use mizar_artifact::registration_summary::{ArtifactHashClass, ArtifactHashRef};
        use mizar_artifact::store::{CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion};
        let digest = mizar_session::hash_text("original captured input");
        for (producer, class, hash_class) in [
            (
                "mizar-build/parsed-lock-content/v1",
                ArtifactHashClass::Artifact,
                HashClass::Artifact,
            ),
            (
                "mizar-build/parsed-lock-content/v2",
                ArtifactHashClass::Artifact,
                HashClass::Artifact,
            ),
            (
                "mizar-build/package-config-content/v1",
                ArtifactHashClass::Interface,
                HashClass::Interface,
            ),
            (
                "mizar-vc/source-theorem-canonical-vc/v1",
                ArtifactHashClass::Interface,
                HashClass::Interface,
            ),
            (
                "mizar-vc/source-theorem-canonical-context/v1",
                ArtifactHashClass::Interface,
                HashClass::Interface,
            ),
            (
                "mizar-vc-dependency-slice-v1",
                ArtifactHashClass::Interface,
                HashClass::Interface,
            ),
        ] {
            let family = "mizar-driver/source-artifact-owner-fingerprint";
            let version = SchemaVersion::new(1, 0);
            let value = CanonicalJson::Object(std::collections::BTreeMap::from([
                ("producer".into(), CanonicalJson::string(producer)),
                (
                    "digest".into(),
                    CanonicalJson::string(digest.to_lower_hex()),
                ),
            ]));
            assert_eq!(
                ArtifactHashRef::source_owner_fingerprint(producer, digest),
                Some(ArtifactHashRef::new(
                    class,
                    family,
                    version,
                    CanonicalHashDomain::new(hash_class, family, version).hash(&value, &[]),
                ))
            );
        }
        for unsupported in [
            "mizar-build/parsed-lock-content/v3",
            "mizar-build/package-config-content/v2",
            "forged",
        ] {
            assert!(ArtifactHashRef::source_owner_fingerprint(unsupported, digest).is_none());
        }
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
    fn mapped_citation_snapshot_requires_original_complete_ancestry_and_files() {
        use mizar_artifact::{
            store::{
                CanonicalJson, artifact_hash_domain, canonical_json_bytes,
                parse_canonical_json_bytes,
            },
            verified_artifact::{
                VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact, VerifiedArtifactReadOptions,
                artifact_hash_excluded_paths, builtin_verified_artifact_json,
                read_builtin_verified_artifact,
            },
        };
        use mizar_ir::{
            identity::{OutputKind, PipelinePhase, WorkUnit},
            publisher::{AllowedWorkUnit, OutputOrigin, PublicationTarget, PublishOutputInput},
            storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion},
        };
        use mizar_proof::policy::VerifierPolicy;

        let fixture = Fixture::new();
        let text = "theorem Base: for X being set holds X = X proof let X be set; thus X = X; end; theorem Main: for Y being set holds Y = Y proof let Y be set; thus Y = Y by Base; end;";
        for module in ["main", "leaf"] {
            fs::write(fixture.root.join(format!("alpha/src/{module}.miz")), text).unwrap();
        }
        let (session, input, owner) = fixture.prepare(1);
        for module in ["main", "leaf"] {
            owner.allow_work_unit(AllowedWorkUnit::new(
                PipelinePhase::new("KernelCheck"),
                OutputKind::new("SourceTheoremReflexivityKernelStatus"),
                WorkUnit::new(mizar_session::module_work_unit_label("alpha", module)),
            ));
        }
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
            .register_source_theorem_vc_generation("source-theorem-local-v1".into(), "vc-v1".into())
            .register_source_theorem_vc_discharge(mizar_vc::discharge::DischargePolicy::default())
            .register_source_theorem_reflexivity_kernel_check(VerifierPolicy::release())
            .register_source_theorem_reflexivity_artifact_commit(
                VerifierPolicy::release(),
                (
                    "proof-witnesses".into(),
                    "main.mizir.json".into(),
                    "artifacts".into(),
                ),
            )
            .unwrap();
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(
                session,
                &fixture.ids,
                &fixture.snapshots,
                input,
                Default::default(),
            )
            .unwrap();
        assert_eq!(snapshot_events(&driver, submission.session.id()).len(), 1);
        let original = fixture
            .resolve(&submission, &owner)
            .expect("genuine mapped publication");
        for kind in [
            "SourceTheoremCore",
            "SourceTheoremPreparation",
            "SourceTheoremVC",
            "SourceTheoremReflexivityKernelStatus",
            "SourceTheoremModuleSummary",
            "SourceTheoremDischarge",
            "FrontendOutput",
        ] {
            for duplicate in [false, true] {
                let mut edited = submission.clone();
                let mut changed = false;
                for result in edited
                    .scheduler_run
                    .as_mut()
                    .unwrap()
                    .phase_results
                    .values_mut()
                    .flatten()
                {
                    if let Some(output) = result
                        .output_refs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new(kind))
                        .cloned()
                    {
                        if duplicate {
                            result.output_refs.push(output);
                        } else {
                            result
                                .output_refs
                                .retain(|output| output.output_kind() != &OutputKind::new(kind));
                        }
                        changed = true;
                        break;
                    }
                }
                assert!(changed);
                assert!(
                    fixture.resolve(&edited, &owner).is_none(),
                    "{kind} duplicate={duplicate}"
                );
            }
        }
        let artifact_output = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremArtifactData"))
            .unwrap();
        let artifact = owner
            .storage()
            .get(
                &owner
                    .storage()
                    .typed_handle::<VerifiedArtifact>(
                        artifact_output,
                        &OutputKind::new("SourceTheoremArtifactData"),
                    )
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(artifact.exports.len(), 2);
        assert_eq!(artifact.obligations.len(), 2);
        assert_eq!(artifact.proof_witnesses.len(), 2);
        let original_value = builtin_verified_artifact_json(&artifact).unwrap();
        let artifact = Arc::new(
            read_builtin_verified_artifact(&original_value, VerifiedArtifactReadOptions::default())
                .unwrap(),
        );
        let parents = artifact_output
            .lineage()
            .parents
            .iter()
            .map(|id| {
                submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|result| &result.output_refs)
                    .find(|output| output.output() == *id)
                    .unwrap()
                    .clone()
            })
            .collect::<Vec<_>>();
        for (projection, mutation) in ["exports", "obligations", "proof_witnesses"]
            .into_iter()
            .flat_map(|projection| {
                ["missing", "extra", "substituted"].map(|mutation| (projection, mutation))
            })
            .chain([("exports", "original")])
        {
            let mut edited_artifact = artifact.as_ref().clone();
            let mut value = original_value.clone();
            let CanonicalJson::Object(fields) = &mut value else {
                unreachable!()
            };
            let CanonicalJson::Array(rows) = fields.get_mut(projection).unwrap() else {
                unreachable!()
            };
            match (projection, mutation) {
                (_, "missing") => {
                    rows.pop().unwrap();
                }
                (_, "extra") => rows.push(rows[0].clone()),
                ("obligations", "substituted") => {
                    let CanonicalJson::Object(row) = &mut rows[0] else {
                        unreachable!()
                    };
                    row.insert("status".into(), CanonicalJson::String("open".into()));
                    row.insert("accepted_witness_obligation_id".into(), CanonicalJson::Null);
                }
                (_, "substituted") => rows[0] = rows[1].clone(),
                (_, "original") => {}
                _ => unreachable!(),
            }
            match (projection, mutation) {
                ("exports", "missing") => {
                    edited_artifact.exports.pop().unwrap();
                }
                ("exports", "extra") => edited_artifact.exports.push(artifact.exports[0].clone()),
                ("exports", "substituted") => {
                    // Keep this replacement internally valid: the retained disk/source association
                    // must still reject an export origin absent from the original source owner.
                    edited_artifact.exports[0].origin_id = "unmapped-owner".into();
                    edited_artifact.refresh_hashes().unwrap();
                    value = builtin_verified_artifact_json(&edited_artifact).unwrap();
                    assert!(
                        read_builtin_verified_artifact(
                            &value,
                            VerifiedArtifactReadOptions::default(),
                        )
                        .is_ok()
                    );
                }
                ("obligations", "missing") => {
                    edited_artifact.obligations.pop().unwrap();
                }
                ("obligations", "extra") => edited_artifact
                    .obligations
                    .push(artifact.obligations[0].clone()),
                ("obligations", "substituted") => {
                    edited_artifact.obligations[0].status =
                        mizar_artifact::verified_artifact::ObligationStatus::Open;
                    edited_artifact.obligations[0].accepted_witness_obligation_id = None;
                }
                ("proof_witnesses", "missing") => {
                    edited_artifact.proof_witnesses.pop().unwrap();
                }
                ("proof_witnesses", "extra") => edited_artifact
                    .proof_witnesses
                    .push(artifact.proof_witnesses[0].clone()),
                ("proof_witnesses", "substituted") => {
                    edited_artifact.proof_witnesses[0] = artifact.proof_witnesses[1].clone()
                }
                (_, "original") => {}
                _ => unreachable!(),
            }
            let snapshot = artifact_output.snapshot();
            let domain =
                artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
            let bytes = canonical_json_bytes(&value);
            let work_unit = WorkUnit::new(format!(
                "{}:coverage:{projection}:{mutation}",
                artifact_output.work_unit().as_str(),
            ));
            owner.allow_work_unit(AllowedWorkUnit::new(
                artifact_output.phase().clone(),
                artifact_output.output_kind().clone(),
                work_unit.clone(),
            ));
            let replacement = super::super::publish_native_output(
                &owner,
                snapshot,
                (
                    artifact_output.phase().clone(),
                    work_unit,
                    artifact_output.output_kind().clone(),
                ),
                |slot, phase, work_unit, output_kind| PublishOutputInput {
                    slot,
                    snapshot,
                    phase,
                    work_unit,
                    output_kind,
                    schema_version: SchemaVersion::new(1),
                    payload: edited_artifact,
                    canonical_payload: Some(
                        domain
                            .hash_input(&value, &artifact_hash_excluded_paths())
                            .into_bytes(),
                    ),
                    storage_payload: Some(bytes),
                    decode: BlobDecoder::new(|bytes| {
                        let value = parse_canonical_json_bytes(bytes, "coverage-control")
                            .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                        read_builtin_verified_artifact(
                            &value,
                            VerifiedArtifactReadOptions::default(),
                        )
                        .map_err(|error| BlobDecodeError::new(error.to_string()))
                    }),
                    parents: parents.clone(),
                    named_input_hashes: artifact_output.lineage().named_input_hashes.clone(),
                    side_tables: IrSideTables::default(),
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                },
            )
            .unwrap()
            .erase();
            let mut edited = submission.clone();
            for result in edited
                .scheduler_run
                .as_mut()
                .unwrap()
                .phase_results
                .values_mut()
                .flatten()
            {
                for output in &mut result.output_refs {
                    if output.output() == artifact_output.output() {
                        *output = replacement.clone();
                    }
                }
            }
            assert_eq!(
                fixture.resolve(&edited, &owner).is_some(),
                mutation == "original",
                "{projection} {mutation}",
            );
        }
        let manifest = &original.manifests()[0].manifest;
        let root = fixture.root.join("alpha").join(&manifest.artifact_root);
        let path = root.join(&manifest.modules[0].artifact_file);
        let bytes = fs::read(&path).unwrap();
        fs::write(&path, b"corrupted").unwrap();
        assert!(fixture.resolve(&submission, &owner).is_none());
        fs::write(&path, bytes).unwrap();
        for witness in &manifest.modules[0].proof_witnesses {
            let path = root.join(&witness.witness_path);
            let bytes = fs::read(&path).unwrap();
            fs::remove_file(&path).unwrap();
            assert!(fixture.resolve(&submission, &owner).is_none());
            fs::write(&path, bytes).unwrap();
        }
        assert!(fixture.resolve(&submission, &owner).is_some());
        owner
            .mark_obsolete(submission.session.captured().snapshot().id)
            .unwrap();
        assert!(fixture.resolve(&submission, &owner).is_none());
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
        assert_eq!(
            published.source_metadata().len(),
            published.artifacts().len()
        );
        for ((metadata, artifact), manifest) in published
            .source_metadata()
            .iter()
            .zip(published.artifacts())
            .zip(published.manifests())
        {
            let metadata = metadata
                .as_ref()
                .expect("genuine source metadata is available");
            assert_eq!(metadata.summary().module, artifact.module);
            assert_eq!(metadata.summary().source_hash, artifact.source_hash);
            let citations = metadata
                .direct_by()
                .expect("original publication syntax capture");
            assert_eq!(citations.module(), &artifact.module);
            assert_eq!(citations.publication_hash(), manifest.artifact_hash);
            assert_eq!(citations.source_hash(), artifact.source_hash);
            assert!(citations.complete());
            assert_eq!(citations.examinations().len(), 1);
            assert!(citations.examinations()[0].proof_range.is_some());
            assert!(citations.examinations()[0].complete);
            assert!(citations.occurrences().is_empty());
            let notation = metadata.notation().expect("complete-empty local notation");
            assert!(notation.declarations().is_empty());
            assert!(notation.operator_events().is_empty());
            assert_eq!(notation.source_hash(), artifact.source_hash);
            let imports = metadata.imports().expect("complete-empty imports");
            assert_eq!(imports.source_id(), notation.source_id());
            assert_eq!(imports.source_hash(), notation.source_hash());
            assert_eq!(imports.visible_surfaces().unwrap().count(), 0);
            let entry = &manifest.manifest.modules[0];
            assert_eq!(
                entry.module_summary_interface_hash.as_ref().unwrap().digest,
                metadata.summary().interface_hash
            );
            assert!(entry.module_summary_file.is_some());
            assert!(entry.module_summary_hash.is_some());
        }
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
        assert!(before.source_metadata().iter().all(Option::is_some));
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
