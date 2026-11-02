use super::*;

#[test]
fn scheduled_theorem_registration_resolution_matches_genuine_publication() {
    use mizar_checker::{
        cluster_trace::{ClusterClosureOutput, ClusterClosureStatus},
        registration_resolution::RegistrationDatabase,
    };
    for (text, open, threshold) in [
        (
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            ),
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
            |fixture, inputs, old, baseline, outputs, (session, ids, snapshots)| {
                let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                for (phase, kind) in [
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ] {
                    baseline.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        unit.clone(),
                    ));
                }
                let signature = inputs
                    .publish_native_theorem_module_summary(old, baseline)
                    .unwrap()
                    .unwrap();
                let type_input = PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        old.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                baseline,
                                old.snapshot,
                                signature.erase(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let typed = inputs
                    .publish_native_theorem_typed_ast(&type_input, baseline)
                    .unwrap()
                    .unwrap();
                let reg_input = PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        old.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                baseline,
                                old.snapshot,
                                typed.erase(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let expected = inputs
                    .publish_native_theorem_registration_closure(&reg_input, baseline)
                    .unwrap()
                    .unwrap();
                let closure = baseline.storage().get(&expected).unwrap();
                assert_eq!(closure.status(), ClusterClosureStatus::Complete);
                assert_eq!(
                    closure.trace().source_id(),
                    inputs.snapshot.source_versions[0].source_id
                );
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                assert_eq!(closure.trace().module_id(), &module);
                assert_eq!(
                    closure
                        .trace()
                        .traversal_profile()
                        .cache_key_material()
                        .as_str(),
                    "order=cluster-trace-order-v1;max_depth=128;max_generated=4096"
                );
                assert!(closure.closure_facts().is_empty());
                assert!(closure.diagnostics().is_empty());
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let frontend = baseline
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        frontend,
                        frontend.output_kind(),
                    )
                    .unwrap();
                let retained = baseline.storage().get(&frontend).unwrap();
                let ast = retained.ast.as_ref().unwrap();
                let (_, env) =
                    mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
                        .unwrap()
                        .unwrap();
                let database = RegistrationDatabase::from_symbol_env(&env, []);
                assert!(database.pending().is_empty());
                assert!(database.activated().is_empty());
                assert!(database.rejected().is_empty());
                assert!(database.diagnostics().is_empty());
                assert!(database.initial_obligations().is_empty());
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
                        .register_source_theorem_registration_resolution();
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
                            .any(|gap| gap.phase == PipelinePhase::OverloadResolution)
                    );
                    assert!(!submission.missing_services.iter().any(|gap| matches!(
                        gap.phase,
                        PipelinePhase::SignatureCollection
                            | PipelinePhase::TypeChecking
                            | PipelinePhase::RegistrationResolution
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
                        TaskState::Blocked
                    );
                    let [sig_result, type_result, reg_result] =
                        run.phase_results[&task.id].as_slice()
                    else {
                        panic!("three genuine same-task phase results")
                    };
                    for result in [sig_result, type_result, reg_result] {
                        assert_eq!(result.status, PhaseStatus::Complete);
                        assert!(result.diagnostics.is_empty());
                        assert_eq!(result.output_refs.len(), 1);
                    }
                    let actual_sig = &sig_result.output_refs[0];
                    let actual_type = &type_result.output_refs[0];
                    let actual = &reg_result.output_refs[0];
                    assert_eq!(actual_sig, &signature.erase());
                    assert_eq!(actual_type, &typed.erase());
                    assert_eq!(actual, &expected.erase());
                    assert_eq!(actual_type.lineage().parents, vec![actual_sig.output()]);
                    assert_eq!(actual.lineage().parents, vec![actual_type.output()]);
                    let actual_handle = owner
                        .storage()
                        .typed_handle::<ClusterClosureOutput>(actual, actual.output_kind())
                        .unwrap();
                    let actual_closure = owner.storage().get(&actual_handle).unwrap();
                    assert_eq!(*actual_closure, *closure);
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
                    let module_parent = run
                        .phase_results
                        .values()
                        .flatten()
                        .flat_map(|result| &result.output_refs)
                        .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                        .unwrap();
                    for (phase, parent, result) in [
                        (
                            PipelinePhase::SignatureCollection,
                            module_parent,
                            sig_result,
                        ),
                        (PipelinePhase::TypeChecking, actual_sig, type_result),
                        (
                            PipelinePhase::RegistrationResolution,
                            actual_type,
                            reg_result,
                        ),
                    ] {
                        let derived = PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                old.identities().dependency_hashes().to_vec(),
                                vec![
                                    SealedParentOutputHandle::from_published_output(
                                        &owner,
                                        old.snapshot,
                                        parent.clone(),
                                    )
                                    .unwrap(),
                                ],
                            )
                            .unwrap(),
                        );
                        assert_eq!(
                            derived.identities().parent_output_hashes(),
                            &[parent.output().hash()]
                        );
                        let probe = driver
                            .registry()
                            .execute_phase_with_resources(
                                phase,
                                derived.clone(),
                                PhaseExecutionResources::default(),
                            )
                            .unwrap();
                        assert_eq!(result.cache_observation, Some(probe.observation));
                        let cache = driver
                            .registry()
                            .cache_key_for_phase(phase, &derived)
                            .unwrap();
                        assert!(
                            matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                        );
                        if phase != PipelinePhase::SignatureCollection {
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
                        }
                    }
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
                            "RegistrationResolution"
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
fn scheduled_theorem_registration_resolution_preserves_resource_refusals() {
    let text = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
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
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    unit.clone(),
                ));
            }
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = PhaseInput::new(
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
            let typed = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
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
                            typed.erase(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("RegistrationResolution"),
                    OutputKind::new("SourceTheoremRegistrationClosure"),
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
                    5
                );
            }
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_theorem_registration_resolution();
            let registry = builder.build().unwrap();
            let descriptor = registry
                .descriptor_for_phase(PipelinePhase::RegistrationResolution)
                .unwrap();
            assert_eq!(
                descriptor.service_name,
                "SourceTheoremRegistrationResolution"
            );
            assert_eq!(
                descriptor.owner,
                mizar_driver::registry::PhaseOwner::MizarChecker
            );
            assert_eq!(descriptor.phases, [PipelinePhase::RegistrationResolution]);
            assert_eq!(
                descriptor.schema_version,
                "mizar-checker/source-theorem-registration-closure-publication/v1"
            );
            assert_eq!(descriptor.output_kind, "SourceTheoremRegistrationClosure");
            let result = registry
                .execute_phase_with_resources(
                    PipelinePhase::RegistrationResolution,
                    input,
                    PhaseExecutionResources {
                        cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                            CancellationToken {
                                snapshot: if case == "foreign-token" {
                                    snapshot_id(0xe7)
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
                if case == "collected" { 0 } else { 5 }
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
    let unsupported = private.replace("set", "object");
    with_native_summary_parent(
        &unsupported,
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
                .register_source_theorem_registration_resolution();
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
            let [signature, typed] = run.phase_results[&task.id].as_slice() else {
                panic!("unsupported source stops before registration")
            };
            assert_eq!(signature.status, PhaseStatus::Complete);
            assert_eq!(typed.status, PhaseStatus::Blocking);
            assert!(typed.output_refs.is_empty());
            assert!(typed.diagnostics.is_empty());
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
            assert!(!run.phase_results.values().flatten().flat_map(|result| &result.output_refs).any(|output| output.phase() == &IrPipelinePhase::new("RegistrationResolution")));
        },
    );
}
