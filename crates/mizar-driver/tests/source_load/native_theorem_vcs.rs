use super::*;

#[test]
fn native_theorem_vcs_core_preparation_caller_preserves_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, SeedOriginRef, SeedVcMapping, VcSchemaVersion, VcStatus},
    };
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut positions = std::collections::BTreeSet::new();
        for variant in 0..64 {
            let text = format!(
                "theorem Placement{variant}: for X being set holds X = X proof let X be set; thus X = X; end;"
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
                    let core_input = child(vec![parent.erase()]);
                    let core = inputs
                        .publish_native_theorem_core_with_metadata_parent(&core_input, owner)
                        .unwrap()
                        .unwrap();
                    let sole_core = child(vec![core.erase()]);
                    let preparation = inputs
                        .publish_native_theorem_preparation(&sole_core, owner)
                        .unwrap()
                        .unwrap();
                    let input = child(vec![preparation.erase(), core.erase()]);
                    positions.insert(
                        input
                            .parent_outputs()
                            .iter()
                            .position(|parent| parent.output() == core.output())
                            .unwrap(),
                    );
                    let retained_core = owner.storage().get(&core).unwrap();
                    let retained_preparation = owner.storage().get(&preparation).unwrap();
                    assert_eq!(core.placement() == &StoragePlacement::Resident, !open);
                    assert_eq!(
                        preparation.placement() == &StoragePlacement::Resident,
                        !open
                    );
                    assert_eq!(
                        retained_core.source_id(),
                        inputs.snapshot.source_versions[0].source_id
                    );
                    assert_eq!(
                        retained_core.module_id(),
                        &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                    );
                    assert!(!retained_core.proofs().is_empty());
                    assert!(
                        retained_core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    assert_eq!(
                        *retained_preparation,
                        inputs
                            .native_theorem_preparation_with_core_parent(&sole_core, owner)
                            .unwrap()
                            .unwrap()
                    );
                    let before = (
                        input.clone(),
                        (*retained_core).clone(),
                        (*retained_preparation).clone(),
                        session.captured().clone(),
                    );
                    for (generation, schema) in [(" \nraw generation\t", "\0vc"), ("", "")] {
                        let generation = GenerationSchemaVersion::new(generation);
                        let schema = VcSchemaVersion::new(schema);
                        let schemas_before = (generation.clone(), schema.clone());
                        let expected =
                            generate(&retained_core, old.snapshot, &generation, &schema).unwrap();
                        assert_eq!(expected.source(), retained_core.source_id());
                        assert_eq!(expected.snapshot(), old.snapshot);
                        assert_eq!(expected.module().as_str(), "package=5:alpha;module=4:main");
                        assert_eq!(expected.schema_version(), &schema);
                        assert!(!expected.vcs().is_empty());
                        assert!(expected.vcs().iter().all(|vc| vc.status == VcStatus::Open
                            && vc.anchor.generation_schema_version == generation));
                        assert_eq!(
                            expected.seed_accounting().len(),
                            retained_preparation.1.entries.len()
                        );
                        for row in expected.seed_accounting() {
                            let SeedOriginRef::ExistingCore { seed } = row.origin else {
                                panic!("Core seed")
                            };
                            let entry = retained_preparation.1.entries.get(row.handoff).unwrap();
                            assert_eq!(
                                &entry.seed,
                                retained_core.obligation_seeds().get(seed).unwrap()
                            );
                            assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                            assert_eq!(entry.seed.status, ObligationSeedStatus::Active);
                            let SeedVcMapping::One { vc } = row.mapping else {
                                panic!("one VC")
                            };
                            let vc = expected.vc(vc).unwrap();
                            assert_eq!(vc.seed.handoff, row.handoff);
                            assert_eq!(vc.source.primary, entry.seed.source);
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
                            for _ in 0..2 {
                                assert_eq!(
                                    inputs
                                        .native_theorem_vcs_with_core_preparation_parents(
                                            &input,
                                            owner,
                                            &generation,
                                            &schema
                                        )
                                        .unwrap(),
                                    Some(expected.clone())
                                );
                            }
                        }
                        assert_eq!((generation, schema), schemas_before);
                        fixture.write(&[0xff]);
                    }
                    assert_eq!(
                        (
                            input,
                            (*owner.storage().get(&core).unwrap()).clone(),
                            (*owner.storage().get(&preparation).unwrap()).clone(),
                            session.captured().clone()
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 9);
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
fn native_theorem_vcs_core_preparation_caller_preserves_refusals() {
    use mizar_core::control_flow::{build_control_flow_ir, build_obligation_seed_handoff};
    use mizar_driver::{
        native_theorem_core_from_storage_bytes as decode_core,
        native_theorem_core_storage_bytes as encode_core,
        native_theorem_preparation_from_storage_bytes as decode,
        native_theorem_preparation_storage_bytes as encode,
    };
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let generation = GenerationSchemaVersion::new(" \n\t");
    let schema = VcSchemaVersion::new("");
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing-core",
        "missing-preparation",
        "wrong-core",
        "wrong-preparation",
        "hash",
        "dependencies",
        "link",
        "obsolete",
        "collected-core",
        "collected-preparation",
        "pair-mismatch",
        "source-priority",
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
                let actual_core = inputs
                    .native_theorem_core_with_metadata_parent(&metadata_input, owner)
                    .unwrap()
                    .unwrap();
                let actual_flow = build_control_flow_ir(&actual_core);
                let actual_pair = (
                    actual_flow.clone(),
                    build_obligation_seed_handoff(&actual_core, &actual_flow),
                );
                let foreign = if matches!(case, "pair-mismatch" | "source-priority") {
                    let changed = Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
                    let (_, other_submission, other_adapter, other_frontend, _) =
                        native_disk_frontend_record_submission(&changed, ids, snapshots, 0);
                    let (other_inputs, other_old) =
                        native_disk_lookup_context(&other_submission, ids);
                    let other_owner = other_adapter.publisher();
                    let other_input = PhaseInput::new(
                        other_old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            other_old.snapshot,
                            other_old.identities().input_hash(),
                            vec![],
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    other_owner,
                                    other_old.snapshot,
                                    other_frontend.erase(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    let other_core = other_inputs
                        .native_theorem_core_with_frontend_parent(&other_input, other_owner)
                        .unwrap()
                        .unwrap();
                    let output = other_owner.storage().get(&other_frontend).unwrap();
                    let ast = output.ast.as_ref().unwrap();
                    assert_ne!(ast.source_id, actual_core.source_id());
                    let module = actual_core.module_id();
                    let bytes = encode_core(ast, module, &other_core).unwrap().unwrap();
                    let core = decode_core(&bytes, actual_core.source_id(), module)
                        .unwrap()
                        .unwrap();
                    let rebound = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
                        &ast.canonical_bytes().unwrap(),
                        actual_core.source_id(),
                    )
                    .unwrap();
                    let flow = build_control_flow_ir(&core);
                    let pair = (flow.clone(), build_obligation_seed_handoff(&core, &flow));
                    assert_eq!(core.source_id(), actual_core.source_id());
                    assert_eq!(core.module_id(), actual_core.module_id());
                    assert_ne!(core, actual_core);
                    assert_ne!(pair, actual_pair);
                    let preparation_bytes =
                        encode(&rebound, module, &core, &pair).unwrap().unwrap();
                    assert_eq!(
                        decode(&preparation_bytes, actual_core.source_id(), module).unwrap(),
                        Some(pair.clone())
                    );
                    Some((core, bytes, pair, preparation_bytes))
                } else {
                    None
                };
                let core = if case == "source-priority" {
                    let (core, bytes, _, _) = foreign.as_ref().unwrap();
                    let source_id = core.source_id();
                    let module = core.module_id().clone();
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
                            payload: core.clone(),
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes.clone()),
                            decode: BlobDecoder::new(move |bytes| {
                                decode_core(bytes, source_id, &module)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| BlobDecodeError::new("negative Core replay"))
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
                let sole_core = child(vec![core.erase()]);
                let preparation = if matches!(case, "link" | "pair-mismatch" | "source-priority") {
                    let frontend = outputs
                        .iter()
                        .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                        .unwrap();
                    let handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend,
                            frontend.output_kind(),
                        )
                        .unwrap();
                    let output = owner.storage().get(&handle).unwrap();
                    let (pair, bytes) = if let Some((_, _, pair, bytes)) = &foreign {
                        (pair.clone(), bytes.clone())
                    } else {
                        (
                            actual_pair.clone(),
                            encode(
                                output.ast.as_ref().unwrap(),
                                actual_core.module_id(),
                                &actual_core,
                                &actual_pair,
                            )
                            .unwrap()
                            .unwrap(),
                        )
                    };
                    if case == "pair-mismatch" {
                        assert_eq!(
                            inputs
                                .native_theorem_preparation_with_core_parent(&sole_core, owner)
                                .unwrap(),
                            Some(actual_pair.clone())
                        );
                    } else if case == "source-priority" {
                        assert_eq!(
                            inputs
                                .native_theorem_preparation_with_core_parent(&sole_core, owner)
                                .unwrap(),
                            None
                        );
                    }
                    let source_id = actual_core.source_id();
                    let module = actual_core.module_id().clone();
                    let phase = IrPipelinePhase::new("AlgorithmPreparation");
                    let kind = OutputKind::new("SourceTheoremPreparation");
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
                            payload: pair.clone(),
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode(bytes, source_id, &module)
                                    .map_err(BlobDecodeError::new)?
                                    .ok_or_else(|| {
                                        BlobDecodeError::new("negative Preparation replay")
                                    })
                            }),
                            parents: vec![if case == "link" {
                                parent.erase()
                            } else {
                                core.erase()
                            }],
                            named_input_hashes: parent.lineage().named_input_hashes.clone(),
                            side_tables: IrSideTables::default(),
                            origin: OutputOrigin::PackageSource,
                            target: PublicationTarget::CurrentPackage,
                        })
                        .unwrap();
                    assert_eq!(*owner.storage().get(&negative).unwrap(), pair);
                    negative
                } else {
                    inputs
                        .publish_native_theorem_preparation(&sole_core, owner)
                        .unwrap()
                        .unwrap()
                };
                let parents = match case {
                    "missing-core" => vec![preparation.erase()],
                    "missing-preparation" => vec![core.erase()],
                    "wrong-core" => vec![parent.erase(), preparation.erase()],
                    "wrong-preparation" => vec![core.erase(), parent.erase()],
                    _ => vec![core.erase(), preparation.erase()],
                };
                let mut input = child(parents);
                if case == "hash" {
                    input = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            hash(0xe7),
                            vec![],
                            input.parent_outputs().to_vec(),
                        )
                        .unwrap(),
                    );
                }
                let upstream = if case == "dependencies" {
                    let [dual, sole] = [input.clone(), sole_core].map(|prior| {
                        PhaseInput::new(
                            prior.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                prior.snapshot,
                                prior.identities().input_hash(),
                                vec![hash(0xe8)],
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
                } else if matches!(case, "collected-core" | "collected-preparation") {
                    let protected = if case == "collected-core" {
                        preparation.output()
                    } else {
                        core.output()
                    };
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: old.snapshot,
                                protected_outputs: vec![protected]
                            })
                            .outputs_dropped,
                        8
                    );
                }
                let result = inputs.native_theorem_vcs_with_core_preparation_parents(
                    &input,
                    owner,
                    &generation,
                    &schema,
                );
                match case {
                    "source-priority" => assert_eq!(result.unwrap(), None),
                    "dependencies" => assert_eq!(result.unwrap_err(), upstream.unwrap()),
                    "pair-mismatch" => assert_eq!(
                        result.unwrap_err(),
                        "native theorem vc complete Preparation mismatch"
                    ),
                    "link" => assert_eq!(
                        result.unwrap_err(),
                        "native theorem vc Preparation parent link mismatch"
                    ),
                    _ => assert!(result.is_err(), "{case}"),
                }
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case.starts_with("collected-") { 1 } else { 9 }
                );
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}

#[test]
fn native_theorem_vcs_core_preparation_publication_preserves_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_core_semantic_payload as semantics,
        native_theorem_vc_storage_bytes as encode,
    };
    use mizar_ir::{
        publisher::PublishError,
        storage::{CollectInput, IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, SeedOriginRef, VcSchemaVersion, VcStatus},
    };
    let field = |hasher: &mut blake3::Hasher, bytes: &[u8]| {
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    };
    let variants = [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "generation",
            "vc",
        ),
        (
            "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            "generation",
            "vc",
        ),
        (
            "theorem T: for X being set holds X = X\nproof\n let X be set;\n thus X = X;\nend;",
            "generation",
            "vc",
        ),
        (
            "theorem Renamed: for X being set holds X = X proof let X be set; thus X = X; end;",
            "generation",
            "vc",
        ),
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            " \n\t",
            "",
        ),
    ];
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut representations = Vec::new();
        for (text, generation, schema) in variants {
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, old, owner, outputs, (session, ids, snapshots)| {
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
                    let child = |publisher, parents: Vec<AnyPhaseOutputRef>| {
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
                                            publisher,
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
                    let type_input = child(owner, vec![signature.erase()]);
                    let typed = inputs
                        .publish_native_theorem_typed_ast(&type_input, owner)
                        .unwrap()
                        .unwrap();
                    let reg_input = child(owner, vec![typed.erase()]);
                    let registration = inputs
                        .publish_native_theorem_registration_closure(&reg_input, owner)
                        .unwrap()
                        .unwrap();
                    let meta_input = child(owner, vec![typed.erase(), registration.erase()]);
                    let parent = inputs
                        .publish_native_theorem_metadata(&meta_input, owner)
                        .unwrap()
                        .unwrap();
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("VcGenerate"),
                        OutputKind::new("SourceTheoremVC"),
                        parent.work_unit().clone(),
                    ));
                    let core_input = child(owner, vec![parent.erase()]);
                    let core = inputs
                        .publish_native_theorem_core_with_metadata_parent(&core_input, owner)
                        .unwrap()
                        .unwrap();
                    let preparation_input = child(owner, vec![core.erase()]);
                    let preparation = inputs
                        .publish_native_theorem_preparation(&preparation_input, owner)
                        .unwrap()
                        .unwrap();
                    let input = child(owner, vec![core.erase(), preparation.erase()]);
                    let retained_core = owner.storage().get(&core).unwrap();
                    let retained_preparation = owner.storage().get(&preparation).unwrap();
                    let generation = GenerationSchemaVersion::new(generation);
                    let schema = VcSchemaVersion::new(schema);
                    let expected =
                        generate(&retained_core, old.snapshot, &generation, &schema).unwrap();
                    assert_eq!(
                        inputs
                            .native_theorem_vcs_with_core_preparation_parents(
                                &input,
                                owner,
                                &generation,
                                &schema
                            )
                            .unwrap(),
                        Some(expected.clone())
                    );
                    assert_eq!(
                        expected.source(),
                        inputs.snapshot.source_versions[0].source_id
                    );
                    assert_eq!(expected.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(expected.snapshot(), old.snapshot);
                    assert_eq!(expected.schema_version(), &schema);
                    assert!(
                        !expected.vcs().is_empty()
                            && !expected.seed_accounting().is_empty()
                            && !retained_core.proofs().is_empty()
                    );
                    assert!(expected.vcs().iter().all(|vc| vc.status == VcStatus::Open
                        && vc.anchor.generation_schema_version == generation));
                    assert!(
                        retained_core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    for row in expected.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!("Core seed")
                        };
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(
                            &retained_preparation
                                .1
                                .entries
                                .get(row.handoff)
                                .unwrap()
                                .seed,
                            retained_core.obligation_seeds().get(seed).unwrap()
                        );
                    }
                    let frontend = outputs
                        .iter()
                        .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                        .unwrap();
                    let handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend,
                            frontend.output_kind(),
                        )
                        .unwrap();
                    let output = owner.storage().get(&handle).unwrap();
                    let ast = output.ast.as_ref().unwrap();
                    let module = retained_core.module_id();
                    let opaque = semantics(ast, module, &retained_core).unwrap().unwrap();
                    let bytes = encode(
                        ast,
                        module,
                        &retained_core,
                        &expected,
                        old.snapshot,
                        &generation,
                    )
                    .unwrap()
                    .unwrap();
                    let mut canonical = b"mizar-driver/native-theorem-vc-content/v1\0".to_vec();
                    for value in [
                        generation.as_str().as_bytes(),
                        schema.as_str().as_bytes(),
                        opaque.as_slice(),
                    ] {
                        canonical.extend_from_slice(&(value.len() as u64).to_le_bytes());
                        canonical.extend_from_slice(value);
                    }
                    let version = &inputs.snapshot.source_versions[0];
                    let mut content = blake3::Hasher::new();
                    field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                    field(&mut content, &canonical);
                    content.update(&2_u64.to_le_bytes());
                    let mut parents = [core.erase(), preparation.erase()];
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
                    for parent in parents {
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
                    let digest = Hash::from_bytes(blake3::derive_key(
                        "mizar-driver/native-theorem-vc-storage-side-table/v1",
                        &bytes,
                    ));
                    let tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "generate-theorem-vc-storage",
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
                            field(&mut side, b"generate-theorem-vc-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    let before = (
                        input.clone(),
                        (*retained_core).clone(),
                        (*retained_preparation).clone(),
                        generation.clone(),
                        schema.clone(),
                        session.captured().clone(),
                    );
                    let sealed = inputs
                        .publish_native_theorem_vcs_with_core_preparation_parents(
                            &input,
                            owner,
                            &generation,
                            &schema,
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), old.snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("VcGenerate"));
                    assert_eq!(sealed.output_kind(), &OutputKind::new("SourceTheoremVC"));
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), core.work_unit());
                    assert_eq!(sealed.lineage().parents.len(), 2);
                    assert!(
                        sealed.lineage().parents.contains(&core.output())
                            && sealed.lineage().parents.contains(&preparation.output())
                    );
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        core.lineage().named_input_hashes
                    );
                    assert_eq!(sealed.content_hash(), expected_content);
                    assert_eq!(sealed.side_table_hash(), expected_side);
                    assert_eq!(*owner.storage().side_tables(&sealed).unwrap(), tables);
                    match sealed.placement() {
                        StoragePlacement::Resident => assert!(!open),
                        StoragePlacement::Blob { blob, len } => {
                            assert!(open);
                            assert_eq!(blob.hash(), expected_blob);
                            assert_eq!(*len, bytes.len());
                        }
                        _ => panic!("unexpected placement"),
                    }
                    if representations.is_empty() {
                        let independent = publisher(old.snapshot, threshold, &old.work_unit);
                        allow_frontend(&independent, &old.work_unit);
                        independent.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new("VcGenerate"),
                            OutputKind::new("SourceTheoremVC"),
                            core.work_unit().clone(),
                        ));
                        let mut builder = PhaseRegistryBuilder::new();
                        builder.register_source_load().register_frontend(vec![]);
                        let (_, submit) = fixture.submit_request(vec![], vec![]);
                        let submission = CompilerDriver::new(builder.build().unwrap())
                            .with_output_publisher(independent.clone())
                            .submit_captured(session.clone(), ids, snapshots, submit)
                            .unwrap();
                        let frontend = submission
                            .scheduler_run
                            .as_ref()
                            .unwrap()
                            .phase_results
                            .values()
                            .flatten()
                            .flat_map(|result| &result.output_refs)
                            .find(|output| {
                                output.output_kind() == &OutputKind::new("FrontendOutput")
                            })
                            .unwrap();
                        let old_input = child(&independent, vec![frontend.clone()]);
                        let old_sealed = inputs
                            .publish_native_theorem_vcs(
                                &old_input,
                                &independent,
                                &generation,
                                &schema,
                            )
                            .unwrap()
                            .unwrap();
                        assert_eq!(*independent.storage().get(&old_sealed).unwrap(), expected);
                        assert_eq!(old_sealed.lineage().parents, vec![frontend.output()]);
                        assert_eq!(old_sealed.placement(), sealed.placement());
                        assert_eq!(
                            *independent.storage().side_tables(&old_sealed).unwrap(),
                            tables
                        );
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
                                .publish_native_theorem_vcs_with_core_preparation_parents(
                                    &input,
                                    owner,
                                    &generation,
                                    &schema
                                )
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
                            (*owner.storage().get(&core).unwrap()).clone(),
                            (*owner.storage().get(&preparation).unwrap()).clone(),
                            generation,
                            schema,
                            session.captured().clone()
                        ),
                        before
                    );
                    let collected = owner.storage().collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 10);
                    assert_eq!(collected.abandoned_slots_dropped, 3);
                    representations.push((canonical, bytes));
                },
            );
        }
        assert_eq!(representations[0].0, representations[1].0);
        assert_eq!(representations[0].0, representations[2].0);
        assert_ne!(representations[0].0, representations[3].0);
        assert_ne!(representations[0].0, representations[4].0);
        assert_ne!(representations[0].1, representations[1].1);
        assert_ne!(representations[0].1, representations[2].1);
    }
}

#[test]
fn native_theorem_vcs_core_preparation_publication_preserves_refusals() {
    use mizar_core::control_flow::{build_control_flow_ir, build_obligation_seed_handoff};
    use mizar_driver::{
        native_theorem_core_from_storage_bytes as decode_core,
        native_theorem_core_storage_bytes as encode_core,
        native_theorem_preparation_from_storage_bytes as decode,
        native_theorem_preparation_storage_bytes as encode,
    };
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishError, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let generation = GenerationSchemaVersion::new(" \n\t");
    for case in [
        "missing",
        "obsolete",
        "denied",
        "oversized",
        "lemma",
        "source-none",
    ] {
        let lemma = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        let text = if case == "lemma" {
            lemma
        } else {
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;"
        };
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
                if case != "denied" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("VcGenerate"),
                        OutputKind::new("SourceTheoremVC"),
                        parent.work_unit().clone(),
                    ));
                }
                let core_input = child(vec![parent.erase()]);
                let (core, preparation, semantic_error) = if matches!(case, "lemma" | "source-none")
                {
                    let frontend = outputs
                        .iter()
                        .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                        .unwrap();
                    let handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend,
                            frontend.output_kind(),
                        )
                        .unwrap();
                    let output = owner.storage().get(&handle).unwrap();
                    let mut ast = output.ast.as_ref().unwrap().clone();
                    let mut core = inputs
                        .native_theorem_core_with_metadata_parent(&core_input, owner)
                        .unwrap()
                        .unwrap();
                    if case == "source-none" {
                        let other = Fixture::new(lemma.as_bytes());
                        let (_, submission, adapter, frontend, _) =
                            native_disk_frontend_record_submission(&other, ids, snapshots, 0);
                        let (other_inputs, old) = native_disk_lookup_context(&submission, ids);
                        let other_owner = adapter.publisher();
                        let input = PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                vec![
                                    SealedParentOutputHandle::from_published_output(
                                        other_owner,
                                        old.snapshot,
                                        frontend.erase(),
                                    )
                                    .unwrap(),
                                ],
                            )
                            .unwrap(),
                        );
                        let original = other_inputs
                            .native_theorem_core_with_frontend_parent(&input, other_owner)
                            .unwrap()
                            .unwrap();
                        let output = other_owner.storage().get(&frontend).unwrap();
                        let other_ast = output.ast.as_ref().unwrap();
                        assert_ne!(original.source_id(), ast.source_id);
                        core = original;
                        ast = other_ast.clone();
                    }
                    let module = core.module_id().clone();
                    let source_id = ast.source_id;
                    let semantic_error =
                        mizar_driver::native_theorem_core_semantic_payload(&ast, &module, &core)
                            .unwrap_err();
                    let flow = build_control_flow_ir(&core);
                    let pair = (flow.clone(), build_obligation_seed_handoff(&core, &flow));
                    let core_bytes = encode_core(&ast, &module, &core).unwrap().unwrap();
                    let bytes = encode(&ast, &module, &core, &pair).unwrap().unwrap();
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
                            payload: pair,
                            canonical_payload: Some(bytes.clone()),
                            storage_payload: Some(bytes),
                            decode: BlobDecoder::new(move |bytes| {
                                decode(bytes, source_id, &module)
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
                    (core, preparation, Some(semantic_error))
                } else {
                    let core = inputs
                        .publish_native_theorem_core_with_metadata_parent(&core_input, owner)
                        .unwrap()
                        .unwrap();
                    let sole = child(vec![core.erase()]);
                    let preparation = inputs
                        .publish_native_theorem_preparation(&sole, owner)
                        .unwrap()
                        .unwrap();
                    (core, preparation, None)
                };
                let input = child(if case == "missing" {
                    vec![core.erase()]
                } else {
                    vec![core.erase(), preparation.erase()]
                });
                if case == "obsolete" {
                    owner.mark_obsolete(old.snapshot).unwrap();
                }
                let schema = VcSchemaVersion::new(if case == "oversized" {
                    "x".repeat(32 * 1024 * 1024)
                } else {
                    String::new()
                });
                let raw = inputs.native_theorem_vcs_with_core_preparation_parents(
                    &input,
                    owner,
                    &generation,
                    &schema,
                );
                let expected = match case {
                    "source-none" => {
                        assert_eq!(raw.unwrap(), None);
                        Ok(false)
                    }
                    "lemma" => {
                        assert!(raw.unwrap().is_some());
                        Err(semantic_error.unwrap())
                    }
                    "oversized" => {
                        let graph = raw.unwrap().unwrap();
                        assert_eq!(graph.schema_version(), &schema);
                        assert!(!graph.vcs().is_empty());
                        drop(graph);
                        Ok(false)
                    }
                    "denied" => {
                        assert!(raw.unwrap().is_some());
                        Err(PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("VcGenerate"),
                            work_unit: parent.work_unit().clone(),
                            output_kind: OutputKind::new("SourceTheoremVC"),
                        }
                        .to_string())
                    }
                    _ => Err(raw.unwrap_err()),
                };
                let actual = inputs
                    .publish_native_theorem_vcs_with_core_preparation_parents(
                        &input,
                        owner,
                        &generation,
                        &schema,
                    )
                    .map(|output| output.is_some());
                assert_eq!(actual, expected, "{case}");
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 9);
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(case == "denied")
                );
            },
        );
    }
}
