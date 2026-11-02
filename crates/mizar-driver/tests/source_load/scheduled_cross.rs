use super::*;

#[test]
fn scheduled_cross_package_workspace_parent_uses_target_owned_lexical_identity() {
    let mut active_hashes = Vec::new();
    for (core_version, chain, mutation) in [
        ("1.2.0", false, "none"),
        ("2.0.0", false, "none"),
        ("1.2.0", true, "none"),
        ("1.2.0", false, "plan_version"),
        ("1.2.0", false, "index_version"),
        ("1.2.0", false, "index_root"),
        ("1.2.0", false, "source_version"),
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        fs::create_dir_all(fixture.root.join("app/src")).unwrap();
        fs::create_dir_all(fixture.root.join("core/src")).unwrap();
        fs::write(
            fixture.root.join("app/src/main.miz"),
            if chain {
                b"import app.mid;\ntheorem T: a arrange b = a;\n".as_slice()
            } else {
                b"import core.base;\ntheorem T: a combine b = a;\n".as_slice()
            },
        )
        .unwrap();
        if chain {
            fs::write(
                fixture.root.join("app/src/mid.miz"),
                b"import core.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n",
            )
            .unwrap();
        }
        fs::write(
            fixture.root.join("core/src/base.miz"),
            b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        )
        .unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let version = |package: &str, module: &str| {
            let root = fixture.root.join(package);
            let relative = format!("src/{module}.miz");
            let loaded = mizar_session::SourceLoader::load(
                &DiskSourceLoader::new(&root),
                snapshot_id(0x52),
                SourceInput {
                    package_id: PackageId::new(package),
                    module_path: ModulePath::new(module),
                    normalized_path: normalize_path(&root, Path::new(&relative)).unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: PathBuf::from(&relative),
                    },
                },
                &ids,
            )
            .unwrap();
            SourceVersion {
                source_id: loaded.source_id,
                package_id: loaded.package_id,
                module_path: loaded.module_path,
                normalized_path: loaded.normalized_path,
                source_hash: loaded.source_hash,
                edition: loaded.edition,
                origin: loaded.origin,
            }
        };
        let mut versions = vec![version("app", "main"), version("core", "base")];
        if chain {
            versions.push(version("app", "mid"));
        }
        let make_request = || {
            let (mut request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
            request.source_inputs.versions = versions.clone();
            input.workspace_packages = vec![
                WorkspacePackage {
                    member_path: "app".to_owned(),
                    manifest: parse_package_manifest(&format!(
                        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\ncore = \"{core_version}\"\n"
                    )).unwrap(),
                },
                WorkspacePackage {
                    member_path: "core".to_owned(),
                    manifest: parse_package_manifest(&format!(
                        "[package]\nname = \"core\"\nversion = \"{core_version}\"\n"
                    )).unwrap(),
                },
            ];
            input.lockfile = parse_lockfile(&format!(
                "schema_version = 1\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"app\" }}\ndependencies = [{{ name = \"core\", version = \"{core_version}\" }}]\n[[package]]\nname = \"core\"\nversion = \"{core_version}\"\nsource = {{ kind = \"workspace\", path = \"core\" }}\ndependencies = []\n"
            )).unwrap();
            let mut app_files = vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")];
            if chain {
                app_files.push(WorkspaceSourceFile::new("src/mid.miz", "mid.miz"));
            }
            input.source_layout = StaticSourceLayout::new(vec![
                WorkspaceSourcePackage {
                    package_id: PackageId::new("app"),
                    files: app_files,
                },
                WorkspaceSourcePackage {
                    package_id: PackageId::new("core"),
                    files: vec![WorkspaceSourceFile::new("src/base.miz", "base.miz")],
                },
            ]);
            let app_main = mizar_build::module_index::ModuleId::new(
                PackageId::new("app"),
                ModulePath::new("main"),
            );
            let core_base = mizar_build::module_index::ModuleId::new(
                PackageId::new("core"),
                ModulePath::new("base"),
            );
            let mut edges = Vec::new();
            if chain {
                let app_mid = mizar_build::module_index::ModuleId::new(
                    PackageId::new("app"),
                    ModulePath::new("mid"),
                );
                edges.push(ModuleDependencyEdge::new(
                    app_main,
                    app_mid.clone(),
                    ModuleDependencyKind::ImportSummary,
                ));
                edges.push(ModuleDependencyEdge::new(
                    app_mid,
                    core_base,
                    ModuleDependencyKind::ImportSummary,
                ));
            } else {
                edges.push(ModuleDependencyEdge::new(
                    app_main,
                    core_base,
                    ModuleDependencyKind::ImportSummary,
                ));
            }
            input.dependency_overlay = ModuleDependencyOverlay::complete(edges);
            (request, input)
        };
        let (request, input) = make_request();
        let baseline = CompilerDriver::new(source_registry())
            .submit(request, &ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            baseline.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let tasks = baseline.task_graph.as_ref().unwrap().tasks();
        let first = tasks
            .iter()
            .find(|task| task.kind == TaskKind::SourceLoad)
            .unwrap();
        let publisher = publisher(
            baseline.session.captured().snapshot().id,
            usize::MAX,
            &first.unit,
        );
        for task in tasks
            .iter()
            .filter(|task| task.kind == TaskKind::SourceLoad)
        {
            let WorkUnit::Module { module } = &task.unit else {
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
            allow_frontend(&publisher, &task.unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let registry = builder.build().unwrap();
        if mutation != "none" {
            let mut direct = WorkspaceLeafFixture {
                fixture,
                ids,
                submission: baseline,
                publisher,
            };
            let core_source = direct.source("base");
            assert_eq!(core_source.status, PhaseStatus::Complete);
            let core_frontend = direct.frontend("base", core_source.output_refs, None);
            let mut forged_snapshot = direct.submission.session.captured().snapshot().clone();
            assert_eq!(core_frontend.status, PhaseStatus::Complete);
            let app_source = direct.source("main");
            assert_eq!(app_source.status, PhaseStatus::Complete);
            match mutation {
                "plan_version" => {
                    direct
                        .submission
                        .build_plan
                        .as_mut()
                        .unwrap()
                        .packages
                        .iter_mut()
                        .find(|entry| entry.package_id.as_str() == "core")
                        .unwrap()
                        .version = "9.9.9".parse().unwrap();
                }
                "index_version" => {
                    direct
                        .submission
                        .module_index
                        .as_mut()
                        .unwrap()
                        .packages
                        .iter_mut()
                        .find(|entry| entry.package_id.as_str() == "core")
                        .unwrap()
                        .version = "9.9.9".parse().unwrap();
                }
                "index_root" => {
                    let entry = direct
                        .submission
                        .module_index
                        .as_mut()
                        .unwrap()
                        .packages
                        .iter_mut()
                        .find(|entry| entry.package_id.as_str() == "core")
                        .unwrap();
                    let mizar_build::module_index::PackageIndexSource::Workspace {
                        package_root,
                        ..
                    } = &mut entry.source
                    else {
                        unreachable!()
                    };
                    *package_root = "app".to_owned();
                }
                "source_version" => {
                    forged_snapshot
                        .source_versions
                        .iter_mut()
                        .find(|version| version.package_id.as_str() == "core")
                        .unwrap()
                        .source_hash = hash(0xee);
                }
                _ => unreachable!(),
            }
            let result = direct.frontend(
                "main",
                vec![
                    app_source.output_refs[0].clone(),
                    core_frontend.output_refs[0].clone(),
                ],
                Some(&forged_snapshot),
            );
            assert_eq!(result.status, PhaseStatus::Blocking, "{mutation}");
            assert!(result.output_refs.is_empty(), "{mutation}");
            continue;
        }
        let mut driver = CompilerDriver::new(registry).with_output_publisher(publisher.clone());
        let (request, mut input) = make_request();
        input.worker_count = 4;
        let submission = driver
            .submit(request, &ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            baseline.session.captured().snapshot().id
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let graph = submission.task_graph.as_ref().unwrap();
        let task = |kind, package: &str, module: &str| {
            graph
                .tasks()
                .iter()
                .find(|task| {
                    task.kind == kind
                        && matches!(&task.unit,
                WorkUnit::Module { module: id }
                if id.package.as_str() == package && id.path.as_str() == module)
                })
                .unwrap()
        };
        let core = task(TaskKind::Frontend, "core", "base");
        let app = task(TaskKind::Frontend, "app", "main");
        assert_eq!(run.phase_results[&core.id][0].status, PhaseStatus::Complete);
        if chain {
            assert_eq!(
                run.phase_results[&task(TaskKind::Frontend, "app", "mid").id][0].status,
                PhaseStatus::Complete
            );
        }
        assert_eq!(run.phase_results[&app.id][0].status, PhaseStatus::Complete);
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let core_output = &run.phase_results[&core.id][0].output_refs[0];
        let app_output = &run.phase_results[&app.id][0].output_refs[0];
        let own_source =
            &run.phase_results[&task(TaskKind::SourceLoad, "app", "main").id][0].output_refs[0];
        let lineage = publisher
            .registry()
            .output_lineage(app_output.output())
            .unwrap();
        assert_eq!(lineage.parents.len(), 2);
        assert!(lineage.parents.contains(&own_source.output()));
        if chain {
            let mid =
                &run.phase_results[&task(TaskKind::Frontend, "app", "mid").id][0].output_refs[0];
            let mid_source =
                &run.phase_results[&task(TaskKind::SourceLoad, "app", "mid").id][0].output_refs[0];
            assert!(lineage.parents.contains(&mid.output()));
            assert!(!lineage.parents.contains(&core_output.output()));
            let mid_lineage = publisher.registry().output_lineage(mid.output()).unwrap();
            assert_eq!(mid_lineage.parents.len(), 2);
            assert!(mid_lineage.parents.contains(&mid_source.output()));
            assert!(mid_lineage.parents.contains(&core_output.output()));
            let typed_mid = publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    mid,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded_mid = publisher.storage().get(&typed_mid).unwrap();
            assert!(
                loaded_mid
                    .tokens
                    .tokens()
                    .iter()
                    .any(|token| token.text.as_ref() == "combine"
                        && token.kind == TokenKind::UserSymbol)
            );
        } else {
            assert!(lineage.parents.contains(&core_output.output()));
        }
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                app_output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = publisher.storage().get(&typed).unwrap();
        let spelling = if chain { "arrange" } else { "combine" };
        assert!(loaded.tokens.tokens().iter().any(|token| token.text.as_ref() == spelling
            && token.kind == TokenKind::UserSymbol));
        if !chain {
            active_hashes.push(loaded.cache_keys.active_lexical_environment.stable_hash());
        }
    }
    assert_ne!(active_hashes[0], active_hashes[1]);
}
