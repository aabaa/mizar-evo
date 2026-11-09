use super::*;

#[test]
fn native_theorem_candidate_core_vc_discharge_caller_matches_genuine_inputs() {
    use mizar_atp::{backend::build_source_theorem_candidate_inputs as build, problem::*};
    use mizar_build::task_graph::{VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId};
    use mizar_core::{
        control_flow::{ControlFlowOutput, ObligationSeedHandoff},
        core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus},
    };
    use mizar_ir::storage::CollectInput;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcSet, VcStatus,
        },
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold, fragment, format, generation, schema, key, max_steps) in [
        (
            false,
            usize::MAX,
            LogicFragment::Fof,
            ConcreteFormat::Tptp,
            " λ\n\0",
            "",
            "",
            0,
        ),
        (
            true,
            0,
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
            "",
            "核\t\0",
            "計算\0",
            u32::MAX,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, threshold, &old.work_unit);
                let storage = owner.storage();
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
                let profile = LogicProfile::try_new(
                    "caller λ\n",
                    fragment,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let config_before = (
                    generation.clone(),
                    schema.clone(),
                    policy.clone(),
                    profile.clone(),
                );
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone());
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!()
                };
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-candidate:09"),
                    module.clone(),
                    VcOrderKey::new("opaque-order"),
                    vec![],
                    vec![],
                )];
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
                    .flat_map(|row| &row.output_refs)
                    .cloned()
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 11);
                let parent = |kind: &str| {
                    outputs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                        .clone()
                };
                let core_ref = parent("SourceTheoremCore");
                let prep_ref = parent("SourceTheoremPreparation");
                let vc_ref = parent("SourceTheoremVC");
                let discharge_ref = parent("SourceTheoremDischarge");
                let child = |parents: Vec<AnyPhaseOutputRef>| {
                    PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            parents
                                .into_iter()
                                .map(|parent| {
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
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
                let core_handle = storage
                    .typed_handle::<CoreIr>(&core_ref, core_ref.output_kind())
                    .unwrap();
                let prep_handle = storage
                    .typed_handle::<(ControlFlowOutput, ObligationSeedHandoff)>(
                        &prep_ref,
                        prep_ref.output_kind(),
                    )
                    .unwrap();
                let vc_handle = storage
                    .typed_handle::<VcSet>(&vc_ref, vc_ref.output_kind())
                    .unwrap();
                let discharge_handle = storage
                    .typed_handle::<DischargeOutput>(&discharge_ref, discharge_ref.output_kind())
                    .unwrap();
                let core = storage.get(&core_handle).unwrap();
                let preparation = storage.get(&prep_handle).unwrap();
                let vcs = storage.get(&vc_handle).unwrap();
                let discharge = storage.get(&discharge_handle).unwrap();
                let id = vcs.vcs()[0].id;
                let raw_input = child(vec![vc_ref.clone(), core_ref.clone()]);
                let raw = inputs
                    .native_theorem_discharge_with_core_vc_parents(&raw_input, &owner, id, &policy)
                    .unwrap()
                    .unwrap();
                assert_eq!(*discharge, raw);
                let expected = build(
                    &core,
                    &vcs,
                    id,
                    &discharge,
                    &policy,
                    profile.clone(),
                    format,
                )
                .unwrap();
                let input = child(vec![
                    discharge_ref.clone(),
                    core_ref.clone(),
                    vc_ref.clone(),
                ]);
                let before = (
                    input.clone(),
                    (*core).clone(),
                    (*preparation).clone(),
                    (*vcs).clone(),
                    (*discharge).clone(),
                    session.captured().clone(),
                    outputs.clone(),
                );
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
                assert_eq!(vcs.vcs().len(), 1);
                assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
                );
                for seed in vcs.seed_accounting() {
                    let SeedOriginRef::ExistingCore { seed: original } = seed.origin else {
                        panic!()
                    };
                    let original = core.obligation_seeds().get(original).unwrap();
                    assert_eq!(seed.seed_status, ObligationSeedStatus::Active);
                    assert_eq!(original.status, ObligationSeedStatus::Active);
                    assert_eq!(
                        &preparation.1.entries.get(seed.handoff).unwrap().seed,
                        original
                    );
                }
                for vc in vcs.vcs() {
                    assert_eq!(vc.status, VcStatus::Open);
                    assert_eq!(vc.anchor.generation_schema_version, generation);
                    assert!(vcs.seed_accounting_for(vc.seed.handoff).is_some());
                }
                assert!(
                    discharge
                        .vc_set()
                        .vcs()
                        .iter()
                        .all(|row| row.status == VcStatus::NeedsAtp)
                );
                assert_eq!(expected.0.concrete_format(), format);
                assert_eq!(expected.0.logic_profile_name(), profile.name().as_str());
                assert_eq!(discharge_ref.lineage().parents.len(), 2);
                assert!(
                    discharge_ref.lineage().parents.contains(&core_ref.output())
                        && discharge_ref.lineage().parents.contains(&vc_ref.output())
                );
                assert_eq!(
                    matches!(discharge_ref.placement(), StoragePlacement::Blob { .. }),
                    open
                );
                for change in [0, 1, 2] {
                    if change == 1 {
                        fixture.write(b"invalid changed disk");
                    }
                    if change == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    assert_eq!(
                        inputs
                            .native_theorem_candidate_with_core_vc_discharge_parents(
                                &input,
                                &owner,
                                id,
                                &policy,
                                profile.clone(),
                                format
                            )
                            .unwrap()
                            .unwrap(),
                        expected
                    );
                }
                if !open {
                    let error = build(
                        &core,
                        &vcs,
                        id,
                        &discharge,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::SmtLib,
                    )
                    .unwrap_err();
                    assert_eq!(
                        inputs
                            .native_theorem_candidate_with_core_vc_discharge_parents(
                                &input,
                                &owner,
                                id,
                                &policy,
                                profile.clone(),
                                ConcreteFormat::SmtLib
                            )
                            .unwrap_err(),
                        error
                    );
                }
                let equivalent = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("different"),
                        max_steps: 17,
                    },
                };
                let other_raw = inputs
                    .native_theorem_discharge_with_core_vc_parents(
                        &raw_input,
                        &owner,
                        id,
                        &equivalent,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(other_raw, raw);
                let other_pair = build(
                    &core,
                    &vcs,
                    id,
                    &other_raw,
                    &equivalent,
                    profile.clone(),
                    format,
                )
                .unwrap();
                assert_eq!(
                    inputs
                        .native_theorem_candidate_with_core_vc_discharge_parents(
                            &input,
                            &owner,
                            id,
                            &equivalent,
                            profile.clone(),
                            format
                        )
                        .unwrap()
                        .unwrap(),
                    other_pair
                );
                assert_eq!(
                    (
                        input,
                        (*storage.get(&core_handle).unwrap()).clone(),
                        (*storage.get(&prep_handle).unwrap()).clone(),
                        (*storage.get(&vc_handle).unwrap()).clone(),
                        (*storage.get(&discharge_handle).unwrap()).clone(),
                        session.captured().clone(),
                        outputs
                    ),
                    before
                );
                assert_eq!((generation, schema, policy, profile), config_before);
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 11);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}

#[test]
fn native_theorem_candidate_core_vc_discharge_caller_preserves_refusals() {
    use mizar_atp::problem::*;
    use mizar_core::core_ir::CoreIr;
    use mizar_driver::{
        native_theorem_core_from_storage_bytes as decode_core,
        native_theorem_core_storage_bytes as encode_core,
        native_theorem_discharge_from_storage_bytes as decode_discharge,
        native_theorem_discharge_storage_bytes as encode_discharge,
    };
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for version in [1, 2] {
        with_native_summary_parent(
            text,
            false,
            0,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                let storage = owner.storage();
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
                let generation = GenerationSchemaVersion::new("λ\n\0");
                let schema = VcSchemaVersion::new("");
                let policy = DischargePolicy::default();
                let profile = LogicProfile::try_new(
                    "refusal",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Unsupported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let bad_profile = LogicProfile::try_new(
                    "bad",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::GuardPredicates,
                    NativePropertySupport::Unsupported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
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
                    .flat_map(|row| &row.output_refs)
                    .cloned()
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 10);
                assert!(outputs.iter().all(
                    |output| output.output_kind() != &OutputKind::new("SourceTheoremDischarge")
                ));
                let parent = |kind: &str| {
                    outputs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                        .clone()
                };
                let core_ref = parent("SourceTheoremCore");
                let prep_ref = parent("SourceTheoremPreparation");
                let vc_ref = parent("SourceTheoremVC");
                let child = |parents: Vec<AnyPhaseOutputRef>, dependencies: Vec<Hash>| {
                    PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            dependencies,
                            parents
                                .into_iter()
                                .map(|parent| {
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
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
                let core_handle = storage
                    .typed_handle::<CoreIr>(&core_ref, core_ref.output_kind())
                    .unwrap();
                let vc_handle = storage
                    .typed_handle::<VcSet>(&vc_ref, vc_ref.output_kind())
                    .unwrap();
                let core = storage.get(&core_handle).unwrap();
                let vcs = storage.get(&vc_handle).unwrap();
                let id = vcs.vcs()[0].id;
                let raw_input = child(vec![core_ref.clone(), vc_ref.clone()], vec![]);
                let wanted = inputs
                    .native_theorem_discharge_with_core_vc_parents(&raw_input, &owner, id, &policy)
                    .unwrap()
                    .unwrap();
                let changed = Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
                let (_, foreign_submission, adapter, frontend, _) =
                    native_disk_frontend_record_submission(&changed, ids, snapshots, 0);
                let (foreign_inputs, foreign_old) =
                    native_disk_lookup_context(&foreign_submission, ids);
                let foreign_owner = adapter.publisher();
                let foreign_input = PhaseInput::new(
                    foreign_old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        foreign_old.snapshot,
                        foreign_old.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                foreign_owner,
                                foreign_old.snapshot,
                                frontend.erase(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let foreign_core = foreign_inputs
                    .native_theorem_core_with_frontend_parent(&foreign_input, foreign_owner)
                    .unwrap()
                    .unwrap();
                let foreign_frontend = foreign_owner.storage().get(&frontend).unwrap();
                let ast = foreign_frontend.ast.as_ref().unwrap();
                assert_ne!(ast.source_id, core.source_id());
                let bytes = encode_core(ast, foreign_core.module_id(), &foreign_core)
                    .unwrap()
                    .unwrap();
                let rebound = decode_core(&bytes, core.source_id(), core.module_id())
                    .unwrap()
                    .unwrap();
                assert_eq!(rebound.source_id(), core.source_id());
                assert_eq!(rebound.module_id(), core.module_id());
                assert_ne!(rebound, *core);
                let rebound_vcs = generate(&rebound, old.snapshot, &generation, &schema).unwrap();
                let negative = try_discharge(DischargeInput {
                    vc_set: &rebound_vcs,
                    policy: &policy,
                })
                .unwrap();
                assert_eq!(negative.vc_set().source(), wanted.vc_set().source());
                assert_eq!(negative.vc_set().module(), wanted.vc_set().module());
                assert_eq!(negative.vc_set().snapshot(), wanted.vc_set().snapshot());
                assert_ne!(negative, wanted);
                assert!(
                    negative.evidence_records().is_empty() && wanted.evidence_records().is_empty()
                );
                let foreign_vcs =
                    generate(&foreign_core, old.snapshot, &generation, &schema).unwrap();
                let foreign_discharge = try_discharge(DischargeInput {
                    vc_set: &foreign_vcs,
                    policy: &policy,
                })
                .unwrap();
                let wire = encode_discharge(
                    ast,
                    foreign_core.module_id(),
                    &foreign_core,
                    DischargeInput {
                        vc_set: &foreign_vcs,
                        policy: &policy,
                    },
                    &foreign_discharge,
                    &generation,
                )
                .unwrap()
                .unwrap();
                assert_eq!(
                    decode_discharge(
                        &wire,
                        core.source_id(),
                        core.module_id(),
                        old.snapshot,
                        &policy
                    )
                    .unwrap()
                    .unwrap(),
                    negative
                );
                let phase = IrPipelinePhase::new("VcDischarge");
                let kind = OutputKind::new("SourceTheoremDischarge");
                let source_id = core.source_id();
                let module = core.module_id().clone();
                let decode_policy = policy.clone();
                let snapshot = old.snapshot;
                let output = owner
                    .publish(PublishOutputInput {
                        slot: owner.allocate(
                            snapshot,
                            phase.clone(),
                            core_ref.work_unit().clone(),
                            kind.clone(),
                            SchemaVersion::new(version),
                        ),
                        snapshot,
                        phase,
                        work_unit: core_ref.work_unit().clone(),
                        output_kind: kind,
                        schema_version: SchemaVersion::new(version),
                        payload: negative.clone(),
                        canonical_payload: Some(wire.clone()),
                        storage_payload: Some(wire.clone()),
                        decode: BlobDecoder::new(move |bytes| {
                            decode_discharge(bytes, source_id, &module, snapshot, &decode_policy)
                                .map_err(BlobDecodeError::new)?
                                .ok_or_else(|| BlobDecodeError::new("negative discharge replay"))
                        }),
                        parents: vec![vc_ref.clone(), core_ref.clone()],
                        named_input_hashes: core_ref.lineage().named_input_hashes.clone(),
                        side_tables: IrSideTables::default(),
                        origin: OutputOrigin::PackageSource,
                        target: PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                assert_eq!(*storage.get(&output).unwrap(), negative);
                assert!(matches!(output.placement(), StoragePlacement::Blob { .. }));
                let discharge_ref = output.erase();
                let valid = child(
                    vec![discharge_ref.clone(), vc_ref.clone(), core_ref.clone()],
                    vec![],
                );
                let check = |input: &PhaseInput| {
                    inputs.native_theorem_candidate_with_core_vc_discharge_parents(
                        input,
                        &owner,
                        id,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                };
                assert_eq!(
                    check(&valid).unwrap_err(),
                    if version == 1 {
                        "native theorem candidate complete Discharge mismatch"
                    } else {
                        "native theorem candidate parent binding mismatch"
                    }
                );
                assert_eq!(
                    inputs
                        .publish_native_theorem_candidate(
                            &valid,
                            &owner,
                            id,
                            &policy,
                            profile.clone(),
                            ConcreteFormat::Tptp
                        )
                        .unwrap_err(),
                    if version == 1 {
                        "native theorem candidate complete Discharge mismatch"
                    } else {
                        "native theorem candidate parent binding mismatch"
                    }
                );
                if version == 1 {
                    let absent = VcId::new(usize::MAX);
                    for dependencies in [vec![], vec![hash(0xd1)]] {
                        let dual =
                            child(vec![core_ref.clone(), vc_ref.clone()], dependencies.clone());
                        let three = child(
                            vec![core_ref.clone(), discharge_ref.clone(), vc_ref.clone()],
                            dependencies,
                        );
                        let expected = inputs
                            .native_theorem_discharge_with_core_vc_parents(
                                &dual, &owner, absent, &policy,
                            )
                            .unwrap_err();
                        assert_eq!(
                            inputs
                                .native_theorem_candidate_with_core_vc_discharge_parents(
                                    &three,
                                    &owner,
                                    absent,
                                    &policy,
                                    bad_profile.clone(),
                                    ConcreteFormat::SmtLib
                                )
                                .unwrap_err(),
                            expected
                        );
                        assert_eq!(
                            inputs
                                .publish_native_theorem_candidate(
                                    &three,
                                    &owner,
                                    absent,
                                    &policy,
                                    bad_profile.clone(),
                                    ConcreteFormat::SmtLib
                                )
                                .unwrap_err(),
                            expected
                        );
                    }
                    for (parents, expected) in [
                        (vec![], "requires three parents"),
                        (
                            vec![
                                core_ref.clone(),
                                vc_ref.clone(),
                                discharge_ref.clone(),
                                prep_ref.clone(),
                            ],
                            "requires three parents",
                        ),
                        (
                            vec![prep_ref.clone(), vc_ref.clone(), discharge_ref.clone()],
                            "requires one Core parent",
                        ),
                        (
                            vec![core_ref.clone(), prep_ref.clone(), discharge_ref.clone()],
                            "requires one VC parent",
                        ),
                        (
                            vec![core_ref.clone(), vc_ref.clone(), prep_ref.clone()],
                            "requires one Discharge parent",
                        ),
                    ] {
                        assert_eq!(
                            check(&child(parents, vec![])).unwrap_err(),
                            format!("native theorem candidate {expected}")
                        );
                    }
                    let vc_unit = WorkUnit::Vc {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new("main"),
                        ),
                        descriptor: mizar_build::task_graph::VcTaskDescriptorId::new(
                            "opaque-control",
                        ),
                    };
                    let nonmodule = PhaseInput::new(
                        vc_unit,
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            valid.parent_outputs().to_vec(),
                        )
                        .unwrap(),
                    );
                    assert_eq!(
                        check(&nonmodule).unwrap_err(),
                        "native theorem candidate requires a module"
                    );
                    let wrong_hash = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            hash(0x96),
                            vec![],
                            valid.parent_outputs().to_vec(),
                        )
                        .unwrap(),
                    );
                    assert_eq!(
                        check(&wrong_hash).unwrap_err(),
                        "native theorem candidate parent binding mismatch"
                    );
                    let alien = publisher(old.snapshot, 0, &old.work_unit);
                    assert!(
                        inputs
                            .native_theorem_candidate_with_core_vc_discharge_parents(
                                &valid,
                                &alien,
                                id,
                                &policy,
                                profile.clone(),
                                ConcreteFormat::Tptp
                            )
                            .is_err()
                    );
                }
                assert_eq!(*storage.get(&output).unwrap(), negative);
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 11);
                assert_eq!(collected.abandoned_slots_dropped, 0);
                assert!(check(&valid).is_err());
            },
        );
    }
}

#[test]
fn native_theorem_candidate_storage_replays_genuine_inputs() {
    use mizar_atp::{backend::build_source_theorem_candidate_inputs as build, problem::*};
    use mizar_driver::{
        native_theorem_candidate_from_storage_bytes as decode,
        native_theorem_candidate_storage_bytes as encode,
        native_theorem_core_from_storage_bytes as old_decode,
        native_theorem_core_storage_bytes as old_encode,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let text = b"theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (fragment, chosen, native, formats, tail, generation, schema, key, max_steps) in [
        (
            LogicFragment::Fof,
            ConcreteFormat::Tptp,
            NativePropertySupport::Supported,
            vec![ConcreteFormat::Tptp, ConcreteFormat::SmtLib],
            [0, 1, 1, 3, 1, 3, 0],
            "λ\n\0",
            "",
            "",
            0,
        ),
        (
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
            NativePropertySupport::Supported,
            vec![ConcreteFormat::Tptp, ConcreteFormat::SmtLib],
            [2, 1, 1, 3, 1, 3, 1],
            "",
            "核\t\0",
            "計算\0",
            u32::MAX,
        ),
        (
            LogicFragment::Fof,
            ConcreteFormat::Tptp,
            NativePropertySupport::Unsupported,
            vec![ConcreteFormat::Tptp],
            [0, 1, 1, 3, 0, 1, 0],
            "",
            "",
            "exact",
            17,
        ),
        (
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
            NativePropertySupport::Unsupported,
            vec![ConcreteFormat::SmtLib],
            [2, 1, 1, 3, 0, 2, 1],
            "explicit",
            "schema",
            "",
            64,
        ),
    ] {
        let fixture = Fixture::new(text);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
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
        let frontend = owner.storage().get(&handle).unwrap();
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let opaque = old_encode(ast, &module, &core).unwrap().unwrap();
        let generation = GenerationSchemaVersion::new(generation);
        let schema = VcSchemaVersion::new(schema);
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(key),
                max_steps,
            },
        };
        let profile = LogicProfile::try_new(
            "caller λ\n\0",
            fragment,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::ScopedGuardPredicates,
            native,
            formats.into_iter().collect(),
        )
        .unwrap();
        let vcs = generate(&core, original.snapshot, &generation, &schema).unwrap();
        let id = vcs.vcs()[0].id;
        let discharge = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        })
        .unwrap();
        let pair = build(
            &core,
            &vcs,
            id,
            &discharge,
            &policy,
            profile.clone(),
            chosen,
        )
        .unwrap();
        let mut record = [0u8; 19];
        record[..8].copy_from_slice(&u64::try_from(id.index()).unwrap().to_le_bytes());
        record[8..12].copy_from_slice(&max_steps.to_le_bytes());
        record[12..19].copy_from_slice(&tail);
        let mut bytes = b"mizar-driver/native-theorem-candidate-storage/v1\0".to_vec();
        for field in [
            opaque.as_slice(),
            generation.as_str().as_bytes(),
            schema.as_str().as_bytes(),
            key.as_bytes(),
            profile.name().as_str().as_bytes(),
            record.as_slice(),
        ] {
            bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(field);
        }
        let before = (
            ast.clone(),
            core.clone(),
            generation.clone(),
            schema.clone(),
            policy.clone(),
            profile.clone(),
            pair.clone(),
            bytes.clone(),
        );
        for _ in 0..2 {
            assert_eq!(
                encode(
                    ast,
                    &module,
                    &core,
                    (original.snapshot, &generation, &schema, id),
                    (&policy, &profile, chosen),
                    &pair
                )
                .unwrap(),
                Some(bytes.clone())
            );
        }
        let shifted = snapshot_id(0x97);
        let mut relocated = ids.next_source_id(shifted).unwrap();
        if relocated == ast.source_id {
            relocated = ids.next_source_id(shifted).unwrap();
        }
        assert_ne!(relocated, ast.source_id);
        for (source_id, snapshot) in [(ast.source_id, original.snapshot), (relocated, shifted)] {
            let replayed = old_decode(&opaque, source_id, &module).unwrap().unwrap();
            let original = generate(&replayed, snapshot, &generation, &schema).unwrap();
            let processed = try_discharge(DischargeInput {
                vc_set: &original,
                policy: &policy,
            })
            .unwrap();
            let expected = build(
                &replayed,
                &original,
                id,
                &processed,
                &policy,
                profile.clone(),
                chosen,
            )
            .unwrap();
            for _ in 0..2 {
                assert_eq!(
                    decode(&bytes, source_id, &module, snapshot).unwrap(),
                    Some(expected.clone())
                );
            }
        }
        let wrong = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other"));
        assert_eq!(old_decode(&opaque, ast.source_id, &wrong).unwrap(), None);
        assert_eq!(
            decode(&bytes, ast.source_id, &wrong, original.snapshot).unwrap(),
            None
        );
        assert_eq!(
            (
                ast.clone(),
                core,
                generation,
                schema,
                policy,
                profile,
                pair,
                bytes,
            ),
            before
        );
    }
}

#[test]
fn native_theorem_candidate_storage_preserves_refusals() {
    use mizar_atp::{backend::build_source_theorem_candidate_inputs as build, problem::*};
    use mizar_driver::{
        native_theorem_candidate_from_storage_bytes as decode,
        native_theorem_candidate_storage_bytes as encode,
        native_theorem_core_from_storage_bytes as old_decode,
        native_theorem_core_storage_bytes as old_encode,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("計算\0"),
            max_steps: 0,
        },
    };
    let profile = LogicProfile::try_new(
        "caller",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Supported,
        [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
            .into_iter()
            .collect(),
    )
    .unwrap();
    let fixtures = [
        Fixture::new(
            b"theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        ),
        Fixture::new(
            b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
        ),
    ];
    let mut sources = Vec::new();
    for fixture in &fixtures {
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(fixture, &ids, &snapshots, 0);
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
        let opaque = old_encode(ast, &module, &core).unwrap().unwrap();
        let vcs = generate(&core, original.snapshot, &generation, &schema).unwrap();
        let id = vcs.vcs()[0].id;
        let discharge = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        })
        .unwrap();
        let pair = build(
            &core,
            &vcs,
            id,
            &discharge,
            &policy,
            profile.clone(),
            ConcreteFormat::Tptp,
        )
        .unwrap();
        sources.push((ast.clone(), core, opaque, pair, original.snapshot, id));
    }
    let (ast, core, opaque, pair, snapshot, id) = &sources[0];
    let (other_ast, other_core, other_opaque, other_pair, other_snapshot, other_id) = &sources[1];
    let frame = |fields: [&[u8]; 6]| {
        let mut bytes = b"mizar-driver/native-theorem-candidate-storage/v1\0".to_vec();
        for field in fields {
            bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(field);
        }
        bytes
    };
    let mut record = [0u8; 19];
    record[..8].copy_from_slice(&u64::try_from(id.index()).unwrap().to_le_bytes());
    record[8..12].copy_from_slice(&0u32.to_le_bytes());
    record[12..].copy_from_slice(&[0, 1, 1, 3, 1, 3, 0]);
    let fields = [
        opaque.as_slice(),
        generation.as_str().as_bytes(),
        schema.as_str().as_bytes(),
        policy.computation_limit.policy.as_str().as_bytes(),
        profile.name().as_str().as_bytes(),
        record.as_slice(),
    ];
    let bytes = frame(fields);
    let mut other_record = record;
    other_record[..8].copy_from_slice(&u64::try_from(other_id.index()).unwrap().to_le_bytes());
    let other_bytes = frame([
        other_opaque,
        fields[1],
        fields[2],
        fields[3],
        fields[4],
        &other_record,
    ]);
    assert_eq!(
        encode(
            other_ast,
            &module,
            other_core,
            (*other_snapshot, &generation, &schema, *other_id),
            (&policy, &profile, ConcreteFormat::Tptp),
            other_pair
        )
        .unwrap(),
        Some(other_bytes)
    );
    assert_eq!(
        encode(
            ast,
            &module,
            other_core,
            (*other_snapshot, &generation, &schema, *other_id),
            (&policy, &profile, ConcreteFormat::Tptp),
            other_pair
        )
        .unwrap(),
        None
    );
    assert_ne!(pair, other_pair);
    assert_eq!(
        encode(
            ast,
            &module,
            core,
            (*snapshot, &generation, &schema, *id),
            (&policy, &profile, ConcreteFormat::Tptp),
            other_pair
        )
        .unwrap(),
        None
    );
    for (changed_gen, changed_schema, key, max_steps, name) in [
        ("changed", "", "計算\0", 0, "caller"),
        ("λ\n\0", "changed", "計算\0", 0, "caller"),
        ("λ\n\0", "", "different", u32::MAX, "caller"),
        ("λ\n\0", "", "計算\0", 0, "different"),
    ] {
        let generation = GenerationSchemaVersion::new(changed_gen);
        let schema = VcSchemaVersion::new(changed_schema);
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(key),
                max_steps,
            },
        };
        let profile = LogicProfile::try_new(
            name,
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::ScopedGuardPredicates,
            NativePropertySupport::Supported,
            [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                .into_iter()
                .collect(),
        )
        .unwrap();
        let vcs = generate(core, *snapshot, &generation, &schema).unwrap();
        let discharge = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        })
        .unwrap();
        let expected = build(
            core,
            &vcs,
            *id,
            &discharge,
            &policy,
            profile.clone(),
            ConcreteFormat::Tptp,
        )
        .unwrap();
        let mut changed = record;
        changed[8..12].copy_from_slice(&max_steps.to_le_bytes());
        let mutated = frame([
            opaque,
            changed_gen.as_bytes(),
            changed_schema.as_bytes(),
            key.as_bytes(),
            name.as_bytes(),
            &changed,
        ]);
        assert_ne!(mutated, bytes);
        if key == "different" {
            assert_eq!(&expected, pair);
        }
        assert_eq!(
            decode(&mutated, ast.source_id, &module, *snapshot).unwrap(),
            Some(expected.clone())
        );
    }
    let vcs = generate(core, *snapshot, &generation, &schema).unwrap();
    let discharge = try_discharge(DischargeInput {
        vc_set: &vcs,
        policy: &policy,
    })
    .unwrap();
    let mut bad_profile = profile.clone();
    for (offset, value) in [
        (0, 0),
        (12, 1),
        (13, 0),
        (14, 0),
        (15, 0),
        (15, 1),
        (15, 2),
        (18, 1),
    ] {
        let selected = if offset == 0 {
            VcId::new(usize::MAX)
        } else {
            *id
        };
        let fragment = if offset == 12 {
            LogicFragment::TffLike
        } else {
            LogicFragment::Fof
        };
        let equality = if offset == 13 {
            EqualitySupport::Unsupported
        } else {
            EqualitySupport::Supported
        };
        let quantifiers = if offset == 14 {
            QuantifierPolicy::PropositionalOnly
        } else {
            QuantifierPolicy::FirstOrder
        };
        let soft = match (offset, value) {
            (15, 0) => SoftTypeStrategy::BackendSorts,
            (15, 1) => SoftTypeStrategy::GuardPredicates,
            (15, 2) => SoftTypeStrategy::SortsAndGuards,
            _ => SoftTypeStrategy::ScopedGuardPredicates,
        };
        let chosen = if offset == 18 {
            ConcreteFormat::SmtLib
        } else {
            ConcreteFormat::Tptp
        };
        let unsupported = LogicProfile::try_new(
            "caller",
            fragment,
            equality,
            quantifiers,
            soft,
            NativePropertySupport::Supported,
            [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                .into_iter()
                .collect(),
        )
        .unwrap();
        if (offset, value) == (15, 1) {
            bad_profile = unsupported.clone();
        }
        let expected = build(
            core,
            &vcs,
            selected,
            &discharge,
            &policy,
            unsupported.clone(),
            chosen,
        )
        .unwrap_err();
        let mut changed = record;
        changed[offset] = value;
        changed[..8].copy_from_slice(&u64::try_from(selected.index()).unwrap().to_le_bytes());
        let mutated = frame([
            fields[0], fields[1], fields[2], fields[3], fields[4], &changed,
        ]);
        assert_eq!(
            decode(&mutated, ast.source_id, &module, *snapshot).unwrap_err(),
            expected
        );
        assert_eq!(
            encode(
                ast,
                &module,
                core,
                (*snapshot, &generation, &schema, selected),
                (&policy, &unsupported, chosen),
                pair
            )
            .unwrap_err(),
            expected
        );
    }
    let private = Fixture::new(include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    ).replacen("thus X = X;", "thus X = 0;", 1).as_bytes());
    let (_, _, adapter, handle, _) =
        native_disk_frontend_record_submission(&private, &ids, &snapshots, 0);
    let output = adapter.publisher().storage().get(&handle).unwrap();
    let private_ast = output.ast.as_ref().unwrap();
    let expected = old_encode(private_ast, &module, core).unwrap_err();
    let syntax = private_ast.canonical_bytes().unwrap();
    let mut private_core = b"mizar-driver/native-theorem-core-storage/v1\0".to_vec();
    for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
        private_core.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        private_core.extend_from_slice(field);
    }
    assert_eq!(
        old_decode(&private_core, private_ast.source_id, &module).unwrap_err(),
        expected
    );
    let huge = GenerationSchemaVersion::new("x".repeat(32 * 1024 * 1024));
    assert_eq!(
        encode(
            private_ast,
            &module,
            core,
            (*snapshot, &huge, &schema, VcId::new(usize::MAX)),
            (&policy, &bad_profile, ConcreteFormat::SmtLib),
            pair
        )
        .unwrap_err(),
        expected
    );
    assert_eq!(
        encode(
            ast,
            &module,
            core,
            (*snapshot, &huge, &schema, VcId::new(usize::MAX)),
            (&policy, &profile, ConcreteFormat::SmtLib),
            pair
        )
        .unwrap(),
        None
    );
    let empty_name = frame([fields[0], fields[1], fields[2], fields[3], b"", fields[5]]);
    assert_eq!(
        decode(&empty_name, ast.source_id, &module, *snapshot).unwrap(),
        None
    );
    let private_empty_name = frame([
        &private_core,
        fields[1],
        fields[2],
        fields[3],
        b"",
        fields[5],
    ]);
    assert_eq!(
        decode(
            &private_empty_name,
            private_ast.source_id,
            &module,
            *snapshot
        )
        .unwrap_err(),
        expected
    );
    for index in 1..5 {
        let mut malformed = fields;
        malformed[0] = &private_core;
        malformed[index] = &[0xff];
        assert_eq!(
            decode(&frame(malformed), private_ast.source_id, &module, *snapshot).unwrap(),
            None
        );
    }
    for (offset, value) in [
        (12, 3),
        (13, 2),
        (14, 2),
        (15, 4),
        (16, 2),
        (17, 0),
        (17, 4),
        (18, 2),
    ] {
        let mut malformed = record;
        malformed[offset] = value;
        assert_eq!(
            decode(
                &frame([
                    &private_core,
                    fields[1],
                    fields[2],
                    fields[3],
                    fields[4],
                    &malformed
                ]),
                private_ast.source_id,
                &module,
                *snapshot
            )
            .unwrap(),
            None
        );
    }
    for record in [record[..18].to_vec(), [record.as_slice(), &[0]].concat()] {
        assert_eq!(
            decode(
                &frame([
                    &private_core,
                    fields[1],
                    fields[2],
                    fields[3],
                    fields[4],
                    &record
                ]),
                private_ast.source_id,
                &module,
                *snapshot
            )
            .unwrap(),
            None
        );
    }
    let domain = b"mizar-driver/native-theorem-candidate-storage/v1\0";
    let mut oversized = fields;
    oversized[1] = huge.as_str().as_bytes();
    let mut malformed = vec![
        bytes[..domain.len() + 4].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
        bytes[..bytes.len() - 27].to_vec(),
        frame(oversized),
    ];
    let mut bad = bytes.clone();
    bad[0] = b'x';
    malformed.push(bad);
    let mut bad = bytes.clone();
    bad[domain.len() - 2] = b'2';
    malformed.push(bad);
    let mut bad = bytes.clone();
    bad[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    malformed.push(bad);
    let mut bad = bytes.clone();
    bad.push(0);
    malformed.push(bad);
    for bytes in malformed {
        assert_eq!(
            decode(&bytes, ast.source_id, &module, *snapshot).unwrap(),
            None
        );
    }
}

#[test]
fn native_theorem_candidate_publication_seals_genuine_payload() {
    use mizar_atp::{backend::build_source_theorem_candidate_inputs as build, problem::*};
    use mizar_build::task_graph::{VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId};
    use mizar_core::core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_candidate_storage_bytes as encode,
        native_theorem_core_semantic_payload as semantics,
    };
    use mizar_ir::{
        publisher::PublishError,
        storage::{CollectInput, IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcSet, VcStatus,
        },
    };
    let field = |hasher: &mut blake3::Hasher, value: &[u8]| {
        hasher.update(&u64::try_from(value.len()).unwrap().to_le_bytes());
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
                for (key, max_steps) in if open {
                    vec![("計算\0", u32::MAX)]
                } else {
                    vec![("", 0), ("計算\0", u32::MAX)]
                } {
                    let owner = publisher(old.snapshot, threshold, &old.work_unit);
                    let storage = owner.storage();
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
                        ("AtpSolve", "SourceTheoremCandidateInputs"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                    let generation = GenerationSchemaVersion::new(if open { "" } else { "λ\n\0" });
                    let schema = VcSchemaVersion::new(if open { "核\t\0" } else { "" });
                    let policy = DischargePolicy {
                        computation_limit: ComputationLimit {
                            policy: PolicyKey::new(key),
                            max_steps,
                        },
                    };
                    let (fragment, format, tail) = if open {
                        (
                            LogicFragment::SmtLibUninterpreted,
                            ConcreteFormat::SmtLib,
                            [2, 1, 1, 3, 1, 3, 1],
                        )
                    } else {
                        (
                            LogicFragment::Fof,
                            ConcreteFormat::Tptp,
                            [0, 1, 1, 3, 1, 3, 0],
                        )
                    };
                    let profile = LogicProfile::try_new(
                        "caller λ\n\0",
                        fragment,
                        EqualitySupport::Supported,
                        QuantifierPolicy::FirstOrder,
                        SoftTypeStrategy::ScopedGuardPredicates,
                        NativePropertySupport::Supported,
                        [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                            .into_iter()
                            .collect(),
                    )
                    .unwrap();
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
                        .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                        .register_source_theorem_vc_discharge(policy.clone());
                    let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                    let WorkUnit::Module { module } = &old.work_unit else {
                        panic!()
                    };
                    submit.vc_descriptors = vec![VcTaskDescriptor::new(
                        VcTaskDescriptorId::new("opaque-publish:09"),
                        module.clone(),
                        VcOrderKey::new("opaque-order"),
                        vec![],
                        vec![],
                    )];
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
                        .flat_map(|row| &row.output_refs)
                        .cloned()
                        .collect::<Vec<_>>();
                    assert_eq!(outputs.len(), 11);
                    let parent = |kind: &str| {
                        outputs
                            .iter()
                            .find(|output| output.output_kind() == &OutputKind::new(kind))
                            .unwrap()
                            .clone()
                    };
                    let core_ref = parent("SourceTheoremCore");
                    let vc_ref = parent("SourceTheoremVC");
                    let discharge_ref = parent("SourceTheoremDischarge");
                    let frontend_ref = parent("FrontendOutput");
                    let core_handle = storage
                        .typed_handle::<CoreIr>(&core_ref, core_ref.output_kind())
                        .unwrap();
                    let vc_handle = storage
                        .typed_handle::<VcSet>(&vc_ref, vc_ref.output_kind())
                        .unwrap();
                    let discharge_handle = storage
                        .typed_handle::<DischargeOutput>(
                            &discharge_ref,
                            discharge_ref.output_kind(),
                        )
                        .unwrap();
                    let frontend_handle = storage
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            &frontend_ref,
                            frontend_ref.output_kind(),
                        )
                        .unwrap();
                    let core = storage.get(&core_handle).unwrap();
                    let vcs = storage.get(&vc_handle).unwrap();
                    let discharge = storage.get(&discharge_handle).unwrap();
                    let frontend = storage.get(&frontend_handle).unwrap();
                    let ast = frontend.ast.as_ref().unwrap();
                    let id = vcs.vcs()[0].id;
                    let input = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            [discharge_ref.clone(), core_ref.clone(), vc_ref.clone()]
                                .into_iter()
                                .map(|parent| {
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
                                        old.snapshot,
                                        parent,
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        )
                        .unwrap(),
                    );
                    let pair = build(
                        &core,
                        &vcs,
                        id,
                        &discharge,
                        &policy,
                        profile.clone(),
                        format,
                    )
                    .unwrap();
                    assert_eq!(
                        inputs
                            .native_theorem_candidate_with_core_vc_discharge_parents(
                                &input,
                                &owner,
                                id,
                                &policy,
                                profile.clone(),
                                format
                            )
                            .unwrap(),
                        Some(pair.clone())
                    );
                    let semantic = semantics(ast, core.module_id(), &core).unwrap().unwrap();
                    let bytes = encode(
                        ast,
                        core.module_id(),
                        &core,
                        (old.snapshot, &generation, &schema, id),
                        (&policy, &profile, format),
                        &pair,
                    )
                    .unwrap()
                    .unwrap();
                    let mut record = [0u8; 19];
                    record[..8].copy_from_slice(&u64::try_from(id.index()).unwrap().to_le_bytes());
                    record[8..12].copy_from_slice(&max_steps.to_le_bytes());
                    record[12..].copy_from_slice(&tail);
                    let mut suffix = Vec::new();
                    for value in [
                        generation.as_str().as_bytes(),
                        schema.as_str().as_bytes(),
                        key.as_bytes(),
                        profile.name().as_str().as_bytes(),
                        record.as_slice(),
                    ] {
                        suffix
                            .extend_from_slice(&u64::try_from(value.len()).unwrap().to_le_bytes());
                        suffix.extend_from_slice(value);
                    }
                    assert!(bytes.ends_with(&suffix));
                    let mut canonical =
                        b"mizar-driver/native-theorem-candidate-content/v1\0".to_vec();
                    for value in [semantic.as_slice(), suffix.as_slice()] {
                        canonical
                            .extend_from_slice(&u64::try_from(value.len()).unwrap().to_le_bytes());
                        canonical.extend_from_slice(value);
                    }
                    let version = &inputs.snapshot.source_versions[0];
                    let mut content = blake3::Hasher::new();
                    field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                    field(&mut content, &canonical);
                    content.update(&3u64.to_le_bytes());
                    let mut parents = [&core_ref, &vc_ref, &discharge_ref];
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
                    content.update(&1u64.to_le_bytes());
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
                    blob.update(&1u32.to_le_bytes());
                    field(&mut blob, &bytes);
                    let expected_blob = Hash::from_bytes(*blob.finalize().as_bytes());
                    let digest = Hash::from_bytes(blake3::derive_key(
                        "mizar-driver/native-theorem-candidate-storage-side-table/v1",
                        &bytes,
                    ));
                    let tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "candidate-theorem-storage",
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
                            field(&mut side, b"candidate-theorem-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    assert_eq!(core.source_id(), ast.source_id);
                    assert_eq!(vcs.source(), ast.source_id);
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.schema_version(), &schema);
                    assert_eq!(vcs.vcs().len(), 1);
                    assert!(
                        core.proofs()
                            .iter()
                            .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
                    );
                    for row in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!()
                        };
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(
                            core.obligation_seeds().get(seed).unwrap().status,
                            ObligationSeedStatus::Active
                        );
                    }
                    assert!(vcs.vcs().iter().all(|row| row.status == VcStatus::Open
                        && row.anchor.generation_schema_version == generation
                        && vcs.seed_accounting_for(row.seed.handoff).is_some()));
                    assert!(
                        discharge
                            .vc_set()
                            .vcs()
                            .iter()
                            .all(|row| row.status == VcStatus::NeedsAtp)
                    );
                    let before = (
                        input.clone(),
                        ast.clone(),
                        (*core).clone(),
                        (*vcs).clone(),
                        (*discharge).clone(),
                        policy.clone(),
                        profile.clone(),
                        pair.clone(),
                        session.captured().clone(),
                    );
                    fixture.write(&[0xff]);
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    let sealed = inputs
                        .publish_native_theorem_candidate(
                            &input,
                            &owner,
                            id,
                            &policy,
                            profile.clone(),
                            format,
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), old.snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("AtpSolve"));
                    assert_eq!(
                        sealed.output_kind(),
                        &OutputKind::new("SourceTheoremCandidateInputs")
                    );
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), core_ref.work_unit());
                    assert_eq!(sealed.lineage().parents.len(), 3);
                    for parent in [&core_ref, &vc_ref, &discharge_ref] {
                        assert!(sealed.lineage().parents.contains(&parent.output()));
                    }
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        core_ref.lineage().named_input_hashes
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
                        _ => panic!(),
                    }
                    assert_eq!(*storage.get(&sealed).unwrap(), pair);
                    owner
                        .validate_published_output(old.snapshot, sealed.any())
                        .unwrap();
                    assert_eq!(
                        owner
                            .validate_current_output(old.snapshot, sealed.any())
                            .is_ok(),
                        !open
                    );
                    let duplicate = PublishError::Storage {
                        error: Box::new(StorageError::OutputAlreadyStored {
                            output: sealed.output(),
                        }),
                    }
                    .to_string();
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate(
                                &input,
                                &owner,
                                id,
                                &policy,
                                profile.clone(),
                                format
                            )
                            .unwrap_err(),
                        duplicate
                    );
                    assert_eq!(
                        (
                            input,
                            ast.clone(),
                            (*storage.get(&core_handle).unwrap()).clone(),
                            (*storage.get(&vc_handle).unwrap()).clone(),
                            (*storage.get(&discharge_handle).unwrap()).clone(),
                            policy,
                            profile,
                            pair.clone(),
                            session.captured().clone()
                        ),
                        before
                    );
                    let collected = storage.collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 12);
                    assert_eq!(collected.abandoned_slots_dropped, 1);
                    representations.push((pair, canonical, bytes, expected_content));
                }
                if !open {
                    assert_eq!(representations[0].0, representations[1].0);
                    assert_ne!(representations[0].1, representations[1].1);
                    assert_ne!(representations[0].2, representations[1].2);
                    assert_ne!(representations[0].3, representations[1].3);
                }
            },
        );
    }
}

#[test]
fn native_theorem_candidate_publication_preserves_refusals() {
    use mizar_atp::problem::*;
    use mizar_build::task_graph::{VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId};
    use mizar_core::core_ir::CoreIr;
    use mizar_driver::{
        native_theorem_candidate_storage_bytes as encode,
        native_theorem_core_semantic_payload as semantics,
    };
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    use mizar_vc::{
        discharge::{
            ComputationLimit, DischargeExplanationCategory, DischargeInput, DischargeOutput,
            DischargePolicy, try_discharge,
        },
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for huge in [false, true] {
        with_native_summary_parent(
            text,
            false,
            0,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                let storage = owner.storage();
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
                    ("VcDischarge", "SourceTheoremDischarge"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        unit.clone(),
                    ));
                }
                if huge {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("AtpSolve"),
                        OutputKind::new("SourceTheoremCandidateInputs"),
                        unit.clone(),
                    ));
                }
                let generation = GenerationSchemaVersion::new("λ\n\0");
                let schema = VcSchemaVersion::new("");
                let base_policy = DischargePolicy::default();
                let profile = LogicProfile::try_new(
                    "normal",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(base_policy);
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!()
                };
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-refusal"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![],
                    vec![],
                )];
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
                    .flat_map(|row| &row.output_refs)
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 11);
                let parent = |kind: &str| {
                    *outputs
                        .iter()
                        .find(|output| output.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                };
                let core_ref = parent("SourceTheoremCore");
                let vc_ref = parent("SourceTheoremVC");
                let discharge_ref = parent("SourceTheoremDischarge");
                let frontend_ref = parent("FrontendOutput");
                let core = storage
                    .get(
                        &storage
                            .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let vcs = storage
                    .get(
                        &storage
                            .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let discharge = storage
                    .get(
                        &storage
                            .typed_handle::<DischargeOutput>(
                                discharge_ref,
                                discharge_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let frontend = storage
                    .get(
                        &storage
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                let id = vcs.vcs()[0].id;
                let input = PhaseInput::new(
                    original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        vec![],
                        [core_ref, vc_ref, discharge_ref]
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
                            "small".to_owned()
                        }),
                        max_steps: u32::MAX,
                    },
                };
                assert_eq!(
                    try_discharge(DischargeInput {
                        vc_set: &vcs,
                        policy: &policy
                    })
                    .unwrap(),
                    *discharge
                );
                let pair = inputs
                    .native_theorem_candidate_with_core_vc_discharge_parents(
                        &input,
                        &owner,
                        id,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                    .unwrap()
                    .unwrap();
                assert!(
                    discharge
                        .explanations()
                        .iter()
                        .all(|row| row.category == DischargeExplanationCategory::UnsupportedRule)
                );
                let semantic = semantics(ast, core.module_id(), &core).unwrap().unwrap();
                let bytes = encode(
                    ast,
                    core.module_id(),
                    &core,
                    (old.snapshot, &generation, &schema, id),
                    (&policy, &profile, ConcreteFormat::Tptp),
                    &pair,
                )
                .unwrap();
                assert_eq!(bytes.is_none(), huge);
                if !huge {
                    let mut record = [0u8; 19];
                    record[..8].copy_from_slice(&u64::try_from(id.index()).unwrap().to_le_bytes());
                    record[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
                    record[12..].copy_from_slice(&[0, 1, 1, 3, 1, 3, 0]);
                    let mut suffix = Vec::new();
                    for field in [
                        generation.as_str().as_bytes(),
                        schema.as_str().as_bytes(),
                        b"small",
                        b"normal",
                        record.as_slice(),
                    ] {
                        suffix
                            .extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                        suffix.extend_from_slice(field);
                    }
                    assert!(bytes.as_ref().unwrap().ends_with(&suffix));
                    let mut content =
                        b"mizar-driver/native-theorem-candidate-content/v1\0".to_vec();
                    for field in [semantic.as_slice(), suffix.as_slice()] {
                        content
                            .extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                        content.extend_from_slice(field);
                    }
                    assert!(content.len() <= 32 * 1024 * 1024);
                }
                let result = inputs.publish_native_theorem_candidate(
                    &input,
                    &owner,
                    id,
                    &policy,
                    profile,
                    ConcreteFormat::Tptp,
                );
                if huge {
                    assert_eq!(result.unwrap(), None);
                } else {
                    assert_eq!(
                        result.unwrap_err(),
                        PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("AtpSolve"),
                            work_unit: unit,
                            output_kind: OutputKind::new("SourceTheoremCandidateInputs")
                        }
                        .to_string()
                    );
                }
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 11);
                assert_eq!(collected.abandoned_slots_dropped, usize::from(!huge));
            },
        );
    }
}

#[test]
fn native_theorem_candidate_check_four_parent_caller_matches_genuine_check() {
    use mizar_atp::{
        backend::{BackendCandidateEvidence, EncodedBackendProblem},
        problem::*,
    };
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus};
    use mizar_ir::storage::CollectInput;
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::check_source_theorem_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcSet, VcStatus,
        },
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold, fragment, format) in [
        (false, usize::MAX, LogicFragment::Fof, ConcreteFormat::Tptp),
        (
            true,
            0,
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, threshold, &old.work_unit);
                let storage = owner.storage();
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
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                }
                let generation = GenerationSchemaVersion::new(if open { "" } else { "λ\n\0" });
                let schema = VcSchemaVersion::new(if open { "核\t\0" } else { "" });
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("計算\0"),
                        max_steps: u32::MAX,
                    },
                };
                let profile = LogicProfile::try_new(
                    "caller λ\n\0",
                    fragment,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone())
                    .register_source_theorem_atp_solve(policy.clone(), profile.clone(), format)
                    .unwrap();
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!()
                };
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-check:09"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![BackendProfileId::new("missing-backend")],
                    vec![EvidenceCandidateId::new("declared-evidence")],
                )];
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
                    .flat_map(|row| &row.output_refs)
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 12);
                let parent = |kind: &str| {
                    *outputs
                        .iter()
                        .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                };
                let core_ref = parent("SourceTheoremCore");
                let vc_ref = parent("SourceTheoremVC");
                let discharge_ref = parent("SourceTheoremDischarge");
                let atp_ref = parent("SourceTheoremCandidateInputs");
                let frontend_ref = parent("FrontendOutput");
                let core_handle = storage
                    .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                    .unwrap();
                let vc_handle = storage
                    .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                    .unwrap();
                let discharge_handle = storage
                    .typed_handle::<DischargeOutput>(discharge_ref, discharge_ref.output_kind())
                    .unwrap();
                let atp_handle = storage
                    .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                        atp_ref,
                        atp_ref.output_kind(),
                    )
                    .unwrap();
                let core = storage.get(&core_handle).unwrap();
                let vcs = storage.get(&vc_handle).unwrap();
                let discharge = storage.get(&discharge_handle).unwrap();
                let pair = storage.get(&atp_handle).unwrap();
                let frontend = storage
                    .get(
                        &storage
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                let module = core.module_id();
                let (arena, env) = source_theorem_module_original_inputs(ast, module)
                    .unwrap()
                    .unwrap();
                assert!(
                    source_theorem_module_candidates(ast, &arena, &env)
                        .unwrap()
                        .is_some()
                );
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, module, &env),
                )
                .unwrap();
                let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
                let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena,
                    &metadata.2,
                    &scope,
                    &env,
                    &labels,
                    &resolved,
                    None,
                )
                .unwrap();
                let id = vcs.vcs()[0].id;
                let actual_generation = &vcs.vc(id).unwrap().anchor.generation_schema_version;
                let input = PhaseInput::new(
                    original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        vec![],
                        [atp_ref, discharge_ref, vc_ref, core_ref]
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
                assert_eq!(ast.source_id, core.source_id());
                assert_eq!(vcs.source(), ast.source_id);
                assert_eq!(
                    core.module_id(),
                    &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                );
                assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                assert_eq!(vcs.snapshot(), old.snapshot);
                assert_eq!(vcs.schema_version(), &schema);
                assert_eq!(actual_generation, &generation);
                assert_eq!(vcs.vcs().len(), 1);
                assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(vcs.vcs().iter().all(|row| row.status == VcStatus::Open));
                assert!(
                    discharge
                        .vc_set()
                        .vcs()
                        .iter()
                        .all(|row| row.status == VcStatus::NeedsAtp)
                );
                for row in vcs.seed_accounting() {
                    let SeedOriginRef::ExistingCore { seed } = row.origin else {
                        panic!()
                    };
                    assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                    assert_eq!(
                        core.obligation_seeds().get(seed).unwrap().status,
                        ObligationSeedStatus::Active
                    );
                }
                let before = (
                    input.clone(),
                    ast.clone(),
                    (*core).clone(),
                    (*vcs).clone(),
                    (*discharge).clone(),
                    (*pair).clone(),
                    policy.clone(),
                    profile.clone(),
                    env.clone(),
                    format!("{check:?}"),
                    session.captured().clone(),
                );
                fixture.write(&[0xff]);
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                for verifier in [
                    VerifierPolicy::default(),
                    VerifierPolicy::release()
                        .with_require_kernel_certificates(true)
                        .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
                ] {
                    let expected = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        id,
                        (
                            actual_generation,
                            vcs.schema_version(),
                            profile.clone(),
                            format,
                        ),
                        (&policy, &verifier),
                    )
                    .unwrap();
                    assert_eq!(
                        (
                            &expected.0,
                            &expected.1,
                            &expected.2,
                            &expected.3,
                            &expected.4
                        ),
                        (&*core, &*vcs, &*discharge, &pair.0, &pair.1)
                    );
                    assert_eq!(expected.5.status(), KernelCheckStatus::Accepted);
                    assert_eq!(
                        expected.5.evidence_check_kind(),
                        Some(KernelEvidenceCheckKind::ProofObligation)
                    );
                    assert!(!expected.5.policy_taint());
                    assert_eq!(expected.6.class, CandidatePolicyClass::KernelVerified);
                    let verifier_before = verifier.clone();
                    for _ in 0..2 {
                        assert_eq!(
                            inputs
                                .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                                    &input,
                                    &owner,
                                    id,
                                    (profile.clone(), format),
                                    (&policy, &verifier)
                                )
                                .unwrap(),
                            Some((expected.5.clone(), expected.6.clone()))
                        );
                    }
                    assert_eq!(verifier, verifier_before);
                }
                assert_eq!(
                    (
                        input,
                        ast.clone(),
                        (*storage.get(&core_handle).unwrap()).clone(),
                        (*storage.get(&vc_handle).unwrap()).clone(),
                        (*storage.get(&discharge_handle).unwrap()).clone(),
                        (*storage.get(&atp_handle).unwrap()).clone(),
                        policy,
                        profile,
                        env.clone(),
                        format!("{check:?}"),
                        session.captured().clone()
                    ),
                    before
                );
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 12);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}

#[test]
fn native_theorem_candidate_check_four_parent_caller_preserves_refusals() {
    use mizar_atp::{
        backend::{
            BackendCandidateEvidence, EncodedBackendProblem,
            build_source_theorem_candidate_inputs as build,
        },
        problem::*,
    };
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::CoreIr;
    use mizar_driver::{
        native_theorem_candidate_from_storage_bytes as decode,
        native_theorem_candidate_storage_bytes as encode,
        native_theorem_core_from_storage_bytes as decode_core,
        native_theorem_core_storage_bytes as encode_core,
    };
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::check_source_theorem_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargeOutput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcSet},
    };
    use std::sync::Arc;
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    with_native_summary_parent(
        text,
        false,
        0,
        |fixture, _, old, _, _, (session, ids, snapshots)| {
            let generation = GenerationSchemaVersion::new("λ\n\0");
            let schema = VcSchemaVersion::new("");
            let policy = DischargePolicy::default();
            let profile = LogicProfile::try_new(
                "caller",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
            let disabled = VerifierPolicy::default().with_kernel_evidence_formats([]);
            let normal = VerifierPolicy::default();
            let WorkUnit::Module { module } = &old.work_unit else {
                panic!()
            };
            let unit = IrWorkUnit::new("\"alpha\":\"main\"");
            let prerequisites = |owner: Arc<PhaseOutputPublisher>, candidate| {
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
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone());
                if candidate {
                    builder
                        .register_source_theorem_atp_solve(
                            policy.clone(),
                            profile.clone(),
                            ConcreteFormat::Tptp,
                        )
                        .unwrap();
                }
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-four-parents"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    if candidate {
                        vec![BackendProfileId::new("missing-backend")]
                    } else {
                        vec![]
                    },
                    if candidate {
                        vec![EvidenceCandidateId::new("declared-evidence")]
                    } else {
                        vec![]
                    },
                )];
                CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner)
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap()
            };
            let outputs = |submission: &mizar_driver::driver::BuildSubmission| {
                submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter().cloned())
                    .collect::<Vec<_>>()
            };
            let input = |owner: &PhaseOutputPublisher, parents: Vec<AnyPhaseOutputRef>| {
                PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        source_key(&session.captured().snapshot().source_versions[0]),
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
            let baseline = publisher(old.snapshot, 0, &old.work_unit);
            let submission = prerequisites(baseline.clone(), true);
            let (inputs, _) = native_disk_lookup_context(&submission, ids);
            let refs = outputs(&submission);
            assert_eq!(refs.len(), 12);
            let parent = |kind: &str| {
                refs.iter()
                    .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                    .unwrap()
                    .clone()
            };
            let core_ref = parent("SourceTheoremCore");
            let vc_ref = parent("SourceTheoremVC");
            let discharge_ref = parent("SourceTheoremDischarge");
            let atp_ref = parent("SourceTheoremCandidateInputs");
            let frontend_ref = parent("FrontendOutput");
            let storage = baseline.storage();
            let core = storage
                .get(
                    &storage
                        .typed_handle::<CoreIr>(&core_ref, core_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let vcs = storage
                .get(
                    &storage
                        .typed_handle::<VcSet>(&vc_ref, vc_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let discharge = storage
                .get(
                    &storage
                        .typed_handle::<DischargeOutput>(
                            &discharge_ref,
                            discharge_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let pair = storage
                .get(
                    &storage
                        .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                            &atp_ref,
                            atp_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let frontend = storage
                .get(
                    &storage
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            &frontend_ref,
                            frontend_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let ast = frontend.ast.as_ref().unwrap();
            let resolver_module = core.module_id();
            let (arena, env) = source_theorem_module_original_inputs(ast, resolver_module)
                .unwrap()
                .unwrap();
            assert!(
                source_theorem_module_candidates(ast, &arena, &env)
                    .unwrap()
                    .is_some()
            );
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, resolver_module, &env),
            )
            .unwrap();
            let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
            let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
            let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena,
                &metadata.2,
                &scope,
                &env,
                &labels,
                &resolved,
                None,
            )
            .unwrap();
            let id = vcs.vcs()[0].id;
            let expected = fresh(
                &arena,
                &check,
                &env,
                old.snapshot,
                id,
                (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                (&policy, &normal),
            )
            .unwrap();
            assert_eq!(
                (
                    &expected.0,
                    &expected.1,
                    &expected.2,
                    &expected.3,
                    &expected.4
                ),
                (&*core, &*vcs, &*discharge, &pair.0, &pair.1)
            );
            let four = input(
                &baseline,
                vec![
                    core_ref.clone(),
                    vc_ref.clone(),
                    discharge_ref.clone(),
                    atp_ref.clone(),
                ],
            );
            let three = input(
                &baseline,
                vec![core_ref.clone(), vc_ref.clone(), discharge_ref.clone()],
            );
            for verifier in [
                disabled.clone(),
                normal
                    .clone()
                    .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
            ] {
                let expected = fresh(
                    &arena,
                    &check,
                    &env,
                    old.snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                )
                .unwrap_err();
                assert_eq!(
                    inputs
                        .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                            &four,
                            &baseline,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier)
                        )
                        .unwrap_err(),
                    expected
                );
            }
            let bad = LogicProfile::try_new(
                "bad",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::GuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
            for (selected, encoding, format) in [
                (VcId::new(usize::MAX), &bad, ConcreteFormat::SmtLib),
                (id, &bad, ConcreteFormat::Tptp),
                (id, &profile, ConcreteFormat::SmtLib),
            ] {
                let expected = inputs
                    .native_theorem_candidate_with_core_vc_discharge_parents(
                        &three,
                        &baseline,
                        selected,
                        &policy,
                        encoding.clone(),
                        format,
                    )
                    .unwrap_err();
                assert_eq!(
                    inputs
                        .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                            &four,
                            &baseline,
                            selected,
                            (encoding.clone(), format),
                            (&policy, &disabled)
                        )
                        .unwrap_err(),
                    expected
                );
            }
            let foreign=Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
            let (_, foreign_submission, adapter, handle, _) =
                native_disk_frontend_record_submission(&foreign, ids, snapshots, 0);
            let (foreign_inputs, foreign_old) =
                native_disk_lookup_context(&foreign_submission, ids);
            let foreign_owner = adapter.publisher();
            let frontend_input = PhaseInput::new(
                foreign_old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    foreign_old.snapshot,
                    foreign_old.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            foreign_owner,
                            foreign_old.snapshot,
                            handle.erase(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            let foreign_core = foreign_inputs
                .native_theorem_core_with_frontend_parent(&frontend_input, foreign_owner)
                .unwrap()
                .unwrap();
            let foreign_frontend = foreign_owner.storage().get(&handle).unwrap();
            let foreign_ast = foreign_frontend.ast.as_ref().unwrap();
            assert_ne!(foreign_ast.source_id, ast.source_id);
            let upstream = foreign_inputs
                .native_theorem_candidate_with_core_vc_discharge_parents(
                    &three,
                    &baseline,
                    VcId::new(usize::MAX),
                    &policy,
                    bad.clone(),
                    ConcreteFormat::SmtLib,
                )
                .unwrap_err();
            assert_eq!(
                foreign_inputs
                    .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                        &four,
                        &baseline,
                        VcId::new(usize::MAX),
                        (bad.clone(), ConcreteFormat::SmtLib),
                        (&policy, &disabled)
                    )
                    .unwrap_err(),
                upstream
            );
            let core_bytes = encode_core(foreign_ast, resolver_module, &foreign_core)
                .unwrap()
                .unwrap();
            let rebound = decode_core(&core_bytes, ast.source_id, resolver_module)
                .unwrap()
                .unwrap();
            assert_eq!(rebound.source_id(), ast.source_id);
            assert_eq!(rebound.module_id(), resolver_module);
            let foreign_vcs = generate(&rebound, old.snapshot, &generation, &schema).unwrap();
            let foreign_discharge = try_discharge(DischargeInput {
                vc_set: &foreign_vcs,
                policy: &policy,
            })
            .unwrap();
            let foreign_pair = build(
                &rebound,
                &foreign_vcs,
                foreign_vcs.vcs()[0].id,
                &foreign_discharge,
                &policy,
                profile.clone(),
                ConcreteFormat::Tptp,
            )
            .unwrap();
            assert_ne!(foreign_pair, *pair);
            let rebound_ast = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
                &foreign_ast.canonical_bytes().unwrap(),
                ast.source_id,
            )
            .unwrap();
            let foreign_bytes = encode(
                &rebound_ast,
                resolver_module,
                &rebound,
                (old.snapshot, &generation, &schema, foreign_vcs.vcs()[0].id),
                (&policy, &profile, ConcreteFormat::Tptp),
                &foreign_pair,
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                decode(&foreign_bytes, ast.source_id, resolver_module, old.snapshot).unwrap(),
                Some(foreign_pair.clone())
            );
            let original_bytes = encode(
                ast,
                resolver_module,
                &core,
                (old.snapshot, &generation, &schema, id),
                (&policy, &profile, ConcreteFormat::Tptp),
                &pair,
            )
            .unwrap()
            .unwrap();
            let refuse = |source: SourceLoadInputs<'_>,
                          query: &PhaseInput,
                          publisher: &PhaseOutputPublisher| {
                source
                    .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                        query,
                        publisher,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &disabled),
                    )
                    .unwrap_err()
            };
            for case in ["pair", "lineage", "schema", "binding"] {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                let storage = owner.storage();
                let submission = prerequisites(owner.clone(), false);
                let (inputs, _) = native_disk_lookup_context(&submission, ids);
                let refs = outputs(&submission);
                assert_eq!(refs.len(), 11);
                let parent = |kind: &str| {
                    refs.iter()
                        .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                        .clone()
                };
                let parents = [
                    parent("SourceTheoremCore"),
                    parent("SourceTheoremVC"),
                    parent("SourceTheoremDischarge"),
                ];
                assert_eq!(
                    *storage
                        .get(
                            &storage
                                .typed_handle::<CoreIr>(&parents[0], parents[0].output_kind())
                                .unwrap()
                        )
                        .unwrap(),
                    expected.0
                );
                assert_eq!(
                    *storage
                        .get(
                            &storage
                                .typed_handle::<VcSet>(&parents[1], parents[1].output_kind())
                                .unwrap()
                        )
                        .unwrap(),
                    expected.1
                );
                assert_eq!(
                    *storage
                        .get(
                            &storage
                                .typed_handle::<DischargeOutput>(
                                    &parents[2],
                                    parents[2].output_kind()
                                )
                                .unwrap()
                        )
                        .unwrap(),
                    expected.2
                );
                let three = input(&owner, parents.to_vec());
                assert_eq!(
                    inputs
                        .native_theorem_candidate_with_core_vc_discharge_parents(
                            &three,
                            &owner,
                            id,
                            &policy,
                            profile.clone(),
                            ConcreteFormat::Tptp
                        )
                        .unwrap(),
                    Some((*pair).clone())
                );
                let payload = if case == "pair" {
                    foreign_pair.clone()
                } else {
                    (*pair).clone()
                };
                let bytes = if case == "pair" {
                    foreign_bytes.clone()
                } else {
                    original_bytes.clone()
                };
                let phase = IrPipelinePhase::new("AtpSolve");
                let kind = OutputKind::new("SourceTheoremCandidateInputs");
                let version = SchemaVersion::new(if case == "schema" { 2 } else { 1 });
                let source_id = ast.source_id;
                let module = resolver_module.clone();
                let snapshot = old.snapshot;
                let mut named = parents[0].lineage().named_input_hashes.clone();
                if case == "binding" {
                    named[0].digest = hash(0x64);
                }
                let atp = owner
                    .publish(PublishOutputInput {
                        slot: owner.allocate(
                            snapshot,
                            phase.clone(),
                            unit.clone(),
                            kind.clone(),
                            version,
                        ),
                        snapshot,
                        phase,
                        work_unit: unit.clone(),
                        output_kind: kind,
                        schema_version: version,
                        payload: payload.clone(),
                        canonical_payload: Some(bytes.clone()),
                        storage_payload: Some(bytes),
                        decode: BlobDecoder::new(move |bytes| {
                            decode(bytes, source_id, &module, snapshot)
                                .map_err(BlobDecodeError::new)?
                                .ok_or_else(|| BlobDecodeError::new("negative ATP replay"))
                        }),
                        parents: parents[..if case == "lineage" { 2 } else { 3 }].to_vec(),
                        named_input_hashes: named,
                        side_tables: IrSideTables::default(),
                        origin: OutputOrigin::PackageSource,
                        target: PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                assert_eq!(*storage.get(&atp).unwrap(), payload);
                let mut parents = parents.to_vec();
                parents.push(atp.erase());
                let four = input(&owner, parents);
                assert_eq!(
                    refuse(inputs, &four, &owner),
                    match case {
                        "pair" => "native theorem candidate check complete ATP mismatch",
                        "lineage" => "native theorem candidate check ATP parent link mismatch",
                        _ => "native theorem candidate check parent binding mismatch",
                    }
                );
                if case == "pair" {
                    let lower = inputs
                        .native_theorem_candidate_with_core_vc_discharge_parents(
                            &three,
                            &owner,
                            id,
                            &policy,
                            bad.clone(),
                            ConcreteFormat::SmtLib,
                        )
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                                &four,
                                &owner,
                                id,
                                (bad.clone(), ConcreteFormat::SmtLib),
                                (&policy, &disabled)
                            )
                            .unwrap_err(),
                        lower
                    );
                }
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 12);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            }
            for case in ["roles", "foreign", "obsolete", "collected"] {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                let storage = owner.storage();
                let submission = prerequisites(owner.clone(), true);
                let (inputs, _) = native_disk_lookup_context(&submission, ids);
                let refs = outputs(&submission);
                let parent = |kind: &str| {
                    refs.iter()
                        .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                        .clone()
                };
                let parents = [
                    parent("SourceTheoremCore"),
                    parent("SourceTheoremVC"),
                    parent("SourceTheoremDischarge"),
                    parent("SourceTheoremCandidateInputs"),
                ];
                let four = input(&owner, parents.to_vec());
                if case == "collected" {
                    assert_eq!(
                        storage
                            .collect(CollectInput {
                                snapshot: old.snapshot,
                                protected_outputs: parents[..3]
                                    .iter()
                                    .map(|parent| parent.output())
                                    .collect()
                            })
                            .outputs_dropped,
                        9
                    );
                    let lower = storage
                        .validate_handle(&parents[3])
                        .unwrap_err()
                        .to_string();
                    assert_eq!(refuse(inputs, &four, &owner), lower);
                } else if case == "roles" {
                    let mut wrong = parents.to_vec();
                    wrong[3] = parent("SourceTheoremPreparation");
                    assert_eq!(
                        refuse(inputs, &input(&owner, wrong), &owner),
                        "native theorem candidate check requires one ATP parent"
                    );
                    for wrong in [
                        parents[..3].to_vec(),
                        [parents.as_slice(), &[parent("SourceTheoremPreparation")]].concat(),
                    ] {
                        assert_eq!(
                            refuse(inputs, &input(&owner, wrong), &owner),
                            "native theorem candidate check requires four parents"
                        );
                    }
                } else {
                    let alien = publisher(old.snapshot, 0, &old.work_unit);
                    let publisher = if case == "foreign" { &alien } else { &owner };
                    if case == "obsolete" {
                        owner.mark_obsolete(old.snapshot).unwrap();
                    }
                    let lower = if case == "foreign" {
                        alien
                            .storage()
                            .validate_handle(&parents[0])
                            .unwrap_err()
                            .to_string()
                    } else {
                        owner
                            .validate_published_output(old.snapshot, &parents[0])
                            .unwrap_err()
                            .to_string()
                    };
                    assert_eq!(refuse(inputs, &four, publisher), lower);
                }
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected" { 3 } else { 12 }
                );
                assert_eq!(collected.abandoned_slots_dropped, 0);
            }
            let collected = baseline.storage().collect(CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 12);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        },
    );
}

#[test]
fn native_theorem_candidate_check_storage_replays_genuine_inputs() {
    use mizar_atp::{
        backend::{BackendCandidateEvidence, EncodedBackendProblem},
        problem::*,
    };
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_candidate_check_from_storage_bytes as decode,
        native_theorem_candidate_check_storage_bytes as encode,
        native_theorem_candidate_storage_bytes as old_encode,
    };
    use mizar_ir::storage::CollectInput;
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::check_source_theorem_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcSet, VcStatus,
        },
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold, fragment, format) in [
        (false, usize::MAX, LogicFragment::Fof, ConcreteFormat::Tptp),
        (
            true,
            0,
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, threshold, &old.work_unit);
                let storage = owner.storage();
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
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                }
                let generation = GenerationSchemaVersion::new(if open { "" } else { "λ\n\0" });
                let schema = VcSchemaVersion::new(if open { "核\t\0" } else { "" });
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("計算\0"),
                        max_steps: u32::MAX,
                    },
                };
                let profile = LogicProfile::try_new(
                    "caller λ\n\0",
                    fragment,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone())
                    .register_source_theorem_atp_solve(policy.clone(), profile.clone(), format)
                    .unwrap();
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!()
                };
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-check:09"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![BackendProfileId::new("missing-backend")],
                    vec![EvidenceCandidateId::new("declared-evidence")],
                )];
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
                    .flat_map(|row| &row.output_refs)
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 12);
                let parent = |kind: &str| {
                    *outputs
                        .iter()
                        .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                };
                let core_ref = parent("SourceTheoremCore");
                let vc_ref = parent("SourceTheoremVC");
                let discharge_ref = parent("SourceTheoremDischarge");
                let atp_ref = parent("SourceTheoremCandidateInputs");
                let frontend_ref = parent("FrontendOutput");
                let core_handle = storage
                    .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                    .unwrap();
                let vc_handle = storage
                    .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                    .unwrap();
                let discharge_handle = storage
                    .typed_handle::<DischargeOutput>(discharge_ref, discharge_ref.output_kind())
                    .unwrap();
                let atp_handle = storage
                    .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                        atp_ref,
                        atp_ref.output_kind(),
                    )
                    .unwrap();
                let core = storage.get(&core_handle).unwrap();
                let vcs = storage.get(&vc_handle).unwrap();
                let discharge = storage.get(&discharge_handle).unwrap();
                let pair = storage.get(&atp_handle).unwrap();
                let frontend = storage
                    .get(
                        &storage
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                let module = core.module_id();
                let (arena, env) = source_theorem_module_original_inputs(ast, module)
                    .unwrap()
                    .unwrap();
                assert!(
                    source_theorem_module_candidates(ast, &arena, &env)
                        .unwrap()
                        .is_some()
                );
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, module, &env),
                )
                .unwrap();
                let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
                let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena,
                    &metadata.2,
                    &scope,
                    &env,
                    &labels,
                    &resolved,
                    None,
                )
                .unwrap();
                let id = vcs.vcs()[0].id;
                let actual_generation = &vcs.vc(id).unwrap().anchor.generation_schema_version;
                let input = PhaseInput::new(
                    original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        vec![],
                        [atp_ref, discharge_ref, vc_ref, core_ref]
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
                assert_eq!(ast.source_id, core.source_id());
                assert_eq!(vcs.source(), ast.source_id);
                assert_eq!(
                    core.module_id(),
                    &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                );
                assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                assert_eq!(vcs.snapshot(), old.snapshot);
                assert_eq!(vcs.schema_version(), &schema);
                assert_eq!(actual_generation, &generation);
                assert_eq!(vcs.vcs().len(), 1);
                assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(vcs.vcs().iter().all(|row| row.status == VcStatus::Open));
                assert!(
                    discharge
                        .vc_set()
                        .vcs()
                        .iter()
                        .all(|row| row.status == VcStatus::NeedsAtp)
                );
                for row in vcs.seed_accounting() {
                    let SeedOriginRef::ExistingCore { seed } = row.origin else {
                        panic!()
                    };
                    assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                    assert_eq!(
                        core.obligation_seeds().get(seed).unwrap().status,
                        ObligationSeedStatus::Active
                    );
                }
                let before = (
                    input.clone(),
                    ast.clone(),
                    (*core).clone(),
                    (*vcs).clone(),
                    (*discharge).clone(),
                    (*pair).clone(),
                    policy.clone(),
                    profile.clone(),
                    env.clone(),
                    format!("{check:?}"),
                    session.captured().clone(),
                );
                fixture.write(&[0xff]);
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                for verifier in [
                    VerifierPolicy::default(),
                    VerifierPolicy::release()
                        .with_require_kernel_certificates(true)
                        .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
                ] {
                    let expected = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        id,
                        (
                            actual_generation,
                            vcs.schema_version(),
                            profile.clone(),
                            format,
                        ),
                        (&policy, &verifier),
                    )
                    .unwrap();
                    assert_eq!(
                        (
                            &expected.0,
                            &expected.1,
                            &expected.2,
                            &expected.3,
                            &expected.4
                        ),
                        (&*core, &*vcs, &*discharge, &pair.0, &pair.1)
                    );
                    assert_eq!(expected.5.status(), KernelCheckStatus::Accepted);
                    assert_eq!(
                        expected.5.evidence_check_kind(),
                        Some(KernelEvidenceCheckKind::ProofObligation)
                    );
                    assert!(!expected.5.policy_taint());
                    assert_eq!(expected.6.class, CandidatePolicyClass::KernelVerified);
                    let checked = inputs
                        .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                            &input,
                            &owner,
                            id,
                            (profile.clone(), format),
                            (&policy, &verifier),
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(checked, (expected.5, expected.6));
                    let inner = old_encode(
                        ast,
                        module,
                        &core,
                        (old.snapshot, actual_generation, vcs.schema_version(), id),
                        (&policy, &profile, format),
                        &pair,
                    )
                    .unwrap()
                    .unwrap();
                    let fingerprint = verifier.policy_fingerprint();
                    let mut bytes =
                        b"mizar-driver/native-theorem-candidate-check-storage/v1\0".to_vec();
                    for field in [inner.as_slice(), fingerprint.as_bytes().as_slice()] {
                        bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                        bytes.extend_from_slice(field);
                    }
                    let before_codec = (
                        verifier.clone(),
                        inner.clone(),
                        checked.clone(),
                        bytes.clone(),
                    );
                    for _ in 0..2 {
                        assert_eq!(
                            encode(
                                ast,
                                module,
                                &core,
                                (old.snapshot, actual_generation, vcs.schema_version(), id),
                                (&policy, &profile, format),
                                &verifier,
                                (&pair, &checked)
                            )
                            .unwrap(),
                            Some(bytes.clone())
                        );
                        assert_eq!(
                            decode(
                                &bytes,
                                ast,
                                module,
                                (old.snapshot, actual_generation, vcs.schema_version(), id),
                                (&policy, &profile, format),
                                &verifier
                            )
                            .unwrap(),
                            Some(checked.clone())
                        );
                    }
                    assert_eq!((verifier.clone(), inner, checked, bytes), before_codec);
                }
                assert_eq!(
                    (
                        input,
                        ast.clone(),
                        (*storage.get(&core_handle).unwrap()).clone(),
                        (*storage.get(&vc_handle).unwrap()).clone(),
                        (*storage.get(&discharge_handle).unwrap()).clone(),
                        (*storage.get(&atp_handle).unwrap()).clone(),
                        policy,
                        profile,
                        env.clone(),
                        format!("{check:?}"),
                        session.captured().clone()
                    ),
                    before
                );
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 12);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
}

#[test]
fn native_theorem_candidate_check_storage_preserves_refusals() {
    use mizar_atp::{
        backend::{BackendCandidateEvidence, EncodedBackendProblem},
        problem::{
            ConcreteFormat::{SmtLib, Tptp},
            EqualitySupport::{Supported, Unsupported},
            LogicFragment::{Fof, TffLike},
            LogicProfile,
            NativePropertySupport::{
                Supported as NativeSupported, Unsupported as NativeUnsupported,
            },
            QuantifierPolicy::{FirstOrder, PropositionalOnly},
            SoftTypeStrategy::{GuardPredicates, ScopedGuardPredicates},
        },
    };
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_core::core_ir::CoreIr;
    use mizar_driver::{
        native_theorem_candidate_check_from_storage_bytes as decode,
        native_theorem_candidate_check_storage_bytes as encode,
        native_theorem_candidate_from_storage_bytes as old_decode,
        native_theorem_candidate_storage_bytes as old_encode,
    };
    use mizar_ir::storage::CollectInput;
    use mizar_proof::{
        policy::{
            BuildMode, ExternalEvidenceMode, KernelEvidenceFormat, OpenObligationMode,
            PolicyAssumptionMode, VerifierPolicy,
        },
        status::check_source_theorem_candidate,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    with_native_summary_parent(
        text,
        false,
        0,
        |fixture, _, old, _, _, (session, ids, snapshots)| {
            let owner = publisher(old.snapshot, 0, &old.work_unit);
            let storage = owner.storage();
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
                ("AtpSolve", "SourceTheoremCandidateInputs"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let generation = GenerationSchemaVersion::new("λ\n\0");
            let schema = VcSchemaVersion::new("");
            let policy = DischargePolicy {
                computation_limit: ComputationLimit {
                    policy: PolicyKey::new("計算\0"),
                    max_steps: 0,
                },
            };
            let profile = LogicProfile::try_new(
                "caller",
                Fof,
                Supported,
                FirstOrder,
                ScopedGuardPredicates,
                NativeSupported,
                [Tptp, SmtLib].into_iter().collect(),
            )
            .unwrap();
            let verifier = VerifierPolicy::default();
            let disabled = verifier.clone().with_kernel_evidence_formats([]);
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
                .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                .register_source_theorem_vc_discharge(policy.clone())
                .register_source_theorem_atp_solve(policy.clone(), profile.clone(), Tptp)
                .unwrap();
            let (_, mut submit) = fixture.submit_request(vec![], vec![]);
            let WorkUnit::Module { module } = &old.work_unit else {
                panic!()
            };
            submit.vc_descriptors = vec![VcTaskDescriptor::new(
                VcTaskDescriptorId::new("opaque-checked-storage"),
                module.clone(),
                VcOrderKey::new("order"),
                vec![BackendProfileId::new("missing-backend")],
                vec![EvidenceCandidateId::new("declared-evidence")],
            )];
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
                .flat_map(|row| &row.output_refs)
                .collect::<Vec<_>>();
            assert_eq!(outputs.len(), 12);
            let parent = |kind: &str| {
                *outputs
                    .iter()
                    .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                    .unwrap()
            };
            let core_ref = parent("SourceTheoremCore");
            let vc_ref = parent("SourceTheoremVC");
            let d_ref = parent("SourceTheoremDischarge");
            let atp_ref = parent("SourceTheoremCandidateInputs");
            let frontend_ref = parent("FrontendOutput");
            let core = storage
                .get(
                    &storage
                        .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let vcs = storage
                .get(
                    &storage
                        .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let discharge = storage
                .get(
                    &storage
                        .typed_handle::<DischargeOutput>(d_ref, d_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let pair = storage
                .get(
                    &storage
                        .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                            atp_ref,
                            atp_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let frontend = storage
                .get(
                    &storage
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend_ref,
                            frontend_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let ast = frontend.ast.as_ref().unwrap();
            let module = core.module_id();
            let id = vcs.vcs()[0].id;
            let proof = |ast: &<MizarParserSeam as ParserSeam>::Ast,
                         vc_context: (
                BuildSnapshotId,
                &GenerationSchemaVersion,
                &VcSchemaVersion,
                VcId,
            ),
                         atp_context: (
                &DischargePolicy,
                &LogicProfile,
                mizar_atp::problem::ConcreteFormat,
            ),
                         verifier: &VerifierPolicy| {
                let (arena, env) =
                    mizar_resolve::symbols::source_theorem_module_original_inputs(ast, module)
                        .unwrap()
                        .unwrap();
                assert!(
                    mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)
                        .unwrap()
                        .is_some()
                );
                let scope =
                    mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(
                        mizar_resolve::names::SourceVariableScopeInput::new(ast, module, &env),
                    )
                    .unwrap();
                let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                    &arena, &env, &scope,
                )
                .unwrap();
                let (scope, labels, resolved) =
                    mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                let check=mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(&arena,&metadata.2,&scope,&env,&labels,&resolved,None).unwrap();
                check_source_theorem_candidate(
                    &arena,
                    &check,
                    &env,
                    vc_context.0,
                    vc_context.3,
                    (
                        vc_context.1,
                        vc_context.2,
                        atp_context.1.clone(),
                        atp_context.2,
                    ),
                    (atp_context.0, verifier),
                )
            };
            let vc_context = (old.snapshot, &generation, &schema, id);
            let atp_context = (&policy, &profile, Tptp);
            let fresh = proof(ast, vc_context, atp_context, &verifier).unwrap();
            assert_eq!(
                (&fresh.0, &fresh.1, &fresh.2, &fresh.3, &fresh.4),
                (&*core, &*vcs, &*discharge, &pair.0, &pair.1)
            );
            let input = PhaseInput::new(
                original.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    original.identities().input_hash(),
                    vec![],
                    [core_ref, vc_ref, d_ref, atp_ref]
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
            let checked = inputs
                .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                    &input,
                    &owner,
                    id,
                    (profile.clone(), Tptp),
                    (&policy, &verifier),
                )
                .unwrap()
                .unwrap();
            assert_eq!(checked, (fresh.5, fresh.6));
            let inner = old_encode(ast, module, &core, vc_context, atp_context, &pair)
                .unwrap()
                .unwrap();
            let frame = |inner: &[u8], fingerprint: &[u8]| {
                let mut bytes =
                    b"mizar-driver/native-theorem-candidate-check-storage/v1\0".to_vec();
                for field in [inner, fingerprint] {
                    bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                    bytes.extend_from_slice(field);
                }
                bytes
            };
            let bytes = frame(&inner, verifier.policy_fingerprint().as_bytes());
            let replay = |bytes: &[u8], verifier: &VerifierPolicy| {
                decode(bytes, ast, module, vc_context, atp_context, verifier)
            };
            let mut policies = vec![
                verifier.clone().with_schema_version(u16::MAX),
                verifier.clone().with_profile_id("λ\n\0"),
                verifier.clone().with_build_mode(BuildMode::Development),
                verifier.clone().with_require_kernel_certificates(false),
                verifier
                    .clone()
                    .with_external_evidence(ExternalEvidenceMode::RecordDevelopment),
                verifier
                    .clone()
                    .with_open_obligation(OpenObligationMode::AllowPolicyOpen),
                verifier
                    .clone()
                    .with_policy_assumption(PolicyAssumptionMode::Record),
                verifier
                    .clone()
                    .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
                verifier.clone().with_checker_schema_version(u16::MAX),
                verifier.clone().with_instance_search_limits(Some([0; 4])),
            ];
            for index in 0..4 {
                let mut limits = [0; 4];
                limits[index] = u64::MAX;
                policies.push(verifier.clone().with_instance_search_limits(Some(limits)));
            }
            let mut fingerprints = vec![verifier.policy_fingerprint()];
            for policy in policies {
                let fresh = proof(ast, vc_context, atp_context, &policy).unwrap();
                assert_eq!((&fresh.0, &fresh.3, &fresh.4), (&*core, &pair.0, &pair.1));
                let expected = (fresh.5, fresh.6);
                assert_eq!(expected, checked);
                let fingerprint = policy.policy_fingerprint();
                assert!(!fingerprints.contains(&fingerprint));
                fingerprints.push(fingerprint);
                let variant = frame(&inner, fingerprint.as_bytes());
                assert_ne!(variant, bytes);
                assert_eq!(
                    encode(
                        ast,
                        module,
                        &core,
                        vc_context,
                        atp_context,
                        &policy,
                        (&pair, &expected)
                    )
                    .unwrap(),
                    Some(variant.clone())
                );
                assert_eq!(replay(&variant, &policy).unwrap(), Some(expected));
                assert_eq!(replay(&variant, &verifier).unwrap(), None);
                assert_eq!(replay(&bytes, &policy).unwrap(), None);
            }
            let mut equal_pair_cases = Vec::new();
            for case in 1..=13 {
                let generation =
                    GenerationSchemaVersion::new(if case == 1 { "changed" } else { "λ\n\0" });
                let schema = VcSchemaVersion::new(if case == 2 { "changed" } else { "" });
                let selected = if case == 3 { VcId::new(usize::MAX) } else { id };
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new(if case == 4 { "changed" } else { "計算\0" }),
                        max_steps: if case == 5 { u32::MAX } else { 0 },
                    },
                };
                let profile = LogicProfile::try_new(
                    if case == 6 { "changed" } else { "caller" },
                    [Fof, TffLike][usize::from(case == 7)],
                    [Supported, Unsupported][usize::from(case == 8)],
                    [FirstOrder, PropositionalOnly][usize::from(case == 9)],
                    [ScopedGuardPredicates, GuardPredicates][usize::from(case == 10)],
                    [NativeSupported, NativeUnsupported][usize::from(case == 11)],
                    [Tptp, SmtLib]
                        .into_iter()
                        .filter(|format| case != 12 || format == &Tptp)
                        .collect(),
                )
                .unwrap();
                let format = [Tptp, SmtLib][usize::from(case == 13)];
                let context = (old.snapshot, &generation, &schema, selected);
                let atp = (&policy, &profile, format);
                let old = old_encode(ast, module, &core, context, atp, &pair);
                let fresh = proof(ast, context, atp, &verifier);
                match fresh {
                    Ok(fresh) => {
                        let new_pair = (fresh.3, fresh.4);
                        let new_checked = (fresh.5, fresh.6);
                        let regenerated =
                            old_encode(ast, module, &fresh.0, context, atp, &new_pair)
                                .unwrap()
                                .unwrap();
                        assert_ne!(regenerated, inner);
                        if new_pair == *pair {
                            equal_pair_cases.push(case);
                            assert!(old.unwrap().is_some(), "equal-pair producer case {case}");
                        }
                        let variant = frame(&regenerated, verifier.policy_fingerprint().as_bytes());
                        assert_eq!(
                            encode(
                                ast,
                                module,
                                &fresh.0,
                                context,
                                atp,
                                &verifier,
                                (&new_pair, &new_checked)
                            )
                            .unwrap(),
                            Some(variant.clone())
                        );
                        assert_eq!(
                            decode(&variant, ast, module, context, atp, &verifier).unwrap(),
                            Some(new_checked)
                        );
                        assert_eq!(replay(&variant, &verifier).unwrap(), None);
                        assert_eq!(
                            decode(&bytes, ast, module, context, atp, &verifier).unwrap(),
                            None
                        );
                    }
                    Err(expected) => {
                        let old_error = old.unwrap_err();
                        assert_eq!(
                            encode(
                                ast,
                                module,
                                &core,
                                context,
                                atp,
                                &disabled,
                                (&pair, &checked)
                            )
                            .unwrap_err(),
                            old_error
                        );
                        assert_eq!(
                            decode(&bytes, ast, module, context, atp, &verifier).unwrap_err(),
                            expected
                        );
                    }
                }
            }
            assert!(
                !equal_pair_cases.is_empty(),
                "no equal-pair producer cases among1..=13"
            );
            let foreign=Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
            let (_, foreign_submission, adapter, handle, _) =
                native_disk_frontend_record_submission(&foreign, ids, snapshots, 0);
            let foreign_frontend = adapter.publisher().storage().get(&handle).unwrap();
            let foreign_ast = foreign_frontend.ast.as_ref().unwrap();
            assert_ne!(foreign_ast.source_id, ast.source_id);
            let other_context = (
                foreign_submission.session.captured().snapshot().id,
                &generation,
                &schema,
                id,
            );
            let other = proof(foreign_ast, other_context, atp_context, &verifier).unwrap();
            let other_pair = (other.3, other.4);
            let other_checked = (other.5, other.6);
            assert_ne!(other_checked, checked);
            let other_inner = old_encode(
                foreign_ast,
                module,
                &other.0,
                other_context,
                atp_context,
                &other_pair,
            )
            .unwrap()
            .unwrap();
            assert!(
                encode(
                    foreign_ast,
                    module,
                    &other.0,
                    other_context,
                    atp_context,
                    &verifier,
                    (&other_pair, &other_checked)
                )
                .unwrap()
                .is_some()
            );
            assert_eq!(
                old_encode(
                    ast,
                    module,
                    &other.0,
                    other_context,
                    atp_context,
                    &other_pair
                )
                .unwrap(),
                None
            );
            assert_eq!(
                encode(
                    ast,
                    module,
                    &other.0,
                    other_context,
                    atp_context,
                    &disabled,
                    (&other_pair, &other_checked)
                )
                .unwrap(),
                None
            );
            assert!(
                old_encode(ast, module, &core, vc_context, atp_context, &pair)
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                encode(
                    ast,
                    module,
                    &core,
                    vc_context,
                    atp_context,
                    &verifier,
                    (&pair, &other_checked)
                )
                .unwrap(),
                None
            );
            let rebound = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
                &ast.canonical_bytes().unwrap(),
                foreign_ast.source_id,
            )
            .unwrap();
            for (source, snapshot) in [
                (ast, other_context.0),
                (&rebound, old.snapshot),
                (&rebound, other_context.0),
            ] {
                let context = (snapshot, &generation, &schema, id);
                let old = old_decode(&inner, source.source_id, module, snapshot)
                    .unwrap()
                    .unwrap();
                let fresh = proof(source, context, atp_context, &verifier).unwrap();
                let fresh_pair = (fresh.3, fresh.4);
                let fresh_checked = (fresh.5, fresh.6);
                let regenerated =
                    old_encode(source, module, &fresh.0, context, atp_context, &fresh_pair)
                        .unwrap()
                        .unwrap();
                let expected = (fresh_pair == old && regenerated == inner).then_some(fresh_checked);
                assert_eq!(
                    decode(&bytes, source, module, context, atp_context, &verifier).unwrap(),
                    expected
                );
            }
            let old_other = old_decode(&inner, foreign_ast.source_id, module, other_context.0)
                .unwrap()
                .unwrap();
            assert_ne!(old_other, other_pair);
            assert_eq!(
                decode(
                    &bytes,
                    foreign_ast,
                    module,
                    other_context,
                    atp_context,
                    &verifier
                )
                .unwrap(),
                None
            );
            let wrong_module =
                ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other"));
            assert_eq!(
                old_decode(&inner, ast.source_id, &wrong_module, old.snapshot).unwrap(),
                None
            );
            assert_eq!(
                decode(
                    &bytes,
                    ast,
                    &wrong_module,
                    vc_context,
                    atp_context,
                    &verifier
                )
                .unwrap(),
                None
            );
            let private = Fixture::new(include_str!(
                "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
            ).replacen("thus X = X;", "thus X = 0;", 1).as_bytes());
            let (_, _, adapter, handle, _) =
                native_disk_frontend_record_submission(&private, ids, snapshots, 0);
            let private_frontend = adapter.publisher().storage().get(&handle).unwrap();
            let private_ast = private_frontend.ast.as_ref().unwrap();
            let huge_generation = GenerationSchemaVersion::new("x".repeat(32 * 1024 * 1024));
            let huge_context = (old.snapshot, &huge_generation, &schema, id);
            let old_error =
                old_encode(private_ast, module, &core, huge_context, atp_context, &pair)
                    .unwrap_err();
            assert_eq!(
                encode(
                    private_ast,
                    module,
                    &core,
                    huge_context,
                    atp_context,
                    &disabled,
                    (&pair, &checked)
                )
                .unwrap_err(),
                old_error
            );
            let mut unsupported_inner = inner.clone();
            assert_eq!(*unsupported_inner.last().unwrap(), 0);
            *unsupported_inner.last_mut().unwrap() = 1;
            let inner_error =
                old_decode(&unsupported_inner, ast.source_id, module, old.snapshot).unwrap_err();
            let wrong_hash = disabled.policy_fingerprint();
            assert_eq!(
                replay(&frame(&unsupported_inner, wrong_hash.as_bytes()), &verifier).unwrap_err(),
                inner_error
            );
            let truncated = &inner[..inner.len() - 1];
            assert_eq!(
                old_decode(truncated, ast.source_id, module, old.snapshot).unwrap(),
                None
            );
            assert_eq!(
                replay(&frame(truncated, wrong_hash.as_bytes()), &verifier).unwrap(),
                None
            );
            assert!(proof(ast, vc_context, atp_context, &disabled).is_err());
            assert_eq!(replay(&bytes, &disabled).unwrap(), None);
            const LIMIT: usize = 32 * 1024 * 1024;
            let overhead = inner.len() - profile.name().as_str().len();
            let padded = LogicProfile::try_new(
                "x".repeat(LIMIT - overhead),
                profile.fragment(),
                profile.equality(),
                profile.quantifiers(),
                profile.soft_types(),
                profile.native_properties(),
                profile.concrete_formats().clone(),
            )
            .unwrap();
            let big_atp = (&policy, &padded, Tptp);
            let big = proof(ast, vc_context, big_atp, &verifier).unwrap();
            let big_pair = (big.3, big.4);
            let big_checked = (big.5, big.6);
            let large = old_encode(ast, module, &core, vc_context, big_atp, &big_pair)
                .unwrap()
                .unwrap();
            assert_eq!(large.len(), LIMIT);
            assert!(proof(ast, vc_context, big_atp, &disabled).is_err());
            assert_eq!(
                encode(
                    ast,
                    module,
                    &core,
                    vc_context,
                    big_atp,
                    &disabled,
                    (&big_pair, &big_checked)
                )
                .unwrap(),
                None
            );
            let oversized = frame(&large, verifier.policy_fingerprint().as_bytes());
            assert!(oversized.len() > LIMIT);
            assert_eq!(replay(&oversized, &verifier).unwrap(), None);
            for fingerprint in [vec![0; 31], vec![0; 33]] {
                assert_eq!(
                    replay(&frame(&unsupported_inner, &fingerprint), &verifier).unwrap(),
                    None
                );
            }
            let domain = b"mizar-driver/native-theorem-candidate-check-storage/v1\0";
            let mut malformed = vec![
                Vec::new(),
                bytes[..domain.len() + 4].to_vec(),
                bytes[..bytes.len() - 1].to_vec(),
                bytes[..bytes.len() - 40].to_vec(),
            ];
            let mut bad = bytes.clone();
            bad[0] = b'x';
            malformed.push(bad);
            let mut bad = bytes.clone();
            bad[domain.len() - 2] = b'2';
            malformed.push(bad);
            let mut bad = bytes.clone();
            bad[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
            malformed.push(bad);
            let mut bad = bytes.clone();
            bad.push(0);
            malformed.push(bad);
            for malformed in malformed {
                assert_eq!(replay(&malformed, &verifier).unwrap(), None);
            }
            assert_eq!(
                old_decode(&other_inner, foreign_ast.source_id, module, other_context.0).unwrap(),
                Some(other_pair)
            );
            let collected = storage.collect(CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 12);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        },
    );
}

#[test]
fn native_theorem_candidate_check_publication_seals_genuine_payload() {
    use mizar_atp::{
        backend::{BackendCandidateEvidence, EncodedBackendProblem},
        problem::*,
    };
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_candidate_check_from_storage_bytes as decode,
        native_theorem_candidate_check_storage_bytes as encode,
        native_theorem_candidate_storage_bytes as old_encode,
        native_theorem_core_semantic_payload as semantics,
    };
    use mizar_ir::{
        publisher::PublishError,
        storage::{CollectInput, IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{CandidatePolicyClass, VerifierPolicy},
        status::check_source_theorem_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcSet, VcStatus,
        },
    };
    let field = |hasher: &mut blake3::Hasher, value: &[u8]| {
        hasher.update(&u64::try_from(value.len()).unwrap().to_le_bytes());
        hasher.update(value);
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold, fragment, format) in [
        (false, usize::MAX, LogicFragment::Fof, ConcreteFormat::Tptp),
        (
            true,
            0,
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let mut representations = Vec::new();
                for variant in 0..3 {
                    let owner = publisher(old.snapshot, threshold, &old.work_unit);
                    let storage = owner.storage();
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
                        ("AtpSolve", "SourceTheoremCandidateInputs"),
                        ("KernelCheck", "SourceTheoremCandidateCheck"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                    let generation = GenerationSchemaVersion::new(if open { "" } else { "λ\n\0" });
                    let schema = VcSchemaVersion::new(if open { "核\t\0" } else { "" });
                    let policy = DischargePolicy {
                        computation_limit: ComputationLimit {
                            policy: PolicyKey::new("計算\0"),
                            max_steps: u32::MAX,
                        },
                    };
                    let profile = LogicProfile::try_new(
                        "caller λ\n\0",
                        fragment,
                        EqualitySupport::Supported,
                        QuantifierPolicy::FirstOrder,
                        SoftTypeStrategy::ScopedGuardPredicates,
                        NativePropertySupport::Supported,
                        [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                            .into_iter()
                            .collect(),
                    )
                    .unwrap();
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
                        .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                        .register_source_theorem_vc_discharge(policy.clone())
                        .register_source_theorem_atp_solve(policy.clone(), profile.clone(), format)
                        .unwrap();
                    let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                    let WorkUnit::Module { module } = &old.work_unit else {
                        panic!()
                    };
                    submit.vc_descriptors = vec![VcTaskDescriptor::new(
                        VcTaskDescriptorId::new("opaque-check:09"),
                        module.clone(),
                        VcOrderKey::new("order"),
                        vec![BackendProfileId::new("missing-backend")],
                        vec![EvidenceCandidateId::new("declared-evidence")],
                    )];
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
                        .flat_map(|row| &row.output_refs)
                        .collect::<Vec<_>>();
                    assert_eq!(outputs.len(), 12);
                    let parent = |kind: &str| {
                        *outputs
                            .iter()
                            .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                            .unwrap()
                    };
                    let core_ref = parent("SourceTheoremCore");
                    let vc_ref = parent("SourceTheoremVC");
                    let discharge_ref = parent("SourceTheoremDischarge");
                    let atp_ref = parent("SourceTheoremCandidateInputs");
                    let frontend_ref = parent("FrontendOutput");
                    let core_handle = storage
                        .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                        .unwrap();
                    let vc_handle = storage
                        .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                        .unwrap();
                    let discharge_handle = storage
                        .typed_handle::<DischargeOutput>(discharge_ref, discharge_ref.output_kind())
                        .unwrap();
                    let atp_handle = storage
                        .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                            atp_ref,
                            atp_ref.output_kind(),
                        )
                        .unwrap();
                    let core = storage.get(&core_handle).unwrap();
                    let vcs = storage.get(&vc_handle).unwrap();
                    let discharge = storage.get(&discharge_handle).unwrap();
                    let pair = storage.get(&atp_handle).unwrap();
                    let frontend = storage
                    .get(
                        &storage
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                    let ast = frontend.ast.as_ref().unwrap();
                    let module = core.module_id();
                    let (arena, env) = source_theorem_module_original_inputs(ast, module)
                        .unwrap()
                        .unwrap();
                    assert!(
                        source_theorem_module_candidates(ast, &arena, &env)
                            .unwrap()
                            .is_some()
                    );
                    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                        SourceVariableScopeInput::new(ast, module, &env),
                    )
                    .unwrap();
                    let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
                    let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &arena,
                        &metadata.2,
                        &scope,
                        &env,
                        &labels,
                        &resolved,
                        None,
                    )
                    .unwrap();
                    let id = vcs.vcs()[0].id;
                    let actual_generation = &vcs.vc(id).unwrap().anchor.generation_schema_version;
                    let input = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            [atp_ref, discharge_ref, vc_ref, core_ref]
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
                    assert_eq!(ast.source_id, core.source_id());
                    assert_eq!(vcs.source(), ast.source_id);
                    assert_eq!(
                        core.module_id(),
                        &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                    );
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.schema_version(), &schema);
                    assert_eq!(actual_generation, &generation);
                    assert_eq!(vcs.vcs().len(), 1);
                    assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
                    assert!(
                        core.proofs()
                            .iter()
                            .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
                    );
                    assert!(vcs.vcs().iter().all(|row| row.status == VcStatus::Open));
                    assert!(
                        discharge
                            .vc_set()
                            .vcs()
                            .iter()
                            .all(|row| row.status == VcStatus::NeedsAtp)
                    );
                    for row in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!()
                        };
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(
                            core.obligation_seeds().get(seed).unwrap().status,
                            ObligationSeedStatus::Active
                        );
                    }
                    let mut current_policy = policy.clone();
                    if variant == 1 {
                        current_policy.computation_limit.policy = PolicyKey::new("current計算\0");
                    }
                    let verifier = if variant == 2 {
                        VerifierPolicy::default().with_profile_id("unused核\0")
                    } else {
                        VerifierPolicy::default()
                    };
                    let expected = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        id,
                        (
                            actual_generation,
                            vcs.schema_version(),
                            profile.clone(),
                            format,
                        ),
                        (&current_policy, &verifier),
                    )
                    .unwrap();
                    assert_eq!(
                        (
                            &expected.0,
                            &expected.1,
                            &expected.2,
                            &expected.3,
                            &expected.4
                        ),
                        (&*core, &*vcs, &*discharge, &pair.0, &pair.1)
                    );
                    assert_eq!(expected.5.status(), KernelCheckStatus::Accepted);
                    assert_eq!(
                        expected.5.evidence_check_kind(),
                        Some(KernelEvidenceCheckKind::ProofObligation)
                    );
                    assert!(!expected.5.policy_taint());
                    assert_eq!(expected.6.class, CandidatePolicyClass::KernelVerified);
                    assert_eq!(pair.1.target_binding(), pair.0.target_binding());
                    let handoff = mizar_vc::kernel_evidence_handoff::build_source_theorem_discharge_kernel_handoff(&core, &vcs, id, &discharge, &current_policy).unwrap();
                    let target = handoff.canonical_evidence().target_vc();
                    assert_eq!(
                        pair.0.target_binding().fingerprint(),
                        &mizar_atp::problem::AtpFingerprint::new(
                            target.algorithm_id,
                            target.digest.clone()
                        )
                        .unwrap()
                    );
                    let checked = inputs
                        .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                            &input,
                            &owner,
                            id,
                            (profile.clone(), format),
                            (&current_policy, &verifier),
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(checked, (expected.5, expected.6));
                    let context = (old.snapshot, actual_generation, vcs.schema_version(), id);
                    let semantic = semantics(ast, module, &core).unwrap().unwrap();
                    let inner = old_encode(
                        ast,
                        module,
                        &core,
                        context,
                        (&current_policy, &profile, format),
                        &pair,
                    )
                    .unwrap()
                    .unwrap();
                    let bytes = encode(
                        ast,
                        module,
                        &core,
                        context,
                        (&current_policy, &profile, format),
                        &verifier,
                        (&pair, &checked),
                    )
                    .unwrap()
                    .unwrap();
                    assert_eq!(
                        decode(
                            &bytes,
                            ast,
                            module,
                            context,
                            (&current_policy, &profile, format),
                            &verifier
                        )
                        .unwrap(),
                        Some(checked.clone())
                    );
                    let mut record = [0u8; 19];
                    record[..8].copy_from_slice(&u64::try_from(id.index()).unwrap().to_le_bytes());
                    record[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
                    record[12..].copy_from_slice(if open {
                        &[2, 1, 1, 3, 1, 3, 1]
                    } else {
                        &[0, 1, 1, 3, 1, 3, 0]
                    });
                    let mut suffix = Vec::new();
                    for value in [
                        generation.as_str().as_bytes(),
                        schema.as_str().as_bytes(),
                        current_policy.computation_limit.policy.as_str().as_bytes(),
                        b"caller \xce\xbb\n\0",
                        record.as_slice(),
                    ] {
                        suffix
                            .extend_from_slice(&u64::try_from(value.len()).unwrap().to_le_bytes());
                        suffix.extend_from_slice(value);
                    }
                    assert!(inner.ends_with(&suffix));
                    let parent_recipe = old_encode(
                        ast,
                        module,
                        &core,
                        context,
                        (&policy, &profile, format),
                        &pair,
                    )
                    .unwrap()
                    .unwrap();
                    if variant == 1 {
                        assert_ne!(inner, parent_recipe);
                    } else {
                        assert_eq!(inner, parent_recipe);
                    }
                    let fingerprint = verifier.policy_fingerprint();
                    let mut canonical =
                        b"mizar-driver/native-theorem-candidate-check-content/v1\0".to_vec();
                    for value in [
                        semantic.as_slice(),
                        suffix.as_slice(),
                        fingerprint.as_bytes().as_slice(),
                    ] {
                        canonical
                            .extend_from_slice(&u64::try_from(value.len()).unwrap().to_le_bytes());
                        canonical.extend_from_slice(value);
                    }
                    let version = &inputs.snapshot.source_versions[0];
                    let mut content = blake3::Hasher::new();
                    field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                    field(&mut content, &canonical);
                    content.update(&4u64.to_le_bytes());
                    let mut parents = [core_ref, vc_ref, discharge_ref, atp_ref];
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
                    content.update(&1u64.to_le_bytes());
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
                    blob.update(&1u32.to_le_bytes());
                    field(&mut blob, &bytes);
                    let expected_blob = Hash::from_bytes(*blob.finalize().as_bytes());
                    let digest = Hash::from_bytes(blake3::derive_key(
                        "mizar-driver/native-theorem-candidate-check-storage-side-table/v1",
                        &bytes,
                    ));
                    let tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "candidate-check-theorem-storage",
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
                            field(&mut side, b"candidate-check-theorem-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    let before = (
                        input.clone(),
                        ast.clone(),
                        (*core).clone(),
                        (*vcs).clone(),
                        (*discharge).clone(),
                        (*pair).clone(),
                        current_policy.clone(),
                        verifier.clone(),
                        profile.clone(),
                        env.clone(),
                        format!("{check:?}"),
                        session.captured().clone(),
                    );
                    fixture.write(&[0xff]);
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    let sealed = inputs
                        .publish_native_theorem_candidate_check(
                            &input,
                            &owner,
                            id,
                            (profile.clone(), format),
                            (&current_policy, &verifier),
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), old.snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("KernelCheck"));
                    assert_eq!(
                        sealed.output_kind(),
                        &OutputKind::new("SourceTheoremCandidateCheck")
                    );
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), core_ref.work_unit());
                    assert_eq!(sealed.lineage().parents.len(), 4);
                    for parent in [core_ref, vc_ref, discharge_ref, atp_ref] {
                        assert!(sealed.lineage().parents.contains(&parent.output()));
                    }
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        core_ref.lineage().named_input_hashes
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
                        _ => panic!(),
                    }
                    assert_eq!(*storage.get(&sealed).unwrap(), checked);
                    owner
                        .validate_published_output(old.snapshot, sealed.any())
                        .unwrap();
                    assert_eq!(
                        owner
                            .validate_current_output(old.snapshot, sealed.any())
                            .is_ok(),
                        !open
                    );
                    let kept = storage.collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: outputs
                            .iter()
                            .map(|output| output.output())
                            .chain([sealed.output()])
                            .collect(),
                    });
                    assert_eq!(kept.outputs_dropped, 0);
                    assert_eq!(kept.protected_outputs, 13);
                    assert_eq!(kept.abandoned_slots_dropped, 0);
                    let duplicate = PublishError::Storage {
                        error: Box::new(StorageError::OutputAlreadyStored {
                            output: sealed.output(),
                        }),
                    }
                    .to_string();
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_check(
                                &input,
                                &owner,
                                id,
                                (profile.clone(), format),
                                (&current_policy, &verifier)
                            )
                            .unwrap_err(),
                        duplicate
                    );
                    assert_eq!(
                        (
                            input,
                            ast.clone(),
                            (*storage.get(&core_handle).unwrap()).clone(),
                            (*storage.get(&vc_handle).unwrap()).clone(),
                            (*storage.get(&discharge_handle).unwrap()).clone(),
                            (*storage.get(&atp_handle).unwrap()).clone(),
                            current_policy,
                            verifier,
                            profile,
                            env.clone(),
                            format!("{check:?}"),
                            session.captured().clone()
                        ),
                        before
                    );
                    drop(before);
                    drop(frontend);
                    let collected = storage.collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![sealed.output()],
                    });
                    assert_eq!(collected.outputs_dropped, 12);
                    assert_eq!(collected.protected_outputs, 1);
                    assert_eq!(collected.abandoned_slots_dropped, 1);
                    assert_eq!(*storage.get(&sealed).unwrap(), checked);
                    representations.push((
                        (*pair).clone(),
                        checked,
                        canonical,
                        bytes,
                        expected_content,
                        digest,
                    ));
                    let collected = storage.collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 1);
                    assert_eq!(collected.abandoned_slots_dropped, 0);
                }
                for variant in &representations[1..] {
                    assert_eq!(variant.0, representations[0].0);
                    assert_eq!(variant.1, representations[0].1);
                    assert_ne!(variant.2, representations[0].2);
                    assert_ne!(variant.3, representations[0].3);
                    assert_ne!(variant.4, representations[0].4);
                    assert_ne!(variant.5, representations[0].5);
                }
            },
        );
    }
}

#[test]
fn native_theorem_candidate_check_publication_preserves_refusals() {
    use mizar_atp::{
        backend::{BackendCandidateEvidence, EncodedBackendProblem},
        problem::*,
    };
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::CoreIr;
    use mizar_driver::{
        native_theorem_candidate_check_from_storage_bytes as decode,
        native_theorem_candidate_check_storage_bytes as encode,
        native_theorem_candidate_storage_bytes as old_encode,
        native_theorem_core_semantic_payload as semantics,
    };
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    use mizar_proof::{policy::VerifierPolicy, status::check_source_theorem_candidate as fresh};
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{
            ComputationLimit, DischargeInput, DischargeOutput, DischargePolicy, try_discharge,
        },
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for refusal in 0..4 {
        let (open, threshold, fragment, format) =
            (false, 0, LogicFragment::Fof, ConcreteFormat::Tptp);
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, threshold, &old.work_unit);
                let storage = owner.storage();
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
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                }
                let generation = GenerationSchemaVersion::new(if open { "" } else { "λ\n\0" });
                let schema = VcSchemaVersion::new(if open { "核\t\0" } else { "" });
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("計算\0"),
                        max_steps: u32::MAX,
                    },
                };
                let profile = LogicProfile::try_new(
                    "caller λ\n\0",
                    fragment,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone())
                    .register_source_theorem_atp_solve(policy.clone(), profile.clone(), format)
                    .unwrap();
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!()
                };
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-check:09"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![BackendProfileId::new("missing-backend")],
                    vec![EvidenceCandidateId::new("declared-evidence")],
                )];
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
                    .flat_map(|row| &row.output_refs)
                    .collect::<Vec<_>>();
                assert_eq!(outputs.len(), 12);
                let parent = |kind: &str| {
                    *outputs
                        .iter()
                        .find(|parent| parent.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                };
                let core_ref = parent("SourceTheoremCore");
                let vc_ref = parent("SourceTheoremVC");
                let discharge_ref = parent("SourceTheoremDischarge");
                let atp_ref = parent("SourceTheoremCandidateInputs");
                let frontend_ref = parent("FrontendOutput");
                let core_handle = storage
                    .typed_handle::<CoreIr>(core_ref, core_ref.output_kind())
                    .unwrap();
                let vc_handle = storage
                    .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                    .unwrap();
                let discharge_handle = storage
                    .typed_handle::<DischargeOutput>(discharge_ref, discharge_ref.output_kind())
                    .unwrap();
                let atp_handle = storage
                    .typed_handle::<(EncodedBackendProblem, BackendCandidateEvidence)>(
                        atp_ref,
                        atp_ref.output_kind(),
                    )
                    .unwrap();
                let core = storage.get(&core_handle).unwrap();
                let vcs = storage.get(&vc_handle).unwrap();
                let discharge = storage.get(&discharge_handle).unwrap();
                let pair = storage.get(&atp_handle).unwrap();
                let frontend = storage
                    .get(
                        &storage
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                let module = core.module_id();
                let (arena, env) = source_theorem_module_original_inputs(ast, module)
                    .unwrap()
                    .unwrap();
                assert!(
                    source_theorem_module_candidates(ast, &arena, &env)
                        .unwrap()
                        .is_some()
                );
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, module, &env),
                )
                .unwrap();
                let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
                let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena,
                    &metadata.2,
                    &scope,
                    &env,
                    &labels,
                    &resolved,
                    None,
                )
                .unwrap();
                let id = vcs.vcs()[0].id;
                let actual_generation = &vcs.vc(id).unwrap().anchor.generation_schema_version;
                let input = PhaseInput::new(
                    original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        vec![],
                        [atp_ref, discharge_ref, vc_ref, core_ref]
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
                assert_eq!(ast.source_id, core.source_id());
                assert_eq!(vcs.source(), ast.source_id);
                assert_eq!(
                    core.module_id(),
                    &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                );
                assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                assert_eq!(vcs.snapshot(), old.snapshot);
                assert_eq!(vcs.schema_version(), &schema);
                assert_eq!(actual_generation, &generation);
                assert_eq!(vcs.vcs().len(), 1);
                let verifier = VerifierPolicy::default();
                let expected = fresh(
                    &arena,
                    &check,
                    &env,
                    old.snapshot,
                    id,
                    (
                        actual_generation,
                        vcs.schema_version(),
                        profile.clone(),
                        format,
                    ),
                    (&policy, &verifier),
                )
                .unwrap();
                assert_eq!(
                    (
                        &expected.0,
                        &expected.1,
                        &expected.2,
                        &expected.3,
                        &expected.4
                    ),
                    (&*core, &*vcs, &*discharge, &pair.0, &pair.1)
                );
                let checked = inputs
                    .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                        &input,
                        &owner,
                        id,
                        (profile.clone(), format),
                        (&policy, &verifier),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(checked, (expected.5, expected.6));
                let context = (old.snapshot, actual_generation, vcs.schema_version(), id);
                assert!(semantics(ast, module, &core).unwrap().is_some());
                let bytes = encode(
                    ast,
                    module,
                    &core,
                    context,
                    (&policy, &profile, format),
                    &verifier,
                    (&pair, &checked),
                )
                .unwrap()
                .unwrap();
                assert_eq!(
                    decode(
                        &bytes,
                        ast,
                        module,
                        context,
                        (&policy, &profile, format),
                        &verifier
                    )
                    .unwrap(),
                    Some(checked.clone())
                );
                let disabled = verifier.clone().with_kernel_evidence_formats([]);
                let probe = |candidate: &PhaseInput,
                             publisher: &PhaseOutputPublisher,
                             selected: VcId,
                             policy: &DischargePolicy,
                             verifier: &VerifierPolicy| {
                    let old_error = inputs
                        .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                            candidate,
                            publisher,
                            selected,
                            (profile.clone(), format),
                            (policy, verifier),
                        )
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_check(
                                candidate,
                                publisher,
                                selected,
                                (profile.clone(), format),
                                (policy, verifier)
                            )
                            .unwrap_err(),
                        old_error
                    );
                };
                let before = (
                    input.clone(),
                    (*core).clone(),
                    (*vcs).clone(),
                    (*discharge).clone(),
                    (*pair).clone(),
                    policy.clone(),
                    profile.clone(),
                    verifier.clone(),
                    session.captured().clone(),
                );
                if refusal == 0 {
                    let empty = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            vec![],
                        )
                        .unwrap(),
                    );
                    probe(&empty, &owner, VcId::new(usize::MAX), &policy, &disabled);
                    probe(&input, &owner, id, &policy, &disabled);
                    probe(&input, &owner, VcId::new(usize::MAX), &policy, &disabled);
                    let foreign = publisher(old.snapshot, 0, &old.work_unit);
                    probe(&input, &foreign, id, &policy, &verifier);
                    let empty_foreign = foreign.storage().collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(
                        (
                            empty_foreign.outputs_dropped,
                            empty_foreign.abandoned_slots_dropped
                        ),
                        (0, 0)
                    );
                    let role = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            [core_ref, vc_ref, discharge_ref, frontend_ref]
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
                    probe(&role, &owner, id, &policy, &verifier);
                    let protected = storage.collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: outputs.iter().map(|output| output.output()).collect(),
                    });
                    assert_eq!(protected.outputs_dropped, 0);
                    assert_eq!(protected.protected_outputs, 12);
                    assert_eq!(protected.abandoned_slots_dropped, 0);
                    let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_check(
                                &input,
                                &owner,
                                id,
                                (profile.clone(), format),
                                (&policy, &verifier)
                            )
                            .unwrap_err(),
                        PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("KernelCheck"),
                            work_unit: unit,
                            output_kind: OutputKind::new("SourceTheoremCandidateCheck")
                        }
                        .to_string()
                    );
                } else if refusal == 1 {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("KernelCheck"),
                        OutputKind::new("SourceTheoremCandidateCheck"),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                    let base_inner = old_encode(
                        ast,
                        module,
                        &core,
                        context,
                        (&policy, &profile, format),
                        &pair,
                    )
                    .unwrap()
                    .unwrap();
                    let overhead =
                        base_inner.len() - policy.computation_limit.policy.as_str().len();
                    let mut huge = policy.clone();
                    huge.computation_limit.policy =
                        PolicyKey::new("x".repeat(32 * 1024 * 1024 - overhead));
                    assert_eq!(
                        try_discharge(DischargeInput {
                            vc_set: &vcs,
                            policy: &huge
                        })
                        .unwrap(),
                        *discharge
                    );
                    let actual_pair = mizar_atp::backend::build_source_theorem_candidate_inputs(
                        &core,
                        &vcs,
                        id,
                        &discharge,
                        &huge,
                        profile.clone(),
                        format,
                    )
                    .unwrap();
                    assert_eq!(actual_pair, *pair);
                    let actual = inputs
                        .native_theorem_candidate_check_with_core_vc_discharge_atp_parents(
                            &input,
                            &owner,
                            id,
                            (profile.clone(), format),
                            (&huge, &verifier),
                        )
                        .unwrap()
                        .unwrap();
                    let independent = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        id,
                        (
                            actual_generation,
                            vcs.schema_version(),
                            profile.clone(),
                            format,
                        ),
                        (&huge, &verifier),
                    )
                    .unwrap();
                    assert_eq!(
                        (
                            &independent.0,
                            &independent.1,
                            &independent.2,
                            &independent.3,
                            &independent.4
                        ),
                        (&*core, &*vcs, &*discharge, &actual_pair.0, &actual_pair.1)
                    );
                    assert_eq!(actual, (independent.5, independent.6));
                    assert_eq!(actual, checked);
                    assert!(semantics(ast, module, &core).unwrap().is_some());
                    let large = old_encode(
                        ast,
                        module,
                        &core,
                        context,
                        (&huge, &profile, format),
                        &actual_pair,
                    )
                    .unwrap()
                    .unwrap();
                    assert_eq!(large.len(), 32 * 1024 * 1024);
                    assert_eq!(
                        encode(
                            ast,
                            module,
                            &core,
                            context,
                            (&huge, &profile, format),
                            &verifier,
                            (&actual_pair, &actual)
                        )
                        .unwrap(),
                        None
                    );
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_check(
                                &input,
                                &owner,
                                id,
                                (profile.clone(), format),
                                (&huge, &verifier)
                            )
                            .unwrap(),
                        None
                    );
                } else if refusal == 2 {
                    let collected = storage.collect(CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 12);
                    assert_eq!(collected.abandoned_slots_dropped, 0);
                    probe(&input, &owner, id, &policy, &disabled);
                } else {
                    owner.mark_obsolete(old.snapshot).unwrap();
                    probe(&input, &owner, id, &policy, &disabled);
                }
                assert_eq!(
                    (
                        input,
                        (*core).clone(),
                        (*vcs).clone(),
                        (*discharge).clone(),
                        (*pair).clone(),
                        policy,
                        profile,
                        verifier,
                        session.captured().clone()
                    ),
                    before
                );
                let collected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, if refusal == 2 { 0 } else { 12 });
                assert_eq!(collected.abandoned_slots_dropped, usize::from(refusal == 0));
            },
        );
    }
}
#[test]
fn native_theorem_candidate_selection_matches_genuine_check() {
    use mizar_atp::problem::*;
    use mizar_build::task_graph::{VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId};
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus};
    use mizar_ir::storage::{CollectInput, StoragePlacement};
    use mizar_kernel::checker::{KernelCheckResult, KernelCheckStatus};
    use mizar_proof::{
        policy::{CandidatePolicyClass, PolicyDecision, VerifierPolicy},
        selection::{ProofWinnerClass, ProofWitnessPublication},
        status::select_source_theorem_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeOutput, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcSet, VcStatus,
        },
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (open, threshold, fragment, format) in [
        (false, usize::MAX, LogicFragment::Fof, ConcreteFormat::Tptp),
        (
            true,
            0,
            LogicFragment::SmtLibUninterpreted,
            ConcreteFormat::SmtLib,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let generation = GenerationSchemaVersion::new("λ\n\0");
                let schema = VcSchemaVersion::new("");
                let profile = LogicProfile::try_new(
                    "selection λ\n\0",
                    fragment,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("計算\0"),
                        max_steps: u32::MAX,
                    },
                };
                let verifier = VerifierPolicy::release();
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!()
                };
                let owner = publisher(old.snapshot, threshold, &old.work_unit);
                let storage = owner.storage();
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
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                    ("KernelCheck", "SourceTheoremCandidateCheck"),
                ] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        IrWorkUnit::new("\"alpha\":\"main\""),
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone());
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-prerequisite:selection"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![],
                    vec![],
                )];
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                let (inputs, original) = native_disk_lookup_context(&submission, ids);
                let refs = submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter().cloned())
                    .collect::<Vec<_>>();
                assert_eq!(refs.len(), 11);
                let parent = |kind: &str| {
                    refs.iter()
                        .find(|row| row.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                        .clone()
                };
                let core_ref = parent("SourceTheoremCore");
                let vc_ref = parent("SourceTheoremVC");
                let d_ref = parent("SourceTheoremDischarge");
                let frontend_ref = parent("FrontendOutput");
                let core = storage
                    .get(
                        &storage
                            .typed_handle::<CoreIr>(&core_ref, core_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let vcs = storage
                    .get(
                        &storage
                            .typed_handle::<VcSet>(&vc_ref, vc_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let discharge = storage
                    .get(
                        &storage
                            .typed_handle::<DischargeOutput>(&d_ref, d_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let frontend = storage
                    .get(
                        &storage
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                &frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                let id = vcs.vcs()[0].id;
                let (arena, env) = source_theorem_module_original_inputs(ast, core.module_id())
                    .unwrap()
                    .unwrap();
                assert!(
                    source_theorem_module_candidates(ast, &arena, &env)
                        .unwrap()
                        .is_some()
                );
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, core.module_id(), &env),
                )
                .unwrap();
                let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
                let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena,
                    &metadata.2,
                    &scope,
                    &env,
                    &labels,
                    &resolved,
                    None,
                )
                .unwrap();
                let input = |parents: Vec<AnyPhaseOutputRef>| {
                    PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            original.identities().dependency_hashes().to_vec(),
                            parents
                                .into_iter()
                                .map(|row| {
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
                                        old.snapshot,
                                        row,
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        )
                        .unwrap(),
                    )
                };
                let three = input(vec![core_ref.clone(), vc_ref.clone(), d_ref.clone()]);
                let atp = inputs
                    .publish_native_theorem_candidate(
                        &three,
                        &owner,
                        id,
                        &policy,
                        profile.clone(),
                        format,
                    )
                    .unwrap()
                    .unwrap();
                let old_four = input(vec![
                    core_ref.clone(),
                    vc_ref.clone(),
                    d_ref.clone(),
                    atp.erase(),
                ]);
                let kernel = inputs
                    .publish_native_theorem_candidate_check(
                        &old_four,
                        &owner,
                        id,
                        (profile.clone(), format),
                        (&policy, &verifier),
                    )
                    .unwrap()
                    .unwrap();
                let kernel_ref = kernel.erase();
                let four = input(vec![
                    core_ref.clone(),
                    vc_ref.clone(),
                    d_ref.clone(),
                    kernel_ref.clone(),
                ]);
                let all = refs
                    .iter()
                    .map(|row| row.output())
                    .chain([atp.output(), kernel.output()])
                    .collect::<Vec<_>>();
                assert_eq!(all.len(), 13);
                let stored = storage
                    .get(
                        &storage
                            .typed_handle::<(KernelCheckResult, PolicyDecision)>(
                                &kernel_ref,
                                kernel_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(*stored, *storage.get(&kernel).unwrap());
                assert_eq!(
                    matches!(kernel_ref.placement(), StoragePlacement::Blob { .. }),
                    open
                );
                let hidden = kernel_ref
                    .lineage()
                    .parents
                    .iter()
                    .find(|output| {
                        ![core_ref.output(), vc_ref.output(), d_ref.output()].contains(output)
                    })
                    .unwrap();
                assert_eq!(*hidden, atp.output());
                assert_eq!(
                    owner.registry().output_lineage(*hidden).unwrap(),
                    atp.lineage().clone()
                );
                assert_eq!(kernel_ref.lineage().parents.len(), 4);
                assert_eq!(ast.source_id, core.source_id());
                assert_eq!(vcs.source(), ast.source_id);
                assert_eq!(vcs.snapshot(), old.snapshot);
                assert_eq!(vcs.schema_version(), &schema);
                assert_eq!(
                    vcs.vc(id).unwrap().anchor.generation_schema_version,
                    generation
                );
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, row)| row.status == ObligationSeedStatus::Active)
                );
                assert!(vcs.seed_accounting().iter().all(|row| {
                    matches!(row.origin, SeedOriginRef::ExistingCore { .. })
                        && row.seed_status == ObligationSeedStatus::Active
                }));
                assert!(vcs.vcs().iter().all(|row| row.status == VcStatus::Open));
                assert!(
                    discharge
                        .vc_set()
                        .vcs()
                        .iter()
                        .all(|row| row.status == VcStatus::NeedsAtp)
                );
                let source_before = session.captured().clone();
                fixture.write(&[0xff]);
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                for variant in 0..3 {
                    let mut current_policy = policy.clone();
                    if variant == 1 {
                        current_policy.computation_limit.policy =
                            PolicyKey::new("unused-current-key");
                    }
                    let current_verifier = if variant == 2 {
                        verifier.clone().with_profile_id("unused-current-verifier")
                    } else {
                        verifier.clone()
                    };
                    let raw = inputs
                        .native_theorem_candidate_with_core_vc_discharge_parents(
                            &three,
                            &owner,
                            id,
                            &current_policy,
                            profile.clone(),
                            format,
                        )
                        .unwrap()
                        .unwrap();
                    let expected = fresh(
                        &arena,
                        &check,
                        &env,
                        old.snapshot,
                        id,
                        (
                            &vcs.vc(id).unwrap().anchor.generation_schema_version,
                            vcs.schema_version(),
                            profile.clone(),
                            format,
                        ),
                        (&current_policy, &current_verifier),
                    )
                    .unwrap();
                    assert_eq!(
                        (&expected.0, &expected.1, &expected.2),
                        (&*core, &*vcs, &*discharge)
                    );
                    assert_eq!((&expected.3, &expected.4), (&raw.0, &raw.1));
                    assert_eq!((&expected.5, &expected.6), (&stored.0, &stored.1));
                    assert_eq!(expected.5.status(), KernelCheckStatus::Accepted);
                    assert_eq!(expected.6.class, CandidatePolicyClass::KernelVerified);
                    assert_eq!(
                        expected.7.selected_class(),
                        ProofWinnerClass::KernelVerified
                    );
                    assert_eq!(
                        expected.7.selected_candidate_id().unwrap().as_str(),
                        expected.4.candidate_id()
                    );
                    let reuse = expected.7.reuse_metadata();
                    assert_eq!(
                        reuse.policy_fingerprint(),
                        current_verifier.policy_fingerprint()
                    );
                    assert_eq!(reuse.encoded_problem_hash(), Some(expected.3.input_hash()));
                    assert_eq!(
                        reuse.selected_candidate_provenance_hash(),
                        Some(expected.4.provenance_hash())
                    );
                    assert_eq!(
                        reuse.proof_witness_publication(),
                        ProofWitnessPublication::NotApplicable
                    );
                    assert_eq!(reuse.selected_proof_witness_hash(), None);
                    let before = (
                        four.clone(),
                        current_policy.clone(),
                        current_verifier.clone(),
                        source_before.clone(),
                    );
                    assert_eq!(inputs.native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                        &four, &owner, id, (profile.clone(), format), (&current_policy, &current_verifier),
                    ).unwrap(), Some(expected.7));
                    assert_eq!(
                        (
                            four.clone(),
                            current_policy,
                            current_verifier,
                            session.captured().clone()
                        ),
                        before
                    );
                }
                let protected = storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: [
                        core_ref.output(),
                        vc_ref.output(),
                        d_ref.output(),
                        kernel_ref.output(),
                    ]
                    .to_vec(),
                });
                assert_eq!(
                    (
                        protected.outputs_dropped,
                        protected.protected_outputs,
                        protected.abandoned_slots_dropped
                    ),
                    (9, 4, 0)
                );
                assert!(storage.validate_handle(&atp.erase()).is_err());
                assert!(owner.registry().output_lineage(*hidden).is_some());
                let baseline = fresh(
                    &arena,
                    &check,
                    &env,
                    old.snapshot,
                    id,
                    (
                        &vcs.vc(id).unwrap().anchor.generation_schema_version,
                        vcs.schema_version(),
                        profile.clone(),
                        format,
                    ),
                    (&policy, &verifier),
                )
                .unwrap();
                assert_eq!(
                    inputs
                        .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                            &four,
                            &owner,
                            id,
                            (profile, format),
                            (&policy, &verifier),
                        )
                        .unwrap(),
                    Some(baseline.7)
                );
                assert_eq!(*storage.get(&kernel).unwrap(), (baseline.5, baseline.6));
                assert_eq!(session.captured().clone(), source_before);
            },
        );
    }
}

#[test]
fn native_theorem_candidate_selection_preserves_refusals() {
    use mizar_atp::{backend::BackendCandidatePayload, problem::*};
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_core::core_ir::CoreIr;
    use mizar_ir::{
        publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            KernelCheckPolicy, KernelCheckStatus, KernelEvidenceCheckInput,
            KernelEvidenceCheckKind, KernelEvidenceCheckLimits, check_kernel_evidence,
        },
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{
            CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, PolicyCandidate,
            ProofPolicyEvaluator, VerifierPolicy,
        },
        status::select_source_theorem_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_vc::{
        discharge::{DischargeOutput, DischargePolicy},
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_theorem_discharge_kernel_handoff,
        },
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    with_native_summary_parent(
        text,
        false,
        0,
        |fixture, _, old, _, _, (session, ids, snapshots)| {
            let generation = GenerationSchemaVersion::new("λ\n\0");
            let schema = VcSchemaVersion::new("");
            let policy = DischargePolicy::default();
            let verifier = VerifierPolicy::release();
            let disabled = verifier.clone().with_kernel_evidence_formats([]);
            let profile = LogicProfile::try_new(
                "selection-refusal",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp].into_iter().collect(),
            )
            .unwrap();
            let WorkUnit::Module { module } = &old.work_unit else {
                panic!()
            };
            let owner = publisher(old.snapshot, 0, &old.work_unit);
            let storage = owner.storage();
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
                ("VcDischarge", "SourceTheoremDischarge"),
                ("AtpSolve", "SourceTheoremCandidateInputs"),
                ("KernelCheck", "SourceTheoremCandidateCheck"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
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
                .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                .register_source_theorem_vc_discharge(policy.clone());
            let (_, mut submit) = fixture.submit_request(vec![], vec![]);
            submit.vc_descriptors = vec![VcTaskDescriptor::new(
                VcTaskDescriptorId::new("opaque-selection-refusal"),
                module.clone(),
                VcOrderKey::new("order"),
                vec![BackendProfileId::new("missing-backend")],
                vec![EvidenceCandidateId::new("declared-evidence")],
            )];
            let submission = CompilerDriver::new(builder.build().unwrap())
                .with_output_publisher(owner.clone())
                .submit_captured(session.clone(), ids, snapshots, submit)
                .unwrap();
            let (inputs, original) = native_disk_lookup_context(&submission, ids);
            let refs = submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .flat_map(|row| row.output_refs.iter().cloned())
                .collect::<Vec<_>>();
            assert_eq!(refs.len(), 11);
            let parent = |kind: &str| {
                refs.iter()
                    .find(|row| row.output_kind() == &OutputKind::new(kind))
                    .unwrap()
                    .clone()
            };
            let core_ref = parent("SourceTheoremCore");
            let vc_ref = parent("SourceTheoremVC");
            let d_ref = parent("SourceTheoremDischarge");
            let frontend_ref = parent("FrontendOutput");
            let core = storage
                .get(
                    &storage
                        .typed_handle::<CoreIr>(&core_ref, core_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let vcs = storage
                .get(
                    &storage
                        .typed_handle::<VcSet>(&vc_ref, vc_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let discharge = storage
                .get(
                    &storage
                        .typed_handle::<DischargeOutput>(&d_ref, d_ref.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let frontend = storage
                .get(
                    &storage
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            &frontend_ref,
                            frontend_ref.output_kind(),
                        )
                        .unwrap(),
                )
                .unwrap();
            let ast = frontend.ast.as_ref().unwrap();
            let (arena, env) = source_theorem_module_original_inputs(ast, core.module_id())
                .unwrap()
                .unwrap();
            assert!(
                source_theorem_module_candidates(ast, &arena, &env)
                    .unwrap()
                    .is_some()
            );
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, core.module_id(), &env),
            )
            .unwrap();
            let metadata = check_source_theorem_metadata(&arena, &env, &scope).unwrap();
            let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
            let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena,
                &metadata.2,
                &scope,
                &env,
                &labels,
                &resolved,
                None,
            )
            .unwrap();
            let id = vcs.vcs()[0].id;
            let input = |work_unit: WorkUnit, parents: Vec<AnyPhaseOutputRef>| {
                PhaseInput::new(
                    work_unit,
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        original.identities().input_hash(),
                        original.identities().dependency_hashes().to_vec(),
                        parents
                            .into_iter()
                            .map(|row| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner,
                                    old.snapshot,
                                    row,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                )
            };
            let original_roles = vec![core_ref.clone(), vc_ref.clone(), d_ref.clone()];
            let three = input(original.work_unit.clone(), original_roles.clone());
            let atp = inputs
                .publish_native_theorem_candidate(
                    &three,
                    &owner,
                    id,
                    &policy,
                    profile.clone(),
                    ConcreteFormat::Tptp,
                )
                .unwrap()
                .unwrap();
            let old_four = input(
                original.work_unit.clone(),
                [original_roles.as_slice(), &[atp.erase()]].concat(),
            );
            let kernel = inputs
                .publish_native_theorem_candidate_check(
                    &old_four,
                    &owner,
                    id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                )
                .unwrap()
                .unwrap();
            let four_roles = [original_roles.as_slice(), &[kernel.erase()]].concat();
            let four = input(original.work_unit.clone(), four_roles.clone());
            assert_eq!(refs.len() + 2, 13);
            let raw = inputs
                .native_theorem_candidate_with_core_vc_discharge_parents(
                    &three,
                    &owner,
                    id,
                    &policy,
                    profile.clone(),
                    ConcreteFormat::Tptp,
                )
                .unwrap()
                .unwrap();
            let baseline = fresh(
                &arena,
                &check,
                &env,
                old.snapshot,
                id,
                (
                    &vcs.vc(id).unwrap().anchor.generation_schema_version,
                    vcs.schema_version(),
                    profile.clone(),
                    ConcreteFormat::Tptp,
                ),
                (&policy, &verifier),
            )
            .unwrap();
            assert_eq!(
                (&baseline.0, &baseline.1, &baseline.2),
                (&*core, &*vcs, &*discharge)
            );
            assert_eq!((&baseline.3, &baseline.4), (&raw.0, &raw.1));
            assert_eq!(
                (&baseline.5, &baseline.6),
                (
                    &storage.get(&kernel).unwrap().0,
                    &storage.get(&kernel).unwrap().1
                )
            );
            assert_eq!(baseline.5.status(), KernelCheckStatus::Accepted);
            let query =
                |candidate: &PhaseInput, profile: LogicProfile, verifier: &VerifierPolicy| {
                    inputs.native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                        candidate,
                        &owner,
                        id,
                        (profile, ConcreteFormat::Tptp),
                        (&policy, verifier),
                    )
                };
            assert_eq!(
                query(&four, profile.clone(), &verifier).unwrap(),
                Some(baseline.7.clone())
            );
            let reject = |candidate: &PhaseInput, expected: &str| {
                assert_eq!(
                    query(candidate, profile.clone(), &verifier).unwrap_err(),
                    expected
                );
            };
            for parents in [
                four_roles[..3].to_vec(),
                [four_roles.as_slice(), std::slice::from_ref(&frontend_ref)].concat(),
            ] {
                reject(
                    &input(original.work_unit.clone(), parents),
                    "native theorem candidate selection requires four parents",
                );
            }
            let mut wrong_role = four_roles.clone();
            wrong_role[3] = frontend_ref.clone();
            reject(
                &input(original.work_unit.clone(), wrong_role),
                "native theorem candidate selection requires one Kernel parent",
            );
            let wrong_unit = WorkUnit::EvidenceCandidate {
                module: module.clone(),
                descriptor: VcTaskDescriptorId::new("opaque-selection"),
                evidence_candidate: EvidenceCandidateId::new("opaque"),
            };
            reject(
                &input(wrong_unit, four_roles.clone()),
                "native theorem candidate selection requires a module",
            );
            let wrong_module = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("other"),
                ),
            };
            reject(
                &input(wrong_module, four_roles.clone()),
                "native theorem candidate selection parent binding mismatch",
            );
            let wrong_hash = PhaseInput::new(
                original.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    hash(0x64),
                    vec![],
                    four.parent_outputs().to_vec(),
                )
                .unwrap(),
            );
            reject(
                &wrong_hash,
                "native theorem candidate selection parent binding mismatch",
            );
            let missing = VcId::new(usize::MAX);
            let lower = inputs.native_theorem_candidate_with_core_vc_discharge_parents(
                &three,
                &owner,
                missing,
                &policy,
                profile.clone(),
                ConcreteFormat::Tptp,
            );
            assert_eq!(
                inputs
                    .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                        &four,
                        &owner,
                        missing,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                    )
                    .map(|selection| selection.map(|_| ())),
                lower.map(|pair| pair.map(|_| ()))
            );
            let unsupported = LogicProfile::try_new(
                "unsupported",
                LogicFragment::TffLike,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp].into_iter().collect(),
            )
            .unwrap();
            let lower = inputs
                .native_theorem_candidate_with_core_vc_discharge_parents(
                    &three,
                    &owner,
                    id,
                    &policy,
                    unsupported.clone(),
                    ConcreteFormat::Tptp,
                )
                .unwrap_err();
            assert_eq!(query(&four, unsupported, &verifier).unwrap_err(), lower);
            let policy_error = fresh(
                &arena,
                &check,
                &env,
                old.snapshot,
                id,
                (
                    &vcs.vc(id).unwrap().anchor.generation_schema_version,
                    vcs.schema_version(),
                    profile.clone(),
                    ConcreteFormat::Tptp,
                ),
                (&policy, &disabled),
            )
            .unwrap_err();
            assert_eq!(
                query(&four, profile.clone(), &disabled).unwrap_err(),
                policy_error
            );
            let alien = publisher(old.snapshot, 0, &old.work_unit);
            assert_eq!(
                inputs
                    .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                        &four,
                        &alien,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                    )
                    .unwrap_err(),
                alien
                    .storage()
                    .validate_handle(&core_ref)
                    .unwrap_err()
                    .to_string()
            );

            // The wrong tuple is produced by the ordinary public Kernel on the real, same-target wire.
            let handoff =
                build_source_theorem_discharge_kernel_handoff(&core, &vcs, id, &discharge, &policy)
                    .unwrap();
            let envelope = handoff.canonical_evidence();
            let target = envelope.target_vc();
            let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
            let profile_record = envelope.kernel_profile();
            let tautology = match profile_record.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!(),
            };
            let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(profile_record.profile_id, tautology),
            );
            parse_context.accepted_schema_versions.insert(2);
            let BackendCandidatePayload::FormulaSubstitutionBytes(wire) = raw.1.payload() else {
                panic!()
            };
            let parsed = parse_formula_evidence(wire, &parse_context).unwrap();
            let rejected = check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: &target,
                evidence: &parsed,
                formula_context: None,
                check_kind: KernelEvidenceCheckKind::ProofObligation,
                policy: KernelCheckPolicy::default(),
                limits: KernelEvidenceCheckLimits::default(),
            });
            assert_eq!(rejected.status(), KernelCheckStatus::Rejected);
            assert_eq!(
                rejected.target_vc_fingerprint(),
                baseline.5.target_vc_fingerprint()
            );
            let evaluator = ProofPolicyEvaluator::new(verifier.clone());
            let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(
                KernelPolicyInput::from_kernel_result(
                    &rejected,
                    KernelEvidenceOrigin::AtpFormulaSubstitution,
                ),
            ));
            assert_ne!(
                (rejected.clone(), decision.clone()),
                (baseline.5, baseline.6)
            );
            assert_ne!(decision.class, CandidatePolicyClass::KernelVerified);
            let payload = (rejected, decision);
            for case in ["valid", "link", "hidden", "schema"] {
                let wrong_owner = publisher(old.snapshot, 0, &old.work_unit);
                allow_frontend(&wrong_owner, &old.work_unit);
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
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                    ("KernelCheck", "SourceTheoremCandidateCheck"),
                ] {
                    wrong_owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        unit.clone(),
                    ));
                }
                let mut wrong_builder = PhaseRegistryBuilder::new();
                wrong_builder
                    .register_source_load()
                    .register_frontend(vec![])
                    .register_source_theorem_module_resolver()
                    .register_source_theorem_signature_collection()
                    .register_source_theorem_type_checking()
                    .register_source_theorem_registration_resolution()
                    .register_source_theorem_overload_resolution()
                    .register_source_theorem_elaboration()
                    .register_source_theorem_algorithm_preparation()
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone())
                    .register_source_theorem_vc_discharge(policy.clone());
                let (_, mut wrong_submit) = fixture.submit_request(vec![], vec![]);
                wrong_submit.vc_descriptors = vec![VcTaskDescriptor::new(
                    VcTaskDescriptorId::new("opaque-selection-refusal"),
                    module.clone(),
                    VcOrderKey::new("order"),
                    vec![BackendProfileId::new("missing-backend")],
                    vec![EvidenceCandidateId::new("declared-evidence")],
                )];
                let wrong_submission = CompilerDriver::new(wrong_builder.build().unwrap())
                    .with_output_publisher(wrong_owner.clone())
                    .submit_captured(session.clone(), ids, snapshots, wrong_submit)
                    .unwrap();
                let (wrong_inputs, wrong_original) =
                    native_disk_lookup_context(&wrong_submission, ids);
                let wrong_refs = wrong_submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter().cloned())
                    .collect::<Vec<_>>();
                assert_eq!(wrong_refs.len(), 11);
                let wrong_parent = |kind: &str| {
                    wrong_refs
                        .iter()
                        .find(|row| row.output_kind() == &OutputKind::new(kind))
                        .unwrap()
                        .clone()
                };
                let wrong_core_ref = wrong_parent("SourceTheoremCore");
                let wrong_vc_ref = wrong_parent("SourceTheoremVC");
                let wrong_d_ref = wrong_parent("SourceTheoremDischarge");
                let wrong_storage = wrong_owner.storage();
                let wrong_core = wrong_storage
                    .get(
                        &wrong_storage
                            .typed_handle::<CoreIr>(&wrong_core_ref, wrong_core_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let wrong_vcs = wrong_storage
                    .get(
                        &wrong_storage
                            .typed_handle::<VcSet>(&wrong_vc_ref, wrong_vc_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let wrong_discharge = wrong_storage
                    .get(
                        &wrong_storage
                            .typed_handle::<DischargeOutput>(
                                &wrong_d_ref,
                                wrong_d_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let wrong_roles = [
                    wrong_core_ref.clone(),
                    wrong_vc_ref.clone(),
                    wrong_d_ref.clone(),
                ];
                let wrong_three = PhaseInput::new(
                    wrong_original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        wrong_original.identities().input_hash(),
                        wrong_original.identities().dependency_hashes().to_vec(),
                        wrong_roles
                            .iter()
                            .cloned()
                            .map(|row| {
                                SealedParentOutputHandle::from_published_output(
                                    &wrong_owner,
                                    old.snapshot,
                                    row,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                );
                let wrong_raw = wrong_inputs
                    .native_theorem_candidate_with_core_vc_discharge_parents(
                        &wrong_three,
                        &wrong_owner,
                        id,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                    .unwrap()
                    .unwrap();
                let wrong_baseline = fresh(
                    &arena,
                    &check,
                    &env,
                    old.snapshot,
                    id,
                    (
                        &wrong_vcs.vc(id).unwrap().anchor.generation_schema_version,
                        wrong_vcs.schema_version(),
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    ),
                    (&policy, &verifier),
                )
                .unwrap();
                assert_eq!(
                    (&wrong_baseline.0, &wrong_baseline.1, &wrong_baseline.2),
                    (&*wrong_core, &*wrong_vcs, &*wrong_discharge)
                );
                assert_eq!(
                    (&wrong_baseline.3, &wrong_baseline.4),
                    (&wrong_raw.0, &wrong_raw.1)
                );
                assert_eq!(
                    (&wrong_baseline.5, &wrong_baseline.6),
                    (
                        &storage.get(&kernel).unwrap().0,
                        &storage.get(&kernel).unwrap().1
                    )
                );
                assert_eq!(wrong_raw, raw);
                assert_eq!(
                    payload.0.target_vc_fingerprint(),
                    wrong_baseline.5.target_vc_fingerprint()
                );
                let wrong_atp = wrong_inputs
                    .publish_native_theorem_candidate(
                        &wrong_three,
                        &wrong_owner,
                        id,
                        &policy,
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(*wrong_storage.get(&wrong_atp).unwrap(), wrong_raw);
                let parse_for_decode = parse_context.clone();
                let target_for_decode = target.clone();
                let verifier_for_decode = verifier.clone();
                let bytes = wire.to_vec();
                let kernel_schema = SchemaVersion::new(if case == "schema" { 2 } else { 1 });
                let kernel_parents = match case {
                    "link" => wrong_roles.to_vec(),
                    "hidden" => {
                        [wrong_roles.as_slice(), &[wrong_parent("FrontendOutput")]].concat()
                    }
                    _ => [wrong_roles.as_slice(), &[wrong_atp.erase()]].concat(),
                };
                let wrong = wrong_owner
                    .publish(PublishOutputInput {
                        slot: wrong_owner.allocate(
                            old.snapshot,
                            IrPipelinePhase::new("KernelCheck"),
                            unit.clone(),
                            OutputKind::new("SourceTheoremCandidateCheck"),
                            kernel_schema,
                        ),
                        snapshot: old.snapshot,
                        phase: IrPipelinePhase::new("KernelCheck"),
                        work_unit: unit.clone(),
                        output_kind: OutputKind::new("SourceTheoremCandidateCheck"),
                        schema_version: kernel_schema,
                        payload: payload.clone(),
                        canonical_payload: Some(bytes.clone()),
                        storage_payload: Some(bytes),
                        decode: BlobDecoder::new(move |bytes| {
                            let parsed = parse_formula_evidence(bytes, &parse_for_decode)
                                .map_err(|error| BlobDecodeError::new(format!("{error:?}")))?;
                            let checked = check_kernel_evidence(KernelEvidenceCheckInput {
                                target_vc_fingerprint: &target_for_decode,
                                evidence: &parsed,
                                formula_context: None,
                                check_kind: KernelEvidenceCheckKind::ProofObligation,
                                policy: KernelCheckPolicy::default(),
                                limits: KernelEvidenceCheckLimits::default(),
                            });
                            let classified = ProofPolicyEvaluator::new(verifier_for_decode.clone())
                                .evaluate_candidate(&PolicyCandidate::KernelResult(
                                    KernelPolicyInput::from_kernel_result(
                                        &checked,
                                        KernelEvidenceOrigin::AtpFormulaSubstitution,
                                    ),
                                ));
                            Ok((checked, classified))
                        }),
                        parents: kernel_parents,
                        named_input_hashes: wrong_core_ref.lineage().named_input_hashes.clone(),
                        side_tables: IrSideTables::default(),
                        origin: OutputOrigin::PackageSource,
                        target: PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                assert_eq!(*wrong_storage.get(&wrong).unwrap(), payload);
                let wrong_four = PhaseInput::new(
                    wrong_original.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        wrong_original.identities().input_hash(),
                        wrong_original.identities().dependency_hashes().to_vec(),
                        wrong_roles
                            .iter()
                            .cloned()
                            .chain([wrong.erase()])
                            .map(|row| {
                                SealedParentOutputHandle::from_published_output(
                                    &wrong_owner,
                                    old.snapshot,
                                    row,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                );
                let wrong_query = |verifier: &VerifierPolicy| {
                    wrong_inputs
                        .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                            &wrong_four,
                            &wrong_owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, verifier),
                        )
                };
                assert_eq!(
                    wrong_query(&verifier).unwrap_err(),
                    match case {
                        "link" => "native theorem candidate selection Kernel parent link mismatch",
                        "hidden" => "native theorem candidate selection ATP lineage mismatch",
                        "schema" => "native theorem candidate selection parent binding mismatch",
                        _ => "native theorem candidate selection complete pipeline mismatch",
                    }
                );
                if case != "valid" {
                    continue;
                }
                assert_eq!(wrong_query(&disabled).unwrap_err(), policy_error);
                let collected = wrong_storage.collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: [
                        wrong_core_ref.output(),
                        wrong_vc_ref.output(),
                        wrong_d_ref.output(),
                        wrong_atp.output(),
                    ]
                    .to_vec(),
                });
                assert_eq!(collected.outputs_dropped, 9);
                assert!(wrong_storage.validate_handle(&wrong.erase()).is_err());
                assert_eq!(
                    wrong_query(&verifier).unwrap_err(),
                    wrong_storage
                        .validate_handle(&wrong.erase())
                        .unwrap_err()
                        .to_string()
                );
                wrong_owner.mark_obsolete(old.snapshot).unwrap();
                assert_eq!(
                    wrong_query(&verifier).unwrap_err(),
                    wrong_owner
                        .validate_published_output(old.snapshot, &wrong_core_ref)
                        .unwrap_err()
                        .to_string()
                );
            }
        },
    );
}
#[test]
fn native_theorem_candidate_witness_file_matches_current_kernel() {
    use mizar_artifact::store::{
        CanonicalJson, SchemaVersion as ArtifactVersion, artifact_hash_domain,
        canonical_json_bytes, parse_canonical_json_bytes,
    };
    use mizar_atp::problem::*;
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_ir::storage::{CollectInput, IrSideTables, StoragePlacement};
    use mizar_proof::{
        policy::{KernelEvidenceOrigin, KernelPolicyInput, PolicyDecision, VerifierPolicy},
        status::{TrustedUsedAxiomsRef, source_theorem_candidate_witness_file_candidate as fresh},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{source_proof_inputs, source_theorem_module_original_inputs},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for open in [false, true] {
        with_native_theorem_witness_kernel(
            text,
            open,
            if open { 0 } else { usize::MAX },
            |fixture, inputs, four, owner, refs, (session, _, _)| {
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new(if open { "unused-current" } else { "explicit" }),
                        max_steps: 64,
                    },
                };
                let verifier = if open {
                    VerifierPolicy::default().with_profile_id("current λ\0")
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "witness-data",
                    if open {
                        LogicFragment::SmtLibUninterpreted
                    } else {
                        LogicFragment::Fof
                    },
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let format = if open {
                    ConcreteFormat::SmtLib
                } else {
                    ConcreteFormat::Tptp
                };
                let parent = |kind: &str| {
                    four.parent_outputs()
                        .iter()
                        .find(|row| row.as_output_ref().output_kind() == &OutputKind::new(kind))
                        .unwrap()
                        .as_output_ref()
                };
                let vc_ref = parent("SourceTheoremVC");
                let kernel = parent("SourceTheoremCandidateCheck");
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let id = vcs.vcs()[0].id;
                let physical = owner.storage().get(&owner.storage().typed_handle::<(mizar_kernel::checker::KernelCheckResult, PolicyDecision)>(kernel, kernel.output_kind()).unwrap()).unwrap();
                let selected = inputs
                    .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                        four,
                        owner,
                        id,
                        (profile.clone(), format),
                        (&policy, &verifier),
                    )
                    .unwrap()
                    .unwrap();
                let physical_input = KernelPolicyInput::from_kernel_result(
                    &physical.0,
                    KernelEvidenceOrigin::AtpFormulaSubstitution,
                );
                let axioms = TrustedUsedAxiomsRef::from_kernel_result(
                    &physical.0,
                    KernelEvidenceOrigin::AtpFormulaSubstitution,
                )
                .unwrap();
                let frontend_ref = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("FrontendOutput"))
                    .unwrap();
                let frontend = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                let WorkUnit::Module { module } = &four.work_unit else {
                    panic!()
                };
                let resolver_module =
                    ResolverModuleId::new(module.package.clone(), module.path.clone());
                let (arena, env) = source_theorem_module_original_inputs(ast, &resolver_module)
                    .unwrap()
                    .unwrap();
                let variable_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, &resolver_module, &env),
                )
                .unwrap();
                let metadata =
                    check_source_theorem_metadata(&arena, &env, &variable_scope).unwrap();
                let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena,
                    &metadata.2,
                    &scope,
                    &env,
                    &labels,
                    &resolved,
                    None,
                )
                .unwrap();
                let path = "proof-witnesses/current λ/checked.json";
                let expected = fresh(
                    &arena,
                    &check,
                    &env,
                    session.captured().snapshot(),
                    id,
                    (
                        &vcs.vc(id).unwrap().anchor.generation_schema_version,
                        vcs.schema_version(),
                        profile.clone(),
                        format,
                    ),
                    (&policy, &verifier),
                    path,
                )
                .unwrap();
                assert_eq!(expected.0.0.trusted_used_axioms(), Some(&axioms));
                assert_eq!(
                    Some(expected.2.kernel_acceptance.accepted_result_hash.digest),
                    physical_input.accepted_evidence_hash()
                );
                assert_eq!(
                    expected.0.0.reuse_metadata().accepted_goal_polarity(),
                    physical_input.accepted_goal_polarity()
                );
                assert_eq!(
                    expected.0.0.reuse_metadata().selected_candidate_id(),
                    selected.selected_candidate_id()
                );
                assert_eq!(expected.2.witness_path, path);
                let bytes = canonical_json_bytes(&expected.1);
                let domain = artifact_hash_domain(
                    "mizar-proof/source-theorem-witness-file",
                    ArtifactVersion::new(1, 0),
                );
                assert_eq!(
                    domain.hash(&expected.1, &[]),
                    expected.2.witness_artifact_hash.digest
                );
                assert_eq!(
                    parse_canonical_json_bytes(&bytes, path).unwrap(),
                    expected.1
                );
                assert!(parse_canonical_json_bytes(&[0xff], path).is_err());
                assert_ne!(
                    domain.hash(&CanonicalJson::Null, &[]),
                    expected.2.witness_artifact_hash.digest
                );
                let before = (
                    four.clone(),
                    policy.clone(),
                    profile.clone(),
                    verifier.clone(),
                    session.captured().clone(),
                    (*physical).clone(),
                );
                fixture.write(&[0xff]);
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                let raw = inputs
                    .native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(
                        four,
                        owner,
                        id,
                        (profile.clone(), format),
                        (&policy, &verifier),
                        path,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(raw, (expected.1.clone(), expected.2.clone()));
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("KernelCheck"),
                    OutputKind::new("SourceTheoremCandidateWitnessFile"),
                    kernel.work_unit().clone(),
                ));
                let file = inputs
                    .publish_native_theorem_candidate_witness_file(
                        four,
                        owner,
                        id,
                        (profile.clone(), format),
                        (&policy, &verifier),
                        path,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(*owner.storage().get(&file).unwrap(), expected.1);
                assert_eq!(file.phase(), &IrPipelinePhase::new("KernelCheck"));
                assert_eq!(
                    file.output_kind(),
                    &OutputKind::new("SourceTheoremCandidateWitnessFile")
                );
                assert_eq!(
                    file.schema_version(),
                    mizar_ir::storage::SchemaVersion::new(1)
                );
                assert_eq!(file.work_unit(), kernel.work_unit());
                assert_eq!(file.snapshot(), four.snapshot);
                assert_eq!(file.lineage().parents, [kernel.output()]);
                assert_eq!(
                    file.lineage().named_input_hashes,
                    kernel.lineage().named_input_hashes
                );
                assert_eq!(
                    *owner.storage().side_tables(&file).unwrap(),
                    IrSideTables::default()
                );
                let mut content = blake3::Hasher::new();
                for value in [
                    b"mizar-ir/publisher-content-hash/v1".as_slice(),
                    bytes.as_slice(),
                ] {
                    content.update(&u64::try_from(value.len()).unwrap().to_le_bytes());
                    content.update(value);
                }
                content.update(&1u64.to_le_bytes());
                let value = b"parent_content";
                content.update(&u64::try_from(value.len()).unwrap().to_le_bytes());
                content.update(value);
                content.update(kernel.content_hash().as_bytes());
                content.update(&1u64.to_le_bytes());
                for value in [
                    "source",
                    SOURCE_UNIT_CACHE_KEY_VERSION,
                    "mizar-ir/publisher-named-input-hash/v1",
                ] {
                    content.update(&u64::try_from(value.len()).unwrap().to_le_bytes());
                    content.update(value.as_bytes());
                }
                content.update(source_key(&inputs.snapshot.source_versions[0]).as_bytes());
                assert_eq!(
                    file.content_hash(),
                    Hash::from_bytes(*content.finalize().as_bytes())
                );
                assert_ne!(file.content_hash(), expected.2.witness_artifact_hash.digest);
                let generic = mizar_proof::witness_store::stage(expected.0.2.clone()).unwrap();
                assert_ne!(
                    generic.staged_payload_hash().digest,
                    expected.2.witness_artifact_hash.digest
                );
                if let StoragePlacement::Blob { blob, len } = file.placement() {
                    assert!(open);
                    assert_eq!(*len, bytes.len());
                    let mut hash = blake3::Hasher::new();
                    let tag = b"mizar-ir/content-blob/v1";
                    hash.update(&u64::try_from(tag.len()).unwrap().to_le_bytes());
                    hash.update(tag);
                    hash.update(&1u32.to_le_bytes());
                    hash.update(&u64::try_from(bytes.len()).unwrap().to_le_bytes());
                    hash.update(&bytes);
                    assert_eq!(blob.hash(), Hash::from_bytes(*hash.finalize().as_bytes()));
                } else {
                    assert!(!open);
                }
                owner
                    .validate_published_output(four.snapshot, file.any())
                    .unwrap();
                assert_eq!(
                    owner
                        .validate_current_output(four.snapshot, file.any())
                        .is_ok(),
                    !open
                );
                assert_eq!(
                    (
                        four.clone(),
                        policy,
                        profile,
                        verifier,
                        session.captured().clone(),
                        (*physical).clone()
                    ),
                    before.clone()
                );
                let retained = four
                    .parent_outputs()
                    .iter()
                    .map(SealedParentOutputHandle::output)
                    .chain([file.output()])
                    .collect();
                let intermediate = owner.storage().collect(CollectInput {
                    snapshot: four.snapshot,
                    protected_outputs: retained,
                });
                assert_eq!(
                    (
                        intermediate.outputs_dropped,
                        intermediate.abandoned_slots_dropped
                    ),
                    (9, 0)
                );
                assert_eq!(
                    inputs
                        .native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(
                            four,
                            owner,
                            id,
                            (before.2.clone(), format),
                            (&before.1, &before.3),
                            path,
                        )
                        .unwrap(),
                    Some((expected.1.clone(), expected.2.clone()))
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: four.snapshot,
                    protected_outputs: vec![file.output()],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (4, 0)
                );
                assert_eq!(*owner.storage().get(&file).unwrap(), expected.1);
            },
        );
    }
}

#[test]
fn native_theorem_candidate_witness_file_preserves_refusals() {
    use mizar_atp::problem::*;
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    use mizar_proof::{
        policy::VerifierPolicy, status::source_theorem_candidate_witness_file_candidate as fresh,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{source_proof_inputs, source_theorem_module_original_inputs},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in ["ordinary", "denied", "obsolete", "collected"] {
        with_native_theorem_witness_kernel(
            text,
            false,
            0,
            |_, inputs, four, owner, refs, (session, _, _)| {
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default();
                let disabled = verifier.clone().with_kernel_evidence_formats([]);
                let profile = LogicProfile::try_new(
                    "witness-data",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc_ref = four
                    .parent_outputs()
                    .iter()
                    .find(|row| {
                        row.as_output_ref().output_kind() == &OutputKind::new("SourceTheoremVC")
                    })
                    .unwrap()
                    .as_output_ref();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc_ref, vc_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let id = vcs.vcs()[0].id;
                assert!(
                    inputs
                        .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                            four,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier)
                        )
                        .unwrap()
                        .is_some()
                );
                let frontend_ref = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("FrontendOutput"))
                    .unwrap();
                let frontend = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                frontend_ref.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let (arena, env) = source_theorem_module_original_inputs(ast, &module)
                    .unwrap()
                    .unwrap();
                let variable_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, &module, &env),
                )
                .unwrap();
                let metadata =
                    check_source_theorem_metadata(&arena, &env, &variable_scope).unwrap();
                let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena,
                    &metadata.2,
                    &scope,
                    &env,
                    &labels,
                    &resolved,
                    None,
                )
                .unwrap();
                let expected = fresh(
                    &arena,
                    &check,
                    &env,
                    session.captured().snapshot(),
                    id,
                    (
                        &vcs.vc(id).unwrap().anchor.generation_schema_version,
                        vcs.schema_version(),
                        profile.clone(),
                        ConcreteFormat::Tptp,
                    ),
                    (&policy, &verifier),
                    "proof-witnesses/checked.json",
                )
                .unwrap();
                let before = (
                    four.clone(),
                    policy.clone(),
                    verifier.clone(),
                    profile.clone(),
                    session.captured().clone(),
                );
                if case == "ordinary" {
                    let alien = publisher(four.snapshot, 0, &four.work_unit);
                    for (shape, current) in [
                        ("missing", owner.as_ref()),
                        ("surplus", owner.as_ref()),
                        ("module", owner.as_ref()),
                        ("foreign", alien.as_ref()),
                    ] {
                        let mut parents = four.parent_outputs().to_vec();
                        if shape == "missing" {
                            parents.pop();
                        }
                        if shape == "surplus" {
                            parents.push(
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    four.snapshot,
                                    frontend_ref.clone(),
                                )
                                .unwrap(),
                            );
                        }
                        let work_unit = if shape == "module" {
                            WorkUnit::Module {
                                module: mizar_build::module_index::ModuleId::new(
                                    PackageId::new("alpha"),
                                    ModulePath::new("other"),
                                ),
                            }
                        } else {
                            four.work_unit.clone()
                        };
                        let input = PhaseInput::new(
                            work_unit,
                            PhaseDispatchInputBundle::new(
                                four.snapshot,
                                four.identities().input_hash(),
                                four.identities().dependency_hashes().to_vec(),
                                parents,
                            )
                            .unwrap(),
                        );
                        let error = inputs.native_theorem_candidate_selection_with_core_vc_discharge_check_parents(&input, current, id, (profile.clone(), ConcreteFormat::Tptp), (&policy, &verifier)).unwrap_err();
                        assert_eq!(inputs.native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(&input, current, id, (profile.clone(), ConcreteFormat::Tptp), (&policy, &verifier), "proof-witnesses/checked.json").unwrap_err(), error);
                        assert_eq!(
                            inputs
                                .publish_native_theorem_candidate_witness_file(
                                    &input,
                                    current,
                                    id,
                                    (profile.clone(), ConcreteFormat::Tptp),
                                    (&policy, &verifier),
                                    "proof-witnesses/checked.json"
                                )
                                .unwrap_err(),
                            error
                        );
                    }
                    let error = inputs
                        .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                            four,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &disabled),
                        )
                        .unwrap_err();
                    assert_eq!(inputs.native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(four, owner, id, (profile.clone(), ConcreteFormat::Tptp), (&policy, &disabled), "").unwrap_err(), error);
                    let path_error = fresh(
                        &arena,
                        &check,
                        &env,
                        session.captured().snapshot(),
                        id,
                        (
                            &vcs.vc(id).unwrap().anchor.generation_schema_version,
                            vcs.schema_version(),
                            profile.clone(),
                            ConcreteFormat::Tptp,
                        ),
                        (&policy, &verifier),
                        "",
                    )
                    .unwrap_err();
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_witness_file(
                                four,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ""
                            )
                            .unwrap_err(),
                        path_error
                    );
                } else if case == "denied" {
                    assert_eq!(inputs.native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(four, owner, id, (profile.clone(), ConcreteFormat::Tptp), (&policy, &verifier), "proof-witnesses/checked.json").unwrap(), Some((expected.1, expected.2)));
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_witness_file(
                                four,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                "proof-witnesses/checked.json"
                            )
                            .unwrap_err(),
                        PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("KernelCheck"),
                            work_unit: IrWorkUnit::new("\"alpha\":\"main\""),
                            output_kind: OutputKind::new("SourceTheoremCandidateWitnessFile")
                        }
                        .to_string()
                    );
                } else {
                    if case == "obsolete" {
                        owner.mark_obsolete(four.snapshot).unwrap();
                    } else {
                        let keep = four
                            .parent_outputs()
                            .iter()
                            .filter(|row| {
                                row.as_output_ref().output_kind()
                                    != &OutputKind::new("SourceTheoremCandidateCheck")
                            })
                            .map(SealedParentOutputHandle::output)
                            .collect();
                        assert_eq!(
                            owner
                                .storage()
                                .collect(CollectInput {
                                    snapshot: four.snapshot,
                                    protected_outputs: keep
                                })
                                .outputs_dropped,
                            10
                        );
                    }
                    let error = inputs
                        .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                            four,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier),
                        )
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .publish_native_theorem_candidate_witness_file(
                                four,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                "proof-witnesses/checked.json"
                            )
                            .unwrap_err(),
                        error
                    );
                }
                assert_eq!(
                    (
                        four.clone(),
                        policy,
                        verifier,
                        profile,
                        session.captured().clone()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: four.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected" { 3 } else { 13 }
                );
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(case == "denied")
                );
            },
        );
    }
}
