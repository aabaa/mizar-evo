use super::*;

#[test]
fn native_algorithm_core_storage_replays_genuine_outputs() {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    use mizar_core::{
        control_flow::{
            ControlFlowObligationSiteKind, build_control_flow_ir, build_obligation_seed_handoff,
        },
        core_ir::{
            CoreAlgorithmStmtKind, CoreFormulaKind, ObligationSeedKind, ObligationSeedStatus,
        },
    };
    use mizar_driver::{
        native_algorithm_core_from_storage_bytes, native_algorithm_core_storage_bytes,
    };
    for text in [
        include_str!(
            "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
        ),
        include_str!(
            "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
        ),
        include_str!(
            "../../../../tests/miz/fail/algorithms/fail_proof_verification_algorithm_ensures_unprovable_001.miz"
        ),
        "definition\n :: alpha/trivia control\n let b be object;\n algorithm loopalgo(b) -> object ensures result = b do\n var y := b;\n while not y = b do invariant y = y; y := b; end;\n return y; end; end;\n",
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
                    .submit_captured(session, &ids, &snapshots, submit, Default::default())
                    .unwrap();
                let (inputs, _) = native_disk_lookup_context(&submission, &ids);
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
                let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                    ast,
                    &resolver_module,
                )
                .unwrap()
                .unwrap();
                let neutral = TypedArena::try_new(
                    Some(TypedNodeId::new(arena.arena().root().index())),
                    arena
                        .arena()
                        .iter()
                        .map(|(id, node)| {
                            TypedNode::new(
                                format!("{:?}", node.kind()),
                                node.origin().anchor().clone(),
                            )
                            .with_resolved_node(id)
                            .with_children(
                                node.children()
                                    .iter()
                                    .map(|child| TypedNodeId::new(child.index()))
                                    .collect(),
                            )
                        })
                        .collect(),
                )
                .unwrap();
                let check = mizar_checker::type_checker::check_source_algorithm_types(
                    &arena, &neutral, &env,
                )
                .unwrap();
                let expected = mizar_core::elaborator::lower_source_algorithms(&check).unwrap();
                let expected_flow = build_control_flow_ir(&expected);
                let expected_handoff = build_obligation_seed_handoff(&expected, &expected_flow);
                assert_eq!(expected.source_id(), loaded.source_id);
                assert_eq!(
                    expected.module_id(),
                    &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                );
                assert_eq!(expected.items().len(), 1);
                assert_eq!(expected.algorithms().len(), 1);
                assert!(!expected.algorithm_statements().is_empty());
                assert!(expected.proofs().is_empty());
                let choice = text.contains("return the set");
                assert_eq!(
                    expected.obligation_seeds().iter().any(|(_, seed)| seed.kind
                        == ObligationSeedKind::GeneratedNonEmptiness
                        && seed.status == ObligationSeedStatus::Active),
                    choice
                );
                assert!(expected.diagnostics().is_empty());
                let (_, algorithm) = expected.algorithms().iter().next().unwrap();
                assert_eq!(algorithm.params.len(), 1);
                assert_eq!(algorithm.contracts.ensures.len(), 1);
                assert!(!algorithm.statements.is_empty());
                assert_eq!(expected_flow.flows.len(), 1);
                assert!(!expected_handoff.entries.is_empty());
                assert!(expected_handoff.entries.iter().all(|(_, entry)| matches!(
                    entry.seed.status,
                    ObligationSeedStatus::Active | ObligationSeedStatus::Deferred
                )));
                assert!(
                    expected_handoff
                        .entries
                        .iter()
                        .any(|(_, entry)| entry.seed.kind
                            == ObligationSeedKind::AlgorithmTermination
                            && entry.seed.status == ObligationSeedStatus::Deferred
                            && entry.flow_site.as_ref().is_some_and(|site| site.kind
                                == ControlFlowObligationSiteKind::PartialTermination))
                );
                let (_, flow) = expected_flow.flows.iter().next().unwrap();
                assert!(!flow.blocks.is_empty());
                assert!(
                    flow.blocks
                        .iter()
                        .any(|(_, block)| !block.statements.is_empty())
                );
                assert!(flow.diagnostics.is_empty());
                if text.contains("while") {
                    let (_, statement) = expected
                        .algorithm_statements()
                        .iter()
                        .find(|(_, statement)| {
                            matches!(statement.kind, CoreAlgorithmStmtKind::While { .. })
                        })
                        .unwrap();
                    let CoreAlgorithmStmtKind::While {
                        condition,
                        invariants,
                        decreasing,
                        body,
                    } = &statement.kind
                    else {
                        unreachable!()
                    };
                    let CoreFormulaKind::Not(equality) =
                        expected.formulas().get(*condition).unwrap().kind
                    else {
                        panic!("while guard must retain negation")
                    };
                    assert!(matches!(
                        expected.formulas().get(equality).unwrap().kind,
                        CoreFormulaKind::Equals { .. }
                    ));
                    assert_eq!(invariants.len(), 1);
                    assert!(matches!(
                        expected.formulas().get(invariants[0]).unwrap().kind,
                        CoreFormulaKind::Equals { .. }
                    ));
                    assert!(decreasing.is_empty());
                    assert_eq!(body.len(), 1);
                    assert!(matches!(
                        expected.algorithm_statements().get(body[0]).unwrap().kind,
                        CoreAlgorithmStmtKind::AssignLocal { .. }
                    ));
                    assert_eq!(flow.loops.len(), 1);
                    assert!(expected_handoff.entries.iter().any(|(_, entry)| {
                        entry.flow_site.as_ref().is_some_and(|site| {
                            site.kind == ControlFlowObligationSiteKind::LoopInvariant
                        })
                    }));
                } else if text.contains("assert") {
                    assert!(flow.loops.is_empty());
                    assert!(
                        expected
                            .algorithm_statements()
                            .iter()
                            .any(|(_, statement)| matches!(
                                statement.kind,
                                CoreAlgorithmStmtKind::Assert { .. }
                            ))
                    );
                    assert!(expected.algorithm_statements().iter().any(|(_, statement)| matches!(&statement.kind, CoreAlgorithmStmtKind::Let { binder, .. } if binder.role.as_str() == "local:const")));
                }
                let syntax = ast.canonical_bytes().unwrap();
                let domain = b"mizar-driver/native-algorithm-core-storage/v1\0";
                let mut bytes = domain.to_vec();
                for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
                    bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                    bytes.extend_from_slice(field);
                }
                let foreign_fixture = Fixture::new(text.as_bytes());
                let (_, _, foreign_adapter, foreign_handle, _) =
                    native_disk_frontend_record_submission(
                        &foreign_fixture,
                        &ids,
                        &snapshots,
                        threshold,
                    );
                let foreign_output = foreign_adapter
                    .publisher()
                    .storage()
                    .get(&foreign_handle)
                    .unwrap();
                let foreign_ast = foreign_output.ast.as_ref().unwrap();
                let foreign_core = genuine_algorithm_core(foreign_ast, &resolver_module);
                assert_ne!(foreign_core.source_id(), ast.source_id);
                // Explicit interpretation rebinding does not authenticate the other capture.
                let rebound = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
                    &syntax,
                    foreign_core.source_id(),
                )
                .unwrap();
                let (rebound_arena, rebound_env) =
                    mizar_resolve::symbols::source_theorem_module_original_inputs(
                        &rebound,
                        &resolver_module,
                    )
                    .unwrap()
                    .unwrap();
                let rebound_neutral = TypedArena::try_new(
                    Some(TypedNodeId::new(rebound_arena.arena().root().index())),
                    rebound_arena
                        .arena()
                        .iter()
                        .map(|(id, node)| {
                            TypedNode::new(
                                format!("{:?}", node.kind()),
                                node.origin().anchor().clone(),
                            )
                            .with_resolved_node(id)
                            .with_children(
                                node.children()
                                    .iter()
                                    .map(|child| TypedNodeId::new(child.index()))
                                    .collect(),
                            )
                        })
                        .collect(),
                )
                .unwrap();
                let rebound_check = mizar_checker::type_checker::check_source_algorithm_types(
                    &rebound_arena,
                    &rebound_neutral,
                    &rebound_env,
                )
                .unwrap();
                let rebound_expected =
                    mizar_core::elaborator::lower_source_algorithms(&rebound_check).unwrap();
                assert_eq!(rebound_expected, foreign_core);
                assert_eq!(
                    native_algorithm_core_storage_bytes(ast, &resolver_module, &foreign_core),
                    Ok(None)
                );
                let version = &inputs.snapshot.source_versions[0];
                assert_eq!(loaded.source_id, version.source_id);
                assert_eq!(loaded.source_hash, version.source_hash);
                assert_eq!(loaded.source_hash, mizar_session::source::hash_text(text));
                assert_eq!(loaded.origin, version.origin);
                assert_eq!(
                    matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
                    open
                );
                let codec_before = (
                    ast.clone(),
                    resolver_module.clone(),
                    expected.clone(),
                    bytes.clone(),
                );
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    for _ in 0..2 {
                        let stored =
                            native_algorithm_core_storage_bytes(ast, &resolver_module, &expected)
                                .unwrap()
                                .unwrap();
                        assert_eq!(stored, bytes);
                        let mut remaining = stored.strip_prefix(domain).unwrap();
                        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
                            let length = usize::try_from(u64::from_le_bytes(
                                remaining[..8].try_into().unwrap(),
                            ))
                            .unwrap();
                            remaining = &remaining[8..];
                            assert_eq!(&remaining[..length], field);
                            remaining = &remaining[length..];
                        }
                        assert!(remaining.is_empty());
                        let actual = native_algorithm_core_from_storage_bytes(
                            &stored,
                            ast.source_id,
                            &resolver_module,
                        )
                        .unwrap()
                        .unwrap();
                        assert_eq!(
                            actual, expected,
                            "open={open} threshold={threshold} stage={stage}"
                        );
                        assert_eq!(
                            native_algorithm_core_from_storage_bytes(
                                &stored,
                                foreign_core.source_id(),
                                &resolver_module
                            ),
                            Ok(Some(rebound_expected.clone()))
                        );
                        let flow = build_control_flow_ir(&actual);
                        assert_eq!(flow, expected_flow);
                        assert_eq!(
                            build_obligation_seed_handoff(&actual, &flow),
                            expected_handoff
                        );
                    }
                    assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                }
                assert_eq!(captured_input, &descriptor);
                assert_eq!(
                    (ast.clone(), resolver_module, expected, bytes),
                    codec_before
                );
            }
        }
    }
}

#[test]
fn native_algorithm_core_storage_preserves_refusals() {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    use mizar_driver::{
        native_algorithm_core_from_storage_bytes, native_algorithm_core_storage_bytes,
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let valid = Fixture::new(include_bytes!(
        "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
    ));
    let (_, submission, adapter, handle, _) =
        native_disk_frontend_record_submission(&valid, &ids, &snapshots, 0);
    let (_, _source_input) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
    let retained = owner.storage().get(&handle).unwrap();
    let ast = retained.ast.as_ref().unwrap();
    let core = genuine_algorithm_core(ast, &module);
    let domain = b"mizar-driver/native-algorithm-core-storage/v1\0";
    for (text, case) in [
        (
            "definition let a be object; algorithm unsupported(a) -> object do return 0; end; end;",
            "unsupported-algorithm",
        ),
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "ordinary-theorem",
        ),
        (
            include_str!(
                "../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz"
            ),
            "original-none",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, _, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let output = adapter.publisher().storage().get(&handle).unwrap();
        assert!(output.diagnostics.is_empty());
        let ast = output.ast.as_ref().unwrap();
        let original =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module).unwrap();
        let expected = match original {
            None => {
                assert_eq!(case, "original-none");
                Ok(None)
            }
            Some((arena, env)) => {
                assert_ne!(case, "original-none");
                let neutral = TypedArena::try_new(
                    Some(TypedNodeId::new(arena.arena().root().index())),
                    arena
                        .arena()
                        .iter()
                        .map(|(id, node)| {
                            TypedNode::new(
                                format!("{:?}", node.kind()),
                                node.origin().anchor().clone(),
                            )
                            .with_resolved_node(id)
                            .with_children(
                                node.children()
                                    .iter()
                                    .map(|child| TypedNodeId::new(child.index()))
                                    .collect(),
                            )
                        })
                        .collect(),
                )
                .unwrap();
                let lower = mizar_checker::type_checker::check_source_algorithm_types(
                    &arena, &neutral, &env,
                )
                .and_then(|check| mizar_core::elaborator::lower_source_algorithms(&check));
                assert_eq!(
                    lower.as_ref().unwrap_err(),
                    "algorithms.unsupported_source_types"
                );
                lower.map(Some)
            }
        };
        let syntax = ast.canonical_bytes().unwrap();
        let mut bytes = domain.to_vec();
        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
            bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(field);
        }
        let before = (ast.clone(), module.clone(), core.clone(), bytes.clone());
        assert_eq!(
            native_algorithm_core_from_storage_bytes(&bytes, ast.source_id, &module),
            expected,
            "{case}"
        );
        assert_eq!(
            native_algorithm_core_storage_bytes(ast, &module, &core),
            expected.map(|_| None::<Vec<u8>>),
            "{case}"
        );
        assert_eq!((ast.clone(), module.clone(), core.clone(), bytes), before);
    }
    let output = owner.storage().get(&handle).unwrap();
    let ast = output.ast.as_ref().unwrap();
    let syntax = ast.canonical_bytes().unwrap();
    let mut bytes = domain.to_vec();
    for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
        bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(field);
    }
    let mut corruptions = vec![
        Vec::new(),
        bytes[..domain.len() + 3].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
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
    let mut bad_syntax = bytes.clone();
    bad_syntax[bytes.len() - syntax.len()] = b'!';
    corruptions.push(bad_syntax);
    let mut oversized = bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(
            native_algorithm_core_from_storage_bytes(&malformed, ast.source_id, &module),
            Ok(None)
        );
    }
    for wrong_module in [
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("main")),
        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other")),
    ] {
        assert_eq!(
            native_algorithm_core_from_storage_bytes(&bytes, ast.source_id, &wrong_module),
            Ok(None)
        );
    }
}

#[test]
fn native_algorithm_vc_storage_replays_genuine_graphs() {
    use mizar_driver::{
        native_algorithm_core_from_storage_bytes, native_algorithm_core_storage_bytes,
        native_algorithm_vc_from_storage_bytes, native_algorithm_vc_storage_bytes,
    };
    use mizar_vc::vc_ir::{
        GenerationSchemaVersion, SeedVcMapping, VcKind, VcSchemaVersion, VcStatus,
    };
    for (text, obligation, open) in [
        (
            include_str!(
                "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
            ),
            "while",
            false,
        ),
        (
            include_str!(
                "../../../../tests/miz/fail/algorithms/fail_proof_verification_algorithm_ensures_unprovable_001.miz"
            ),
            "pick",
            true,
        ),
        (
            "definition let a be object; algorithm plain(a) -> object do return a; end; end;",
            "empty",
            true,
        ),
    ] {
        let threshold = if open { 0 } else { usize::MAX };
        let (gs, schema_text) = if obligation == "pick" {
            (" \nraw algorithm\t", "")
        } else {
            ("driver-native-algorithm", "driver-native-vc")
        };
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
                    descriptor,
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
        let owner = publisher(snapshot, threshold, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let submission = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(owner.clone())
            .submit_captured(session, &ids, &snapshots, submit, Default::default())
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
        assert_eq!(parent.placement() == &StoragePlacement::Resident, !open);
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
        let typed = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let retained = owner.storage().get(&typed).unwrap();
        let ast = retained.ast.as_ref().unwrap();
        let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        let core = genuine_algorithm_core(ast, &module);
        let (_, loaded, _) = &inputs.captured.as_ref().unwrap().native_sources().unwrap()[0];
        let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        assert_eq!(loaded.text.as_ref(), text);
        assert_eq!(loaded.source_id, ast.source_id);
        assert_eq!(core.source_id(), loaded.source_id);
        assert_eq!(core.module_id(), &module);
        assert_eq!(
            matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
            open
        );
        let core_bytes = native_algorithm_core_storage_bytes(ast, &module, &core)
            .unwrap()
            .unwrap();
        assert_eq!(
            native_algorithm_core_from_storage_bytes(&core_bytes, loaded.source_id, &module),
            Ok(Some(core.clone())),
        );
        let before = (input.clone(), parent.clone(), (*retained).clone());
        let generation = GenerationSchemaVersion::new(gs);
        let schema = VcSchemaVersion::new(schema_text);
        let schema_before = (generation.clone(), schema.clone());
        let expected = mizar_vc::generator::generate_source_algorithm_postconditions(
            &core,
            snapshot,
            &generation,
            &schema,
        )
        .unwrap();
        assert_eq!(expected.source(), loaded.source_id);
        assert_eq!(expected.snapshot(), snapshot);
        assert_eq!(expected.module().as_str(), "package=5:alpha;module=4:main");
        assert_eq!(expected.schema_version(), &schema);
        assert!(expected.vcs().iter().all(|vc| {
            vc.status == VcStatus::Open && vc.anchor.generation_schema_version == generation
        }));
        match obligation {
            "while" => assert!(
                expected
                    .vcs()
                    .iter()
                    .any(|vc| matches!(vc.kind, VcKind::LoopInvariant { .. }))
            ),
            "pick" => {
                assert!(
                    expected
                        .vcs()
                        .iter()
                        .any(|vc| vc.kind == VcKind::AlgorithmPostcondition)
                );
                assert!(
                    expected
                        .vcs()
                        .iter()
                        .any(|vc| vc.kind == VcKind::GeneratedNonEmptiness)
                );
            }
            "empty" => {
                assert!(expected.vcs().is_empty());
                assert!(matches!(
                    expected.seed_accounting(),
                    [mizar_vc::vc_ir::SeedAccounting {
                        seed_status: mizar_core::core_ir::ObligationSeedStatus::Deferred,
                        mapping: SeedVcMapping::NoConcreteVc { .. },
                        ..
                    }]
                ));
            }
            _ => unreachable!(),
        }
        let mut framed = b"mizar-driver/native-algorithm-vc-storage/v1\0".to_vec();
        for field in [
            generation.as_str().as_bytes(),
            schema.as_str().as_bytes(),
            core_bytes.as_slice(),
        ] {
            framed.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            framed.extend_from_slice(field);
        }
        let bytes = native_algorithm_vc_storage_bytes(
            ast,
            &module,
            &core,
            &expected,
            snapshot,
            &generation,
        )
        .unwrap()
        .unwrap();
        assert_eq!(bytes, framed);
        for stage in 0..3 {
            if stage == 1 {
                fixture.write(b"open theorem Changed: 0 = 1;\n");
            } else if stage == 2 {
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            }
            assert_eq!(
                native_algorithm_vc_storage_bytes(
                    ast,
                    &module,
                    &core,
                    &expected,
                    snapshot,
                    &generation,
                ),
                Ok(Some(bytes.clone()))
            );
            assert_eq!(
                native_algorithm_vc_from_storage_bytes(&bytes, loaded.source_id, &module, snapshot),
                Ok(Some(expected.clone())),
            );
        }
        if obligation == "while" {
            let alternate = Fixture::new(text.replace("loopalgo", "rebound").as_bytes());
            let (_, alt_submission, _alt_adapter, _alt_handle, _) =
                native_disk_frontend_record_submission(&alternate, &ids, &snapshots, 0);
            let (alt_inputs, alt_source) = native_disk_lookup_context(&alt_submission, &ids);
            let alt_id = alt_inputs.snapshot.source_versions[0].source_id;
            let rebound_core =
                native_algorithm_core_from_storage_bytes(&core_bytes, alt_id, &module)
                    .unwrap()
                    .unwrap();
            let rebound = mizar_vc::generator::generate_source_algorithm_postconditions(
                &rebound_core,
                alt_source.snapshot,
                &generation,
                &schema,
            )
            .unwrap();
            assert_ne!(alt_source.snapshot, snapshot);
            assert_ne!(rebound.source(), expected.source());
            let changed = GenerationSchemaVersion::new("different generation");
            let changed_graph = mizar_vc::generator::generate_source_algorithm_postconditions(
                &core, snapshot, &changed, &schema,
            )
            .unwrap();
            assert_ne!(changed_graph, expected);
            assert_eq!(
                native_algorithm_vc_storage_bytes(
                    ast, &module, &core, &expected, snapshot, &changed,
                ),
                Ok(None)
            );
            assert_eq!(
                native_algorithm_vc_storage_bytes(
                    ast,
                    &module,
                    &core,
                    &rebound,
                    snapshot,
                    &generation,
                ),
                Ok(None)
            );
            assert_eq!(
                native_algorithm_vc_from_storage_bytes(
                    &framed,
                    alt_id,
                    &module,
                    alt_source.snapshot,
                ),
                Ok(Some(rebound))
            );
        }
        assert_eq!((generation, schema), schema_before);
        assert_eq!(
            (
                input,
                parent.clone(),
                (*owner.storage().get(&typed).unwrap()).clone()
            ),
            before
        );
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.outputs_dropped, 2);
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
}

#[test]
fn native_algorithm_vc_storage_preserves_refusals() {
    use mizar_driver::{
        native_algorithm_core_from_storage_bytes, native_algorithm_core_storage_bytes,
        native_algorithm_vc_from_storage_bytes, native_algorithm_vc_storage_bytes,
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let valid = Fixture::new(
        b"definition let a be object; algorithm plain(a) -> object do return a; end; end;",
    );
    let (_, submission, adapter, handle, _) =
        native_disk_frontend_record_submission(&valid, &ids, &snapshots, 0);
    let (_, source_input) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
    let snapshot = source_input.snapshot;
    let retained = owner.storage().get(&handle).unwrap();
    let ast = retained.ast.as_ref().unwrap();
    let core = genuine_algorithm_core(ast, &module);
    let generation = GenerationSchemaVersion::new(" \n\t");
    let schema = VcSchemaVersion::new("");
    let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
        &core,
        snapshot,
        &generation,
        &schema,
    )
    .unwrap();
    let inner = native_algorithm_core_storage_bytes(ast, &module, &core)
        .unwrap()
        .unwrap();
    let domain = b"mizar-driver/native-algorithm-vc-storage/v1\0";
    let mut bytes = domain.to_vec();
    for field in [
        generation.as_str().as_bytes(),
        schema.as_str().as_bytes(),
        inner.as_slice(),
    ] {
        bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(field);
    }
    let mut corruptions = vec![
        bytes[..domain.len() + 3].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
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
    for fields in [
        [&[0xff][..], schema.as_str().as_bytes(), inner.as_slice()],
        [
            generation.as_str().as_bytes(),
            &[0xff][..],
            inner.as_slice(),
        ],
    ] {
        let mut bad_utf8 = domain.to_vec();
        for field in fields {
            bad_utf8.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            bad_utf8.extend_from_slice(field);
        }
        corruptions.push(bad_utf8);
    }
    let mut oversized = bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(
            native_algorithm_vc_from_storage_bytes(&malformed, ast.source_id, &module, snapshot,),
            Ok(None)
        );
    }
    let huge_schema = VcSchemaVersion::new("x".repeat(32 * 1024 * 1024));
    let huge_graph = mizar_vc::generator::generate_source_algorithm_postconditions(
        &core,
        snapshot,
        &generation,
        &huge_schema,
    )
    .unwrap();
    assert!(huge_graph.vcs().is_empty());
    assert_eq!(
        native_algorithm_vc_storage_bytes(ast, &module, &core, &huge_graph, snapshot, &generation,),
        Ok(None)
    );
    for (text, case) in [
        (
            include_str!(
                "../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz"
            ),
            "none",
        ),
        (
            "definition let a be object; algorithm unsupported(a) -> object do return 0; end; end;",
            "string",
        ),
    ] {
        let refused = Fixture::new(text.as_bytes());
        let (_, _, refused_adapter, refused_handle, _) =
            native_disk_frontend_record_submission(&refused, &ids, &snapshots, 0);
        let refused_owner = refused_adapter.publisher();
        let output = refused_owner.storage().get(&refused_handle).unwrap();
        let refused_ast = output.ast.as_ref().unwrap();
        let old = native_algorithm_core_storage_bytes(refused_ast, &module, &core);
        if case == "none" {
            assert_eq!(old, Ok(None));
        } else {
            assert_eq!(
                old.as_ref().unwrap_err(),
                "algorithms.unsupported_source_types"
            );
        }
        assert_eq!(
            native_algorithm_vc_storage_bytes(
                refused_ast,
                &module,
                &core,
                &vcs,
                snapshot,
                &generation,
            ),
            old
        );
        let syntax = refused_ast.canonical_bytes().unwrap();
        let mut actual_inner = b"mizar-driver/native-algorithm-core-storage/v1\0".to_vec();
        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
            actual_inner.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            actual_inner.extend_from_slice(field);
        }
        let decoded =
            native_algorithm_core_from_storage_bytes(&actual_inner, refused_ast.source_id, &module);
        assert_eq!(
            decoded.as_ref().map(Option::is_none),
            old.as_ref().map(Option::is_none)
        );
        let mut outer = domain.to_vec();
        for field in [
            generation.as_str().as_bytes(),
            schema.as_str().as_bytes(),
            actual_inner.as_slice(),
        ] {
            outer.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            outer.extend_from_slice(field);
        }
        assert_eq!(
            native_algorithm_vc_from_storage_bytes(
                &outer,
                refused_ast.source_id,
                &module,
                snapshot,
            ),
            decoded.map(|_| None)
        );
        let collected = refused_owner
            .storage()
            .collect(mizar_ir::storage::CollectInput {
                snapshot: refused_handle.erase().snapshot(),
                protected_outputs: vec![],
            });
        assert_eq!(collected.outputs_dropped, 2);
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
    let twice = Fixture::new(
        b"definition let a be object; algorithm twice(a) -> object do return a; return a; end; end;",
    );
    let (_, twice_submission, twice_adapter, twice_handle, _) =
        native_disk_frontend_record_submission(&twice, &ids, &snapshots, 0);
    let (_, twice_source) = native_disk_lookup_context(&twice_submission, &ids);
    let twice_owner = twice_adapter.publisher();
    let twice_retained = twice_owner.storage().get(&twice_handle).unwrap();
    let twice_ast = twice_retained.ast.as_ref().unwrap();
    let twice_core = genuine_algorithm_core(twice_ast, &module);
    let direct = mizar_vc::generator::generate_source_algorithm_postconditions(
        &twice_core,
        twice_source.snapshot,
        &generation,
        &schema,
    )
    .unwrap_err();
    assert_eq!(direct, "algorithms.postcondition.unsupported_core");
    let twice_inner = native_algorithm_core_storage_bytes(twice_ast, &module, &twice_core)
        .unwrap()
        .unwrap();
    assert_eq!(
        native_algorithm_core_from_storage_bytes(&twice_inner, twice_ast.source_id, &module,),
        Ok(Some(twice_core.clone()))
    );
    assert_eq!(
        native_algorithm_vc_storage_bytes(
            twice_ast,
            &module,
            &twice_core,
            &vcs,
            twice_source.snapshot,
            &generation,
        ),
        Err(direct.clone())
    );
    let mut twice_outer = domain.to_vec();
    for field in [
        generation.as_str().as_bytes(),
        schema.as_str().as_bytes(),
        twice_inner.as_slice(),
    ] {
        twice_outer.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        twice_outer.extend_from_slice(field);
    }
    assert_eq!(
        native_algorithm_vc_from_storage_bytes(
            &twice_outer,
            twice_ast.source_id,
            &module,
            twice_source.snapshot,
        ),
        Err(direct)
    );
}

fn genuine_algorithm_core(
    ast: &<MizarParserSeam as ParserSeam>::Ast,
    module: &ResolverModuleId,
) -> mizar_core::core_ir::CoreIr {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};

    let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(ast, module)
        .unwrap()
        .unwrap();
    let neutral = TypedArena::try_new(
        Some(TypedNodeId::new(arena.arena().root().index())),
        arena
            .arena()
            .iter()
            .map(|(id, node)| {
                TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                    .with_resolved_node(id)
                    .with_children(
                        node.children()
                            .iter()
                            .map(|child| TypedNodeId::new(child.index()))
                            .collect(),
                    )
            })
            .collect(),
    )
    .unwrap();
    let check =
        mizar_checker::type_checker::check_source_algorithm_types(&arena, &neutral, &env).unwrap();
    mizar_core::elaborator::lower_source_algorithms(&check).unwrap()
}
