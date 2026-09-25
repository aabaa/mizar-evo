use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mizar_artifact::{
    module_summary::ModuleSummaryIdentity,
    store::{CanonicalJson, canonical_json_string},
};
use mizar_build::{
    cancel::{CancellationGeneration, CancellationReason, CancellationToken},
    module_index::{
        DependencyArtifactIndex, DependencyModuleSummaryRef, ModuleIndex, ModuleIndexLocation,
        StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage, build_module_index,
    },
    planner::{
        BuildPlan, DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile,
        parse_package_manifest, produce_build_plan,
    },
    task_graph::{ModuleDependencyOverlay, PipelinePhase, TaskKind, WorkUnit},
};
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue, DiagnosticPrimaryLocation, FailureCategory,
        PipelinePhase as DiagnosticPhase,
    },
    sink::{DiagnosticProducerScope, DiagnosticSink},
};
use mizar_driver::{
    driver::{CompilerDriver, DriverSubmissionStatus, DriverSubmitInput},
    registry::{
        PhaseCacheIntent, PhaseExecutionResources, PhaseInput, PhaseRegistryBuilder, PhaseStatus,
        SourceLoadInputs,
    },
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
        SourceInputSet, VerifierConfigInput,
    },
};
use mizar_frontend::{
    cache_key::{SOURCE_UNIT_CACHE_KEY_VERSION, SourceUnitCacheKey},
    lexical_env::{
        FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
        build_active_lexical_environment,
    },
    orchestration::FrontendOutput,
    parsing::{MizarParserSeam, ParserSeam},
    preprocess::preprocess,
    source::SourceUnit,
    source::register_source_unit,
    span_bridge::SpanBridge,
};
use mizar_ir::{
    dispatch_input::PhaseDispatchInputBundle,
    identity::{
        OutputKind, PipelinePhase as IrPipelinePhase, SnapshotHandleRegistry,
        WorkUnit as IrWorkUnit,
    },
    publisher::{AllowedWorkUnit, PhaseOutputPublisher},
    storage::{AnyPhaseOutputRef, IrStorageService, StoragePlacement, StoragePolicy},
};
use mizar_resolve::{
    declarations::DeclarationShellCollector,
    env::NamespacePath,
    imports::{ImportPathCandidate, ImportPathResolver},
    module_index::ModuleIndexInput,
    resolved_ast::ModuleId as ResolverModuleId,
    symbols::SignatureProjectionExtractor,
};
use mizar_session::{
    BuildRequestId, BuildSessionId, BuildSnapshot, BuildSnapshotId, DependencyArtifactRef,
    DiskSourceLoader, Edition, Hash, IdError, InMemorySessionIdAllocator, LineMap, LoadingMap,
    ModulePath, PackageId, SessionIdAllocator, SnapshotLeaseId, SnapshotRegistry, SourceAnchor,
    SourceId, SourceInput, SourceMapId, SourceOrigin, SourceOriginInput, SourceVersion,
    ToolchainInfo, WorkspaceRoot, normalize_path,
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    bytes: Vec<u8>,
    version: SourceVersion,
    line_map: LineMap,
    loading_map: Option<LoadingMap>,
}

impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "mizar-driver-source-load-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("alpha/src")).unwrap();
        fs::write(root.join("alpha/src/main.miz"), bytes).unwrap();
        let package_root = root.join("alpha");
        let input = SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("main"),
            normalized_path: normalize_path(&package_root, Path::new("src/main.miz")).unwrap(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: PathBuf::from("src/main.miz"),
            },
        };
        let provisional = snapshot_id(0x51);
        let loaded = mizar_session::SourceLoader::load(
            &DiskSourceLoader::new(&package_root),
            provisional,
            input,
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
        let line_map = loaded.line_map.clone();
        let loading_map = loaded.loading_map.clone();
        let version = SourceVersion {
            source_id: loaded.source_id,
            package_id: loaded.package_id,
            module_path: loaded.module_path,
            normalized_path: loaded.normalized_path,
            source_hash: loaded.source_hash,
            edition: loaded.edition,
            origin: loaded.origin,
        };
        Self {
            root,
            bytes: bytes.to_vec(),
            version,
            line_map,
            loading_map,
        }
    }

    fn write(&self, bytes: &[u8]) {
        fs::write(self.root.join("alpha/src/main.miz"), bytes).unwrap();
    }

    fn submit(
        &self,
        ids: &InMemorySessionIdAllocator,
        snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>,
    ) -> mizar_driver::driver::BuildSubmission {
        self.submit_with_dependencies(ids, snapshots, Vec::new(), Vec::new())
    }

    fn submit_with_dependencies(
        &self,
        ids: &InMemorySessionIdAllocator,
        snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>,
        captured_artifacts: Vec<DependencyArtifactRef>,
        indexed_artifacts: Vec<DependencyArtifactIndex>,
    ) -> mizar_driver::driver::BuildSubmission {
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        let registry = builder.build().unwrap();
        let mut driver = CompilerDriver::new(registry);
        let has_dependency = !indexed_artifacts.is_empty();
        let mut input = DriverSubmitInput::new(
            PlanRequest {
                workspace_root: WorkspaceRoot::new(self.root.to_string_lossy().into_owned()),
                dependency_selection: DependencySelection::Normal,
                toolchain: ToolchainInfo::new("mizar-evo-test"),
            },
            vec![WorkspacePackage {
        member_path: "alpha".to_owned(),
        manifest: parse_package_manifest(if has_dependency {
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dependencies]\ndep = \"1.0.0\"\n"
        } else {
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n"
        }).unwrap(),
    }],
            parse_lockfile(if has_dependency {
                "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n"
            } else {
                "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n"
            }).unwrap(),
            StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            }]),
        );
        input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
        input.dependency_artifacts = indexed_artifacts;
        let request = BuildRequestDraft {
            lane: BuildLaneId::new(1),
            origin: BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation::default(),
            }),
            generation: BuildRequestGeneration::new(0),
            workspace_root: WorkspaceRoot::new(self.root.to_string_lossy().into_owned()),
            profile: BuildProfile::new("check"),
            targets: BuildTargets::default(),
            source_inputs: SourceInputSet {
                versions: vec![self.version.clone()],
            },
            dependency_inputs: DependencyInputSet::new(
                captured_artifacts,
                hash(1),
                ToolchainInfo::new("mizar-evo-test"),
            ),
            verifier_config: VerifierConfigInput::new(hash(2)),
        };
        let submission = driver.submit(request, ids, snapshots, input).unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        submission
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn source_load_publishes_resident_and_blob_with_captured_identity_and_maps() {
    for threshold in [usize::MAX, 3] {
        let fixture = Fixture::new(b"\xef\xbb\xbfalpha\r\nbeta\r\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (result, publisher) = execute(&submission, &ids, threshold);
        assert_eq!(result.status, PhaseStatus::Complete);
        assert_eq!(result.output_refs.len(), 1);
        let output = &result.output_refs[0];
        if threshold == usize::MAX {
            assert!(matches!(output.placement(), StoragePlacement::Resident));
        } else {
            assert!(matches!(output.placement(), StoragePlacement::Blob { .. }));
        }
        let typed = publisher
            .storage()
            .typed_handle::<SourceUnit>(output, &OutputKind::new("SourceUnit"))
            .unwrap();
        let source = publisher.storage().get(&typed).unwrap();
        assert_eq!(source.package_id, fixture.version.package_id);
        assert_eq!(source.module_path, fixture.version.module_path);
        assert_eq!(source.normalized_path, fixture.version.normalized_path);
        assert_eq!(source.edition, fixture.version.edition);
        assert_eq!(source.origin, fixture.version.origin);
        assert_eq!(source.file_path, PathBuf::from("src/main.miz"));
        assert_eq!(source.source_id, fixture.version.source_id);
        assert_eq!(source.source_hash, fixture.version.source_hash);
        assert_eq!(source.line_map, fixture.line_map);
        assert_eq!(source.loading_map, fixture.loading_map);
        assert_eq!(source.source_text.as_ref(), "alpha\nbeta\n");
    }
}

#[test]
fn frontend_publishes_only_clean_loaded_source_with_parent_and_storage_maps() {
    for threshold in [usize::MAX, 3] {
        let fixture = Fixture::new(b"\xef\xbb\xbfdefinition\r\nend;\r\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, threshold);
        assert_eq!(source.status, PhaseStatus::Complete);
        fixture.write(b"replaced after source publication");

        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Complete);
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.output_refs.len(), 1);
        let output = &result.output_refs[0];
        assert_eq!(output.phase(), &IrPipelinePhase::new("Frontend"));
        assert_eq!(output.output_kind(), &OutputKind::new("FrontendOutput"));
        assert_eq!(output.snapshot(), submission.session.captured.snapshot.id);
        assert!(
            matches!(
                output.placement(),
                StoragePlacement::Resident if threshold == usize::MAX
            ) || matches!(
                output.placement(),
                StoragePlacement::Blob { .. } if threshold != usize::MAX
            )
        );
        let lineage = publisher
            .registry()
            .output_lineage(output.output())
            .unwrap();
        assert_eq!(lineage.parents, vec![source.output_refs[0].output()]);
        assert!(
            lineage
                .named_input_hashes
                .iter()
                .any(|input| input.name == "active-lexical-environment")
        );
        assert!(
            lineage
                .named_input_hashes
                .iter()
                .any(|input| input.name == "tokens")
        );
        let tables = publisher.storage().side_tables_by_ref(output).unwrap();
        assert_eq!(tables.source_maps.len(), 1);
        assert_eq!(tables.source_maps[0].domain, "frontend-storage");
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = publisher.storage().get(&typed).unwrap();
        assert_eq!(loaded.source.source_id, fixture.version.source_id);
        assert_eq!(loaded.source.source_text.as_ref(), "definition\nend;\n");
        assert!(loaded.ast.is_some());
        assert!(loaded.diagnostics.is_empty());
        let wrong_parent = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![output.clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(wrong_parent.status, PhaseStatus::Blocking);
        assert!(wrong_parent.output_refs.is_empty());
    }
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
fn frontend_recovery_and_unrecoverable_input_emit_diagnostics_without_output() {
    for (text, status, code) in [
        (b"definition\n".as_slice(), PhaseStatus::Recoverable, 10),
        (b"end".as_slice(), PhaseStatus::Fatal, 52),
    ] {
        let fixture = Fixture::new(text);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, status);
        assert!(result.output_refs.is_empty());
        assert_eq!(result.diagnostics.len(), 1);
        assert!(
            result.diagnostics[0]
                .drafts()
                .iter()
                .any(|draft| draft.code().number() == code)
        );
    }
}

#[test]
fn frontend_diagnostic_only_import_reports_five_path_failures_from_live_source() {
    for (text, code, dependency_modules, needs_dependency, paths, primary_text) in [
        (
            "import mml.no_such;\ndefinition\nend;\n",
            220,
            &[][..],
            false,
            &["mml.no_such"][..],
            "mml.no_such",
        ),
        (
            "import dep.missing;\ndefinition\nend;\n",
            221,
            &["present"][..],
            true,
            &["dep.missing"][..],
            "dep.missing",
        ),
        (
            "import ..common;\ndefinition\nend;\n",
            222,
            &[][..],
            false,
            &["..common"][..],
            "..common",
        ),
        (
            "import dep.core as shared, dep.core as shared, dep.other as shared;\ndefinition\nend;\n",
            223,
            &["core", "other"][..],
            true,
            &["dep.core", "dep.core", "dep.other"][..],
            "shared",
        ),
        (
            "import dep.core as mml;\ndefinition\nend;\n",
            224,
            &["core"][..],
            true,
            &["dep.core"][..],
            "mml",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let references = dependency_modules
            .iter()
            .enumerate()
            .map(|(ordinal, path)| {
                let artifact = format!("dep/{path}.summary.json");
                let digest = hash(0x60 + ordinal as u8);
                (
                    DependencyModuleSummaryRef {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("dep"),
                            ModulePath::new(*path),
                        ),
                        artifact: artifact.clone(),
                        content_hash: digest,
                    },
                    DependencyArtifactRef::new(artifact, digest),
                )
            })
            .collect::<Vec<_>>();
        let mut dependency_hashes = references
            .iter()
            .map(|(indexed, _)| indexed.content_hash)
            .collect::<Vec<_>>();
        dependency_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        let submission = fixture.submit_with_dependencies(
            &ids,
            &snapshots,
            references
                .iter()
                .map(|(_, captured)| captured.clone())
                .collect(),
            if needs_dependency {
                vec![DependencyArtifactIndex::new(
                    PackageId::new("dep"),
                    Vec::new(),
                    references.into_iter().map(|(indexed, _)| indexed).collect(),
                )]
            } else {
                Vec::new()
            },
        );
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete, "{text}");
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), dependency_hashes),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Recoverable, "{text}");
        assert!(result.output_refs.is_empty(), "{text}");
        assert_eq!(result.diagnostics.len(), 2, "{text}");
        let frontend = &result.diagnostics[0];
        assert_eq!(frontend.scope().phase(), DiagnosticPhase::Frontend);
        assert_eq!(frontend.drafts().len(), paths.len());
        assert!(
            frontend
                .drafts()
                .iter()
                .all(|draft| draft.code().number() == 22)
        );
        let resolver = &result.diagnostics[1];
        assert_eq!(resolver.scope().phase(), DiagnosticPhase::Resolver);
        assert_eq!(resolver.drafts().len(), paths.len(), "{text}");
        assert!(resolver.drafts().iter().all(|draft| {
            draft.code().number() == code
                && draft.phase() == DiagnosticPhase::Resolver
                && draft.category() == FailureCategory::ResolveError
        }));
        for (ordinal, draft) in resolver.drafts().iter().enumerate() {
            assert_eq!(
                draft.stable_detail_key(),
                mizar_diagnostics::registry::DiagnosticRegistry::builtin()
                    .lookup(draft.code())
                    .unwrap()
                    .semantic_name
            );
            let details = draft.details().entries();
            assert_eq!(
                details.get("import.source_module"),
                Some(&DiagnosticDetailValue::List(vec![
                    DiagnosticDetailValue::String("alpha".to_owned()),
                    DiagnosticDetailValue::String("main".to_owned()),
                ])),
                "{text}"
            );
            assert_eq!(
                details.get("import.ordinal"),
                Some(&DiagnosticDetailValue::Integer(ordinal as i64)),
                "{text}"
            );
            assert_eq!(
                details.get("import.path"),
                Some(&DiagnosticDetailValue::String(paths[ordinal].to_owned()))
            );
            let DiagnosticPrimaryLocation::Span(primary) = draft.primary_location() else {
                panic!("semantic failure has a loaded-source primary span");
            };
            let SourceAnchor::Range(primary_range) = primary.anchor() else {
                panic!("semantic failure retains Range shape");
            };
            assert_eq!(primary_range.source_id, fixture.version.source_id);
            assert_eq!(&text[primary_range.start..primary_range.end], primary_text);
            if code == 223 {
                assert_eq!(
                    details.get("import.alias"),
                    Some(&DiagnosticDetailValue::String("shared".to_owned()))
                );
                assert_eq!(
                    details.get("import.target"),
                    Some(&DiagnosticDetailValue::List(vec![
                        DiagnosticDetailValue::String("dep".to_owned()),
                        DiagnosticDetailValue::String(
                            if ordinal == 2 { "other" } else { "core" }.to_owned(),
                        ),
                    ]))
                );
                let alias_offsets = text
                    .match_indices("shared")
                    .map(|(start, _)| start)
                    .collect::<Vec<_>>();
                assert_eq!(primary.range().start, alias_offsets[ordinal]);
                assert_eq!(
                    draft
                        .secondary_spans()
                        .iter()
                        .map(|span| match span.anchor() {
                            SourceAnchor::Range(range) => *range,
                            _ => panic!("peer retains Range shape"),
                        })
                        .collect::<Vec<_>>(),
                    alias_offsets
                        .into_iter()
                        .enumerate()
                        .filter_map(|(peer, start)| {
                            (peer != ordinal).then_some(mizar_session::SourceRange {
                                source_id: fixture.version.source_id,
                                start,
                                end: start + "shared".len(),
                            })
                        })
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn frontend_diagnostic_only_import_keeps_distinct_branch_members() {
    let text = "import dep.{missing, absent};\ndefinition\nend;\n";
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let digest = hash(0x60);
    let artifact = "dep/present.summary.json";
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![DependencyArtifactRef::new(artifact, digest)],
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            vec![DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new("present"),
                ),
                artifact: artifact.to_owned(),
                content_hash: digest,
            }],
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![digest]),
        Vec::new(),
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 2);
    let [first, second] = result.diagnostics[1].drafts() else {
        panic!("one E0221 draft per branch member");
    };
    for (ordinal, (draft, member)) in [(first, "missing"), (second, "absent")]
        .into_iter()
        .enumerate()
    {
        assert_eq!(draft.code().number(), 221);
        let DiagnosticPrimaryLocation::Span(primary) = draft.primary_location() else {
            panic!("branch member source span");
        };
        let SourceAnchor::Range(member_range) = primary.anchor() else {
            panic!("retained range anchor");
        };
        assert_eq!(&text[member_range.start..member_range.end], member);
        assert_eq!(draft.secondary_spans().len(), 1);
        assert_eq!(
            &text[draft.secondary_spans()[0].range().start..draft.secondary_spans()[0].range().end],
            "dep"
        );
        let details = draft.details().entries();
        assert_eq!(
            details.get("import.branch_member"),
            Some(&DiagnosticDetailValue::Source(*member_range))
        );
        assert_eq!(
            details.get("import.path"),
            Some(&DiagnosticDetailValue::String(format!("dep.{member}")))
        );
        assert_eq!(
            details.get("import.ordinal"),
            Some(&DiagnosticDetailValue::Integer(ordinal as i64))
        );
    }
}

#[test]
fn frontend_mixed_or_malformed_import_diagnostics_do_not_continue_resolution() {
    for text in [
        "import std., pkg.math as ;\ndefinition\nend;\n",
        "import mml.no_such;\ndefinition\n",
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert!(matches!(
            result.status,
            PhaseStatus::Recoverable | PhaseStatus::Fatal
        ));
        assert!(result.output_refs.is_empty());
        assert_eq!(result.diagnostics.len(), 1, "{text}");
        let drafts = result.diagnostics[0].drafts();
        assert_eq!(
            result.diagnostics[0].scope().phase(),
            DiagnosticPhase::Frontend
        );
        assert!(
            drafts.iter().any(|draft| draft.code().number() == 22),
            "{text}"
        );
        assert!(
            drafts.iter().any(|draft| draft.code().number() != 22),
            "{text}"
        );
    }

    let fixture = Fixture::new(b"import mml.no_such, dep.core;\ndefinition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let artifact = "dep/core.summary.json";
    let digest = hash(0x60);
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![DependencyArtifactRef::new(artifact, digest)],
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            vec![DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new("core"),
                ),
                artifact: artifact.to_owned(),
                content_hash: digest,
            }],
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let artifact_root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![digest]),
        vec![(PackageId::new("dep"), artifact_root)],
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    let codes = result.diagnostics[0]
        .drafts()
        .iter()
        .map(|draft| draft.code().number())
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&22),
        "unresolved path keeps E0022: {codes:?}"
    );
    assert!(
        codes.contains(&23),
        "missing summary keeps E0023: {codes:?}"
    );
}

#[test]
fn frontend_same_normalized_source_separates_semantic_and_storage_identity() {
    let mut hashes = Vec::new();
    for bytes in [
        b"\xef\xbb\xbfdefinition\r\nend;\r\n".as_slice(),
        b"definition\nend;\n".as_slice(),
    ] {
        let fixture = Fixture::new(bytes);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Complete);
        hashes.push((
            result.output_refs[0].content_hash(),
            result.output_refs[0].side_table_hash(),
        ));
    }
    assert_eq!(hashes[0].0, hashes[1].0);
    assert_ne!(hashes[0].1, hashes[1].1);
}

#[test]
fn frontend_rejects_missing_wrong_or_mismatched_parent_identities() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let source_ref = source.output_refs[0].clone();
    for (parents, input_hash, dependencies) in [
        (Vec::new(), source_key(&fixture.version), Vec::new()),
        (vec![source_ref.clone()], hash(7), Vec::new()),
        (
            vec![source_ref.clone()],
            source_key(&fixture.version),
            vec![hash(8)],
        ),
    ] {
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            parents,
            (input_hash, dependencies),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
    }
}

#[test]
fn frontend_imports_require_one_captured_summary_artifact_identity() {
    use mizar_artifact::{
        module_summary::{
            ModuleLexicalSummary, ModuleSummary, ModuleSummaryIdentity, current_schema_version,
            module_summary_json,
        },
        store::{PublishedArtifactPath, artifact_hash_domain, write_published_artifact},
    };

    let fixture = Fixture::new(b"import dep.core;\ndefinition\nend;\n");
    let root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&root).unwrap();
    let artifact = "dep/core.summary.json";
    let mut summary = ModuleSummary {
        schema_version: current_schema_version(),
        module: ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: "core".to_owned(),
            language_edition: "2025".to_owned(),
        },
        source_hash: hash(9),
        interface_hash: hash(8),
        exported_symbols: Vec::new(),
        exported_labels: Vec::new(),
        lexical_summary: ModuleLexicalSummary {
            schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
            fingerprint: None,
            contributions: Vec::new(),
        },
        reexports: Vec::new(),
        dependency_interfaces: Vec::new(),
    };
    summary.refresh_interface_hash().unwrap();
    let content_hash = write_published_artifact(
        &root,
        &PublishedArtifactPath::new(artifact).unwrap(),
        &module_summary_json(&summary).unwrap(),
        &artifact_hash_domain(
            mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
            summary.schema_version,
        ),
        &[],
    )
    .unwrap()
    .artifact_hash;
    let indexed = DependencyArtifactIndex::new(
        PackageId::new("dep"),
        Vec::new(),
        vec![DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new("core"),
            ),
            artifact: artifact.to_owned(),
            content_hash,
        }],
    );
    let matching = DependencyArtifactRef::new(artifact, content_hash);
    for (captured, expected) in [
        (vec![matching.clone()], PhaseStatus::Complete),
        (Vec::new(), PhaseStatus::Blocking),
        (
            vec![DependencyArtifactRef::new(artifact, hash(0x98))],
            PhaseStatus::Blocking,
        ),
        (
            vec![matching.clone(), matching.clone()],
            PhaseStatus::Blocking,
        ),
        (
            vec![
                matching.clone(),
                DependencyArtifactRef::new(artifact, hash(0x98)),
            ],
            PhaseStatus::Complete,
        ),
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission =
            fixture.submit_with_dependencies(&ids, &snapshots, captured, vec![indexed.clone()]);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), vec![content_hash]),
            vec![(PackageId::new("dep"), root.clone())],
            None,
        );
        assert_eq!(result.status, expected);
        assert_eq!(
            result.output_refs.len(),
            usize::from(expected == PhaseStatus::Complete)
        );
        if expected == PhaseStatus::Complete {
            assert!(result.diagnostics.is_empty());
            let typed = publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &result.output_refs[0],
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded = publisher.storage().get(&typed).unwrap();
            let candidates = ImportPathCandidate::from_surface_ast(
                loaded.ast.as_ref().expect("sealed import AST"),
            )
            .expect("parsed import candidates");
            let [candidate] = candidates.as_slice() else {
                panic!("one parsed import candidate");
            };
            assert_eq!(candidate.components(), ["dep", "core"]);
            assert_eq!(candidate.ordinal(), 0);
            assert_eq!(candidate.range().source_id, loaded.source.source_id);
            assert_eq!(
                &loaded.source.source_text[candidate.range().start..candidate.range().end],
                "dep.core"
            );
            let current = ResolverModuleId::new(
                loaded.source.package_id.clone(),
                loaded.source.module_path.clone(),
            );
            let resolution = ImportPathResolver::new(ModuleIndexInput::new(
                submission.module_index.as_ref().unwrap(),
            ))
            .resolve(&current, &candidates);
            assert!(resolution.unresolved().is_empty());
            let [resolved] = resolution.resolved() else {
                panic!("one resolved import");
            };
            assert_eq!(
                resolved.target(),
                &ResolverModuleId::new(PackageId::new("dep"), ModulePath::new("core"))
            );
            assert_eq!(resolved.range(), candidate.range());
            assert_eq!(resolved.ordinal(), candidate.ordinal());
            let lineage = publisher
                .registry()
                .output_lineage(result.output_refs[0].output())
                .unwrap();
            assert!(
                lineage
                    .named_input_hashes
                    .iter()
                    .any(|input| { input.name == "dependency.0" && input.digest == content_hash })
            );
        }
    }
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![matching.clone()],
        vec![indexed.clone()],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    for mutation in 0..4 {
        let mut changed = submission.module_index.as_ref().unwrap().clone();
        let dependency_module = changed
            .modules
            .iter()
            .position(|entry| entry.module.package.as_str() == "dep")
            .unwrap();
        match mutation {
            0 => {
                let ModuleIndexLocation::DependencySummary { artifact, .. } =
                    &mut changed.modules[dependency_module].location
                else {
                    unreachable!()
                };
                *artifact = "dep/other.summary.json".to_owned();
            }
            1 => changed.dependency_summaries[0].artifact = "dep/other.summary.json".to_owned(),
            2 => changed
                .dependency_summaries
                .push(changed.dependency_summaries[0].clone()),
            3 => changed
                .modules
                .push(changed.modules[dependency_module].clone()),
            _ => unreachable!(),
        }
        let dependencies = changed
            .dependency_summaries
            .iter()
            .map(|reference| reference.content_hash)
            .collect();
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), dependencies),
            vec![(PackageId::new("dep"), root.clone())],
            Some(&changed),
        );
        assert_eq!(
            result.status,
            PhaseStatus::Blocking,
            "index mutation {mutation}"
        );
        assert!(result.output_refs.is_empty());
    }
    fs::remove_file(root.join(artifact)).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission =
        fixture.submit_with_dependencies(&ids, &snapshots, vec![matching], vec![indexed]);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let missing_summary = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![content_hash]),
        vec![(PackageId::new("dep"), root)],
        None,
    );
    assert_eq!(missing_summary.status, PhaseStatus::Recoverable);
    assert!(missing_summary.output_refs.is_empty());
    assert!(
        missing_summary.diagnostics[0]
            .drafts()
            .iter()
            .any(|draft| draft.code().number() == 23)
    );
}

#[test]
fn frontend_imported_symbol_conflict_uses_real_captured_summaries() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, LexicalContributionSummary, ModuleLexicalSummary, ModuleSummary,
            ModuleSummaryIdentity, SourceRangeSummary, current_schema_version, module_summary_json,
        },
        store::{
            CanonicalJson, PublishedArtifactPath, artifact_hash_domain, canonical_json_string,
            write_published_artifact,
        },
    };
    use mizar_frontend::lexical_env::{
        ExportRank, ExportedSymbolShape, ModuleId as LexerModuleId, SymbolId, UserSymbolArity,
        UserSymbolKind,
    };

    let fixture = Fixture::new(b"import dep.core, dep.other;\ndefinition\nend;\n");
    let root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&root).unwrap();
    let mut indexed = Vec::new();
    let mut captured = Vec::new();
    for module_path in ["core", "other"] {
        let module = ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2025".to_owned(),
        };
        let origin_id = "symbol:combine";
        let shape = ExportedSymbolShape {
            spelling: "combine".to_owned(),
            symbol_id: SymbolId::new(canonical_json_string(&CanonicalJson::array([
                module.canonical_json().unwrap(),
                CanonicalJson::string(origin_id),
            ]))),
            source_module: LexerModuleId::new(canonical_json_string(
                &module.canonical_json().unwrap(),
            )),
            export_rank: ExportRank::new(0),
            kind: UserSymbolKind::Functor,
            arity: UserSymbolArity::exact(2),
            operator: None,
        };
        let mut summary = ModuleSummary {
            schema_version: current_schema_version(),
            module,
            source_hash: hash(9),
            interface_hash: hash(8),
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: origin_id.to_owned(),
                fully_qualified_name: format!("{module_path}.combine"),
                namespace_path: vec![module_path.to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "functor".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 1,
                },
                rendered_signature: format!("functor {module_path}.combine"),
                interface_fingerprint: hash(8),
                proof_status: None,
            }],
            exported_labels: Vec::new(),
            lexical_summary: ModuleLexicalSummary {
                schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
                fingerprint: None,
                contributions: vec![LexicalContributionSummary {
                    kind: "exported-symbol".to_owned(),
                    key: origin_id.to_owned(),
                    payload: String::from_utf8(shape.canonical_bytes().unwrap()).unwrap(),
                }],
            },
            reexports: Vec::new(),
            dependency_interfaces: Vec::new(),
        };
        summary.refresh_interface_hash().unwrap();
        let artifact = format!("dep/{module_path}.summary.json");
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
        indexed.push(DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new(module_path),
            ),
            artifact: artifact.clone(),
            content_hash,
        });
        captured.push(DependencyArtifactRef::new(artifact, content_hash));
    }
    let mut dependencies = captured
        .iter()
        .map(|item| item.content_hash)
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        captured,
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            indexed,
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), dependencies),
        vec![(PackageId::new("dep"), root)],
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert!(
        result.diagnostics[0]
            .drafts()
            .iter()
            .any(|draft| draft.code().number() == 24)
    );
}

#[test]
fn frontend_requires_current_resources_and_scopes_cancellation() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let snapshot = submission.session.captured.snapshot.id;
    let task = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let parent = mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
        &publisher,
        snapshot,
        source.output_refs[0].clone(),
    )
    .unwrap();
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(Vec::new());
    let registry = builder.build().unwrap();
    let sink = || {
        DiagnosticSink::new(DiagnosticProducerScope::new(
            DiagnosticPhase::Frontend,
            snapshot,
            "frontend.run_loaded",
        ))
    };
    let run = |resources: PhaseExecutionResources<'_>| {
        registry
            .execute_phase_with_resources(
                PipelinePhase::Frontend,
                PhaseInput::new(
                    task.unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_key(&fixture.version),
                        Vec::new(),
                        vec![parent.clone()],
                    )
                    .unwrap(),
                ),
                resources,
            )
            .unwrap()
            .result
    };
    for resources in [
        PhaseExecutionResources {
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(sink()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                DiagnosticPhase::SourceLoad,
                snapshot,
                "wrong.phase",
            ))),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
    ] {
        let result = run(resources);
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
    }
    let cancelled = run(PhaseExecutionResources {
        cancellation: Some(CancellationToken {
            snapshot,
            generation: CancellationGeneration::new(1),
            reason: CancellationReason::ExplicitRequest,
        }),
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
    });
    assert_eq!(cancelled.status, PhaseStatus::Cancelled);
    assert!(cancelled.output_refs.is_empty());
    let foreign = run(PhaseExecutionResources {
        cancellation: Some(CancellationToken {
            snapshot: snapshot_id(0xaa),
            generation: CancellationGeneration::new(1),
            reason: CancellationReason::ExplicitRequest,
        }),
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
    });
    assert_eq!(foreign.status, PhaseStatus::Blocking);
    assert!(foreign.output_refs.is_empty());
    let denied = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(denied.status, PhaseStatus::Blocking);
    assert!(denied.output_refs.is_empty());
    publisher.mark_obsolete(snapshot).unwrap();
    let stale = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(publisher),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(stale.status, PhaseStatus::Blocking);
    assert!(stale.output_refs.is_empty());
}

#[test]
fn frontend_real_diagnostic_producer_matrix() {
    let cases: &[(&str, &[u16])] = &[
        ("definition\rend;", &[13, 21, 29, 38]),
        ("α\n::=\n", &[14, 15]),
        ("import ;\ndefinition\nend;", &[16, 43]),
        (
            "import std., pkg.math as ;\ndefinition\nend;",
            &[17, 18, 22, 41, 51],
        ),
        ("import std.core\ndefinition\nend;", &[19]),
        ("import @;\ndefinition\nend;", &[20]),
        ("let , x be set;\nfor + y holds thesis;", &[30, 31, 34, 47]),
        ("definition\nlet x,x be set;\nend;", &[32]),
        ("end;\ndefinition\nlet x be set;", &[33, 10, 52]),
        ("\"abc\"", &[35]),
        ("infix_operator(\"bad\\n\",left,80);", &[2, 42]),
        ("theorem T: x = y &;\n", &[48]),
        ("private ;", &[45]),
        ("export 123;", &[44]),
        ("theorem T: thesis by ;", &[49]),
        ("@[ label theorem T: thesis;", &[50]),
        ("theorem T: thesis;\nexport foo;", &[51]),
        ("definition\nlet x be ;\nend;", &[46]),
        (
            "func Plus: +(x,y) -> set; infix_operator(\"+\",left,80); theorem T: a + ;",
            &[39],
        ),
        (
            "func Plus: +(x,y) -> set; infix_operator(\"+\",none,80); theorem T: a + b + c;",
            &[40],
        ),
    ];
    for &(text, expected) in cases {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert!(matches!(
            result.status,
            PhaseStatus::Recoverable | PhaseStatus::Fatal
        ));
        assert!(result.output_refs.is_empty());
        let actual = result
            .diagnostics
            .iter()
            .flat_map(|batch| batch.drafts())
            .map(|draft| draft.code().number())
            .collect::<Vec<_>>();
        for &code in expected {
            assert!(
                actual.contains(&code),
                "{text:?} must emit E{code:04}, got {actual:?}"
            );
        }
    }
}

#[test]
fn source_load_same_text_with_distinct_raw_maps_keeps_semantic_key_and_storage_identity() {
    let first = Fixture::new(b"\xef\xbb\xbfalpha\r\nbeta");
    let second = Fixture::new(b"alpha\nbeta");
    assert_eq!(first.version.source_hash, second.version.source_hash);
    let first_key = source_key(&first.version);
    let second_key = source_key(&second.version);
    assert_eq!(first_key, second_key);
    assert_ne!(first.bytes, second.bytes);
    let first_ids = InMemorySessionIdAllocator::new();
    let first_snapshots = SnapshotRegistry::new();
    let first_submission = first.submit(&first_ids, &first_snapshots);
    let second_ids = InMemorySessionIdAllocator::new();
    let second_snapshots = SnapshotRegistry::new();
    let second_submission = second.submit(&second_ids, &second_snapshots);
    let (first_result, _) = execute(&first_submission, &first_ids, usize::MAX);
    let (second_result, _) = execute(&second_submission, &second_ids, usize::MAX);
    assert_eq!(
        first_result.output_refs[0].content_hash(),
        second_result.output_refs[0].content_hash()
    );
    assert_ne!(
        first_result.output_refs[0].side_table_hash(),
        second_result.output_refs[0].side_table_hash()
    );
}

#[test]
fn source_load_changed_disk_bytes_is_blocking_without_publication() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    fixture.write(b"changed");
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
}

#[test]
fn source_load_real_errors_allocate_shared_source_location_diagnostics() {
    let cases: &[(Option<&[u8]>, u16)] = &[(Some(&[0xff]), 601), (None, 602)];
    for (replacement, code) in cases {
        let fixture = Fixture::new(b"alpha");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        if let Some(bytes) = replacement {
            fixture.write(bytes);
        } else {
            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        }
        let (result, _) = execute(&submission, &ids, usize::MAX);
        assert_source_error(
            &result,
            &fixture.version,
            *code,
            if *code == 601 {
                "source.invalid_utf8"
            } else {
                "source.unreadable_file"
            },
        );
    }
}

#[cfg(unix)]
#[test]
fn source_load_symlink_escape_is_an_allocated_outside_root_diagnostic() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new(b"alpha");
    let outside = fixture.root.join("outside.miz");
    fs::write(&outside, b"outside").unwrap();
    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
    symlink(&outside, fixture.root.join("alpha/src/main.miz")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_source_error(
        &result,
        &fixture.version,
        603,
        "source.outside_package_root",
    );
}

#[test]
fn source_load_allocator_failure_is_generic_source_diagnostic() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let failing = SourceIdFailureAllocator {
        inner: InMemorySessionIdAllocator::new(),
    };
    let task = source_task(&submission);
    let snapshot = submission.session.captured.snapshot.id;
    let publisher = publisher(snapshot, usize::MAX, &task.unit);
    let sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        DiagnosticPhase::SourceLoad,
        snapshot,
        "frontend.source_load",
    ));
    let result = source_registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            PhaseInput::new(
                task.unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    source_key(&fixture.version),
                    Vec::new(),
                ),
            ),
            PhaseExecutionResources {
                diagnostics: Some(sink),
                output_publisher: Some(publisher),
                source_load: Some(SourceLoadInputs {
                    snapshot: &submission.session.captured.snapshot,
                    build_plan: submission.build_plan.as_ref().unwrap(),
                    module_index: submission.module_index.as_ref().unwrap(),
                    allocator: &failing,
                }),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap();
    assert_source_error(
        &result.result,
        &fixture.version,
        600,
        "source.source_id_allocation",
    );
}

#[test]
fn source_load_blocks_missing_or_stale_resources_and_scopes_cancellation() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let task = source_task(&submission);
    let snapshot = submission.session.captured.snapshot.id;
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let sink = || {
        DiagnosticSink::new(DiagnosticProducerScope::new(
            DiagnosticPhase::SourceLoad,
            snapshot,
            "frontend.source_load",
        ))
    };
    let input = || {
        PhaseInput::new(
            task.unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot,
                source_key(&fixture.version),
                Vec::new(),
            ),
        )
    };
    let run = |resources: PhaseExecutionResources<'_>| {
        source_registry()
            .execute_phase_with_resources(PipelinePhase::SourceLoad, input(), resources)
            .unwrap()
            .result
    };
    let publisher = publisher(snapshot, usize::MAX, &task.unit);
    let missing_sink = run(PhaseExecutionResources {
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(missing_sink.status, PhaseStatus::Blocking);
    assert!(missing_sink.output_refs.is_empty());
    let denied = std::sync::Arc::new(PhaseOutputPublisher::new(
        std::sync::Arc::new(IrStorageService::new()),
        std::sync::Arc::new(SnapshotHandleRegistry::new()),
    ));
    denied.register_current_snapshot(snapshot);
    denied.allow_work_unit(AllowedWorkUnit::new(
        IrPipelinePhase::new("SourceLoad"),
        OutputKind::new("SourceUnit"),
        IrWorkUnit::new("another-module"),
    ));
    let denied = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(denied),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(denied.status, PhaseStatus::Blocking);
    assert!(denied.output_refs.is_empty());
    let mut sealed = sink();
    sealed.seal();
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sealed),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                DiagnosticPhase::Frontend,
                snapshot,
                "frontend.source_load",
            ))),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                DiagnosticPhase::SourceLoad,
                snapshot_id(0xab),
                "frontend.source_load",
            ))),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: None,
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            source_load: None,
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            cancellation: Some(CancellationToken {
                snapshot,
                generation: CancellationGeneration::new(1),
                reason: CancellationReason::ExplicitRequest,
            }),
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
        })
        .status,
        PhaseStatus::Cancelled
    );
    let foreign = snapshot_id(0xaa);
    assert_eq!(
        run(PhaseExecutionResources {
            cancellation: Some(CancellationToken {
                snapshot: foreign,
                generation: CancellationGeneration::new(1),
                reason: CancellationReason::ExplicitRequest,
            }),
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
        })
        .status,
        PhaseStatus::Blocking
    );
    publisher.mark_obsolete(snapshot).unwrap();
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: Some(publisher),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
}

#[test]
fn source_load_rejects_duplicate_foreign_nondisk_missing_and_summary_bindings() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let base_snapshot = submission.session.captured.snapshot.clone();
    let plan = submission.build_plan.as_ref().unwrap();
    let index = submission.module_index.as_ref().unwrap();
    let module_unit = source_task(&submission).unit.clone();
    let run = |snapshot: &BuildSnapshot, plan: &BuildPlan, index: &ModuleIndex, unit: WorkUnit| {
        binding_result(
            &submission,
            SourceLoadInputs {
                snapshot,
                build_plan: plan,
                module_index: index,
                allocator: &ids,
            },
            unit,
        )
    };

    let mut duplicate = base_snapshot.clone();
    duplicate.source_versions.push(fixture.version.clone());
    assert_eq!(
        run(&duplicate, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut foreign = base_snapshot.clone();
    foreign.source_versions[0].package_id = PackageId::new("foreign");
    assert_eq!(
        run(&foreign, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut nondisk = base_snapshot.clone();
    nondisk.source_versions[0].origin = SourceOrigin::OpenBuffer { version: 1 };
    assert_eq!(
        run(&nondisk, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut missing = base_snapshot.clone();
    missing.source_versions.clear();
    assert_eq!(
        run(&missing, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut summary = index.clone();
    summary.modules[0].location = ModuleIndexLocation::DependencySummary {
        artifact: "alpha-summary".to_owned(),
        content_hash: hash(0x91),
    };
    assert_eq!(
        run(&base_snapshot, plan, &summary, module_unit),
        PhaseStatus::Blocking
    );

    let mut mismatched_version = base_snapshot.clone();
    mismatched_version.source_versions[0].edition = Edition::new("2030");
    assert_eq!(
        run(
            &mismatched_version,
            plan,
            index,
            source_task(&submission).unit.clone()
        ),
        PhaseStatus::Blocking
    );

    let mut mismatched_index = index.clone();
    mismatched_index.modules[0].edition = Edition::new("2030");
    assert_eq!(
        run(
            &base_snapshot,
            plan,
            &mismatched_index,
            source_task(&submission).unit.clone()
        ),
        PhaseStatus::Blocking
    );
}

#[test]
fn source_load_rejects_mismatched_plan_and_disallowed_work_unit() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let snapshot = submission.session.captured.snapshot.clone();
    let index = submission.module_index.as_ref().unwrap();
    let mut plan = submission.build_plan.as_ref().unwrap().clone();
    plan.workspace_root = WorkspaceRoot::new("other-workspace");
    let module_unit = source_task(&submission).unit.clone();
    assert_eq!(
        binding_result(
            &submission,
            SourceLoadInputs {
                snapshot: &snapshot,
                build_plan: &plan,
                module_index: index,
                allocator: &ids,
            },
            module_unit.clone(),
        ),
        PhaseStatus::Blocking
    );
    assert_eq!(
        binding_result(
            &submission,
            SourceLoadInputs {
                snapshot: &snapshot,
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: index,
                allocator: &ids,
            },
            WorkUnit::Package {
                package_id: PackageId::new("alpha"),
            },
        ),
        PhaseStatus::Blocking
    );
}

#[test]
fn source_load_requires_owner_dispatch_hash_and_no_parents_or_dependencies() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let task = source_task(&submission);
    let snapshot = submission.session.captured.snapshot.id;
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let (parent, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(parent.status, PhaseStatus::Complete);
    let parent = mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
        &publisher,
        snapshot,
        parent.output_refs[0].clone(),
    )
    .unwrap();
    let registry = source_registry();
    for (input_hash, dependencies, parents) in [
        (hash(7), Vec::new(), Vec::new()),
        (source_key(&fixture.version), vec![hash(8)], Vec::new()),
        (source_key(&fixture.version), Vec::new(), vec![parent]),
    ] {
        let input = PhaseInput::new(
            task.unit.clone(),
            PhaseDispatchInputBundle::new(snapshot, input_hash, dependencies, parents).unwrap(),
        );
        let cache = registry
            .cache_key_for_phase(PipelinePhase::SourceLoad, &input)
            .unwrap();
        assert!(matches!(cache.intent, PhaseCacheIntent::NoKey { .. }));
        let result = registry
            .execute_phase_with_resources(
                PipelinePhase::SourceLoad,
                input,
                PhaseExecutionResources {
                    diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::SourceLoad,
                        snapshot,
                        "frontend.source_load",
                    ))),
                    output_publisher: Some(publisher.clone()),
                    source_load: Some(source_load),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
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
fn dependency_lexical_provider_reads_real_source_and_summary_boundaries() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, LexicalContributionSummary, ModuleLexicalSummary, ModuleSummary,
            ModuleSummaryIdentity, SourceRangeSummary, current_schema_version, module_summary_json,
        },
        store::{
            PublishedArtifactPath, artifact_hash_domain, canonical_json_string,
            write_published_artifact,
        },
    };
    use mizar_frontend::{
        lexical_env::{
            ExportRank, ExportedSymbolShape, ModuleId as LexerModuleId, SymbolId, UserSymbolArity,
            UserSymbolKind,
        },
        lexing::TokenKind,
        orchestration::{DiagnosticCode, DiagnosticLocation, Frontend, FrontendOutput},
        parsing::MizarParserSeam,
        source::FrontendSourceLoader,
    };
    use std::sync::Arc;

    let fixture = Fixture::new(
        b"\xef\xbb\xbfimport dep.core as A, dep.core as B; import dep.{core, absent};\r\ndefinition\r\nend;\r\ntheorem Combined: combine(x,y) = x;\r\n",
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (result, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(result.status, PhaseStatus::Complete);
    let output = &result.output_refs[0];
    let typed = publisher
        .storage()
        .typed_handle::<SourceUnit>(output, &OutputKind::new("SourceUnit"))
        .unwrap();
    let source = publisher.storage().get(&typed).unwrap().clone();
    let mut bridge = SpanBridge::new();
    register_source_unit(&mut bridge, &source).unwrap();
    let preprocessed = preprocess(&source, &mut bridge).unwrap();
    assert_eq!(preprocessed.import_stubs.len(), 4);

    let workspace_root = WorkspaceRoot::new(fixture.root.to_string_lossy().into_owned());
    let plan = produce_build_plan(
        PlanRequest {
            workspace_root: workspace_root.clone(),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-test"),
        },
        vec![WorkspacePackage {
            member_path: "alpha".to_owned(),
            manifest: parse_package_manifest(
                "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dependencies]\ndep = \"1.0.0\"\n",
            )
            .unwrap(),
        }],
        parse_lockfile(
            "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n",
        )
        .unwrap(),
    )
    .unwrap();

    let artifact_root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let artifact = "dep/core.summary.json";
    let module = ModuleSummaryIdentity {
        package_id: "dep".to_owned(),
        package_version: Some("1.0.0".to_owned()),
        lockfile_identity: Some("dependency-lock".to_owned()),
        module_path: "core".to_owned(),
        language_edition: "2025".to_owned(),
    };
    let identity = canonical_json_string(&module.canonical_json().unwrap());
    let origin_id = "symbol:combine";
    let shape = ExportedSymbolShape {
        spelling: "combine".to_owned(),
        symbol_id: SymbolId::new(canonical_json_string(
            &mizar_artifact::store::CanonicalJson::array([
                module.canonical_json().unwrap(),
                mizar_artifact::store::CanonicalJson::string(origin_id),
            ]),
        )),
        source_module: LexerModuleId::new(identity),
        export_rank: ExportRank::new(0),
        kind: UserSymbolKind::Functor,
        arity: UserSymbolArity::exact(2),
        operator: None,
    };
    let payload = String::from_utf8(shape.canonical_bytes().unwrap()).unwrap();
    // Published dependency fixture only, not a substitute for a source export producer.
    let mut summary = ModuleSummary {
        schema_version: current_schema_version(),
        module: module.clone(),
        source_hash: hash(9),
        interface_hash: hash(0),
        exported_symbols: vec![ExportedSymbolSummary {
            origin_id: origin_id.to_owned(),
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
                key: origin_id.to_owned(),
                payload,
            }],
        },
        reexports: Vec::new(),
        dependency_interfaces: Vec::new(),
    };
    summary.refresh_interface_hash().unwrap();
    let publish = |value: &ModuleSummary| {
        let json = module_summary_json(value).unwrap();
        let domain = artifact_hash_domain(
            mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
            value.schema_version,
        );
        write_published_artifact(
            &artifact_root,
            &PublishedArtifactPath::new(artifact).unwrap(),
            &json,
            &domain,
            &[],
        )
        .unwrap()
        .artifact_hash
    };
    let content_hash = publish(&summary);
    let dependency = DependencyArtifactIndex::new(
        PackageId::new("dep"),
        Vec::new(),
        vec![DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new("core"),
            ),
            artifact: artifact.to_owned(),
            content_hash,
        }],
    );
    let module_index = build_module_index(
        &plan,
        &StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
        }]),
        &[dependency],
    )
    .unwrap();
    let request = LexicalEnvironmentRequest {
        source_id: source.source_id,
        import_stubs: &preprocessed.import_stubs,
        edition: source.edition.clone(),
    };
    let roots = vec![(PackageId::new("dep"), artifact_root.clone())];
    let inputs = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
        build_plan: &plan,
        module_index: &module_index,
        allocator: &ids,
    };
    let provider = inputs.dependency_lexical_provider(&roots);
    let resolved = provider.resolve_imports(&request).unwrap();
    assert_eq!(resolved.imports.len(), 3);
    assert!(!resolved.summaries.is_empty());
    assert!(
        resolved
            .summaries
            .iter()
            .all(|summary| summary.module_id == shape.source_module)
    );
    assert!(
        resolved
            .imports
            .iter()
            .all(|entry| entry.import.module_id == resolved.summaries[0].module_id)
    );
    assert_eq!(
        resolved
            .imports
            .iter()
            .map(|entry| entry.stub_ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    for entry in &resolved.imports {
        assert_eq!(
            entry.stub_span,
            preprocessed.import_stubs[entry.stub_ordinal].span
        );
    }
    let active = build_active_lexical_environment(&request, &provider).unwrap();
    assert!(active.environment.user_symbol("combine").is_some());
    assert_eq!(active.environment.visible_user_symbols().len(), 1);
    assert!(active.diagnostics.iter().any(|diagnostic| {
        diagnostic.code
            == mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode::UnresolvedImport
    }));
    let repeat = inputs.dependency_lexical_provider(&roots);
    assert_eq!(resolved, repeat.resolve_imports(&request).unwrap());
    let repeated = build_active_lexical_environment(&request, &repeat).unwrap();
    assert_eq!(active.fingerprint, repeated.fingerprint);

    let frontend_output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
        inputs.dependency_lexical_provider(&roots),
        MizarParserSeam,
    )
    .run_loaded(source.as_ref().clone())
    .unwrap();
    assert_eq!(frontend_output.source, *source);
    assert_eq!(frontend_output.preprocessed, preprocessed);
    assert_eq!(
        frontend_output
            .cache_keys
            .active_lexical_environment
            .fingerprint,
        active.fingerprint
    );
    let combine = frontend_output
        .tokens
        .tokens
        .iter()
        .find(|token| token.text.as_ref() == "combine")
        .unwrap();
    assert_eq!(combine.kind, TokenKind::UserSymbol);
    let ast = frontend_output.ast.as_ref().unwrap();
    assert!(ast.node_views().any(|view| {
        view.as_theorem_item().is_some()
            && view.range().start <= combine.span.start
            && combine.span.end <= view.range().end
            && !view.is_recovered()
    }));
    assert!(ast.node_views().any(|view| {
        view.as_application_term().is_some()
            && view.range().start <= combine.span.start
            && combine.span.end <= view.range().end
            && !view.is_recovered()
    }));
    assert!(ast.node_views().all(|view| !view.is_recovered()));
    assert_eq!(frontend_output.diagnostics.len(), 1);
    assert_eq!(
        frontend_output.diagnostics[0].class,
        mizar_frontend::orchestration::DiagnosticClass::LexicalEnvironment
    );
    assert_eq!(
        frontend_output.diagnostics[0].code,
        DiagnosticCode::LexicalEnvironment(
            mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode::UnresolvedImport
        )
    );
    let DiagnosticLocation::SourceRange(primary) = &frontend_output.diagnostics[0].location else {
        panic!("expected unresolved import source range");
    };
    let input = SourceInput {
        package_id: source.package_id.clone(),
        module_path: source.module_path.clone(),
        normalized_path: source.normalized_path.clone(),
        edition: source.edition.clone(),
        origin: SourceOriginInput::Disk {
            path: source.file_path.clone(),
        },
    };
    let bytes = frontend_output.canonical_disk_bytes().unwrap();
    assert_eq!(
        FrontendOutput::from_canonical_disk_bytes(&bytes, source.source_id, &input).unwrap(),
        frontend_output
    );
    let fresh_id = ids.next_source_id(inputs.snapshot.id).unwrap();
    assert_ne!(fresh_id, source.source_id);
    let rebound = FrontendOutput::from_canonical_disk_bytes(&bytes, fresh_id, &input).unwrap();
    assert_eq!(rebound.source.source_id, fresh_id);
    assert_eq!(rebound.preprocessed.source_id, fresh_id);
    assert_eq!(rebound.tokens.source_id, fresh_id);
    assert_eq!(rebound.ast.as_ref().unwrap().source_id, fresh_id);
    assert_eq!(
        rebound.diagnostics[0].location,
        DiagnosticLocation::SourceRange(mizar_session::SourceRange {
            source_id: fresh_id,
            ..*primary
        })
    );
    assert_eq!(rebound.canonical_disk_bytes().unwrap(), bytes);

    for corrupt in [false, true] {
        if corrupt {
            fs::write(artifact_root.join(artifact), b"not canonical json").unwrap();
        } else {
            fs::remove_file(artifact_root.join(artifact)).unwrap();
        }
        let unavailable = inputs.dependency_lexical_provider(&roots);
        let unavailable_result = unavailable.resolve_imports(&request).unwrap();
        assert_eq!(unavailable_result.imports.len(), 3);
        assert!(unavailable_result.summaries.is_empty());
        let environment = build_active_lexical_environment(&request, &unavailable).unwrap();
        assert!(environment.environment.user_symbol("combine").is_none());
        assert!(environment.diagnostics.iter().any(|diagnostic| {
            diagnostic.code
                == mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode::MissingSummary
        }));
        publish(&summary);
    }

    let mut malformed = summary.clone();
    malformed.lexical_summary.contributions[0].payload.push('x');
    malformed.refresh_interface_hash().unwrap();
    let malformed_hash = publish(&malformed);
    let mut malformed_index = module_index.clone();
    for reference in &mut malformed_index.dependency_summaries {
        if reference.module.package.as_str() == "dep" {
            reference.content_hash = malformed_hash;
        }
    }
    for entry in &mut malformed_index.modules {
        if entry.module.package.as_str() == "dep"
            && let ModuleIndexLocation::DependencySummary { content_hash, .. } = &mut entry.location
        {
            *content_hash = malformed_hash;
        }
    }
    let malformed_inputs = SourceLoadInputs {
        module_index: &malformed_index,
        ..inputs
    };
    assert!(matches!(
        malformed_inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
    publish(&summary);

    for roots in [
        Vec::new(),
        vec![
            (PackageId::new("dep"), artifact_root.clone()),
            (PackageId::new("dep"), artifact_root.clone()),
        ],
        vec![(PackageId::new("other"), artifact_root.clone())],
    ] {
        assert!(matches!(
            inputs
                .dependency_lexical_provider(&roots)
                .resolve_imports(&request),
            Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
        ));
    }

    for invalid_request in [
        LexicalEnvironmentRequest {
            edition: Edition::new("2026"),
            ..request.clone()
        },
        LexicalEnvironmentRequest {
            source_id: ids.next_source_id(inputs.snapshot.id).unwrap(),
            ..request.clone()
        },
    ] {
        assert!(matches!(
            inputs
                .dependency_lexical_provider(&roots)
                .resolve_imports(&invalid_request),
            Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
        ));
    }
    let mut mismatched_index = module_index.clone();
    mismatched_index.dependency_summaries[0].content_hash = hash(71);
    assert!(matches!(
        SourceLoadInputs {
            module_index: &mismatched_index,
            ..inputs
        }
        .dependency_lexical_provider(&roots)
        .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
    let mut duplicate_snapshot = submission.session.captured.snapshot.clone();
    duplicate_snapshot
        .source_versions
        .push(fixture.version.clone());
    let duplicate_inputs = SourceLoadInputs {
        snapshot: &duplicate_snapshot,
        ..inputs
    };
    assert!(matches!(
        duplicate_inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
    let mut bad_index = module_index.clone();
    bad_index
        .modules
        .iter_mut()
        .find(|entry| entry.module.package.as_str() == "alpha")
        .unwrap()
        .edition = Edition::new("2026");
    let bad_index_inputs = SourceLoadInputs {
        module_index: &bad_index,
        ..inputs
    };
    assert!(matches!(
        bad_index_inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));

    let mut source_backed = preprocessed.clone();
    source_backed.import_stubs[0].path.relative =
        Some(mizar_frontend::preprocess::ImportStubRelativePrefix::Current);
    source_backed.import_stubs[0].path.spelling = Arc::from(".main");
    source_backed.import_stubs[0].path.components = vec![Arc::from("main")];
    let source_backed_span = source_backed.import_stubs[0].path.span;
    source_backed.import_stubs[0].path.source_segments = vec![source_backed_span];
    let source_backed_request = LexicalEnvironmentRequest {
        import_stubs: &source_backed.import_stubs,
        ..request
    };
    assert!(matches!(
        inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&source_backed_request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
}

fn execute(
    submission: &mizar_driver::driver::BuildSubmission,
    ids: &InMemorySessionIdAllocator,
    threshold: usize,
) -> (
    mizar_driver::registry::PhaseResult,
    std::sync::Arc<PhaseOutputPublisher>,
) {
    let task = source_task(submission);
    let snapshot = submission.session.captured.snapshot.id;
    // Force the loader to allocate a different ID from the captured source.
    ids.next_source_id(snapshot).unwrap();
    let publisher = publisher(snapshot, threshold, &task.unit);
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: ids,
    };
    let sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        DiagnosticPhase::SourceLoad,
        snapshot,
        "frontend.source_load",
    ));
    let input = PhaseInput::new(
        task.unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            snapshot,
            source_key(&submission.session.request.source_inputs.versions[0]),
            Vec::new(),
        ),
    );
    let result = source_registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            input,
            PhaseExecutionResources {
                diagnostics: Some(sink),
                output_publisher: Some(publisher.clone()),
                source_load: Some(source_load),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result;
    (result, publisher)
}

fn execute_frontend(
    submission: &mizar_driver::driver::BuildSubmission,
    ids: &InMemorySessionIdAllocator,
    publisher: &std::sync::Arc<PhaseOutputPublisher>,
    parents: Vec<AnyPhaseOutputRef>,
    dispatch: (Hash, Vec<Hash>),
    artifact_roots: Vec<(PackageId, PathBuf)>,
    module_index: Option<&ModuleIndex>,
) -> mizar_driver::registry::PhaseResult {
    let snapshot = submission.session.captured.snapshot.id;
    let task = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let ir_unit = IrWorkUnit::new(match &task.unit {
        WorkUnit::Module { module } => {
            format!("{:?}:{:?}", module.package.as_str(), module.path.as_str())
        }
        _ => unreachable!(),
    });
    publisher.allow_work_unit(AllowedWorkUnit::new(
        IrPipelinePhase::new("Frontend"),
        OutputKind::new("FrontendOutput"),
        ir_unit,
    ));
    let parents = parents
        .into_iter()
        .map(|parent| {
            mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
                publisher, snapshot, parent,
            )
            .unwrap()
        })
        .collect();
    let input = PhaseInput::new(
        task.unit.clone(),
        PhaseDispatchInputBundle::new(snapshot, dispatch.0, dispatch.1, parents).unwrap(),
    );
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: module_index.unwrap_or(submission.module_index.as_ref().unwrap()),
        allocator: ids,
    };
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(artifact_roots);
    let registry = builder.build().unwrap();
    let cache = registry
        .cache_key_for_phase(PipelinePhase::Frontend, &input)
        .unwrap();
    assert!(matches!(cache.intent, PhaseCacheIntent::NoKey { .. }));
    registry
        .execute_phase_with_resources(
            PipelinePhase::Frontend,
            input,
            PhaseExecutionResources {
                diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::Frontend,
                    snapshot,
                    "frontend.run_loaded",
                ))),
                output_publisher: Some(publisher.clone()),
                source_load: Some(source_load),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result
}

fn binding_result(
    submission: &mizar_driver::driver::BuildSubmission,
    source_load: SourceLoadInputs<'_>,
    unit: WorkUnit,
) -> PhaseStatus {
    let snapshot = submission.session.captured.snapshot.id;
    let module_unit = source_task(submission).unit.clone();
    let publisher = publisher(snapshot, usize::MAX, &module_unit);
    let sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        DiagnosticPhase::SourceLoad,
        snapshot,
        "frontend.source_load",
    ));
    source_registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    source_key(&submission.session.request.source_inputs.versions[0]),
                    Vec::new(),
                ),
            ),
            PhaseExecutionResources {
                diagnostics: Some(sink),
                output_publisher: Some(publisher),
                source_load: Some(source_load),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result
        .status
}

fn source_registry() -> mizar_driver::registry::PhaseRegistry {
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.build().unwrap()
}

fn publisher(
    snapshot: BuildSnapshotId,
    threshold: usize,
    unit: &WorkUnit,
) -> std::sync::Arc<PhaseOutputPublisher> {
    let phase = IrPipelinePhase::new("SourceLoad");
    let kind = OutputKind::new("SourceUnit");
    let ir_unit = IrWorkUnit::new(match unit {
        WorkUnit::Module { module } => {
            format!("{:?}:{:?}", module.package.as_str(), module.path.as_str())
        }
        _ => unreachable!(),
    });
    let publisher = std::sync::Arc::new(PhaseOutputPublisher::new(
        std::sync::Arc::new(IrStorageService::with_policy(
            StoragePolicy::with_blob_spill_threshold(threshold),
        )),
        std::sync::Arc::new(SnapshotHandleRegistry::new()),
    ));
    publisher.register_current_snapshot(snapshot);
    publisher.allow_work_unit(AllowedWorkUnit::new(phase, kind, ir_unit));
    publisher
}

fn source_task(
    submission: &mizar_driver::driver::BuildSubmission,
) -> &mizar_build::task_graph::BuildTask {
    submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::SourceLoad)
        .unwrap()
}

fn source_key(version: &SourceVersion) -> Hash {
    SourceUnitCacheKey {
        version: std::sync::Arc::from(SOURCE_UNIT_CACHE_KEY_VERSION),
        package_id: version.package_id.clone(),
        module_path: version.module_path.clone(),
        normalized_path: version.normalized_path.clone(),
        source_hash: version.source_hash,
        edition: version.edition.clone(),
    }
    .stable_hash()
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let encoded = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&encoded).unwrap()
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

#[derive(Debug)]
struct SourceIdFailureAllocator {
    inner: InMemorySessionIdAllocator,
}

impl SessionIdAllocator for SourceIdFailureAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, _snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        Err(IdError::AllocatorOverflow)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        self.inner.next_lease_id(snapshot)
    }
}

fn assert_source_error(
    result: &mizar_driver::registry::PhaseResult,
    version: &SourceVersion,
    code: u16,
    detail: &str,
) {
    assert_eq!(result.status, PhaseStatus::Fatal);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].drafts().len(), 1);
    let draft = &result.diagnostics[0].drafts()[0];
    assert_eq!(draft.code().number(), code);
    assert_eq!(draft.stable_detail_key(), detail);
    assert_eq!(
        draft.primary_location(),
        &mizar_diagnostics::failure_record::DiagnosticPrimaryLocation::SourceLoad {
            package_id: version.package_id.clone(),
            path: version.normalized_path.clone(),
        }
    );
}

#[cfg(unix)]
#[test]
fn source_load_rejects_package_root_symlink_outside_workspace() {
    let fixture = Fixture::new(b"alpha");
    let outside = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    fs::remove_dir_all(fixture.root.join("alpha")).unwrap();
    std::os::unix::fs::symlink(outside.root.join("alpha"), fixture.root.join("alpha")).unwrap();
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
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
