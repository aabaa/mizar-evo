use super::*;

#[test]
fn scheduled_theorem_signature_matches_genuine_summary_and_worker_replay() {
    use mizar_artifact::module_summary::ModuleSummary;
    use mizar_resolve::{env::SymbolEnv, resolved_ast::ResolvedAst};
    for (text, open, threshold) in [
        (
            include_str!(
                "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
            ),
            false,
            usize::MAX,
        ),
        (
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
            ),
            true,
            0,
        ),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, inputs, input, baseline_owner, outputs, (session, ids, snapshots)| {
                let captured = session.captured().clone();
                let WorkUnit::Module { module } = &input.work_unit else {
                    unreachable!()
                };
                let expected_pair = inputs
                    .native_theorem_module_candidates(module)
                    .unwrap()
                    .unwrap();
                let expected_summary = inputs
                    .native_theorem_module_summary_candidate(module)
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    expected_summary.source_hash,
                    captured.snapshot().source_versions[0].source_hash
                );
                assert!(expected_summary.lexical_summary.fingerprint.is_none());
                assert!(
                    expected_summary
                        .exported_symbols
                        .iter()
                        .all(|row| row.proof_status.is_none())
                );
                assert!(expected_pair.1.symbols().iter().any(|symbol| matches!(symbol.signature(), Some(mizar_resolve::env::SignatureShell::Opaque { schema, .. }) if schema == "source-theorem-statement-tokens-v1")));
                let baseline_parent = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                    .unwrap();
                baseline_owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SignatureCollection"),
                    OutputKind::new("SourceTheoremModuleSummary"),
                    baseline_parent.work_unit().clone(),
                ));
                let expected = inputs
                    .publish_native_theorem_module_summary(input, baseline_owner)
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    *baseline_owner.storage().get(&expected).unwrap(),
                    expected_summary
                );
                let mut previous = None;
                for workers in [1, 4] {
                    let owner = publisher(input.snapshot, threshold, &input.work_unit);
                    // Each independent publisher explicitly grants all four output rights.
                    allow_frontend(&owner, &input.work_unit);
                    for (phase, kind) in [
                        ("ModuleResolve", "SourceTheoremModuleCandidates"),
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            baseline_parent.work_unit().clone(),
                        ));
                    }
                    let mut builder = PhaseRegistryBuilder::new();
                    builder
                        .register_source_load()
                        .register_frontend(vec![])
                        .register_source_theorem_module_resolver()
                        .register_source_theorem_signature_collection();
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
                            .any(|gap| gap.phase == PipelinePhase::TypeChecking)
                    );
                    assert!(
                        !submission
                            .missing_services
                            .iter()
                            .any(|gap| gap.phase == PipelinePhase::SignatureCollection)
                    );
                    let graph = submission.task_graph.as_ref().unwrap();
                    let task = graph
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == TaskKind::CheckAndElaborate && task.unit == input.work_unit
                        })
                        .unwrap();
                    assert_eq!(
                        task.phases,
                        [
                            PipelinePhase::SignatureCollection,
                            PipelinePhase::TypeChecking,
                            PipelinePhase::RegistrationResolution,
                            PipelinePhase::OverloadResolution,
                            PipelinePhase::Elaboration,
                            PipelinePhase::AlgorithmPreparation
                        ]
                    );
                    let run = submission.scheduler_run.as_ref().unwrap();
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|row| row.task_id == task.id)
                            .unwrap()
                            .state,
                        TaskState::Blocked
                    );
                    let [result] = run.phase_results[&task.id].as_slice() else {
                        panic!("one signature result before missing TypeChecking")
                    };
                    assert_eq!(result.status, PhaseStatus::Complete);
                    assert!(result.diagnostics.is_empty());
                    let [actual] = result.output_refs.as_slice() else {
                        panic!("one genuine summary")
                    };
                    assert_eq!(actual, &expected.erase());
                    assert_eq!(actual.lineage(), expected.lineage());
                    assert_eq!(actual.lineage().parents, vec![baseline_parent.output()]);
                    assert_eq!(actual.placement() == &StoragePlacement::Resident, !open);
                    let typed = owner
                        .storage()
                        .typed_handle::<ModuleSummary>(actual, actual.output_kind())
                        .unwrap();
                    assert_eq!(*owner.storage().get(&typed).unwrap(), expected_summary);
                    assert_eq!(
                        owner.storage().side_tables(&typed).unwrap(),
                        baseline_owner.storage().side_tables(&expected).unwrap()
                    );
                    let parent = run
                        .phase_results
                        .values()
                        .flatten()
                        .flat_map(|row| &row.output_refs)
                        .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                        .unwrap();
                    assert_eq!(parent, baseline_parent);
                    let typed_parent = owner
                        .storage()
                        .typed_handle::<(ResolvedAst, SymbolEnv)>(parent, parent.output_kind())
                        .unwrap();
                    assert_eq!(*owner.storage().get(&typed_parent).unwrap(), expected_pair);
                    let later = graph
                        .tasks()
                        .iter()
                        .filter(|candidate| candidate.dependencies.contains(&task.id))
                        .collect::<Vec<_>>();
                    assert!(!later.is_empty());
                    for dependent in later {
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
                            .flat_map(|row| &row.output_refs)
                            .map(|output| output.phase().as_str())
                            .collect::<std::collections::BTreeSet<_>>(),
                        [
                            "SourceLoad",
                            "Frontend",
                            "ModuleResolve",
                            "SignatureCollection"
                        ]
                        .into_iter()
                        .collect()
                    );
                    let cache = driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::SignatureCollection, input)
                        .unwrap();
                    assert!(
                        matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                    );
                    assert!(result.cache_observation.is_some());
                    if let Some((previous_result, previous_cache)) = &previous {
                        assert_eq!(result, previous_result);
                        assert_eq!(&cache, previous_cache);
                    }
                    previous = Some((result.clone(), cache));
                    owner
                        .validate_published_output(input.snapshot, actual)
                        .unwrap();
                    assert_eq!(
                        owner
                            .validate_current_output(input.snapshot, actual)
                            .is_ok(),
                        !open
                    );
                    assert_eq!(submission.session.captured(), &captured);
                }
                assert_eq!(session.captured(), &captured);
            },
        );
    }
}

#[test]
fn scheduled_theorem_signature_preserves_resource_refusals() {
    let text = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for case in [
        "cancel",
        "foreign-token",
        "no-publisher",
        "no-inputs",
        "obsolete",
        "denied",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, input, owner, _, _| {
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SignatureCollection"),
                    OutputKind::new("SourceTheoremModuleSummary"),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            if case == "obsolete" {
                owner.mark_obsolete(input.snapshot).unwrap();
            }
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_theorem_signature_collection();
            let registry = builder.build().unwrap();
            let query = registry
                .execute_phase_with_resources(
                    PipelinePhase::SignatureCollection,
                    input.clone(),
                    PhaseExecutionResources {
                        cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                            CancellationToken {
                                snapshot: if case == "foreign-token" {
                                    snapshot_id(0xf4)
                                } else {
                                    input.snapshot
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
                query.result.status,
                if case == "cancel" {
                    PhaseStatus::Cancelled
                } else {
                    PhaseStatus::Blocking
                },
                "{case}"
            );
            assert!(query.result.output_refs.is_empty(), "{case}");
            assert!(query.result.diagnostics.is_empty(), "{case}");
            assert_eq!(query.result.cache_observation, Some(query.observation));
        });
    }
}
