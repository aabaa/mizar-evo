use super::*;

#[test]
fn native_theorem_preparation_core_caller_preserves_genuine_outputs() {
    use mizar_core::{
        control_flow::{
            ObligationHandoffOrigin, build_control_flow_ir, build_obligation_seed_handoff,
        },
        core_ir::{CoreProofStatus, ObligationSeedStatus},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, inputs, old, owner, _, (session, _, _)| {
                for (phase, kind) in [
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ("OverloadResolution", "SourceTheoremMetadata"),
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
                let meta_input = child(vec![typed.erase(), registration.erase()]);
                let parent = inputs
                    .publish_native_theorem_metadata(&meta_input, owner)
                    .unwrap()
                    .unwrap();
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("Elaboration"),
                    OutputKind::new("SourceTheoremCore"),
                    parent.work_unit().clone(),
                ));
                let metadata_input = child(vec![parent.erase()]);
                let core_parent = inputs
                    .publish_native_theorem_core_with_metadata_parent(&metadata_input, owner)
                    .unwrap()
                    .unwrap();
                let input = child(vec![core_parent.erase()]);
                let core = owner.storage().get(&core_parent).unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                assert_eq!(
                    core.source_id(),
                    inputs.snapshot.source_versions[0].source_id
                );
                assert_eq!(core.module_id(), &module);
                assert_eq!(
                    core_parent.placement() == &StoragePlacement::Resident,
                    !open
                );
                assert!(core.algorithms().is_empty());
                assert!(!core.proofs().is_empty());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                let flow = build_control_flow_ir(&core);
                let handoff = build_obligation_seed_handoff(&core, &flow);
                assert!(flow.flows.is_empty());
                assert!(flow.flow_map.is_empty());
                assert!(!handoff.entries.is_empty());
                assert_eq!(handoff.entries.len(), core.obligation_seeds().len());
                assert_eq!(handoff.source_map.len(), handoff.entries.len());
                for (id, entry) in handoff.entries.iter() {
                    let ObligationHandoffOrigin::ExistingCore { seed } = entry.origin else {
                        panic!("existing Core seed")
                    };
                    let original = core.obligation_seeds().get(seed).unwrap();
                    assert_eq!(&entry.seed, original);
                    assert_eq!(entry.seed.status, ObligationSeedStatus::Active);
                    assert!(core.items().get(entry.seed.owner).is_some());
                    assert!(core.formulas().get(entry.seed.goal.unwrap()).is_some());
                    assert_eq!(
                        handoff.source_map.get(&id),
                        Some(&core.source_map().obligation_sources[&seed])
                    );
                    assert_eq!(
                        entry.seed.source,
                        core.source_map().obligation_sources[&seed]
                    );
                    assert!(entry.flow_site.is_none());
                }
                let expected = (flow, handoff);
                let before = (
                    input.clone(),
                    core_parent.clone(),
                    (*core).clone(),
                    session.captured().clone(),
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
                                .native_theorem_preparation_with_core_parent(&input, owner)
                                .unwrap(),
                            Some(expected.clone())
                        );
                    }
                }
                assert_eq!(
                    (
                        input,
                        core_parent.clone(),
                        (*owner.storage().get(&core_parent).unwrap()).clone(),
                        session.captured().clone()
                    ),
                    before
                );
                let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 8);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}

#[test]
fn native_theorem_preparation_core_caller_preserves_refusals() {
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing",
        "wrong",
        "hash",
        "obsolete",
        "collected",
        "frontend-core",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, outputs, _| {
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
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
            let meta_input = child(vec![typed.erase(), registration.erase()]);
            let parent = inputs
                .publish_native_theorem_metadata(&meta_input, owner)
                .unwrap()
                .unwrap();
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("Elaboration"),
                OutputKind::new("SourceTheoremCore"),
                parent.work_unit().clone(),
            ));
            let core_parent = if case == "frontend-core" {
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let input = child(vec![frontend.clone()]);
                inputs
                    .publish_native_theorem_core(&input, owner)
                    .unwrap()
                    .unwrap()
            } else {
                let input = child(vec![parent.erase()]);
                inputs
                    .publish_native_theorem_core_with_metadata_parent(&input, owner)
                    .unwrap()
                    .unwrap()
            };
            let input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    if case == "hash" {
                        hash(0xe3)
                    } else {
                        old.identities().input_hash()
                    },
                    vec![],
                    match case {
                        "missing" => vec![],
                        "wrong" => vec![parent.erase()],
                        _ => vec![core_parent.erase()],
                    }
                    .into_iter()
                    .map(|parent| {
                        SealedParentOutputHandle::from_published_output(owner, old.snapshot, parent)
                            .unwrap()
                    })
                    .collect(),
                )
                .unwrap(),
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
                    8
                );
            }
            let expected = match case {
                "missing" | "hash" => {
                    "native theorem preparation requires the leaf input profile".to_owned()
                }
                "wrong" => "native theorem preparation parent binding mismatch".to_owned(),
                "frontend-core" => "native theorem preparation ancestry mismatch".to_owned(),
                "obsolete" => owner
                    .validate_published_output(old.snapshot, core_parent.any())
                    .unwrap_err()
                    .to_string(),
                "collected" => owner
                    .storage()
                    .validate_handle(core_parent.any())
                    .unwrap_err()
                    .to_string(),
                _ => unreachable!(),
            };
            assert_eq!(
                inputs
                    .native_theorem_preparation_with_core_parent(&input, owner)
                    .unwrap_err(),
                expected,
                "{case}"
            );
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 8 }
            );
            assert_eq!(collected.abandoned_slots_dropped, 0);
        });
    }
    with_native_summary_parent(
        text,
        false,
        0,
        |_, inputs, old, owner, outputs, (_, ids, snapshots)| {
            use mizar_driver::{
                native_theorem_core_from_storage_bytes as decode,
                native_theorem_core_semantic_payload as semantic,
                native_theorem_core_storage_bytes as encode,
            };
            use mizar_ir::{
                publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
                storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion},
            };
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
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
            let meta_input = child(vec![typed.erase(), registration.erase()]);
            let parent = inputs
                .publish_native_theorem_metadata(&meta_input, owner)
                .unwrap()
                .unwrap();
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("Elaboration"),
                OutputKind::new("SourceTheoremCore"),
                parent.work_unit().clone(),
            ));
            let metadata_input = child(vec![parent.erase()]);
            let core_a = inputs
                .native_theorem_core_with_metadata_parent(&metadata_input, owner)
                .unwrap()
                .unwrap();
            let frontend_a = outputs
                .iter()
                .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                .unwrap();
            let handle_a = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    frontend_a,
                    frontend_a.output_kind(),
                )
                .unwrap();
            let output_a = owner.storage().get(&handle_a).unwrap();
            let ast_a = output_a.ast.as_ref().unwrap();
            let changed = Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
            let (_, submission_b, adapter_b, frontend_b, _) =
                native_disk_frontend_record_submission(&changed, ids, snapshots, 0);
            let (inputs_b, old_b) = native_disk_lookup_context(&submission_b, ids);
            let publisher_b = adapter_b.publisher();
            let input_b = PhaseInput::new(
                old_b.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old_b.snapshot,
                    old_b.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            publisher_b,
                            old_b.snapshot,
                            frontend_b.erase(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            let original_b = inputs_b
                .native_theorem_core_with_frontend_parent(&input_b, publisher_b)
                .unwrap()
                .unwrap();
            let output_b = publisher_b.storage().get(&frontend_b).unwrap();
            let ast_b = output_b.ast.as_ref().unwrap();
            assert_ne!(ast_a.source_id, ast_b.source_id);
            let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let bytes = encode(ast_b, &module, &original_b).unwrap().unwrap();
            let core_b = decode(&bytes, ast_a.source_id, &module).unwrap().unwrap();
            let rebound = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
                &ast_b.canonical_bytes().unwrap(),
                ast_a.source_id,
            )
            .unwrap();
            assert_eq!(core_a.source_id(), core_b.source_id());
            assert_eq!(core_a.module_id(), core_b.module_id());
            assert_ne!(core_a, core_b);
            assert_eq!(encode(ast_a, &module, &core_b).unwrap(), None);
            assert_eq!(
                encode(&rebound, &module, &core_b).unwrap(),
                Some(bytes.clone())
            );
            let canonical = semantic(&rebound, &module, &core_b).unwrap().unwrap();
            let source_id = ast_a.source_id;
            let phase = IrPipelinePhase::new("Elaboration");
            let kind = OutputKind::new("SourceTheoremCore");
            let negative = owner
                .publish(PublishOutputInput {
                    slot: owner.allocate(
                        old.snapshot,
                        phase.clone(),
                        parent.work_unit().clone(),
                        kind.clone(),
                        SchemaVersion::new(1),
                    ),
                    snapshot: old.snapshot,
                    phase,
                    work_unit: parent.work_unit().clone(),
                    output_kind: kind,
                    schema_version: SchemaVersion::new(1),
                    payload: core_b.clone(),
                    canonical_payload: Some(canonical),
                    storage_payload: Some(bytes),
                    decode: BlobDecoder::new(move |bytes| {
                        decode(bytes, source_id, &module)
                            .map_err(BlobDecodeError::new)?
                            .ok_or_else(|| BlobDecodeError::new("negative Core replay"))
                    }),
                    parents: vec![parent.erase()],
                    named_input_hashes: parent.lineage().named_input_hashes.clone(),
                    side_tables: IrSideTables::default(),
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                })
                .unwrap();
            assert_eq!(*owner.storage().get(&negative).unwrap(), core_b);
            let input = child(vec![negative.erase()]);
            assert_eq!(
                inputs
                    .native_theorem_preparation_with_core_parent(&input, owner)
                    .unwrap(),
                None
            );
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 8);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        },
    );
}

#[test]
fn native_theorem_preparation_storage_replay_preserves_genuine_outputs() {
    use mizar_core::{
        control_flow::{
            ObligationHandoffOrigin, build_control_flow_ir, build_obligation_seed_handoff,
        },
        core_ir::{CoreProofStatus, ObligationSeedStatus},
    };
    use mizar_driver::{
        native_theorem_core_storage_bytes as encode_core,
        native_theorem_preparation_from_storage_bytes as decode,
        native_theorem_preparation_storage_bytes as encode,
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, inputs, old, owner, outputs, (_, ids, snapshots)| {
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
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
                                frontend.clone(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let core = inputs
                    .native_theorem_core_with_frontend_parent(&input, owner)
                    .unwrap()
                    .unwrap();
                let frontend_handle = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        frontend,
                        frontend.output_kind(),
                    )
                    .unwrap();
                let frontend_output = owner.storage().get(&frontend_handle).unwrap();
                let ast = frontend_output.ast.as_ref().unwrap();
                assert_eq!(
                    frontend.placement() == &StoragePlacement::Resident,
                    threshold != 0
                );
                assert_eq!(core.source_id(), ast.source_id);
                assert_eq!(core.module_id(), &module);
                assert!(core.algorithms().is_empty());
                assert!(!core.proofs().is_empty());
                assert!(
                    core.proofs().iter().all(|(_, proof)| {
                        proof.status == CoreProofStatus::PendingAutomaticProof
                    })
                );
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| { seed.status == ObligationSeedStatus::Active })
                );
                let flow = build_control_flow_ir(&core);
                let handoff = build_obligation_seed_handoff(&core, &flow);
                assert!(flow.flows.is_empty() && flow.flow_map.is_empty());
                assert!(!handoff.entries.is_empty());
                for (id, entry) in handoff.entries.iter() {
                    let ObligationHandoffOrigin::ExistingCore { seed } = entry.origin else {
                        panic!("existing Core seed")
                    };
                    assert_eq!(&entry.seed, core.obligation_seeds().get(seed).unwrap());
                    assert_eq!(
                        handoff.source_map.get(&id),
                        Some(&core.source_map().obligation_sources[&seed])
                    );
                    assert_eq!(
                        entry.seed.source,
                        core.source_map().obligation_sources[&seed]
                    );
                    assert!(core.items().get(entry.seed.owner).is_some());
                    assert!(core.formulas().get(entry.seed.goal.unwrap()).is_some());
                    assert!(entry.flow_site.is_none());
                }
                let pair = (flow, handoff);
                let core_bytes = encode_core(ast, &module, &core).unwrap().unwrap();
                let mut expected = b"mizar-driver/native-theorem-preparation-storage/v1\0".to_vec();
                expected.extend_from_slice(&u64::try_from(core_bytes.len()).unwrap().to_le_bytes());
                expected.extend_from_slice(&core_bytes);
                assert_eq!(
                    encode(ast, &module, &core, &pair).unwrap(),
                    Some(expected.clone())
                );
                assert_eq!(
                    decode(&expected, ast.source_id, &module).unwrap(),
                    Some(pair.clone())
                );
                let relocated_fixture = Fixture::new(text.as_bytes());
                let (_, submission, adapter, relocated_handle, _) =
                    native_disk_frontend_record_submission(&relocated_fixture, ids, snapshots, 0);
                let (relocated_inputs, relocated_old) =
                    native_disk_lookup_context(&submission, ids);
                let relocated_owner = adapter.publisher();
                let relocated_input = PhaseInput::new(
                    relocated_old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        relocated_old.snapshot,
                        relocated_old.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                relocated_owner,
                                relocated_old.snapshot,
                                relocated_handle.erase(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let relocated_core = relocated_inputs
                    .native_theorem_core_with_frontend_parent(&relocated_input, relocated_owner)
                    .unwrap()
                    .unwrap();
                let relocated_frontend = relocated_owner.storage().get(&relocated_handle).unwrap();
                let relocated_ast = relocated_frontend.ast.as_ref().unwrap();
                assert_ne!(ast.source_id, relocated_ast.source_id);
                let relocated_flow = build_control_flow_ir(&relocated_core);
                let relocated_handoff =
                    build_obligation_seed_handoff(&relocated_core, &relocated_flow);
                let relocated_pair = (relocated_flow, relocated_handoff);
                assert_ne!(pair, relocated_pair);
                assert_ne!(core, relocated_core);
                assert_eq!(
                    decode(&expected, relocated_ast.source_id, &module).unwrap(),
                    Some(relocated_pair.clone())
                );
                assert_eq!(encode_core(ast, &module, &relocated_core).unwrap(), None);
                assert_eq!(encode(ast, &module, &core, &relocated_pair).unwrap(), None);
                assert!(
                    encode(relocated_ast, &module, &relocated_core, &relocated_pair)
                        .unwrap()
                        .is_some()
                );
                assert_eq!(
                    encode(ast, &module, &relocated_core, &relocated_pair).unwrap(),
                    None
                );
                let before = (ast.clone(), module.clone(), core.clone(), pair.clone());
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
                    assert_eq!(
                        encode(ast, &module, &core, &pair).unwrap(),
                        Some(expected.clone())
                    );
                    assert_eq!(
                        decode(&expected, ast.source_id, &module).unwrap(),
                        Some(pair.clone())
                    );
                }
                assert_eq!(
                    (ast.clone(), module.clone(), core.clone(), pair.clone()),
                    before
                );
            },
        );
    }

    let text = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, submission, adapter, handle, _) =
        native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
    let (inputs, old) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
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
                    handle.erase(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    );
    let core = inputs
        .native_theorem_core_with_frontend_parent(&input, owner)
        .unwrap()
        .unwrap();
    let output = owner.storage().get(&handle).unwrap();
    let ast = output.ast.as_ref().unwrap();
    assert!(core.algorithms().is_empty());
    let core_bytes = encode_core(ast, &module, &core).unwrap().unwrap();
    let flow = build_control_flow_ir(&core);
    let handoff = build_obligation_seed_handoff(&core, &flow);
    let pair = (flow, handoff);
    let mut outer = b"mizar-driver/native-theorem-preparation-storage/v1\0".to_vec();
    outer.extend_from_slice(&u64::try_from(core_bytes.len()).unwrap().to_le_bytes());
    outer.extend_from_slice(&core_bytes);
    assert_eq!(
        encode(ast, &module, &core, &pair).unwrap(),
        Some(outer.clone())
    );
    assert_eq!(decode(&outer, ast.source_id, &module).unwrap(), Some(pair));
}

#[test]
fn native_theorem_preparation_storage_replay_preserves_refusals() {
    use mizar_driver::{
        native_theorem_core_from_storage_bytes as decode_core,
        native_theorem_core_storage_bytes as encode_core,
        native_theorem_preparation_from_storage_bytes as decode,
        native_theorem_preparation_storage_bytes as encode,
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, submission, adapter, handle, _) =
        native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
    let (inputs, old) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
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
                    handle.erase(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    );
    let core = inputs
        .native_theorem_core_with_frontend_parent(&input, owner)
        .unwrap()
        .unwrap();
    let output = owner.storage().get(&handle).unwrap();
    let ast = output.ast.as_ref().unwrap();
    let inner = encode_core(ast, &module, &core).unwrap().unwrap();
    let flow = mizar_core::control_flow::build_control_flow_ir(&core);
    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
    let pair = (flow, handoff);
    assert!(core.algorithms().is_empty());
    let mut outer = b"mizar-driver/native-theorem-preparation-storage/v1\0".to_vec();
    outer.extend_from_slice(&u64::try_from(inner.len()).unwrap().to_le_bytes());
    outer.extend_from_slice(&inner);
    let domain = b"mizar-driver/native-theorem-preparation-storage/v1\0";
    let mut bad_domain = outer.clone();
    bad_domain[0] = b'x';
    let mut bad_version = outer.clone();
    bad_version[domain.len() - 2] = b'2';
    let mut bad_length = outer.clone();
    bad_length[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    let mut trailing = outer.clone();
    trailing.push(0);
    let mut truncated = outer.clone();
    truncated.pop();
    let mut short_header = domain.to_vec();
    short_header.extend_from_slice(&[1, 2, 3]);
    let mut oversized = outer.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    for malformed in [
        Vec::new(),
        bad_domain,
        bad_version,
        bad_length,
        trailing,
        truncated,
        short_header,
        oversized,
    ] {
        assert_eq!(decode(&malformed, ast.source_id, &module), Ok(None));
    }
    let mut invalid_inner = inner.clone();
    invalid_inner[inner.len() - ast.canonical_bytes().unwrap().len()] = b'!';
    assert_eq!(
        decode_core(&invalid_inner, ast.source_id, &module),
        Ok(None)
    );
    let mut framed_none = domain.to_vec();
    framed_none.extend_from_slice(&u64::try_from(invalid_inner.len()).unwrap().to_le_bytes());
    framed_none.extend_from_slice(&invalid_inner);
    assert_eq!(decode(&framed_none, ast.source_id, &module), Ok(None));

    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let fixture = Fixture::new(private.replacen("thus X = X;", "thus X = 0;", 1).as_bytes());
    let (_, _, adapter, handle, _) =
        native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
    let output = adapter.publisher().storage().get(&handle).unwrap();
    let ast = output.ast.as_ref().unwrap();
    let syntax = ast.canonical_bytes().unwrap();
    let mut private_inner = b"mizar-driver/native-theorem-core-storage/v1\0".to_vec();
    for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
        private_inner.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        private_inner.extend_from_slice(field);
    }
    let old_error = encode_core(ast, &module, &core).unwrap_err();
    assert_eq!(encode(ast, &module, &core, &pair), Err(old_error.clone()));
    let expected = decode_core(&private_inner, ast.source_id, &module).map(|_| None);
    assert!(expected.is_err());
    assert_eq!(expected.as_ref().unwrap_err(), &old_error);
    let mut framed_error = domain.to_vec();
    framed_error.extend_from_slice(&u64::try_from(private_inner.len()).unwrap().to_le_bytes());
    framed_error.extend_from_slice(&private_inner);
    assert_eq!(decode(&framed_error, ast.source_id, &module), expected);
}

#[test]
fn native_theorem_preparation_publication_seals_genuine_payload_and_streams() {
    use mizar_core::{
        control_flow::{
            ObligationHandoffOrigin, build_control_flow_ir, build_obligation_seed_handoff,
        },
        core_ir::{CoreProofStatus, ObligationSeedStatus},
    };
    use mizar_driver::{
        native_theorem_core_semantic_payload as semantics,
        native_theorem_preparation_storage_bytes as encode,
    };
    use mizar_ir::{
        publisher::PublishError,
        storage::{IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    let field = |hasher: &mut blake3::Hasher, value: &[u8]| {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value);
    };
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut representations = Vec::new();
        for text in [
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            "theorem T: for X being set holds X = X\nproof\n let X be set;\n thus X = X;\nend;",
            "theorem Renamed: for X being set holds X = X proof let X be set; thus X = X; end;",
        ] {
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, old, owner, outputs, (session, _, _)| {
                    let storage = owner.storage();
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                        ("Elaboration", "SourceTheoremCore"),
                        ("AlgorithmPreparation", "SourceTheoremPreparation"),
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
                    let typed = inputs
                        .publish_native_theorem_typed_ast(&child(vec![signature.erase()]), owner)
                        .unwrap()
                        .unwrap();
                    let reg_input = child(vec![typed.erase()]);
                    let registration = inputs
                        .publish_native_theorem_registration_closure(&reg_input, owner)
                        .unwrap()
                        .unwrap();
                    let meta_input = child(vec![typed.erase(), registration.erase()]);
                    let parent = inputs
                        .publish_native_theorem_metadata(&meta_input, owner)
                        .unwrap()
                        .unwrap();

                    let metadata_input = child(vec![parent.erase()]);
                    let core_parent = inputs
                        .publish_native_theorem_core_with_metadata_parent(&metadata_input, owner)
                        .unwrap()
                        .unwrap();
                    let input = child(vec![core_parent.erase()]);

                    let core = storage.get(&core_parent).unwrap();
                    let module =
                        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                    assert_eq!(
                        core.source_id(),
                        inputs.snapshot.source_versions[0].source_id
                    );
                    assert_eq!(core.module_id(), &module);
                    assert!(!core.proofs().is_empty());
                    assert!(
                        core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    let flow = build_control_flow_ir(&core);
                    let handoff = build_obligation_seed_handoff(&core, &flow);
                    assert!(flow.flows.is_empty() && flow.flow_map.is_empty());
                    assert_eq!(handoff.entries.len(), 1);
                    for (id, entry) in handoff.entries.iter() {
                        let ObligationHandoffOrigin::ExistingCore { seed } = entry.origin else {
                            panic!("existing seed")
                        };
                        assert_eq!(&entry.seed, core.obligation_seeds().get(seed).unwrap());
                        assert_eq!(entry.seed.status, ObligationSeedStatus::Active);
                        assert_eq!(
                            handoff.source_map.get(&id),
                            Some(&core.source_map().obligation_sources[&seed])
                        );
                        assert!(entry.flow_site.is_none());
                    }
                    let expected = (flow, handoff);
                    let frontend = outputs
                        .iter()
                        .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                        .unwrap();
                    let handle = storage
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend,
                            frontend.output_kind(),
                        )
                        .unwrap();
                    let retained = storage.get(&handle).unwrap();
                    let ast = retained.ast.as_ref().unwrap();
                    let semantic = semantics(ast, &module, &core).unwrap().unwrap();
                    let bytes = encode(ast, &module, &core, &expected).unwrap().unwrap();
                    let mut canonical =
                        b"mizar-driver/native-theorem-preparation-content/v1\0".to_vec();
                    canonical.extend_from_slice(&(semantic.len() as u64).to_le_bytes());
                    canonical.extend_from_slice(&semantic);
                    let version = &inputs.snapshot.source_versions[0];
                    let mut content = blake3::Hasher::new();
                    field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                    field(&mut content, &canonical);
                    content.update(&1_u64.to_le_bytes());
                    field(&mut content, b"parent_content");
                    content.update(core_parent.content_hash().as_bytes());
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
                    let digest = Hash::from_bytes(blake3::derive_key(
                        "mizar-driver/native-theorem-preparation-storage-side-table/v1",
                        &bytes,
                    ));
                    let tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "prepare-theorem-storage",
                            version.normalized_path.as_str(),
                            digest,
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
                            field(&mut side, b"prepare-theorem-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    let before = (input.clone(), (*core).clone(), session.captured().clone());
                    let sealed = inputs
                        .publish_native_theorem_preparation(&input, owner)
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), old.snapshot);
                    assert_eq!(
                        sealed.phase(),
                        &IrPipelinePhase::new("AlgorithmPreparation")
                    );
                    assert_eq!(
                        sealed.output_kind(),
                        &OutputKind::new("SourceTheoremPreparation")
                    );
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), core_parent.work_unit());
                    assert_eq!(sealed.lineage().parents, vec![core_parent.output()]);
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        core_parent.lineage().named_input_hashes
                    );
                    assert_eq!(sealed.content_hash(), expected_content);
                    assert_eq!(sealed.side_table_hash(), expected_side);
                    assert_eq!(*storage.side_tables(&sealed).unwrap(), tables);
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
                        assert_eq!(*storage.get(&sealed).unwrap(), expected);
                        let duplicate = PublishError::Storage {
                            error: Box::new(StorageError::OutputAlreadyStored {
                                output: sealed.output(),
                            }),
                        }
                        .to_string();
                        assert_eq!(
                            inputs
                                .publish_native_theorem_preparation(&input, owner)
                                .unwrap_err(),
                            duplicate
                        );
                        owner
                            .validate_published_output(old.snapshot, sealed.any())
                            .unwrap();
                        assert_eq!(
                            owner
                                .validate_current_output(old.snapshot, sealed.any())
                                .is_ok(),
                            !open
                        );
                    }
                    assert_eq!(
                        (
                            input,
                            (*storage.get(&core_parent).unwrap()).clone(),
                            session.captured().clone()
                        ),
                        before
                    );
                    let collected = storage.collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 9);
                    assert_eq!(collected.abandoned_slots_dropped, 3);
                    representations.push((semantic, bytes));
                },
            );
        }
        assert_eq!(representations[0].0, representations[1].0);
        assert_eq!(representations[0].0, representations[2].0);
        assert_ne!(representations[0].0, representations[3].0);
        assert_ne!(representations[0].1, representations[1].1);
        assert_ne!(representations[0].1, representations[2].1);
    }
}

#[test]
fn native_theorem_preparation_publication_preserves_refusals() {
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishError, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing",
        "wrong",
        "hash",
        "obsolete",
        "collected",
        "denied",
        "lemma",
    ] {
        let source = if case == "lemma" {
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            )
        } else {
            text
        };
        with_native_summary_parent(source, false, 0, |_, inputs, old, owner, outputs, _| {
            let storage = owner.storage();
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
                ("Elaboration", "SourceTheoremCore"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let key = old.identities().input_hash();
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
            let typed = inputs
                .publish_native_theorem_typed_ast(&child(vec![signature.erase()], key), owner)
                .unwrap()
                .unwrap();
            let reg_input = child(vec![typed.erase()], key);
            let registration = inputs
                .publish_native_theorem_registration_closure(&reg_input, owner)
                .unwrap()
                .unwrap();
            let meta_input = child(vec![typed.erase(), registration.erase()], key);
            let parent = inputs
                .publish_native_theorem_metadata(&meta_input, owner)
                .unwrap()
                .unwrap();

            let metadata_input = child(vec![parent.erase()], key);
            let mut semantic_error = None;
            let core_parent = if case == "lemma" {
                let core = inputs
                    .native_theorem_core_with_metadata_parent(&metadata_input, owner)
                    .unwrap()
                    .unwrap();
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let handle = storage
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        frontend,
                        frontend.output_kind(),
                    )
                    .unwrap();
                let output = storage.get(&handle).unwrap();
                let ast = output.ast.as_ref().unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                semantic_error = Some(
                    mizar_driver::native_theorem_core_semantic_payload(ast, &module, &core)
                        .unwrap_err(),
                );
                let bytes = mizar_driver::native_theorem_core_storage_bytes(ast, &module, &core)
                    .unwrap()
                    .unwrap();
                let source_id = ast.source_id;
                let phase = IrPipelinePhase::new("Elaboration");
                let kind = OutputKind::new("SourceTheoremCore");
                owner
                    .publish(PublishOutputInput {
                        slot: owner.allocate(
                            old.snapshot,
                            phase.clone(),
                            parent.work_unit().clone(),
                            kind.clone(),
                            SchemaVersion::new(1),
                        ),
                        snapshot: old.snapshot,
                        phase,
                        work_unit: parent.work_unit().clone(),
                        output_kind: kind,
                        schema_version: SchemaVersion::new(1),
                        payload: core,
                        canonical_payload: Some(bytes.clone()),
                        storage_payload: Some(bytes),
                        decode: BlobDecoder::new(move |bytes| {
                            mizar_driver::native_theorem_core_from_storage_bytes(
                                bytes, source_id, &module,
                            )
                            .map_err(BlobDecodeError::new)?
                            .ok_or_else(|| BlobDecodeError::new("negative Lemma Core replay"))
                        }),
                        parents: vec![parent.erase()],
                        named_input_hashes: parent.lineage().named_input_hashes.clone(),
                        side_tables: IrSideTables::default(),
                        origin: OutputOrigin::PackageSource,
                        target: PublicationTarget::CurrentPackage,
                    })
                    .unwrap()
            } else {
                inputs
                    .publish_native_theorem_core_with_metadata_parent(&metadata_input, owner)
                    .unwrap()
                    .unwrap()
            };
            let phase = IrPipelinePhase::new("AlgorithmPreparation");
            let kind = OutputKind::new("SourceTheoremPreparation");
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    phase.clone(),
                    kind.clone(),
                    parent.work_unit().clone(),
                ));
            }
            let input = child(
                match case {
                    "missing" => vec![],
                    "wrong" => vec![parent.erase()],
                    _ => vec![core_parent.erase()],
                },
                if case == "hash" { hash(0xe5) } else { key },
            );
            let collection = CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            };
            if case == "obsolete" {
                owner.mark_obsolete(old.snapshot).unwrap();
            } else if case == "collected" {
                assert_eq!(storage.collect(collection.clone()).outputs_dropped, 8);
            }
            let raw = inputs.native_theorem_preparation_with_core_parent(&input, owner);
            let expected = if case == "lemma" {
                assert!(raw.unwrap().is_some());
                semantic_error.unwrap()
            } else if case == "denied" {
                assert!(raw.unwrap().is_some());
                PublishError::WorkUnitNotAllowed {
                    phase,
                    work_unit: parent.work_unit().clone(),
                    output_kind: kind,
                }
                .to_string()
            } else {
                raw.unwrap_err()
            };
            assert_eq!(
                inputs
                    .publish_native_theorem_preparation(&input, owner)
                    .unwrap_err(),
                expected,
                "{case}"
            );
            let collected = storage.collect(collection);
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 8 }
            );
            assert_eq!(
                collected.abandoned_slots_dropped,
                usize::from(case == "denied")
            );
        });
    }
}
