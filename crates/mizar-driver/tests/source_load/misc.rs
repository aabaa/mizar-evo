use super::*;

#[test]
fn scheduled_workspace_intermediate_consumes_only_its_own_public_symbols() {
    let importer = b"import alpha.leaf;\ntheorem T: a arrange b = a;\n";
    let intermediate = b"import alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n";
    let mut lexical_hashes = Vec::new();
    for ancestor in [
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice(),
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\npublic func Spare: x spare y -> set equals x;\nend;\n".as_slice(),
    ] {
        let fixture = WorkspaceLeafFixture::new(importer, intermediate, Some(ancestor), usize::MAX);
        let mut overlay = complete_leaf_import_overlay();
        overlay.edges.push(ModuleDependencyEdge::new(
            mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
            mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("base")),
            ModuleDependencyKind::ImportSummary,
        ));
        let (request, mut input) = fixture.scheduled_request(overlay, Vec::new(), Vec::new());
        input.worker_count = 4;
        let mut driver = fixture.scheduled_driver();
        let submission = driver
            .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            fixture.submission.session.captured().snapshot().id
        );
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        for module in ["base", "leaf", "main"] {
            let task = fixture.task(TaskKind::Frontend, module);
            assert_eq!(run.phase_results.get(&task.id).unwrap()[0].status, PhaseStatus::Complete, "{module}");
        }
        let base = &run.phase_results[&fixture.task(TaskKind::Frontend, "base").id][0].output_refs[0];
        let leaf = &run.phase_results[&fixture.task(TaskKind::Frontend, "leaf").id][0].output_refs[0];
        let main = &run.phase_results[&fixture.task(TaskKind::Frontend, "main").id][0].output_refs[0];
        let leaf_source = &run.phase_results[&fixture.task(TaskKind::SourceLoad, "leaf").id][0].output_refs[0];
        let main_source = &run.phase_results[&fixture.task(TaskKind::SourceLoad, "main").id][0].output_refs[0];
        let leaf_lineage = fixture.publisher.registry().output_lineage(leaf.output()).unwrap();
        assert_eq!(leaf_lineage.parents.len(), 2);
        assert!(leaf_lineage.parents.contains(&leaf_source.output()));
        assert!(leaf_lineage.parents.contains(&base.output()));
        let main_lineage = fixture.publisher.registry().output_lineage(main.output()).unwrap();
        assert_eq!(main_lineage.parents.len(), 2);
        assert!(main_lineage.parents.contains(&main_source.output()));
        assert!(main_lineage.parents.contains(&leaf.output()));
        assert!(!main_lineage.parents.contains(&base.output()));
        let mut active_hashes = Vec::new();
        for (output, spelling) in [(leaf, "combine"), (main, "arrange")] {
            let typed = fixture.publisher.storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output, &OutputKind::new("FrontendOutput")).unwrap();
            let loaded = fixture.publisher.storage().get(&typed).unwrap();
            assert!(loaded.tokens.tokens().iter().any(|token| {
                token.text.as_ref() == spelling && token.kind == TokenKind::UserSymbol
            }));
            active_hashes.push(loaded.cache_keys.active_lexical_environment.stable_hash());
        }
        lexical_hashes.push((active_hashes[0], active_hashes[1]));
    }
    assert_ne!(lexical_hashes[0].0, lexical_hashes[1].0);
    assert_eq!(lexical_hashes[0].1, lexical_hashes[1].1);
}

#[test]
fn scheduled_artifact_importing_intermediate_exports_only_its_own_symbol() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, LexicalContributionSummary, ModuleLexicalSummary, ModuleSummary,
            SourceRangeSummary, current_schema_version, module_summary_json,
        },
        store::{PublishedArtifactPath, artifact_hash_domain, write_published_artifact},
    };
    use mizar_frontend::lexical_env::{
        ExportRank, ExportedSymbolShape, ModuleId as LexerModuleId, SymbolId, UserSymbolArity,
        UserSymbolKind,
    };

    let mut lexical_hashes = Vec::new();
    for mode in ["base", "unused_public_symbol", "missing_core_artifact"] {
        let mut fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a arrange b = a;\n",
        b"import dep.core;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: combine(a,b) = a;\n",
        None,
        usize::MAX,
    );
        let root = fixture.fixture.root.join("dep-artifacts");
        fs::create_dir_all(&root).unwrap();
        let identity = ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: "core".to_owned(),
            language_edition: "2025".to_owned(),
        };
        let origin = "symbol:combine";
        let shape = ExportedSymbolShape {
            spelling: "combine".to_owned(),
            symbol_id: SymbolId::new(canonical_json_string(&CanonicalJson::array([
                identity.canonical_json().unwrap(),
                CanonicalJson::string(origin),
            ]))),
            source_module: LexerModuleId::new(canonical_json_string(
                &identity.canonical_json().unwrap(),
            )),
            export_rank: ExportRank::new(0),
            kind: UserSymbolKind::Functor,
            arity: UserSymbolArity::exact(2),
            operator: None,
        };
        let mut core = ModuleSummary {
            schema_version: current_schema_version(),
            module: identity,
            source_hash: hash(9),
            interface_hash: hash(0),
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: origin.to_owned(),
                fully_qualified_name: "core.combine".to_owned(),
                namespace_path: vec!["core".to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "functor".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 1,
                },
                rendered_signature: "functor core.combine".to_owned(),
                interface_fingerprint: hash(8),
                proof_status: None,
            }],
            exported_labels: Vec::new(),
            lexical_summary: ModuleLexicalSummary {
                schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
                fingerprint: None,
                contributions: vec![LexicalContributionSummary {
                    kind: "exported-symbol".to_owned(),
                    key: origin.to_owned(),
                    payload: String::from_utf8(shape.canonical_bytes().unwrap()).unwrap(),
                }],
            },
            reexports: Vec::new(),
            dependency_interfaces: Vec::new(),
        };
        if mode == "unused_public_symbol" {
            let origin = "symbol:spare";
            let shape = ExportedSymbolShape {
                spelling: "spare".to_owned(),
                symbol_id: SymbolId::new(canonical_json_string(&CanonicalJson::array([
                    core.module.canonical_json().unwrap(),
                    CanonicalJson::string(origin),
                ]))),
                source_module: LexerModuleId::new(canonical_json_string(
                    &core.module.canonical_json().unwrap(),
                )),
                export_rank: ExportRank::new(1),
                kind: UserSymbolKind::Functor,
                arity: UserSymbolArity::exact(2),
                operator: None,
            };
            core.exported_symbols.push(ExportedSymbolSummary {
                origin_id: origin.to_owned(),
                fully_qualified_name: "core.spare".to_owned(),
                namespace_path: vec!["core".to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "functor".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 2,
                    end_byte: 3,
                },
                rendered_signature: "functor core.spare".to_owned(),
                interface_fingerprint: hash(7),
                proof_status: None,
            });
            core.lexical_summary
                .contributions
                .push(LexicalContributionSummary {
                    kind: "exported-symbol".to_owned(),
                    key: origin.to_owned(),
                    payload: String::from_utf8(shape.canonical_bytes().unwrap()).unwrap(),
                });
        }
        core.refresh_interface_hash().unwrap();
        let mut other = core.clone();
        other.module.module_path = "other".to_owned();
        other.exported_symbols.clear();
        other.lexical_summary.contributions.clear();
        other.refresh_interface_hash().unwrap();
        let mut references = Vec::new();
        let mut captured = Vec::new();
        for summary in [core, other] {
            let module = summary.module.module_path.as_str();
            let artifact = format!("dep/{module}.summary.json");
            let content_hash = write_published_artifact(
                &root,
                &PublishedArtifactPath::new(&artifact).unwrap(),
                &module_summary_json(&summary).unwrap(),
                &artifact_hash_domain(
                    mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
                    summary.schema_version,
                ),
                &[],
            )
            .unwrap()
            .artifact_hash;
            references.push(DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new(module),
                ),
                artifact: artifact.clone(),
                content_hash,
            });
            captured.push(DependencyArtifactRef::new(artifact, content_hash));
        }
        references.sort_by(|left, right| {
            right
                .content_hash
                .as_bytes()
                .cmp(left.content_hash.as_bytes())
        });
        let indexed = vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            references,
        )];
        let (request, input) = fixture.scheduled_request(
            complete_leaf_import_overlay(),
            captured.clone(),
            indexed.clone(),
        );
        let baseline = CompilerDriver::new(source_registry())
            .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        fixture
            .publisher
            .register_current_snapshot(baseline.session.captured().snapshot().id);
        fixture.submission = baseline;
        if mode == "missing_core_artifact" {
            fs::remove_file(root.join("dep/core.summary.json")).unwrap();
        }
        let (request, mut input) =
            fixture.scheduled_request(complete_leaf_import_overlay(), captured, indexed);
        input.worker_count = 4;
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![(PackageId::new("dep"), root)]);
        let mut driver = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(fixture.publisher.clone());
        let submission = driver
            .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            fixture.submission.session.captured().snapshot().id
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        if mode == "missing_core_artifact" {
            let leaf_result = &run.phase_results[&fixture.task(TaskKind::Frontend, "leaf").id][0];
            assert_ne!(leaf_result.status, PhaseStatus::Complete);
            assert!(leaf_result.output_refs.is_empty());
            assert!(
                run.phase_results
                    .get(&fixture.task(TaskKind::Frontend, "main").id)
                    .is_none_or(|results| results
                        .iter()
                        .all(|result| result.output_refs.is_empty()))
            );
            continue;
        }
        for module in ["leaf", "main"] {
            let result = &run.phase_results[&fixture.task(TaskKind::Frontend, module).id][0];
            assert_eq!(
                result.status,
                PhaseStatus::Complete,
                "{module}: {:?}",
                result.diagnostics
            );
        }
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let leaf =
            &run.phase_results[&fixture.task(TaskKind::Frontend, "leaf").id][0].output_refs[0];
        let main =
            &run.phase_results[&fixture.task(TaskKind::Frontend, "main").id][0].output_refs[0];
        let leaf_source =
            &run.phase_results[&fixture.task(TaskKind::SourceLoad, "leaf").id][0].output_refs[0];
        let main_source =
            &run.phase_results[&fixture.task(TaskKind::SourceLoad, "main").id][0].output_refs[0];
        let leaf_lineage = fixture
            .publisher
            .registry()
            .output_lineage(leaf.output())
            .unwrap();
        assert_eq!(leaf_lineage.parents, vec![leaf_source.output()]);
        let main_lineage = fixture
            .publisher
            .registry()
            .output_lineage(main.output())
            .unwrap();
        assert_eq!(main_lineage.parents.len(), 2);
        assert!(main_lineage.parents.contains(&main_source.output()));
        assert!(main_lineage.parents.contains(&leaf.output()));
        let hashes = fixture
            .submission
            .module_index
            .as_ref()
            .unwrap()
            .dependency_summaries
            .iter()
            .map(|reference| reference.content_hash)
            .collect::<Vec<_>>();
        assert_eq!(hashes.len(), 2);
        let mut sorted = hashes.clone();
        sorted.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        assert_ne!(hashes, sorted);
        assert_eq!(
            leaf_lineage
                .named_input_hashes
                .iter()
                .map(|input| input.name.as_str())
                .collect::<Vec<_>>(),
            [
                "active-lexical-environment",
                "dependency.0",
                "dependency.1",
                "tokens"
            ]
        );
        assert_eq!(leaf_lineage.named_input_hashes[1].digest, sorted[0]);
        assert_eq!(leaf_lineage.named_input_hashes[2].digest, sorted[1]);
        let mut active_hashes = Vec::new();
        for (output, spelling) in [(leaf, "combine"), (main, "arrange")] {
            let typed = fixture
                .publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded = fixture.publisher.storage().get(&typed).unwrap();
            assert!(loaded.tokens.tokens().iter().any(|token| {
                token.text.as_ref() == spelling && token.kind == TokenKind::UserSymbol
            }));
            active_hashes.push(loaded.cache_keys.active_lexical_environment.stable_hash());
        }
        lexical_hashes.push((active_hashes[0], active_hashes[1]));
    }
    assert_ne!(lexical_hashes[0].0, lexical_hashes[1].0);
    assert_eq!(lexical_hashes[0].1, lexical_hashes[1].1);
}

#[test]
fn workspace_leaf_is_not_injected_by_ordinary_no_provider_submit() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ndefinition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (request, input) = fixture.scheduled_request(
        ModuleDependencyOverlay::complete(Vec::new()),
        Vec::new(),
        Vec::new(),
    );
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    let frontend = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| {
            task.kind == TaskKind::Frontend
                && matches!(&task.unit,
            WorkUnit::Module { module } if module.path.as_str() == "main")
        })
        .unwrap();
    let result = &submission
        .scheduler_run
        .as_ref()
        .unwrap()
        .phase_results
        .get(&frontend.id)
        .unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
}

#[test]
fn scheduled_complete_import_overlay_hands_off_real_leaf_output() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        None,
        usize::MAX,
    );
    let (request, mut input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    input.worker_count = 4;
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByMissingPhaseServices
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    let graph = submission.task_graph.as_ref().unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let leaf_frontend = graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend
        && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == "leaf")).unwrap();
    let importer_frontend = graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend
        && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == "main")).unwrap();
    let importer_source = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::SourceLoad && task.unit == importer_frontend.unit)
        .unwrap();
    assert!(importer_frontend.dependencies.contains(&leaf_frontend.id));
    for task in [leaf_frontend, importer_frontend] {
        assert_eq!(
            run.task_states
                .iter()
                .find(|state| state.task_id == task.id)
                .unwrap()
                .state,
            TaskState::Completed
        );
        assert_eq!(
            run.phase_results.get(&task.id).unwrap()[0].status,
            PhaseStatus::Complete
        );
        assert_eq!(
            run.phase_results.get(&task.id).unwrap()[0]
                .output_refs
                .len(),
            1
        );
    }
    let own = &run.phase_results.get(&importer_source.id).unwrap()[0].output_refs[0];
    let leaf = &run.phase_results.get(&leaf_frontend.id).unwrap()[0].output_refs[0];
    let output = &run.phase_results.get(&importer_frontend.id).unwrap()[0].output_refs[0];
    let lineage = fixture
        .publisher
        .registry()
        .output_lineage(output.output())
        .unwrap();
    assert_eq!(lineage.parents.len(), 2);
    assert!(lineage.parents.contains(&own.output()));
    assert!(lineage.parents.contains(&leaf.output()));
    let typed = fixture
        .publisher
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            output,
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    let loaded = fixture.publisher.storage().get(&typed).unwrap();
    assert!(
        loaded.tokens.tokens().iter().any(|token| {
            token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
        })
    );
}

#[test]
fn scheduled_import_overlay_cannot_invent_a_source_import() {
    let fixture = WorkspaceLeafFixture::new(
        b"definition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (request, input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    let run = submission.scheduler_run.as_ref().unwrap();
    let leaf = fixture.task(TaskKind::Frontend, "leaf");
    let importer = fixture.task(TaskKind::Frontend, "main");
    assert_eq!(
        run.phase_results.get(&leaf.id).unwrap()[0].status,
        PhaseStatus::Complete
    );
    let result = &run.phase_results.get(&importer.id).unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
}

#[test]
fn scheduled_import_overlay_does_not_synthesize_a_cached_leaf_output() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        None,
        usize::MAX,
    );
    let (request, mut input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    input.cache_policy = CacheSchedulingPolicy::Enabled;
    input.cache_decisions = CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
        fixture.task(TaskKind::Frontend, "leaf").id.clone(),
        CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
            vec![CacheOutputRef::new("leaf", "cached")],
            Vec::new(),
        )),
    )]);
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    let run = submission.scheduler_run.as_ref().unwrap();
    let leaf = fixture.task(TaskKind::Frontend, "leaf");
    let importer = fixture.task(TaskKind::Frontend, "main");
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == leaf.id)
            .unwrap()
            .state,
        TaskState::CacheHit
    );
    assert!(!run.phase_results.contains_key(&leaf.id));
    assert_ne!(
        run.task_states
            .iter()
            .find(|state| state.task_id == importer.id)
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert!(
        run.phase_results
            .get(&importer.id)
            .is_none_or(|results| { results.iter().all(|result| result.output_refs.is_empty()) })
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
}

#[test]
fn scheduled_import_overlay_keeps_supplied_dispatch_provider_authoritative() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        None,
        usize::MAX,
    );
    let (request, mut input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    input.phase_dispatch_inputs = Some(Box::new(SuppliedInput::None));
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByPhaseDispatchGap
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    assert!(
        submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .is_empty()
    );
}

#[test]
fn scheduled_source_frontend_prefix_publishes_real_parent_and_blocks_later_services() {
    for (workers, cached_later) in [(1, false), (4, false), (1, true)] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let second = if workers == 4 {
            fs::write(
                fixture.root.join("alpha/src/other.miz"),
                b"definition\nend;\n",
            )
            .unwrap();
            let package_root = fixture.root.join("alpha");
            ids.next_source_id(snapshot_id(0x52)).unwrap();
            let loaded = mizar_session::SourceLoader::load(
                &DiskSourceLoader::new(&package_root),
                snapshot_id(0x52),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new("other"),
                    normalized_path: normalize_path(&package_root, Path::new("src/other.miz"))
                        .unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: PathBuf::from("src/other.miz"),
                    },
                },
                &ids,
            )
            .unwrap();
            Some(SourceVersion {
                source_id: loaded.source_id,
                package_id: loaded.package_id,
                module_path: loaded.module_path,
                normalized_path: loaded.normalized_path,
                source_hash: loaded.source_hash,
                edition: loaded.edition,
                origin: loaded.origin,
            })
        } else {
            None
        };
        let make_request = || {
            let (mut request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
            if let Some(version) = &second {
                request.source_inputs.versions.push(version.clone());
                input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                    package_id: PackageId::new("alpha"),
                    files: vec![
                        WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                        WorkspaceSourceFile::new("src/other.miz", "other.miz"),
                    ],
                }]);
            }
            (request, input)
        };
        let (request, input) = make_request();
        let baseline = CompilerDriver::new(source_registry())
            .submit(request, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            baseline.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let snapshot = baseline.session.captured().snapshot().id;
        let source_tasks = baseline
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .filter(|task| task.kind == TaskKind::SourceLoad)
            .collect::<Vec<_>>();
        assert_eq!(source_tasks.len(), if workers == 4 { 2 } else { 1 });
        let publisher = publisher(snapshot, usize::MAX, &source_tasks[0].unit);
        for source in &source_tasks {
            let WorkUnit::Module { module } = &source.unit else {
                unreachable!()
            };
            publisher.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                )),
            ));
            allow_frontend(&publisher, &source.unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher.clone());
        let (request, mut input) = make_request();
        input.worker_count = workers;
        if cached_later {
            input.cache_policy = CacheSchedulingPolicy::Enabled;
            input.cache_decisions = CacheSchedulingPlan::new(
                baseline
                    .task_graph
                    .as_ref()
                    .unwrap()
                    .tasks()
                    .iter()
                    .filter(|task| !matches!(task.kind, TaskKind::SourceLoad | TaskKind::Frontend))
                    .map(|task| {
                        CacheTaskDecision::new(
                            task.id.clone(),
                            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                                vec![CacheOutputRef::new(task.id.as_str(), "cached")],
                                Vec::new(),
                            )),
                        )
                    })
                    .collect(),
            );
        }
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(submission.session.captured().snapshot().id, snapshot);
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked)
        );
        assert!(
            submission
                .missing_services
                .iter()
                .any(|missing| { missing.phase == PipelinePhase::ModuleResolve })
        );
        assert!(
            driver
                .events(submission.session.id())
                .events()
                .iter()
                .any(|event| {
                    matches!(
                        event.kind,
                        BuildEventKind::PhaseServiceGap {
                            phase: PipelinePhase::ModuleResolve,
                            ..
                        }
                    )
                })
        );
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        if cached_later {
            for task in graph.tasks().iter().filter(|task| {
                submission
                    .missing_services
                    .iter()
                    .any(|missing| task.phases.contains(&missing.phase))
            }) {
                assert_ne!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == task.id)
                        .unwrap()
                        .state,
                    TaskState::CacheHit
                );
            }
        }
        for source in graph
            .tasks()
            .iter()
            .filter(|task| task.kind == TaskKind::SourceLoad)
        {
            let frontend = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::Frontend && task.unit == source.unit)
                .unwrap();
            let resolve = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::ModuleResolve && task.unit == source.unit)
                .unwrap();
            for task in [source, frontend] {
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == task.id)
                        .unwrap()
                        .state,
                    TaskState::Completed
                );
                assert_eq!(
                    run.phase_results.get(&task.id).unwrap()[0].status,
                    PhaseStatus::Complete
                );
                assert_eq!(
                    run.phase_results.get(&task.id).unwrap()[0]
                        .output_refs
                        .len(),
                    1
                );
            }
            assert_eq!(
                run.task_states
                    .iter()
                    .find(|state| state.task_id == resolve.id)
                    .unwrap()
                    .state,
                TaskState::Blocked
            );
            let source_output = &run.phase_results.get(&source.id).unwrap()[0].output_refs[0];
            let frontend_output = &run.phase_results.get(&frontend.id).unwrap()[0].output_refs[0];
            assert_eq!(source_output.output_kind(), &OutputKind::new("SourceUnit"));
            assert_eq!(
                frontend_output.output_kind(),
                &OutputKind::new("FrontendOutput")
            );
            assert_eq!(source_output.snapshot(), snapshot);
            assert_eq!(frontend_output.snapshot(), snapshot);
            assert_eq!(
                publisher
                    .registry()
                    .output_lineage(frontend_output.output())
                    .unwrap()
                    .parents,
                vec![source_output.output()]
            );
            assert!(publisher.storage().validate_handle(source_output).is_ok());
            assert!(publisher.storage().validate_handle(frontend_output).is_ok());
        }
    }
}

#[test]
fn scheduled_frontend_import_failure_retains_diagnostics_and_blocks_dependents() {
    let fixture = Fixture::new(b"import mml.no_such;\ndefinition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let baseline = fixture.submit(&ids, &snapshots);
    let snapshot = baseline.session.captured().snapshot().id;
    let source_unit = &source_task(&baseline).unit;
    let publisher = publisher(snapshot, usize::MAX, source_unit);
    allow_frontend(&publisher, source_unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(Vec::new());
    let mut driver = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
    let (request, input) = fixture.submit_request(Vec::new(), Vec::new());
    let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
    assert_eq!(submission.session.captured().snapshot().id, snapshot);
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::SchedulerValidated
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    let graph = submission.task_graph.as_ref().unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let source = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::SourceLoad)
        .unwrap();
    let frontend = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let resolve = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::ModuleResolve)
        .unwrap();
    assert_eq!(
        run.phase_results.get(&source.id).unwrap()[0].status,
        PhaseStatus::Complete
    );
    assert_eq!(
        run.phase_results.get(&source.id).unwrap()[0]
            .output_refs
            .len(),
        1
    );
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == frontend.id)
            .unwrap()
            .state,
        TaskState::Failed
    );
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == resolve.id)
            .unwrap()
            .state,
        TaskState::Blocked
    );
    let result = &run.phase_results.get(&frontend.id).unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    let codes = result
        .diagnostics
        .iter()
        .flat_map(|batch| batch.drafts())
        .map(|draft| draft.code().number())
        .collect::<Vec<_>>();
    assert!(codes.contains(&22), "missing E0022: {codes:?}");
    assert!(codes.contains(&220), "missing E0220: {codes:?}");
}

#[test]
fn scheduled_prefix_requires_current_publisher_and_both_builtin_services() {
    for mode in [
        "missing_publisher",
        "obsolete_publisher",
        "missing_frontend",
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let baseline = fixture.submit(&ids, &snapshots);
        let snapshot = baseline.session.captured().snapshot().id;
        let publisher = publisher(snapshot, usize::MAX, &source_task(&baseline).unit);
        allow_frontend(&publisher, &source_task(&baseline).unit);
        if mode == "obsolete_publisher" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        if mode != "missing_frontend" {
            builder.register_frontend(Vec::new());
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if mode != "missing_publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let (request, input) = fixture.submit_request(Vec::new(), Vec::new());
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            snapshot,
            "{mode}"
        );
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices,
            "{mode}"
        );
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked),
            "{mode}"
        );
        assert!(submission.scheduler_run.is_none(), "{mode}");
    }
}

#[test]
fn scheduled_prefix_respects_source_and_frontend_publication_rights() {
    for deny_source in [true, false] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let baseline = fixture.submit(&ids, &snapshots);
        let snapshot = baseline.session.captured().snapshot().id;
        let publisher = if deny_source {
            let publisher = std::sync::Arc::new(PhaseOutputPublisher::new(
                std::sync::Arc::new(IrStorageService::new()),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(snapshot);
            publisher
        } else {
            publisher(snapshot, usize::MAX, &source_task(&baseline).unit)
        };
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
        let (request, input) = fixture.submit_request(Vec::new(), Vec::new());
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(submission.session.captured().snapshot().id, snapshot);
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked)
        );
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let source = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::SourceLoad)
            .unwrap();
        let frontend = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend)
            .unwrap();
        let blocked = if deny_source { source } else { frontend };
        assert_eq!(
            run.task_states
                .iter()
                .find(|state| state.task_id == blocked.id)
                .unwrap()
                .state,
            TaskState::Blocked
        );
        let blocked_result = &run.phase_results.get(&blocked.id).unwrap()[0];
        assert_eq!(blocked_result.status, PhaseStatus::Blocking);
        assert!(blocked_result.output_refs.is_empty());
        if deny_source {
            assert!(!run.phase_results.contains_key(&frontend.id));
        } else {
            assert_eq!(
                run.phase_results.get(&source.id).unwrap()[0]
                    .output_refs
                    .len(),
                1
            );
        }
    }
}

#[test]
fn scheduled_prefix_never_replaces_supplied_dispatch_inputs() {
    for mode in [
        SuppliedInput::None,
        SuppliedInput::Error,
        SuppliedInput::ForeignSnapshot,
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let baseline = fixture.submit(&ids, &snapshots);
        let snapshot = baseline.session.captured().snapshot().id;
        let publisher = publisher(snapshot, usize::MAX, &source_task(&baseline).unit);
        allow_frontend(&publisher, &source_task(&baseline).unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
        let (request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
        input.phase_dispatch_inputs = Some(Box::new(mode));
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(submission.session.captured().snapshot().id, snapshot);
        let (expected_state, expected_status, expected_task_state) = match mode {
            SuppliedInput::None => (
                BuildSessionOutcome::Blocked,
                DriverSubmissionStatus::BlockedByPhaseDispatchGap,
                TaskState::Blocked,
            ),
            SuppliedInput::Error | SuppliedInput::ForeignSnapshot => (
                BuildSessionOutcome::Failed,
                DriverSubmissionStatus::SchedulerValidated,
                TaskState::Failed,
            ),
        };
        assert_eq!(submission.status, expected_status, "{mode:?}");
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(expected_state),
            "{mode:?}"
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let source = source_task(&submission);
        assert_eq!(
            run.task_states
                .iter()
                .find(|state| state.task_id == source.id)
                .unwrap()
                .state,
            expected_task_state,
            "{mode:?}"
        );
        assert!(
            run.phase_results.is_empty(),
            "{mode:?} must not fall back to captured source"
        );
    }
}

#[test]
fn scheduled_frontend_rejects_cache_hit_without_retained_source_parent() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let baseline = fixture.submit(&ids, &snapshots);
    let snapshot = baseline.session.captured().snapshot().id;
    let source = source_task(&baseline);
    let publisher = publisher(snapshot, usize::MAX, &source.unit);
    allow_frontend(&publisher, &source.unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(Vec::new());
    let mut driver = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
    let (request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
    input.cache_policy = CacheSchedulingPolicy::Enabled;
    input.cache_decisions = CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
        source.id.clone(),
        CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
            vec![CacheOutputRef::new("source", "cached")],
            Vec::new(),
        )),
    )]);
    let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
    assert_eq!(submission.session.captured().snapshot().id, snapshot);
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByPhaseDispatchGap
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    let graph = submission.task_graph.as_ref().unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let frontend = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == source.id)
            .unwrap()
            .state,
        TaskState::CacheHit
    );
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == frontend.id)
            .unwrap()
            .state,
        TaskState::Blocked
    );
    assert!(!run.phase_results.contains_key(&source.id));
    assert!(!run.phase_results.contains_key(&frontend.id));
    assert!(
        submission
            .dispatch_gap_phases
            .contains(&PipelinePhase::Frontend)
    );
}

#[test]
fn scheduled_frontend_uses_sorted_captured_dependency_summary_hashes() {
    let fixture = Fixture::new(b"import dep.core;\ndefinition\nend;\n");
    let references = [("core", hash(0xf0)), ("other", hash(0x10))];
    let captured = references
        .iter()
        .map(|(path, digest)| {
            DependencyArtifactRef::new(format!("dep/{path}.summary.json"), *digest)
        })
        .collect::<Vec<_>>();
    let indexed = vec![DependencyArtifactIndex::new(
        PackageId::new("dep"),
        Vec::new(),
        references
            .iter()
            .map(|(path, digest)| DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new(*path),
                ),
                artifact: format!("dep/{path}.summary.json"),
                content_hash: *digest,
            })
            .collect(),
    )];
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let baseline =
        fixture.submit_with_dependencies(&ids, &snapshots, captured.clone(), indexed.clone());
    assert_eq!(
        baseline
            .module_index
            .as_ref()
            .unwrap()
            .dependency_summaries
            .iter()
            .map(|summary| summary.content_hash)
            .collect::<Vec<_>>(),
        vec![hash(0xf0), hash(0x10)]
    );
    let snapshot = baseline.session.captured().snapshot().id;
    let publisher = publisher(snapshot, usize::MAX, &source_task(&baseline).unit);
    allow_frontend(&publisher, &source_task(&baseline).unit);
    let artifact_root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![(PackageId::new("dep"), artifact_root)]);
    let mut driver = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
    let (request, input) = fixture.submit_request(captured, indexed);
    let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
    assert_eq!(submission.session.captured().snapshot().id, snapshot);
    let graph = submission.task_graph.as_ref().unwrap();
    let frontend = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let result = &run.phase_results.get(&frontend.id).unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert!(
        result
            .diagnostics
            .iter()
            .flat_map(|batch| batch.drafts())
            .any(|draft| draft.code().number() == 23)
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
}

#[test]
fn sealed_frontend_output_feeds_source_lexical_contributions() {
    let text = b"definition\n\
        let x, y be set;\n\
        private pred Hidden: x hidden y means thesis;\n\
        public func Pair: |. x .| -> set equals x;\n\
        func Infix: x combine y -> set equals x;\n\
        mode Visible: Carrier is set;\nend;\n";
    let mut previous = None;
    for threshold in [usize::MAX, 3] {
        let fixture = Fixture::new(text);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, threshold);
        assert_eq!(source.status, PhaseStatus::Complete);
        let frontend = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(frontend.status, PhaseStatus::Complete);
        assert!(frontend.diagnostics.is_empty());
        let output = frontend
            .output_refs
            .first()
            .expect("sealed frontend output");
        assert_eq!(frontend.output_refs.len(), 1);
        assert_eq!(
            matches!(output.placement(), StoragePlacement::Resident),
            threshold == usize::MAX
        );
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = publisher.storage().get(&typed).unwrap();
        assert_eq!(loaded.source.source_id, fixture.version.source_id);
        assert_eq!(loaded.source.source_hash, fixture.version.source_hash);
        assert!(loaded.diagnostics.is_empty());
        let ast = loaded.ast.as_ref().expect("clean frontend AST");
        let module = ResolverModuleId::new(
            loaded.source.package_id.clone(),
            loaded.source.module_path.clone(),
        );
        let package = submission
            .build_plan
            .as_ref()
            .unwrap()
            .packages
            .iter()
            .find(|package| package.package_id == *module.package())
            .expect("planned source package");
        let identity = ModuleSummaryIdentity {
            package_id: loaded.source.package_id.as_str().to_owned(),
            package_version: Some(package.version.to_string()),
            lockfile_identity: None,
            module_path: loaded.source.module_path.as_str().to_owned(),
            language_edition: loaded.source.edition.as_str().to_owned(),
        };
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collection = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(collection.diagnostics().is_empty());
        let pairs = collection
            .pair_frontend_lexical_declarations(&loaded)
            .expect("sealed source lexical pairs");
        assert_eq!(
            pairs
                .iter()
                .map(|(_, local)| (local.spelling.as_str(), local.export_rank.get()))
                .collect::<Vec<_>>(),
            [("|.", 1), (".|", 2), ("combine", 3), ("Carrier", 4)]
        );
        assert_eq!(loaded.tokens.local_declarations().user_symbols.len(), 5);
        let contributions = collection
            .export_frontend_lexical_contributions(&loaded, &identity)
            .expect("sealed source lexical contributions");
        assert_eq!(contributions.len(), pairs.len());
        let identity_json = identity.canonical_json().unwrap();
        let source_module = canonical_json_string(&identity_json);
        for ((entry, _), contribution) in pairs.iter().zip(&contributions) {
            assert_eq!(contribution.key, entry.symbol().local().as_str());
            let SourceAnchor::Range(origin) = entry.origin().anchor() else {
                panic!("source lexical origin must be a range");
            };
            assert_eq!(origin.source_id, loaded.source.source_id);
            let shape = mizar_frontend::lexical_env::ExportedSymbolShape::from_canonical_bytes(
                contribution.payload.as_bytes(),
            )
            .unwrap();
            assert_eq!(shape.source_module.as_str(), source_module);
            assert_eq!(
                shape.symbol_id.as_str(),
                canonical_json_string(&CanonicalJson::array([
                    identity_json.clone(),
                    CanonicalJson::string(entry.symbol().local().as_str()),
                ]))
            );
        }
        if let Some(previous) = &previous {
            assert_eq!(&contributions, previous);
        }
        previous = Some(contributions);
    }
}

#[test]
fn ordinary_submission_stays_blocked_when_only_source_load_is_registered() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    assert!(!submission.missing_services.is_empty());
    assert!(
        submission
            .task_graph
            .unwrap()
            .tasks()
            .iter()
            .any(|task| task.kind == TaskKind::SourceLoad)
    );
}

#[test]
fn prepared_native_prefix_uses_frozen_disk_and_open_sources_with_real_storage_and_no_keys() {
    use mizar_frontend::{orchestration::Frontend, source::FrontendSourceLoader};
    use mizar_ir::dispatch_input::SealedParentOutputHandle;
    let raw = "\u{feff}definition\r\nend;\r\n";
    for open in [false, true] {
        for (threshold, workers, order) in [
            (usize::MAX, 1, CompletionOrder::Canonical),
            (0, 4, CompletionOrder::Reverse),
        ] {
            let fixture = Fixture::new(raw.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
            request.source_inputs.versions.clear();
            input.worker_count = workers;
            input.completion_order = order;
            let descriptor = SourceInput {
                package_id: fixture.version.package_id.clone(),
                module_path: fixture.version.module_path.clone(),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: fixture.version.edition.clone(),
                origin: if open {
                    SourceOriginInput::OpenBuffer {
                        uri: format!(
                            "file://{}",
                            fixture.root.join("alpha/src/main.miz").display()
                        ),
                        expected_version: 7,
                        actual_version: 7,
                        text: std::sync::Arc::from(raw),
                    }
                } else {
                    SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    }
                },
            };
            if open {
                fixture.write(&[0xff]);
            }
            let session = request
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        descriptor.clone(),
                    )],
                )
                .unwrap();
            let snapshot = session.captured().snapshot().id;
            let captured_id = session.captured().snapshot().source_versions[0].source_id;
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("main"),
                ),
            };
            let publisher = publisher(snapshot, threshold, &unit);
            allow_frontend(&publisher, &unit);
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver = CompilerDriver::new(builder.build().unwrap())
                .with_output_publisher(publisher.clone());
            fixture.write(b"changed disk content");
            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            let submission = driver
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Blocked)
            );
            let run = submission.scheduler_run.as_ref().unwrap();
            let handles = run
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .collect::<Vec<_>>();
            assert_eq!(handles.len(), 2);
            let source_ref = *handles
                .iter()
                .find(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap();
            let source_handle = publisher
                .storage()
                .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                .unwrap();
            let source = publisher.storage().get(&source_handle).unwrap();
            assert_eq!(source.source_id, captured_id);
            assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
            assert_eq!(
                source.source_hash,
                mizar_session::hash_text("definition\nend;\n")
            );
            assert!(source.loading_map.is_some());
            assert_eq!(
                source.origin,
                if open {
                    SourceOrigin::OpenBuffer { version: 7 }
                } else {
                    SourceOrigin::Disk
                }
            );
            let output_ref = *handles
                .iter()
                .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
                .unwrap();
            let output_handle = publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output_ref,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let output = publisher.storage().get(&output_handle).unwrap();
            let inputs = SourceLoadInputs {
                snapshot: submission.session.captured().snapshot(),
                captured: Some(submission.session.captured()),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let expected = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(source.as_ref().clone())
            .unwrap();
            assert_eq!(*output, expected);
            assert_eq!(output.source, *source);
            assert!(output.diagnostics.is_empty());
            assert_eq!(output_ref.lineage().parents, vec![source_ref.output()]);
            for handle in handles {
                assert_eq!(
                    matches!(handle.placement(), StoragePlacement::Blob { .. }),
                    threshold == 0
                );
                assert!(
                    publisher
                        .validate_published_output(snapshot, handle)
                        .is_ok()
                );
                assert_eq!(
                    publisher.validate_current_output(snapshot, handle).is_ok(),
                    !open
                );
            }
            let source_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    SourceUnitCacheKey::from_source(&source).stable_hash(),
                    vec![],
                ),
            );
            for source_summary in [false, true] {
                let mut forged = inputs.snapshot.clone();
                if source_summary {
                    forged.source_versions[0].source_hash = hash(91);
                } else {
                    forged.verifier_config_hash = hash(92);
                }
                let rejected = driver
                    .registry()
                    .execute_phase_with_resources(
                        PipelinePhase::SourceLoad,
                        source_input.clone(),
                        PhaseExecutionResources {
                            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                                DiagnosticPhase::SourceLoad,
                                snapshot,
                                "frontend.source_load",
                            ))),
                            output_publisher: Some(publisher.clone()),
                            source_load: Some(SourceLoadInputs {
                                snapshot: &forged,
                                ..inputs
                            }),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap()
                    .result;
                assert_eq!(rejected.status, PhaseStatus::Blocking);
                assert!(rejected.output_refs.is_empty());
                assert!(rejected.diagnostics.is_empty());
            }
            let frontend_input = PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::new(
                    snapshot,
                    SourceUnitCacheKey::from_source(&source).stable_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            &publisher,
                            snapshot,
                            source_ref.clone(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::SourceLoad, &source_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: "current disk source maps require a real source load".into()
                }
            );
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::Frontend, &frontend_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: "frontend inputs require a live source and dependency run".into()
                }
            );
        }
    }
}

#[test]
fn native_scheduled_diagnostics_cancellation_generated_and_denied_publication_remain_real() {
    use mizar_frontend::{
        orchestration::{DiagnosticCode, DiagnosticLocation, Frontend},
        source::{FrontendSourceLoader, SourceUnitLoader, SourceUnitRequest},
    };
    for mode in ["diagnostic", "cancel", "generated", "denied"] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: fixture.version.package_id.clone(),
            module_path: fixture.version.module_path.clone(),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: fixture.version.edition.clone(),
            origin: if mode == "generated" {
                SourceOriginInput::Generated {
                    generator: mizar_session::GeneratedSourceKind::new("native-test"),
                    text: std::sync::Arc::from("definition\nend;\n"),
                    anchor: None,
                }
            } else {
                SourceOriginInput::OpenBuffer {
                    uri: format!(
                        "file://{}",
                        fixture.root.join("alpha/src/main.miz").display()
                    ),
                    expected_version: 7,
                    actual_version: 7,
                    text: std::sync::Arc::from(if mode == "diagnostic" {
                        "end"
                    } else {
                        "definition\nend;\n"
                    }),
                }
            },
        };
        let session = request
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    descriptor.clone(),
                )],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let publisher = publisher(snapshot, 0, &unit);
        if mode != "denied" {
            allow_frontend(&publisher, &unit);
        }
        if mode == "cancel" {
            input.cancellation.supersede_snapshot(snapshot);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher.clone());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let results = run.phase_results.values().flatten().collect::<Vec<_>>();
        if mode == "diagnostic" {
            let drafts = results
                .iter()
                .flat_map(|result| &result.diagnostics)
                .flat_map(|batch| batch.drafts())
                .collect::<Vec<_>>();
            assert!(!drafts.is_empty());
            assert!(
                results
                    .iter()
                    .any(|result| result.status == PhaseStatus::Fatal)
            );
            let inputs = SourceLoadInputs {
                snapshot: submission.session.captured().snapshot(),
                captured: Some(submission.session.captured()),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let native_loader =
                FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha")));
            let actual = native_loader
                .load_source_unit(
                    SourceUnitRequest {
                        snapshot,
                        input: descriptor,
                    },
                    &InMemorySessionIdAllocator::new(),
                )
                .unwrap();
            assert_eq!(
                actual.source_id,
                inputs.snapshot.source_versions[0].source_id
            );
            let oracle = Frontend::new(
                native_loader,
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(actual)
            .unwrap();
            assert_eq!(drafts.len(), oracle.diagnostics.len());
            for (draft, diagnostic) in drafts.iter().zip(&oracle.diagnostics) {
                let (number, class, local_code) = match &diagnostic.code {
                    DiagnosticCode::Lexing(
                        mizar_frontend::lexing::LexingDiagnosticKind::ScopeSkeleton(
                            mizar_frontend::lexing::ScopeSkeletonDiagnosticCode::UnmatchedEnd,
                        ),
                    ) => (33, "scope_skeleton", "lexing.scope_skeleton.unmatched_end"),
                    DiagnosticCode::Syntax(key) if key.as_ref() == "unrecoverable_input" => {
                        (52, "syntax", "syntax.unrecoverable_input")
                    }
                    code => panic!("unexpected producer diagnostic {code:?}"),
                };
                assert_eq!(
                    draft.code(),
                    mizar_diagnostics::registry::DiagnosticCode::from_parts(
                        mizar_diagnostics::registry::DiagnosticSeverity::Error,
                        number
                    )
                    .unwrap()
                );
                assert_eq!(
                    draft.stable_detail_key(),
                    mizar_diagnostics::registry::DiagnosticRegistry::builtin()
                        .lookup(draft.code())
                        .unwrap()
                        .semantic_name
                );
                assert_eq!(draft.source_snapshot(), snapshot);
                assert_eq!(draft.phase(), DiagnosticPhase::Frontend);
                assert_eq!(draft.category(), FailureCategory::ParseError);
                assert_eq!(draft.message(), diagnostic.message.as_ref());
                let DiagnosticLocation::SourceRange(range) = diagnostic.location else {
                    panic!("actual source range");
                };
                let DiagnosticPrimaryLocation::Span(span) = draft.primary_location() else {
                    panic!("converted source range");
                };
                assert_eq!(span.range(), range);
                assert_eq!(range.source_id, oracle.source.source_id);
                assert_eq!(
                    draft
                        .secondary_spans()
                        .iter()
                        .map(|span| span.anchor())
                        .collect::<Vec<_>>(),
                    diagnostic.secondary.iter().collect::<Vec<_>>()
                );
                assert_eq!(
                    draft
                        .notes()
                        .iter()
                        .map(|note| note.message())
                        .collect::<Vec<_>>(),
                    diagnostic
                        .recovery_note
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    draft.details().entries().get("frontend.class"),
                    Some(&DiagnosticDetailValue::String(class.into()))
                );
                assert_eq!(
                    draft.details().entries().get("frontend.code"),
                    Some(&DiagnosticDetailValue::String(local_code.into()))
                );
                assert_eq!(draft.details().entries().len(), 2);
                assert!(draft.notes().iter().all(|note| note.span().is_none()));
                assert!(draft.fixes().is_empty());
                assert!(draft.explanation().is_none());
            }
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Failed)
            );
        } else if mode == "cancel" {
            assert!(
                run.task_states
                    .iter()
                    .any(|record| record.state == TaskState::Cancelled)
            );
            assert!(results.iter().all(|result| result.output_refs.is_empty()));
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Cancelled)
            );
        } else {
            assert!(
                results
                    .iter()
                    .any(|result| result.status == PhaseStatus::Blocking)
            );
            assert!(
                results
                    .iter()
                    .flat_map(|result| &result.output_refs)
                    .all(|output| output.output_kind() != &OutputKind::new("FrontendOutput"))
            );
            if mode == "generated" {
                assert!(results.iter().all(|result| result.output_refs.is_empty()));
            }
        }
    }
}

#[test]
fn native_capture_and_prepared_submission_fail_closed_without_replacing_pending_context() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (request, _) = fixture.submit_request(vec![], vec![]);
    let control = InMemorySessionIdAllocator::new();
    assert_eq!(
        ids.next_request_id().unwrap(),
        control.next_request_id().unwrap()
    );
    let error = request
        .clone()
        .capture_sources(&ids, &snapshots, vec![])
        .unwrap_err();
    assert!(
        matches!(error, DriverSubmitError::NativeSourceVersionsProvided { request: original } if *original == request)
    );
    assert_eq!(
        ids.next_request_id().unwrap(),
        control.next_request_id().unwrap()
    );
    assert_eq!(
        ids.next_session_id().unwrap(),
        control.next_session_id().unwrap()
    );
    let (mut request, input) = fixture.submit_request(vec![], vec![]);
    request.source_inputs.versions.clear();
    let descriptor = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::OpenBuffer {
            uri: format!(
                "file://{}",
                fixture.root.join("alpha/src/main.miz").display()
            ),
            expected_version: 7,
            actual_version: 6,
            text: std::sync::Arc::from("definition\nend;\n"),
        },
    };
    let mut valid = descriptor.clone();
    let SourceOriginInput::OpenBuffer { actual_version, .. } = &mut valid.origin else {
        unreachable!()
    };
    *actual_version = 7;
    request.clone().allocate(&control).unwrap();
    let seeded = request
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                valid.clone(),
            )],
        )
        .unwrap();
    let expected_pending = request.clone().allocate(&control).unwrap();
    let error = request
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor.clone(),
            )],
        )
        .unwrap_err();
    let DriverSubmitError::SnapshotCapture { error } = error else {
        panic!("native capture error")
    };
    assert_eq!(error.pending, expected_pending);
    assert!(matches!(
        error.error,
        mizar_session::SnapshotError::SourceLoading {
            error: mizar_session::SourceLoadError::StaleLspDocumentVersion {
                expected: 7,
                actual: 6
            }
        }
    ));
    assert!(snapshots.is_current_for_request(seeded.captured().snapshot().id, seeded.request().id));
    assert!(
        !snapshots
            .is_current_for_request(seeded.captured().snapshot().id, error.pending.request.id)
    );
    let session = request
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                valid.clone(),
            )],
        )
        .unwrap();
    let mut driver = CompilerDriver::new(source_registry());
    let mut newer = request;
    newer.generation = BuildRequestGeneration::new(10);
    let mut cancelled = newer
        .capture_sources(
            &ids,
            &snapshots,
            vec![(DiskSourceLoader::new(fixture.root.join("alpha")), valid)],
        )
        .unwrap();
    cancelled.cancel();
    let cancelled_id = cancelled.id();
    assert!(
        matches!(driver.submit_captured(cancelled, &ids, &snapshots, input), Err(DriverSubmitError::InvalidCapturedSession { session }) if session.id() == cancelled_id)
    );
    assert!(driver.session(cancelled_id).is_none());
    let (_, input) = fixture.submit_request(vec![], vec![]);
    assert!(matches!(
        driver.submit_captured(session.clone(), &ids, &SnapshotRegistry::new(), input),
        Err(DriverSubmitError::InvalidCapturedSession { .. })
    ));
    let (_, input) = fixture.submit_request(vec![], vec![]);
    let submission = driver
        .submit_captured(session.clone(), &ids, &snapshots, input)
        .unwrap();
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByMissingPhaseServices
    );
    let (_, input) = fixture.submit_request(vec![], vec![]);
    assert!(matches!(
        driver.submit_captured(session, &ids, &snapshots, input),
        Err(DriverSubmitError::InvalidCapturedSession { .. })
    ));
    let (legacy_request, input) = fixture.submit_request(vec![], vec![]);
    let legacy = legacy_request
        .allocate(&ids)
        .unwrap()
        .capture_snapshot(&snapshots)
        .unwrap();
    assert!(matches!(
        driver.submit_captured(legacy, &ids, &snapshots, input),
        Err(DriverSubmitError::InvalidCapturedSession { .. })
    ));
}

#[test]
fn missing_package_root_retains_real_loader_diagnostic() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    fs::remove_dir_all(fixture.root.join("alpha")).unwrap();
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_source_error(&result, &fixture.version, 602, "source.unreadable_file");
}

#[test]
fn computation_replay_native_inputs_retain_metadata_in_protected_driver_plans() {
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
    let root = WorkspaceRoot::new(fixture.root.join("alpha").to_str().unwrap());
    input.plan_request.workspace_root = root.clone();
    request.workspace_root = root.clone();
    let manifest =
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[verifier]\ncomputation_replay = true\n";
    let lock = "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = []\n";
    fs::write(fixture.root.join("alpha/mizar.pkg"), manifest).unwrap();
    fs::write(fixture.root.join("alpha/mizar.lock"), lock).unwrap();
    let acquired = mizar_build::planner::read_native_workspace_inputs(&root).unwrap();
    let explicit = (
        vec![WorkspacePackage {
            member_path: ".".into(),
            manifest: parse_package_manifest(manifest).unwrap(),
        }],
        parse_lockfile(lock).unwrap(),
    );
    assert_eq!(acquired, explicit);
    let plan = produce_build_plan(
        input.plan_request.clone(),
        acquired.0.clone(),
        acquired.1.clone(),
    )
    .unwrap();
    let native = StaticSourceLayout::from_native_disk(&plan).unwrap();
    assert_eq!(native, input.source_layout);
    request.source_inputs.versions.clear();
    let session = request
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new("main"),
                    normalized_path: normalize_path(
                        &fixture.root.join("alpha"),
                        Path::new("src/main.miz"),
                    )
                    .unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    },
                },
            )],
        )
        .unwrap();
    let snapshot = session.captured().snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
    let mut clean = None;
    for ((packages, lockfile), layout) in
        [(explicit, input.source_layout.clone()), (acquired, native)]
    {
        let (_, mut current) = fixture.submit_request(vec![], vec![]);
        current.plan_request = input.plan_request.clone();
        current.workspace_packages = packages;
        current.lockfile = lockfile;
        current.source_layout = layout;
        current.worker_count = 4;
        current.completion_order = CompletionOrder::Reverse;
        let owner = publisher(snapshot, 0, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(session.clone(), &ids, &snapshots, current)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let results = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .collect::<Vec<_>>();
        let refs = results
            .iter()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        assert_eq!(refs.len(), 2);
        assert!(results.iter().all(|result| result.diagnostics.is_empty()));
        let source_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
            .unwrap();
        let source_handle = owner
            .storage()
            .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
            .unwrap();
        let source = owner.storage().get(&source_handle).unwrap();
        let frontend_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        let frontend_handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                frontend_ref,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let frontend = owner.storage().get(&frontend_handle).unwrap();
        assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
        assert!(source.loading_map.is_some());
        assert_eq!(
            source.source_id,
            session.captured().snapshot().source_versions[0].source_id
        );
        let retained = submission.build_plan.as_ref().unwrap();
        assert_eq!(retained, &plan);
        assert!(retained.packages[0].verifier_config.computation_replay);
        assert!(
            retained.verifier_config.packages[0]
                .config
                .computation_replay
        );
        let actual = (
            submission.build_plan.unwrap(),
            submission.module_index.unwrap(),
            source.as_ref().clone(),
            frontend.as_ref().clone(),
        );
        if let Some(expected) = &clean {
            assert_eq!(&actual, expected);
        } else {
            clean = Some(actual);
        }
    }
}

#[test]
fn workspace_lock_paths_reject_actual_native_inputs_before_source_phases() {
    use mizar_build::planner::{ManifestDiagnosticKind, read_native_workspace_inputs};
    use mizar_driver::events::PlanningEventStatus;
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut request, clean_input) = fixture.submit_request(vec![], vec![]);
    fs::write(
        fixture.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"alpha\"]\n",
    )
    .unwrap();
    fs::write(
        fixture.root.join("alpha/mizar.pkg"),
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    request.source_inputs.versions.clear();
    let session = request
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new("main"),
                    normalized_path: normalize_path(
                        &fixture.root.join("alpha"),
                        Path::new("src/main.miz"),
                    )
                    .unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    },
                },
            )],
        )
        .unwrap();
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    for path in ["alpha", "unbound"] {
        let text = format!(
            "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"{path}\" }}\ndependencies = []\n"
        );
        fs::write(fixture.root.join("mizar.lock"), &text).unwrap();
        let (packages, lock) =
            read_native_workspace_inputs(&clean_input.plan_request.workspace_root).unwrap();
        assert_eq!(packages, clean_input.workspace_packages);
        assert_eq!(lock, parse_lockfile(&text).unwrap());
        let (_, mut input) = fixture.submit_request(vec![], vec![]);
        input.workspace_packages = packages;
        input.lockfile = lock;
        let owner = publisher(session.captured().snapshot().id, 0, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let result = driver.submit_captured(session.clone(), &ids, &snapshots, input);
        if path == "alpha" {
            let submission = result.unwrap();
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
            let plan = produce_build_plan(
                clean_input.plan_request.clone(),
                clean_input.workspace_packages.clone(),
                clean_input.lockfile.clone(),
            )
            .unwrap();
            assert_eq!(submission.build_plan.as_ref().unwrap(), &plan);
            let results = submission.scheduler_run.unwrap().phase_results;
            assert!(
                results
                    .values()
                    .flatten()
                    .all(|result| result.status == PhaseStatus::Complete
                        && result.diagnostics.is_empty())
            );
            let refs = results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .collect::<Vec<_>>();
            assert_eq!(refs.len(), 2);
            assert!(
                refs.iter()
                    .any(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            );
            let source_ref = *refs
                .iter()
                .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap();
            let handle = owner
                .storage()
                .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                .unwrap();
            assert_eq!(
                owner.storage().get(&handle).unwrap().source_text.as_ref(),
                "definition\nend;\n"
            );
        } else {
            let DriverSubmitError::Planning {
                session,
                diagnostics,
            } = result.unwrap_err()
            else {
                panic!("expected native coordinate planning failure");
            };
            assert_eq!(
                session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Failed)
            );
            let actual = diagnostics.diagnostics();
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].kind, ManifestDiagnosticKind::InvalidLockSource);
            assert_eq!(actual[0].location.path, "<lockfile>");
            assert_eq!(actual[0].location.key, "alpha.source.path");
            assert_eq!(actual[0].value.as_deref(), Some("unbound != alpha"));
            let events = driver.events(session.id());
            assert!(events.events().iter().any(|event| matches!(
                event.kind,
                BuildEventKind::PlanningReady {
                    status: PlanningEventStatus::Failed
                }
            )));
            assert!(!events.events().iter().any(|event| matches!(
                event.kind,
                BuildEventKind::TaskProgress { .. } | BuildEventKind::PhaseReady { .. }
            )));
        }
    }
}

#[test]
fn lock_content_hash_from_actual_files_feeds_genuine_snapshot_and_source_load_carriers() {
    use mizar_build::planner::{lockfile_content_hash, read_native_workspace_inputs};
    use mizar_proof::policy::VerifierPolicy;
    let fixture = Fixture::new(b"definition\nend;\n");
    let manifest =
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dev-dependencies]\ndep = \"^1.0\"\n";
    fs::write(
        fixture.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"alpha\"]\n",
    )
    .unwrap();
    fs::write(fixture.root.join("alpha/mizar.pkg"), manifest).unwrap();
    let mut previous = None;
    for (version, comment) in [
        ("1.0.0", ""),
        ("1.0.0", "# only raw formatting changed\n"),
        ("1.1.0", ""),
    ] {
        let text = format!(
            "{comment}schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"alpha\" }}\ndependencies = [{{ name = \"dep\", version = \"{version}\" }}]\n[[package]]\nname = \"dep\"\nversion = \"{version}\"\nsource = {{ kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }}\ndependencies = []\n"
        );
        fs::write(fixture.root.join("mizar.lock"), &text).unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
        let (packages, lock) =
            read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap();
        let clean = parse_lockfile(&text).unwrap();
        assert_eq!(lock, clean);
        assert_eq!(
            packages[0].manifest,
            parse_package_manifest(manifest).unwrap()
        );
        let digest = lockfile_content_hash(&lock);
        assert_eq!(digest, lockfile_content_hash(&clean));
        draft.dependency_inputs.lockfile_hash = digest;
        input.workspace_packages = packages;
        input.lockfile = lock;
        draft.source_inputs.versions.clear();
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: fixture.version.package_id.clone(),
                        module_path: fixture.version.module_path.clone(),
                        normalized_path: fixture.version.normalized_path.clone(),
                        edition: fixture.version.edition.clone(),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        },
                    },
                )],
            )
            .unwrap();
        assert_eq!(session.captured().snapshot().lockfile_hash, digest);
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(session.captured().snapshot().id, 0, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let captured = submission.session.captured();
        let actual_version = &captured.snapshot().source_versions[0];
        let phase_input = PhaseInput::new(
            unit,
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(actual_version),
                vec![],
            ),
        );
        let request = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        }
        .prepare_native_disk_cache_request(&phase_input, &VerifierPolicy::release())
        .unwrap();
        let lock_input = request
            .input_hashes
            .iter()
            .find(|entry| entry.name == "captured-lockfile")
            .unwrap();
        assert_eq!(
            lock_input.domain,
            "mizar-driver/captured-lockfile-identity/v1"
        );
        assert_eq!(lock_input.digest, digest);
        let results = &submission.scheduler_run.as_ref().unwrap().phase_results;
        let refs = results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        assert_eq!(refs.len(), 2);
        let source_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
            .unwrap();
        let handle = owner
            .storage()
            .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
            .unwrap();
        let source = owner.storage().get(&handle).unwrap();
        assert_eq!(source.source_id, actual_version.source_id);
        assert_eq!(source.source_hash, actual_version.source_hash);
        for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
            assert!(matches!(
                driver
                    .registry()
                    .cache_key_for_phase(phase, &phase_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey { .. }
            ));
        }
        if let Some((prior_digest, prior_snapshot, prior_source, prior_carrier)) = &previous {
            assert_eq!(source.source_hash, *prior_source);
            if version == "1.0.0" {
                assert_eq!(digest, *prior_digest);
                assert_eq!(captured.snapshot().id, *prior_snapshot);
                assert_eq!(&request, prior_carrier);
            } else {
                assert_ne!(digest, *prior_digest);
                assert_ne!(captured.snapshot().id, *prior_snapshot);
                let mut normalized = request.clone();
                normalized
                    .input_hashes
                    .iter_mut()
                    .find(|entry| entry.name == "captured-lockfile")
                    .unwrap()
                    .digest = *prior_digest;
                assert_eq!(&normalized, prior_carrier);
            }
        }
        previous = Some((digest, captured.snapshot().id, source.source_hash, request));
    }
}

#[test]
fn config_content_hash_from_native_plans_feeds_genuine_snapshot_and_source_load_carriers() {
    use mizar_build::planner::{read_native_workspace_inputs, workspace_config_content_hash};
    use mizar_proof::policy::VerifierPolicy;
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    fs::write(
        fixture.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"alpha\"]\n",
    )
    .unwrap();
    let lock_text = "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n";
    fs::write(fixture.root.join("mizar.lock"), lock_text).unwrap();
    let mut baseline = None;
    for (unchanged, suffix) in [
        (true, ""),
        (
            true,
            "# spelling and omission are outside typed content\n[verifier]\nmax_axioms = 128\natp_timeout = 30\ndefault_solver = \"auto\"\ncomputation_replay = false\nrequire_kernel_certificates = true\n[build]\nincremental = true\ncache_dir = \".mizar-cache\"\nartifact_dir = \"build\"\n",
        ),
        (false, "[verifier]\ncomputation_replay = true\n"),
        (false, "[build]\ncache_dir = \"different-cache\"\n"),
        (false, "[build]\nincremental = false\n"),
    ] {
        fixture.write(&fixture.bytes);
        let text = format!("[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n{suffix}");
        fs::write(fixture.root.join("alpha/mizar.pkg"), &text).unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
        let (packages, lock) =
            read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap();
        let clean_plan = produce_build_plan(
            input.plan_request.clone(),
            vec![WorkspacePackage {
                member_path: "alpha".into(),
                manifest: parse_package_manifest(&text).unwrap(),
            }],
            parse_lockfile(lock_text).unwrap(),
        )
        .unwrap();
        let plan =
            produce_build_plan(input.plan_request.clone(), packages.clone(), lock.clone()).unwrap();
        assert_eq!(plan, clean_plan);
        let digest = workspace_config_content_hash(&plan.verifier_config, &plan.build_config);
        assert_eq!(
            digest,
            workspace_config_content_hash(&clean_plan.verifier_config, &clean_plan.build_config)
        );
        draft.verifier_config = VerifierConfigInput::new(digest);
        draft.source_inputs.versions.clear();
        input.source_layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
        input.workspace_packages = packages;
        input.lockfile = lock;
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: fixture.version.package_id.clone(),
                        module_path: fixture.version.module_path.clone(),
                        normalized_path: fixture.version.normalized_path.clone(),
                        edition: fixture.version.edition.clone(),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        },
                    },
                )],
            )
            .unwrap();
        assert_eq!(session.captured().snapshot().verifier_config_hash, digest);
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(session.captured().snapshot().id, 0, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        assert_eq!(submission.build_plan.as_ref().unwrap(), &clean_plan);
        let (inputs, phase_input) = native_disk_lookup_context(&submission, &ids);
        let request = inputs
            .prepare_native_disk_cache_request(&phase_input, &VerifierPolicy::release())
            .unwrap();
        assert_eq!(request.config_hash, digest);
        let results = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .collect::<Vec<_>>();
        assert!(results.iter().all(|result| result.diagnostics.is_empty()));
        let refs = results
            .iter()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        assert_eq!(refs.len(), 2);
        let source_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
            .unwrap();
        let source_handle = owner
            .storage()
            .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
            .unwrap();
        let source = owner.storage().get(&source_handle).unwrap();
        let frontend_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        let frontend_handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                frontend_ref,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let frontend = owner.storage().get(&frontend_handle).unwrap();
        let captured = submission.session.captured();
        assert_eq!(
            source.source_id,
            captured.snapshot().source_versions[0].source_id
        );
        assert_eq!(
            source.source_hash,
            captured.snapshot().source_versions[0].source_hash
        );
        assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
        assert!(source.loading_map.is_some());
        let source_bytes = source.canonical_disk_bytes().unwrap();
        let frontend_bytes = frontend.canonical_disk_bytes().unwrap();
        for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
            assert!(matches!(
                driver
                    .registry()
                    .cache_key_for_phase(phase, &phase_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey { .. }
            ));
        }
        if let Some((
            prior_digest,
            prior_snapshot,
            prior_hash,
            prior_source,
            prior_frontend,
            prior_request,
        )) = &baseline
        {
            assert_eq!(source.source_hash, *prior_hash);
            assert_eq!(&source_bytes, prior_source);
            assert_eq!(&frontend_bytes, prior_frontend);
            if unchanged {
                assert_eq!(digest, *prior_digest);
                assert_eq!(captured.snapshot().id, *prior_snapshot);
                assert_eq!(&request, prior_request);
            } else {
                assert_ne!(digest, *prior_digest);
                assert_ne!(captured.snapshot().id, *prior_snapshot);
                let mut normalized = request.clone();
                normalized.config_hash = *prior_digest;
                assert_eq!(&normalized, prior_request);
            }
        } else {
            baseline = Some((
                digest,
                captured.snapshot().id,
                source.source_hash,
                source_bytes,
                frontend_bytes,
                request,
            ));
        }
    }
}

#[test]
fn package_toolchain_native_inputs_retain_association_in_protected_driver_plans() {
    use mizar_build::planner::{
        VersionComparator, VersionComparison, VersionConstraint, read_native_workspace_inputs,
    };
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    fs::create_dir_all(fixture.root.join("beta/src")).unwrap();
    fs::write(
        fixture.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"beta\", \"alpha\"]\n",
    )
    .unwrap();
    let alpha = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\ntoolchain = \"<1\"\n";
    let beta = "[package]\nname = \"beta\"\nversion = \"0.1.0\"\ntoolchain = \">=2\"\n";
    let lock = "schema_version = 1\n[[package]]\nname = \"beta\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"beta\" }\ndependencies = []\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n";
    fs::write(fixture.root.join("alpha/mizar.pkg"), alpha).unwrap();
    fs::write(fixture.root.join("beta/mizar.pkg"), beta).unwrap();
    fs::write(fixture.root.join("mizar.lock"), lock).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, input) = fixture.submit_request(vec![], vec![]);
    let acquired = read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap();
    let explicit = (
        [("alpha", alpha), ("beta", beta)]
            .map(|(member_path, text)| WorkspacePackage {
                member_path: member_path.into(),
                manifest: parse_package_manifest(text).unwrap(),
            })
            .to_vec(),
        parse_lockfile(lock).unwrap(),
    );
    // Compare canonical plans independently of the native reader's member order.
    let clean_plan = produce_build_plan(
        input.plan_request.clone(),
        explicit.0.clone(),
        explicit.1.clone(),
    )
    .unwrap();
    let native_plan = produce_build_plan(
        input.plan_request.clone(),
        acquired.0.clone(),
        acquired.1.clone(),
    )
    .unwrap();
    assert_eq!(native_plan, clean_plan);
    assert_eq!(native_plan.packages[0].package_id.as_str(), "alpha");
    assert_eq!(native_plan.packages[1].package_id.as_str(), "beta");
    assert_eq!(
        native_plan.packages[0].toolchain_constraint,
        Some(VersionConstraint::Range(vec![VersionComparator {
            op: VersionComparison::Less,
            version: "1.0.0".parse().unwrap(),
        }]))
    );
    assert_eq!(
        native_plan.packages[1].toolchain_constraint,
        Some(VersionConstraint::Range(vec![VersionComparator {
            op: VersionComparison::GreaterEqual,
            version: "2.0.0".parse().unwrap(),
        }]))
    );
    assert_eq!(native_plan.toolchain, input.plan_request.toolchain);
    let native_layout = StaticSourceLayout::from_native_disk(&native_plan).unwrap();
    let explicit_layout = StaticSourceLayout::new(vec![
        WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
        },
        WorkspaceSourcePackage {
            package_id: PackageId::new("beta"),
            files: vec![],
        },
    ]);
    assert_eq!(native_layout, explicit_layout);
    draft.source_inputs.versions.clear();
    let session = draft
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                SourceInput {
                    package_id: fixture.version.package_id.clone(),
                    module_path: fixture.version.module_path.clone(),
                    normalized_path: fixture.version.normalized_path.clone(),
                    edition: fixture.version.edition.clone(),
                    origin: SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    },
                },
            )],
        )
        .unwrap();
    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    let mut baseline = None;
    for ((packages, lockfile), layout) in [(explicit, explicit_layout), (acquired, native_layout)] {
        let (_, mut current) = fixture.submit_request(vec![], vec![]);
        current.plan_request = input.plan_request.clone();
        current.workspace_packages = packages;
        current.lockfile = lockfile;
        current.source_layout = layout;
        current.worker_count = 4;
        current.completion_order = CompletionOrder::Reverse;
        let owner = publisher(session.captured().snapshot().id, 0, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(session.clone(), &ids, &snapshots, current)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        assert_eq!(submission.build_plan.as_ref().unwrap(), &clean_plan);
        let (_, phase_input) = native_disk_lookup_context(&submission, &ids);
        for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
            assert!(matches!(
                driver
                    .registry()
                    .cache_key_for_phase(phase, &phase_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey { .. }
            ));
        }
        let results = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .collect::<Vec<_>>();
        assert!(results.iter().all(|result| result.diagnostics.is_empty()));
        let refs = results
            .iter()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        assert_eq!(refs.len(), 2);
        let source_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
            .unwrap();
        let source_handle = owner
            .storage()
            .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
            .unwrap();
        let source = owner.storage().get(&source_handle).unwrap();
        let frontend_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        let frontend_handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                frontend_ref,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let frontend = owner.storage().get(&frontend_handle).unwrap();
        assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
        assert!(source.loading_map.is_some());
        assert_eq!(
            source.source_id,
            session.captured().snapshot().source_versions[0].source_id
        );
        let actual = (
            submission.build_plan.unwrap(),
            submission.module_index.unwrap(),
            source.as_ref().clone(),
            frontend.as_ref().clone(),
        );
        if let Some(expected) = &baseline {
            assert_eq!(&actual, expected);
        } else {
            baseline = Some(actual);
        }
    }
    for text in ["toolchain = true", "toolchain = \"^bad\""] {
        let invalid = format!("[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n{text}\n");
        fs::write(fixture.root.join("alpha/mizar.pkg"), &invalid).unwrap();
        assert_eq!(
            read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap_err(),
            parse_package_manifest(&invalid).unwrap_err().to_string()
        );
    }
}

#[test]
fn configured_native_cache_parsed_settings_and_store_fallbacks() {
    for threshold in [usize::MAX, 0] {
        for (build, store, policy, insert, hit) in [
            (
                "",
                "alpha/.mizar-cache",
                CacheSchedulingPolicy::Enabled,
                true,
                true,
            ),
            (
                "[build]\ncache_dir = \"nested/cache\"\n",
                "alpha/nested/cache",
                CacheSchedulingPolicy::Enabled,
                true,
                true,
            ),
            (
                "",
                "alpha/.mizar-cache",
                CacheSchedulingPolicy::Disabled,
                true,
                false,
            ),
            (
                "",
                "alpha/.mizar-cache",
                CacheSchedulingPolicy::Enabled,
                false,
                false,
            ),
            (
                "",
                "foreign-cache",
                CacheSchedulingPolicy::Enabled,
                true,
                false,
            ),
        ] {
            configured_native_cache_case(
                &[("alpha", build, hit)],
                None,
                threshold,
                store,
                (policy, insert),
                1,
                false,
            );
        }
    }
}

#[test]
fn configured_native_cache_opt_out_and_direct_typed_invalid_paths() {
    configured_native_cache_case(
        &[("alpha", "[build]\nincremental = false\n", false)],
        None,
        usize::MAX,
        "$configured",
        (CacheSchedulingPolicy::Enabled, true),
        1,
        false,
    );
    for path in [
        "$absolute",
        "",
        ".",
        "..",
        "../cache",
        "cache\\nested",
        #[cfg(not(windows))]
        "cache:tag",
        "nested//cache",
        "nested/./cache",
        "nested/../cache",
    ] {
        configured_native_cache_case(
            &[("alpha", "", false)],
            Some(path),
            usize::MAX,
            "$configured",
            (CacheSchedulingPolicy::Enabled, true),
            1,
            false,
        );
    }
}

#[test]
fn configured_native_cache_uses_each_module_package_under_reordering() {
    for (workers, reverse) in [(1, false), (4, true)] {
        for (alpha_enabled, beta_enabled, store, alpha_hit, beta_hit) in [
            (true, true, "alpha/cache/a", true, false),
            (true, true, "beta/cache/b", false, true),
            (false, true, "beta/cache/b", false, true),
            (true, false, "alpha/cache/a", true, false),
        ] {
            let alpha =
                format!("[build]\nincremental = {alpha_enabled}\ncache_dir = \"cache/a\"\n");
            let beta = format!("[build]\nincremental = {beta_enabled}\ncache_dir = \"cache/b\"\n");
            configured_native_cache_case(
                &[("alpha", &alpha, alpha_hit), ("beta", &beta, beta_hit)],
                None,
                0,
                store,
                (CacheSchedulingPolicy::Enabled, true),
                workers,
                reverse,
            );
        }
    }
}

#[test]
fn persistent_native_cache_cold_write_then_fresh_warm_matches_clean() {
    for threshold in [usize::MAX, 0] {
        for (build, store) in [
            ("", "alpha/.mizar-cache"),
            (
                "[build]\ncache_dir = \"nested/cache\"\n",
                "alpha/nested/cache",
            ),
        ] {
            persistent_native_cache_case(
                &[("alpha", build, true)],
                "hit",
                &["\u{feff}definition\r\nend;\r\n"],
                threshold,
                store,
                1,
                false,
            );
        }
    }
}

#[test]
fn persistent_native_cache_cold_guards_cancellation_and_real_store_error() {
    for case in [
        "disabled",
        "foreign",
        "provider",
        "context",
        "cancel",
        "obsolete",
        "cancel-frontend",
        "publisher",
        "store-error",
        "opt-out",
    ] {
        let build = if case == "opt-out" {
            "[build]\nincremental = false\n"
        } else {
            ""
        };
        persistent_native_cache_case(
            &[("alpha", build, true)],
            case,
            &["definition\nend;\n"],
            usize::MAX,
            if case == "foreign" {
                "foreign-cache"
            } else {
                "alpha/.mizar-cache"
            },
            1,
            false,
        );
    }
}

#[test]
fn persistent_native_cache_edits_failed_frontend_and_each_package() {
    for (workers, reverse, store, alpha, beta) in [
        (1, false, "alpha/cache/a", true, false),
        (4, true, "beta/cache/b", false, true),
    ] {
        persistent_native_cache_case(
            &[
                ("alpha", "[build]\ncache_dir = \"cache/a\"\n", alpha),
                ("beta", "[build]\ncache_dir = \"cache/b\"\n", beta),
            ],
            "edits",
            &[
                "\u{feff}definition\r\nend;\r\n",
                "\u{feff}theorem T: ;\r\n",
                "theorem T: ;\n",
            ],
            0,
            store,
            workers,
            reverse,
        );
    }
}

#[test]
fn persistent_native_frontend_cold_and_source_hit_records_reconstruct_like_clean() {
    for threshold in [usize::MAX, 0] {
        persistent_native_frontend_case(
            &[("alpha", "")],
            "hit",
            &["\u{feff}definition\r\nend;\r\n"],
            threshold,
            "alpha/.mizar-cache",
            false,
        );
    }
}

#[test]
fn persistent_native_frontend_edits_and_package_order_keep_real_associations() {
    for (reverse, store) in [(false, "alpha/cache/a"), (true, "beta/cache/b")] {
        persistent_native_frontend_case(
            &[
                ("alpha", "[build]\ncache_dir = \"cache/a\"\n"),
                ("beta", "[build]\ncache_dir = \"cache/b\"\n"),
            ],
            "edits",
            &[
                "\u{feff}definition\r\nend;\r\n",
                "\u{feff}definition\r\nlet x be set;\r\nend;\r\n",
                "definition\nlet x be set;\nend;\n",
            ],
            0,
            store,
            reverse,
        );
    }
}

#[test]
fn persistent_native_frontend_cold_guards_errors_and_older_entries() {
    for case in [
        "disabled",
        "foreign",
        "provider",
        "context",
        "cancel",
        "obsolete",
        "cancel-frontend",
        "store-error",
        "opt-out",
        "source-only",
        "read-only",
        "failed",
        "imports",
    ] {
        persistent_native_frontend_case(
            &[(
                "alpha",
                if case == "opt-out" {
                    "[build]\nincremental = false\n"
                } else {
                    ""
                },
            )],
            case,
            &[if case == "failed" {
                "theorem T: ;\n"
            } else if case == "imports" {
                "import alpha.leaf;\ndefinition\nend;\n"
            } else {
                "definition\nend;\n"
            }],
            usize::MAX,
            if case == "foreign" {
                "foreign-cache"
            } else {
                "alpha/.mizar-cache"
            },
            false,
        );
    }
}

#[test]
fn native_public_theorem_formula_text_candidates_preserve_captured_original_owners_and_text() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let statuses = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
    );
    let plain_formula = "for X being set holds X = X";
    let commented_formula = "for X being set\n  holds X ::= 日本語 formula =::  =  X";
    for (text, formulas) in [
        (private.to_owned(), vec![("UsePriv1", plain_formula)]),
        (
            private.replace("theorem UsePriv1", "lemma UsePriv1"),
            vec![("UsePriv1", plain_formula)],
        ),
        (
            private.replacen(
                "thus X = X by PrivT1;",
                "Inner: X = X;\n  thus X = X by Inner;",
                1,
            ),
            vec![("UsePriv1", plain_formula)],
        ),
        (
            private.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
            vec![],
        ),
        (
            statuses.replace(
                "assumed theorem AxT1: for X being set holds X = X;",
                "assumed theorem AxT1: for Y being set holds Y = Y;",
            ),
            vec![
                ("OpenT1", plain_formula),
                ("AxT1", "for Y being set holds Y = Y"),
            ],
        ),
        (
            format!(
                "\u{feff}{}",
                private
                    .replace(plain_formula, commented_formula)
                    .replace('\n', "\r\n")
            ),
            vec![("UsePriv1", commented_formula)],
        ),
    ] {
        for open in [false, true] {
            let fixture = Fixture::new(text.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut draft, input) = fixture.submit_request(vec![], vec![]);
            draft.source_inputs.versions.clear();
            let descriptor = SourceInput {
                package_id: fixture.version.package_id.clone(),
                module_path: fixture.version.module_path.clone(),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: fixture.version.edition.clone(),
                origin: if open {
                    SourceOriginInput::OpenBuffer {
                        uri: format!(
                            "file://{}",
                            fixture.root.join("alpha/src/main.miz").display()
                        ),
                        expected_version: 7,
                        actual_version: 7,
                        text: std::sync::Arc::from(text.as_str()),
                    }
                } else {
                    SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    }
                },
            };
            if open {
                fixture.write(&[0xff]);
            }
            let session = draft
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        descriptor.clone(),
                    )],
                )
                .unwrap();
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            let submission = CompilerDriver::new(builder.build().unwrap())
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            let (inputs, _) = native_disk_lookup_context(&submission, &ids);
            let module = mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            );
            let (_, loaded, captured_input) =
                &inputs.captured.unwrap().native_sources().unwrap()[0];
            let source = mizar_frontend::source::source_unit_from_loaded(
                loaded.clone(),
                mizar_frontend::source::diagnostic_file_path(captured_input),
            );
            assert_eq!(
                source.source_text.as_ref(),
                text.trim_start_matches('\u{feff}').replace("\r\n", "\n")
            );
            let output = mizar_frontend::orchestration::Frontend::new(
                mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
                    fixture.root.join("alpha"),
                )),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(source.clone())
            .unwrap();
            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
            let ast = output.ast.as_ref().unwrap();
            let resolver_module = mizar_resolve::resolved_ast::ModuleId::new(
                module.package.clone(),
                module.path.clone(),
            );
            let shells =
                mizar_resolve::declarations::DeclarationShellCollector::new(ast, &resolver_module)
                    .collect();
            let collection = mizar_resolve::symbols::SignatureProjectionExtractor::new(
                ast,
                &shells,
                mizar_resolve::env::NamespacePath::new("main"),
            )
            .collect(&resolver_module);
            assert!(collection.diagnostics().is_empty());
            let mut expected = std::collections::BTreeMap::new();
            for (label, formula) in &formulas {
                let owners = collection
                    .env()
                    .symbols()
                    .iter()
                    .filter(|owner| owner.primary_spelling() == *label)
                    .collect::<Vec<_>>();
                let [owner] = owners.as_slice() else {
                    panic!("one independently collected owner for {label}");
                };
                expected.insert(owner.symbol().clone(), (*formula).to_owned());
            }
            let before = (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                captured_input.clone(),
            );
            for stage in 0..3 {
                if stage == 1 {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                } else if stage == 2 {
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                }
                assert_eq!(
                    inputs
                        .native_public_theorem_formula_text_candidates(&module)
                        .unwrap(),
                    Some(expected.clone()),
                    "open={open}, stage={stage}"
                );
            }
            assert_eq!(
                (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                ),
                before
            );
            assert_eq!(captured_input, &descriptor);
        }
    }
}

#[test]
fn native_public_theorem_formula_text_candidates_refuse_unbound_and_incomplete_sources() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, diagnostic) in [
        (format!("{private}\ntheorem Plain: 0 = 0;\n"), false),
        ("theorem Broken: ;\n".to_owned(), true),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let foreign = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, _, _, _) =
            native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
        let (_, foreign_submission, _, _, _) =
            native_disk_record_submission(&foreign, &ids, &snapshots, usize::MAX);
        let (inputs, _) = native_disk_lookup_context(&submission, &ids);
        let module = mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        );
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let output = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
                fixture.root.join("alpha"),
            )),
            inputs.dependency_lexical_provider(&[]),
            MizarParserSeam,
        )
        .run_loaded(mizar_frontend::source::source_unit_from_loaded(
            loaded.clone(),
            mizar_frontend::source::diagnostic_file_path(descriptor),
        ))
        .unwrap();
        assert_eq!(!output.diagnostics.is_empty(), diagnostic);
        assert_eq!(
            inputs
                .native_public_theorem_formula_text_candidates(&module)
                .unwrap(),
            None
        );
        for requested in [
            mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("missing"),
            ),
            mizar_build::module_index::ModuleId::new(
                PackageId::new("foreign"),
                ModulePath::new("main"),
            ),
        ] {
            assert!(
                inputs
                    .native_public_theorem_formula_text_candidates(&requested)
                    .is_err()
            );
        }
        let mut ambiguous = inputs.snapshot.clone();
        ambiguous
            .source_versions
            .push(ambiguous.source_versions[0].clone());
        let mut absent_index = inputs.module_index.clone();
        absent_index.modules.clear();
        let mut ambiguous_index = inputs.module_index.clone();
        ambiguous_index
            .modules
            .push(ambiguous_index.modules[0].clone());
        for invalid in [
            SourceLoadInputs {
                captured: None,
                ..inputs
            },
            SourceLoadInputs {
                captured: Some(foreign_submission.session.captured()),
                ..inputs
            },
            SourceLoadInputs {
                snapshot: &ambiguous,
                ..inputs
            },
            SourceLoadInputs {
                module_index: &absent_index,
                ..inputs
            },
            SourceLoadInputs {
                module_index: &ambiguous_index,
                ..inputs
            },
        ] {
            assert!(
                invalid
                    .native_public_theorem_formula_text_candidates(&module)
                    .is_err()
            );
        }
    }
}

#[test]
fn retained_native_frontend_cache_hits_match_clean_resident_blob_replay_and_order() {
    for threshold in [usize::MAX, 0] {
        for reverse in [false, true] {
            retained_native_frontend_cache_case("hit", threshold, reverse);
        }
    }
}

#[test]
fn retained_native_frontend_cache_misses_and_unsupported_profiles_execute_normally() {
    for case in [
        "source-miss",
        "frontend-miss",
        "source-corrupt",
        "frontend-corrupt",
        "producer",
        "store-error",
        "disabled",
        "provider",
        "context",
        "binding",
        "graph",
        "oracle-foreign",
        "oracle-stale",
        "oracle-collected",
        "oracle-status",
        "oracle-empty",
        "oracle-multiple",
        "oracle-diagnostics",
    ] {
        retained_native_frontend_cache_case(case, usize::MAX, false);
    }
}

#[test]
fn retained_native_frontend_cache_rejected_skips_release_actual_owners() {
    for case in [
        "cancel-source",
        "cancel-frontend",
        "cancel-root",
        "obsolete-source",
        "obsolete-frontend",
        "resource",
        "duplicate",
    ] {
        retained_native_frontend_cache_case(case, 0, true);
    }
}

#[test]
fn locked_toolchain_metadata_native_capture_matches_clean_consumers_and_rejects_reassociation() {
    use mizar_build::planner::{lockfile_content_hash, read_native_workspace_inputs};
    use mizar_proof::policy::VerifierPolicy;
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    let manifest = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\ntoolchain = \"<1\"\n";
    let body = "[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n";
    fs::write(
        fixture.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"alpha\"]\n",
    )
    .unwrap();
    fs::write(fixture.root.join("alpha/mizar.pkg"), manifest).unwrap();
    let mut previous = None;
    for version in [None, Some("4.5.6-rc.7+build.8"), Some("4.5.6-rc.7+build.9")] {
        let field = version.map_or(String::new(), |v| format!("toolchain_version = \"{v}\"\n"));
        let text = format!("schema_version = 1\n{field}{body}");
        fs::write(fixture.root.join("mizar.lock"), &text).unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, input) = fixture.submit_request(vec![], vec![]);
        let native = read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap();
        let clean = (
            vec![WorkspacePackage {
                member_path: "alpha".into(),
                manifest: parse_package_manifest(manifest).unwrap(),
            }],
            parse_lockfile(&text).unwrap(),
        );
        assert_eq!(native, clean);
        assert_eq!(
            native.1.toolchain_version,
            version.map(|v| v.parse().unwrap())
        );
        let plan = produce_build_plan(input.plan_request.clone(), clean.0.clone(), clean.1.clone())
            .unwrap();
        assert_eq!(plan.toolchain, input.plan_request.toolchain);
        let digest = lockfile_content_hash(&native.1);
        assert_eq!(digest, lockfile_content_hash(&clean.1));
        draft.dependency_inputs.lockfile_hash = digest;
        draft.source_inputs.versions.clear();
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: fixture.version.package_id.clone(),
                        module_path: fixture.version.module_path.clone(),
                        normalized_path: fixture.version.normalized_path.clone(),
                        edition: fixture.version.edition.clone(),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        },
                    },
                )],
            )
            .unwrap();
        assert_eq!(session.captured().snapshot().lockfile_hash, digest);
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let mut baseline = None;
        for (packages, lockfile) in [clean, native] {
            let (_, mut current) = fixture.submit_request(vec![], vec![]);
            current.workspace_packages = packages.clone();
            current.lockfile = lockfile;
            let owner = publisher(session.captured().snapshot().id, 0, &unit);
            allow_frontend(&owner, &unit);
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let submission = driver
                .submit_captured(session.clone(), &ids, &snapshots, current)
                .unwrap();
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
            assert_eq!(submission.build_plan.as_ref().unwrap(), &plan);
            let (inputs, phase_input) = native_disk_lookup_context(&submission, &ids);
            assert_eq!(inputs.validate_native_lock_content(), Ok(()));
            let request = inputs
                .prepare_native_disk_cache_request(&phase_input, &VerifierPolicy::release())
                .unwrap();
            let lock_input = request
                .input_hashes
                .iter()
                .find(|entry| entry.name == "captured-lockfile")
                .unwrap();
            assert_eq!(
                lock_input.domain,
                "mizar-driver/captured-lockfile-identity/v1"
            );
            assert_eq!(lock_input.digest, digest);
            for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
                assert!(matches!(
                    driver
                        .registry()
                        .cache_key_for_phase(phase, &phase_input)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey { .. }
                ));
            }
            let changed_text =
                format!("schema_version = 1\ntoolchain_version = \"9.8.7-rc.1+other\"\n{body}");
            let changed_plan = produce_build_plan(
                input.plan_request.clone(),
                packages,
                parse_lockfile(&changed_text).unwrap(),
            )
            .unwrap();
            let mut expected = plan.clone();
            expected.lockfile = changed_plan.lockfile.clone();
            assert_eq!(changed_plan, expected);
            assert_ne!(lockfile_content_hash(&changed_plan.lockfile), digest);
            assert!(
                SourceLoadInputs {
                    build_plan: &changed_plan,
                    ..inputs
                }
                .validate_native_lock_content()
                .is_err()
            );
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.diagnostics.is_empty());
            let results = run.phase_results.values().flatten().collect::<Vec<_>>();
            assert!(results.iter().all(|result| result.diagnostics.is_empty()));
            let refs = results
                .iter()
                .flat_map(|result| &result.output_refs)
                .collect::<Vec<_>>();
            assert_eq!(refs.len(), 2);
            let source_ref = *refs
                .iter()
                .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap();
            let source_handle = owner
                .storage()
                .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                .unwrap();
            let source = owner.storage().get(&source_handle).unwrap();
            let frontend_ref = *refs
                .iter()
                .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
                .unwrap();
            let frontend_handle = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    frontend_ref,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let frontend = owner.storage().get(&frontend_handle).unwrap();
            assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
            assert!(source.loading_map.is_some());
            assert_eq!(
                source.source_id,
                session.captured().snapshot().source_versions[0].source_id
            );
            let actual = (
                submission.clone(),
                request.clone(),
                source.as_ref().clone(),
                frontend.as_ref().clone(),
                owner.storage().side_tables(&source_handle).unwrap(),
                owner.storage().side_tables(&frontend_handle).unwrap(),
            );
            if let Some(expected) = &baseline {
                assert_eq!(&actual, expected);
            } else {
                baseline = Some(actual);
            }
        }
        let (_, request, source, _, _, _) = baseline.unwrap();
        if let Some((prior_digest, prior_snapshot, prior_source, prior_request)) = &previous {
            assert_ne!(digest, *prior_digest);
            assert_ne!(session.captured().snapshot().id, *prior_snapshot);
            assert_eq!(source.source_hash, *prior_source);
            let mut normalized = request.clone();
            normalized
                .input_hashes
                .iter_mut()
                .find(|entry| entry.name == "captured-lockfile")
                .unwrap()
                .digest = *prior_digest;
            assert_eq!(&normalized, prior_request);
        }
        previous = Some((
            digest,
            session.captured().snapshot().id,
            source.source_hash,
            request,
        ));
    }
    for value in ["true", "\"^4.5.6\""] {
        let invalid = format!("schema_version = 1\ntoolchain_version = {value}\n{body}");
        fs::write(fixture.root.join("mizar.lock"), &invalid).unwrap();
        let (_, input) = fixture.submit_request(vec![], vec![]);
        assert_eq!(
            read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap_err(),
            parse_lockfile(&invalid).unwrap_err().to_string()
        );
    }
}
