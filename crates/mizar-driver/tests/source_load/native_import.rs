use super::*;

#[test]
fn native_import_discovery_replays_raw_versions_with_fresh_owners_and_rejects_conflicts() {
    use mizar_ir::dispatch_input::SealedParentOutputHandle;
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let leaf_text =
        "definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    let main_text = "import alpha.leaf;\ntheorem T: a combine b = a;\n";
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![]);
    let mut driver = CompilerDriver::new(builder.build().unwrap());
    let mut canonical = None;
    let mut previous: Option<(std::sync::Arc<PhaseOutputPublisher>, AnyPhaseOutputRef)> = None;
    let mut oldest = None;
    for stage in 0..3 {
        fixture.write(main_text.as_bytes());
        fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf_text).unwrap();
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        request.generation = BuildRequestGeneration::new(stage);
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
            ],
        }]);
        input.dependency_overlay = ModuleDependencyOverlay::unavailable();
        input.worker_count = if stage == 0 { 1 } else { 4 };
        input.completion_order = if stage == 1 {
            CompletionOrder::Reverse
        } else {
            CompletionOrder::Canonical
        };
        let mut descriptors = Vec::new();
        for (module, raw) in [("main", main_text), ("leaf", leaf_text)] {
            let path = format!("src/{module}.miz");
            let normalized_path =
                normalize_path(&fixture.root.join("alpha"), Path::new(&path)).unwrap();
            let origin = if stage == 0 {
                SourceOriginInput::Disk {
                    path: path.clone().into(),
                }
            } else {
                SourceOriginInput::OpenBuffer {
                    uri: format!(
                        "file://{}",
                        fixture.root.join("alpha").join(&path).display()
                    ),
                    expected_version: stage as i64 + 7,
                    actual_version: stage as i64 + 7,
                    text: std::sync::Arc::from(if stage == 0 {
                        raw.to_owned()
                    } else {
                        format!("\u{feff}{}", raw.replace('\n', "\r\n"))
                    }),
                }
            };
            descriptors.push((
                DiskSourceLoader::new(fixture.root.join("alpha")),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new(module),
                    normalized_path,
                    edition: Edition::new("2025"),
                    origin,
                },
            ));
        }
        if stage == 1 {
            descriptors.reverse();
        }
        let session = request
            .clone()
            .capture_sources(&ids, &snapshots, descriptors.clone())
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        assert_eq!(*canonical.get_or_insert(snapshot), snapshot);
        if stage == 0 {
            oldest = Some(
                request
                    .clone()
                    .capture_sources(&ids, &snapshots, descriptors.clone())
                    .unwrap(),
            );
        }
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let publisher = publisher(snapshot, 0, &unit);
        for module in ["main", "leaf"] {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new(module),
                ),
            };
            let ir_unit = IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module));
            publisher.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                ir_unit,
            ));
            allow_frontend(&publisher, &unit);
        }
        if stage == 0 {
            // Real acquisition keeps canonical metadata/IDs while changing raw Disk provenance.
            fs::write(
                fixture.root.join("alpha/src/leaf.miz"),
                format!("\u{feff}{}", leaf_text.replace('\n', "\r\n")),
            )
            .unwrap();
            let producer_ids = InMemorySessionIdAllocator::new();
            let producer_snapshots = SnapshotRegistry::new();
            let producer_session = request
                .capture_sources(&producer_ids, &producer_snapshots, descriptors)
                .unwrap();
            assert_eq!(
                producer_session.captured().snapshot(),
                session.captured().snapshot()
            );
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut producer = CompilerDriver::new(builder.build().unwrap());
            let mut producer_input = DriverSubmitInput::new(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
                input.source_layout.clone(),
            );
            producer_input.dependency_overlay = ModuleDependencyOverlay::unavailable();
            let produced = producer
                .submit_captured_with_import_discovery(
                    producer_session.clone(),
                    &producer_ids,
                    &producer_snapshots,
                    producer_input,
                )
                .unwrap();
            for module in ["leaf", "main"] {
                // Each owner starts without the tested Frontend slot, so duplicate storage
                // cannot mask the whole-source equality rejection.
                let owner = crate::publisher(snapshot, 0, &unit);
                for name in ["main", "leaf"] {
                    let unit = WorkUnit::Module {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new(name),
                        ),
                    };
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("SourceLoad"),
                        OutputKind::new("SourceUnit"),
                        IrWorkUnit::new(format!("{:?}:{:?}", "alpha", name)),
                    ));
                    allow_frontend(&owner, &unit);
                }
                let run =
                    |phase,
                     name: &str,
                     parents: Vec<AnyPhaseOutputRef>,
                     capture: &mizar_driver::request::CapturedSnapshot| {
                        let version = capture
                            .snapshot()
                            .source_versions
                            .iter()
                            .find(|version| version.module_path.as_str() == name)
                            .unwrap();
                        let parents = parents
                            .into_iter()
                            .map(|handle| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner, snapshot, handle,
                                )
                                .unwrap()
                            })
                            .collect();
                        producer
                            .registry()
                            .execute_phase_with_resources(
                                phase,
                                PhaseInput::new(
                                    WorkUnit::Module {
                                        module: mizar_build::module_index::ModuleId::new(
                                            PackageId::new("alpha"),
                                            ModulePath::new(name),
                                        ),
                                    },
                                    PhaseDispatchInputBundle::new(
                                        snapshot,
                                        source_key(version),
                                        vec![],
                                        parents,
                                    )
                                    .unwrap(),
                                ),
                                PhaseExecutionResources {
                                    diagnostics: Some(DiagnosticSink::new(
                                        DiagnosticProducerScope::new(
                                            if phase == PipelinePhase::SourceLoad {
                                                DiagnosticPhase::SourceLoad
                                            } else {
                                                DiagnosticPhase::Frontend
                                            },
                                            snapshot,
                                            "native.parent_binding",
                                        ),
                                    )),
                                    output_publisher: Some(owner.clone()),
                                    source_load: Some(SourceLoadInputs {
                                        snapshot: capture.snapshot(),
                                        captured: Some(capture),
                                        build_plan: produced.build_plan.as_ref().unwrap(),
                                        module_index: produced.module_index.as_ref().unwrap(),
                                        allocator: &producer_ids,
                                    }),
                                    ..PhaseExecutionResources::default()
                                },
                            )
                            .unwrap()
                            .result
                    };
                let leaf = run(
                    PipelinePhase::SourceLoad,
                    "leaf",
                    vec![],
                    producer_session.captured(),
                );
                assert_eq!(leaf.status, PhaseStatus::Complete);
                let leaf_source = leaf.output_refs[0].clone();
                let typed = owner
                    .storage()
                    .typed_handle::<SourceUnit>(&leaf_source, &OutputKind::new("SourceUnit"))
                    .unwrap();
                let actual = owner.storage().get(&typed).unwrap();
                assert!(actual.loading_map.is_some());
                assert_eq!(actual.source_text.as_ref(), leaf_text);
                assert_eq!(actual.origin, SourceOrigin::Disk);
                assert!(actual.canonical_disk_bytes().is_some());
                let parents = if module == "main" {
                    let root = run(
                        PipelinePhase::SourceLoad,
                        "main",
                        vec![],
                        producer_session.captured(),
                    );
                    assert_eq!(root.status, PhaseStatus::Complete);
                    let leaf = run(
                        PipelinePhase::Frontend,
                        "leaf",
                        vec![leaf_source],
                        producer_session.captured(),
                    );
                    assert_eq!(leaf.status, PhaseStatus::Complete);
                    vec![root.output_refs[0].clone(), leaf.output_refs[0].clone()]
                } else {
                    vec![leaf_source]
                };
                let rejected = run(PipelinePhase::Frontend, module, parents, session.captured());
                assert_eq!(
                    rejected.status,
                    PhaseStatus::Blocking,
                    "{module} raw-map binding"
                );
                assert!(rejected.output_refs.is_empty());
                assert!(rejected.diagnostics.is_empty());
            }
        }
        if let Some((owner, old)) = &previous {
            assert!(
                SealedParentOutputHandle::from_published_output(&publisher, snapshot, old.clone())
                    .is_err()
            );
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut conflicting =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let mut conflicting_input = DriverSubmitInput::new(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
                input.source_layout.clone(),
            );
            conflicting_input.dependency_overlay = ModuleDependencyOverlay::unavailable();
            let rejected = conflicting
                .submit_captured_with_import_discovery(
                    session.clone(),
                    &ids,
                    &snapshots,
                    conflicting_input,
                )
                .unwrap();
            assert!(
                rejected
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .all(|result| result.output_refs.is_empty())
            );
            assert!(
                rejected
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .any(|result| result.status == PhaseStatus::Blocking)
            );
        }
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        fs::remove_file(fixture.root.join("alpha/src/leaf.miz")).unwrap();
        driver = driver.with_output_publisher(publisher.clone());
        let submission = driver
            .submit_captured_with_import_discovery(session, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let handles = run
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        assert_eq!(handles.len(), 4);
        let main = *handles
            .iter()
            .find(|handle| {
                handle.output_kind() == &OutputKind::new("FrontendOutput")
                    && handle.work_unit() == &IrWorkUnit::new(format!("{:?}:{:?}", "alpha", "main"))
            })
            .unwrap();
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                main,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let output = publisher.storage().get(&typed).unwrap();
        assert!(output.diagnostics.is_empty());
        assert!(
            output.tokens.tokens().iter().any(
                |token| token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
            )
        );
        assert_eq!(
            output.source.origin,
            if stage == 0 {
                SourceOrigin::Disk
            } else {
                SourceOrigin::OpenBuffer {
                    version: stage as i64 + 7,
                }
            }
        );
        assert_eq!(output.source.source_text.as_ref(), main_text);
        assert_eq!(main.lineage().parents.len(), 2);
        assert!(matches!(main.placement(), StoragePlacement::Blob { .. }));
        previous = Some((publisher, main.clone()));
    }
    let (_, input) = fixture.submit_request(vec![], vec![]);
    let obsolete = driver
        .submit_captured(oldest.unwrap(), &ids, &snapshots, input)
        .unwrap();
    assert_eq!(
        obsolete.status,
        DriverSubmissionStatus::SupersededBeforeSubmission
    );
    assert_eq!(
        obsolete.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Superseded)
    );
    assert!(obsolete.scheduler_run.is_none());
    assert!(matches!(
        obsolete.publication_decision,
        mizar_driver::request::PublicationDecision::Suppressed(_)
    ));
}
