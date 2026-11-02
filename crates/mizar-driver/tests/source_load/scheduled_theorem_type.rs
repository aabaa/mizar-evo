use super::*;

#[test]
fn scheduled_theorem_type_checking_matches_genuine_publication() {
    use mizar_checker::typed_ast::{TypedAst, TypingState};
    use mizar_driver::native_theorem_typed_ast_storage_bytes as encode;
    use mizar_ir::storage::SchemaVersion;
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
                let expected = inputs
                    .publish_native_theorem_typed_ast(&type_input, baseline)
                    .unwrap()
                    .unwrap();
                let expected_typed = baseline.storage().get(&expected).unwrap();
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
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let bytes = encode(ast, &module, &expected_typed).unwrap().unwrap();
                assert_eq!(
                    expected_typed.source_id(),
                    inputs.snapshot.source_versions[0].source_id
                );
                assert_eq!(expected_typed.module_id(), &module);
                assert!(!expected_typed.types().is_empty());
                let declarations = expected_typed.source_context().unwrap().declarations();
                assert!(declarations.iter().next().is_some());
                for (_, declaration) in declarations.iter() {
                    let node = expected_typed
                        .nodes()
                        .node(declaration.site.node())
                        .unwrap();
                    assert_eq!(node.typing, TypingState::Successful);
                    assert!(node.links.type_entry.is_some());
                }
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
                        .register_source_theorem_type_checking();
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
                            .any(|gap| gap.phase == PipelinePhase::RegistrationResolution)
                    );
                    assert!(!submission.missing_services.iter().any(|gap| matches!(
                        gap.phase,
                        PipelinePhase::SignatureCollection | PipelinePhase::TypeChecking
                    )));
                    let graph = submission.task_graph.as_ref().unwrap();
                    let task = graph
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == TaskKind::CheckAndElaborate && task.unit == old.work_unit
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
                    let [sig_result, typed_result] = run.phase_results[&task.id].as_slice() else {
                        panic!("two real phases in the same blocked task")
                    };
                    for result in [sig_result, typed_result] {
                        assert_eq!(result.status, PhaseStatus::Complete);
                        assert!(result.diagnostics.is_empty());
                        assert_eq!(result.output_refs.len(), 1);
                    }
                    let actual_signature = &sig_result.output_refs[0];
                    let actual = &typed_result.output_refs[0];
                    assert_eq!(actual_signature, &signature.erase());
                    assert_eq!(actual, &expected.erase());
                    assert_eq!(actual.lineage(), expected.lineage());
                    assert_eq!(actual.snapshot(), old.snapshot);
                    assert_eq!(actual.phase(), &IrPipelinePhase::new("TypeChecking"));
                    assert_eq!(
                        actual.output_kind(),
                        &OutputKind::new("SourceTheoremTypedAst")
                    );
                    assert_eq!(actual.work_unit(), &unit);
                    assert_eq!(actual.schema_version(), SchemaVersion::new(1));
                    assert_eq!(actual.lineage().parents, vec![actual_signature.output()]);
                    let typed = owner
                        .storage()
                        .typed_handle::<TypedAst>(actual, actual.output_kind())
                        .unwrap();
                    assert_eq!(*owner.storage().get(&typed).unwrap(), *expected_typed);
                    assert_eq!(
                        owner.storage().side_tables(&typed).unwrap(),
                        baseline.storage().side_tables(&expected).unwrap()
                    );
                    assert_eq!(actual.content_hash(), expected.content_hash());
                    assert_eq!(actual.side_table_hash(), expected.side_table_hash());
                    assert_eq!(
                        encode(ast, &module, &owner.storage().get(&typed).unwrap()).unwrap(),
                        Some(bytes.clone())
                    );
                    assert_eq!(actual.placement() == &StoragePlacement::Resident, !open);
                    if let StoragePlacement::Blob { blob, len } = actual.placement() {
                        let StoragePlacement::Blob {
                            blob: expected_blob,
                            ..
                        } = expected.placement()
                        else {
                            panic!("blob baseline")
                        };
                        assert_eq!(blob, expected_blob);
                        assert_eq!(*len, bytes.len());
                    }
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
                                    actual_signature.clone(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    assert_eq!(
                        derived.identities().parent_output_hashes(),
                        &[actual_signature.output().hash()]
                    );
                    assert_ne!(
                        derived.identities().parent_output_hashes(),
                        old.identities().parent_output_hashes()
                    );
                    let probe = driver
                        .registry()
                        .execute_phase_with_resources(
                            PipelinePhase::TypeChecking,
                            derived.clone(),
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_eq!(typed_result.cache_observation, Some(probe.observation));
                    let wrong_probe = driver
                        .registry()
                        .execute_phase_with_resources(
                            PipelinePhase::TypeChecking,
                            old.clone(),
                            PhaseExecutionResources::default(),
                        )
                        .unwrap();
                    assert_ne!(probe.observation, wrong_probe.observation);
                    for (phase, input) in [
                        (PipelinePhase::SignatureCollection, old),
                        (PipelinePhase::TypeChecking, &derived),
                    ] {
                        let cache = driver.registry().cache_key_for_phase(phase, input).unwrap();
                        assert!(
                            matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                        );
                    }
                    owner
                        .validate_published_output(old.snapshot, actual)
                        .unwrap();
                    assert_eq!(
                        owner.validate_current_output(old.snapshot, actual).is_ok(),
                        !open
                    );
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
                            .flat_map(|row| &row.output_refs)
                            .map(|output| output.phase().as_str())
                            .collect::<std::collections::BTreeSet<_>>(),
                        [
                            "SourceLoad",
                            "Frontend",
                            "ModuleResolve",
                            "SignatureCollection",
                            "TypeChecking"
                        ]
                        .into_iter()
                        .collect()
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
fn scheduled_theorem_type_checking_preserves_resource_refusals() {
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let unsupported = private.replace("set", "object");
    for case in [
        "cancel",
        "foreign-token",
        "no-publisher",
        "no-inputs",
        "foreign-inputs",
        "obsolete",
        "collected",
        "denied",
        "unsupported-type",
    ] {
        with_native_summary_parent(
            if case == "unsupported-type" {
                &unsupported
            } else {
                lemma
            },
            false,
            0,
            |_, inputs, old, owner, _, _| {
                let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SignatureCollection"),
                    OutputKind::new("SourceTheoremModuleSummary"),
                    unit.clone(),
                ));
                let signature = inputs
                    .publish_native_theorem_module_summary(old, owner)
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
                                signature.erase(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                if case != "denied" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("TypeChecking"),
                        OutputKind::new("SourceTheoremTypedAst"),
                        unit,
                    ));
                }
                if case == "unsupported-type" {
                    let raw = inputs
                        .native_theorem_typed_ast_with_signature_parent(&input, owner)
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .publish_native_theorem_typed_ast(&input, owner)
                            .unwrap_err(),
                        raw
                    );
                } else if case == "obsolete" {
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
                        4
                    );
                }
                let mut foreign = inputs.snapshot.clone();
                foreign.id = snapshot_id(0xf7);
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_theorem_type_checking();
                let registry = builder.build().unwrap();
                let descriptor = registry
                    .descriptor_for_phase(PipelinePhase::TypeChecking)
                    .unwrap();
                assert_eq!(descriptor.service_name, "SourceTheoremTypeChecking");
                assert_eq!(
                    descriptor.owner,
                    mizar_driver::registry::PhaseOwner::MizarChecker
                );
                assert_eq!(descriptor.phases, [PipelinePhase::TypeChecking]);
                assert_eq!(
                    descriptor.schema_version,
                    "mizar-checker/source-theorem-typed-ast-publication/v1"
                );
                assert_eq!(descriptor.output_kind, "SourceTheoremTypedAst");
                assert!(matches!(
                    registry
                        .cache_key_for_phase(PipelinePhase::TypeChecking, &input)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey { .. }
                ));
                let result = registry
                    .execute_phase_with_resources(
                        PipelinePhase::TypeChecking,
                        input,
                        PhaseExecutionResources {
                            cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                                CancellationToken {
                                    snapshot: if case == "foreign-token" {
                                        snapshot_id(0xf6)
                                    } else {
                                        old.snapshot
                                    },
                                    generation: CancellationGeneration::new(1),
                                    reason: CancellationReason::ExplicitRequest,
                                },
                            ),
                            output_publisher: (case != "no-publisher").then(|| owner.clone()),
                            source_load: (case != "no-inputs").then_some(
                                if case == "foreign-inputs" {
                                    SourceLoadInputs {
                                        snapshot: &foreign,
                                        ..inputs
                                    }
                                } else {
                                    inputs
                                },
                            ),
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
                    if case == "collected" { 0 } else { 4 }
                );
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(case == "denied")
                );
            },
        );
    }
}
