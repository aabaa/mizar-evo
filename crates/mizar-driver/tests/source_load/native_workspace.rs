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
            .submit_captured_with_import_discovery(session.clone(), &ids, &snapshots, current)
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
