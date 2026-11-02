use super::*;

#[test]
fn native_theorem_discharge_core_vc_caller_matches_genuine_discharge() {
    use mizar_core::control_flow::ObligationHandoffOrigin;
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_vc::{
        discharge::{
            ComputationLimit, DischargeExplanationCategory, DischargeInput, DischargePolicy,
            try_discharge,
        },
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcProvenancePhase, VcSchemaVersion, VcStatus},
    };
    for (open, threshold, generation, schema) in
        [(false, usize::MAX, " λ\n\0", ""), (true, 0, "", "核\t\0")]
    {
        let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
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
                let typed = inputs
                    .publish_native_theorem_typed_ast(&child(vec![signature.erase()]), owner)
                    .unwrap()
                    .unwrap();
                let registration = inputs
                    .publish_native_theorem_registration_closure(&child(vec![typed.erase()]), owner)
                    .unwrap()
                    .unwrap();
                let metadata_parent = inputs
                    .publish_native_theorem_metadata(
                        &child(vec![typed.erase(), registration.erase()]),
                        owner,
                    )
                    .unwrap()
                    .unwrap();
                let core_parent = inputs
                    .publish_native_theorem_core_with_metadata_parent(
                        &child(vec![metadata_parent.erase()]),
                        owner,
                    )
                    .unwrap()
                    .unwrap();
                let preparation = inputs
                    .publish_native_theorem_preparation(&child(vec![core_parent.erase()]), owner)
                    .unwrap()
                    .unwrap();
                let generation = GenerationSchemaVersion::new(generation);
                let schema = VcSchemaVersion::new(schema);
                let vc_parent = inputs
                    .publish_native_theorem_vcs_with_core_preparation_parents(
                        &child(vec![core_parent.erase(), preparation.erase()]),
                        owner,
                        &generation,
                        &schema,
                    )
                    .unwrap()
                    .unwrap();
                let input = child(vec![vc_parent.erase(), core_parent.erase()]);
                let core = owner.storage().get(&core_parent).unwrap();
                let vcs = owner.storage().get(&vc_parent).unwrap();
                let selected = vcs.vcs()[0].id;
                let handoff = inputs
                    .native_theorem_kernel_handoff_with_core_vc_parents(&input, owner, selected)
                    .unwrap()
                    .unwrap();
                let preparation_value = owner.storage().get(&preparation).unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                assert_eq!(
                    core.source_id(),
                    inputs.snapshot.source_versions[0].source_id
                );
                assert_eq!(core.module_id(), &module);
                assert_eq!(vcs.source(), inputs.snapshot.source_versions[0].source_id);
                assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                assert_eq!(vcs.snapshot(), input.snapshot);
                assert_eq!(vcs.schema_version(), &schema);
                assert!(
                    !core.proofs().is_empty()
                        && !vcs.vcs().is_empty()
                        && !vcs.seed_accounting().is_empty()
                );
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open
                    && vc.anchor.generation_schema_version == generation));
                assert!(!preparation_value.1.entries.is_empty());
                for (id, entry) in preparation_value.1.entries.iter() {
                    let ObligationHandoffOrigin::ExistingCore { seed } = entry.origin else {
                        panic!("original Core seed")
                    };
                    assert_eq!(&entry.seed, core.obligation_seeds().get(seed).unwrap());
                    assert_eq!(entry.seed.status, ObligationSeedStatus::Active);
                    assert_eq!(
                        preparation_value.1.source_map.get(&id),
                        core.source_map().obligation_sources.get(&seed)
                    );
                    assert_eq!(
                        vcs.seed_accounting_for(id).unwrap().seed_status,
                        ObligationSeedStatus::Active
                    );
                }
                assert_eq!(handoff.diagnostics().vc, selected);
                let before = (
                    input.clone(),
                    (*core).clone(),
                    (*vcs).clone(),
                    (*preparation_value).clone(),
                    session.captured().clone(),
                );
                for (key, max_steps) in [("計算\n\0", 0), ("", u32::MAX)] {
                    let policy = DischargePolicy {
                        computation_limit: ComputationLimit {
                            policy: PolicyKey::new(key),
                            max_steps,
                        },
                    };
                    let policy_before = policy.clone();
                    let expected = try_discharge(DischargeInput {
                        vc_set: &vcs,
                        policy: &policy,
                    })
                    .unwrap();
                    let processed = expected.vc_set();
                    assert_eq!(processed.source(), vcs.source());
                    assert_eq!(processed.module(), vcs.module());
                    assert_eq!(processed.snapshot(), input.snapshot);
                    assert_eq!(processed.schema_version(), &schema);
                    assert_eq!(processed.seed_accounting(), vcs.seed_accounting());
                    assert!(expected.evidence_records().is_empty());
                    let [explanation] = expected.explanations() else {
                        panic!("one actual theorem explanation")
                    };
                    assert_eq!(explanation.vc, selected);
                    assert_eq!(
                        explanation.category,
                        DischargeExplanationCategory::UnsupportedRule
                    );
                    assert_eq!(explanation.rule, None);
                    assert_eq!(
                        explanation.detail.as_str(),
                        "no task-11 deterministic discharge rule matched"
                    );
                    let [vc] = processed.vcs() else {
                        panic!("one processed theorem VC")
                    };
                    assert_eq!(vc.status, VcStatus::NeedsAtp);
                    let provenance = vc.provenance.last().unwrap();
                    assert_eq!(provenance.phase, VcProvenancePhase::Discharge);
                    assert_eq!(
                        provenance.key.as_str(),
                        "task-11-discharge:unsupported-rule"
                    );
                    assert_eq!(provenance.core, None);
                    assert_eq!(
                        &vc.provenance[..vc.provenance.len() - 1],
                        vcs.vcs()[0].provenance.as_slice()
                    );
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(if open {
                                &[0xff]
                            } else {
                                b"open theorem Changed: 0 = 1;\n"
                            });
                        }
                        if stage == 2 && fixture.root.join("alpha/src/main.miz").exists() {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        for _ in 0..2 {
                            assert_eq!(
                                inputs
                                    .native_theorem_discharge_with_core_vc_parents(
                                        &input, owner, selected, &policy
                                    )
                                    .unwrap(),
                                Some(expected.clone())
                            );
                        }
                    }
                    assert_eq!(policy, policy_before);
                }
                assert_eq!(
                    core_parent.placement() == &StoragePlacement::Resident,
                    !open
                );
                assert_eq!(vc_parent.placement() == &StoragePlacement::Resident, !open);
                assert_eq!(
                    (
                        input,
                        (*owner.storage().get(&core_parent).unwrap()).clone(),
                        (*owner.storage().get(&vc_parent).unwrap()).clone(),
                        (*owner.storage().get(&preparation).unwrap()).clone(),
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
    }
}

#[test]
fn native_theorem_discharge_core_vc_caller_preserves_refusals() {
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
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let generation = GenerationSchemaVersion::new(" λ\n\0");
    let schema = VcSchemaVersion::new("");
    let policy = mizar_vc::discharge::DischargePolicy {
        computation_limit: mizar_vc::discharge::ComputationLimit {
            policy: mizar_vc::vc_ir::PolicyKey::new("拒否\n"),
            max_steps: 0,
        },
    };
    for case in [
        "obsolete",
        "collected-vc",
        "dependencies",
        "absent",
        "source-none",
    ] {
        with_native_summary_parent(
            text,
            false,
            0,
            |_, inputs, old, owner, _, (_, ids, snapshots)| {
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
                let (core_parent, preparation, foreign_ast) = if case == "source-none" {
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
                    assert_ne!(ast.source_id, inputs.snapshot.source_versions[0].source_id);
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
                let vc_parent = if case == "source-none" {
                    let ast = foreign_ast.unwrap();
                    let snapshot = old.snapshot;
                    let graph = generate(&core, snapshot, &generation, &schema).unwrap();
                    assert!(
                        mizar_vc::kernel_evidence_handoff::build_source_theorem_kernel_handoff(
                            &core,
                            &graph,
                            graph.vcs()[0].id
                        )
                        .is_ok()
                    );
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
                let mut input = child(vec![core_parent.erase(), vc_parent.erase()]);
                if case != "source-none" {
                    assert!(
                        inputs
                            .native_theorem_kernel_handoff_with_core_vc_parents(
                                &input, owner, valid
                            )
                            .unwrap()
                            .is_some()
                    );
                    let baseline =
                        mizar_vc::discharge::try_discharge(mizar_vc::discharge::DischargeInput {
                            vc_set: &vcs,
                            policy: &policy,
                        })
                        .unwrap();
                    assert_eq!(
                        baseline.vc_set().vcs()[0].status,
                        mizar_vc::vc_ir::VcStatus::NeedsAtp
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
                } else if case == "collected-vc" {
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: old.snapshot,
                                protected_outputs: vec![core_parent.output()]
                            })
                            .outputs_dropped,
                        9
                    );
                }
                let before = (input.clone(), policy.clone());
                let raw = inputs
                    .native_theorem_kernel_handoff_with_core_vc_parents(&input, owner, selected);
                if case == "source-none" {
                    let baseline =
                        mizar_vc::discharge::try_discharge(mizar_vc::discharge::DischargeInput {
                            vc_set: &vcs,
                            policy: &policy,
                        })
                        .unwrap();
                    assert_eq!(baseline.vc_set().source(), core.source_id());
                    assert_eq!(
                        baseline.vc_set().vcs()[0].status,
                        mizar_vc::vc_ir::VcStatus::NeedsAtp
                    );
                    assert!(raw.unwrap().is_none());
                    assert_eq!(
                        inputs
                            .native_theorem_discharge_with_core_vc_parents(
                                &input, owner, selected, &policy
                            )
                            .unwrap(),
                        None
                    );
                    assert_eq!(
                        inputs
                            .publish_native_theorem_discharge(&input, owner, selected, &policy)
                            .unwrap(),
                        None
                    );
                } else {
                    let expected = raw.unwrap_err();
                    if case == "dependencies" {
                        assert_eq!(expected, upstream.unwrap());
                    }
                    assert_eq!(
                        inputs
                            .native_theorem_discharge_with_core_vc_parents(
                                &input, owner, selected, &policy
                            )
                            .unwrap_err(),
                        expected
                    );
                    assert_eq!(
                        inputs
                            .publish_native_theorem_discharge(&input, owner, selected, &policy)
                            .unwrap_err(),
                        expected
                    );
                }
                assert_eq!((input, policy.clone()), before);
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected-vc" { 1 } else { 10 }
                );
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}

#[test]
fn native_theorem_discharge_storage_replays_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_discharge_from_storage_bytes as decode,
        native_theorem_discharge_storage_bytes as encode,
        native_theorem_vc_from_storage_bytes as old_decode,
        native_theorem_vc_storage_bytes as old_encode,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcProvenancePhase, VcSchemaVersion, VcStatus},
    };
    let singleton = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let multiple = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (text, open, threshold, generation, schema, key, max_steps, rows) in [
        (singleton, false, usize::MAX, " λ\n\0", "", "計算\0", 0, 1),
        (multiple, true, 0, "", "核\t\0", "", u32::MAX, 2),
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
        let owner = publisher(snapshot, threshold, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        let submission = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(owner.clone())
            .submit_captured(session, &ids, &snapshots, submit)
            .unwrap();
        let (inputs, original) = native_disk_lookup_context(&submission, &ids);
        let parent = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
            .unwrap();
        let input = PhaseInput::new(
            unit,
            PhaseDispatchInputBundle::new(
                snapshot,
                original.identities().input_hash(),
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
            .native_theorem_core_with_frontend_parent(&input, &owner)
            .unwrap()
            .unwrap();
        let parent = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                parent.output_kind(),
            )
            .unwrap();
        let frontend = owner.storage().get(&parent).unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
        let generation = GenerationSchemaVersion::new(generation);
        let schema = VcSchemaVersion::new(schema);
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(key),
                max_steps,
            },
        };
        let vcs = generate(&core, snapshot, &generation, &schema).unwrap();
        let original_bytes = old_encode(ast, &module, &core, &vcs, snapshot, &generation)
            .unwrap()
            .unwrap();
        let discharge_input = DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        };
        let expected = try_discharge(discharge_input).unwrap();
        let mut manual = b"mizar-driver/native-theorem-discharge-storage/v1\0".to_vec();
        for field in [
            original_bytes.as_slice(),
            key.as_bytes(),
            max_steps.to_le_bytes().as_slice(),
        ] {
            manual.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            manual.extend_from_slice(field);
        }
        assert_eq!(core.source_id(), ast.source_id);
        assert_eq!(core.module_id(), &module);
        assert_eq!(vcs.source(), ast.source_id);
        assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
        assert_eq!(vcs.snapshot(), snapshot);
        assert_eq!(vcs.schema_version(), &schema);
        assert_eq!(vcs.vcs().len(), rows);
        assert_eq!(core.proofs().len(), rows);
        assert!(
            core.proofs()
                .iter()
                .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
        );
        assert_eq!(core.obligation_seeds().len(), rows);
        assert!(
            core.obligation_seeds()
                .iter()
                .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
        );
        assert!(
            vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open
                && vc.anchor.generation_schema_version == generation)
        );
        assert_eq!(expected.vc_set().vcs().len(), rows);
        assert_eq!(expected.vc_set().seed_accounting(), vcs.seed_accounting());
        assert_eq!(expected.explanations().len(), rows);
        assert!(expected.evidence_records().is_empty());
        for (processed, original) in expected.vc_set().vcs().iter().zip(vcs.vcs()) {
            assert_eq!(processed.id, original.id);
            assert_eq!(processed.status, VcStatus::NeedsAtp);
            assert_eq!(processed.goal, original.goal);
            assert_eq!(processed.local_context, original.local_context);
            assert_eq!(
                processed.provenance.last().unwrap().phase,
                VcProvenancePhase::Discharge
            );
            assert!(
                expected
                    .explanations()
                    .iter()
                    .any(|explanation| explanation.vc == original.id)
            );
        }
        assert_eq!(parent.placement() == &StoragePlacement::Resident, !open);
        let before = (
            input.clone(),
            (*frontend).clone(),
            core.clone(),
            vcs.clone(),
            policy.clone(),
            submission.session.captured().clone(),
        );
        let other = Fixture::new(singleton.as_bytes());
        let (_, other_submission, _, _, _) =
            native_disk_frontend_record_submission(&other, &ids, &snapshots, threshold);
        let other_source = other_submission
            .session
            .captured()
            .snapshot()
            .source_versions[0]
            .source_id;
        let other_snapshot = other_submission.session.captured().snapshot().id;
        assert_ne!(other_source, ast.source_id);
        assert_ne!(other_snapshot, snapshot);
        let rebound = old_decode(&original_bytes, other_source, &module, other_snapshot)
            .unwrap()
            .unwrap();
        let rebound_expected = try_discharge(DischargeInput {
            vc_set: &rebound,
            policy: &policy,
        })
        .unwrap();
        assert_eq!(rebound.source(), other_source);
        assert_eq!(rebound.snapshot(), other_snapshot);
        assert_eq!(
            decode(&manual, other_source, &module, other_snapshot, &policy).unwrap(),
            Some(rebound_expected)
        );
        for stage in 0..3 {
            if stage == 1 {
                fixture.write(if open {
                    &[0xff]
                } else {
                    b"open theorem Changed: 0 = 1;"
                });
            }
            if stage == 2 {
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            }
            for _ in 0..2 {
                assert_eq!(
                    encode(ast, &module, &core, discharge_input, &expected, &generation).unwrap(),
                    Some(manual.clone())
                );
                assert_eq!(
                    decode(&manual, ast.source_id, &module, snapshot, &policy).unwrap(),
                    Some(expected.clone())
                );
            }
        }
        assert_eq!(
            (
                input,
                (*frontend).clone(),
                core,
                vcs,
                policy,
                submission.session.captured().clone()
            ),
            before
        );
    }
}

#[test]
fn native_theorem_discharge_storage_preserves_refusals() {
    use mizar_driver::{
        native_theorem_discharge_from_storage_bytes as decode,
        native_theorem_discharge_storage_bytes as encode,
        native_theorem_vc_from_storage_bytes as old_decode,
        native_theorem_vc_storage_bytes as old_encode,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new(""),
            max_steps: 0,
        },
    };
    let mut products = Vec::new();
    for text in [
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        "theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, submission, adapter, frontend, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, original) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let input = PhaseInput::new(
            original.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                original.snapshot,
                original.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        original.snapshot,
                        frontend.erase(),
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
        let output = owner.storage().get(&frontend).unwrap();
        let ast = output.ast.as_ref().unwrap().clone();
        let vcs = generate(&core, original.snapshot, &generation, &schema).unwrap();
        let old = old_encode(&ast, &module, &core, &vcs, original.snapshot, &generation)
            .unwrap()
            .unwrap();
        let result = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        })
        .unwrap();
        let bytes = encode(
            &ast,
            &module,
            &core,
            DischargeInput {
                vc_set: &vcs,
                policy: &policy,
            },
            &result,
            &generation,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            decode(&bytes, ast.source_id, &module, original.snapshot, &policy).unwrap(),
            Some(result.clone())
        );
        products.push((ast, core, vcs, result, old, bytes));
    }
    let [
        (ast, core, vcs, result, old, bytes),
        (other_ast, other_core, other_vcs, other_result, _, _),
    ] = products.as_slice()
    else {
        panic!("two genuine source products")
    };
    assert_ne!(ast.source_id, other_ast.source_id);
    assert_ne!(result, other_result);
    let input = DischargeInput {
        vc_set: vcs,
        policy: &policy,
    };
    assert_eq!(
        encode(ast, &module, core, input, other_result, &generation),
        Ok(None)
    );
    let other_input = DischargeInput {
        vc_set: other_vcs,
        policy: &policy,
    };
    let old_foreign = old_encode(
        ast,
        &module,
        other_core,
        other_vcs,
        other_vcs.snapshot(),
        &generation,
    );
    assert_eq!(old_foreign, Ok(None));
    assert_eq!(
        encode(
            ast,
            &module,
            other_core,
            other_input,
            other_result,
            &generation
        ),
        old_foreign
    );
    let wrong_generation = GenerationSchemaVersion::new("other-generation");
    assert!(!vcs.vcs().is_empty());
    assert_eq!(
        old_encode(ast, &module, core, vcs, vcs.snapshot(), &wrong_generation),
        Ok(None)
    );
    assert_eq!(
        encode(ast, &module, core, input, result, &wrong_generation),
        Ok(None)
    );
    let domain = b"mizar-driver/native-theorem-discharge-storage/v1\0";
    let frame = |fields: [&[u8]; 3]| {
        let mut bytes = domain.to_vec();
        for field in fields {
            bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(field);
        }
        bytes
    };
    assert_eq!(*bytes, frame([old.as_slice(), b"", &0u32.to_le_bytes()]));
    for other_policy in [
        DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new("別\0"),
                max_steps: 0,
            },
        },
        DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(""),
                max_steps: u32::MAX,
            },
        },
    ] {
        let actual = try_discharge(DischargeInput {
            vc_set: vcs,
            policy: &other_policy,
        })
        .unwrap();
        assert_eq!(&actual, result);
        let altered = frame([
            old.as_slice(),
            other_policy.computation_limit.policy.as_str().as_bytes(),
            &other_policy.computation_limit.max_steps.to_le_bytes(),
        ]);
        assert_eq!(
            decode(&altered, ast.source_id, &module, vcs.snapshot(), &policy),
            Ok(None)
        );
        assert_eq!(
            decode(bytes, ast.source_id, &module, vcs.snapshot(), &other_policy),
            Ok(None)
        );
        assert_eq!(
            decode(
                &altered,
                ast.source_id,
                &module,
                vcs.snapshot(),
                &other_policy
            )
            .unwrap(),
            Some(actual)
        );
        assert_eq!(
            encode(
                ast,
                &module,
                core,
                DischargeInput {
                    vc_set: vcs,
                    policy: &other_policy
                },
                result,
                &generation
            )
            .unwrap(),
            Some(altered)
        );
    }
    let mut corruptions = vec![
        Vec::new(),
        bytes[..domain.len() + 3].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
    ];
    let mut wrong_domain = bytes.clone();
    wrong_domain[0] = b'x';
    corruptions.push(wrong_domain);
    let mut wrong_version = bytes.clone();
    wrong_version[domain.len() - 2] = b'2';
    corruptions.push(wrong_version);
    let mut overflow = bytes.clone();
    overflow[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    corruptions.push(overflow);
    let mut trailing = bytes.clone();
    trailing.push(0);
    corruptions.push(trailing);
    for fields in [
        [old.as_slice(), &[0xff][..], &[0; 4][..]],
        [old.as_slice(), b"", &[0; 3][..]],
    ] {
        corruptions.push(frame(fields));
    }
    let mut oversized = bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(
            decode(&malformed, ast.source_id, &module, vcs.snapshot(), &policy),
            Ok(None)
        );
    }
    let wrong_module = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("other"));
    let old_wrong = old_decode(old, ast.source_id, &wrong_module, vcs.snapshot());
    assert_eq!(old_wrong, Ok(None));
    assert_eq!(
        decode(bytes, ast.source_id, &wrong_module, vcs.snapshot(), &policy),
        old_wrong.map(|_| None)
    );
    let malformed_old = b"invalid original VC stream";
    let old_refused = old_decode(malformed_old, ast.source_id, &module, vcs.snapshot());
    assert_eq!(old_refused, Ok(None));
    let malformed = frame([malformed_old, b"mismatched-key", &u32::MAX.to_le_bytes()]);
    assert_eq!(
        decode(&malformed, ast.source_id, &module, vcs.snapshot(), &policy),
        old_refused.map(|_| None)
    );
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let oversized_policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("x".repeat(32 * 1024 * 1024)),
            max_steps: 0,
        },
    };
    assert_eq!(
        try_discharge(DischargeInput {
            vc_set: vcs,
            policy: &oversized_policy
        })
        .unwrap(),
        *result
    );
    assert_eq!(
        encode(
            ast,
            &module,
            core,
            DischargeInput {
                vc_set: vcs,
                policy: &oversized_policy
            },
            result,
            &generation
        ),
        Ok(None)
    );
    for (text, is_none) in [
        (format!("{private}\nreserve Y for set;\n"), true),
        (private.replace("set", "object"), false),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, _, adapter, frontend, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let output = adapter.publisher().storage().get(&frontend).unwrap();
        let refused_ast = output.ast.as_ref().unwrap();
        let prior = old_encode(refused_ast, &module, core, vcs, vcs.snapshot(), &generation);
        if is_none {
            assert_eq!(prior, Ok(None));
        } else {
            assert!(prior.is_err());
        }
        assert_eq!(
            encode(
                refused_ast,
                &module,
                core,
                DischargeInput {
                    vc_set: vcs,
                    policy: &oversized_policy
                },
                other_result,
                &generation
            ),
            prior
        );
    }
}

#[test]
fn native_theorem_discharge_publication_preserves_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_core_semantic_payload as semantics,
        native_theorem_discharge_storage_bytes as encode,
    };
    use mizar_ir::{
        publisher::PublishError,
        storage::{CollectInput, IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    use mizar_vc::{
        discharge::{
            ComputationLimit, DischargeExplanationCategory, DischargeInput, DischargePolicy,
            try_discharge,
        },
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcProvenancePhase, VcSchemaVersion,
            VcStatus,
        },
    };
    let field = |hasher: &mut blake3::Hasher, value: &[u8]| {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value);
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let mut representations = Vec::new();
                for (generation, schema, key, max_steps) in [
                    ("λ\n\0", "", "", 0),
                    ("λ\n\0", "", "計算\0", 0),
                    ("λ\n\0", "", "", u32::MAX),
                    ("別\t", "", "", 0),
                    ("λ\n\0", "核\0", "", 0),
                ] {
                    let owner = publisher(old.snapshot, threshold, &old.work_unit);
                    allow_frontend(&owner, &old.work_unit);
                    for (phase, kind) in [
                        ("ModuleResolve", "SourceTheoremModuleCandidates"),
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                        ("Elaboration", "SourceTheoremCore"),
                        ("AlgorithmPreparation", "SourceTheoremPreparation"),
                        ("VcGenerate", "SourceTheoremVC"),
                        ("VcDischarge", "SourceTheoremDischarge"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                    let generation = GenerationSchemaVersion::new(generation);
                    let schema = VcSchemaVersion::new(schema);
                    let policy = DischargePolicy {
                        computation_limit: ComputationLimit {
                            policy: PolicyKey::new(key),
                            max_steps,
                        },
                    };
                    let mut builder = PhaseRegistryBuilder::new();
                    builder
                        .register_source_load()
                        .register_frontend(vec![])
                        .register_source_theorem_module_resolver()
                        .register_source_theorem_signature_collection()
                        .register_source_theorem_type_checking()
                        .register_source_theorem_registration_resolution()
                        .register_source_theorem_overload_resolution()
                        .register_source_theorem_elaboration()
                        .register_source_theorem_algorithm_preparation()
                        .register_source_theorem_vc_generation(generation.clone(), schema.clone());
                    let (_, submit) = fixture.submit_request(vec![], vec![]);
                    let submission = CompilerDriver::new(builder.build().unwrap())
                        .with_output_publisher(owner.clone())
                        .submit_captured(session.clone(), ids, snapshots, submit)
                        .unwrap();
                    let (inputs, original) = native_disk_lookup_context(&submission, ids);
                    let outputs = submission
                        .scheduler_run
                        .as_ref()
                        .unwrap()
                        .phase_results
                        .values()
                        .flatten()
                        .flat_map(|result| &result.output_refs)
                        .collect::<Vec<_>>();
                    assert_eq!(outputs.len(), 10);
                    let core_ref = outputs
                        .iter()
                        .find(|output| {
                            output.output_kind() == &OutputKind::new("SourceTheoremCore")
                        })
                        .unwrap();
                    let vc_ref = outputs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremVC"))
                        .unwrap();
                    let frontend_ref = outputs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
                        .unwrap();
                    let core_handle = owner
                        .storage()
                        .typed_handle::<mizar_core::core_ir::CoreIr>(
                            core_ref,
                            core_ref.output_kind(),
                        )
                        .unwrap();
                    let vc_handle = owner
                        .storage()
                        .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, vc_ref.output_kind())
                        .unwrap();
                    let frontend_handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend_ref,
                            frontend_ref.output_kind(),
                        )
                        .unwrap();
                    let core = owner.storage().get(&core_handle).unwrap();
                    let vcs = owner.storage().get(&vc_handle).unwrap();
                    let frontend = owner.storage().get(&frontend_handle).unwrap();
                    let input = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            [*vc_ref, *core_ref]
                                .into_iter()
                                .map(|parent| {
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
                                        old.snapshot,
                                        parent.clone(),
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        )
                        .unwrap(),
                    );
                    let selected = vcs.vcs()[0].id;
                    let expected = try_discharge(DischargeInput {
                        vc_set: &vcs,
                        policy: &policy,
                    })
                    .unwrap();
                    assert_eq!(
                        inputs
                            .native_theorem_discharge_with_core_vc_parents(
                                &input, &owner, selected, &policy
                            )
                            .unwrap(),
                        Some(expected.clone())
                    );
                    let ast = frontend.ast.as_ref().unwrap();
                    let semantic = semantics(ast, core.module_id(), &core).unwrap().unwrap();
                    let bytes = encode(
                        ast,
                        core.module_id(),
                        &core,
                        DischargeInput {
                            vc_set: &vcs,
                            policy: &policy,
                        },
                        &expected,
                        &generation,
                    )
                    .unwrap()
                    .unwrap();
                    let mut canonical =
                        b"mizar-driver/native-theorem-discharge-content/v1\0".to_vec();
                    for value in [
                        semantic.as_slice(),
                        generation.as_str().as_bytes(),
                        schema.as_str().as_bytes(),
                        key.as_bytes(),
                        &max_steps.to_le_bytes(),
                    ] {
                        canonical.extend_from_slice(&(value.len() as u64).to_le_bytes());
                        canonical.extend_from_slice(value);
                    }
                    let version = &inputs.snapshot.source_versions[0];
                    let mut content = blake3::Hasher::new();
                    field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                    field(&mut content, &canonical);
                    content.update(&2_u64.to_le_bytes());
                    let mut parents = [*core_ref, *vc_ref];
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
                        "mizar-driver/native-theorem-discharge-storage-side-table/v1",
                        &bytes,
                    ));
                    let tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "discharge-theorem-storage",
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
                            field(&mut side, b"discharge-theorem-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    assert_eq!(core.source_id(), ast.source_id);
                    assert_eq!(
                        core.module_id(),
                        &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                    );
                    assert_eq!(vcs.source(), ast.source_id);
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.schema_version(), &schema);
                    assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
                    assert!(
                        core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open
                        && vc.anchor.generation_schema_version == generation));
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
                    assert!(expected.evidence_records().is_empty());
                    assert_eq!(
                        expected.explanations()[0].category,
                        DischargeExplanationCategory::UnsupportedRule
                    );
                    assert_eq!(expected.explanations()[0].rule, None);
                    assert!(expected.vc_set().vcs().iter().all(|vc| vc.status
                        == VcStatus::NeedsAtp
                        && vc.provenance.last().unwrap().phase == VcProvenancePhase::Discharge));
                    let before = (
                        input.clone(),
                        (*core).clone(),
                        (*vcs).clone(),
                        policy.clone(),
                        session.captured().clone(),
                    );
                    let sealed = inputs
                        .publish_native_theorem_discharge(&input, &owner, selected, &policy)
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), old.snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("VcDischarge"));
                    assert_eq!(
                        sealed.output_kind(),
                        &OutputKind::new("SourceTheoremDischarge")
                    );
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), core_ref.work_unit());
                    assert_eq!(sealed.lineage().parents.len(), 2);
                    assert!(
                        sealed.lineage().parents.contains(&core_ref.output())
                            && sealed.lineage().parents.contains(&vc_ref.output())
                    );
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        core_ref.lineage().named_input_hashes
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
                    let duplicate = PublishError::Storage {
                        error: Box::new(StorageError::OutputAlreadyStored {
                            output: sealed.output(),
                        }),
                    }
                    .to_string();
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(if open {
                                &[0xff]
                            } else {
                                b"open theorem Changed: 0 = 1;"
                            });
                        }
                        if stage == 2 {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                        assert_eq!(
                            inputs
                                .publish_native_theorem_discharge(&input, &owner, selected, &policy)
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
                            (*owner.storage().get(&core_handle).unwrap()).clone(),
                            (*owner.storage().get(&vc_handle).unwrap()).clone(),
                            policy,
                            session.captured().clone()
                        ),
                        before
                    );
                    let collected = owner.storage().collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 11);
                    assert_eq!(collected.abandoned_slots_dropped, 3);
                    representations.push((expected, canonical, bytes));
                }
                assert_eq!(representations[0].0, representations[1].0);
                assert_eq!(representations[0].0, representations[2].0);
                for other in &representations[1..] {
                    assert_ne!(representations[0].1, other.1);
                    assert_ne!(representations[0].2, other.2);
                }
            },
        );
    }
}

#[test]
fn native_theorem_discharge_publication_preserves_refusals() {
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for huge in [false, true] {
        with_native_summary_parent(
            text,
            false,
            0,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                allow_frontend(&owner, &old.work_unit);
                let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                for (phase, kind) in [
                    ("ModuleResolve", "SourceTheoremModuleCandidates"),
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
                        unit.clone(),
                    ));
                }
                if huge {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("VcDischarge"),
                        OutputKind::new("SourceTheoremDischarge"),
                        unit.clone(),
                    ));
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_load()
                    .register_frontend(vec![])
                    .register_source_theorem_module_resolver()
                    .register_source_theorem_signature_collection()
                    .register_source_theorem_type_checking()
                    .register_source_theorem_registration_resolution()
                    .register_source_theorem_overload_resolution()
                    .register_source_theorem_elaboration()
                    .register_source_theorem_algorithm_preparation()
                    .register_source_theorem_vc_generation(
                        GenerationSchemaVersion::new("λ\0"),
                        VcSchemaVersion::new(""),
                    );
                let (_, submit) = fixture.submit_request(vec![], vec![]);
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                let (inputs, original) = native_disk_lookup_context(&submission, ids);
                let outputs = submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|result| &result.output_refs)
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 10);
                let core = outputs
                    .iter()
                    .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremCore"))
                    .unwrap();
                let vcs = outputs
                    .iter()
                    .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let input = PhaseInput::new(
                    original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        vec![],
                        [*core, *vcs]
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner,
                                    old.snapshot,
                                    parent.clone(),
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new(if huge {
                            "x".repeat(32 * 1024 * 1024)
                        } else {
                            String::new()
                        }),
                        max_steps: u32::MAX,
                    },
                };
                let selected = mizar_vc::vc_ir::VcId::new(0);
                let baseline = inputs
                    .native_theorem_discharge_with_core_vc_parents(
                        &input, &owner, selected, &policy,
                    )
                    .unwrap();
                assert!(baseline.is_some());
                drop(baseline);
                let result =
                    inputs.publish_native_theorem_discharge(&input, &owner, selected, &policy);
                if huge {
                    assert_eq!(result.unwrap(), None);
                } else {
                    assert_eq!(
                        result.unwrap_err(),
                        PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("VcDischarge"),
                            work_unit: unit,
                            output_kind: OutputKind::new("SourceTheoremDischarge")
                        }
                        .to_string()
                    );
                }
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 10);
                assert_eq!(collected.abandoned_slots_dropped, usize::from(!huge));
            },
        );
    }
}
