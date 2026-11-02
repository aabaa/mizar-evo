use super::*;

#[test]
fn explicit_native_lock_content_association_preserves_real_source_consumers() {
    use mizar_build::planner::{lockfile_content_hash, read_native_workspace_inputs};

    // Spec §23.2/23.4: elect the existing typed lock profile explicitly.
    let lock_text = "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }, { name = \"other\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n[[package]]\nname = \"other\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:other\" }\ndependencies = []\n";
    let changed_locks = [
        "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }, { name = \"other\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"alternate\", checksum = \"sha256:dep\" }\ndependencies = []\n[[package]]\nname = \"other\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:other\" }\ndependencies = []\n",
        "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }, { name = \"other\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:changed\" }\ndependencies = []\n[[package]]\nname = \"other\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:other\" }\ndependencies = []\n",
    ];
    let main = "import alpha.leaf;\ndefinition\nend;\n";
    let leaf = "\u{feff}definition\r\nend;\r\n";
    for open in [false, true] {
        for threshold in [usize::MAX, 0] {
            let fixture = Fixture::new(main.as_bytes());
            fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf).unwrap();
            fs::write(
                fixture.root.join("mizar.workspace"),
                "[workspace]\nmembers = [\"alpha\"]\n",
            )
            .unwrap();
            fs::write(fixture.root.join("alpha/mizar.pkg"), "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dev-dependencies]\ndep = \"^1.0\"\nother = \"^1.0\"\n").unwrap();
            fs::write(fixture.root.join("mizar.lock"), lock_text).unwrap();
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
            let (packages, lock) =
                read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap();
            assert_eq!(lock, parse_lockfile(lock_text).unwrap());
            let digest = lockfile_content_hash(&lock);
            request.dependency_inputs.lockfile_hash = digest;
            request.source_inputs.versions.clear();
            input.workspace_packages = packages;
            input.lockfile = lock;
            input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![
                    WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                    WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
                ],
            }]);
            let descriptors = [("leaf", leaf), ("main", main)]
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
                .collect::<Vec<_>>();
            let session = request
                .clone()
                .capture_sources(&ids, &snapshots, descriptors.clone())
                .unwrap();
            let before = session.captured().snapshot().clone();
            let expected = mizar_frontend::source::source_unit_from_loaded(
                mizar_session::SourceLoader::load(
                    &descriptors[0].0,
                    before.id,
                    descriptors[0].1.clone(),
                    &InMemorySessionIdAllocator::new(),
                )
                .unwrap(),
                mizar_frontend::source::diagnostic_file_path(&descriptors[0].1),
            );
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver = CompilerDriver::new(builder.build().unwrap());
            let submission = driver
                .submit_captured_with_import_discovery(
                    session,
                    &ids,
                    &snapshots,
                    DriverSubmitInput::new(
                        input.plan_request.clone(),
                        input.workspace_packages.clone(),
                        input.lockfile.clone(),
                        input.source_layout.clone(),
                    ),
                )
                .unwrap();
            let captured = submission.session.captured();
            let snapshot = captured.snapshot();
            let graph = submission.task_graph.as_ref().unwrap();
            let frontend = |name| {
                graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
            };
            assert!(frontend("main").dependencies.contains(&frontend("leaf").id));
            let unit = frontend("leaf").unit.clone();
            let inputs = SourceLoadInputs {
                captured: Some(captured),
                snapshot,
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let plan_before = inputs.build_plan.clone();
            assert_eq!(snapshot.lockfile_hash, digest);
            assert_eq!(inputs.validate_native_lock_content(), Ok(()));
            let mut summary_request = request.clone();
            summary_request.source_inputs.versions = snapshot.source_versions.clone();
            let summary = summary_request
                .allocate(&ids)
                .unwrap()
                .capture_snapshot(&snapshots)
                .unwrap();
            assert_eq!(summary.captured().snapshot(), snapshot);
            let mut foreign_request = request.clone();
            foreign_request.verifier_config = VerifierConfigInput::new(hash(9));
            let foreign = foreign_request
                .capture_sources(
                    &InMemorySessionIdAllocator::new(),
                    &SnapshotRegistry::new(),
                    descriptors.clone(),
                )
                .unwrap();
            assert_ne!(foreign.captured().snapshot(), snapshot);
            let mut borrowed = snapshot.clone();
            borrowed.verifier_config_hash = hash(9);
            let mut other_toolchain = plan_before.clone();
            other_toolchain.toolchain = ToolchainInfo::new("opaque other toolchain");
            for rejected in [
                SourceLoadInputs {
                    captured: None,
                    ..inputs
                },
                SourceLoadInputs {
                    captured: Some(summary.captured()),
                    ..inputs
                },
                SourceLoadInputs {
                    captured: Some(foreign.captured()),
                    ..inputs
                },
                SourceLoadInputs {
                    snapshot: &borrowed,
                    ..inputs
                },
                SourceLoadInputs {
                    build_plan: &other_toolchain,
                    ..inputs
                },
            ] {
                assert!(rejected.validate_native_lock_content().is_err());
            }
            for text in changed_locks {
                let changed_lock = parse_lockfile(text).unwrap();
                assert_ne!(lockfile_content_hash(&changed_lock), digest);
                let changed_plan = produce_build_plan(
                    input.plan_request.clone(),
                    input.workspace_packages.clone(),
                    changed_lock,
                )
                .unwrap();
                let mut expected_plan = plan_before.clone();
                expected_plan.lockfile = changed_plan.lockfile.clone();
                assert_eq!(
                    changed_plan, expected_plan,
                    "only typed lock content differs"
                );
                assert!(
                    SourceLoadInputs {
                        build_plan: &changed_plan,
                        ..inputs
                    }
                    .validate_native_lock_content()
                    .is_err()
                );
            }
            let mut reordered_lock = plan_before.lockfile.clone();
            reordered_lock.packages.reverse();
            for package in &mut reordered_lock.packages {
                package.dependencies.reverse();
            }
            let reordered_plan = produce_build_plan(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                reordered_lock,
            )
            .unwrap();
            assert_ne!(reordered_plan.lockfile, plan_before.lockfile);
            assert_eq!(lockfile_content_hash(&reordered_plan.lockfile), digest);
            assert_eq!(
                SourceLoadInputs {
                    build_plan: &reordered_plan,
                    ..inputs
                }
                .validate_native_lock_content(),
                Ok(())
            );
            let mut changed_hash_request = request;
            changed_hash_request.dependency_inputs.lockfile_hash =
                lockfile_content_hash(&parse_lockfile(changed_locks[0]).unwrap());
            let changed_hash = changed_hash_request
                .capture_sources(
                    &InMemorySessionIdAllocator::new(),
                    &SnapshotRegistry::new(),
                    descriptors,
                )
                .unwrap();
            assert_eq!(
                changed_hash.captured().snapshot().source_versions,
                snapshot.source_versions
            );
            assert!(
                SourceLoadInputs {
                    captured: Some(changed_hash.captured()),
                    snapshot: changed_hash.captured().snapshot(),
                    ..inputs
                }
                .validate_native_lock_content()
                .is_err()
            );

            let owner = publisher(snapshot.id, threshold, &unit);
            allow_frontend(&owner, &unit);
            let source_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot.id,
                    source_key(&snapshot.source_versions[0]),
                    vec![],
                ),
            );
            let run = |phase, dispatch| {
                driver
                    .registry()
                    .execute_phase_with_resources(
                        phase,
                        dispatch,
                        PhaseExecutionResources {
                            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                                if phase == PipelinePhase::SourceLoad {
                                    DiagnosticPhase::SourceLoad
                                } else {
                                    DiagnosticPhase::Frontend
                                },
                                snapshot.id,
                                "native.lock_content_association",
                            ))),
                            output_publisher: Some(owner.clone()),
                            source_load: Some(inputs),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap()
                    .result
            };
            let source = run(PipelinePhase::SourceLoad, source_input.clone());
            assert_eq!(source.status, PhaseStatus::Complete);
            assert!(source.diagnostics.is_empty());
            assert_eq!(source.output_refs.len(), 1);
            let handle = owner
                .storage()
                .typed_handle::<SourceUnit>(&source.output_refs[0], &OutputKind::new("SourceUnit"))
                .unwrap();
            assert_eq!(*owner.storage().get(&handle).unwrap(), expected);
            let frontend_input = PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::new(
                    snapshot.id,
                    source_input.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            snapshot.id,
                            source.output_refs[0].clone(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            let parsed = run(PipelinePhase::Frontend, frontend_input.clone());
            assert_eq!(parsed.status, PhaseStatus::Complete);
            assert!(parsed.diagnostics.is_empty());
            assert_eq!(parsed.output_refs.len(), 1);
            let handle = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &parsed.output_refs[0],
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let payload = owner.storage().get(&handle).unwrap();
            assert_eq!(payload.source, expected);
            assert!(payload.ast.is_some());
            assert!(payload.diagnostics.is_empty());
            for (phase, dispatch, result) in [
                (PipelinePhase::SourceLoad, &source_input, &source),
                (PipelinePhase::Frontend, &frontend_input, &parsed),
            ] {
                assert_eq!(
                    matches!(
                        result.output_refs[0].placement(),
                        StoragePlacement::Blob { .. }
                    ),
                    threshold == 0
                );
                assert!(matches!(
                    driver
                        .registry()
                        .cache_key_for_phase(phase, dispatch)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey { .. }
                ));
            }
            let payload_before = (*payload).clone();
            assert_eq!(inputs.validate_native_lock_content(), Ok(()));
            assert_eq!(captured.snapshot(), &before);
            assert_eq!(inputs.build_plan, &plan_before);
            assert_eq!(*owner.storage().get(&handle).unwrap(), payload_before);
        }
    }
}

#[test]
fn explicit_native_config_content_association_preserves_real_source_consumers() {
    use mizar_build::planner::{read_native_workspace_inputs, workspace_config_content_hash};

    // Spec §23.1/23.4: elect the existing package configuration profile explicitly.
    let lock_text = "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n[[package]]\nname = \"beta\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"beta\" }\ndependencies = []\n";
    let manifest = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n";
    let beta_manifest = "[package]\nname = \"beta\"\nversion = \"0.1.0\"\n[verifier]\nmax_axioms = 64\n[build]\ncache_dir = \"beta-cache\"\n";
    let main = "import alpha.leaf;\ndefinition\nend;\n";
    let leaf = "\u{feff}definition\r\nend;\r\n";
    for open in [false, true] {
        for threshold in [usize::MAX, 0] {
            let fixture = Fixture::new(main.as_bytes());
            fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf).unwrap();
            fs::write(
                fixture.root.join("mizar.workspace"),
                "[workspace]\nmembers = [\"alpha\", \"beta\"]\n",
            )
            .unwrap();
            fs::create_dir_all(fixture.root.join("beta/src")).unwrap();
            fs::write(fixture.root.join("alpha/mizar.pkg"), manifest).unwrap();
            fs::write(fixture.root.join("beta/mizar.pkg"), beta_manifest).unwrap();
            fs::write(fixture.root.join("mizar.lock"), lock_text).unwrap();
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
            let (packages, lock) =
                read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap();
            let explicit = vec![
                WorkspacePackage {
                    member_path: "alpha".into(),
                    manifest: parse_package_manifest(manifest).unwrap(),
                },
                WorkspacePackage {
                    member_path: "beta".into(),
                    manifest: parse_package_manifest(beta_manifest).unwrap(),
                },
            ];
            assert_eq!(packages, explicit);
            assert_eq!(lock, parse_lockfile(lock_text).unwrap());
            let plan =
                produce_build_plan(input.plan_request.clone(), packages.clone(), lock.clone())
                    .unwrap();
            assert_eq!(
                plan,
                produce_build_plan(
                    input.plan_request.clone(),
                    explicit,
                    parse_lockfile(lock_text).unwrap(),
                )
                .unwrap()
            );
            let digest = workspace_config_content_hash(&plan.verifier_config, &plan.build_config);
            request.verifier_config = VerifierConfigInput::new(digest);
            request.source_inputs.versions.clear();
            input.workspace_packages = packages;
            input.lockfile = lock;
            input.source_layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
            let descriptors = [("leaf", leaf), ("main", main)]
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
                .collect::<Vec<_>>();
            let session = request
                .clone()
                .capture_sources(&ids, &snapshots, descriptors.clone())
                .unwrap();
            let before = session.captured().snapshot().clone();
            let expected = mizar_frontend::source::source_unit_from_loaded(
                mizar_session::SourceLoader::load(
                    &descriptors[0].0,
                    before.id,
                    descriptors[0].1.clone(),
                    &InMemorySessionIdAllocator::new(),
                )
                .unwrap(),
                mizar_frontend::source::diagnostic_file_path(&descriptors[0].1),
            );
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver = CompilerDriver::new(builder.build().unwrap());
            let submission = driver
                .submit_captured_with_import_discovery(
                    session,
                    &ids,
                    &snapshots,
                    DriverSubmitInput::new(
                        input.plan_request.clone(),
                        input.workspace_packages.clone(),
                        input.lockfile.clone(),
                        input.source_layout.clone(),
                    ),
                )
                .unwrap();
            let captured = submission.session.captured();
            let snapshot = captured.snapshot();
            let graph = submission.task_graph.as_ref().unwrap();
            let frontend = |name| {
                graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
            };
            assert!(frontend("main").dependencies.contains(&frontend("leaf").id));
            let unit = frontend("leaf").unit.clone();
            let inputs = SourceLoadInputs {
                captured: Some(captured),
                snapshot,
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let plan_before = inputs.build_plan.clone();
            assert_eq!(plan_before, plan);
            assert_eq!(snapshot.verifier_config_hash, digest);
            assert_eq!(inputs.validate_native_config_content(), Ok(()));
            let mut summary_request = request.clone();
            summary_request.source_inputs.versions = snapshot.source_versions.clone();
            let summary = summary_request
                .allocate(&ids)
                .unwrap()
                .capture_snapshot(&snapshots)
                .unwrap();
            assert_eq!(summary.captured().snapshot(), snapshot);
            let mut foreign_request = request.clone();
            foreign_request.verifier_config = VerifierConfigInput::new(hash(9));
            let foreign = foreign_request
                .capture_sources(
                    &InMemorySessionIdAllocator::new(),
                    &SnapshotRegistry::new(),
                    descriptors.clone(),
                )
                .unwrap();
            assert_ne!(foreign.captured().snapshot(), snapshot);
            let mut borrowed = snapshot.clone();
            borrowed.lockfile_hash = hash(9);
            let mut other_toolchain = plan_before.clone();
            other_toolchain.toolchain = ToolchainInfo::new("opaque other toolchain");
            let rejected_inputs = [
                SourceLoadInputs {
                    captured: None,
                    ..inputs
                },
                SourceLoadInputs {
                    captured: Some(summary.captured()),
                    ..inputs
                },
                SourceLoadInputs {
                    captured: Some(foreign.captured()),
                    ..inputs
                },
                SourceLoadInputs {
                    snapshot: &borrowed,
                    ..inputs
                },
                SourceLoadInputs {
                    build_plan: &other_toolchain,
                    ..inputs
                },
            ];
            for rejected in rejected_inputs {
                let message = if rejected.captured.is_none() {
                    "native config content requires captured sources"
                } else {
                    "native config content context mismatch"
                };
                assert_eq!(
                    rejected.validate_native_config_content(),
                    Err(message.into())
                );
            }
            let mut changed_digest = None;
            for suffix in [
                "[verifier]\ncomputation_replay = true\n",
                "[build]\ncache_dir = \"changed-cache\"\n",
            ] {
                fs::write(
                    fixture.root.join("alpha/mizar.pkg"),
                    format!("{manifest}{suffix}"),
                )
                .unwrap();
                let (changed_packages, changed_lock) =
                    read_native_workspace_inputs(&input.plan_request.workspace_root).unwrap();
                let changed_plan =
                    produce_build_plan(input.plan_request.clone(), changed_packages, changed_lock)
                        .unwrap();
                let mut normalized = changed_plan.clone();
                normalized.verifier_config = plan_before.verifier_config.clone();
                normalized.build_config = plan_before.build_config.clone();
                for (package, original) in normalized.packages.iter_mut().zip(&plan_before.packages)
                {
                    package.verifier_config = original.verifier_config.clone();
                    package.build_config = original.build_config.clone();
                }
                assert_eq!(normalized, plan_before, "only typed configuration differs");
                let changed = workspace_config_content_hash(
                    &changed_plan.verifier_config,
                    &changed_plan.build_config,
                );
                assert_ne!(changed, digest);
                changed_digest = Some(changed);
                let mut changed_toolchain = changed_plan.clone();
                changed_toolchain.toolchain = other_toolchain.toolchain.clone();
                for rejected in rejected_inputs {
                    let message = if rejected.captured.is_none() {
                        "native config content requires captured sources"
                    } else {
                        "native config content context mismatch"
                    };
                    assert_eq!(
                        SourceLoadInputs {
                            build_plan: if rejected.build_plan == &other_toolchain {
                                &changed_toolchain
                            } else {
                                &changed_plan
                            },
                            ..rejected
                        }
                        .validate_native_config_content(),
                        Err(message.into())
                    );
                }
                assert_eq!(
                    SourceLoadInputs {
                        build_plan: &changed_plan,
                        ..inputs
                    }
                    .validate_native_config_content(),
                    Err("native config content mismatch".into())
                );
            }
            assert_eq!(plan_before.verifier_config.packages.len(), 2);
            assert_eq!(plan_before.build_config.packages.len(), 2);
            for (verifier, build) in [(true, false), (false, true), (true, true)] {
                let mut reordered_plan = plan_before.clone();
                if verifier {
                    reordered_plan.verifier_config.packages.reverse();
                }
                if build {
                    reordered_plan.build_config.packages.reverse();
                }
                assert_ne!(reordered_plan, plan_before);
                assert_eq!(
                    workspace_config_content_hash(
                        &reordered_plan.verifier_config,
                        &reordered_plan.build_config,
                    ),
                    digest
                );
                assert_eq!(
                    SourceLoadInputs {
                        build_plan: &reordered_plan,
                        ..inputs
                    }
                    .validate_native_config_content(),
                    Ok(())
                );
            }
            let mut changed_hash_request = request;
            changed_hash_request.verifier_config =
                VerifierConfigInput::new(changed_digest.unwrap());
            let changed_hash = changed_hash_request
                .capture_sources(
                    &InMemorySessionIdAllocator::new(),
                    &SnapshotRegistry::new(),
                    descriptors,
                )
                .unwrap();
            let mut normalized_snapshot = changed_hash.captured().snapshot().clone();
            normalized_snapshot.id = snapshot.id;
            normalized_snapshot.verifier_config_hash = snapshot.verifier_config_hash;
            assert_eq!(
                &normalized_snapshot, snapshot,
                "only produced config identity differs"
            );
            assert_eq!(
                SourceLoadInputs {
                    captured: Some(changed_hash.captured()),
                    snapshot: changed_hash.captured().snapshot(),
                    ..inputs
                }
                .validate_native_config_content(),
                Err("native config content mismatch".into())
            );
            for path in [
                "mizar.workspace",
                "mizar.lock",
                "alpha/mizar.pkg",
                "beta/mizar.pkg",
                "alpha/src/main.miz",
                "alpha/src/leaf.miz",
            ] {
                fs::remove_file(fixture.root.join(path)).unwrap();
            }
            assert_eq!(inputs.validate_native_config_content(), Ok(()));

            let owner = publisher(snapshot.id, threshold, &unit);
            allow_frontend(&owner, &unit);
            let source_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot.id,
                    source_key(&snapshot.source_versions[0]),
                    vec![],
                ),
            );
            let run = |phase, dispatch| {
                driver
                    .registry()
                    .execute_phase_with_resources(
                        phase,
                        dispatch,
                        PhaseExecutionResources {
                            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                                if phase == PipelinePhase::SourceLoad {
                                    DiagnosticPhase::SourceLoad
                                } else {
                                    DiagnosticPhase::Frontend
                                },
                                snapshot.id,
                                "native.config_content_association",
                            ))),
                            output_publisher: Some(owner.clone()),
                            source_load: Some(inputs),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap()
                    .result
            };
            let source = run(PipelinePhase::SourceLoad, source_input.clone());
            assert_eq!(source.status, PhaseStatus::Complete);
            assert!(source.diagnostics.is_empty());
            assert_eq!(source.output_refs.len(), 1);
            let handle = owner
                .storage()
                .typed_handle::<SourceUnit>(&source.output_refs[0], &OutputKind::new("SourceUnit"))
                .unwrap();
            assert_eq!(*owner.storage().get(&handle).unwrap(), expected);
            let frontend_input = PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::new(
                    snapshot.id,
                    source_input.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            snapshot.id,
                            source.output_refs[0].clone(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            let parsed = run(PipelinePhase::Frontend, frontend_input.clone());
            assert_eq!(parsed.status, PhaseStatus::Complete);
            assert!(parsed.diagnostics.is_empty());
            assert_eq!(parsed.output_refs.len(), 1);
            let handle = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &parsed.output_refs[0],
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            assert_eq!(
                parsed.output_refs[0].lineage().parents,
                vec![source.output_refs[0].output()]
            );
            let payload = owner.storage().get(&handle).unwrap();
            let oracle = mizar_frontend::orchestration::Frontend::new(
                mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
                    fixture.root.join("alpha"),
                )),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(expected.clone())
            .unwrap();
            assert_eq!(*payload, oracle);
            assert!(payload.ast.is_some());
            assert!(payload.diagnostics.is_empty());
            for (phase, dispatch, result) in [
                (PipelinePhase::SourceLoad, &source_input, &source),
                (PipelinePhase::Frontend, &frontend_input, &parsed),
            ] {
                assert_eq!(
                    matches!(
                        result.output_refs[0].placement(),
                        StoragePlacement::Blob { .. }
                    ),
                    threshold == 0
                );
                assert!(matches!(
                    driver
                        .registry()
                        .cache_key_for_phase(phase, dispatch)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey { .. }
                ));
            }
            let payload_before = (*payload).clone();
            assert_eq!(inputs.validate_native_config_content(), Ok(()));
            assert_eq!(captured.snapshot(), &before);
            assert_eq!(inputs.build_plan, &plan_before);
            assert_eq!(*owner.storage().get(&handle).unwrap(), payload_before);
        }
    }
}
