use super::*;

#[test]
fn native_toolchain_context_binds_source_and_frontend_with_valid_own_parents() {
    let raw = "\u{feff}definition\r\nend;\r\n";
    let mut rejected = Vec::new();
    for open in [false, true] {
        for threshold in [usize::MAX, 0] {
            let fixture = Fixture::new(raw.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
            request.source_inputs.versions.clear();
            request.dependency_inputs.toolchain =
                ToolchainInfo::new("opaque native/toolchain@context");
            input.plan_request.toolchain = request.dependency_inputs.toolchain.clone();
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
            let loader = DiskSourceLoader::new(fixture.root.join("alpha"));
            let session = request
                .capture_sources(&ids, &snapshots, vec![(loader.clone(), descriptor.clone())])
                .unwrap();
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver = CompilerDriver::new(builder.build().unwrap());
            let submission = driver
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            let captured = submission.session.captured();
            let snapshot = captured.snapshot();
            let unit = source_task(&submission).unit.clone();
            let source_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot.id,
                    source_key(&snapshot.source_versions[0]),
                    vec![],
                ),
            );
            let inputs = SourceLoadInputs {
                captured: Some(captured),
                snapshot,
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let mut foreign = inputs.build_plan.clone();
            foreign.toolchain = ToolchainInfo::new("opaque native/toolchain@other-context");
            assert_eq!(inputs.build_plan.toolchain, snapshot.toolchain);
            assert!(snapshots.is_current_for_request(snapshot.id, submission.session.request().id));
            let display_path = mizar_frontend::source::diagnostic_file_path(&descriptor);
            let expected = mizar_frontend::source::source_unit_from_loaded(
                mizar_session::SourceLoader::load(
                    &loader,
                    snapshot.id,
                    descriptor,
                    &InMemorySessionIdAllocator::new(),
                )
                .unwrap(),
                display_path,
            );
            for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
                for mismatch in [false, true] {
                    // Fresh slots and a genuine own parent keep context rejection independent of storage conflicts.
                    let owner = publisher(snapshot.id, threshold, &unit);
                    allow_frontend(&owner, &unit);
                    let run = |phase, input, source_load| {
                        driver
                            .registry()
                            .execute_phase_with_resources(
                                phase,
                                input,
                                PhaseExecutionResources {
                                    diagnostics: Some(DiagnosticSink::new(
                                        DiagnosticProducerScope::new(
                                            if phase == PipelinePhase::SourceLoad {
                                                DiagnosticPhase::SourceLoad
                                            } else {
                                                DiagnosticPhase::Frontend
                                            },
                                            snapshot.id,
                                            "native.toolchain_context",
                                        ),
                                    )),
                                    output_publisher: Some(owner.clone()),
                                    source_load: Some(source_load),
                                    ..PhaseExecutionResources::default()
                                },
                            )
                            .unwrap()
                            .result
                    };
                    let dispatch = if phase == PipelinePhase::Frontend {
                        let parent = run(PipelinePhase::SourceLoad, source_input.clone(), inputs);
                        assert_eq!(parent.status, PhaseStatus::Complete);
                        assert!(parent.diagnostics.is_empty());
                        let handle = owner
                            .storage()
                            .typed_handle::<SourceUnit>(
                                &parent.output_refs[0],
                                &OutputKind::new("SourceUnit"),
                            )
                            .unwrap();
                        assert_eq!(*owner.storage().get(&handle).unwrap(), expected);
                        PhaseInput::new(
                            unit.clone(),
                            PhaseDispatchInputBundle::new(
                                snapshot.id,
                                source_input.identities().input_hash(),
                                vec![],
                                vec![
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
                                        snapshot.id,
                                        parent.output_refs[0].clone(),
                                    )
                                    .unwrap(),
                                ],
                            )
                            .unwrap(),
                        )
                    } else {
                        source_input.clone()
                    };
                    assert!(matches!(
                        driver
                            .registry()
                            .cache_key_for_phase(phase, &dispatch)
                            .unwrap()
                            .intent,
                        PhaseCacheIntent::NoKey { .. }
                    ));
                    let tested = SourceLoadInputs {
                        build_plan: if mismatch {
                            &foreign
                        } else {
                            inputs.build_plan
                        },
                        ..inputs
                    };
                    if mismatch && !open && phase == PipelinePhase::SourceLoad {
                        assert_eq!(
                            tested
                                .prepare_native_disk_cache_request(
                                    &dispatch,
                                    &mizar_proof::policy::VerifierPolicy::release()
                                )
                                .unwrap_err(),
                            "native Disk request context mismatch"
                        );
                    }
                    let result = run(phase, dispatch, tested);
                    if mismatch {
                        rejected.push((
                            open,
                            threshold,
                            phase,
                            result.status,
                            result.output_refs.is_empty(),
                            result.diagnostics.is_empty(),
                        ));
                    } else {
                        assert_eq!(result.status, PhaseStatus::Complete);
                        assert!(result.diagnostics.is_empty());
                        let output = &result.output_refs[0];
                        assert_eq!(
                            matches!(output.placement(), StoragePlacement::Blob { .. }),
                            threshold == 0
                        );
                        if phase == PipelinePhase::SourceLoad {
                            let handle = owner
                                .storage()
                                .typed_handle::<SourceUnit>(output, &OutputKind::new("SourceUnit"))
                                .unwrap();
                            assert_eq!(*owner.storage().get(&handle).unwrap(), expected);
                        } else {
                            let handle = owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(output, &OutputKind::new("FrontendOutput")).unwrap();
                            let payload = owner.storage().get(&handle).unwrap();
                            assert_eq!(payload.source, expected);
                            assert!(payload.ast.is_some());
                            assert!(payload.diagnostics.is_empty());
                        }
                    }
                }
            }
        }
    }
    assert!(
        rejected
            .iter()
            .all(
                |(_, _, _, status, no_output, no_diagnostics)| *status == PhaseStatus::Blocking
                    && *no_output
                    && *no_diagnostics
            ),
        "plan-only mismatches: {rejected:?}"
    );
}

#[test]
fn native_toolchain_context_blocks_discovery_before_inferred_import_edges() {
    let main = "import alpha.leaf;\ndefinition\nend;\n";
    let leaf = "definition\nend;\n";
    let mut rejected = Vec::new();
    for open in [false, true] {
        let fixture = Fixture::new(main.as_bytes());
        fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf).unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        request.dependency_inputs.toolchain = ToolchainInfo::new("opaque native/toolchain@context");
        input.plan_request.toolchain = request.dependency_inputs.toolchain.clone();
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
            ],
        }]);
        let descriptors = [("main", main), ("leaf", leaf)]
            .into_iter()
            .map(|(module, text)| {
                let path = format!("src/{module}.miz");
                (
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new(module),
                        normalized_path: normalize_path(
                            &fixture.root.join("alpha"),
                            Path::new(&path),
                        )
                        .unwrap(),
                        edition: Edition::new("2025"),
                        origin: if open {
                            SourceOriginInput::OpenBuffer {
                                uri: format!(
                                    "file://{}",
                                    fixture.root.join("alpha").join(&path).display()
                                ),
                                expected_version: 7,
                                actual_version: 7,
                                text: std::sync::Arc::from(text),
                            }
                        } else {
                            SourceOriginInput::Disk { path: path.into() }
                        },
                    },
                )
            })
            .collect();
        let session = request
            .capture_sources(&ids, &snapshots, descriptors)
            .unwrap();
        for mismatch in [false, true] {
            let mut current = DriverSubmitInput::new(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
                input.source_layout.clone(),
            );
            if mismatch {
                current.plan_request.toolchain =
                    ToolchainInfo::new("opaque native/toolchain@other-context");
            }
            let mut driver = CompilerDriver::new(source_registry());
            let result = driver.submit_captured_with_import_discovery(
                session.clone(),
                &ids,
                &snapshots,
                current,
            );
            if mismatch {
                rejected.push((
                    open,
                    match result {
                        Err(DriverSubmitError::TaskGraph {
                            session: failed,
                            diagnostics,
                        }) => {
                            assert_eq!(failed.captured().snapshot(), session.captured().snapshot());
                            assert_eq!(
                                failed.state(),
                                BuildSessionState::Finished(BuildSessionOutcome::Failed)
                            );
                            assert!(
                                diagnostics
                                    .diagnostics()
                                    .iter()
                                    .all(|diagnostic| diagnostic.kind
                                        == TaskGraphDiagnosticKind::MissingModuleDependencyOverlay)
                            );
                            !diagnostics.diagnostics().is_empty()
                        }
                        Ok(_) => false,
                        Err(error) => panic!("unexpected discovery result: {error:?}"),
                    },
                ));
            } else {
                let submission = result.unwrap();
                let graph = submission.task_graph.as_ref().unwrap();
                let frontend = |name| {
                    graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
                };
                assert!(frontend("main").dependencies.contains(&frontend("leaf").id));
                assert_eq!(
                    submission.session.captured().snapshot(),
                    session.captured().snapshot()
                );
            }
        }
    }
    assert!(
        rejected.iter().all(|(_, rejected)| *rejected),
        "plan-only mismatch discovery: {rejected:?}"
    );
}
