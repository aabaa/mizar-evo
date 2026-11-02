use super::*;

#[test]
fn native_theorem_reflexivity_core_vc_caller_matches_genuine_check() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{CandidatePolicyClass, VerifierPolicy},
        status::check_source_theorem_reflexivity_with_original_graph as check_original,
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, SeedOriginRef, VcSchemaVersion, VcStatus};
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        for binder in ["X", "Renamed"] {
            let text = format!(
                ":: captured λ\ntheorem T: for {binder} being set holds {binder} = {binder} proof let {binder} be set; thus {binder} = {binder}; end;"
            );
            with_native_summary_parent(
                &text,
                open,
                threshold,
                |fixture, inputs, old, owner, outputs, (session, _, _)| {
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
                        .publish_native_theorem_registration_closure(
                            &child(vec![typed.erase()]),
                            owner,
                        )
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
                        .publish_native_theorem_preparation(
                            &child(vec![core_parent.erase()]),
                            owner,
                        )
                        .unwrap()
                        .unwrap();
                    let generation = GenerationSchemaVersion::new("source-theorem-local-v1");
                    let schema = VcSchemaVersion::new("vc-v1");
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
                    let module =
                        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                    let (arena, env) =
                        mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
                            .unwrap()
                            .unwrap();
                    assert!(
                        mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)
                            .unwrap()
                            .is_some()
                    );
                    let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(mizar_resolve::names::SourceVariableScopeInput::new(ast, &module, &env)).unwrap();
                    let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                        &arena, &env, &scope,
                    )
                    .unwrap();
                    let (scope, labels, resolved) =
                        mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                    let check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(&arena, &metadata.2, &scope, &env, &labels, &resolved, None).unwrap();
                    assert_eq!(core.source_id(), ast.source_id);
                    assert_eq!(core.module_id(), &module);
                    assert_eq!(vcs.source(), ast.source_id);
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.snapshot(), input.snapshot);
                    assert_eq!(vcs.schema_version().as_str(), "vc-v1");
                    assert!(
                        !core.proofs().is_empty()
                            && !vcs.vcs().is_empty()
                            && !vcs.seed_accounting().is_empty()
                    );
                    assert!(
                        core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open
                        && vc.anchor.generation_schema_version.as_str()
                            == "source-theorem-local-v1"));
                    for row in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!("original Core seed")
                        };
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(
                            core.obligation_seeds().get(seed).unwrap().status,
                            ObligationSeedStatus::Active
                        );
                    }
                    assert_eq!(handoff.diagnostics().vc, selected);
                    let target = handoff.canonical_evidence().target_vc();
                    let before = (
                        input.clone(),
                        (*core).clone(),
                        (*vcs).clone(),
                        session.captured().clone(),
                    );
                    for policy in [
                        VerifierPolicy::default(),
                        VerifierPolicy::release().with_require_kernel_certificates(true),
                    ] {
                        let policy_before = policy.clone();
                        let (checked_core, checked_vcs, result, decision) =
                            check_original(&arena, &check, &env, input.snapshot, selected, &policy)
                                .unwrap();
                        assert_eq!(checked_core, *core);
                        assert_eq!(checked_vcs, *vcs);
                        assert_eq!(result.status(), KernelCheckStatus::Accepted);
                        assert_eq!(
                            result.evidence_check_kind(),
                            Some(KernelEvidenceCheckKind::ProofObligation)
                        );
                        assert!(!result.policy_taint());
                        assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                        assert_eq!(
                            result.target_vc_fingerprint(),
                            &mizar_kernel::rejection::TargetVcFingerprint::new(
                                target.algorithm_id,
                                target.digest.clone()
                            )
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
                                        .native_theorem_reflexivity_with_core_vc_parents(
                                            &input, owner, selected, &policy
                                        )
                                        .unwrap(),
                                    Some((result.clone(), decision.clone()))
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
}

#[test]
fn native_theorem_reflexivity_core_vc_caller_preserves_refusals() {
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
    for case in [
        "obsolete",
        "collected-vc",
        "dependencies",
        "absent",
        "source-none",
        "generation",
        "vc-schema",
    ] {
        let generation = GenerationSchemaVersion::new(if case == "generation" {
            "custom-generation"
        } else {
            "source-theorem-local-v1"
        });
        let schema = VcSchemaVersion::new(if case == "vc-schema" {
            "custom-vc"
        } else {
            "vc-v1"
        });
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
                let metadata_input = child(vec![parent.erase()]);
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
                    let raw = inputs
                        .native_theorem_preparation_with_core_parent(&core_input, owner)
                        .unwrap();
                    assert!(raw.is_none());
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
                let disabled =
                    mizar_proof::policy::VerifierPolicy::default().with_kernel_evidence_formats([]);
                let raw = inputs
                    .native_theorem_kernel_handoff_with_core_vc_parents(&input, owner, selected);
                if case == "source-none" {
                    assert!(raw.unwrap().is_none());
                    assert_eq!(
                        inputs
                            .native_theorem_reflexivity_with_core_vc_parents(
                                &input, owner, selected, &disabled
                            )
                            .unwrap(),
                        None
                    );
                } else if matches!(case, "generation" | "vc-schema") {
                    assert!(raw.unwrap().is_some());
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
                    let module = core.module_id();
                    let (arena, env) =
                        mizar_resolve::symbols::source_theorem_module_original_inputs(ast, module)
                            .unwrap()
                            .unwrap();
                    assert!(
                        mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)
                            .unwrap()
                            .is_some()
                    );
                    let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_proof_occurrences(mizar_resolve::names::SourceVariableScopeInput::new(ast, module, &env)).unwrap();
                    let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                        &arena, &env, &scope,
                    )
                    .unwrap();
                    let (scope, labels, resolved) =
                        mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                    let check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(&arena, &metadata.2, &scope, &env, &labels, &resolved, None).unwrap();
                    let policy = mizar_proof::policy::VerifierPolicy::default();
                    let (checked_core, checked_vcs, result, _) =
                        mizar_proof::status::check_source_theorem_reflexivity_with_original_graph(
                            &arena,
                            &check,
                            &env,
                            input.snapshot,
                            selected,
                            &policy,
                        )
                        .unwrap();
                    assert_eq!(
                        result.status(),
                        mizar_kernel::checker::KernelCheckStatus::Accepted
                    );
                    assert_eq!(checked_core, *core);
                    assert_ne!(checked_vcs, *vcs);
                    assert_eq!(
                        inputs
                            .native_theorem_reflexivity_with_core_vc_parents(
                                &input, owner, selected, &policy
                            )
                            .unwrap_err(),
                        "native theorem reflexivity original Core/VC mismatch"
                    );
                    let expected =
                        mizar_proof::status::check_source_theorem_reflexivity_with_original_graph(
                            &arena,
                            &check,
                            &env,
                            input.snapshot,
                            selected,
                            &disabled,
                        )
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .native_theorem_reflexivity_with_core_vc_parents(
                                &input, owner, selected, &disabled
                            )
                            .unwrap_err(),
                        expected
                    );
                } else {
                    let expected = raw.unwrap_err();
                    if case == "dependencies" {
                        assert_eq!(expected, upstream.unwrap());
                    }
                    assert_eq!(
                        inputs
                            .native_theorem_reflexivity_with_core_vc_parents(
                                &input, owner, selected, &disabled
                            )
                            .unwrap_err(),
                        expected
                    );
                }
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
