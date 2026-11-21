use super::*;

#[test]
fn native_workspace_inputs_feed_protected_prefix_like_explicit_clean_inputs() {
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
    let root = WorkspaceRoot::new(fixture.root.join("alpha").to_str().unwrap());
    input.plan_request.workspace_root = root.clone();
    request.workspace_root = root.clone();
    let manifest = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n";
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
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                current,
                Default::default(),
            )
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
        let phase_input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot,
                SourceUnitCacheKey::from_source(&source).stable_hash(),
                vec![],
            ),
        );
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
        let actual = (
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
fn native_workspace_inputs_drive_listed_cross_package_import_discovery() {
    let fixture = Fixture::new(b"definition\nend;\n");
    for (path, bytes) in [
        ("app/src/main.miz", b"import core.groups.base;\ntheorem T: a combine b = a;\n".as_slice()),
        ("core/src/groups/base.miz", b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice()),
    ] {
        fs::create_dir_all(fixture.root.join(path).parent().unwrap()).unwrap();
        fs::write(fixture.root.join(path), bytes).unwrap();
    }
    let (mut request, input) = fixture.submit_request(vec![], vec![]);
    let app = "[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\ncore = \"1.0.0\"\n";
    let core = "[package]\nname = \"core\"\nversion = \"1.0.0\"\n";
    let workspace = "[workspace]\nmembers = [\"core\", \"app\"]\n";
    let lock = "schema_version = 1\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"app\" }\ndependencies = [{ name = \"core\", version = \"1.0.0\" }]\n[[package]]\nname = \"core\"\nversion = \"1.0.0\"\nsource = { kind = \"workspace\", path = \"core\" }\ndependencies = []\n";
    for (path, text) in [
        ("mizar.workspace", workspace),
        ("mizar.lock", lock),
        ("app/mizar.pkg", app),
        ("core/mizar.pkg", core),
        ("mizar.pkg", "malformed unlisted root package"),
    ] {
        fs::write(fixture.root.join(path), text).unwrap();
    }
    let acquired =
        mizar_build::planner::read_native_workspace_inputs(&input.plan_request.workspace_root)
            .unwrap();
    let explicit_inputs = (
        mizar_build::planner::parse_workspace_manifest(workspace)
            .unwrap()
            .members
            .into_iter()
            .map(|member_path| WorkspacePackage {
                manifest: parse_package_manifest(if member_path == "app" { app } else { core })
                    .unwrap(),
                member_path,
            })
            .collect::<Vec<_>>(),
        parse_lockfile(lock).unwrap(),
    );
    assert_eq!(acquired, explicit_inputs);
    let plan = produce_build_plan(
        input.plan_request.clone(),
        acquired.0.clone(),
        acquired.1.clone(),
    )
    .unwrap();
    let native = StaticSourceLayout::from_native_disk(&plan).unwrap();
    let explicit = StaticSourceLayout::new(vec![
        WorkspaceSourcePackage {
            package_id: PackageId::new("app"),
            files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
        },
        WorkspaceSourcePackage {
            package_id: PackageId::new("core"),
            files: vec![WorkspaceSourceFile::new(
                "src/groups/base.miz",
                "groups/base.miz",
            )],
        },
    ]);
    assert_eq!(native, explicit);
    assert_eq!(
        build_module_index(&plan, &native, &[]).unwrap(),
        build_module_index(&plan, &explicit, &[]).unwrap()
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    request.source_inputs.versions.clear();
    let descriptors = [
        ("app", "main", "src/main.miz"),
        ("core", "groups.base", "src/groups/base.miz"),
    ]
    .into_iter()
    .map(|(package, module, path)| {
        let root = fixture.root.join(package);
        (
            DiskSourceLoader::new(&root),
            SourceInput {
                package_id: PackageId::new(package),
                module_path: ModulePath::new(module),
                normalized_path: normalize_path(&root, Path::new(path)).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk { path: path.into() },
            },
        )
    })
    .collect();
    let session = request
        .capture_sources(&ids, &snapshots, descriptors)
        .unwrap();
    let snapshot = session.captured().snapshot().id;
    let mut clean = None;
    for ((packages, lockfile), layout, workers, order) in [
        (explicit_inputs, explicit, 1, CompletionOrder::Canonical),
        (acquired, native, 4, CompletionOrder::Reverse),
    ] {
        let mut current =
            DriverSubmitInput::new(input.plan_request.clone(), packages, lockfile, layout);
        current.worker_count = workers;
        current.completion_order = order;
        let main = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("app"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, 0, &main);
        for (package, module) in [("app", "main"), ("core", "groups.base")] {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new(package),
                    ModulePath::new(module),
                ),
            };
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!("{package:?}:{module:?}")),
            ));
            allow_frontend(&owner, &unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                current,
                SubmitOptions {
                    discover_imports: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let main_task = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend && task.unit == main)
            .unwrap();
        let leaf_task = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend && task.unit != main)
            .unwrap();
        assert!(
            graph
                .edges()
                .iter()
                .any(|edge| edge.dependent == main_task.id && edge.dependency == leaf_task.id)
        );
        let result = &run.phase_results[&main_task.id][0];
        assert_eq!(result.status, PhaseStatus::Complete);
        assert!(result.diagnostics.is_empty());
        let handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &result.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let output = owner.storage().get(&handle).unwrap();
        assert!(
            output.tokens.tokens().iter().any(
                |token| token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
            )
        );
        assert_eq!(result.output_refs[0].lineage().parents.len(), 2);
        let actual = (submission.module_index.unwrap(), output.as_ref().clone());
        if let Some(expected) = &clean {
            assert_eq!(&actual, expected);
        } else {
            clean = Some(actual);
        }
    }
}

#[test]
fn native_workspace_inputs_fail_closed_and_leave_validation_to_existing_owners() {
    use mizar_build::planner::{parse_workspace_manifest, read_native_workspace_inputs};
    let fixture = Fixture::new(b"definition\nend;\n");
    let root = WorkspaceRoot::new(fixture.root.join("alpha").to_str().unwrap());
    let directory = fixture.root.join("alpha");
    let manifest = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n";
    let lock = "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = []\n";
    fs::write(directory.join("mizar.pkg"), manifest).unwrap();
    fs::write(directory.join("mizar.lock"), lock).unwrap();
    let clean = read_native_workspace_inputs(&root).unwrap();
    let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
    input.plan_request.workspace_root = root.clone();
    request.workspace_root = root.clone();
    let plan =
        produce_build_plan(input.plan_request.clone(), clean.0.clone(), clean.1.clone()).unwrap();
    fixture.write(&[0xff]);
    assert_eq!(read_native_workspace_inputs(&root).unwrap(), clean);
    assert_eq!(
        StaticSourceLayout::from_native_disk(&plan).unwrap(),
        input.source_layout
    );
    request.source_inputs.versions.clear();
    assert!(
        request
            .capture_sources(
                &InMemorySessionIdAllocator::new(),
                &SnapshotRegistry::new(),
                vec![(
                    DiskSourceLoader::new(&directory),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new("main"),
                        normalized_path: normalize_path(&directory, Path::new("src/main.miz"))
                            .unwrap(),
                        edition: Edition::new("2025"),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into()
                        },
                    }
                )]
            )
            .is_err()
    );
    for text in [
        "",
        "[workspace]\nmembers = [\".\", \".\"]",
        "[workspace]\nmembers = [\"../alpha\"]",
        "[workspace]\nmembers = [\"/foreign\"]",
        "[workspace]\nunknown = 1",
    ] {
        fs::write(directory.join("mizar.workspace"), text).unwrap();
        assert_eq!(
            read_native_workspace_inputs(&root).unwrap_err(),
            parse_workspace_manifest(text).unwrap_err().to_string()
        );
    }
    fs::remove_file(directory.join("mizar.workspace")).unwrap();
    for text in [
        "",
        "[package]\nname = \"Bad-Name\"\nversion = \"0.1.0\"",
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\ntoolchain_extra = \"^1.4\"",
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[verifier]\ncomputation_replay = 1",
    ] {
        fs::write(directory.join("mizar.pkg"), text).unwrap();
        assert_eq!(
            read_native_workspace_inputs(&root).unwrap_err(),
            parse_package_manifest(text).unwrap_err().to_string()
        );
    }
    fs::write(directory.join("mizar.pkg"), manifest).unwrap();
    for text in ["", "schema_version = 2", "schema_version = 1\nunknown = 1"] {
        fs::write(directory.join("mizar.lock"), text).unwrap();
        assert_eq!(
            read_native_workspace_inputs(&root).unwrap_err(),
            parse_lockfile(text).unwrap_err().to_string()
        );
    }
    fs::write(directory.join("mizar.lock"), lock).unwrap();
    for (name, text) in [
        ("mizar.workspace", "[workspace]\nmembers = [\".\"]"),
        ("mizar.pkg", manifest),
        ("mizar.lock", lock),
    ] {
        fs::write(directory.join(name), [0xff]).unwrap();
        assert!(
            read_native_workspace_inputs(&root).is_err(),
            "{name} non-UTF-8"
        );
        fs::remove_file(directory.join(name)).unwrap();
        if name != "mizar.workspace" {
            assert!(
                read_native_workspace_inputs(&root).is_err(),
                "{name} missing"
            );
        }
        fs::create_dir(directory.join(name)).unwrap();
        assert!(
            read_native_workspace_inputs(&root).is_err(),
            "{name} directory"
        );
        fs::remove_dir(directory.join(name)).unwrap();
        fs::write(directory.join(name), text).unwrap();
    }
    for path in [directory.join("missing"), directory.join("mizar.pkg")] {
        assert!(read_native_workspace_inputs(&WorkspaceRoot::new(path.to_str().unwrap())).is_err());
    }
    fs::write(
        directory.join("mizar.workspace"),
        "[workspace]\nmembers = [\"mizar.pkg\"]",
    )
    .unwrap();
    assert!(read_native_workspace_inputs(&root).is_err());
    fs::create_dir_all(directory.join("nested/one")).unwrap();
    fs::write(directory.join("nested/one/mizar.pkg"), manifest).unwrap();
    fs::write(
        directory.join("mizar.workspace"),
        "[workspace]\nmembers = [\"nested/one\", \"z_late\"]",
    )
    .unwrap();
    assert!(read_native_workspace_inputs(&root).is_err());
    fs::create_dir(directory.join("z_late")).unwrap();
    assert!(read_native_workspace_inputs(&root).is_err());
    fs::write(directory.join("z_late/mizar.pkg"), "malformed").unwrap();
    assert!(read_native_workspace_inputs(&root).is_err());
    fs::write(directory.join("z_late/mizar.pkg"), manifest).unwrap();
    let duplicate = read_native_workspace_inputs(&root).unwrap();
    assert_eq!(duplicate.0.len(), 2);
    assert!(produce_build_plan(input.plan_request.clone(), duplicate.0, duplicate.1).is_err());
    fs::write(directory.join("mizar.lock"), "malformed").unwrap();
    assert!(read_native_workspace_inputs(&root).is_err());
    fs::write(
        directory.join("mizar.workspace"),
        "[workspace]\nmembers = [\".\"]",
    )
    .unwrap();
    for invalid in [
        lock.replace("version = \"0.1.0\"", "version = \"0.2.0\""),
        "schema_version = 1\npackage = []\n".into(),
    ] {
        fs::write(directory.join("mizar.lock"), invalid).unwrap();
        let parsed = read_native_workspace_inputs(&root).unwrap();
        assert!(produce_build_plan(input.plan_request.clone(), parsed.0, parsed.1).is_err());
    }
    fs::write(
        directory.join("mizar.lock"),
        lock.replace("path = \".\"", "path = \"unbound\""),
    )
    .unwrap();
    let unbound = read_native_workspace_inputs(&root).unwrap();
    assert!(produce_build_plan(input.plan_request.clone(), unbound.0, unbound.1).is_err());
    fs::write(directory.join("mizar.lock"), lock).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::{
            ffi::OsStringExt,
            fs::{PermissionsExt, symlink},
        };
        for suffix in ["", "/", "/."] {
            let link = fixture.root.join("root_link");
            symlink(&directory, &link).unwrap();
            assert!(
                read_native_workspace_inputs(&WorkspaceRoot::new(format!(
                    "{}{suffix}",
                    link.display()
                )))
                .is_err()
            );
            fs::remove_file(link).unwrap();
        }
        for (name, text) in [
            ("mizar.workspace", "[workspace]\nmembers = [\".\"]"),
            ("mizar.pkg", manifest),
            ("mizar.lock", lock),
        ] {
            fs::remove_file(directory.join(name)).unwrap();
            let ordinary = directory.join("ordinary_metadata");
            fs::write(&ordinary, text).unwrap();
            for target in [ordinary.clone(), directory.join("absent")] {
                symlink(&target, directory.join(name)).unwrap();
                assert!(
                    read_native_workspace_inputs(&root).is_err(),
                    "{name} -> {target:?}"
                );
                fs::remove_file(directory.join(name)).unwrap();
            }
            fs::remove_file(ordinary).unwrap();
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(directory.join(name))
                    .status()
                    .unwrap()
                    .success()
            );
            assert!(read_native_workspace_inputs(&root).is_err(), "{name} FIFO");
            fs::remove_file(directory.join(name)).unwrap();
            fs::write(directory.join(name), text).unwrap();
            let path = directory.join(name);
            let permissions = fs::metadata(&path).unwrap().permissions();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
            if fs::read_to_string(&path).is_err() {
                assert!(
                    read_native_workspace_inputs(&root).is_err(),
                    "{name} unreadable"
                );
            }
            fs::set_permissions(path, permissions).unwrap();
        }
        fs::write(
            directory.join("mizar.workspace"),
            "[workspace]\nmembers = [\"linked/one\"]",
        )
        .unwrap();
        fs::create_dir_all(fixture.root.join("foreign/one")).unwrap();
        fs::write(fixture.root.join("foreign/one/mizar.pkg"), manifest).unwrap();
        for target in [
            directory.join("nested"),
            directory.join("absent"),
            fixture.root.join("foreign"),
        ] {
            symlink(target, directory.join("linked")).unwrap();
            assert!(read_native_workspace_inputs(&root).is_err());
            fs::remove_file(directory.join("linked")).unwrap();
        }
        fs::write(
            directory.join("mizar.workspace"),
            "[workspace]\nmembers = [\"nested/linked\"]",
        )
        .unwrap();
        symlink("one", directory.join("nested/linked")).unwrap();
        assert!(read_native_workspace_inputs(&root).is_err());
        fs::remove_file(directory.join("nested/linked")).unwrap();
        let non_utf8 = fixture.root.join(std::ffi::OsString::from_vec(vec![0xff]));
        fs::create_dir_all(non_utf8.join("ordinary")).unwrap();
        fs::write(non_utf8.join("ordinary/mizar.pkg"), manifest).unwrap();
        fs::write(non_utf8.join("ordinary/mizar.lock"), lock).unwrap();
        let link = fixture.root.join("encoded_link");
        symlink(non_utf8, &link).unwrap();
        assert!(
            read_native_workspace_inputs(&WorkspaceRoot::new(
                link.join("ordinary").to_str().unwrap()
            ))
            .is_err()
        );
    }
    fs::remove_file(directory.join("mizar.workspace")).unwrap();
    assert_eq!(read_native_workspace_inputs(&root).unwrap(), clean);
}

#[cfg(target_os = "linux")]
#[test]
fn native_lsp_producer_captures_transitive_workspace_and_runs_frontend() {
    use mizar_build::planner::{lockfile_content_hash, workspace_config_content_hash};
    use mizar_driver::request::{BuildSession, LspPriority, LspRequest};

    let fixture = Fixture::new(b"definition\nend;\n");
    let mut packages = Vec::new();
    let mut lock = String::from("schema_version = 1\n");
    fs::write(
        fixture.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"alpha\", \"beta\", \"gamma\"]\n",
    )
    .unwrap();
    for (name, dependency) in [
        ("alpha", Some("beta")),
        ("beta", Some("gamma")),
        ("gamma", None),
    ] {
        fs::create_dir_all(fixture.root.join(name).join("src")).unwrap();
        fs::write(
            fixture.root.join(name).join("src/main.miz"),
            b"definition\nend;\n",
        )
        .unwrap();
        let mut manifest = format!("[package]\nname = {name:?}\nversion = \"0.1.0\"\n");
        if let Some(dependency) = dependency {
            manifest.push_str(&format!("[dependencies]\n{dependency} = \"0.1.0\"\n"));
        }
        fs::write(fixture.root.join(name).join("mizar.pkg"), &manifest).unwrap();
        packages.push(WorkspacePackage {
            member_path: name.into(),
            manifest: parse_package_manifest(&manifest).unwrap(),
        });
        let dependencies = dependency.map_or_else(String::new, |name| {
            format!("{{ name = {name:?}, version = \"0.1.0\" }}")
        });
        lock.push_str(&format!("[[package]]\nname = {name:?}\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = {name:?} }}\ndependencies = [{dependencies}]\n"));
    }
    fs::write(fixture.root.join("mizar.lock"), &lock).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (session, input) = BuildSession::capture_native_lsp(
        WorkspaceRoot::new(fixture.root.to_str().unwrap()),
        BuildLaneId::new(71),
        BuildRequestGeneration::new(3),
        LspRequest {
            focus: None,
            priority: LspPriority::Normal,
        },
        &ids,
        &snapshots,
    )
    .unwrap();
    let plan = produce_build_plan(
        input.plan_request.clone(),
        packages.clone(),
        parse_lockfile(&lock).unwrap(),
    )
    .unwrap();
    assert_eq!(input.workspace_packages, packages);
    assert_eq!(input.lockfile, parse_lockfile(&lock).unwrap());
    assert_eq!(
        input.plan_request.dependency_selection,
        DependencySelection::Normal
    );
    assert_eq!(plan.dependency_graph.edges.len(), 2);
    let snapshot = session.captured().snapshot();
    assert_eq!(snapshot.toolchain, plan.toolchain);
    let mut image = blake3::Hasher::new_derive_key("mizar-driver/running-native-image/v1");
    image.update(&fs::read("/proc/self/exe").unwrap());
    assert_eq!(
        snapshot.toolchain.identity(),
        format!(
            "mizar-driver/running-native-image/v1:{}",
            image.finalize().to_hex()
        )
    );
    assert_eq!(
        snapshot.lockfile_hash,
        lockfile_content_hash(&parse_lockfile(&lock).unwrap())
    );
    assert_eq!(
        snapshot.verifier_config_hash,
        workspace_config_content_hash(&plan.verifier_config, &plan.build_config)
    );
    assert!(snapshot.dependency_artifacts.is_empty());
    assert!(input.dependency_artifacts.is_empty());
    assert_eq!(session.request().profile.name, "lsp");
    let mut targets = session.request().targets.packages.clone();
    targets.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    assert_eq!(targets, ["alpha", "beta", "gamma"].map(PackageId::new));
    assert_eq!(session.request().lane, BuildLaneId::new(71));
    assert_eq!(session.request().generation, BuildRequestGeneration::new(3));
    let captured = session.captured().native_sources().unwrap();
    assert_eq!(captured.len(), 3);
    assert_eq!(snapshot.source_versions.len(), 3);
    for name in ["alpha", "beta", "gamma"] {
        let root = fixture.root.join(name);
        let descriptor = SourceInput {
            package_id: PackageId::new(name),
            module_path: ModulePath::new("main"),
            normalized_path: normalize_path(&root, Path::new("src/main.miz")).unwrap(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: "src/main.miz".into(),
            },
        };
        let oracle = mizar_session::SourceLoader::load(
            &DiskSourceLoader::new(&root),
            snapshot.id,
            descriptor.clone(),
            &ids,
        )
        .unwrap();
        let (_, loaded, actual) = captured
            .iter()
            .find(|(_, _, source)| source.package_id == PackageId::new(name))
            .unwrap();
        assert_eq!(actual, &descriptor);
        assert_eq!(loaded.source_hash, oracle.source_hash);
        assert_eq!(loaded.text, oracle.text);
        assert_eq!(
            snapshot
                .source_versions
                .iter()
                .find(|version| version.package_id == PackageId::new(name))
                .unwrap()
                .source_id,
            loaded.source_id
        );
    }
    let units = ["alpha", "beta", "gamma"].map(|name| WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new(name),
            ModulePath::new("main"),
        ),
    });
    let owner = publisher(snapshot.id, usize::MAX, &units[0]);
    for (name, unit) in ["alpha", "beta", "gamma"].into_iter().zip(&units) {
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("SourceLoad"),
            OutputKind::new("SourceUnit"),
            IrWorkUnit::new(format!("{name:?}:{:?}", "main")),
        ));
        allow_frontend(&owner, unit);
    }
    let mut driver =
        CompilerDriver::new(assemble_registry(&StageConfig::frontend(vec![])).unwrap())
            .with_output_publisher(owner.clone());
    let submission = driver
        .submit_captured(
            session,
            &ids,
            &snapshots,
            input,
            SubmitOptions {
                discover_imports: true,
                ..Default::default()
            },
        )
        .unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    for kind in ["SourceUnit", "FrontendOutput"] {
        let outputs = run
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .filter(|output| output.output_kind() == &OutputKind::new(kind))
            .count();
        assert_eq!(outputs, 3, "{kind}");
    }
    assert!(
        run.phase_results
            .values()
            .flatten()
            .all(|result| result.diagnostics.is_empty())
    );
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
}

fn native_lsp_producer_fixture() -> Fixture {
    let fixture = Fixture::new(b"definition\nend;\n");
    fs::write(
        fixture.root.join("alpha/mizar.pkg"),
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(fixture.root.join("alpha/mizar.lock"), "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = []\n").unwrap();
    fixture
}

#[cfg(target_os = "linux")]
#[test]
fn native_lsp_producer_reconstructs_config_and_retains_guarded_capture() {
    use mizar_driver::request::{BuildSession, LspPriority, LspRequest};
    let fixture = native_lsp_producer_fixture();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let capture = || {
        BuildSession::capture_native_lsp(
            WorkspaceRoot::new(fixture.root.join("alpha").to_str().unwrap()),
            BuildLaneId::new(72),
            BuildRequestGeneration::new(0),
            LspRequest {
                focus: None,
                priority: LspPriority::Normal,
            },
            &ids,
            &snapshots,
        )
        .unwrap()
    };
    let (session, input) = capture();
    let original = session.captured().snapshot().clone();
    fs::write(fixture.root.join("alpha/mizar.pkg"), "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[verifier]\ncomputation_replay = true\n[build]\ncache_dir = \"local-cache\"\n").unwrap();
    let (changed, changed_input) = capture();
    let changed_plan = produce_build_plan(
        changed_input.plan_request,
        changed_input.workspace_packages,
        changed_input.lockfile,
    )
    .unwrap();
    assert_eq!(
        original.lockfile_hash,
        changed.captured().snapshot().lockfile_hash
    );
    assert_eq!(
        original.source_versions[0].source_hash,
        changed.captured().snapshot().source_versions[0].source_hash
    );
    assert_ne!(
        original.verifier_config_hash,
        changed.captured().snapshot().verifier_config_hash
    );
    assert_ne!(original.id, changed.captured().snapshot().id);
    let config = &changed_plan.packages[0].verifier_config;
    assert!(config.computation_replay);
    assert_eq!(
        config.max_axioms,
        mizar_build::planner::VerifierConfig::default().max_axioms
    );
    assert_eq!(
        changed_plan.packages[0].build_config.cache_dir,
        "local-cache"
    );
    assert_eq!(
        changed.captured().snapshot().verifier_config_hash,
        mizar_build::planner::workspace_config_content_hash(
            &changed_plan.verifier_config,
            &changed_plan.build_config
        )
    );
    fixture.write(b"definition\nlet x be set;\nend;\n");
    assert_eq!(session.captured().snapshot(), &original);
    assert_eq!(
        session.captured().native_sources().unwrap()[0]
            .1
            .text
            .as_ref(),
        "definition\nend;\n"
    );
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    let owner = publisher(original.id, usize::MAX, &unit);
    allow_frontend(&owner, &unit);
    let mut driver =
        CompilerDriver::new(assemble_registry(&StageConfig::frontend(vec![])).unwrap())
            .with_output_publisher(owner.clone());
    let submission = driver
        .submit_captured(
            session,
            &ids,
            &snapshots,
            input,
            SubmitOptions {
                discover_imports: true,
                ..Default::default()
            },
        )
        .unwrap();
    let (inputs, dispatch) = native_disk_lookup_context(&submission, &ids);
    assert_eq!(inputs.validate_native_config_content(), Ok(()));
    assert_eq!(inputs.validate_native_lock_content(), Ok(()));
    let result =
        &submission.scheduler_run.as_ref().unwrap().phase_results[&source_task(&submission).id][0];
    let handle = owner
        .storage()
        .typed_handle::<SourceUnit>(&result.output_refs[0], &OutputKind::new("SourceUnit"))
        .unwrap();
    assert_eq!(
        owner.storage().get(&handle).unwrap().source_text.as_ref(),
        "definition\nend;\n"
    );
    for mismatch in ["plan", "runtime", "config", "lock"] {
        let mut foreign = inputs.build_plan.clone();
        match mismatch {
            "plan" => foreign.workspace_root = WorkspaceRoot::new("/foreign"),
            "runtime" => foreign.toolchain = ToolchainInfo::new("foreign-runtime"),
            "config" => {
                foreign.verifier_config.packages[0]
                    .config
                    .computation_replay = true
            }
            "lock" => foreign.lockfile.packages[0].name = "foreign-lock-package".into(),
            _ => unreachable!(),
        }
        let tested = SourceLoadInputs {
            build_plan: &foreign,
            ..inputs
        };
        if mismatch == "config" {
            assert!(tested.validate_native_config_content().is_err());
        }
        if mismatch == "lock" {
            assert!(tested.validate_native_lock_content().is_err());
        }
        if matches!(mismatch, "plan" | "runtime") {
            let result = driver
                .registry()
                .execute_phase_with_resources(
                    PipelinePhase::SourceLoad,
                    dispatch.clone(),
                    PhaseExecutionResources {
                        output_publisher: Some(publisher(original.id, usize::MAX, &unit)),
                        source_load: Some(tested),
                        ..Default::default()
                    },
                )
                .unwrap()
                .result;
            assert_ne!(result.status, PhaseStatus::Complete);
            assert!(result.output_refs.is_empty());
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn native_lsp_producer_preserves_dev_metadata_and_unused_registry_lock() {
    use mizar_driver::request::{BuildSession, LspPriority, LspRequest};
    let fixture = native_lsp_producer_fixture();
    fs::create_dir_all(fixture.root.join("beta/src")).unwrap();
    fs::write(
        fixture.root.join("beta/src/main.miz"),
        b"definition\nend;\n",
    )
    .unwrap();
    fs::write(
        fixture.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"alpha\", \"beta\"]\n",
    )
    .unwrap();
    let alpha = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dev-dependencies]\nbeta = \"0.1.0\"\n[verifier]\ncomputation_replay = true\n";
    let beta = "[package]\nname = \"beta\"\nversion = \"0.1.0\"\n";
    fs::write(fixture.root.join("alpha/mizar.pkg"), alpha).unwrap();
    fs::write(fixture.root.join("beta/mizar.pkg"), beta).unwrap();
    let lock = "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"beta\", version = \"0.1.0\" }]\n[[package]]\nname = \"beta\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"beta\" }\ndependencies = []\n[[package]]\nname = \"unused\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:unused\" }\ndependencies = []\n";
    fs::write(fixture.root.join("mizar.lock"), lock).unwrap();
    let (session, input) = BuildSession::capture_native_lsp(
        WorkspaceRoot::new(fixture.root.to_str().unwrap()),
        BuildLaneId::new(73),
        BuildRequestGeneration::new(0),
        LspRequest {
            focus: None,
            priority: LspPriority::Normal,
        },
        &InMemorySessionIdAllocator::new(),
        &SnapshotRegistry::new(),
    )
    .unwrap();
    let packages = vec![
        WorkspacePackage {
            member_path: "alpha".into(),
            manifest: parse_package_manifest(alpha).unwrap(),
        },
        WorkspacePackage {
            member_path: "beta".into(),
            manifest: parse_package_manifest(beta).unwrap(),
        },
    ];
    assert_eq!(input.workspace_packages, packages);
    assert_eq!(input.lockfile, parse_lockfile(lock).unwrap());
    let plan =
        produce_build_plan(input.plan_request, input.workspace_packages, input.lockfile).unwrap();
    assert_eq!(plan.packages.len(), 2);
    assert!(plan.dependency_graph.edges.is_empty());
    let alpha = plan
        .packages
        .iter()
        .find(|package| package.package_id == PackageId::new("alpha"))
        .unwrap();
    assert_eq!(
        alpha.dependencies[0].kind,
        mizar_build::planner::DependencyKind::Dev
    );
    assert!(alpha.verifier_config.computation_replay);
    assert_eq!(
        plan.packages
            .iter()
            .find(|package| package.package_id == PackageId::new("beta"))
            .unwrap()
            .verifier_config,
        mizar_build::planner::VerifierConfig::default()
    );
    assert_eq!(session.captured().native_sources().unwrap().len(), 2);
    assert_eq!(
        session.captured().snapshot().lockfile_hash,
        mizar_build::planner::lockfile_content_hash(&parse_lockfile(lock).unwrap())
    );
    let mut reduced = parse_lockfile(lock).unwrap();
    reduced.packages.retain(|package| package.name != "unused");
    assert_ne!(
        session.captured().snapshot().lockfile_hash,
        mizar_build::planner::lockfile_content_hash(&reduced)
    );
    assert!(
        session
            .captured()
            .snapshot()
            .dependency_artifacts
            .is_empty()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn native_lsp_producer_refuses_viable_registry_plans() {
    use mizar_driver::request::{BuildSession, LspPriority, LspRequest};
    let fixture = native_lsp_producer_fixture();
    let lock = "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n";
    fs::write(fixture.root.join("alpha/mizar.lock"), lock).unwrap();
    for dev in [false, true] {
        if dev {
            fs::create_dir_all(fixture.root.join("alpha/beta/src")).unwrap();
            fs::write(
                fixture.root.join("alpha/beta/src/main.miz"),
                b"definition\nend;\n",
            )
            .unwrap();
            fs::write(
                fixture.root.join("alpha/mizar.workspace"),
                "[workspace]\nmembers = [\".\", \"beta\"]\n",
            )
            .unwrap();
            fs::write(fixture.root.join("alpha/beta/mizar.pkg"), "[package]\nname = \"beta\"\nversion = \"0.1.0\"\n[dependencies]\ndep = \"1.0.0\"\n").unwrap();
            fs::write(fixture.root.join("alpha/mizar.lock"), format!("{lock}[[package]]\nname = \"beta\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"beta\" }}\ndependencies = [{{ name = \"dep\", version = \"1.0.0\" }}]\n")).unwrap();
        }
        let manifest = format!(
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[{}]\ndep = \"1.0.0\"\n",
            if dev {
                "dev-dependencies"
            } else {
                "dependencies"
            }
        );
        fs::write(fixture.root.join("alpha/mizar.pkg"), &manifest).unwrap();
        let root = WorkspaceRoot::new(fixture.root.join("alpha").to_str().unwrap());
        let (packages, lockfile) =
            mizar_build::planner::read_native_workspace_inputs(&root).unwrap();
        let plan = produce_build_plan(
            PlanRequest {
                workspace_root: root.clone(),
                dependency_selection: DependencySelection::Normal,
                toolchain: ToolchainInfo::new("independent-registry-control"),
            },
            packages,
            lockfile,
        )
        .unwrap();
        assert!(plan.packages.iter().any(|package| matches!(
            package.source,
            mizar_build::planner::PackagePlanSource::Registry { .. }
        )));
        if dev {
            assert!(
                plan.packages
                    .iter()
                    .find(|package| package.package_id == PackageId::new("alpha"))
                    .unwrap()
                    .dependencies
                    .iter()
                    .any(|dependency| dependency.kind == mizar_build::planner::DependencyKind::Dev)
            );
        }
        let result = BuildSession::capture_native_lsp(
            root,
            BuildLaneId::new(74),
            BuildRequestGeneration::new(0),
            LspRequest {
                focus: None,
                priority: LspPriority::Normal,
            },
            &InMemorySessionIdAllocator::new(),
            &SnapshotRegistry::new(),
        );
        assert!(
            matches!(result, Err(error) if error == "native LSP context requires workspace-only plan rows")
        );
    }
}

#[test]
fn native_lsp_producer_keeps_acquisition_failures_unavailable() {
    use mizar_driver::request::{BuildSession, LspPriority, LspRequest};
    let fixture = native_lsp_producer_fixture();
    let capture = || {
        BuildSession::capture_native_lsp(
            WorkspaceRoot::new(fixture.root.join("alpha").to_str().unwrap()),
            BuildLaneId::new(75),
            BuildRequestGeneration::new(0),
            LspRequest {
                focus: None,
                priority: LspPriority::Normal,
            },
            &InMemorySessionIdAllocator::new(),
            &SnapshotRegistry::new(),
        )
    };
    #[cfg(not(target_os = "linux"))]
    assert!(
        matches!(capture(), Err(error) if error == "running native image provenance is unsupported on this platform")
    );
    #[cfg(target_os = "linux")]
    {
        for path in ["mizar.pkg", "mizar.lock"] {
            let path = fixture.root.join("alpha").join(path);
            let original = fs::read(&path).unwrap();
            fs::remove_file(&path).unwrap();
            assert!(capture().is_err());
            fs::write(&path, "invalid TOML = [").unwrap();
            assert!(capture().is_err());
            fs::write(&path, original).unwrap();
        }
        let workspace = fixture.root.join("alpha/mizar.workspace");
        fs::write(&workspace, "[workspace]\nmembers = [\"missing\"]\n").unwrap();
        assert!(capture().is_err());
        fs::remove_file(workspace).unwrap();
        fixture.write(&[0xff]);
        assert!(capture().is_err());
        fs::remove_dir_all(fixture.root.join("alpha/src")).unwrap();
        assert!(capture().is_err());
    }
}
