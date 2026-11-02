use super::*;

#[test]
fn native_theorem_metadata_caller_preserves_genuine_outputs() {
    use mizar_checker::source_atomic_formula::SourceAtomicRequestKind;
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    for text in [
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ),
        "theorem Empty: for A being set holds A = A proof end;",
    ] {
        for open in [false, true] {
            for threshold in [usize::MAX, 0] {
                let fixture = Fixture::new(text.as_bytes());
                let ids = InMemorySessionIdAllocator::new();
                let snapshots = SnapshotRegistry::new();
                let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
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
                            text: std::sync::Arc::from(text),
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
                let snapshot = session.captured().snapshot().id;
                let module = mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("main"),
                );
                let unit = WorkUnit::Module {
                    module: module.clone(),
                };
                let owner = publisher(snapshot, threshold, &unit);
                allow_frontend(&owner, &unit);
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session, &ids, &snapshots, submit)
                    .unwrap();
                let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
                let parent = submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|result| &result.output_refs)
                    .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
                    .unwrap();
                assert_eq!(
                    parent.placement() == &StoragePlacement::Resident,
                    threshold != 0
                );
                let input = PhaseInput::new(
                    unit,
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_input.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot,
                                parent.clone(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let (_, loaded, captured_input) =
                    &inputs.captured.unwrap().native_sources().unwrap()[0];
                let source = mizar_frontend::source::source_unit_from_loaded(
                    loaded.clone(),
                    mizar_frontend::source::diagnostic_file_path(captured_input),
                );
                assert_eq!(source.source_text.as_ref(), text);
                let output = mizar_frontend::orchestration::Frontend::new(
                    mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
                        fixture.root.join("alpha"),
                    )),
                    inputs.dependency_lexical_provider(&[]),
                    MizarParserSeam,
                )
                .run_loaded(source)
                .unwrap();
                assert!(output.diagnostics.is_empty());
                let typed = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        parent,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .unwrap();
                assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                let ast = output.ast.as_ref().unwrap();
                let resolver_module =
                    ResolverModuleId::new(module.package.clone(), module.path.clone());
                let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
                let collection =
                    SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main"))
                        .collect(&resolver_module);
                assert!(collection.diagnostics().is_empty());
                let arena =
                    mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver_module)
                        .unwrap();
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, &resolver_module, collection.env()),
                )
                .unwrap();
                let expected = mizar_checker::type_checker::check_source_theorem_metadata(
                    &arena,
                    collection.env(),
                    &scope,
                )
                .unwrap();
                assert_eq!(expected.0.source_id(), loaded.source_id);
                assert_eq!(expected.0.module_id(), &resolver_module);
                assert_eq!(expected.1.source_id(), loaded.source_id);
                assert_eq!(expected.1.module_id(), &resolver_module);
                assert_eq!(expected.2.source_id(), loaded.source_id);
                assert_eq!(expected.2.module_id(), &resolver_module);
                assert_eq!(expected.3.source_id(), loaded.source_id);
                assert_eq!(expected.3.module_id(), &resolver_module);
                let atomic = expected.2.source_atomic_formula().unwrap();
                assert!(!atomic.requests().is_empty());
                assert!(
                    atomic.requests().iter().all(|(_, request)| request.kind()
                        == SourceAtomicRequestKind::OperandExpectedType)
                );
                assert_eq!(expected.3.source_atomic_formula(), Some(atomic));
                let before = (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    input.clone(),
                    parent.clone(),
                );
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    for _ in 0..2 {
                        assert_eq!(
                            inputs
                                .native_theorem_metadata_with_frontend_parent(&input, &owner)
                                .unwrap(),
                            Some(expected.clone()),
                            "open={open} threshold={threshold} stage={stage}"
                        );
                    }
                    assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                }
                assert_eq!(
                    (
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        inputs.module_index.clone(),
                        loaded.clone(),
                        captured_input.clone(),
                        input,
                        parent.clone()
                    ),
                    before
                );
                assert_eq!(captured_input, &descriptor);
            }
        }
    }
}

#[test]
fn native_theorem_metadata_caller_preserves_refusals() {
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("by PrivT1", "by Missing"), "citation"),
        (private.replace("thus X = X;", "thus Y = Y;"), "unbound"),
        (private.replace("set", "object"), "checker"),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                source_input.snapshot,
                source_input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        source_input.snapshot,
                        handle.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        let retained = owner.storage().get(&handle).unwrap();
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            descriptor.clone(),
            input.clone(),
            handle.clone(),
            (*retained).clone(),
        );
        let lower = inputs.native_theorem_module_candidates_with_frontend_parent(&input, owner);
        let expected = if case == "checker" {
            assert!(lower.unwrap().is_some());
            let WorkUnit::Module { module } = &input.work_unit else {
                unreachable!()
            };
            let resolver_module =
                ResolverModuleId::new(module.package.clone(), module.path.clone());
            let ast = retained.ast.as_ref().unwrap();
            let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                ast,
                &resolver_module,
            )
            .unwrap()
            .unwrap();
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, &resolver_module, &env),
            )
            .unwrap();
            Err(
                mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)
                    .unwrap_err(),
            )
        } else if case == "unbound" {
            Err(lower.unwrap_err())
        } else {
            assert_eq!(lower, Ok(None));
            Ok(None)
        };
        assert_eq!(
            inputs.native_theorem_metadata_with_frontend_parent(&input, owner),
            expected,
            "{case}"
        );
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                descriptor.clone(),
                input,
                handle.clone(),
                (*owner.storage().get(&handle).unwrap()).clone()
            ),
            before
        );
    }
    let text = "theorem Empty: for A being set holds A = A proof end;";
    let fixture = Fixture::new(text.as_bytes());
    let foreign = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, foreign_submission, foreign_adapter, foreign_handle, _) =
        native_disk_frontend_record_submission(&foreign, &ids, &snapshots, 0);
    for case in ["absent", "foreign", "hash", "obsolete", "collected"] {
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(
                owner,
                source_input.snapshot,
                handle.erase(),
            )
            .unwrap(),
        ];
        let mut snapshot = source_input.snapshot;
        let mut hash = source_input.identities().input_hash();
        match case {
            "absent" => parents.clear(),
            "foreign" => {
                snapshot = foreign_submission.session.captured().snapshot().id;
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        foreign_adapter.publisher(),
                        snapshot,
                        foreign_handle.erase(),
                    )
                    .unwrap(),
                ];
            }
            "hash" => hash = Hash::from_bytes([0x61; 32]),
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
            }
            _ => unreachable!(),
        }
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(snapshot, hash, vec![], parents).unwrap(),
        );
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            descriptor.clone(),
            input.clone(),
            handle.clone(),
        );
        let expected = inputs
            .native_theorem_module_candidates_with_frontend_parent(&input, owner)
            .unwrap_err();
        assert_eq!(
            inputs.native_theorem_metadata_with_frontend_parent(&input, owner),
            Err(expected),
            "{case}"
        );
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                descriptor.clone(),
                input,
                handle
            ),
            before
        );
    }
}

#[test]
fn native_theorem_metadata_storage_replay_preserves_genuine_outputs() {
    use mizar_checker::{
        source_atomic_formula::SourceAtomicRequestKind,
        type_checker::{NormalizedTypeStatus, TypeHeadRef, check_source_theorem_metadata as check},
    };
    use mizar_driver::{
        native_theorem_metadata_from_storage_bytes as decode,
        native_theorem_metadata_storage_bytes as encode,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{
            source_theorem_module_candidates as candidates,
            source_theorem_module_original_inputs as original_inputs,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for text in [
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        include_str!(
            "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
        ),
        "theorem Renamed: for Y being set holds Y = Y\r\nproof\r\n let Y be set;\r\n thus Y = Y;\r\nend;\r\n",
    ] {
        for (open, threshold) in [(false, usize::MAX), (true, 0)] {
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, _, owner, outputs, (session, ids, snapshots)| {
                    let parent = outputs
                        .iter()
                        .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                        .unwrap();
                    assert_eq!(parent.placement() == &StoragePlacement::Resident, !open);
                    let handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            parent,
                            parent.output_kind(),
                        )
                        .unwrap();
                    let retained = owner.storage().get(&handle).unwrap();
                    let ast = retained.ast.as_ref().unwrap();
                    let (_, _, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
                    let (mut draft, _) = fixture.submit_request(vec![], vec![]);
                    draft.source_inputs.versions.clear();
                    let relocated = draft
                        .capture_sources(
                            ids,
                            snapshots,
                            vec![(
                                DiskSourceLoader::new(fixture.root.join("alpha")),
                                descriptor.clone(),
                            )],
                        )
                        .unwrap();
                    let second_inputs = SourceLoadInputs {
                        snapshot: relocated.captured().snapshot(),
                        captured: Some(relocated.captured()),
                        ..inputs
                    };
                    let (_, loaded, descriptor) =
                        &relocated.captured().native_sources().unwrap()[0];
                    let second = Frontend::new(
                        FrontendSourceLoader::new(DiskSourceLoader::new(
                            fixture.root.join("alpha"),
                        )),
                        second_inputs.dependency_lexical_provider(&[]),
                        MizarParserSeam,
                    )
                    .run_loaded(source_unit_from_loaded(
                        loaded.clone(),
                        diagnostic_file_path(descriptor),
                    ))
                    .unwrap();
                    assert!(second.diagnostics.is_empty());
                    let second_ast = second.ast.as_ref().unwrap();
                    assert_ne!(second_ast.source_id, ast.source_id);
                    let asts = [ast, second_ast];
                    let metadata = asts.map(|ast| {
                        let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                        assert!(candidates(ast, &arena, &env).unwrap().is_some());
                        let scope =
                            Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
                                .unwrap();
                        check(&arena, &env, &scope).unwrap()
                    });
                    assert_ne!(metadata[0], metadata[1]);
                    for member in 0..4 {
                        let mut mixed = metadata[0].clone();
                        match member {
                            0 => mixed.0 = metadata[1].0.clone(),
                            1 => mixed.1 = metadata[1].1.clone(),
                            2 => mixed.2 = metadata[1].2.clone(),
                            3 => mixed.3 = metadata[1].3.clone(),
                            _ => unreachable!(),
                        }
                        assert_ne!(mixed, metadata[0]);
                        assert_eq!(encode(ast, &module, &mixed), Ok(None));
                    }
                    for (ast, result) in asts.into_iter().zip(&metadata) {
                        for source in [
                            result.0.source_id(),
                            result.1.source_id(),
                            result.2.source_id(),
                            result.3.source_id(),
                        ] {
                            assert_eq!(source, ast.source_id);
                        }
                        for actual_module in [
                            result.0.module_id(),
                            result.1.module_id(),
                            result.2.module_id(),
                            result.3.module_id(),
                        ] {
                            assert_eq!(actual_module, &module);
                        }
                        assert!(!result.1.normalized_types().is_empty());
                        let references = result
                            .3
                            .nodes()
                            .iter()
                            .filter_map(|(_, node)| node.final_type)
                            .collect::<Vec<_>>();
                        assert!(!references.is_empty());
                        for reference in references {
                            let normalized = result.1.normalized_types().get(reference).unwrap();
                            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                            assert_eq!(normalized.status, NormalizedTypeStatus::Known);
                        }
                        let atomic = result.2.source_atomic_formula().unwrap();
                        assert!(!atomic.requests().is_empty());
                        assert!(atomic.requests().iter().all(|(_, request)| request.kind()
                            == SourceAtomicRequestKind::OperandExpectedType));
                        assert_eq!(result.3.source_atomic_formula(), Some(atomic));
                        assert!(result.3.collection_candidates().is_empty());
                        assert!(result.3.resolved_overloads().is_empty());
                    }
                    let syntax = ast.canonical_bytes().unwrap();
                    let mut bytes = b"mizar-driver/native-theorem-metadata-storage/v1\0".to_vec();
                    for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
                        bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
                        bytes.extend_from_slice(field);
                    }
                    assert_eq!(
                        encode(second_ast, &module, &metadata[1]).unwrap(),
                        Some(bytes.clone())
                    );
                    assert_eq!(
                        decode(&bytes, second_ast.source_id, &module).unwrap(),
                        Some(metadata[1].clone())
                    );
                    assert_eq!(encode(ast, &module, &metadata[1]).unwrap(), None);
                    let before = (ast.clone(), metadata.clone(), session.captured().clone());
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(if open {
                                &[0xff]
                            } else {
                                b"open theorem Changed: 0 = 1;\n"
                            });
                        } else if stage == 2 {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        for _ in 0..2 {
                            assert_eq!(
                                encode(ast, &module, &metadata[0]).unwrap(),
                                Some(bytes.clone())
                            );
                            assert_eq!(
                                decode(&bytes, ast.source_id, &module).unwrap(),
                                Some(metadata[0].clone())
                            );
                        }
                        assert_eq!(*owner.storage().get(&handle).unwrap(), *retained);
                    }
                    assert_eq!((ast.clone(), metadata, session.captured().clone()), before);
                },
            );
        }
    }
}

#[test]
fn native_theorem_metadata_storage_replay_preserves_refusals() {
    use mizar_driver::{
        native_theorem_metadata_from_storage_bytes as decode,
        native_theorem_metadata_storage_bytes as encode,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{
            source_theorem_module_candidates as candidates,
            source_theorem_module_original_inputs as original_inputs,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let lower = |ast: &<MizarParserSeam as ParserSeam>::Ast| -> Result<Option<_>, String> {
        let Some((arena, env)) = original_inputs(ast, &module)? else {
            return Ok(None);
        };
        if candidates(ast, &arena, &env)?.is_none() {
            return Ok(None);
        }
        let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
            .map_err(|error| format!("{error:?}"))?;
        mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope).map(Some)
    };
    let domain = b"mizar-driver/native-theorem-metadata-storage/v1\0";
    let frame = |syntax: &[u8]| {
        let mut bytes = domain.to_vec();
        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax] {
            bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
            bytes.extend_from_slice(field);
        }
        bytes
    };
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let valid = Fixture::new(
        b"theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    );
    let (_, _, adapter, parent, _) =
        native_disk_frontend_record_submission(&valid, &ids, &snapshots, 0);
    let output = adapter.publisher().storage().get(&parent).unwrap();
    let ast = output.ast.as_ref().unwrap();
    let metadata = lower(ast).unwrap().unwrap();
    let bytes = frame(&ast.canonical_bytes().unwrap());
    let mut corruptions = vec![
        Vec::new(),
        bytes[..domain.len() + 3].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
        frame(&[0xff]),
    ];
    let mut bad_domain = bytes.clone();
    bad_domain[0] = b'x';
    corruptions.push(bad_domain);
    let mut bad_version = bytes.clone();
    bad_version[domain.len() - 2] = b'2';
    corruptions.push(bad_version);
    let mut bad_length = bytes.clone();
    bad_length[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    corruptions.push(bad_length);
    let mut trailing = bytes.clone();
    trailing.push(0);
    corruptions.push(trailing);
    let mut oversized = bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(decode(&malformed, ast.source_id, &module), Ok(None));
    }
    for wrong_module in [
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("main")),
        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other")),
    ] {
        assert_eq!(decode(&bytes, ast.source_id, &wrong_module), Ok(None));
        assert_eq!(encode(ast, &wrong_module, &metadata), Ok(None));
    }
    let changed = Fixture::new(
        b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
    );
    let (_, _, other_adapter, other_parent, _) =
        native_disk_frontend_record_submission(&changed, &ids, &snapshots, 0);
    let other = other_adapter
        .publisher()
        .storage()
        .get(&other_parent)
        .unwrap();
    let other_ast = other.ast.as_ref().unwrap();
    assert_ne!(other_ast.source_id, ast.source_id);
    let rebound = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
        &other_ast.canonical_bytes().unwrap(),
        ast.source_id,
    )
    .unwrap();
    let other_metadata = lower(&rebound).unwrap().unwrap();
    assert_eq!(other_metadata.0.source_id(), metadata.0.source_id());
    assert_eq!(other_metadata.3.module_id(), metadata.3.module_id());
    assert_ne!(other_metadata, metadata);
    assert_eq!(encode(ast, &module, &other_metadata), Ok(None));
    assert_eq!(encode(&rebound, &module, &metadata), Ok(None));
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let registration = include_str!(
        "../../../../tests/miz/pass/clusters/pass_advanced_semantics_existential_registration_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("by PrivT1", "by Missing"), "citation"),
        (private.replace("thus X = X;", "thus Y = Y;"), "unbound"),
        (private.replace("set", "object"), "unsupported-type"),
        (registration.to_owned(), "registration"),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, _, adapter, parent, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let retained = adapter.publisher().storage().get(&parent).unwrap();
        let ast = retained.ast.as_ref().unwrap();
        let expected = lower(ast);
        assert!(matches!(
            (&expected, case),
            (Ok(None), "reserve" | "citation" | "registration")
                | (Err(_), "unsupported-type" | "unbound")
        ));
        let before = (ast.clone(), metadata.clone());
        assert_eq!(
            decode(
                &frame(&ast.canonical_bytes().unwrap()),
                ast.source_id,
                &module
            ),
            expected,
            "{case}"
        );
        assert_eq!(
            encode(ast, &module, &metadata),
            expected.map(|_| None::<Vec<u8>>),
            "{case}"
        );
        assert_eq!((ast.clone(), metadata.clone()), before);
    }
}

#[test]
fn native_theorem_metadata_type_registration_caller_preserves_genuine_outputs() {
    use mizar_checker::{
        source_atomic_formula::SourceAtomicRequestKind,
        type_checker::{TypeHeadRef, check_source_theorem_metadata as check},
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::source_theorem_module_original_inputs as original_inputs,
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let mut positions = std::collections::BTreeSet::new();
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        for variant in 0..64 {
            let text = match variant {
                0 => include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz").to_owned(),
                1 => "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;".to_owned(),
                _ => format!("theorem Placement{variant}: for Y being set holds Y = Y\nproof\n let Y be set;\n thus Y = Y;\nend;\n"),
            };
            with_native_summary_parent(
                &text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, (session, _, _)| {
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                    let child = |parents: Vec<AnyPhaseOutputRef>| {
                        PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                parents
                                    .into_iter()
                                    .map(|parent| {
                                        SealedParentOutputHandle::from_published_output(
                                            owner,
                                            old.snapshot,
                                            parent,
                                        )
                                        .unwrap()
                                    })
                                    .collect(),
                            )
                            .unwrap(),
                        )
                    };
                    let signature = inputs
                        .publish_native_theorem_module_summary(old, owner)
                        .unwrap()
                        .unwrap();
                    let type_input = child(vec![signature.erase()]);
                    let typed = inputs
                        .publish_native_theorem_typed_ast(&type_input, owner)
                        .unwrap()
                        .unwrap();
                    let reg_input = child(vec![typed.erase()]);
                    let registration = inputs
                        .publish_native_theorem_registration_closure(&reg_input, owner)
                        .unwrap()
                        .unwrap();
                    let input = child(vec![registration.erase(), typed.erase()]);
                    positions.insert(
                        input
                            .parent_outputs()
                            .iter()
                            .position(|parent| {
                                parent.as_output_ref().phase()
                                    == &IrPipelinePhase::new("TypeChecking")
                            })
                            .unwrap(),
                    );
                    assert_eq!(typed.placement() == &StoragePlacement::Resident, !open);
                    assert_eq!(
                        registration.placement() == &StoragePlacement::Resident,
                        !open
                    );
                    let (_, loaded, descriptor) =
                        &inputs.captured.unwrap().native_sources().unwrap()[0];
                    let output = Frontend::new(
                        FrontendSourceLoader::new(DiskSourceLoader::new(
                            fixture.root.join("alpha"),
                        )),
                        inputs.dependency_lexical_provider(&[]),
                        MizarParserSeam,
                    )
                    .run_loaded(source_unit_from_loaded(
                        loaded.clone(),
                        diagnostic_file_path(descriptor),
                    ))
                    .unwrap();
                    assert!(output.diagnostics.is_empty());
                    let ast = output.ast.as_ref().unwrap();
                    let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                    let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
                        .unwrap();
                    let expected = check(&arena, &env, &scope).unwrap();
                    for id in [
                        expected.0.source_id(),
                        expected.1.source_id(),
                        expected.2.source_id(),
                        expected.3.source_id(),
                    ] {
                        assert_eq!(id, loaded.source_id);
                    }
                    for actual_module in [
                        expected.0.module_id(),
                        expected.1.module_id(),
                        expected.2.module_id(),
                        expected.3.module_id(),
                    ] {
                        assert_eq!(actual_module, &module);
                    }
                    let normalized = expected
                        .3
                        .nodes()
                        .iter()
                        .filter_map(|(_, node)| node.final_type)
                        .collect::<Vec<_>>();
                    assert!(!normalized.is_empty());
                    for id in normalized {
                        assert_eq!(
                            expected.1.normalized_types().get(id).unwrap().head,
                            TypeHeadRef::BuiltinSet
                        );
                    }
                    let atomic = expected.2.source_atomic_formula().unwrap();
                    assert!(!atomic.requests().is_empty());
                    assert!(atomic.requests().iter().all(|(_, request)| request.kind()
                        == SourceAtomicRequestKind::OperandExpectedType));
                    assert_eq!(expected.3.source_atomic_formula(), Some(atomic));
                    let original_type = owner.storage().get(&typed).unwrap();
                    let original_reg = owner.storage().get(&registration).unwrap();
                    let before = (
                        input.clone(),
                        typed.clone(),
                        registration.clone(),
                        (*original_type).clone(),
                        (*original_reg).clone(),
                        session.captured().clone(),
                        expected.clone(),
                    );
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(if open {
                                &[0xff]
                            } else {
                                b"open theorem Changed: 0 = 1;\n"
                            });
                        } else if stage == 2 {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        for _ in 0..2 {
                            assert_eq!(
                                inputs
                                    .native_theorem_metadata_with_type_registration_parents(
                                        &input, owner
                                    )
                                    .unwrap(),
                                Some(expected.clone())
                            );
                        }
                    }
                    assert_eq!(
                        (
                            input,
                            typed.clone(),
                            registration.clone(),
                            (*owner.storage().get(&typed).unwrap()).clone(),
                            (*owner.storage().get(&registration).unwrap()).clone(),
                            session.captured().clone(),
                            expected
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 6);
                    assert_eq!(collected.abandoned_slots_dropped, 0);
                },
            );
            if variant >= 2 && positions.len() == 2 {
                break;
            }
        }
    }
    assert_eq!(positions, [0, 1].into_iter().collect());
}

#[test]
fn native_theorem_metadata_type_registration_caller_preserves_refusals() {
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing",
        "extra",
        "wrong",
        "hash",
        "dependencies",
        "foreign-owner",
        "obsolete",
        "collected",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let child = |parents: Vec<AnyPhaseOutputRef>, key, dependencies| {
                PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        key,
                        dependencies,
                        parents
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                )
            };
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = child(
                vec![signature.erase()],
                old.identities().input_hash(),
                vec![],
            );
            let typed = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
                .unwrap();
            let reg_input = child(vec![typed.erase()], old.identities().input_hash(), vec![]);
            let registration = inputs
                .publish_native_theorem_registration_closure(&reg_input, owner)
                .unwrap()
                .unwrap();
            let input = child(
                match case {
                    "missing" => vec![typed.erase()],
                    "extra" => vec![typed.erase(), registration.erase(), signature.erase()],
                    "wrong" => vec![typed.erase(), signature.erase()],
                    _ => vec![typed.erase(), registration.erase()],
                },
                if case == "hash" {
                    hash(0xe5)
                } else {
                    old.identities().input_hash()
                },
                if case == "dependencies" {
                    vec![hash(0xe6)]
                } else {
                    vec![]
                },
            );
            if case == "obsolete" {
                owner.mark_obsolete(old.snapshot).unwrap();
            } else if case == "collected" {
                assert_eq!(
                    owner
                        .storage()
                        .collect(mizar_ir::storage::CollectInput {
                            snapshot: old.snapshot,
                            protected_outputs: vec![]
                        })
                        .outputs_dropped,
                    6
                );
            }
            let foreign = PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            );
            foreign.register_current_snapshot(old.snapshot);
            let selected = if case == "foreign-owner" {
                &foreign
            } else {
                owner.as_ref()
            };
            let actual = inputs
                .native_theorem_metadata_with_type_registration_parents(&input, selected)
                .unwrap_err();
            if case == "dependencies" {
                let raw_input = child(
                    vec![typed.erase()],
                    old.identities().input_hash(),
                    input.identities().dependency_hashes().to_vec(),
                );
                assert_eq!(
                    actual,
                    inputs
                        .native_theorem_registration_closure_with_type_parent(&raw_input, selected)
                        .unwrap_err()
                );
            }
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 6 }
            );
            assert_eq!(collected.abandoned_slots_dropped, 0);
        });
    }
    for case in ["link", "whole", "affinity"] {
        with_native_summary_parent(
            text,
            false,
            0,
            |_, inputs, old, owner, outputs, (_, ids, snapshots)| {
                use mizar_checker::type_checker::{
                    DeclarationChecker, check_source_theorem_registration_closure as close,
                };
                use mizar_driver::{
                    native_theorem_registration_closure_from_storage_bytes as reg_decode,
                    native_theorem_registration_closure_storage_bytes as reg_bytes,
                    native_theorem_typed_ast_from_storage_bytes as type_decode,
                    native_theorem_typed_ast_storage_bytes as type_bytes,
                };
                use mizar_ir::{
                    publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
                    storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion},
                };
                use mizar_resolve::{
                    names::{
                        SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver,
                    },
                    symbols::{
                        source_theorem_module_candidates as candidates,
                        source_theorem_module_original_inputs as original_inputs,
                    },
                };
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                for (phase, kind) in [
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                }
                let signature = inputs
                    .publish_native_theorem_module_summary(old, owner)
                    .unwrap()
                    .unwrap();
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let frontend = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        frontend,
                        frontend.output_kind(),
                    )
                    .unwrap();
                let output = owner.storage().get(&frontend).unwrap();
                let ast = output.ast.as_ref().unwrap();
                let other_ast = if case == "link" {
                    ast.clone()
                } else {
                    let changed = Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
                    let (_, _, adapter, parent, _) =
                        native_disk_frontend_record_submission(&changed, ids, snapshots, 0);
                    let other = adapter.publisher().storage().get(&parent).unwrap();
                    let other = other.ast.as_ref().unwrap();
                    assert_ne!(other.source_id, ast.source_id);
                    if case == "affinity" {
                        <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
                            &other.canonical_bytes().unwrap(),
                            ast.source_id,
                        )
                        .unwrap()
                    } else {
                        other.clone()
                    }
                };
                let [(arena, env, scope, typed_a, closure_a), (_, _, _, typed_b, closure_b)] = [ast, &other_ast].map(|ast| {
                let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                assert!(candidates(ast, &arena, &env).unwrap().is_some());
                let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env)).unwrap();
                let typed = DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(&arena, &env, &scope).unwrap().2;
                let closure = close(&arena, &env, &scope, &typed).unwrap();
                (arena, env, scope, typed, closure)
            });
                let expected = if case == "affinity" {
                    assert_eq!(typed_a.source_id(), typed_b.source_id());
                    assert_eq!(typed_a.module_id(), typed_b.module_id());
                    assert_ne!(typed_a, typed_b);
                    assert_eq!(closure_a, closure_b);
                    close(&arena, &env, &scope, &typed_b).unwrap_err()
                } else if case == "link" {
                    "native theorem metadata Registration parent link mismatch".to_owned()
                } else {
                    assert_ne!(closure_a, closure_b);
                    "native theorem metadata complete Registration mismatch".to_owned()
                };
                let typed = if case == "affinity" {
                    let bytes = type_bytes(&other_ast, &module, &typed_b).unwrap().unwrap();
                    let source = other_ast.source_id;
                    let decoder_module = module.clone();
                    let phase = IrPipelinePhase::new("TypeChecking");
                    let kind = OutputKind::new("SourceTheoremTypedAst");
                    owner
                        .publish(PublishOutputInput {
                            slot: owner.allocate(
                                old.snapshot,
                                phase.clone(),
                                signature.work_unit().clone(),
                                kind.clone(),
                                SchemaVersion::new(1),
                            ),
                            snapshot: old.snapshot,
                            phase,
                            work_unit: signature.work_unit().clone(),
                            output_kind: kind,
                            schema_version: SchemaVersion::new(1),
                            payload: typed_b,
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                type_decode(bytes, source, &decoder_module)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| BlobDecodeError::new("negative typed replay"))
                            }),
                            parents: vec![signature.erase()],
                            named_input_hashes: signature.lineage().named_input_hashes.clone(),
                            side_tables: IrSideTables::default(),
                            origin: OutputOrigin::PackageSource,
                            target: PublicationTarget::CurrentPackage,
                        })
                        .unwrap()
                } else {
                    let input = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            old.identities().input_hash(),
                            vec![],
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    signature.erase(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    inputs
                        .publish_native_theorem_typed_ast(&input, owner)
                        .unwrap()
                        .unwrap()
                };
                let bytes = reg_bytes(&other_ast, &module, &closure_b).unwrap().unwrap();
                let source = other_ast.source_id;
                let phase = IrPipelinePhase::new("RegistrationResolution");
                let kind = OutputKind::new("SourceTheoremRegistrationClosure");
                let registration = owner
                    .publish(PublishOutputInput {
                        slot: owner.allocate(
                            old.snapshot,
                            phase.clone(),
                            typed.work_unit().clone(),
                            kind.clone(),
                            SchemaVersion::new(1),
                        ),
                        snapshot: old.snapshot,
                        phase,
                        work_unit: typed.work_unit().clone(),
                        output_kind: kind,
                        schema_version: SchemaVersion::new(1),
                        payload: closure_b.clone(),
                        canonical_payload: Some(bytes.clone()),
                        storage_payload: Some(bytes),
                        decode: BlobDecoder::new(move |bytes| {
                            reg_decode(bytes, source, &module)
                                .map_err(BlobDecodeError::new)?
                                .ok_or_else(|| BlobDecodeError::new("negative registration replay"))
                        }),
                        parents: vec![if case == "link" {
                            signature.erase()
                        } else {
                            typed.erase()
                        }],
                        named_input_hashes: typed.lineage().named_input_hashes.clone(),
                        side_tables: IrSideTables::default(),
                        origin: OutputOrigin::PackageSource,
                        target: PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                assert_eq!(*owner.storage().get(&registration).unwrap(), closure_b);
                let input = PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        old.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                owner,
                                old.snapshot,
                                typed.erase(),
                            )
                            .unwrap(),
                            SealedParentOutputHandle::from_published_output(
                                owner,
                                old.snapshot,
                                registration.erase(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                for _ in 0..2 {
                    assert_eq!(
                        inputs
                            .native_theorem_metadata_with_type_registration_parents(&input, owner)
                            .unwrap_err(),
                        expected
                    );
                }
                let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 6);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}

#[test]
fn native_theorem_metadata_publication_seals_genuine_payload_and_streams() {
    use mizar_checker::{
        source_atomic_formula::SourceAtomicRequestKind,
        type_checker::{TypeHeadRef, check_source_theorem_metadata as check},
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_ir::{
        publisher::PublishError,
        storage::{IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{
            source_theorem_module_original_inputs as original_inputs,
            source_theorem_module_semantic_payload as semantics,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let field = |hasher: &mut blake3::Hasher, bytes: &[u8]| {
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    };
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut representations = Vec::new();
        for text in [
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem T: for X being set holds X = X\nproof\n  let X be set;\n  thus X = X;\nend;\n",
            "theorem Renamed: for X being set holds X = X proof let X be set; thus X = X; end;",
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            ),
        ] {
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, (session, _, _)| {
                    let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            unit.clone(),
                        ));
                    }
                    let child = |parents: Vec<AnyPhaseOutputRef>| {
                        PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                parents
                                    .into_iter()
                                    .map(|parent| {
                                        SealedParentOutputHandle::from_published_output(
                                            owner,
                                            old.snapshot,
                                            parent,
                                        )
                                        .unwrap()
                                    })
                                    .collect(),
                            )
                            .unwrap(),
                        )
                    };
                    let signature = inputs
                        .publish_native_theorem_module_summary(old, owner)
                        .unwrap()
                        .unwrap();
                    let type_input = child(vec![signature.erase()]);
                    let typed = inputs
                        .publish_native_theorem_typed_ast(&type_input, owner)
                        .unwrap()
                        .unwrap();
                    let reg_input = child(vec![typed.erase()]);
                    let registration = inputs
                        .publish_native_theorem_registration_closure(&reg_input, owner)
                        .unwrap()
                        .unwrap();
                    let input = child(vec![typed.erase(), registration.erase()]);
                    let (_, loaded, descriptor) =
                        &inputs.captured.unwrap().native_sources().unwrap()[0];
                    let output = Frontend::new(
                        FrontendSourceLoader::new(DiskSourceLoader::new(
                            fixture.root.join("alpha"),
                        )),
                        inputs.dependency_lexical_provider(&[]),
                        MizarParserSeam,
                    )
                    .run_loaded(source_unit_from_loaded(
                        loaded.clone(),
                        diagnostic_file_path(descriptor),
                    ))
                    .unwrap();
                    assert!(output.diagnostics.is_empty());
                    assert!(output.cache_keys.ast.is_some());
                    let ast = output.ast.as_ref().unwrap();
                    let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                    let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
                        .unwrap();
                    let expected = check(&arena, &env, &scope).unwrap();
                    for id in [
                        expected.0.source_id(),
                        expected.1.source_id(),
                        expected.2.source_id(),
                        expected.3.source_id(),
                    ] {
                        assert_eq!(id, loaded.source_id);
                    }
                    for actual_module in [
                        expected.0.module_id(),
                        expected.1.module_id(),
                        expected.2.module_id(),
                        expected.3.module_id(),
                    ] {
                        assert_eq!(actual_module, &module);
                    }
                    let normalized = expected
                        .3
                        .nodes()
                        .iter()
                        .filter_map(|(_, node)| node.final_type)
                        .collect::<Vec<_>>();
                    assert!(!normalized.is_empty());
                    for id in normalized {
                        assert_eq!(
                            expected.1.normalized_types().get(id).unwrap().head,
                            TypeHeadRef::BuiltinSet
                        );
                    }
                    let atomic = expected.2.source_atomic_formula().unwrap();
                    assert!(!atomic.requests().is_empty());
                    assert!(atomic.requests().iter().all(|(_, request)| request.kind()
                        == SourceAtomicRequestKind::OperandExpectedType));
                    assert_eq!(expected.3.source_atomic_formula(), Some(atomic));
                    let opaque = semantics(ast, &module).unwrap().unwrap().2;
                    let mut content_bytes =
                        b"mizar-driver/native-theorem-metadata-content/v1\0".to_vec();
                    content_bytes.extend_from_slice(&(opaque.len() as u64).to_le_bytes());
                    content_bytes.extend_from_slice(&opaque);
                    let syntax = ast.canonical_bytes().unwrap();
                    let mut bytes = b"mizar-driver/native-theorem-metadata-storage/v1\0".to_vec();
                    for value in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
                        bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
                        bytes.extend_from_slice(value);
                    }
                    let version = &inputs.snapshot.source_versions[0];
                    let mut content = blake3::Hasher::new();
                    field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                    field(&mut content, &content_bytes);
                    content.update(&2_u64.to_le_bytes());
                    let mut parents = [typed.erase(), registration.erase()];
                    parents.sort_by(|a, b| {
                        a.content_hash()
                            .as_bytes()
                            .cmp(b.content_hash().as_bytes())
                            .then_with(|| {
                                a.side_table_hash()
                                    .as_bytes()
                                    .cmp(b.side_table_hash().as_bytes())
                            })
                    });
                    for parent in &parents {
                        field(&mut content, b"parent_content");
                        content.update(parent.content_hash().as_bytes());
                    }
                    content.update(&1_u64.to_le_bytes());
                    for value in [
                        "source",
                        SOURCE_UNIT_CACHE_KEY_VERSION,
                        "mizar-ir/publisher-named-input-hash/v1",
                    ] {
                        field(&mut content, value.as_bytes());
                    }
                    content.update(source_key(version).as_bytes());
                    let expected_content = Hash::from_bytes(*content.finalize().as_bytes());
                    let mut blob = blake3::Hasher::new();
                    field(&mut blob, b"mizar-ir/content-blob/v1");
                    blob.update(&1_u32.to_le_bytes());
                    field(&mut blob, &bytes);
                    let expected_blob = Hash::from_bytes(*blob.finalize().as_bytes());
                    let storage_digest = Hash::from_bytes(blake3::derive_key(
                        "mizar-driver/native-theorem-metadata-storage-side-table/v1",
                        &bytes,
                    ));
                    let side_tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "overload-theorem-metadata-storage",
                            version.normalized_path.as_str(),
                            storage_digest,
                        )],
                        ..IrSideTables::default()
                    };
                    let mut side = blake3::Hasher::new();
                    field(&mut side, b"mizar-ir/publisher-side-table-hash/v1");
                    for category in [
                        "source_maps",
                        "diagnostics",
                        "explanation_refs",
                        "documentation_attachments",
                    ] {
                        field(&mut side, category.as_bytes());
                        side.update(&u64::from(category == "source_maps").to_le_bytes());
                        if category == "source_maps" {
                            field(&mut side, b"overload-theorem-metadata-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(storage_digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    let before = (
                        input.clone(),
                        (*owner.storage().get(&typed).unwrap()).clone(),
                        (*owner.storage().get(&registration).unwrap()).clone(),
                        session.captured().clone(),
                        expected.clone(),
                    );
                    let sealed = inputs
                        .publish_native_theorem_metadata(&input, owner)
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), input.snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("OverloadResolution"));
                    assert_eq!(
                        sealed.output_kind(),
                        &OutputKind::new("SourceTheoremMetadata")
                    );
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), &unit);
                    assert_eq!(
                        sealed.lineage().parents,
                        input
                            .parent_outputs()
                            .iter()
                            .map(|parent| parent.output())
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        typed.lineage().named_input_hashes
                    );
                    assert_eq!(sealed.content_hash(), expected_content);
                    assert_eq!(sealed.side_table_hash(), expected_side);
                    assert_eq!(*owner.storage().side_tables(&sealed).unwrap(), side_tables);
                    match sealed.placement() {
                        StoragePlacement::Resident => assert!(!open),
                        StoragePlacement::Blob { blob, len } => {
                            assert!(open);
                            assert_eq!(blob.hash(), expected_blob);
                            assert_eq!(*len, bytes.len());
                        }
                        _ => panic!("unexpected placement"),
                    }
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(if open {
                                &[0xff]
                            } else {
                                b"open theorem Changed: 0 = 1;\n"
                            });
                        } else if stage == 2 {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                        let duplicate = PublishError::Storage {
                            error: Box::new(StorageError::OutputAlreadyStored {
                                output: sealed.output(),
                            }),
                        }
                        .to_string();
                        assert_eq!(
                            inputs
                                .publish_native_theorem_metadata(&input, owner)
                                .unwrap_err(),
                            duplicate
                        );
                        owner
                            .validate_published_output(input.snapshot, sealed.any())
                            .unwrap();
                        assert_eq!(
                            owner
                                .validate_current_output(input.snapshot, sealed.any())
                                .is_ok(),
                            !open
                        );
                    }
                    assert_eq!(
                        (
                            input,
                            (*owner.storage().get(&typed).unwrap()).clone(),
                            (*owner.storage().get(&registration).unwrap()).clone(),
                            session.captured().clone(),
                            expected
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 7);
                    assert_eq!(collected.abandoned_slots_dropped, 3);
                    representations.push((content_bytes, bytes));
                },
            );
        }
        assert_eq!(representations[0].0, representations[1].0);
        assert_ne!(representations[0].0, representations[2].0);
        assert_ne!(representations[0].1, representations[1].1);
    }
}

#[test]
fn native_theorem_metadata_publication_preserves_refusals() {
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    let text = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for case in [
        "missing",
        "wrong",
        "hash",
        "obsolete",
        "collected",
        "foreign-owner",
        "denied",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            let unit = IrWorkUnit::new("\"alpha\":\"main\"");
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    unit.clone(),
                ));
            }
            let child = |parents: Vec<AnyPhaseOutputRef>, key| {
                PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        key,
                        vec![],
                        parents
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                )
            };
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = child(vec![signature.erase()], old.identities().input_hash());
            let typed = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
                .unwrap();
            let reg_input = child(vec![typed.erase()], old.identities().input_hash());
            let registration = inputs
                .publish_native_theorem_registration_closure(&reg_input, owner)
                .unwrap()
                .unwrap();
            let input = child(
                match case {
                    "missing" => vec![typed.erase()],
                    "wrong" => vec![typed.erase(), signature.erase()],
                    _ => vec![typed.erase(), registration.erase()],
                },
                if case == "hash" {
                    hash(0xd9)
                } else {
                    old.identities().input_hash()
                },
            );
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("OverloadResolution"),
                    OutputKind::new("SourceTheoremMetadata"),
                    unit.clone(),
                ));
            }
            if case == "obsolete" {
                owner.mark_obsolete(old.snapshot).unwrap();
            } else if case == "collected" {
                assert_eq!(
                    owner
                        .storage()
                        .collect(CollectInput {
                            snapshot: old.snapshot,
                            protected_outputs: vec![]
                        })
                        .outputs_dropped,
                    6
                );
            }
            let foreign = PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            );
            foreign.register_current_snapshot(old.snapshot);
            let selected = if case == "foreign-owner" {
                &foreign
            } else {
                owner.as_ref()
            };
            let raw =
                inputs.native_theorem_metadata_with_type_registration_parents(&input, selected);
            let actual = inputs
                .publish_native_theorem_metadata(&input, selected)
                .unwrap_err();
            if case == "denied" {
                assert!(raw.unwrap().is_some());
                assert_eq!(
                    actual,
                    PublishError::WorkUnitNotAllowed {
                        phase: IrPipelinePhase::new("OverloadResolution"),
                        work_unit: unit,
                        output_kind: OutputKind::new("SourceTheoremMetadata")
                    }
                    .to_string()
                );
            } else {
                assert_eq!(actual, raw.unwrap_err(), "{case}");
            }
            let collected = owner.storage().collect(CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 6 }
            );
            assert_eq!(
                collected.abandoned_slots_dropped,
                usize::from(case == "denied")
            );
        });
    }
}
