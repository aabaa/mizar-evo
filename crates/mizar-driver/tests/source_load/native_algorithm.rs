use super::*;

#[test]
fn native_algorithm_core_caller_preserves_genuine_outputs() {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    use mizar_core::{
        control_flow::{
            ControlFlowObligationSiteKind, build_control_flow_ir, build_obligation_seed_handoff,
        },
        core_ir::{
            CoreAlgorithmStmtKind, CoreFormulaKind, ObligationSeedKind, ObligationSeedStatus,
        },
    };
    for text in [
        include_str!(
            "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
        ),
        include_str!(
            "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
        ),
        "definition\n let b be object;\n algorithm loopalgo(b) -> object ensures result = b do\n var y := b;\n while not y = b do invariant y = y; y := b; end;\n return y; end; end;\n",
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
                assert!(expected.obligation_seeds().is_empty());
                assert!(expected.diagnostics().is_empty());
                let (_, algorithm) = expected.algorithms().iter().next().unwrap();
                assert_eq!(algorithm.params.len(), 1);
                assert_eq!(algorithm.contracts.ensures.len(), 1);
                assert!(!algorithm.statements.is_empty());
                assert_eq!(expected_flow.flows.len(), 1);
                assert!(!expected_handoff.entries.is_empty());
                assert!(
                    expected_handoff
                        .entries
                        .iter()
                        .all(|(_, entry)| entry.seed.status == ObligationSeedStatus::Deferred)
                );
                assert!(
                    expected_handoff
                        .entries
                        .iter()
                        .any(|(_, entry)| entry.seed.kind
                            == ObligationSeedKind::AlgorithmTermination
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
                } else {
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
                let version = &inputs.snapshot.source_versions[0];
                assert_eq!(loaded.source_id, version.source_id);
                assert_eq!(loaded.source_hash, version.source_hash);
                assert_eq!(loaded.source_hash, mizar_session::source::hash_text(text));
                assert_eq!(loaded.origin, version.origin);
                assert_eq!(
                    matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
                    open
                );
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
                        let actual = inputs
                            .native_algorithm_core_with_frontend_parent(&input, &owner)
                            .unwrap()
                            .unwrap();
                        assert_eq!(
                            actual, expected,
                            "open={open} threshold={threshold} stage={stage}"
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
fn native_algorithm_core_caller_preserves_refusals() {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
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
        assert!(retained.diagnostics.is_empty());
        let WorkUnit::Module { module } = &input.work_unit else {
            unreachable!()
        };
        let resolver_module = ResolverModuleId::new(module.package.clone(), module.path.clone());
        let original = mizar_resolve::symbols::source_theorem_module_original_inputs(
            retained.ast.as_ref().unwrap(),
            &resolver_module,
        )
        .unwrap();
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
        assert_eq!(
            inputs.native_algorithm_core_with_frontend_parent(&input, owner),
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
                (*owner.storage().get(&handle).unwrap()).clone(),
            ),
            before,
        );
    }
    let text = include_str!(
        "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
    );
    let fixture = Fixture::new(text.as_bytes());
    let foreign = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, foreign_submission, foreign_adapter, foreign_handle, _) =
        native_disk_frontend_record_submission(&foreign, &ids, &snapshots, 0);
    for case in [
        "absent",
        "foreign",
        "hash",
        "obsolete",
        "collected",
        "complete-mismatch",
    ] {
        let (_, submission, adapter, handle, frontend_input) =
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
        let alternative;
        let selected_owner = if case == "complete-mismatch" {
            alternative = std::sync::Arc::new(PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            alternative.register_current_snapshot(snapshot);
            let ancestor = frontend_input.parent_outputs()[0].as_output_ref().lineage();
            alternative
                .registry()
                .register_output(mizar_ir::identity::OutputIdentityInput {
                    snapshot: ancestor.snapshot,
                    phase: ancestor.phase.clone(),
                    work_unit: ancestor.work_unit.clone(),
                    output_kind: ancestor.output_kind.clone(),
                    content_hash: ancestor.content_hash,
                    side_table_hash: ancestor.side_table_hash,
                    parents: ancestor.parents.clone(),
                    named_input_hashes: ancestor.named_input_hashes.clone(),
                })
                .unwrap();
            &alternative
        } else {
            owner
        };
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
            "hash" => hash = Hash::from_bytes([0x62; 32]),
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
            }
            "complete-mismatch" => {
                // This publication is an adversarial derivative, never a positive producer.
                let original = owner.storage().get(&handle).unwrap();
                let mut negative = (*original).clone();
                negative.cache_keys.ast.as_mut().unwrap().parser_inputs_hash =
                    Hash::from_bytes([0x63; 32]);
                assert_eq!(negative.tokens, original.tokens);
                assert_ne!(negative.cache_keys.ast, original.cache_keys.ast);
                let raw = negative.canonical_disk_bytes().unwrap();
                let descriptor = inputs.captured.unwrap().native_sources().unwrap()[0]
                    .2
                    .clone();
                let source_id = negative.source.source_id;
                assert_eq!(
                    FrontendOutput::from_canonical_storage_bytes(&raw, source_id, &descriptor)
                        .unwrap(),
                    negative
                );
                selected_owner.allow_work_unit(AllowedWorkUnit::new(
                    handle.phase().clone(),
                    handle.output_kind().clone(),
                    handle.work_unit().clone(),
                ));
                let negative = selected_owner
                    .publish(mizar_ir::publisher::PublishOutputInput {
                        slot: selected_owner.allocate(
                            snapshot,
                            handle.phase().clone(),
                            handle.work_unit().clone(),
                            handle.output_kind().clone(),
                            handle.schema_version(),
                        ),
                        snapshot,
                        phase: handle.phase().clone(),
                        work_unit: handle.work_unit().clone(),
                        output_kind: handle.output_kind().clone(),
                        schema_version: handle.schema_version(),
                        payload: negative,
                        canonical_payload: Some(raw.clone()),
                        storage_payload: Some(raw),
                        decode: mizar_ir::storage::BlobDecoder::new(move |bytes| {
                            FrontendOutput::from_canonical_storage_bytes(
                                bytes,
                                source_id,
                                &descriptor,
                            )
                            .ok_or_else(|| {
                                mizar_ir::storage::BlobDecodeError::new("negative aggregate")
                            })
                        }),
                        parents: vec![frontend_input.parent_outputs()[0].as_output_ref().clone()],
                        named_input_hashes: handle.lineage().named_input_hashes.clone(),
                        side_tables: (*owner.storage().side_tables(&handle).unwrap()).clone(),
                        origin: mizar_ir::publisher::OutputOrigin::PackageSource,
                        target: mizar_ir::publisher::PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        selected_owner,
                        snapshot,
                        negative.erase(),
                    )
                    .unwrap(),
                ];
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
            .native_theorem_module_candidates_with_frontend_parent(&input, selected_owner)
            .unwrap_err();
        if case == "complete-mismatch" {
            assert_eq!(
                expected,
                "native Frontend resolver complete output mismatch"
            );
        }
        assert_eq!(
            inputs.native_algorithm_core_with_frontend_parent(&input, selected_owner),
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
                handle,
            ),
            before,
        );
    }
}
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
                assert_eq!(
                    inputs
                        .native_algorithm_core_with_frontend_parent(&input, &owner)
                        .unwrap(),
                    Some(expected.clone())
                );
                let syntax = ast.canonical_bytes().unwrap();
                let domain = b"mizar-driver/native-algorithm-core-storage/v1\0";
                let mut bytes = domain.to_vec();
                for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
                    bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                    bytes.extend_from_slice(field);
                }
                let foreign_fixture = Fixture::new(text.as_bytes());
                let (_, foreign_submission, foreign_adapter, foreign_handle, _) =
                    native_disk_frontend_record_submission(
                        &foreign_fixture,
                        &ids,
                        &snapshots,
                        threshold,
                    );
                let (_, foreign_input) = native_disk_lookup_context(&foreign_submission, &ids);
                let foreign_dispatch = PhaseInput::new(
                    foreign_input.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        foreign_input.snapshot,
                        foreign_input.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                foreign_adapter.publisher(),
                                foreign_input.snapshot,
                                foreign_handle.erase(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let (foreign_inputs, _) = native_disk_lookup_context(&foreign_submission, &ids);
                let foreign_core = foreign_inputs
                    .native_algorithm_core_with_frontend_parent(
                        &foreign_dispatch,
                        foreign_adapter.publisher(),
                    )
                    .unwrap()
                    .unwrap();
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
                let before = (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    input.clone(),
                    parent.clone(),
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
    let core = inputs
        .native_algorithm_core_with_frontend_parent(&input, owner)
        .unwrap()
        .unwrap();
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
fn native_algorithm_core_publication_seals_genuine_core_and_streams() {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    use mizar_core::{
        control_flow::{build_control_flow_ir, build_obligation_seed_handoff},
        core_ir::{ObligationSeedKind, ObligationSeedStatus},
    };
    use mizar_driver::{
        native_algorithm_core_from_storage_bytes, native_algorithm_core_storage_bytes,
    };
    let mut alpha_control = None;
    for (case, text) in [
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
    ]
    .into_iter()
    .enumerate()
    {
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
                let unit = WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        PackageId::new("alpha"),
                        ModulePath::new("main"),
                    ),
                };
                let work_unit = IrWorkUnit::new("\"alpha\":\"main\"");
                let owner = publisher(snapshot, threshold, &unit);
                allow_frontend(&owner, &unit);
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("Elaboration"),
                    OutputKind::new("SourceAlgorithmCore"),
                    work_unit.clone(),
                ));
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
                assert!(retained.diagnostics.is_empty());
                let ast = retained.ast.as_ref().unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let (arena, env) =
                    mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
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
                assert_eq!(
                    inputs
                        .native_algorithm_core_with_frontend_parent(&input, &owner)
                        .unwrap(),
                    Some(expected.clone())
                );
                let version = &inputs.snapshot.source_versions[0];
                let (_, loaded, captured_input) =
                    &inputs.captured.unwrap().native_sources().unwrap()[0];
                assert_eq!(loaded.text.as_ref(), text);
                assert_eq!(expected.source_id(), loaded.source_id);
                assert_eq!(expected.module_id(), &module);
                assert_eq!(expected.algorithms().len(), 1);
                assert!(!expected.algorithm_statements().is_empty());
                let flow = build_control_flow_ir(&expected);
                let handoff = build_obligation_seed_handoff(&expected, &flow);
                assert_eq!(flow.flows.len(), 1);
                assert!(!handoff.entries.is_empty());
                assert!(handoff.entries.iter().any(|(_, entry)| {
                    entry.seed.kind == ObligationSeedKind::AlgorithmTermination
                        && entry.seed.status == ObligationSeedStatus::Deferred
                }));
                assert_eq!(
                    expected.obligation_seeds().iter().any(|(_, seed)| {
                        seed.kind == ObligationSeedKind::GeneratedNonEmptiness
                            && seed.status == ObligationSeedStatus::Active
                    }),
                    text.contains("return the set")
                );
                let semantic = mizar_core::elaborator::source_algorithm_core_semantic_payload(
                    &check, &expected,
                )
                .unwrap();
                let bytes = native_algorithm_core_storage_bytes(ast, &module, &expected)
                    .unwrap()
                    .unwrap();
                assert_ne!(semantic, bytes);
                assert_eq!(
                    native_algorithm_core_from_storage_bytes(&bytes, loaded.source_id, &module)
                        .unwrap(),
                    Some(expected.clone())
                );
                let mut hasher = blake3::Hasher::new();
                for field in [
                    b"mizar-ir/publisher-content-hash/v1".as_slice(),
                    semantic.as_slice(),
                ] {
                    hasher.update(&(field.len() as u64).to_le_bytes());
                    hasher.update(field);
                }
                hasher.update(&1_u64.to_le_bytes());
                hasher.update(&("parent_content".len() as u64).to_le_bytes());
                hasher.update(b"parent_content");
                hasher.update(parent.content_hash().as_bytes());
                hasher.update(&1_u64.to_le_bytes());
                for field in [
                    "source",
                    SOURCE_UNIT_CACHE_KEY_VERSION,
                    "mizar-ir/publisher-named-input-hash/v1",
                ] {
                    hasher.update(&(field.len() as u64).to_le_bytes());
                    hasher.update(field.as_bytes());
                }
                hasher.update(source_key(version).as_bytes());
                let content_hash = Hash::from_bytes(*hasher.finalize().as_bytes());
                if !open && threshold == usize::MAX && (case == 0 || case == 3) {
                    if let Some((baseline_semantic, baseline_content)) = &alpha_control {
                        assert_eq!(&semantic, baseline_semantic);
                        assert_ne!(&content_hash, baseline_content);
                    } else {
                        alpha_control = Some((semantic.clone(), content_hash));
                    }
                }
                let mut blob_hasher = blake3::Hasher::new();
                let domain = b"mizar-ir/content-blob/v1";
                blob_hasher.update(&(domain.len() as u64).to_le_bytes());
                blob_hasher.update(domain);
                blob_hasher.update(&1_u32.to_le_bytes());
                blob_hasher.update(&(bytes.len() as u64).to_le_bytes());
                blob_hasher.update(&bytes);
                let blob_hash = Hash::from_bytes(*blob_hasher.finalize().as_bytes());
                let before = (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    (*retained).clone(),
                    input.clone(),
                    parent.clone(),
                );
                let sealed = inputs
                    .publish_native_algorithm_core(&input, &owner)
                    .unwrap()
                    .unwrap();
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                    assert_eq!(
                        native_algorithm_core_from_storage_bytes(&bytes, loaded.source_id, &module),
                        Ok(Some(expected.clone()))
                    );
                    let duplicate = mizar_ir::publisher::PublishError::Storage {
                        error: Box::new(mizar_ir::storage::StorageError::OutputAlreadyStored {
                            output: sealed.output(),
                        }),
                    }
                    .to_string();
                    assert_eq!(
                        inputs
                            .publish_native_algorithm_core(&input, &owner)
                            .unwrap_err(),
                        duplicate
                    );
                    assert_eq!(sealed.snapshot(), snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("Elaboration"));
                    assert_eq!(sealed.work_unit(), &work_unit);
                    assert_eq!(
                        sealed.output_kind(),
                        &OutputKind::new("SourceAlgorithmCore")
                    );
                    assert_eq!(
                        sealed.schema_version(),
                        mizar_ir::storage::SchemaVersion::new(1)
                    );
                    assert_eq!(sealed.lineage().parents, vec![parent.output()]);
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        vec![mizar_ir::identity::NamedInputHash {
                            name: "source".to_owned(),
                            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                            digest: source_key(version),
                        }]
                    );
                    assert_eq!(sealed.content_hash(), content_hash);
                    match sealed.placement() {
                        StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
                        StoragePlacement::Blob { blob, len } => {
                            assert_eq!(threshold, 0);
                            assert_eq!(*len, bytes.len());
                            assert_eq!(blob.hash(), blob_hash);
                        }
                        _ => panic!("unexpected storage placement"),
                    }
                    assert_eq!(
                        *owner.storage().side_tables(&sealed).unwrap(),
                        mizar_ir::storage::IrSideTables {
                            source_maps: vec![mizar_ir::storage::SideTableRecord::new(
                                "elaborate-algorithm-core-storage",
                                version.normalized_path.as_str(),
                                Hash::from_bytes(blake3::derive_key(
                                    "mizar-driver/native-algorithm-core-storage-side-table/v1",
                                    &bytes
                                )),
                            )],
                            ..mizar_ir::storage::IrSideTables::default()
                        }
                    );
                    assert!(
                        owner
                            .validate_published_output(snapshot, sealed.any())
                            .is_ok()
                    );
                    assert_eq!(
                        owner
                            .validate_current_output(snapshot, sealed.any())
                            .is_ok(),
                        !open
                    );
                }
                assert_eq!(
                    (
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        inputs.module_index.clone(),
                        loaded.clone(),
                        captured_input.clone(),
                        (*owner.storage().get(&typed).unwrap()).clone(),
                        input,
                        parent.clone(),
                    ),
                    before
                );
                assert_eq!(captured_input, &descriptor);
            }
        }
    }
}

#[test]
fn native_algorithm_core_publication_preserves_refusals_and_publisher_boundaries() {
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
        assert!(retained.diagnostics.is_empty());
        let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        let original = mizar_resolve::symbols::source_theorem_module_original_inputs(
            retained.ast.as_ref().unwrap(),
            &module,
        )
        .unwrap();
        assert_eq!(original.is_none(), case == "original-none");
        let old = inputs.native_algorithm_core_with_frontend_parent(&input, owner);
        if case == "original-none" {
            assert_eq!(old, Ok(None));
        } else {
            assert_eq!(
                old.as_ref().unwrap_err(),
                "algorithms.unsupported_source_types"
            );
        }
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            input.clone(),
            handle.clone(),
            (*retained).clone(),
        );
        assert_eq!(
            inputs
                .publish_native_algorithm_core(&input, owner)
                .map(|value| value.map(|handle| (*owner.storage().get(&handle).unwrap()).clone())),
            old,
            "{case}"
        );
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                input,
                handle.clone(),
                (*owner.storage().get(&handle).unwrap()).clone(),
            ),
            before
        );
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot: source_input.snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.outputs_dropped, 2);
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
    for case in ["absent", "obsolete", "collected", "denied-kind"] {
        let fixture = Fixture::new(include_bytes!(
            "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
        ));
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let snapshot = source_input.snapshot;
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("Elaboration"),
            OutputKind::new(if case == "denied-kind" {
                "SourceTheoremCore"
            } else {
                "SourceAlgorithmCore"
            }),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                .unwrap(),
        ];
        match case {
            "absent" => parents.clear(),
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
            }
            _ => {}
        }
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                parents,
            )
            .unwrap(),
        );
        let old = inputs.native_algorithm_core_with_frontend_parent(&input, owner);
        let actual = inputs
            .publish_native_algorithm_core(&input, owner)
            .unwrap_err();
        if case.starts_with("denied-") {
            assert!(old.unwrap().is_some());
            assert_eq!(
                actual,
                mizar_ir::publisher::PublishError::WorkUnitNotAllowed {
                    phase: IrPipelinePhase::new("Elaboration"),
                    work_unit: IrWorkUnit::new("\"alpha\":\"main\""),
                    output_kind: OutputKind::new("SourceAlgorithmCore"),
                }
                .to_string()
            );
        } else {
            assert_eq!(actual, old.unwrap_err(), "{case}");
        }
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            if case == "collected" { 0 } else { 2 }
        );
        assert_eq!(
            collected.abandoned_slots_dropped,
            usize::from(case.starts_with("denied-"))
        );
    }
}

#[test]
fn native_algorithm_vc_caller_preserves_genuine_raw_generation() {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcKind, VcSchemaVersion, VcStatus};
    for (text, obligation) in [
        (
            include_str!(
                "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
            ),
            "while",
        ),
        (
            include_str!(
                "../../../../tests/miz/fail/algorithms/fail_proof_verification_algorithm_ensures_unprovable_001.miz"
            ),
            "pick",
        ),
        (
            "definition let a be object; algorithm plain(a) -> object do return a; end; end;",
            "empty",
        ),
    ] {
        for (open, threshold) in [(false, usize::MAX), (false, 0), (true, 0)] {
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
            let core = inputs
                .native_algorithm_core_with_frontend_parent(&input, &owner)
                .unwrap()
                .unwrap();
            let typed = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    parent,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let retained = owner.storage().get(&typed).unwrap();
            let (_, loaded, captured_input) =
                &inputs.captured.unwrap().native_sources().unwrap()[0];
            assert_eq!(loaded.text.as_ref(), text);
            assert_eq!(captured_input, &descriptor);
            assert_eq!(retained.ast.as_ref().unwrap().source_id, loaded.source_id);
            assert_eq!(core.source_id(), loaded.source_id);
            assert_eq!(
                core.module_id(),
                &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
            );
            let before = ((*retained).clone(), input.clone(), parent.clone());
            for (generation, schema) in [
                ("driver-native-algorithm", "driver-native-vc"),
                (" \nraw algorithm\t", ""),
            ] {
                let generation = GenerationSchemaVersion::new(generation);
                let schema = VcSchemaVersion::new(schema);
                let schemas_before = (generation.clone(), schema.clone());
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
                    "while" => {
                        assert!(
                            expected
                                .vcs()
                                .iter()
                                .any(|vc| matches!(vc.kind, VcKind::LoopInvariant { .. }))
                        );
                        assert!(!expected.seed_accounting().is_empty());
                    }
                    "pick" => {
                        for kind in [
                            VcKind::AlgorithmPostcondition,
                            VcKind::GeneratedNonEmptiness,
                        ] {
                            assert!(expected.vcs().iter().any(|vc| vc.kind == kind));
                        }
                    }
                    "empty" => {
                        assert!(expected.vcs().is_empty());
                        assert!(matches!(
                            expected.seed_accounting(),
                            [mizar_vc::vc_ir::SeedAccounting {
                                seed_status: mizar_core::core_ir::ObligationSeedStatus::Deferred,
                                mapping: mizar_vc::vc_ir::SeedVcMapping::NoConcreteVc { .. },
                                ..
                            }]
                        ));
                    }
                    _ => unreachable!(),
                }
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    for _ in 0..2 {
                        assert_eq!(
                            inputs
                                .native_algorithm_vcs_with_frontend_parent(
                                    &input,
                                    &owner,
                                    &generation,
                                    &schema
                                )
                                .unwrap(),
                            Some(expected.clone()),
                            "{obligation} open={open} threshold={threshold} stage={stage}"
                        );
                    }
                }
                assert_eq!((generation, schema), schemas_before);
            }
            assert_eq!(
                (
                    (*owner.storage().get(&typed).unwrap()).clone(),
                    input,
                    parent.clone()
                ),
                before,
            );
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 2);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        }
    }
}

#[test]
fn native_algorithm_vc_caller_preserves_refusals() {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    for (text, case) in [
        (
            include_str!(
                "../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz"
            ),
            "none",
        ),
        (
            "definition let a be object; algorithm unsupported(a) -> object do return 0; end; end;",
            "unsupported",
        ),
        (
            "definition let a be object; algorithm twice(a) -> object do return a; return a; end; end;",
            "generator",
        ),
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
        let old = inputs.native_algorithm_core_with_frontend_parent(&input, owner);
        if case == "none" {
            assert_eq!(old, Ok(None));
        } else if case != "generator" {
            assert_eq!(
                old.as_ref().unwrap_err(),
                "algorithms.unsupported_source_types"
            );
        }
        let before = (input.clone(), handle.clone());
        let generation = GenerationSchemaVersion::new(" \n");
        let schema = VcSchemaVersion::new("");
        if case == "generator" {
            let expected = mizar_vc::generator::generate_source_algorithm_postconditions(
                old.as_ref().unwrap().as_ref().unwrap(),
                source_input.snapshot,
                &generation,
                &schema,
            )
            .unwrap_err();
            assert_eq!(expected, "algorithms.postcondition.unsupported_core");
            assert_eq!(
                inputs.native_algorithm_vcs_with_frontend_parent(
                    &input,
                    owner,
                    &generation,
                    &schema
                ),
                Err(expected),
            );
        } else {
            assert_eq!(
                inputs.native_algorithm_vcs_with_frontend_parent(
                    &input,
                    owner,
                    &generation,
                    &schema
                ),
                old.map(|_| None),
                "{case}"
            );
        }
        assert_eq!((input, handle), before);
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot: source_input.snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.outputs_dropped, 2);
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
    for case in ["absent", "obsolete", "collected"] {
        let fixture = Fixture::new(include_bytes!(
            "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
        ));
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let snapshot = source_input.snapshot;
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                .unwrap(),
        ];
        match case {
            "absent" => parents.clear(),
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                let dropped = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(dropped.outputs_dropped, 2);
                assert_eq!(dropped.abandoned_slots_dropped, 0);
            }
            _ => unreachable!(),
        }
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                parents,
            )
            .unwrap(),
        );
        let expected = inputs
            .native_algorithm_core_with_frontend_parent(&input, owner)
            .unwrap_err();
        assert_eq!(
            inputs
                .native_algorithm_vcs_with_frontend_parent(
                    &input,
                    owner,
                    &GenerationSchemaVersion::new(""),
                    &VcSchemaVersion::new(" \t"),
                )
                .unwrap_err(),
            expected,
            "{case}"
        );
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            if case == "collected" { 0 } else { 2 }
        );
        assert_eq!(collected.abandoned_slots_dropped, 0);
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
        let core = inputs
            .native_algorithm_core_with_frontend_parent(&input, &owner)
            .unwrap()
            .unwrap();
        let typed = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let retained = owner.storage().get(&typed).unwrap();
        let ast = retained.ast.as_ref().unwrap();
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
        assert_eq!(
            inputs.native_algorithm_vcs_with_frontend_parent(&input, &owner, &generation, &schema,),
            Ok(Some(expected.clone())),
        );
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
    let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
    let snapshot = source_input.snapshot;
    let input = PhaseInput::new(
        source_input.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            snapshot,
            source_input.identities().input_hash(),
            vec![],
            vec![
                SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    let core = inputs
        .native_algorithm_core_with_frontend_parent(&input, owner)
        .unwrap()
        .unwrap();
    let retained = owner.storage().get(&handle).unwrap();
    let ast = retained.ast.as_ref().unwrap();
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
    let (twice_inputs, twice_source) = native_disk_lookup_context(&twice_submission, &ids);
    let twice_owner = twice_adapter.publisher();
    let twice_input = PhaseInput::new(
        twice_source.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            twice_source.snapshot,
            twice_source.identities().input_hash(),
            vec![],
            vec![
                SealedParentOutputHandle::from_published_output(
                    twice_owner,
                    twice_source.snapshot,
                    twice_handle.erase(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    );
    let twice_core = twice_inputs
        .native_algorithm_core_with_frontend_parent(&twice_input, twice_owner)
        .unwrap()
        .unwrap();
    let direct = mizar_vc::generator::generate_source_algorithm_postconditions(
        &twice_core,
        twice_source.snapshot,
        &generation,
        &schema,
    )
    .unwrap_err();
    assert_eq!(direct, "algorithms.postcondition.unsupported_core");
    assert_eq!(
        twice_inputs.native_algorithm_vcs_with_frontend_parent(
            &twice_input,
            twice_owner,
            &generation,
            &schema,
        ),
        Err(direct.clone())
    );
    let twice_retained = twice_owner.storage().get(&twice_handle).unwrap();
    let twice_ast = twice_retained.ast.as_ref().unwrap();
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

#[test]
fn native_algorithm_vc_publication_seals_genuine_graph_and_streams() {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    use mizar_driver::{native_algorithm_vc_from_storage_bytes, native_algorithm_vc_storage_bytes};
    use mizar_vc::vc_ir::{
        GenerationSchemaVersion, SeedVcMapping, VcKind, VcSchemaVersion, VcStatus,
    };
    for (text, obligation, open, generation_text, schema_text) in [
        (
            include_str!(
                "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
            ),
            "while",
            false,
            "native-algorithm-generation",
            "native-algorithm-vc",
        ),
        (
            include_str!(
                "../../../../tests/miz/fail/algorithms/fail_proof_verification_algorithm_ensures_unprovable_001.miz"
            ),
            "pick",
            true,
            " \n世代\t\0",
            "",
        ),
        (
            "definition let a be object; algorithm plain(a) -> object do return a; end; end;",
            "empty",
            true,
            "native-algorithm-generation",
            "native-algorithm-vc",
        ),
    ] {
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
        let owner = publisher(snapshot, if open { 0 } else { usize::MAX }, &unit);
        allow_frontend(&owner, &unit);
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("VcGenerate"),
            OutputKind::new("SourceAlgorithmVC"),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
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
        let generation = GenerationSchemaVersion::new(generation_text);
        let schema = VcSchemaVersion::new(schema_text);
        let schemas_before = (generation.clone(), schema.clone());
        let core = inputs
            .native_algorithm_core_with_frontend_parent(&input, &owner)
            .unwrap()
            .unwrap();
        let graph = mizar_vc::generator::generate_source_algorithm_postconditions(
            &core,
            snapshot,
            &generation,
            &schema,
        )
        .unwrap();
        assert_eq!(
            inputs.native_algorithm_vcs_with_frontend_parent(&input, &owner, &generation, &schema,),
            Ok(Some(graph.clone()))
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
        let version = &inputs.snapshot.source_versions[0];
        let (_, loaded, captured_input) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        assert_eq!(loaded.text.as_ref(), text);
        assert_eq!(loaded.source_id, ast.source_id);
        assert_eq!(core.source_id(), loaded.source_id);
        assert_eq!(core.module_id(), &module);
        assert_eq!(graph.source(), loaded.source_id);
        assert_eq!(graph.snapshot(), snapshot);
        assert_eq!(graph.module().as_str(), "package=5:alpha;module=4:main");
        assert_eq!(graph.schema_version(), &schema);
        assert!(graph.vcs().iter().all(|vc| {
            vc.status == VcStatus::Open && vc.anchor.generation_schema_version == generation
        }));
        match obligation {
            "while" => assert!(
                graph
                    .vcs()
                    .iter()
                    .any(|vc| matches!(vc.kind, VcKind::LoopInvariant { .. }))
            ),
            "pick" => {
                assert!(
                    graph
                        .vcs()
                        .iter()
                        .any(|vc| vc.kind == VcKind::AlgorithmPostcondition)
                );
                assert!(
                    graph
                        .vcs()
                        .iter()
                        .any(|vc| vc.kind == VcKind::GeneratedNonEmptiness)
                );
            }
            "empty" => {
                assert!(graph.vcs().is_empty());
                assert!(matches!(
                    graph.seed_accounting(),
                    [mizar_vc::vc_ir::SeedAccounting {
                        seed_status: mizar_core::core_ir::ObligationSeedStatus::Deferred,
                        mapping: SeedVcMapping::NoConcreteVc { .. },
                        ..
                    }]
                ));
            }
            _ => unreachable!(),
        }
        let (arena, env) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
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
            mizar_checker::type_checker::check_source_algorithm_types(&arena, &neutral, &env)
                .unwrap();
        let core_semantic =
            mizar_core::elaborator::source_algorithm_core_semantic_payload(&check, &core).unwrap();
        let mut content = b"mizar-driver/native-algorithm-vc-content/v1\0".to_vec();
        for field in [
            generation.as_str().as_bytes(),
            schema.as_str().as_bytes(),
            core_semantic.as_slice(),
        ] {
            content.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            content.extend_from_slice(field);
        }
        let bytes =
            native_algorithm_vc_storage_bytes(ast, &module, &core, &graph, snapshot, &generation)
                .unwrap()
                .unwrap();
        assert_eq!(
            native_algorithm_vc_from_storage_bytes(&bytes, loaded.source_id, &module, snapshot,),
            Ok(Some(graph.clone()))
        );
        assert_ne!(content, bytes);
        let mut hasher = blake3::Hasher::new();
        for field in [
            b"mizar-ir/publisher-content-hash/v1".as_slice(),
            content.as_slice(),
        ] {
            hasher.update(&(field.len() as u64).to_le_bytes());
            hasher.update(field);
        }
        hasher.update(&1_u64.to_le_bytes());
        hasher.update(&("parent_content".len() as u64).to_le_bytes());
        hasher.update(b"parent_content");
        hasher.update(parent.content_hash().as_bytes());
        hasher.update(&1_u64.to_le_bytes());
        for field in [
            "source",
            SOURCE_UNIT_CACHE_KEY_VERSION,
            "mizar-ir/publisher-named-input-hash/v1",
        ] {
            hasher.update(&(field.len() as u64).to_le_bytes());
            hasher.update(field.as_bytes());
        }
        hasher.update(source_key(version).as_bytes());
        let content_hash = Hash::from_bytes(*hasher.finalize().as_bytes());
        let mut blob_hasher = blake3::Hasher::new();
        let domain = b"mizar-ir/content-blob/v1";
        blob_hasher.update(&(domain.len() as u64).to_le_bytes());
        blob_hasher.update(domain);
        blob_hasher.update(&1_u32.to_le_bytes());
        blob_hasher.update(&(bytes.len() as u64).to_le_bytes());
        blob_hasher.update(&bytes);
        let blob_hash = Hash::from_bytes(*blob_hasher.finalize().as_bytes());
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            captured_input.clone(),
            (*retained).clone(),
            input.clone(),
            parent.clone(),
        );
        let sealed = inputs
            .publish_native_algorithm_vcs(&input, &owner, &generation, &schema)
            .unwrap()
            .unwrap();
        for stage in 0..3 {
            if stage == 1 {
                fixture.write(b"open theorem Changed: 0 = 1;\n");
            } else if stage == 2 {
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            }
            assert_eq!(
                inputs.native_algorithm_vcs_with_frontend_parent(
                    &input,
                    &owner,
                    &generation,
                    &schema,
                ),
                Ok(Some(graph.clone()))
            );
            assert_eq!(
                native_algorithm_vc_storage_bytes(
                    ast,
                    &module,
                    &core,
                    &graph,
                    snapshot,
                    &generation,
                ),
                Ok(Some(bytes.clone()))
            );
            assert_eq!(
                native_algorithm_vc_from_storage_bytes(&bytes, loaded.source_id, &module, snapshot,),
                Ok(Some(graph.clone()))
            );
            assert_eq!(*owner.storage().get(&sealed).unwrap(), graph);
            assert_eq!(sealed.snapshot(), snapshot);
            assert_eq!(sealed.phase(), &IrPipelinePhase::new("VcGenerate"));
            assert_eq!(sealed.output_kind(), &OutputKind::new("SourceAlgorithmVC"));
            assert_eq!(sealed.work_unit(), &IrWorkUnit::new("\"alpha\":\"main\""));
            assert_eq!(
                sealed.schema_version(),
                mizar_ir::storage::SchemaVersion::new(1)
            );
            assert_eq!(sealed.lineage().parents, vec![parent.output()]);
            assert_eq!(
                sealed.lineage().named_input_hashes,
                vec![mizar_ir::identity::NamedInputHash {
                    name: "source".to_owned(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                    digest: source_key(version),
                }]
            );
            assert_eq!(sealed.content_hash(), content_hash);
            match sealed.placement() {
                StoragePlacement::Resident => assert!(!open),
                StoragePlacement::Blob { blob, len } => {
                    assert!(open);
                    assert_eq!(*len, bytes.len());
                    assert_eq!(blob.hash(), blob_hash);
                }
                _ => panic!("unexpected storage placement"),
            }
            assert_eq!(
                *owner.storage().side_tables(&sealed).unwrap(),
                mizar_ir::storage::IrSideTables {
                    source_maps: vec![mizar_ir::storage::SideTableRecord::new(
                        "generate-algorithm-vc-storage",
                        version.normalized_path.as_str(),
                        Hash::from_bytes(blake3::derive_key(
                            "mizar-driver/native-algorithm-vc-storage-side-table/v1",
                            &bytes,
                        )),
                    )],
                    ..mizar_ir::storage::IrSideTables::default()
                }
            );
            assert!(
                owner
                    .validate_published_output(snapshot, sealed.any())
                    .is_ok()
            );
            assert_eq!(
                owner
                    .validate_current_output(snapshot, sealed.any())
                    .is_ok(),
                !open
            );
            assert_eq!(
                inputs
                    .publish_native_algorithm_vcs(&input, &owner, &generation, &schema,)
                    .unwrap_err(),
                mizar_ir::publisher::PublishError::Storage {
                    error: Box::new(mizar_ir::storage::StorageError::OutputAlreadyStored {
                        output: sealed.output(),
                    }),
                }
                .to_string()
            );
        }
        assert_eq!((generation, schema), schemas_before);
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                captured_input.clone(),
                (*owner.storage().get(&typed).unwrap()).clone(),
                input,
                parent.clone()
            ),
            before,
        );
    }
}

#[test]
fn native_algorithm_vc_publication_preserves_refusals_and_publisher_boundaries() {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let generation = GenerationSchemaVersion::new(" \n\t");
    let schema = VcSchemaVersion::new("");
    for (text, case) in [
        (
            include_str!(
                "../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz"
            ),
            "none",
        ),
        (
            "definition let a be object; algorithm unsupported(a) -> object do return 0; end; end;",
            "unsupported",
        ),
        (
            "definition let a be object; algorithm twice(a) -> object do return a; return a; end; end;",
            "generator",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let snapshot = source_input.snapshot;
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        snapshot,
                        handle.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        let old_core = inputs.native_algorithm_core_with_frontend_parent(&input, owner);
        let old_raw =
            inputs.native_algorithm_vcs_with_frontend_parent(&input, owner, &generation, &schema);
        let expected = match case {
            "none" => {
                assert_eq!(old_core, Ok(None));
                assert_eq!(old_raw, Ok(None));
                Ok(None)
            }
            "unsupported" => {
                assert_eq!(
                    old_core.as_ref().unwrap_err(),
                    "algorithms.unsupported_source_types"
                );
                Err(old_raw.unwrap_err())
            }
            "generator" => {
                let core = old_core.unwrap().unwrap();
                let direct = mizar_vc::generator::generate_source_algorithm_postconditions(
                    &core,
                    snapshot,
                    &generation,
                    &schema,
                )
                .unwrap_err();
                assert_eq!(direct, "algorithms.postcondition.unsupported_core");
                assert_eq!(old_raw, Err(direct.clone()));
                Err(direct)
            }
            _ => unreachable!(),
        };
        assert_eq!(
            inputs
                .publish_native_algorithm_vcs(&input, owner, &generation, &schema,)
                .map(|value| value.map(|output| (*owner.storage().get(&output).unwrap()).clone())),
            expected
        );
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.outputs_dropped, 2);
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
    let fixture = Fixture::new(
        b"definition let a be object; algorithm plain(a) -> object do return a; end; end;",
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, submission, adapter, handle, _) =
        native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
    let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
    let snapshot = source_input.snapshot;
    let input = PhaseInput::new(
        source_input.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            snapshot,
            source_input.identities().input_hash(),
            vec![],
            vec![
                SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    let core = inputs
        .native_algorithm_core_with_frontend_parent(&input, owner)
        .unwrap()
        .unwrap();
    let huge_schema = VcSchemaVersion::new("x".repeat(32 * 1024 * 1024));
    let huge_raw = inputs
        .native_algorithm_vcs_with_frontend_parent(&input, owner, &generation, &huge_schema)
        .unwrap()
        .unwrap();
    assert_eq!(
        huge_raw,
        mizar_vc::generator::generate_source_algorithm_postconditions(
            &core,
            snapshot,
            &generation,
            &huge_schema,
        )
        .unwrap()
    );
    assert!(huge_raw.vcs().is_empty());
    drop(huge_raw);
    assert!(
        inputs
            .publish_native_algorithm_vcs(&input, owner, &generation, &huge_schema,)
            .unwrap()
            .is_none()
    );
    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
        snapshot,
        protected_outputs: vec![],
    });
    assert_eq!(collected.outputs_dropped, 2);
    assert_eq!(collected.abandoned_slots_dropped, 0);
    for case in ["obsolete", "collected", "denied-kind"] {
        let fixture = Fixture::new(include_bytes!(
            "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz"
        ));
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let snapshot = source_input.snapshot;
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("VcGenerate"),
            OutputKind::new(if case == "denied-kind" {
                "WrongKind"
            } else {
                "SourceAlgorithmVC"
            }),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
        let parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                .unwrap(),
        ];
        match case {
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                let dropped = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(dropped.outputs_dropped, 2);
            }
            _ => {}
        }
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                parents,
            )
            .unwrap(),
        );
        let old_raw =
            inputs.native_algorithm_vcs_with_frontend_parent(&input, owner, &generation, &schema);
        let actual = inputs
            .publish_native_algorithm_vcs(&input, owner, &generation, &schema)
            .unwrap_err();
        if case == "denied-kind" {
            assert!(old_raw.unwrap().is_some());
            assert_eq!(
                actual,
                mizar_ir::publisher::PublishError::WorkUnitNotAllowed {
                    phase: IrPipelinePhase::new("VcGenerate"),
                    work_unit: IrWorkUnit::new("\"alpha\":\"main\""),
                    output_kind: OutputKind::new("SourceAlgorithmVC"),
                }
                .to_string()
            );
        } else {
            assert_eq!(actual, old_raw.unwrap_err(), "{case}");
        }
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            if case == "collected" { 0 } else { 2 }
        );
        assert_eq!(
            collected.abandoned_slots_dropped,
            usize::from(case == "denied-kind")
        );
    }
}

#[test]
fn native_algorithm_module_summary_candidate_preserves_captured_projection_and_refusals() {
    use mizar_build::{module_index::ModuleId as BuildModuleId, planner::PackagePlanSource};
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    let fixture_text = include_str!(
        "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut original = None;
    let cases = [
        fixture_text.to_owned(),
        fixture_text.replace("x := c;", "x := a;"),
        format!(":: λ relocation\n{fixture_text}"),
        fixture_text.replace("terminating algorithm", "private terminating algorithm"),
        String::new(),
        fixture_text.replace("terminating ", ""),
        fixture_text.replace("terminating algorithm", "public terminating algorithm"),
        fixture_text.replace("-> object", "-> set"),
        include_str!(
            "../../../../tests/miz/fail/parser/fail_parser_algorithm_verification_recovery_001.miz"
        )
        .to_owned(),
        format!("import std.core;\n{fixture_text}"),
        format!(
            "{fixture_text}\ndefinition\n let X, Y be set;\n private pred Hidden: X secret Y means X = Y;\nend;\n"
        ),
    ];
    for (case, text) in cases.into_iter().enumerate() {
        for open in [false, true] {
            let fixture = Fixture::new(text.as_bytes());
            let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
            draft.source_inputs.versions.clear();
            let descriptor = SourceInput {
                package_id: PackageId::new("alpha"),
                module_path: ModulePath::new("main"),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: Edition::new("2025"),
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
            let mut registry = PhaseRegistryBuilder::new();
            registry.register_source_load();
            let submission = CompilerDriver::new(registry.build().unwrap())
                .submit_captured(session, &ids, &snapshots, submit)
                .unwrap();
            let (inputs, _) = native_disk_lookup_context(&submission, &ids);
            let module = BuildModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let (_, loaded, captured_input) =
                &inputs.captured.unwrap().native_sources().unwrap()[0];
            let output = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(source_unit_from_loaded(
                loaded.clone(),
                diagnostic_file_path(captured_input),
            ))
            .unwrap();
            assert_eq!(output.cache_keys.ast.is_some(), output.ast.is_some());
            assert_eq!(output.source.source_id, loaded.source_id);
            let ast = output.ast.as_ref().unwrap();
            let resolver_module =
                ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
            let collection =
                SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main"))
                    .collect(&resolver_module);
            let identity = ModuleSummaryIdentity {
                package_id: "alpha".into(),
                package_version: Some("0.1.0".into()),
                lockfile_identity: None,
                module_path: "main".into(),
                language_edition: "2025".into(),
            };
            assert_eq!(inputs.build_plan.packages[0].package_id, module.package);
            assert_eq!(inputs.build_plan.packages[0].version.to_string(), "0.1.0");
            assert!(matches!(
                inputs.build_plan.packages[0].source,
                PackagePlanSource::Workspace { .. }
            ));
            let version = &inputs.snapshot.source_versions[0];
            assert_eq!(version.edition.as_str(), identity.language_edition);
            assert_eq!(loaded.source_id, version.source_id);
            assert_eq!(loaded.source_hash, version.source_hash);
            assert_eq!(
                loaded.source_hash,
                mizar_session::hash_text(&output.source.source_text)
            );
            assert_eq!(output.source.source_text.as_ref(), text);
            assert_eq!(loaded.origin, version.origin);
            assert_eq!(
                matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
                open
            );
            let expected = collection.source_algorithm_module_summary_candidate(&output, &identity);
            if case <= 6 {
                assert!(output.diagnostics.is_empty() && output.cache_keys.ast.is_some());
                let summary = expected.as_ref().unwrap().as_ref().unwrap();
                assert_eq!(summary.module, identity);
                assert_eq!(summary.source_hash, loaded.source_hash);
                assert_eq!(
                    summary.exported_symbols.len(),
                    usize::from(!matches!(case, 3 | 4))
                );
                assert!(
                    summary
                        .exported_symbols
                        .iter()
                        .all(|row| row.proof_status.is_none())
                );
                assert!(summary.exported_labels.is_empty());
                assert!(summary.lexical_summary.contributions.is_empty());
                assert!(summary.reexports.is_empty() && summary.dependency_interfaces.is_empty());
            } else {
                assert_eq!(expected, Ok(None));
                if matches!(case, 8 | 9) {
                    assert!(!output.diagnostics.is_empty());
                } else {
                    assert!(output.diagnostics.is_empty() && output.cache_keys.ast.is_some());
                }
                if case == 10 {
                    assert!(!output.tokens.local_declarations().user_symbols.is_empty());
                    assert_eq!(
                        collection.export_frontend_lexical_contributions(&output, &identity),
                        Some(vec![])
                    );
                }
            }
            // The old theorem method calibrates shared native binding, not algorithm eligibility.
            if case == 0 {
                assert_eq!(
                    inputs.native_theorem_module_summary_candidate(&module),
                    Ok(None)
                );
            }
            let before = (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                captured_input.clone(),
                output.clone(),
                collection.clone(),
                identity.clone(),
            );
            for stage in 0..3 {
                if stage == 1 {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                }
                if stage == 2 {
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                }
                let replayed = Frontend::new(
                    FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                    inputs.dependency_lexical_provider(&[]),
                    MizarParserSeam,
                )
                .run_loaded(source_unit_from_loaded(
                    loaded.clone(),
                    diagnostic_file_path(captured_input),
                ))
                .unwrap();
                for _ in 0..2 {
                    assert_eq!(
                        inputs.native_algorithm_module_summary_candidate(&module),
                        expected,
                        "case={case}, open={open}, stage={stage}"
                    );
                }
                assert_eq!(replayed, output);
            }
            if case == 0 && !open {
                for requested in [
                    BuildModuleId::new(PackageId::new("alpha"), ModulePath::new("missing")),
                    BuildModuleId::new(PackageId::new("foreign"), ModulePath::new("main")),
                ] {
                    assert_eq!(
                        inputs
                            .native_algorithm_module_summary_candidate(&requested)
                            .unwrap_err(),
                        "native algorithm summary requires one captured source version"
                    );
                    assert_eq!(
                        inputs
                            .native_theorem_module_summary_candidate(&requested)
                            .unwrap_err(),
                        "native theorem summary requires one captured source version"
                    );
                }
                let mut absent = inputs.snapshot.clone();
                absent.source_versions.clear();
                let mut ambiguous = inputs.snapshot.clone();
                ambiguous
                    .source_versions
                    .push(ambiguous.source_versions[0].clone());
                for snapshot in [&absent, &ambiguous] {
                    assert_eq!(
                        SourceLoadInputs {
                            snapshot,
                            captured: None,
                            ..inputs
                        }
                        .native_algorithm_module_summary_candidate(&module)
                        .unwrap_err(),
                        "native algorithm summary requires one captured source version"
                    );
                    assert_eq!(
                        SourceLoadInputs {
                            snapshot,
                            captured: None,
                            ..inputs
                        }
                        .native_theorem_module_summary_candidate(&module)
                        .unwrap_err(),
                        "native theorem summary requires one captured source version"
                    );
                }
                let mut ambiguous_plan = inputs.build_plan.clone();
                ambiguous_plan
                    .packages
                    .push(ambiguous_plan.packages[0].clone());
                let mut wrong_edition = inputs.build_plan.clone();
                wrong_edition.packages[0].edition = Edition::new("1900");
                let mut foreign_index = inputs.module_index.clone();
                foreign_index.modules[0].module.package = PackageId::new("foreign");
                for invalid in [
                    SourceLoadInputs {
                        captured: None,
                        ..inputs
                    },
                    SourceLoadInputs {
                        build_plan: &ambiguous_plan,
                        ..inputs
                    },
                    SourceLoadInputs {
                        build_plan: &wrong_edition,
                        ..inputs
                    },
                    SourceLoadInputs {
                        module_index: &foreign_index,
                        ..inputs
                    },
                ] {
                    assert_eq!(
                        invalid
                            .native_algorithm_module_summary_candidate(&module)
                            .unwrap_err(),
                        "native algorithm summary capture binding failed"
                    );
                    assert_eq!(
                        invalid
                            .native_theorem_module_summary_candidate(&module)
                            .unwrap_err(),
                        "native theorem summary capture binding failed"
                    );
                }
            }
            if case == 0 && open {
                let (_, foreign_submission) = original.as_ref().unwrap();
                let (foreign, _) = native_disk_lookup_context(foreign_submission, &ids);
                for invalid in [
                    SourceLoadInputs {
                        captured: foreign.captured,
                        ..inputs
                    },
                    SourceLoadInputs {
                        captured: inputs.captured,
                        ..foreign
                    },
                ] {
                    assert_eq!(
                        invalid
                            .native_algorithm_module_summary_candidate(&module)
                            .unwrap_err(),
                        "native algorithm summary capture binding failed"
                    );
                    assert_eq!(
                        invalid
                            .native_theorem_module_summary_candidate(&module)
                            .unwrap_err(),
                        "native theorem summary capture binding failed"
                    );
                }
            }
            assert_eq!(
                (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    output,
                    collection,
                    identity,
                ),
                before
            );
            assert_eq!(captured_input, &descriptor);
            if case == 0 && !open {
                original = Some((fixture, submission));
            }
        }
    }
}

#[test]
fn native_algorithm_summary_frontend_caller_preserves_genuine_results_and_refusals() {
    use mizar_build::{module_index::ModuleId as BuildModuleId, planner::PackagePlanSource};
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_ir::storage::{CollectInput, RetainOwner};
    let text = include_str!(
        "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    for (case, text, open, threshold) in [
        ("disk", text.to_owned(), false, usize::MAX),
        ("buffer", text.to_owned(), true, 0),
        (
            "private",
            text.replace("terminating algorithm", "private terminating algorithm"),
            false,
            usize::MAX,
        ),
        ("empty", String::new(), false, usize::MAX),
        ("unsupported", text.replace("-> object", "-> set"), false, 0),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("main"),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: Edition::new("2025"),
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
        let snapshot = session.captured().snapshot().id;
        let module = BuildModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        let unit = WorkUnit::Module {
            module: module.clone(),
        };
        let owner = publisher(snapshot, threshold, &unit);
        allow_frontend(&owner, &unit);
        let mut registry = PhaseRegistryBuilder::new();
        registry.register_source_load();
        registry.register_frontend(vec![]);
        let submission = CompilerDriver::new(registry.build().unwrap())
            .with_output_publisher(owner.clone())
            .submit_captured(session, &ids, &snapshots, submit)
            .unwrap();
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let handles: Vec<_> = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .cloned()
            .collect();
        let source_parent = handles
            .iter()
            .find(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
            .unwrap();
        let parent = handles
            .iter()
            .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        assert_eq!(handles.len(), 2);
        assert_eq!(
            parent.placement() == &StoragePlacement::Resident,
            threshold != 0
        );
        owner.validate_published_output(snapshot, parent).unwrap();
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
        assert_eq!(input.parent_outputs()[0].as_output_ref(), parent);
        assert_eq!(parent.lineage().parents, vec![source_parent.output()]);
        let typed = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let output = (*owner.storage().get(&typed).unwrap()).clone();
        let (_, loaded, captured_input) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let version = &inputs.snapshot.source_versions[0];
        assert_eq!(loaded.source_id, version.source_id);
        assert_eq!(loaded.source_hash, version.source_hash);
        assert_eq!(loaded.source_hash, mizar_session::hash_text(&text));
        assert_eq!(loaded.origin, version.origin);
        assert_eq!(
            matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
            open
        );
        assert_eq!(captured_input, &descriptor);
        assert_eq!(output.source.source_text.as_ref(), text);
        assert!(output.diagnostics.is_empty() && output.cache_keys.ast.is_some());
        assert_eq!(output.source.source_id, loaded.source_id);
        let expected = inputs.native_algorithm_module_summary_candidate(&module);
        let projected = expected.as_ref().unwrap();
        if case == "unsupported" {
            assert_eq!(expected, Ok(None));
        } else {
            let summary = projected.as_ref().unwrap();
            assert_eq!(
                summary.module,
                ModuleSummaryIdentity {
                    package_id: "alpha".into(),
                    package_version: Some("0.1.0".into()),
                    lockfile_identity: None,
                    module_path: "main".into(),
                    language_edition: "2025".into(),
                }
            );
            assert_eq!(summary.source_hash, loaded.source_hash);
            assert_eq!(
                summary.exported_symbols.len(),
                usize::from(matches!(case, "disk" | "buffer"))
            );
        }
        assert_eq!(inputs.build_plan.packages[0].package_id, module.package);
        assert_eq!(inputs.build_plan.packages[0].version.to_string(), "0.1.0");
        assert!(matches!(
            inputs.build_plan.packages[0].source,
            PackagePlanSource::Workspace { .. }
        ));
        assert_eq!(version.edition.as_str(), "2025");
        // Calibrate only common association; theorem None is not the summary oracle.
        if case == "disk" {
            assert_eq!(
                inputs.native_theorem_module_candidates_with_frontend_parent(&input, &owner),
                Ok(None)
            );
        }
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            captured_input.clone(),
            input.clone(),
            parent.clone(),
            output.clone(),
            expected.clone(),
        );
        for stage in 0..3 {
            if stage == 1 {
                fixture.write(b"open theorem Changed: 0 = 1;\n");
            }
            if stage == 2 {
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            }
            let replayed = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(source_unit_from_loaded(
                loaded.clone(),
                diagnostic_file_path(captured_input),
            ))
            .unwrap();
            assert_eq!(replayed, output, "{case}, stage={stage}");
            assert_eq!(
                inputs.native_algorithm_module_summary_candidate(&module),
                expected.clone()
            );
            for _ in 0..2 {
                assert_eq!(
                    inputs.native_algorithm_module_summary_with_frontend_parent(&input, &owner),
                    expected,
                    "{case}, stage={stage}"
                );
            }
            assert_eq!(*owner.storage().get(&typed).unwrap(), output);
            owner.validate_published_output(snapshot, parent).unwrap();
            let collected = owner.storage().collect(CollectInput {
                snapshot,
                protected_outputs: vec![source_parent.output(), parent.output()],
            });
            assert_eq!(
                (
                    collected.outputs_dropped,
                    collected.abandoned_slots_dropped,
                    collected.protected_outputs
                ),
                (0, 0, 2)
            );
        }
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                captured_input.clone(),
                input,
                parent.clone(),
                (*owner.storage().get(&typed).unwrap()).clone(),
                expected,
            ),
            before
        );
    }
    let fixture = Fixture::new(text.as_bytes());
    let foreign = Fixture::new(text.as_bytes());
    let (_foreign_driver, foreign_submission, foreign_adapter, foreign_handle, _) =
        native_disk_frontend_record_submission(&foreign, &ids, &snapshots, 0);
    for case in [
        "absent",
        "source-parent",
        "foreign",
        "hash",
        "dependencies",
        "capture",
        "complete-mismatch",
        "collected",
        "obsolete",
    ] {
        let (_driver, submission, adapter, handle, frontend_input) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let mut selected_inputs = inputs;
        let snapshot = source_input.snapshot;
        let original = (*owner.storage().get(&handle).unwrap()).clone();
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                .unwrap(),
        ];
        let mut input_snapshot = snapshot;
        let mut hash = source_input.identities().input_hash();
        let mut dependencies = vec![];
        let alternative;
        let selected_owner = if case == "complete-mismatch" {
            alternative = std::sync::Arc::new(PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            alternative.register_current_snapshot(snapshot);
            let ancestor = frontend_input.parent_outputs()[0].as_output_ref().lineage();
            alternative
                .registry()
                .register_output(mizar_ir::identity::OutputIdentityInput {
                    snapshot: ancestor.snapshot,
                    phase: ancestor.phase.clone(),
                    work_unit: ancestor.work_unit.clone(),
                    output_kind: ancestor.output_kind.clone(),
                    content_hash: ancestor.content_hash,
                    side_table_hash: ancestor.side_table_hash,
                    parents: ancestor.parents.clone(),
                    named_input_hashes: ancestor.named_input_hashes.clone(),
                })
                .unwrap();
            &alternative
        } else {
            owner
        };
        match case {
            "absent" => parents.clear(),
            "source-parent" => parents = frontend_input.parent_outputs().to_vec(),
            "foreign" => {
                input_snapshot = foreign_submission.session.captured().snapshot().id;
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        foreign_adapter.publisher(),
                        input_snapshot,
                        foreign_handle.erase(),
                    )
                    .unwrap(),
                ];
            }
            "hash" => hash = Hash::from_bytes([0x62; 32]),
            "dependencies" => dependencies.push(hash),
            "capture" => selected_inputs.captured = None,
            "complete-mismatch" => {
                // Negative-only derivative of a genuine full Frontend, never a positive producer.
                let mut negative = original.clone();
                negative.cache_keys.ast.as_mut().unwrap().parser_inputs_hash =
                    Hash::from_bytes([0x63; 32]);
                let mut restored = negative.clone();
                restored.cache_keys.ast = original.cache_keys.ast.clone();
                assert_eq!(restored, original);
                assert_ne!(negative.cache_keys.ast, original.cache_keys.ast);
                let raw = negative.canonical_disk_bytes().unwrap();
                let descriptor = inputs.captured.unwrap().native_sources().unwrap()[0]
                    .2
                    .clone();
                let source_id = negative.source.source_id;
                assert_eq!(
                    FrontendOutput::from_canonical_storage_bytes(&raw, source_id, &descriptor)
                        .unwrap(),
                    negative
                );
                selected_owner.allow_work_unit(AllowedWorkUnit::new(
                    handle.phase().clone(),
                    handle.output_kind().clone(),
                    handle.work_unit().clone(),
                ));
                let negative = selected_owner
                    .publish(mizar_ir::publisher::PublishOutputInput {
                        slot: selected_owner.allocate(
                            snapshot,
                            handle.phase().clone(),
                            handle.work_unit().clone(),
                            handle.output_kind().clone(),
                            handle.schema_version(),
                        ),
                        snapshot,
                        phase: handle.phase().clone(),
                        work_unit: handle.work_unit().clone(),
                        output_kind: handle.output_kind().clone(),
                        schema_version: handle.schema_version(),
                        payload: negative,
                        canonical_payload: Some(raw.clone()),
                        storage_payload: Some(raw),
                        decode: mizar_ir::storage::BlobDecoder::new(move |bytes| {
                            FrontendOutput::from_canonical_storage_bytes(
                                bytes,
                                source_id,
                                &descriptor,
                            )
                            .ok_or_else(|| {
                                mizar_ir::storage::BlobDecodeError::new("negative aggregate")
                            })
                        }),
                        parents: vec![frontend_input.parent_outputs()[0].as_output_ref().clone()],
                        named_input_hashes: handle.lineage().named_input_hashes.clone(),
                        side_tables: (*owner.storage().side_tables(&handle).unwrap()).clone(),
                        origin: mizar_ir::publisher::OutputOrigin::PackageSource,
                        target: mizar_ir::publisher::PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        selected_owner,
                        snapshot,
                        negative.erase(),
                    )
                    .unwrap(),
                ];
            }
            "collected" | "obsolete" => {}
            _ => unreachable!(),
        }
        // Seal the valid bundle before the real storage/publication invalidation.
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(input_snapshot, hash, dependencies, parents).unwrap(),
        );
        let retain = RetainOwner::new("driver115-obsolete-observer");
        if case == "obsolete" {
            owner
                .storage()
                .retain(handle.output(), retain.clone())
                .unwrap();
            owner.mark_obsolete(snapshot).unwrap();
            let collected = owner.storage().collect(CollectInput {
                snapshot,
                protected_outputs: vec![
                    frontend_input.parent_outputs()[0].as_output_ref().output(),
                ],
            });
            assert_eq!(
                (collected.outputs_dropped, collected.retained_outputs),
                (0, 1)
            );
            assert_eq!(*owner.storage().get(&handle).unwrap(), original);
        }
        if case == "collected" {
            assert_eq!(
                owner
                    .storage()
                    .collect(CollectInput {
                        snapshot,
                        protected_outputs: vec![]
                    })
                    .outputs_dropped,
                2
            );
            assert!(owner.storage().get(&handle).is_err());
        }
        let WorkUnit::Module { module } = &input.work_unit else {
            unreachable!()
        };
        let expected = selected_inputs.native_algorithm_module_summary_candidate(module);
        if case == "capture" {
            assert_eq!(
                expected.unwrap_err(),
                "native algorithm summary capture binding failed"
            );
        } else {
            assert!(expected.unwrap().is_some());
        }
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            descriptor.clone(),
            input.clone(),
            handle.clone(),
            original.clone(),
        );
        let association_error = selected_inputs
            .native_theorem_module_candidates_with_frontend_parent(&input, selected_owner)
            .unwrap_err();
        match case {
            "foreign" => assert_eq!(
                association_error,
                "native Frontend resolver snapshot mismatch"
            ),
            "capture" => assert_eq!(
                association_error,
                "native Frontend resolver capture binding failed"
            ),
            "source-parent" => assert_eq!(
                association_error,
                "native Frontend resolver parent binding mismatch"
            ),
            "absent" | "hash" | "dependencies" => assert_eq!(
                association_error,
                "native Frontend resolver requires the leaf input profile"
            ),
            "complete-mismatch" => assert_eq!(
                association_error,
                "native Frontend resolver complete output mismatch"
            ),
            "collected" => assert_eq!(
                association_error,
                owner
                    .storage()
                    .validate_handle(&handle.erase())
                    .unwrap_err()
                    .to_string()
            ),
            "obsolete" => assert_eq!(
                association_error,
                owner
                    .validate_published_output(snapshot, &handle.erase())
                    .unwrap_err()
                    .to_string()
            ),
            _ => unreachable!(),
        }
        let expected_error = association_error
            .strip_prefix("native Frontend resolver ")
            .map_or_else(
                || association_error.clone(),
                |suffix| format!("native Frontend algorithm summary {suffix}"),
            );
        assert_eq!(
            selected_inputs
                .native_algorithm_module_summary_with_frontend_parent(&input, selected_owner),
            Err(expected_error),
            "{case}"
        );
        if case != "collected" {
            assert_eq!(*owner.storage().get(&handle).unwrap(), original);
        }
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                descriptor.clone(),
                input,
                handle.clone(),
                original,
            ),
            before
        );
        if case == "obsolete" {
            assert!(owner.storage().release(handle.output(), &retain).unwrap());
            assert_eq!(
                owner
                    .storage()
                    .collect(CollectInput {
                        snapshot,
                        protected_outputs: vec![
                            frontend_input.parent_outputs()[0].as_output_ref().output()
                        ],
                    })
                    .outputs_dropped,
                1
            );
        }
    }
}

#[test]
fn native_algorithm_summary_publication_preserves_real_bytes_and_refusals() {
    use mizar_artifact::{
        module_summary::{ModuleSummaryReadOptions, read_module_summary, write_module_summary},
        store::parse_canonical_json_bytes,
    };
    use mizar_build::module_index::ModuleId as BuildModuleId;
    use mizar_ir::{
        identity::NamedInputHash,
        publisher::PublishError,
        storage::{CollectInput, RetainOwner, SchemaVersion},
    };
    let text = include_str!(
        "../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let phase = IrPipelinePhase::new("SignatureCollection");
    let kind = OutputKind::new("SourceAlgorithmModuleSummary");
    let schema = SchemaVersion::new(1);
    let mut displayed_baseline = None;
    for (case, text, open, threshold) in [
        ("disk", text.to_owned(), false, usize::MAX),
        ("buffer", text.to_owned(), true, 0),
        (
            "private",
            text.replace("terminating algorithm", "private terminating algorithm"),
            false,
            usize::MAX,
        ),
        (
            "body",
            text.replace("x := c;", "x := a;"),
            false,
            usize::MAX,
        ),
        ("trivia", format!(":: λ relocation\n{text}"), false, 0),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("main"),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: Edition::new("2025"),
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
        let snapshot = session.captured().snapshot().id;
        let module = BuildModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        let unit = WorkUnit::Module {
            module: module.clone(),
        };
        let owner = publisher(snapshot, threshold, &unit);
        allow_frontend(&owner, &unit);
        let mut registry = PhaseRegistryBuilder::new();
        registry.register_source_load();
        registry.register_frontend(vec![]);
        let submission = CompilerDriver::new(registry.build().unwrap())
            .with_output_publisher(owner.clone())
            .submit_captured(session, &ids, &snapshots, submit)
            .unwrap();
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let handles: Vec<_> = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .cloned()
            .collect();
        assert_eq!(handles.len(), 2);
        let parent = handles
            .iter()
            .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        let source_parent = handles
            .iter()
            .find(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
            .unwrap();
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
        assert_eq!(input.parent_outputs()[0].as_output_ref(), parent);
        assert_eq!(parent.lineage().parents, vec![source_parent.output()]);
        assert_eq!(parent.phase(), &IrPipelinePhase::new("Frontend"));
        assert_eq!(parent.schema_version(), schema);
        assert_eq!(parent.work_unit(), &IrWorkUnit::new("\"alpha\":\"main\""));
        assert_eq!(
            parent.placement() == &StoragePlacement::Resident,
            threshold != 0
        );
        let typed = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let original = (*owner.storage().get(&typed).unwrap()).clone();
        let (_, loaded, captured_input) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let version = &inputs.snapshot.source_versions[0];
        assert_eq!(captured_input, &descriptor);
        assert_eq!(version.edition.as_str(), "2025");
        assert_eq!(loaded.source_id, version.source_id);
        assert_eq!(loaded.source_hash, version.source_hash);
        assert_eq!(loaded.source_hash, mizar_session::hash_text(&text));
        assert_eq!(loaded.origin, version.origin);
        assert_eq!(
            matches!(version.origin, SourceOrigin::OpenBuffer { .. }),
            open
        );
        assert_eq!(original.source.source_text.as_ref(), text);
        assert_eq!(original.source.source_id, loaded.source_id);
        assert!(original.diagnostics.is_empty() && original.cache_keys.ast.is_some());
        let expected = inputs
            .native_algorithm_module_summary_with_frontend_parent(&input, &owner)
            .unwrap()
            .unwrap();
        assert_eq!(
            expected.module,
            ModuleSummaryIdentity {
                package_id: "alpha".into(),
                package_version: Some("0.1.0".into()),
                lockfile_identity: None,
                module_path: "main".into(),
                language_edition: "2025".into(),
            }
        );
        assert_eq!(inputs.build_plan.packages[0].version.to_string(), "0.1.0");
        assert_eq!(expected.source_hash, version.source_hash);
        assert_eq!(
            expected.exported_symbols.len(),
            usize::from(case != "private")
        );
        assert!(
            expected
                .exported_symbols
                .iter()
                .all(|row| row.proof_status.is_none())
        );
        let bytes = write_module_summary(&expected).unwrap();
        let parsed = parse_canonical_json_bytes(&bytes, "algorithm summary baseline").unwrap();
        assert_eq!(
            read_module_summary(
                &parsed,
                ModuleSummaryReadOptions {
                    artifact_path: Some("algorithm summary baseline"),
                    expected_module: Some(&expected.module),
                    expected_interface_hash: Some(expected.interface_hash),
                }
            )
            .unwrap(),
            expected
        );
        if case == "disk" {
            displayed_baseline =
                Some((expected.interface_hash, expected.source_hash, bytes.clone()));
        } else if matches!(case, "body" | "trivia") {
            let (interface, source_hash, original_bytes) = displayed_baseline.as_ref().unwrap();
            assert_eq!(&expected.interface_hash, interface);
            assert_ne!(&expected.source_hash, source_hash);
            assert_ne!(&bytes, original_bytes);
        }
        let source_named = vec![NamedInputHash {
            name: "source".into(),
            domain: SOURCE_UNIT_CACHE_KEY_VERSION.into(),
            digest: source_key(version),
        }];
        assert_eq!(source_input.identities().input_hash(), source_key(version));
        assert_eq!(source_parent.lineage().named_input_hashes, source_named);
        let mut content = blake3::Hasher::new();
        for field in [
            b"mizar-ir/publisher-content-hash/v1".as_slice(),
            bytes.as_slice(),
        ] {
            content.update(&(field.len() as u64).to_le_bytes());
            content.update(field);
        }
        content.update(&1_u64.to_le_bytes());
        content.update(&("parent_content".len() as u64).to_le_bytes());
        content.update(b"parent_content");
        content.update(parent.content_hash().as_bytes());
        content.update(&1_u64.to_le_bytes());
        for field in [
            "source",
            SOURCE_UNIT_CACHE_KEY_VERSION,
            "mizar-ir/publisher-named-input-hash/v1",
        ] {
            content.update(&(field.len() as u64).to_le_bytes());
            content.update(field.as_bytes());
        }
        content.update(source_key(version).as_bytes());
        let expected_content = Hash::from_bytes(*content.finalize().as_bytes());
        let mut blob = blake3::Hasher::new();
        let domain = b"mizar-ir/content-blob/v1";
        blob.update(&(domain.len() as u64).to_le_bytes());
        blob.update(domain);
        blob.update(&1_u32.to_le_bytes());
        blob.update(&(bytes.len() as u64).to_le_bytes());
        blob.update(&bytes);
        let expected_blob = Hash::from_bytes(*blob.finalize().as_bytes());
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            captured_input.clone(),
            input.clone(),
            parent.clone(),
            original.clone(),
            expected.clone(),
            bytes.clone(),
            expected_content,
            expected_blob,
        );
        owner.allow_work_unit(AllowedWorkUnit::new(
            phase.clone(),
            kind.clone(),
            parent.work_unit().clone(),
        ));
        owner.validate_published_output(snapshot, parent).unwrap();
        assert_eq!(
            owner.validate_current_output(snapshot, parent).is_ok(),
            !open
        );
        fixture.write(b"open theorem Changed: 0 = 1;\n");
        assert_eq!(
            inputs.native_algorithm_module_summary_with_frontend_parent(&input, &owner),
            Ok(Some(expected.clone()))
        );
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        assert!(fixture.root.join("alpha").is_dir());
        assert_eq!(
            inputs.native_algorithm_module_summary_with_frontend_parent(&input, &owner),
            Ok(Some(expected.clone()))
        );
        let sealed = inputs
            .publish_native_algorithm_module_summary(&input, &owner)
            .unwrap()
            .unwrap();
        let actual = owner.storage().get(&sealed).unwrap();
        assert_eq!(*actual, expected);
        assert_eq!(write_module_summary(&actual).unwrap(), bytes);
        assert_eq!(sealed.snapshot(), snapshot);
        assert_eq!(sealed.phase(), &phase);
        assert_eq!(sealed.output_kind(), &kind);
        assert_eq!(sealed.schema_version(), schema);
        assert_eq!(sealed.work_unit(), parent.work_unit());
        assert_eq!(sealed.lineage().parents, vec![parent.output()]);
        assert_eq!(sealed.lineage().named_input_hashes, source_named);
        assert_eq!(sealed.content_hash(), expected_content);
        assert_eq!(
            *owner.storage().side_tables(&sealed).unwrap(),
            mizar_ir::storage::IrSideTables::default()
        );
        match sealed.placement() {
            StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
            StoragePlacement::Blob { blob, len } => {
                assert_eq!(threshold, 0);
                assert_eq!(*len, bytes.len());
                assert_eq!(blob.hash(), expected_blob);
            }
            _ => panic!("unexpected summary storage placement"),
        }
        owner
            .validate_published_output(snapshot, sealed.any())
            .unwrap();
        assert_eq!(
            owner
                .validate_current_output(snapshot, sealed.any())
                .is_ok(),
            !open
        );
        assert_eq!(*owner.storage().get(&typed).unwrap(), original);
        let protected_outputs: Vec<_> = handles.iter().map(|handle| handle.output()).collect();
        let protected_outputs = {
            let mut outputs = protected_outputs;
            outputs.push(sealed.output());
            assert_eq!(outputs.len(), 3);
            outputs
        };
        let protected_count = protected_outputs.len();
        let collected = owner.storage().collect(CollectInput {
            snapshot,
            protected_outputs,
        });
        assert_eq!(
            (
                collected.outputs_dropped,
                collected.abandoned_slots_dropped,
                collected.protected_outputs
            ),
            (0, 0, protected_count)
        );
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                captured_input.clone(),
                input,
                parent.clone(),
                original,
                expected,
                bytes,
                expected_content,
                expected_blob,
            ),
            before
        );
    }
    for case in [
        "unsupported",
        "capture",
        "obsolete",
        "collected",
        "denied-phase",
        "denied-kind",
    ] {
        let fixture = Fixture::new(
            if case == "unsupported" {
                text.replace("-> object", "-> set").into_bytes()
            } else {
                text.as_bytes().to_vec()
            }
            .as_slice(),
        );
        let (_driver, submission, adapter, handle, frontend_input) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let snapshot = source_input.snapshot;
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        snapshot,
                        handle.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        assert_eq!(input.parent_outputs()[0].as_output_ref(), &handle.erase());
        let original = (*owner.storage().get(&handle).unwrap()).clone();
        let baseline = inputs.native_algorithm_module_summary_with_frontend_parent(&input, owner);
        if case == "unsupported" {
            assert_eq!(baseline, Ok(None));
        } else {
            assert!(baseline.as_ref().unwrap().is_some());
        }
        let selected_inputs = SourceLoadInputs {
            captured: if case == "capture" {
                None
            } else {
                inputs.captured
            },
            ..inputs
        };
        if !matches!(case, "unsupported" | "capture") {
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new(if case == "denied-phase" {
                    "WrongPhase"
                } else {
                    "SignatureCollection"
                }),
                OutputKind::new(if case == "denied-kind" {
                    "WrongKind"
                } else {
                    "SourceAlgorithmModuleSummary"
                }),
                handle.work_unit().clone(),
            ));
        }
        let expected_permission = PublishError::WorkUnitNotAllowed {
            phase: phase.clone(),
            work_unit: handle.work_unit().clone(),
            output_kind: kind.clone(),
        };
        assert_eq!(
            expected_permission.to_string(),
            "work unit `\"alpha\":\"main\"` is not allowed for phase `SignatureCollection` and output kind `SourceAlgorithmModuleSummary`"
        );
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            input.clone(),
            handle.clone(),
            original.clone(),
        );
        let retain = RetainOwner::new("driver116-obsolete-observer");
        if case == "obsolete" {
            owner
                .storage()
                .retain(handle.output(), retain.clone())
                .unwrap();
            owner.mark_obsolete(snapshot).unwrap();
            let collected = owner.storage().collect(CollectInput {
                snapshot,
                protected_outputs: vec![
                    frontend_input.parent_outputs()[0].as_output_ref().output(),
                ],
            });
            assert_eq!(
                (
                    collected.outputs_dropped,
                    collected.abandoned_slots_dropped,
                    collected.retained_outputs
                ),
                (0, 0, 1)
            );
            assert_eq!(*owner.storage().get(&handle).unwrap(), original);
        }
        if case == "collected" {
            let collected = owner.storage().collect(CollectInput {
                snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                (collected.outputs_dropped, collected.abandoned_slots_dropped),
                (2, 0)
            );
        }
        let expected =
            selected_inputs.native_algorithm_module_summary_with_frontend_parent(&input, owner);
        match case {
            "unsupported" => assert_eq!(expected, Ok(None)),
            "capture" => assert_eq!(
                expected,
                Err("native Frontend algorithm summary capture binding failed".into())
            ),
            "obsolete" => assert_eq!(
                expected,
                Err(owner
                    .validate_published_output(snapshot, &handle.erase())
                    .unwrap_err()
                    .to_string())
            ),
            "collected" => assert_eq!(
                expected,
                Err(owner
                    .storage()
                    .validate_handle(&handle.erase())
                    .unwrap_err()
                    .to_string())
            ),
            "denied-phase" | "denied-kind" => assert_eq!(expected, baseline),
            _ => unreachable!(),
        }
        let actual = selected_inputs.publish_native_algorithm_module_summary(&input, owner);
        if case.starts_with("denied-") {
            assert_eq!(actual.unwrap_err(), expected_permission.to_string());
            let collected = owner.storage().collect(CollectInput {
                snapshot,
                protected_outputs: vec![
                    frontend_input.parent_outputs()[0].as_output_ref().output(),
                    handle.output(),
                ],
            });
            assert_eq!(
                (
                    collected.outputs_dropped,
                    collected.abandoned_slots_dropped,
                    collected.protected_outputs
                ),
                (0, 1, 2)
            );
        } else {
            assert_eq!(
                actual
                    .map(|value| value
                        .map(|handle| (*owner.storage().get(&handle).unwrap()).clone())),
                expected
            );
        }
        if case != "collected" {
            assert_eq!(*owner.storage().get(&handle).unwrap(), original);
        }
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                input,
                handle.clone(),
                original
            ),
            before
        );
        if case == "obsolete" {
            assert!(owner.storage().release(handle.output(), &retain).unwrap());
        }
        let collected = owner.storage().collect(CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            (collected.outputs_dropped, collected.abandoned_slots_dropped),
            (if case == "collected" { 0 } else { 2 }, 0)
        );
    }
}
