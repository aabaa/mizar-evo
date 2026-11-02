use super::*;

#[test]
fn native_theorem_kernel_handoff_core_vc_caller_preserves_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_vc::{
        kernel_evidence_handoff::{
            KernelGoalPolarity, build_source_theorem_kernel_handoff as handoff,
        },
        vc_ir::{GenerationSchemaVersion, SeedOriginRef, VcSchemaVersion, VcStatus},
    };
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut positions = std::collections::BTreeSet::new();
        for variant in 0..64 {
            let binder = if variant == 0 { "X" } else { "Renamed" };
            let text = format!(
                "theorem Placement{variant}: for {binder} being set holds {binder} = {binder} proof let {binder} be set; thus {binder} = {binder}; end;"
            );
            with_native_summary_parent(
                &text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, (session, _, _)| {
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                        ("Elaboration", "SourceTheoremCore"),
                        ("AlgorithmPreparation", "SourceTheoremPreparation"),
                        ("VcGenerate", "SourceTheoremVC"),
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
                    let metadata_input = child(vec![parent.erase()]);
                    let core_parent = inputs
                        .publish_native_theorem_core_with_metadata_parent(&metadata_input, owner)
                        .unwrap()
                        .unwrap();
                    let core_input = child(vec![core_parent.erase()]);
                    let preparation = inputs
                        .publish_native_theorem_preparation(&core_input, owner)
                        .unwrap()
                        .unwrap();
                    assert!(
                        inputs
                            .native_theorem_preparation_with_core_parent(&core_input, owner)
                            .unwrap()
                            .is_some()
                    );
                    let generation =
                        GenerationSchemaVersion::new(if variant == 0 { " \n\t" } else { "" });
                    let schema = VcSchemaVersion::new(if variant == 0 { "" } else { "核\0" });
                    let vc_input = child(vec![core_parent.erase(), preparation.erase()]);
                    let vc_parent = inputs
                        .publish_native_theorem_vcs_with_core_preparation_parents(
                            &vc_input,
                            owner,
                            &generation,
                            &schema,
                        )
                        .unwrap()
                        .unwrap();
                    let input = child(vec![vc_parent.erase(), core_parent.erase()]);
                    positions.insert(
                        input
                            .parent_outputs()
                            .iter()
                            .position(|parent| parent.output() == core_parent.output())
                            .unwrap(),
                    );
                    let core = owner.storage().get(&core_parent).unwrap();
                    let vcs = owner.storage().get(&vc_parent).unwrap();
                    let selected = vcs.vcs()[0].id;
                    let vc = vcs.vc(selected).unwrap();
                    let expected = handoff(&core, &vcs, selected).unwrap();
                    assert_eq!(
                        core.source_id(),
                        inputs.snapshot.source_versions[0].source_id
                    );
                    assert_eq!(
                        core.module_id(),
                        &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                    );
                    assert_eq!(vcs.source(), core.source_id());
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.schema_version(), &schema);
                    assert!(
                        !vcs.vcs().is_empty()
                            && !core.proofs().is_empty()
                            && !vcs.seed_accounting().is_empty()
                    );
                    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open
                        && vc.anchor.generation_schema_version == generation));
                    assert!(
                        core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    for row in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!("Core seed")
                        };
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(
                            core.obligation_seeds().get(seed).unwrap().status,
                            ObligationSeedStatus::Active
                        );
                    }
                    assert_eq!(expected.diagnostics().vc, selected);
                    assert!(expected.context_identity_is_consistent());
                    let envelope = expected.canonical_evidence();
                    assert_eq!(envelope.symbol_manifest().len(), 2);
                    assert_eq!(envelope.variable_manifest().len(), 1);
                    assert!(
                        envelope
                            .symbol_manifest()
                            .iter()
                            .chain(envelope.variable_manifest())
                            .all(|entry| !entry.payload.is_empty())
                    );
                    let local = vc
                        .local_context
                        .entries()
                        .iter()
                        .filter_map(|entry| entry.formula)
                        .collect::<Vec<_>>();
                    assert!(!local.is_empty());
                    assert_eq!(
                        envelope
                            .formula_evidence()
                            .iter()
                            .map(|entry| entry.producer_formula_ref().unwrap())
                            .collect::<Vec<_>>(),
                        local
                    );
                    assert_eq!(
                        expected
                            .context_identity()
                            .entries()
                            .iter()
                            .map(|entry| entry.producer_formula_ref())
                            .collect::<Vec<_>>(),
                        local
                    );
                    assert_eq!(envelope.final_goal().producer_formula_ref, vc.goal);
                    assert_eq!(
                        envelope.final_goal().polarity,
                        KernelGoalPolarity::AssertFalseForRefutation
                    );
                    assert!(!envelope.final_goal().formula_bytes.is_empty());
                    assert_eq!(
                        core_parent.placement() == &StoragePlacement::Resident,
                        !open
                    );
                    assert_eq!(vc_parent.placement() == &StoragePlacement::Resident, !open);
                    let before = (
                        input.clone(),
                        (*core).clone(),
                        (*vcs).clone(),
                        selected,
                        generation.clone(),
                        schema.clone(),
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
                                    .native_theorem_kernel_handoff_with_core_vc_parents(
                                        &input, owner, selected
                                    )
                                    .unwrap(),
                                Some(expected.clone())
                            );
                        }
                    }
                    assert_eq!(
                        (
                            input,
                            (*owner.storage().get(&core_parent).unwrap()).clone(),
                            (*owner.storage().get(&vc_parent).unwrap()).clone(),
                            selected,
                            generation,
                            schema,
                            session.captured().clone()
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 10);
                    assert_eq!(collected.abandoned_slots_dropped, 0);
                },
            );
            if positions.len() == 2 {
                break;
            }
        }
        assert_eq!(positions, [0, 1].into_iter().collect());
    }
}

#[test]
fn native_theorem_kernel_handoff_core_vc_caller_preserves_refusals() {
    use mizar_core::control_flow::{build_control_flow_ir, build_obligation_seed_handoff};
    use mizar_driver::{
        native_theorem_core_from_storage_bytes as decode_core,
        native_theorem_core_storage_bytes as encode_core,
        native_theorem_preparation_from_storage_bytes as decode_pair,
        native_theorem_preparation_storage_bytes as encode_pair,
        native_theorem_vc_from_storage_bytes as decode, native_theorem_vc_storage_bytes as encode,
    };
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        kernel_evidence_handoff::build_source_theorem_kernel_handoff as handoff,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let generation = GenerationSchemaVersion::new(" \n\t");
    let schema = VcSchemaVersion::new("");
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing-core",
        "missing-vc",
        "wrong-core",
        "wrong-vc",
        "hash",
        "obsolete",
        "collected-core",
        "collected-vc",
        "dependencies",
        "absent",
        "frontend-vc",
        "snapshot",
        "source-none",
        "ancestors-collected",
    ] {
        with_native_summary_parent(
            text,
            false,
            0,
            |_, inputs, old, owner, outputs, (_, ids, snapshots)| {
                for (phase, kind) in [
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ("OverloadResolution", "SourceTheoremMetadata"),
                    ("Elaboration", "SourceTheoremCore"),
                    ("AlgorithmPreparation", "SourceTheoremPreparation"),
                    ("VcGenerate", "SourceTheoremVC"),
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
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let metadata_input = child(vec![parent.erase()]);
                let mut other_snapshot = None;
                let (core_parent, preparation, foreign_ast) = if case == "source-none" {
                    let actual_core = inputs
                        .native_theorem_core_with_metadata_parent(&metadata_input, owner)
                        .unwrap()
                        .unwrap();
                    let changed = Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
                    let (_, submission, adapter, frontend, _) =
                        native_disk_frontend_record_submission(&changed, ids, snapshots, 0);
                    let (other_inputs, other_old) = native_disk_lookup_context(&submission, ids);
                    let other_owner = adapter.publisher();
                    let input = PhaseInput::new(
                        other_old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            other_old.snapshot,
                            other_old.identities().input_hash(),
                            vec![],
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    other_owner,
                                    other_old.snapshot,
                                    frontend.erase(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    let core = other_inputs
                        .native_theorem_core_with_frontend_parent(&input, other_owner)
                        .unwrap()
                        .unwrap();
                    let output = other_owner.storage().get(&frontend).unwrap();
                    let ast = output.ast.as_ref().unwrap().clone();
                    assert_ne!(core.source_id(), actual_core.source_id());
                    assert_ne!(core, actual_core);
                    let source_id = ast.source_id;
                    let module = core.module_id().clone();
                    let flow = build_control_flow_ir(&core);
                    let pair = (flow.clone(), build_obligation_seed_handoff(&core, &flow));
                    let core_bytes = encode_core(&ast, &module, &core).unwrap().unwrap();
                    let bytes = encode_pair(&ast, &module, &core, &pair).unwrap().unwrap();
                    let phase = IrPipelinePhase::new("Elaboration");
                    let kind = OutputKind::new("SourceTheoremCore");
                    let core_module = module.clone();
                    let core = owner
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
                            canonical_payload: Some(core_bytes.clone()),
                            storage_payload: Some(core_bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode_core(bytes, source_id, &core_module)
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
                    let phase = IrPipelinePhase::new("AlgorithmPreparation");
                    let kind = OutputKind::new("SourceTheoremPreparation");
                    let preparation = owner
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
                            payload: pair.clone(),
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode_pair(bytes, source_id, &module)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| {
                                        BlobDecodeError::new("negative Preparation replay")
                                    })
                            }),
                            parents: vec![core.erase()],
                            named_input_hashes: parent.lineage().named_input_hashes.clone(),
                            side_tables: IrSideTables::default(),
                            origin: OutputOrigin::PackageSource,
                            target: PublicationTarget::CurrentPackage,
                        })
                        .unwrap();
                    assert_eq!(*owner.storage().get(&preparation).unwrap(), pair);
                    (core, preparation, Some(ast))
                } else {
                    let core = inputs
                        .publish_native_theorem_core_with_metadata_parent(&metadata_input, owner)
                        .unwrap()
                        .unwrap();
                    let sole = child(vec![core.erase()]);
                    let preparation = inputs
                        .publish_native_theorem_preparation(&sole, owner)
                        .unwrap()
                        .unwrap();
                    (core, preparation, None)
                };
                let core_input = child(vec![core_parent.erase()]);
                let core = owner.storage().get(&core_parent).unwrap();
                let vc_parent = if matches!(case, "snapshot" | "source-none") {
                    let ast =
                        if case == "snapshot" {
                            let frontend_handle = owner.storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend, frontend.output_kind(),
                            ).unwrap();
                            let output = owner.storage().get(&frontend_handle).unwrap();
                            output.ast.as_ref().unwrap().clone()
                        } else {
                            foreign_ast.unwrap()
                        };
                    let snapshot = if case == "snapshot" {
                        let other = Fixture::new(text.as_bytes());
                        let (_, submission, _, _, _) =
                            native_disk_frontend_record_submission(&other, ids, snapshots, 0);
                        let snapshot = submission.session.captured().snapshot().id;
                        assert_ne!(snapshot, old.snapshot);
                        other_snapshot = Some(snapshot);
                        snapshot
                    } else {
                        old.snapshot
                    };
                    let graph = generate(&core, snapshot, &generation, &schema).unwrap();
                    assert!(handoff(&core, &graph, graph.vcs()[0].id).is_ok());
                    let raw = inputs
                        .native_theorem_preparation_with_core_parent(&core_input, owner)
                        .unwrap();
                    assert_eq!(raw.is_some(), case == "snapshot");
                    let bytes =
                        encode(&ast, core.module_id(), &core, &graph, snapshot, &generation)
                            .unwrap()
                            .unwrap();
                    let source_id = core.source_id();
                    let module = core.module_id().clone();
                    let phase = IrPipelinePhase::new("VcGenerate");
                    let kind = OutputKind::new("SourceTheoremVC");
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
                            payload: graph.clone(),
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode(bytes, source_id, &module, snapshot)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| BlobDecodeError::new("negative VC replay"))
                            }),
                            parents: vec![core_parent.erase(), preparation.erase()],
                            named_input_hashes: parent.lineage().named_input_hashes.clone(),
                            side_tables: IrSideTables::default(),
                            origin: OutputOrigin::PackageSource,
                            target: PublicationTarget::CurrentPackage,
                        })
                        .unwrap();
                    assert_eq!(*owner.storage().get(&negative).unwrap(), graph);
                    negative
                } else if case == "frontend-vc" {
                    let old_input = child(vec![frontend.clone()]);
                    inputs
                        .publish_native_theorem_vcs(&old_input, owner, &generation, &schema)
                        .unwrap()
                        .unwrap()
                } else {
                    let input = child(vec![core_parent.erase(), preparation.erase()]);
                    inputs
                        .publish_native_theorem_vcs_with_core_preparation_parents(
                            &input,
                            owner,
                            &generation,
                            &schema,
                        )
                        .unwrap()
                        .unwrap()
                };
                let vcs = owner.storage().get(&vc_parent).unwrap();
                let valid = vcs.vcs()[0].id;
                let selected = if matches!(case, "absent" | "source-none" | "dependencies") {
                    VcId::new(vcs.vcs().len())
                } else {
                    valid
                };
                let baseline = handoff(&core, &vcs, selected);
                if matches!(case, "snapshot" | "frontend-vc") {
                    assert!(baseline.is_ok());
                    assert!(
                        inputs
                            .native_theorem_preparation_with_core_parent(&core_input, owner)
                            .unwrap()
                            .is_some()
                    );
                }
                if case == "snapshot" {
                    assert_eq!(vcs.snapshot(), other_snapshot.unwrap());
                }
                let mut input = child(match case {
                    "missing-core" => vec![vc_parent.erase()],
                    "missing-vc" => vec![core_parent.erase()],
                    "wrong-core" => vec![parent.erase(), vc_parent.erase()],
                    "wrong-vc" => vec![core_parent.erase(), parent.erase()],
                    _ => vec![core_parent.erase(), vc_parent.erase()],
                });
                if case == "hash" {
                    input = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            hash(0xd0),
                            vec![],
                            input.parent_outputs().to_vec(),
                        )
                        .unwrap(),
                    );
                }
                let upstream = if case == "dependencies" {
                    let [dual, sole] = [input.clone(), core_input].map(|prior| {
                        PhaseInput::new(
                            prior.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                prior.snapshot,
                                prior.identities().input_hash(),
                                vec![hash(0xd1)],
                                prior.parent_outputs().to_vec(),
                            )
                            .unwrap(),
                        )
                    });
                    input = dual;
                    Some(
                        inputs
                            .native_theorem_preparation_with_core_parent(&sole, owner)
                            .unwrap_err(),
                    )
                } else {
                    None
                };
                if case == "obsolete" {
                    owner.mark_obsolete(old.snapshot).unwrap();
                } else if matches!(
                    case,
                    "collected-core" | "collected-vc" | "ancestors-collected"
                ) {
                    let protected = match case {
                        "collected-core" => vec![vc_parent.output()],
                        "collected-vc" => vec![core_parent.output()],
                        _ => vec![core_parent.output(), vc_parent.output()],
                    };
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: old.snapshot,
                                protected_outputs: protected
                            })
                            .outputs_dropped,
                        if case == "ancestors-collected" { 8 } else { 9 }
                    );
                }
                let result = inputs
                    .native_theorem_kernel_handoff_with_core_vc_parents(&input, owner, selected);
                match case {
                    "source-none" => assert_eq!(result.unwrap(), None),
                    "snapshot" => assert_eq!(
                        result.unwrap_err(),
                        "native theorem kernel handoff VC snapshot mismatch"
                    ),
                    "dependencies" => assert_eq!(result.unwrap_err(), upstream.unwrap()),
                    "absent" => assert_eq!(result.unwrap_err(), baseline.unwrap_err()),
                    "frontend-vc" => assert_eq!(
                        result.unwrap_err(),
                        "native theorem kernel handoff requires Core and Preparation ancestry"
                    ),
                    "ancestors-collected" => assert_eq!(result.unwrap(), Some(baseline.unwrap())),
                    _ => assert!(result.is_err(), "{case}"),
                }
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "ancestors-collected" {
                        2
                    } else if case.starts_with("collected-") {
                        1
                    } else {
                        10
                    }
                );
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}
