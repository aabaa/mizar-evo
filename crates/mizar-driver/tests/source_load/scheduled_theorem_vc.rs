use super::*;

#[test]
fn scheduled_theorem_vc_generation_matches_genuine_publication() {
    use mizar_core::{
        control_flow::ObligationHandoffOrigin,
        core_ir::{CoreProofStatus, ObligationSeedStatus},
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, SeedOriginRef, VcSchemaVersion, VcStatus};
    for (generation_text, schema_text) in [("α\n\0", ""), ("β\t\0", ""), ("α\n\0", "核\r")] {
        let generation = GenerationSchemaVersion::new(generation_text);
        let schema = VcSchemaVersion::new(schema_text);
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
                        ("VcGenerate", "SourceTheoremVC"),
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
                    let preparation_parent = inputs
                        .publish_native_theorem_preparation(&preparation_input, baseline)
                        .unwrap()
                        .unwrap();
                    let preparation = baseline.storage().get(&preparation_parent).unwrap();
                    let vc_input = child(vec![core_parent.erase(), preparation_parent.erase()]);
                    let expected = inputs
                        .publish_native_theorem_vcs_with_core_preparation_parents(
                            &vc_input,
                            baseline,
                            &generation,
                            &schema,
                        )
                        .unwrap()
                        .unwrap();
                    let vcs = baseline.storage().get(&expected).unwrap();
                    assert_eq!(vcs.source(), inputs.snapshot.source_versions[0].source_id);
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.schema_version(), &schema);
                    assert!(!vcs.vcs().is_empty() && !vcs.seed_accounting().is_empty());
                    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open
                        && vc.anchor.generation_schema_version == generation));
                    let mut observations = Vec::new();
                    for (g, v, v_len) in [("α\n\0", "", 0), ("β\t\0", "", 0), ("α\n\0", "核\r", 4)]
                    {
                        let mut builder = PhaseRegistryBuilder::new();
                        builder.register_source_theorem_vc_generation(
                            GenerationSchemaVersion::new(g),
                            VcSchemaVersion::new(v),
                        );
                        let registry = builder.build().unwrap();
                        let descriptor = registry
                            .descriptor_for_phase(PipelinePhase::VcGenerate)
                            .unwrap();
                        assert_eq!(
                            descriptor.schema_version,
                            format!(
                                "mizar-vc/source-theorem-vc-publication/v1;generation=4:{g};vc={v_len}:{v}"
                            )
                        );
                        observations.push(
                            registry
                                .cache_key_for_phase(PipelinePhase::VcGenerate, &vc_input)
                                .unwrap()
                                .observation,
                        );
                    }
                    assert!(observations.iter().enumerate().all(|(i, value)| {
                        observations[..i].iter().all(|earlier| earlier != value)
                    }));
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
                        core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    assert!(!core.obligation_seeds().is_empty());
                    assert!(
                        core.obligation_seeds()
                            .iter()
                            .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                    );
                    for row in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!("Core seed")
                        };
                        assert_eq!(
                            &preparation.1.entries.get(row.handoff).unwrap().seed,
                            core.obligation_seeds().get(seed).unwrap()
                        );
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
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
                                generation.clone(),
                                schema.clone(),
                            );
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
                                .any(|gap| gap.phase == PipelinePhase::ArtifactCommit)
                        );
                        assert!(!submission.missing_services.iter().any(|gap| matches!(
                            gap.phase,
                            PipelinePhase::SignatureCollection
                                | PipelinePhase::TypeChecking
                                | PipelinePhase::RegistrationResolution
                                | PipelinePhase::OverloadResolution
                                | PipelinePhase::Elaboration
                                | PipelinePhase::AlgorithmPreparation
                                | PipelinePhase::VcGenerate
                        )));
                        let graph = submission.task_graph.as_ref().unwrap();
                        let task = graph
                            .tasks()
                            .iter()
                            .find(|task| {
                                task.kind == TaskKind::CheckAndElaborate
                                    && task.unit == old.work_unit
                            })
                            .unwrap();
                        let run = submission.scheduler_run.as_ref().unwrap();
                        let vc = graph
                            .tasks()
                            .iter()
                            .find(|candidate| {
                                candidate.kind == TaskKind::VcGenerate
                                    && candidate.unit == old.work_unit
                            })
                            .unwrap();
                        assert!(vc.dependencies.contains(&task.id));
                        let [vc_result] = run.phase_results[&vc.id].as_slice() else {
                            panic!("one real separate VC row")
                        };
                        assert_eq!(vc_result.status, PhaseStatus::Complete);
                        assert!(vc_result.diagnostics.is_empty());
                        assert_eq!(vc_result.output_refs.len(), 1);

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
                        let actual_preparation = &prep_result.output_refs[0];
                        let actual = &vc_result.output_refs[0];
                        assert_eq!(actual_sig, &signature.erase());
                        assert_eq!(actual_type, &typed.erase());
                        assert_eq!(actual_reg, &registration.erase());
                        assert_eq!(actual_metadata, &metadata_parent.erase());
                        assert_eq!(actual_core, &core_parent.erase());
                        assert_eq!(actual_preparation, &preparation_parent.erase());
                        assert_eq!(actual, &expected.erase());
                        assert_eq!(actual_type.lineage().parents, vec![actual_sig.output()]);
                        assert_eq!(actual_reg.lineage().parents, vec![actual_type.output()]);
                        assert_eq!(actual_metadata.lineage().parents.len(), 2);
                        assert!(
                            actual_metadata
                                .lineage()
                                .parents
                                .contains(&actual_type.output())
                                && actual_metadata
                                    .lineage()
                                    .parents
                                    .contains(&actual_reg.output())
                        );
                        assert_eq!(
                            actual_core.lineage().parents,
                            vec![actual_metadata.output()]
                        );
                        assert_eq!(
                            actual_preparation.lineage().parents,
                            vec![actual_core.output()]
                        );
                        assert_eq!(actual.lineage().parents.len(), 2);
                        assert!(
                            actual.lineage().parents.contains(&actual_core.output())
                                && actual
                                    .lineage()
                                    .parents
                                    .contains(&actual_preparation.output())
                        );
                        let actual_handle = owner
                            .storage()
                            .typed_handle::<mizar_vc::vc_ir::VcSet>(actual, actual.output_kind())
                            .unwrap();
                        assert_eq!(*owner.storage().get(&actual_handle).unwrap(), *vcs);
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
                        let phase = PipelinePhase::VcGenerate;
                        let parents = [actual_core, actual_preparation];
                        let mut hashes = parents
                            .iter()
                            .map(|parent| parent.output().hash())
                            .collect::<Vec<_>>();
                        hashes.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
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
                        assert_eq!(vc_result.cache_observation, Some(probe.observation));
                        let cache = driver
                            .registry()
                            .cache_key_for_phase(phase, &derived)
                            .unwrap();
                        assert!(
                            matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
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
                        assert_eq!(vc.phases, [PipelinePhase::VcGenerate]);
                        assert_eq!(
                            run.task_states
                                .iter()
                                .find(|row| row.task_id == vc.id)
                                .unwrap()
                                .state,
                            TaskState::Completed
                        );
                        let artifact = graph
                            .tasks()
                            .iter()
                            .find(|candidate| {
                                candidate.kind == TaskKind::ArtifactCommit
                                    && candidate.unit == old.work_unit
                            })
                            .unwrap();
                        assert!(artifact.dependencies.contains(&vc.id));
                        assert!(
                            !graph
                                .tasks()
                                .iter()
                                .any(|candidate| candidate.kind == TaskKind::VcDischarge)
                        );
                        assert!(!run.phase_results.contains_key(&artifact.id));
                        let dependents = graph
                            .tasks()
                            .iter()
                            .filter(|candidate| candidate.dependencies.contains(&vc.id))
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
                                "AlgorithmPreparation",
                                "VcGenerate"
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
                            assert_eq!(
                                (
                                    run.phase_results[&task.id].clone(),
                                    run.phase_results[&vc.id].clone()
                                ),
                                *previous
                            );
                        }
                        previous = Some((
                            run.phase_results[&task.id].clone(),
                            run.phase_results[&vc.id].clone(),
                        ));
                        assert_eq!(submission.session.captured(), &captured);
                    }
                    assert_eq!(session.captured(), &captured);
                },
            );
        }
    }
}
#[test]
fn scheduled_theorem_vc_generation_preserves_resource_refusals() {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let generation = GenerationSchemaVersion::new("α\n\0");
    let schema = VcSchemaVersion::new("");
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "cancel",
        "foreign-token",
        "no-publisher",
        "no-inputs",
        "obsolete",
        "collected",
        "denied",
        "snapshot",
        "unit",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            let unit = IrWorkUnit::new("\"alpha\":\"main\"");
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
            let preparation_input = child(vec![core.erase()]);
            let preparation = inputs
                .publish_native_theorem_preparation(&preparation_input, owner)
                .unwrap()
                .unwrap();
            let input = child(vec![core.erase(), preparation.erase()]);
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("VcGenerate"),
                    OutputKind::new("SourceTheoremVC"),
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
                    9
                );
            }
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_theorem_vc_generation(generation.clone(), schema.clone());
            let registry = builder.build().unwrap();
            let descriptor = registry
                .descriptor_for_phase(PipelinePhase::VcGenerate)
                .unwrap();
            assert_eq!(descriptor.service_name, "SourceTheoremVcGeneration");
            assert_eq!(
                descriptor.owner,
                mizar_driver::registry::PhaseOwner::MizarVc
            );
            assert_eq!(descriptor.phases, [PipelinePhase::VcGenerate]);
            assert_eq!(descriptor.output_kind, "SourceTheoremVC");
            let input = if case == "snapshot" {
                PhaseInput::new(
                    input.work_unit.clone(),
                    PhaseDispatchInputBundle::without_parent_outputs(
                        snapshot_id(0xa9),
                        input.identities().input_hash(),
                        vec![],
                    ),
                )
            } else if case == "unit" {
                PhaseInput::new(
                    WorkUnit::Module {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new("other"),
                        ),
                    },
                    PhaseDispatchInputBundle::new(
                        input.snapshot,
                        input.identities().input_hash(),
                        input.identities().dependency_hashes().to_vec(),
                        input.parent_outputs().to_vec(),
                    )
                    .unwrap(),
                )
            } else {
                input
            };
            let result = registry
                .execute_phase_with_resources(
                    PipelinePhase::VcGenerate,
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
                if case == "collected" { 0 } else { 9 }
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
                    ("VcGenerate", "SourceTheoremVC"),
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
                    .register_source_theorem_vc_generation(generation.clone(), schema.clone());
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
                            "Elaboration" | "AlgorithmPreparation" | "VcGenerate"
                        ))
                );
            },
        );
    }
}

#[test]
fn scheduled_theorem_vc_descriptors_bind_genuine_sets_and_preserve_missing_proof_services() {
    use mizar_build::task_graph::{
        BackendProfileId, DependencyCoverage, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor,
        VcTaskDescriptorId,
    };
    use mizar_core::{
        control_flow::{ControlFlowOutput, ObligationSeedHandoff},
        core_ir::{CoreIr, CoreProofStatus, ObligationSeedStatus},
    };
    use mizar_vc::vc_ir::{
        GenerationSchemaVersion, SeedOriginRef, VcId, VcSchemaVersion, VcSet, VcStatus,
    };

    let generation = GenerationSchemaVersion::new("source-theorem-local-v1");
    let schema = VcSchemaVersion::new("vc-v1");
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        with_native_summary_parent(
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            open,
            threshold,
            |fixture, inputs, old, _, _, (session, ids, snapshots)| {
                let captured = session.captured().clone();
                let source = &captured.snapshot().source_versions[0];
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!("module")
                };
                let mut baseline = None::<(_, _, _, VcId)>;

                for configuration in 0..3 {
                    let selected = baseline.as_ref().map(|value| value.3);
                    let descriptor_id = VcTaskDescriptorId::new("caller-explicit-step6c11");
                    let association = selected.map(|selected| (descriptor_id.clone(), selected));
                    let backends = if configuration == 2 {
                        vec![BackendProfileId::new("caller-backend-step6c11")]
                    } else {
                        Vec::new()
                    };
                    let evidence = if configuration == 2 {
                        vec![EvidenceCandidateId::new("caller-evidence-step6c11")]
                    } else {
                        Vec::new()
                    };
                    let descriptors = if configuration == 0 {
                        Vec::new()
                    } else {
                        let (id, selected) = association.as_ref().unwrap();
                        let order = u64::try_from(selected.index()).unwrap();
                        let descriptor = VcTaskDescriptor::new(
                            id.clone(),
                            module.clone(),
                            VcOrderKey::new(format!("{order:020}")),
                            backends.clone(),
                            evidence.clone(),
                        );
                        vec![descriptor]
                    };

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
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new(format!(
                                "{:?}:{:?}",
                                module.package.as_str(),
                                module.path.as_str()
                            )),
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
                        .register_source_theorem_vc_generation(generation.clone(), schema.clone());
                    let mut driver = CompilerDriver::new(builder.build().unwrap())
                        .with_output_publisher(owner.clone());
                    let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                    submit.vc_descriptors = descriptors;
                    submit.worker_count = if configuration == 1 { 4 } else { 1 };
                    submit.completion_order = if configuration == 1 {
                        CompletionOrder::Reverse
                    } else {
                        CompletionOrder::Canonical
                    };
                    let submission = driver
                        .submit_captured(session.clone(), ids, snapshots, submit)
                        .unwrap();
                    assert_eq!(submission.session.captured(), &captured);
                    assert_eq!(
                        submission.status,
                        DriverSubmissionStatus::BlockedByMissingPhaseServices
                    );
                    assert_eq!(
                        submission.session.state(),
                        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                    );
                    assert!(submission.dispatch_gap_phases.is_empty());
                    let expected_missing: &[PipelinePhase] = match configuration {
                        0 => &[PipelinePhase::ArtifactCommit],
                        1 => &[PipelinePhase::VcDischarge, PipelinePhase::ArtifactCommit],
                        _ => &[
                            PipelinePhase::VcDischarge,
                            PipelinePhase::AtpSolve,
                            PipelinePhase::BackendRun,
                            PipelinePhase::KernelCheck,
                            PipelinePhase::ArtifactCommit,
                        ],
                    };
                    assert!(
                        submission
                            .missing_services
                            .iter()
                            .map(|missing| missing.phase)
                            .eq(expected_missing.iter().copied())
                    );

                    let graph = submission.task_graph.as_ref().unwrap();
                    let run = submission.scheduler_run.as_ref().unwrap();
                    let task = |kind, unit: &WorkUnit| {
                        graph
                            .tasks()
                            .iter()
                            .find(|task| task.kind == kind && &task.unit == unit)
                            .unwrap()
                    };
                    let state = |task: &BuildTask| {
                        run.task_states
                            .iter()
                            .find(|record| record.task_id == task.id)
                            .unwrap()
                            .state
                    };
                    let group = task(TaskKind::CheckAndElaborate, &old.work_unit);
                    let vc_task = task(TaskKind::VcGenerate, &old.work_unit);
                    let artifact = task(TaskKind::ArtifactCommit, &old.work_unit);
                    for completed in [group, vc_task] {
                        assert_eq!(state(completed), TaskState::Completed);
                    }
                    assert_eq!(state(artifact), TaskState::Blocked);
                    assert!(!run.phase_results.contains_key(&artifact.id));
                    let group_rows = &run.phase_results[&group.id];
                    assert_eq!(group_rows.len(), 6);
                    assert!(group_rows.iter().all(
                        |row| row.status == PhaseStatus::Complete && row.output_refs.len() == 1
                    ));
                    let [vc_row] = run.phase_results[&vc_task.id].as_slice() else {
                        panic!("one separate VC row")
                    };
                    assert_eq!(
                        (vc_row.status, vc_row.output_refs.len()),
                        (PhaseStatus::Complete, 1)
                    );
                    let core_ref = &group_rows[4].output_refs[0];
                    let prep_ref = &group_rows[5].output_refs[0];
                    let vc_ref = &vc_row.output_refs[0];
                    let storage = owner.storage();
                    let core_handle = storage
                        .typed_handle::<CoreIr>(core_ref, &OutputKind::new("SourceTheoremCore"))
                        .unwrap();
                    let prep_handle = storage
                        .typed_handle::<(ControlFlowOutput, ObligationSeedHandoff)>(
                            prep_ref,
                            &OutputKind::new("SourceTheoremPreparation"),
                        )
                        .unwrap();
                    let vc_handle = storage
                        .typed_handle::<VcSet>(vc_ref, &OutputKind::new("SourceTheoremVC"))
                        .unwrap();
                    let core = storage.get(&core_handle).unwrap();
                    let prep = storage.get(&prep_handle).unwrap();
                    let vcs = storage.get(&vc_handle).unwrap();
                    assert_eq!(core.source_id(), source.source_id);
                    let resolved_module = ResolverModuleId::new(
                        source.package_id.clone(),
                        source.module_path.clone(),
                    );
                    assert_eq!(core.module_id(), &resolved_module);
                    assert!(!core.proofs().is_empty());
                    for (_, proof) in core.proofs().iter() {
                        assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
                    }
                    assert_eq!(vcs.source(), source.source_id);
                    assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(vcs.snapshot(), old.snapshot);
                    assert_eq!(vcs.schema_version(), &schema);
                    assert_eq!(vcs.vcs().len(), 1);
                    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open));
                    assert!(!vcs.seed_accounting().is_empty());
                    for row in vcs.seed_accounting() {
                        let SeedOriginRef::ExistingCore { seed } = row.origin else {
                            panic!("original Core seed")
                        };
                        let original = core.obligation_seeds().get(seed).unwrap();
                        assert_eq!(row.seed_status, ObligationSeedStatus::Active);
                        assert_eq!(original.status, ObligationSeedStatus::Active);
                        assert_eq!(&prep.1.entries.get(row.handoff).unwrap().seed, original);
                    }
                    for output in [core_ref, prep_ref, vc_ref] {
                        assert_eq!(output.placement() == &StoragePlacement::Resident, !open);
                    }

                    if configuration == 0 {
                        let direct = mizar_vc::generator::generate_source_theorem_vcs(
                            &core,
                            old.snapshot,
                            &generation,
                            &schema,
                        )
                        .unwrap();
                        assert_eq!(*vcs, direct);
                        let selected = vcs.vcs()[0].id;
                        let seal = |output: &AnyPhaseOutputRef| {
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                old.snapshot,
                                output.clone(),
                            )
                            .unwrap()
                        };
                        let input = PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                old.identities().dependency_hashes().to_vec(),
                                vec![seal(core_ref), seal(vc_ref)],
                            )
                            .unwrap(),
                        );
                        let policy = mizar_proof::policy::VerifierPolicy::default();
                        let checked = inputs
                            .native_theorem_reflexivity_with_core_vc_parents(
                                &input, &owner, selected, &policy,
                            )
                            .unwrap()
                            .unwrap();
                        assert_eq!(
                            (checked.0.status(), checked.1.class),
                            (
                                mizar_kernel::checker::KernelCheckStatus::Accepted,
                                mizar_proof::policy::CandidatePolicyClass::DischargedBuiltin
                            )
                        );
                        baseline =
                            Some(((*core).clone(), (*prep).clone(), (*vcs).clone(), selected));
                    } else {
                        let (expected_core, expected_prep, expected_vcs, _) =
                            baseline.as_ref().unwrap();
                        assert_eq!(&*core, expected_core);
                        assert_eq!(&*prep, expected_prep);
                        assert_eq!(&*vcs, expected_vcs);
                    }

                    let proof_count = graph
                        .tasks()
                        .iter()
                        .filter(|task| {
                            matches!(
                                task.kind,
                                TaskKind::VcDischarge
                                    | TaskKind::AtpSolve
                                    | TaskKind::BackendRun
                                    | TaskKind::KernelCheck
                            )
                        })
                        .count();
                    assert_eq!(proof_count, [0, 1, 4][configuration]);
                    if configuration > 0 {
                        let (id, _) = association.as_ref().unwrap();
                        let discharge_unit = WorkUnit::Vc {
                            module: module.clone(),
                            descriptor: id.clone(),
                        };
                        let discharge = task(TaskKind::VcDischarge, &discharge_unit);
                        assert_eq!(discharge.dependency_coverage, DependencyCoverage::Complete);
                        assert_eq!(state(discharge), TaskState::Blocked);
                        assert!(!run.phase_results.contains_key(&discharge.id));
                        let root = graph
                            .tasks()
                            .iter()
                            .find(|task| task.kind == TaskKind::PackageResolve)
                            .unwrap();
                        let nonroot = |dependent: &BuildTask| {
                            dependent
                                .dependencies
                                .iter()
                                .filter(|id| *id != &root.id)
                                .cloned()
                                .collect::<std::collections::BTreeSet<_>>()
                        };
                        assert_eq!(
                            nonroot(discharge),
                            [vc_task.id.clone()].into_iter().collect()
                        );
                        let mut expected_artifact = [vc_task.id.clone(), discharge.id.clone()]
                            .into_iter()
                            .collect::<std::collections::BTreeSet<_>>();
                        if configuration == 2 {
                            let atp = task(TaskKind::AtpSolve, &discharge_unit);
                            let backend_unit = WorkUnit::BackendAttempt {
                                module: module.clone(),
                                descriptor: id.clone(),
                                backend_profile: backends[0].clone(),
                            };
                            let backend = task(TaskKind::BackendRun, &backend_unit);
                            let kernel_unit = WorkUnit::EvidenceCandidate {
                                module: module.clone(),
                                descriptor: id.clone(),
                                evidence_candidate: evidence[0].clone(),
                            };
                            let kernel = task(TaskKind::KernelCheck, &kernel_unit);
                            for proof_task in [atp, backend, kernel] {
                                assert_eq!(
                                    proof_task.dependency_coverage,
                                    DependencyCoverage::Complete
                                );
                                assert_eq!(state(proof_task), TaskState::Blocked);
                                assert!(!run.phase_results.contains_key(&proof_task.id));
                                expected_artifact.insert(proof_task.id.clone());
                            }
                            assert_eq!(nonroot(atp), [discharge.id.clone()].into_iter().collect());
                            assert_eq!(nonroot(backend), [atp.id.clone()].into_iter().collect());
                            assert_eq!(nonroot(kernel), [backend.id.clone()].into_iter().collect());
                        }
                        assert_eq!(nonroot(artifact), expected_artifact);
                    }
                }
            },
        );
    }
}

#[test]
fn scheduled_theorem_vc_discharge_matches_genuine_publication() {
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_core::{
        control_flow::{ControlFlowOutput, ObligationSeedHandoff},
        core_ir::{CoreProofStatus, ObligationSeedStatus},
    };
    use mizar_vc::{
        discharge::{
            ComputationLimit, DischargeExplanationCategory, DischargeOutput, DischargePolicy,
        },
        vc_ir::{GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcSchemaVersion, VcStatus},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, _, old, _, _, (session, ids, snapshots)| {
                let WorkUnit::Module { module } = &old.work_unit else {
                    panic!("module")
                };
                let mut policy_products = Vec::new();
                for (key, max_steps) in [("", 0), ("α\n\0", 0), ("", u32::MAX)] {
                    let policy = DischargePolicy {
                        computation_limit: ComputationLimit {
                            policy: PolicyKey::new(key),
                            max_steps,
                        },
                    };
                    let baseline = publisher(old.snapshot, threshold, &old.work_unit);
                    allow_frontend(&baseline, &old.work_unit);
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
                        baseline.allow_work_unit(AllowedWorkUnit::new(
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
                        .register_source_theorem_vc_generation(generation.clone(), schema.clone());
                    let (_, submit) = fixture.submit_request(vec![], vec![]);
                    let baseline_submission = CompilerDriver::new(builder.build().unwrap())
                        .with_output_publisher(baseline.clone())
                        .submit_captured(session.clone(), ids, snapshots, submit)
                        .unwrap();
                    let (inputs, original) = native_disk_lookup_context(&baseline_submission, ids);
                    let outputs = baseline_submission
                        .scheduler_run
                        .as_ref()
                        .unwrap()
                        .phase_results
                        .values()
                        .flatten()
                        .flat_map(|row| &row.output_refs)
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
                    let input = PhaseInput::new(
                        original.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            original.identities().input_hash(),
                            vec![],
                            [*core_ref, *vc_ref]
                                .into_iter()
                                .map(|parent| {
                                    SealedParentOutputHandle::from_published_output(
                                        &baseline,
                                        old.snapshot,
                                        parent.clone(),
                                    )
                                    .unwrap()
                                })
                                .collect(),
                        )
                        .unwrap(),
                    );
                    let vc_handle = baseline
                        .storage()
                        .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, vc_ref.output_kind())
                        .unwrap();
                    let vcs = baseline.storage().get(&vc_handle).unwrap();
                    let selected = vcs.vcs()[0].id;
                    let expected = inputs
                        .publish_native_theorem_discharge(&input, &baseline, selected, &policy)
                        .unwrap()
                        .unwrap();
                    let expected_value = baseline.storage().get(&expected).unwrap();
                    let descriptor_id = VcTaskDescriptorId::new("opaque-discharge-label:09");
                    let vc_unit = WorkUnit::Vc {
                        module: module.clone(),
                        descriptor: descriptor_id.clone(),
                    };
                    let query_input = PhaseInput::new(
                        vc_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            input.identities().input_hash(),
                            vec![],
                            input.parent_outputs().to_vec(),
                        )
                        .unwrap(),
                    );
                    let mut query_builder = PhaseRegistryBuilder::new();
                    query_builder.register_source_theorem_vc_discharge(policy.clone());
                    let query_registry = query_builder.build().unwrap();
                    let descriptor = query_registry
                        .descriptor_for_phase(PipelinePhase::VcDischarge)
                        .unwrap();
                    assert_eq!(descriptor.service_name, "SourceTheoremVcDischarge");
                    assert_eq!(
                        descriptor.owner,
                        mizar_driver::registry::PhaseOwner::MizarVc
                    );
                    assert_eq!(descriptor.phases, [PipelinePhase::VcDischarge]);
                    assert_eq!(descriptor.output_kind, "SourceTheoremDischarge");
                    assert_eq!(
                        descriptor.schema_version,
                        format!(
                            "mizar-vc/source-theorem-discharge-publication/v1;computation_policy={}:{};max_steps={max_steps}",
                            key.len(),
                            key
                        )
                    );
                    let query = query_registry
                        .cache_key_for_phase(PipelinePhase::VcDischarge, &query_input)
                        .unwrap();
                    assert!(matches!(query.intent, PhaseCacheIntent::NoKey { .. }));
                    policy_products.push(((*expected_value).clone(), query.observation));
                    let mut prior = None;
                    for (workers, atp) in [(1, false), (4, false), (1, true)] {
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
                                expected.work_unit().clone(),
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
                                generation.clone(),
                                schema.clone(),
                            )
                            .register_source_theorem_vc_discharge(policy.clone());
                        let mut driver = CompilerDriver::new(builder.build().unwrap())
                            .with_output_publisher(owner.clone());
                        let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                        submit.worker_count = workers;
                        submit.completion_order = if workers == 4 {
                            CompletionOrder::Reverse
                        } else {
                            CompletionOrder::Canonical
                        };
                        submit.vc_descriptors = vec![VcTaskDescriptor::new(
                            descriptor_id.clone(),
                            module.clone(),
                            VcOrderKey::new("opaque-order"),
                            if atp {
                                vec![BackendProfileId::new("backend")]
                            } else {
                                vec![]
                            },
                            if atp {
                                vec![EvidenceCandidateId::new("evidence")]
                            } else {
                                vec![]
                            },
                        )];
                        let submission = driver
                            .submit_captured(session.clone(), ids, snapshots, submit)
                            .unwrap();
                        assert_eq!(submission.session.captured(), session.captured());
                        assert_eq!(
                            submission.session.state(),
                            BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                        );
                        assert!(submission.dispatch_gap_phases.is_empty());
                        let expected_missing: &[PipelinePhase] = if atp {
                            &[
                                PipelinePhase::AtpSolve,
                                PipelinePhase::BackendRun,
                                PipelinePhase::KernelCheck,
                                PipelinePhase::ArtifactCommit,
                            ]
                        } else {
                            &[PipelinePhase::ArtifactCommit]
                        };
                        assert!(
                            submission
                                .missing_services
                                .iter()
                                .map(|gap| gap.phase)
                                .eq(expected_missing.iter().copied())
                        );
                        let graph = submission.task_graph.as_ref().unwrap();
                        let run = submission.scheduler_run.as_ref().unwrap();
                        let group = graph
                            .tasks()
                            .iter()
                            .find(|task| task.kind == TaskKind::CheckAndElaborate)
                            .unwrap();
                        let vc_task = graph
                            .tasks()
                            .iter()
                            .find(|task| task.kind == TaskKind::VcGenerate)
                            .unwrap();
                        let discharge = graph
                            .tasks()
                            .iter()
                            .find(|task| task.kind == TaskKind::VcDischarge)
                            .unwrap();
                        assert_eq!(discharge.unit, vc_unit);
                        assert_eq!(discharge.phases, [PipelinePhase::VcDischarge]);
                        assert!(discharge.dependencies.contains(&vc_task.id));
                        assert!(vc_task.dependencies.contains(&group.id));
                        for task in [group, vc_task, discharge] {
                            assert_eq!(
                                run.task_states
                                    .iter()
                                    .find(|row| row.task_id == task.id)
                                    .unwrap()
                                    .state,
                                TaskState::Completed
                            );
                        }
                        for task in graph.tasks().iter().filter(|task| {
                            matches!(
                                task.kind,
                                TaskKind::AtpSolve
                                    | TaskKind::BackendRun
                                    | TaskKind::KernelCheck
                                    | TaskKind::ArtifactCommit
                            )
                        }) {
                            assert_eq!(
                                run.task_states
                                    .iter()
                                    .find(|row| row.task_id == task.id)
                                    .unwrap()
                                    .state,
                                TaskState::Blocked
                            );
                            assert!(!run.phase_results.contains_key(&task.id));
                        }
                        let group_rows = &run.phase_results[&group.id];
                        assert_eq!(group_rows.len(), 6);
                        let [vc_row] = run.phase_results[&vc_task.id].as_slice() else {
                            panic!("one VC row")
                        };
                        let [row] = run.phase_results[&discharge.id].as_slice() else {
                            panic!("one discharge row")
                        };
                        for row in group_rows.iter().chain([vc_row, row]) {
                            assert_eq!(row.status, PhaseStatus::Complete);
                            assert!(row.diagnostics.is_empty());
                            assert_eq!(row.output_refs.len(), 1);
                        }
                        let actual_core = &group_rows[4].output_refs[0];
                        let actual_prep = &group_rows[5].output_refs[0];
                        let actual_vc = &vc_row.output_refs[0];
                        let actual = &row.output_refs[0];
                        let typed = owner
                            .storage()
                            .typed_handle::<DischargeOutput>(actual, actual.output_kind())
                            .unwrap();
                        assert_eq!(*owner.storage().get(&typed).unwrap(), *expected_value);
                        assert_eq!(actual.phase(), &IrPipelinePhase::new("VcDischarge"));
                        assert_eq!(actual.snapshot(), old.snapshot);
                        assert_eq!(
                            actual.output_kind(),
                            &OutputKind::new("SourceTheoremDischarge")
                        );
                        assert_eq!(
                            actual.schema_version(),
                            mizar_ir::storage::SchemaVersion::new(1)
                        );
                        assert_eq!(actual.work_unit(), expected.work_unit());
                        assert_eq!(actual.content_hash(), expected.content_hash());
                        assert_eq!(actual.side_table_hash(), expected.side_table_hash());
                        assert_eq!(actual.placement(), expected.placement());
                        assert_eq!(
                            *owner.storage().side_tables(&typed).unwrap(),
                            *baseline.storage().side_tables(&expected).unwrap()
                        );
                        assert_eq!(actual.lineage().parents.len(), 2);
                        assert!(
                            actual.lineage().parents.contains(&actual_core.output())
                                && actual.lineage().parents.contains(&actual_vc.output())
                        );
                        assert_eq!(
                            actual.lineage().named_input_hashes,
                            actual_core.lineage().named_input_hashes
                        );
                        let core = owner
                            .storage()
                            .get(
                                &owner
                                    .storage()
                                    .typed_handle::<mizar_core::core_ir::CoreIr>(
                                        actual_core,
                                        actual_core.output_kind(),
                                    )
                                    .unwrap(),
                            )
                            .unwrap();
                        let prep = owner
                            .storage()
                            .get(
                                &owner
                                    .storage()
                                    .typed_handle::<(ControlFlowOutput, ObligationSeedHandoff)>(
                                        actual_prep,
                                        actual_prep.output_kind(),
                                    )
                                    .unwrap(),
                            )
                            .unwrap();
                        let vcs = owner
                            .storage()
                            .get(
                                &owner
                                    .storage()
                                    .typed_handle::<mizar_vc::vc_ir::VcSet>(
                                        actual_vc,
                                        actual_vc.output_kind(),
                                    )
                                    .unwrap(),
                            )
                            .unwrap();
                        assert_eq!(
                            core.source_id(),
                            inputs.snapshot.source_versions[0].source_id
                        );
                        assert_eq!(
                            core.module_id(),
                            &ResolverModuleId::new(
                                PackageId::new("alpha"),
                                ModulePath::new("main")
                            )
                        );
                        assert_eq!(vcs.source(), core.source_id());
                        assert_eq!(vcs.vcs().len(), 1);
                        assert_eq!(vcs.module().as_str(), "package=5:alpha;module=4:main");
                        assert_eq!(vcs.snapshot(), old.snapshot);
                        assert_eq!(vcs.schema_version(), &schema);
                        assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
                        assert!(core.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        ));
                        assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open));
                        for seed in vcs.seed_accounting() {
                            let SeedOriginRef::ExistingCore { seed: id } = seed.origin else {
                                panic!("Core seed")
                            };
                            let original = core.obligation_seeds().get(id).unwrap();
                            assert_eq!(seed.seed_status, ObligationSeedStatus::Active);
                            assert_eq!(original.status, ObligationSeedStatus::Active);
                            assert_eq!(&prep.1.entries.get(seed.handoff).unwrap().seed, original);
                        }
                        assert!(
                            expected_value
                                .vc_set()
                                .vcs()
                                .iter()
                                .all(|vc| vc.status == VcStatus::NeedsAtp
                                    && vc.provenance.last().unwrap().phase
                                        == mizar_vc::vc_ir::VcProvenancePhase::Discharge)
                        );
                        assert_eq!(
                            expected_value.explanations()[0].category,
                            DischargeExplanationCategory::UnsupportedRule
                        );
                        assert_eq!(expected_value.explanations()[0].rule, None);
                        assert!(expected_value.evidence_records().is_empty());
                        let derived = PhaseInput::new(
                            discharge.unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                source_key(&inputs.snapshot.source_versions[0]),
                                vec![],
                                [actual_core, actual_vc]
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
                        let mut hashes = [actual_core.output().hash(), actual_vc.output().hash()];
                        hashes.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                        assert_eq!(derived.identities().parent_output_hashes(), hashes);
                        let probe = driver
                            .registry()
                            .execute_phase_with_resources(
                                PipelinePhase::VcDischarge,
                                derived.clone(),
                                PhaseExecutionResources::default(),
                            )
                            .unwrap();
                        assert_eq!(row.cache_observation, Some(probe.observation));
                        let cache = driver
                            .registry()
                            .cache_key_for_phase(PipelinePhase::VcDischarge, &derived)
                            .unwrap();
                        assert_eq!(cache.observation, query.observation);
                        assert!(matches!(cache.intent, PhaseCacheIntent::NoKey { .. }));
                        let module_probe = driver
                            .registry()
                            .execute_phase_with_resources(
                                PipelinePhase::VcDischarge,
                                PhaseInput::new(
                                    old.work_unit.clone(),
                                    PhaseDispatchInputBundle::new(
                                        old.snapshot,
                                        derived.identities().input_hash(),
                                        vec![],
                                        derived.parent_outputs().to_vec(),
                                    )
                                    .unwrap(),
                                ),
                                PhaseExecutionResources::default(),
                            )
                            .unwrap();
                        assert_ne!(module_probe.observation, probe.observation);
                        if !atp {
                            let rows = (group_rows.clone(), vc_row.clone(), row.clone());
                            if let Some(prior) = &prior {
                                assert_eq!(&rows, prior);
                            }
                            prior = Some(rows);
                        }
                    }
                }
                assert!(
                    policy_products
                        .iter()
                        .all(|value| value.0 == policy_products[0].0)
                );
                for (index, value) in policy_products.iter().enumerate() {
                    assert!(
                        policy_products[..index]
                            .iter()
                            .all(|prior| prior.1 != value.1)
                    );
                }
            },
        );
    }
}

#[test]
fn scheduled_theorem_vc_discharge_preserves_resource_refusals() {
    use mizar_ir::storage::CollectInput;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "cancel",
        "foreign-token",
        "no-publisher",
        "no-inputs",
        "obsolete",
        "collected",
        "denied",
        "snapshot",
        "unit",
        "wrong-vc-module",
        "private",
        "lemma",
    ] {
        let text = match case {
            "private" => include_str!(
                "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
            ),
            "lemma" => include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            ),
            _ => text,
        };
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
                if case != "denied" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("VcDischarge"),
                        OutputKind::new("SourceTheoremDischarge"),
                        unit.clone(),
                    ));
                }
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("α\n\0"),
                        max_steps: 0,
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
                    .register_source_theorem_vc_generation(
                        GenerationSchemaVersion::new("λ\0"),
                        VcSchemaVersion::new(""),
                    );
                let earlier_refusal = matches!(case, "private" | "lemma");
                if earlier_refusal {
                    builder.register_source_theorem_vc_discharge(policy.clone());
                }
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                if earlier_refusal {
                    submit.vc_descriptors = vec![mizar_build::task_graph::VcTaskDescriptor::new(
                        mizar_build::task_graph::VcTaskDescriptorId::new("opaque-refused-source"),
                        mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new("main"),
                        ),
                        mizar_build::task_graph::VcOrderKey::new("opaque-order"),
                        vec![],
                        vec![],
                    )];
                }
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                if earlier_refusal {
                    let graph = submission.task_graph.as_ref().unwrap();
                    let run = submission.scheduler_run.as_ref().unwrap();
                    let group = graph
                        .tasks()
                        .iter()
                        .find(|task| task.kind == TaskKind::CheckAndElaborate)
                        .unwrap();
                    let rows = &run.phase_results[&group.id];
                    assert_eq!(rows.len(), 5);
                    assert_eq!(group.phases[rows.len() - 1], PipelinePhase::Elaboration);
                    let (blocked, completed) = rows.split_last().unwrap();
                    assert!(
                        completed
                            .iter()
                            .all(|row| row.status == PhaseStatus::Complete
                                && row.diagnostics.is_empty()
                                && row.output_refs.len() == 1)
                    );
                    assert_eq!(blocked.status, PhaseStatus::Blocking);
                    assert!(blocked.diagnostics.is_empty() && blocked.output_refs.is_empty());
                    for task in graph.tasks().iter().filter(|task| {
                        matches!(
                            task.kind,
                            TaskKind::CheckAndElaborate
                                | TaskKind::VcGenerate
                                | TaskKind::VcDischarge
                        )
                    }) {
                        assert_eq!(
                            run.task_states
                                .iter()
                                .find(|row| row.task_id == task.id)
                                .unwrap()
                                .state,
                            TaskState::Blocked
                        );
                        if task.kind != TaskKind::CheckAndElaborate {
                            assert!(!run.phase_results.contains_key(&task.id));
                        }
                    }
                    assert!(
                        !run.phase_results
                            .values()
                            .flatten()
                            .flat_map(|row| &row.output_refs)
                            .any(|output| matches!(
                                output.phase().as_str(),
                                "VcGenerate" | "VcDischarge"
                            ))
                    );
                    assert_eq!(
                        submission.session.state(),
                        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                    );
                    return;
                }
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
                let selected = mizar_vc::vc_ir::VcId::new(0);
                assert!(
                    inputs
                        .native_theorem_discharge_with_core_vc_parents(
                            &input, &owner, selected, &policy
                        )
                        .unwrap()
                        .is_some()
                );
                if case == "unit" {
                    assert!(
                        inputs
                            .publish_native_theorem_discharge(&input, &owner, selected, &policy)
                            .unwrap()
                            .is_some()
                    );
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_theorem_vc_discharge(policy);
                let registry = builder.build().unwrap();
                let work_unit = if case == "unit" {
                    input.work_unit.clone()
                } else {
                    WorkUnit::Vc {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new(if case == "wrong-vc-module" {
                                "other"
                            } else {
                                "main"
                            }),
                        ),
                        descriptor: mizar_build::task_graph::VcTaskDescriptorId::new(
                            "opaque-control",
                        ),
                    }
                };
                let input = PhaseInput::new(
                    work_unit,
                    if case == "snapshot" {
                        PhaseDispatchInputBundle::without_parent_outputs(
                            snapshot_id(0xa9),
                            input.identities().input_hash(),
                            vec![],
                        )
                    } else {
                        PhaseDispatchInputBundle::new(
                            input.snapshot,
                            input.identities().input_hash(),
                            vec![],
                            input.parent_outputs().to_vec(),
                        )
                        .unwrap()
                    },
                );
                if case == "obsolete" {
                    owner.mark_obsolete(old.snapshot).unwrap();
                }
                if case == "collected" {
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: old.snapshot,
                                protected_outputs: vec![]
                            })
                            .outputs_dropped,
                        10
                    );
                }
                let result = registry
                    .execute_phase_with_resources(
                        PipelinePhase::VcDischarge,
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
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected" {
                        0
                    } else if case == "unit" {
                        11
                    } else {
                        10
                    }
                );
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(case == "denied")
                );
            },
        );
    }
}
