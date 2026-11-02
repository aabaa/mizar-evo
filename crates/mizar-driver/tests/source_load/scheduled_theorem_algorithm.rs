use super::*;

#[test]
fn scheduled_theorem_algorithm_preparation_matches_genuine_publication() {
    use mizar_core::{
        control_flow::ObligationHandoffOrigin,
        core_ir::{CoreProofStatus, ObligationSeedStatus},
    };
    for (text, open, threshold) in [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            false,
            usize::MAX,
        ),
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            true,
            0,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, inputs, old, baseline, _, (session, ids, snapshots)| {
                let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                for (phase, kind) in [
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ("OverloadResolution", "SourceTheoremMetadata"),
                    ("Elaboration", "SourceTheoremCore"),
                    ("AlgorithmPreparation", "SourceTheoremPreparation"),
                ] {
                    baseline.allow_work_unit(AllowedWorkUnit::new(
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
                                        baseline,
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
                    .publish_native_theorem_module_summary(old, baseline)
                    .unwrap()
                    .unwrap();
                let type_input = child(vec![signature.erase()]);
                let typed = inputs
                    .publish_native_theorem_typed_ast(&type_input, baseline)
                    .unwrap()
                    .unwrap();
                let reg_input = child(vec![typed.erase()]);
                let registration = inputs
                    .publish_native_theorem_registration_closure(&reg_input, baseline)
                    .unwrap()
                    .unwrap();
                let meta_input = child(vec![typed.erase(), registration.erase()]);
                let metadata_parent = inputs
                    .publish_native_theorem_metadata(&meta_input, baseline)
                    .unwrap()
                    .unwrap();
                let core_input = child(vec![metadata_parent.erase()]);
                let core_parent = inputs
                    .publish_native_theorem_core_with_metadata_parent(&core_input, baseline)
                    .unwrap()
                    .unwrap();
                let preparation_input = child(vec![core_parent.erase()]);
                let expected = inputs
                    .publish_native_theorem_preparation(&preparation_input, baseline)
                    .unwrap()
                    .unwrap();
                let preparation = baseline.storage().get(&expected).unwrap();
                let core = baseline.storage().get(&core_parent).unwrap();
                assert!(preparation.0.flows.is_empty() && preparation.0.flow_map.is_empty());
                assert_eq!(preparation.1.entries.len(), core.obligation_seeds().len());
                assert!(!preparation.1.entries.is_empty());
                for (id, entry) in preparation.1.entries.iter() {
                    let ObligationHandoffOrigin::ExistingCore { seed } = entry.origin else {
                        panic!("existing Core seed")
                    };
                    assert_eq!(&entry.seed, core.obligation_seeds().get(seed).unwrap());
                    assert_eq!(
                        preparation.1.source_map.get(&id),
                        Some(&core.source_map().obligation_sources[&seed])
                    );
                    assert!(entry.flow_site.is_none());
                }
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                assert_eq!(
                    core.source_id(),
                    inputs.snapshot.source_versions[0].source_id
                );
                assert_eq!(core.module_id(), &module);
                assert!(!core.proofs().is_empty());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(!core.obligation_seeds().is_empty());
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                );
                let captured = session.captured().clone();
                if open {
                    fixture.write(&[0xff]);
                } else {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                }
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                let mut previous = None;
                for workers in [1, 4] {
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
                        .register_source_theorem_algorithm_preparation();
                    let mut driver = CompilerDriver::new(builder.build().unwrap())
                        .with_output_publisher(owner.clone());
                    let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                    submit.worker_count = workers;
                    submit.completion_order = if workers == 1 {
                        CompletionOrder::Canonical
                    } else {
                        CompletionOrder::Reverse
                    };
                    let submission = driver
                        .submit_captured(session.clone(), ids, snapshots, submit)
                        .unwrap();
                    assert_eq!(
                        submission.status,
                        DriverSubmissionStatus::BlockedByMissingPhaseServices
                    );
                    assert_eq!(
                        submission.session.state(),
                        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                    );
                    assert!(
                        submission
                            .missing_services
                            .iter()
                            .any(|gap| gap.phase == PipelinePhase::VcGenerate)
                    );
                    assert!(!submission.missing_services.iter().any(|gap| matches!(
                        gap.phase,
                        PipelinePhase::SignatureCollection
                            | PipelinePhase::TypeChecking
                            | PipelinePhase::RegistrationResolution
                            | PipelinePhase::OverloadResolution
                            | PipelinePhase::Elaboration
                            | PipelinePhase::AlgorithmPreparation
                    )));
                    let graph = submission.task_graph.as_ref().unwrap();
                    let task = graph
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == TaskKind::CheckAndElaborate && task.unit == old.work_unit
                        })
                        .unwrap();
                    let run = submission.scheduler_run.as_ref().unwrap();
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|row| row.task_id == task.id)
                            .unwrap()
                            .state,
                        TaskState::Completed
                    );
                    let [
                        sig_result,
                        type_result,
                        reg_result,
                        meta_result,
                        core_result,
                        prep_result,
                    ] = run.phase_results[&task.id].as_slice()
                    else {
                        panic!("six real same-task rows")
                    };
                    for result in [
                        sig_result,
                        type_result,
                        reg_result,
                        meta_result,
                        core_result,
                        prep_result,
                    ] {
                        assert_eq!(result.status, PhaseStatus::Complete);
                        assert!(result.diagnostics.is_empty());
                        assert_eq!(result.output_refs.len(), 1);
                    }
                    let actual_sig = &sig_result.output_refs[0];
                    let actual_type = &type_result.output_refs[0];
                    let actual_reg = &reg_result.output_refs[0];
                    let actual_metadata = &meta_result.output_refs[0];
                    let actual_core = &core_result.output_refs[0];
                    let actual = &prep_result.output_refs[0];
                    assert_eq!(actual_sig, &signature.erase());
                    assert_eq!(actual_type, &typed.erase());
                    assert_eq!(actual_reg, &registration.erase());
                    assert_eq!(actual_metadata, &metadata_parent.erase());
                    assert_eq!(actual_core, &core_parent.erase());
                    assert_eq!(actual, &expected.erase());
                    assert_eq!(actual_type.lineage().parents, vec![actual_sig.output()]);
                    assert_eq!(actual_reg.lineage().parents, vec![actual_type.output()]);
                    let mut expected_parents = vec![actual_type.output(), actual_reg.output()];
                    expected_parents.sort_by(|a, b| a.hash().as_bytes().cmp(b.hash().as_bytes()));
                    assert_eq!(actual_metadata.lineage().parents, expected_parents);
                    assert_eq!(
                        actual_core.lineage().parents,
                        vec![actual_metadata.output()]
                    );
                    assert_eq!(actual.lineage().parents, vec![actual_core.output()]);
                    let actual_handle = owner
                        .storage()
                        .typed_handle::<(
                            mizar_core::control_flow::ControlFlowOutput,
                            mizar_core::control_flow::ObligationSeedHandoff,
                        )>(actual, actual.output_kind())
                        .unwrap();
                    assert_eq!(*owner.storage().get(&actual_handle).unwrap(), *preparation);
                    assert_eq!(
                        owner.storage().side_tables(&actual_handle).unwrap(),
                        baseline.storage().side_tables(&expected).unwrap()
                    );
                    assert_eq!(actual.placement() == &StoragePlacement::Resident, !open);
                    if let StoragePlacement::Blob { blob, len } = actual.placement() {
                        let StoragePlacement::Blob {
                            blob: expected_blob,
                            len: expected_len,
                        } = expected.placement()
                        else {
                            panic!("blob baseline")
                        };
                        assert_eq!(blob, expected_blob);
                        assert_eq!(len, expected_len);
                    }
                    let phase = PipelinePhase::AlgorithmPreparation;
                    let parents = [actual_core];
                    let hashes = parents
                        .iter()
                        .map(|parent| parent.output().hash())
                        .collect::<Vec<_>>();
                    let derived = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            old.identities().input_hash(),
                            old.identities().dependency_hashes().to_vec(),
                            parents
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
                    assert_eq!(derived.identities().parent_output_hashes(), hashes);
                    let probe = driver
                        .registry()
                        .execute_phase_with_resources(
                            phase,
                            derived.clone(),
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_eq!(prep_result.cache_observation, Some(probe.observation));
                    let cache = driver
                        .registry()
                        .cache_key_for_phase(phase, &derived)
                        .unwrap();
                    assert!(
                        matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                    );
                    assert_ne!(
                        derived.identities().parent_output_hashes(),
                        old.identities().parent_output_hashes()
                    );
                    let wrong = driver
                        .registry()
                        .execute_phase_with_resources(
                            phase,
                            old.clone(),
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_ne!(probe.observation, wrong.observation);
                    let vc = graph
                        .tasks()
                        .iter()
                        .find(|candidate| {
                            candidate.kind == TaskKind::VcGenerate
                                && candidate.unit == old.work_unit
                        })
                        .unwrap();
                    assert_eq!(vc.phases, [PipelinePhase::VcGenerate]);
                    assert!(vc.dependencies.contains(&task.id));
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|row| row.task_id == vc.id)
                            .unwrap()
                            .state,
                        TaskState::Blocked
                    );
                    assert!(!run.phase_results.contains_key(&vc.id));
                    let dependents = graph
                        .tasks()
                        .iter()
                        .filter(|candidate| candidate.dependencies.contains(&task.id))
                        .collect::<Vec<_>>();
                    assert!(!dependents.is_empty());
                    for dependent in dependents {
                        assert_eq!(
                            run.task_states
                                .iter()
                                .find(|row| row.task_id == dependent.id)
                                .unwrap()
                                .state,
                            TaskState::Blocked
                        );
                        assert!(!run.phase_results.contains_key(&dependent.id));
                    }
                    assert_eq!(
                        run.phase_results
                            .values()
                            .flatten()
                            .flat_map(|result| &result.output_refs)
                            .map(|output| output.phase().as_str())
                            .collect::<std::collections::BTreeSet<_>>(),
                        [
                            "SourceLoad",
                            "Frontend",
                            "ModuleResolve",
                            "SignatureCollection",
                            "TypeChecking",
                            "RegistrationResolution",
                            "OverloadResolution",
                            "Elaboration",
                            "AlgorithmPreparation"
                        ]
                        .into_iter()
                        .collect()
                    );
                    owner
                        .validate_published_output(old.snapshot, actual)
                        .unwrap();
                    assert_eq!(
                        owner.validate_current_output(old.snapshot, actual).is_ok(),
                        !open
                    );
                    if let Some(previous) = &previous {
                        assert_eq!(run.phase_results[&task.id], *previous);
                    }
                    previous = Some(run.phase_results[&task.id].clone());
                    assert_eq!(submission.session.captured(), &captured);
                }
                assert_eq!(session.captured(), &captured);
            },
        );
    }
}

#[test]
fn scheduled_theorem_algorithm_preparation_preserves_resource_refusals() {
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "cancel",
        "foreign-token",
        "no-publisher",
        "no-inputs",
        "obsolete",
        "collected",
        "denied",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            let unit = IrWorkUnit::new("\"alpha\":\"main\"");
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
            let metadata_input = child(vec![typed.erase(), registration.erase()]);
            let metadata = inputs
                .publish_native_theorem_metadata(&metadata_input, owner)
                .unwrap()
                .unwrap();
            let core_input = child(vec![metadata.erase()]);
            let core = inputs
                .publish_native_theorem_core_with_metadata_parent(&core_input, owner)
                .unwrap()
                .unwrap();
            let input = child(vec![core.erase()]);
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("AlgorithmPreparation"),
                    OutputKind::new("SourceTheoremPreparation"),
                    unit,
                ));
            }
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
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_theorem_algorithm_preparation();
            let registry = builder.build().unwrap();
            let descriptor = registry
                .descriptor_for_phase(PipelinePhase::AlgorithmPreparation)
                .unwrap();
            assert_eq!(descriptor.service_name, "SourceTheoremAlgorithmPreparation");
            assert_eq!(
                descriptor.owner,
                mizar_driver::registry::PhaseOwner::MizarCore
            );
            assert_eq!(descriptor.phases, [PipelinePhase::AlgorithmPreparation]);
            assert_eq!(
                descriptor.schema_version,
                "mizar-core/source-theorem-preparation-publication/v1"
            );
            assert_eq!(descriptor.output_kind, "SourceTheoremPreparation");
            let result = registry
                .execute_phase_with_resources(
                    PipelinePhase::AlgorithmPreparation,
                    input,
                    PhaseExecutionResources {
                        cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                            CancellationToken {
                                snapshot: if case == "foreign-token" {
                                    snapshot_id(0xd5)
                                } else {
                                    old.snapshot
                                },
                                generation: CancellationGeneration::new(1),
                                reason: CancellationReason::ExplicitRequest,
                            },
                        ),
                        output_publisher: (case != "no-publisher").then(|| owner.clone()),
                        source_load: (case != "no-inputs").then_some(inputs),
                        ..PhaseExecutionResources::default()
                    },
                )
                .unwrap();
            assert_eq!(
                result.result.status,
                if case == "cancel" {
                    PhaseStatus::Cancelled
                } else {
                    PhaseStatus::Blocking
                }
            );
            assert!(result.result.output_refs.is_empty());
            assert!(result.result.diagnostics.is_empty());
            assert_eq!(result.result.cache_observation, Some(result.observation));
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
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
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for source in [private, lemma] {
        with_native_summary_parent(
            source,
            false,
            0,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let owner = publisher(old.snapshot, 0, &old.work_unit);
                allow_frontend(&owner, &old.work_unit);
                for (phase, kind) in [
                    ("ModuleResolve", "SourceTheoremModuleCandidates"),
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
                    .register_source_theorem_algorithm_preparation();
                let (_, submit) = fixture.submit_request(vec![], vec![]);
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner)
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                let graph = submission.task_graph.as_ref().unwrap();
                let task = graph
                    .tasks()
                    .iter()
                    .find(|task| task.kind == TaskKind::CheckAndElaborate)
                    .unwrap();
                let run = submission.scheduler_run.as_ref().unwrap();
                let rows = &run.phase_results[&task.id];
                assert_eq!(rows.len(), 5);
                let (blocked, completed) = rows.split_last().unwrap();
                assert!(
                    completed
                        .iter()
                        .all(|row| row.status == PhaseStatus::Complete
                            && row.diagnostics.is_empty()
                            && row.output_refs.len() == 1)
                );
                assert_eq!(blocked.status, PhaseStatus::Blocking);
                assert!(blocked.output_refs.is_empty());
                assert!(blocked.diagnostics.is_empty());
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|row| row.task_id == task.id)
                        .unwrap()
                        .state,
                    TaskState::Blocked
                );
                assert_eq!(
                    submission.session.state(),
                    BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                );
                assert!(
                    !run.phase_results
                        .values()
                        .flatten()
                        .flat_map(|result| &result.output_refs)
                        .any(|output| matches!(
                            output.phase().as_str(),
                            "Elaboration" | "AlgorithmPreparation"
                        ))
                );
            },
        );
    }
}
