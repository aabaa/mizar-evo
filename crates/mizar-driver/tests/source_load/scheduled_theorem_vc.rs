use super::*;
use mizar_ir::publisher::OutputAuthorization;

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
                            .submit_captured(
                                session.clone(),
                                ids,
                                snapshots,
                                submit,
                                Default::default(),
                            )
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
                            .validate_output(old.snapshot, actual, OutputAuthorization::Published)
                            .unwrap();
                        assert_eq!(
                            owner
                                .validate_output(
                                    old.snapshot,
                                    actual,
                                    OutputAuthorization::CurrentPackage
                                )
                                .is_ok(),
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
                    .submit_captured(session.clone(), ids, snapshots, submit, Default::default())
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
            |fixture, _, old, _, _, (session, ids, snapshots)| {
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
                        .submit_captured(
                            session.clone(),
                            ids,
                            snapshots,
                            submit,
                            Default::default(),
                        )
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
                        .submit_captured(
                            session.clone(),
                            ids,
                            snapshots,
                            submit,
                            Default::default(),
                        )
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
                            .submit_captured(
                                session.clone(),
                                ids,
                                snapshots,
                                submit,
                                Default::default(),
                            )
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
                    .submit_captured(session.clone(), ids, snapshots, submit, Default::default())
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

#[test]
fn scheduled_reflexivity_kernel_profile_is_exact_and_edit_clean() {
    use mizar_build::task_graph::{
        EvidenceCandidateId, TaskKind, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};

    let sources = [
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        "theorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
        "\n theorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end; \n",
    ];
    let fixture = Fixture::new(sources[0].as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    for (variant, text) in sources.into_iter().enumerate() {
        fixture.write(text.as_bytes());
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        draft.generation = BuildRequestGeneration::new(variant as u64);
        let source = SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("main"),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: "src/main.miz".into(),
            },
        };
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(DiskSourceLoader::new(fixture.root.join("alpha")), source)],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let mut runs = Vec::new();
        for (workers, order) in [
            (1, CompletionOrder::Canonical),
            (4, CompletionOrder::Reverse),
        ] {
            let owner = publisher(snapshot, 0, &unit);
            allow_frontend(&owner, &unit);
            for (phase, kind) in [
                ("ModuleResolve", "SourceTheoremModuleCandidates"),
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
                ("Elaboration", "SourceTheoremCore"),
                ("AlgorithmPreparation", "SourceTheoremPreparation"),
                ("VcGenerate", "SourceTheoremVC"),
                ("KernelCheck", "SourceTheoremReflexivityKernelStatus"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let verifier_policy = VerifierPolicy::release();
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
                    GenerationSchemaVersion::new("source-theorem-local-v1"),
                    VcSchemaVersion::new("vc-v1"),
                )
                .register_source_theorem_reflexivity_kernel_check(verifier_policy.clone());
            let registry = builder.build().unwrap();
            assert_eq!(registry.descriptors().len(), 11);
            let descriptor = registry
                .descriptor_for_phase(PipelinePhase::KernelCheck)
                .unwrap();
            assert_eq!(
                descriptor.service_name,
                "SourceTheoremReflexivityKernelCheck"
            );
            assert_eq!(
                descriptor.owner,
                mizar_driver::registry::PhaseOwner::MizarKernelProof
            );
            assert_eq!(descriptor.phases, [PipelinePhase::KernelCheck]);
            assert_eq!(
                descriptor.output_kind,
                "SourceTheoremReflexivityKernelStatus"
            );
            assert_eq!(
                descriptor.schema_version,
                format!(
                    "mizar-proof/source-theorem-reflexivity-kernel-status/v1;verifier_policy={}",
                    verifier_policy.policy_fingerprint().to_lower_hex()
                )
            );
            let WorkUnit::Module { module } = &unit else {
                panic!("module input")
            };
            let descriptor_id = VcTaskDescriptorId::new(format!("opaque-reflexivity-{variant}"));
            let evidence_id = EvidenceCandidateId::new(format!("opaque-evidence-{variant}"));
            let kernel_input = PhaseInput::new(
                WorkUnit::EvidenceCandidate {
                    module: module.clone(),
                    descriptor: descriptor_id.clone(),
                    evidence_candidate: evidence_id.clone(),
                },
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    session.captured().snapshot().source_versions[0].source_hash,
                    vec![],
                ),
            );
            assert!(matches!(
                registry
                    .cache_key_for_phase(PipelinePhase::KernelCheck, &kernel_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey { .. }
            ));
            let (_, mut submit) = fixture.submit_request(vec![], vec![]);
            submit.vc_descriptors = vec![VcTaskDescriptor::new(
                descriptor_id,
                module.clone(),
                VcOrderKey::new(format!("opaque-order-{variant}")),
                vec![],
                vec![evidence_id],
            )];
            submit.worker_count = workers;
            submit.completion_order = order;
            let submission = CompilerDriver::new(registry)
                .with_output_publisher(owner.clone())
                .submit_captured(
                    session.clone(),
                    &ids,
                    &snapshots,
                    submit,
                    Default::default(),
                )
                .unwrap();
            assert_eq!(submission.session.captured(), session.captured());
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
            assert_eq!(
                submission
                    .missing_services
                    .iter()
                    .map(|missing| missing.phase)
                    .collect::<Vec<_>>(),
                [PipelinePhase::VcDischarge, PipelinePhase::ArtifactCommit]
            );
            let graph = submission.task_graph.as_ref().unwrap();
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(
                !graph
                    .tasks()
                    .iter()
                    .any(|task| matches!(task.kind, TaskKind::AtpSolve | TaskKind::BackendRun))
            );
            let kernel_tasks = graph
                .tasks()
                .iter()
                .filter(|task| task.kind == TaskKind::KernelCheck)
                .collect::<Vec<_>>();
            let [kernel_task] = kernel_tasks.as_slice() else {
                panic!("one native KernelCheck per module")
            };
            assert_eq!(kernel_task.phases, [PipelinePhase::KernelCheck]);
            let WorkUnit::EvidenceCandidate { .. } = &kernel_task.unit else {
                panic!("native evidence candidate unit")
            };
            let parents = kernel_task
                .dependencies
                .iter()
                .filter_map(|dependency| graph.tasks().iter().find(|task| &task.id == dependency))
                .collect::<Vec<_>>();
            let [vc_task] = parents.as_slice() else {
                panic!("KernelCheck directly depends only on VcGenerate")
            };
            assert_eq!(vc_task.kind, TaskKind::VcGenerate);
            assert_eq!(vc_task.phases, [PipelinePhase::VcGenerate]);
            assert_eq!(vc_task.unit, unit);
            let state = run
                .task_states
                .iter()
                .find(|state| state.task_id == kernel_task.id)
                .unwrap()
                .state;
            assert_eq!(
                state,
                TaskState::Completed,
                "native KernelCheck should run with VcDischarge missing"
            );
            let discharge = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::VcDischarge)
                .unwrap();
            assert_eq!(
                run.task_states
                    .iter()
                    .find(|state| state.task_id == discharge.id)
                    .unwrap()
                    .state,
                TaskState::Blocked
            );
            assert!(!run.phase_results.contains_key(&discharge.id));
            let rows = run.phase_results.get(&kernel_task.id).unwrap();
            let [row] = rows.as_slice() else {
                panic!("one KernelCheck result")
            };
            assert_eq!(row.status, PhaseStatus::Complete);
            let [status_ref] = row.output_refs.as_slice() else {
                panic!("one sealed reflexivity status")
            };
            assert_eq!(
                status_ref.output_kind(),
                &OutputKind::new("SourceTheoremReflexivityKernelStatus")
            );
            assert_eq!(status_ref.lineage().parents.len(), 3);
            assert_eq!(status_ref.lineage().named_input_hashes.len(), 1);
            assert_eq!(status_ref.lineage().named_input_hashes[0].name, "source");
            let status = owner
                .storage()
                .typed_handle::<(
                    mizar_core::core_ir::CoreIr,
                    mizar_vc::vc_ir::VcSet,
                    mizar_kernel::checker::KernelCheckResult,
                    mizar_proof::policy::PolicyDecision,
                    mizar_proof::status::ProofStatusProjection,
                    mizar_vc::dependency_slice::DependencySlice,
                )>(status_ref, status_ref.output_kind())
                .unwrap();
            runs.push((
                (*owner.storage().get(&status).unwrap()).clone(),
                status_ref.clone(),
            ));
            assert_eq!(
                graph
                    .tasks()
                    .iter()
                    .filter(|task| task.kind == TaskKind::ArtifactCommit)
                    .count(),
                1
            );
        }
        assert_eq!(
            runs[0].0, runs[1].0,
            "variant {variant}: independent clean stores agree across worker order"
        );
        assert_eq!(runs[0].1.output_kind(), runs[1].1.output_kind());
    }
}

fn reflexivity_artifact_registry(
    producer_policy: mizar_proof::policy::VerifierPolicy,
    consumer_policy: mizar_proof::policy::VerifierPolicy,
    output_parent: &str,
    include_discharge: bool,
) -> mizar_driver::registry::PhaseRegistry {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};

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
            GenerationSchemaVersion::new("source-theorem-local-v1"),
            VcSchemaVersion::new("vc-v1"),
        )
        .register_source_theorem_reflexivity_kernel_check(producer_policy);
    if include_discharge {
        builder
            .register_source_theorem_vc_discharge(mizar_vc::discharge::DischargePolicy::default());
    }
    builder
        .register_source_theorem_reflexivity_artifact_commit(
            consumer_policy,
            (
                "proof-witnesses".to_owned(),
                "main.mizir.json".to_owned(),
                output_parent.to_owned(),
            ),
        )
        .unwrap();
    builder.build().unwrap()
}

fn allow_reflexivity_artifact_outputs(
    owner: &PhaseOutputPublisher,
    unit: &WorkUnit,
    include_discharge: bool,
) {
    allow_frontend(owner, unit);
    for (phase, kind) in [
        ("ModuleResolve", "SourceTheoremModuleCandidates"),
        ("SignatureCollection", "SourceTheoremModuleSummary"),
        ("TypeChecking", "SourceTheoremTypedAst"),
        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
        ("OverloadResolution", "SourceTheoremMetadata"),
        ("Elaboration", "SourceTheoremCore"),
        ("AlgorithmPreparation", "SourceTheoremPreparation"),
        ("VcGenerate", "SourceTheoremVC"),
        ("KernelCheck", "SourceTheoremReflexivityKernelStatus"),
        ("ArtifactCommit", "SourceTheoremArtifactData"),
    ] {
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new(phase),
            OutputKind::new(kind),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
    }
    if include_discharge {
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("VcDischarge"),
            OutputKind::new("SourceTheoremDischarge"),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
    }
}

#[test]
fn scheduled_reflexivity_artifact_consumer_commits_genuine_outputs() {
    use mizar_build::task_graph::{
        EvidenceCandidateId, TaskKind, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_proof::policy::VerifierPolicy;

    for (text, open, wrong_policy, expected_commit) in [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            false,
            false,
            true,
        ),
        (
            "theorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            false,
            false,
            true,
        ),
        (
            "\n theorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end; \n",
            false,
            false,
            true,
        ),
        (
            "private theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            false,
            false,
            false,
        ),
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            true,
            false,
            true,
        ),
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            false,
            true,
            false,
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut request, mut submit) = fixture.submit_request(vec![], vec![]);
        let plan = mizar_build::planner::produce_build_plan(
            submit.plan_request.clone(),
            submit.workspace_packages.clone(),
            submit.lockfile.clone(),
        )
        .unwrap();
        request.dependency_inputs.lockfile_hash =
            mizar_build::planner::lockfile_content_hash(&plan.lockfile);
        request.dependency_inputs.toolchain = plan.toolchain.clone();
        request.verifier_config = mizar_driver::request::VerifierConfigInput::new(
            mizar_build::planner::workspace_config_content_hash(
                &plan.verifier_config,
                &plan.build_config,
            ),
        );
        request.source_inputs.versions.clear();
        let source = mizar_session::SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("main"),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: Edition::new("2025"),
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
        let session = request
            .capture_sources(
                &ids,
                &snapshots,
                vec![(DiskSourceLoader::new(fixture.root.join("alpha")), source)],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        fs::create_dir_all(fixture.root.join("alpha/reflexivity-artifacts")).unwrap();
        let owner = publisher(snapshot, 0, &unit);
        allow_reflexivity_artifact_outputs(&owner, &unit, false);
        let policy = VerifierPolicy::release();
        let consumer_policy = if wrong_policy {
            VerifierPolicy::development()
        } else {
            policy.clone()
        };
        let registry = reflexivity_artifact_registry(
            policy.clone(),
            consumer_policy,
            "reflexivity-artifacts",
            false,
        );
        assert_eq!(registry.descriptors().len(), 12);
        let descriptor = registry
            .descriptor_for_phase(PipelinePhase::ArtifactCommit)
            .unwrap();
        assert_eq!(
            descriptor.service_name,
            "SourceTheoremReflexivityArtifactCommit"
        );
        assert_eq!(
            descriptor.owner,
            mizar_driver::registry::PhaseOwner::MizarArtifact
        );
        assert_eq!(descriptor.output_kind, "SourceTheoremArtifactData");
        assert!(
            descriptor
                .schema_version
                .contains("witness_path_prefix=15:proof-witnesses")
        );
        assert!(
            descriptor
                .schema_version
                .contains("main_path=15:main.mizir.json")
        );
        assert!(!descriptor.schema_version.contains("reflexivity-artifacts"));
        let cache_input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot,
                session.captured().snapshot().source_versions[0].source_hash,
                vec![],
            ),
        );
        assert!(matches!(
            registry
                .cache_key_for_phase(PipelinePhase::ArtifactCommit, &cache_input)
                .unwrap()
                .intent,
            PhaseCacheIntent::NoKey { .. }
        ));
        let WorkUnit::Module { module } = &unit else {
            panic!("module work unit")
        };
        submit.vc_descriptors = vec![VcTaskDescriptor::new(
            VcTaskDescriptorId::new("opaque-reflexivity-artifact"),
            module.clone(),
            VcOrderKey::new("opaque-reflexivity-artifact"),
            vec![],
            vec![EvidenceCandidateId::new("opaque-reflexivity-artifact")],
        )];
        let mut driver = CompilerDriver::new(registry).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                submit,
                Default::default(),
            )
            .unwrap();
        let events = driver.events(submission.session.id());
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices,
            "the explicit twelve-service profile intentionally has no discharge phase"
        );
        assert_eq!(
            submission
                .missing_services
                .iter()
                .map(|missing| missing.phase)
                .collect::<Vec<_>>(),
            [PipelinePhase::VcDischarge]
        );
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked)
        );
        assert!(
            graph
                .tasks()
                .iter()
                .all(|task| !matches!(task.kind, TaskKind::AtpSolve | TaskKind::BackendRun))
        );
        let artifact_task = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::ArtifactCommit && task.unit == unit)
            .unwrap();
        assert!(!artifact_task.dependencies.iter().any(|dependency| {
            graph
                .tasks()
                .iter()
                .any(|task| &task.id == dependency && task.kind == TaskKind::VcDischarge)
        }));
        let artifact_state = run
            .task_states
            .iter()
            .find(|state| state.task_id == artifact_task.id)
            .unwrap()
            .state;
        if !expected_commit {
            assert_ne!(artifact_state, TaskState::Completed);
            assert!(!events.events().iter().any(|event| matches!(
                event.kind,
                mizar_driver::events::BuildEventKind::ArtifactBoundary { .. }
            )));
            assert!(!events.events().iter().any(|event| matches!(
                event.kind,
                mizar_driver::events::BuildEventKind::SnapshotPublished { .. }
            )));
            assert!(
                run.phase_results
                    .get(&artifact_task.id)
                    .is_none_or(|rows| { rows.iter().all(|row| row.output_refs.is_empty()) })
            );
            assert_eq!(
                fs::read_dir(fixture.root.join("alpha/reflexivity-artifacts"))
                    .unwrap()
                    .count(),
                0,
                "unsupported source or policy mismatch cannot publish"
            );
            if text.starts_with("private ") {
                let kernel_task = graph
                    .tasks()
                    .iter()
                    .find(|task| task.kind == TaskKind::KernelCheck)
                    .unwrap();
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == kernel_task.id)
                        .unwrap()
                        .state,
                    TaskState::Completed,
                    "private source has a genuine sealed status before FILE facade refusal"
                );
                let [kernel_result] = run.phase_results[&kernel_task.id].as_slice() else {
                    panic!("one genuine private KernelCheck result")
                };
                let [private_status_ref] = kernel_result.output_refs.as_slice() else {
                    panic!("one sealed private reflexivity status")
                };
                assert_eq!(
                    private_status_ref.output_kind(),
                    &OutputKind::new("SourceTheoremReflexivityKernelStatus")
                );
                let private_status = owner
                    .storage()
                    .typed_handle::<(
                        mizar_core::core_ir::CoreIr,
                        mizar_vc::vc_ir::VcSet,
                        mizar_kernel::checker::KernelCheckResult,
                        mizar_proof::policy::PolicyDecision,
                        mizar_proof::status::ProofStatusProjection,
                        mizar_vc::dependency_slice::DependencySlice,
                    )>(private_status_ref, private_status_ref.output_kind())
                    .unwrap();
                let private_status = owner.storage().get(&private_status).unwrap();
                assert_eq!(
                    private_status.3.class,
                    mizar_proof::policy::CandidatePolicyClass::DischargedBuiltin
                );
                let frontend_ref = run
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter())
                    .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
                    .expect("genuine frontend output for private source");
                let frontend = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        frontend_ref,
                        frontend_ref.output_kind(),
                    )
                    .unwrap();
                let frontend = owner.storage().get(&frontend).unwrap();
                let ast = frontend.ast.as_ref().expect("private source parsed");
                let WorkUnit::Module { module } = &unit else {
                    panic!("module work unit")
                };
                let resolver_module = mizar_resolve::resolved_ast::ModuleId::new(
                    module.package.clone(),
                    module.path.clone(),
                );
                let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                    ast,
                    &resolver_module,
                )
                .unwrap()
                .expect("private source has genuine theorem inputs");
                assert_eq!(
                    mizar_proof::status::source_theorem_reflexivity_witness_file_candidate(
                        &arena,
                        &env,
                        session.captured().snapshot(),
                        &VerifierPolicy::release(),
                        "proof-witnesses",
                    )
                    .unwrap_err(),
                    "theorems.source.invalid"
                );
            }
            continue;
        }
        assert_eq!(artifact_state, TaskState::Completed);
        assert!(events.events().iter().any(|event| matches!(
            event.kind,
            mizar_driver::events::BuildEventKind::ArtifactBoundary { .. }
        )));
        assert!(!events.events().iter().any(|event| matches!(
            event.kind,
            mizar_driver::events::BuildEventKind::SnapshotPublished { .. }
        )));
        let kernel_task = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::KernelCheck)
            .unwrap();
        let [kernel_result] = run.phase_results[&kernel_task.id].as_slice() else {
            panic!("one genuine KernelCheck result")
        };
        let [status_ref] = kernel_result.output_refs.as_slice() else {
            panic!("one sealed reflexivity status")
        };
        let status_handle = owner
            .storage()
            .typed_handle::<(
                mizar_core::core_ir::CoreIr,
                mizar_vc::vc_ir::VcSet,
                mizar_kernel::checker::KernelCheckResult,
                mizar_proof::policy::PolicyDecision,
                mizar_proof::status::ProofStatusProjection,
                mizar_vc::dependency_slice::DependencySlice,
            )>(status_ref, status_ref.output_kind())
            .unwrap();
        let sealed = owner.storage().get(&status_handle).unwrap();
        assert_eq!(
            sealed.3.class,
            mizar_proof::policy::CandidatePolicyClass::DischargedBuiltin
        );
        assert_eq!(
            sealed.4.projected_status(),
            mizar_proof::status::ProjectedProofStatus::Accepted
        );
        assert_eq!(
            sealed.4.artifact_publication(),
            mizar_proof::status::ArtifactStatusPublication::ExternalDependencyGap(
                mizar_proof::status::ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported,
            )
        );
        assert!(sealed.4.trusted_used_axioms().is_some());
        assert!(
            sealed
                .4
                .reuse_metadata()
                .selected_proof_witness_hash()
                .is_none()
        );
        assert_eq!(
            sealed.5.completeness(),
            mizar_vc::dependency_slice::DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(sealed.5.requires_cache_miss());
        let [result] = run.phase_results[&artifact_task.id].as_slice() else {
            panic!("one ArtifactCommit result")
        };
        assert_eq!(result.status, PhaseStatus::Complete);
        let [artifact_ref] = result.output_refs.as_slice() else {
            panic!("one ArtifactCommit DATA output")
        };
        assert_eq!(
            artifact_ref.output_kind(),
            &OutputKind::new("SourceTheoremArtifactData")
        );
        assert_eq!(artifact_ref.lineage().parents.len(), 5);
        let commit = result
            .artifact_commit
            .as_ref()
            .expect("scheduler retains the actual manifest commit");
        assert!(
            commit
                .manifest()
                .artifact_root
                .starts_with("reflexivity-artifacts/")
        );
        let root = fixture
            .root
            .join("alpha")
            .join(&commit.manifest().artifact_root);
        if !open {
            let read = mizar_artifact::manifest::read_manifest_file(
                &root,
                mizar_artifact::manifest::ManifestFileReadOptions {
                    validate_references: true,
                    validate_witness_files: true,
                    validate_development_artifacts: true,
                    expected_manifest_hash: Some(commit.write().artifact_hash),
                },
            )
            .unwrap();
            assert_eq!(read.manifest, *commit.manifest());
            assert_eq!(read.artifact_hash, commit.write().artifact_hash);
        }
        let handle = owner
            .storage()
            .typed_handle::<mizar_artifact::verified_artifact::VerifiedArtifact>(
                artifact_ref,
                artifact_ref.output_kind(),
            )
            .unwrap();
        let artifact = owner.storage().get(&handle).unwrap();
        assert_eq!(
            artifact.schema_version,
            mizar_artifact::store::SchemaVersion::new(2, 0)
        );
        assert_eq!(artifact.proof_witnesses.len(), 1);
        assert_eq!(
            artifact.proof_witnesses[0].proof_status,
            mizar_artifact::proof_witness::ProofStatus::DischargedBuiltin
        );
        assert_eq!(
            artifact.proof_witnesses[0].obligation_id,
            sealed.4.obligation_id()
        );
        assert_eq!(
            artifact.proof_witnesses[0]
                .kernel_acceptance
                .accepted_result_hash
                .digest,
            mizar_proof::selection::TrustedKernelEvidence::from_policy_input(
                &mizar_proof::policy::KernelPolicyInput::from_kernel_result(
                    &sealed.2,
                    mizar_proof::policy::KernelEvidenceOrigin::BuiltinDischarge,
                )
            )
            .unwrap()
            .accepted_evidence_hash()
        );
        let witness = &artifact.proof_witnesses[0];
        let frontend_ref = run
            .phase_results
            .values()
            .flatten()
            .flat_map(|row| row.output_refs.iter())
            .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            .expect("genuine frontend output for captured source");
        let frontend = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                frontend_ref,
                frontend_ref.output_kind(),
            )
            .unwrap();
        let frontend = owner.storage().get(&frontend).unwrap();
        let ast = frontend.ast.as_ref().expect("captured theorem parses");
        let WorkUnit::Module { module } = &unit else {
            panic!("module work unit")
        };
        let resolver_module =
            mizar_resolve::resolved_ast::ModuleId::new(module.package.clone(), module.path.clone());
        let (arena, env) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &resolver_module)
                .unwrap()
                .expect("genuine source theorem metadata");
        let (scope, labels, resolved) =
            mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
        let typed =
            mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)
                .unwrap()
                .2;
        let check =
            mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &typed, &scope, &env, &labels, &resolved, None,
            )
            .unwrap();
        let vc_id = sealed.1.vcs()[0].id;
        let (trusted_metadata, expected_wire) =
            mizar_proof::status::source_theorem_reflexivity_trusted_witness_metadata_with_wire(
                &arena,
                &check,
                &env,
                snapshot,
                vc_id,
                &VerifierPolicy::release(),
            )
            .unwrap();
        let expected_evidence = mizar_proof::selection::TrustedKernelEvidence::from_policy_input(
            &mizar_proof::policy::KernelPolicyInput::from_kernel_result(
                &sealed.2,
                mizar_proof::policy::KernelEvidenceOrigin::BuiltinDischarge,
            ),
        )
        .unwrap();
        assert_eq!(
            trusted_metadata.kernel_acceptance(),
            &witness.kernel_acceptance
        );
        assert_eq!(
            trusted_metadata.trusted_kernel_evidence(),
            &expected_evidence
        );
        if open {
            // Scheduler cleanup removes ephemeral files after the committed handoff.
            // Retained DATA and commit metadata do not restore filesystem reachability.
            assert!(!root.exists());
            let ((_, slice, draft), file, expected_reference) =
                mizar_proof::status::source_theorem_reflexivity_witness_file_candidate(
                    &arena,
                    &env,
                    session.captured().snapshot(),
                    &VerifierPolicy::release(),
                    &witness.witness_path,
                )
                .unwrap();
            assert_eq!(&expected_reference, witness);
            assert_eq!(draft.payload_bytes(), expected_wire);
            assert_eq!(slice, sealed.5);
            let domain = mizar_artifact::store::artifact_hash_domain(
                witness.witness_artifact_hash.schema_family.clone(),
                witness.witness_artifact_hash.schema_version,
            );
            assert_eq!(
                domain.hash(&file, &[]),
                witness.witness_artifact_hash.digest
            );
            let value =
                mizar_artifact::verified_artifact::builtin_verified_artifact_json(&artifact)
                    .unwrap();
            let main_domain = mizar_artifact::store::artifact_hash_domain(
                mizar_artifact::verified_artifact::VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                artifact.schema_version,
            );
            assert_eq!(
                main_domain.hash(
                    &value,
                    &mizar_artifact::verified_artifact::artifact_hash_excluded_paths()
                ),
                commit.manifest().modules[0].artifact_hash.digest
            );
            assert!(mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                commit, &root, &artifact.module, snapshot.fingerprint(),
            ).is_err());
            owner.mark_obsolete(snapshot).unwrap();
            assert_eq!(*owner.storage().get(&handle).unwrap(), *artifact);
            assert!(owner.validate_current_snapshot(snapshot).is_err());
            continue;
        }
        let witness_domain = mizar_artifact::store::artifact_hash_domain(
            witness.witness_artifact_hash.schema_family.clone(),
            witness.witness_artifact_hash.schema_version,
        );
        let witness_read = mizar_artifact::store::read_published_artifact(
            &root,
            &mizar_artifact::store::PublishedArtifactPath::new(&witness.witness_path).unwrap(),
            mizar_artifact::store::PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&witness_domain),
                expected_artifact_hash: Some(witness.witness_artifact_hash.digest),
                ..mizar_artifact::store::PublishedArtifactReadOptions::default()
            },
        )
        .unwrap();
        let wire =
            mizar_proof::witness_store::read_source_theorem_witness_file(&witness_read, witness)
                .unwrap();
        assert_eq!(expected_wire.as_slice(), wire.as_slice());
        assert!(!wire.is_empty());
        assert_eq!(
            fs::read(root.join("main.mizir.json")).unwrap(),
            mizar_artifact::store::canonical_json_bytes(
                &mizar_artifact::verified_artifact::builtin_verified_artifact_json(&artifact)
                    .unwrap()
            )
        );
        let clean_parent = "reflexivity-artifacts/clean-worker4";
        fs::create_dir_all(fixture.root.join("alpha").join(clean_parent)).unwrap();
        let clean_owner = publisher(snapshot, 0, &unit);
        allow_reflexivity_artifact_outputs(&clean_owner, &unit, false);
        let clean_registry = reflexivity_artifact_registry(
            VerifierPolicy::release(),
            VerifierPolicy::release(),
            clean_parent,
            false,
        );
        let (_, mut clean_submit) = fixture.submit_request(vec![], vec![]);
        let WorkUnit::Module { module } = &unit else {
            panic!("module work unit")
        };
        clean_submit.vc_descriptors = vec![VcTaskDescriptor::new(
            VcTaskDescriptorId::new("opaque-reflexivity-artifact"),
            module.clone(),
            VcOrderKey::new("opaque-reflexivity-artifact"),
            vec![],
            vec![EvidenceCandidateId::new("opaque-reflexivity-artifact")],
        )];
        clean_submit.worker_count = 4;
        clean_submit.completion_order = CompletionOrder::Reverse;
        let clean_submission = CompilerDriver::new(clean_registry)
            .with_output_publisher(clean_owner.clone())
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                clean_submit,
                Default::default(),
            )
            .unwrap();
        let clean_run = clean_submission.scheduler_run.as_ref().unwrap();
        let clean_task = clean_submission
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::ArtifactCommit)
            .unwrap();
        let [clean_result] = clean_run.phase_results[&clean_task.id].as_slice() else {
            panic!("one clean ArtifactCommit result")
        };
        let [clean_ref] = clean_result.output_refs.as_slice() else {
            panic!("one clean ArtifactCommit DATA output")
        };
        let clean_handle = clean_owner
            .storage()
            .typed_handle::<mizar_artifact::verified_artifact::VerifiedArtifact>(
                clean_ref,
                clean_ref.output_kind(),
            )
            .unwrap();
        let clean_artifact = clean_owner.storage().get(&clean_handle).unwrap();
        assert_eq!(*clean_artifact, *artifact);
        let clean_commit = clean_result
            .artifact_commit
            .as_ref()
            .expect("clean worker run retains its manifest commit");
        assert_ne!(
            clean_commit.manifest().artifact_root,
            commit.manifest().artifact_root
        );
        let clean_root = fixture
            .root
            .join("alpha")
            .join(&clean_commit.manifest().artifact_root);
        let clean_read = mizar_artifact::manifest::read_manifest_file(
            &clean_root,
            mizar_artifact::manifest::ManifestFileReadOptions {
                validate_references: true,
                validate_witness_files: true,
                validate_development_artifacts: true,
                expected_manifest_hash: Some(clean_commit.write().artifact_hash),
            },
        )
        .unwrap();
        assert_eq!(clean_read.manifest, *clean_commit.manifest());
        let clean_witness = &clean_artifact.proof_witnesses[0];
        let clean_domain = mizar_artifact::store::artifact_hash_domain(
            clean_witness.witness_artifact_hash.schema_family.clone(),
            clean_witness.witness_artifact_hash.schema_version,
        );
        let clean_witness_read = mizar_artifact::store::read_published_artifact(
            &clean_root,
            &mizar_artifact::store::PublishedArtifactPath::new(&clean_witness.witness_path)
                .unwrap(),
            mizar_artifact::store::PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&clean_domain),
                expected_artifact_hash: Some(clean_witness.witness_artifact_hash.digest),
                ..mizar_artifact::store::PublishedArtifactReadOptions::default()
            },
        )
        .unwrap();
        assert_eq!(
            mizar_proof::witness_store::read_source_theorem_witness_file(
                &clean_witness_read,
                clean_witness,
            )
            .unwrap(),
            wire
        );
        let parent_outputs = artifact_ref
            .lineage()
            .parents
            .iter()
            .map(|parent| {
                run.phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter())
                    .find(|output| output.output() == *parent)
                    .expect("all five direct parents remain published")
                    .clone()
            })
            .collect::<Vec<_>>();
        let mut parent_kinds = parent_outputs
            .iter()
            .map(|output| output.output_kind().as_str().to_owned())
            .collect::<Vec<_>>();
        parent_kinds.sort();
        assert_eq!(
            parent_kinds,
            [
                "SourceTheoremCore",
                "SourceTheoremModuleSummary",
                "SourceTheoremPreparation",
                "SourceTheoremReflexivityKernelStatus",
                "SourceTheoremVC",
            ]
        );
        let direct_parent = "reflexivity-artifacts/direct-refusals";
        fs::create_dir_all(fixture.root.join("alpha").join(direct_parent)).unwrap();
        let mut consumer_builder = PhaseRegistryBuilder::new();
        consumer_builder
            .register_source_theorem_reflexivity_artifact_commit(
                VerifierPolicy::release(),
                (
                    "proof-witnesses".to_owned(),
                    "main.mizir.json".to_owned(),
                    direct_parent.to_owned(),
                ),
            )
            .unwrap();
        let consumer_registry = consumer_builder.build().unwrap();
        let captured = submission.session.captured();
        let source_inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        let input_for = |parents: &[AnyPhaseOutputRef],
                         bundle_snapshot,
                         work_unit: WorkUnit,
                         input_hash: mizar_session::Hash|
         -> Result<PhaseInput, String> {
            let handles = parents
                .iter()
                .map(|parent| {
                    SealedParentOutputHandle::from_published_output(
                        &owner,
                        snapshot,
                        parent.clone(),
                    )
                    .map_err(|error| error.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            PhaseDispatchInputBundle::new(bundle_snapshot, input_hash, vec![], handles)
                .map(|bundle| PhaseInput::new(work_unit, bundle))
                .map_err(|error| error.to_string())
        };
        let rejected = |input: PhaseInput, source_load| {
            let result = consumer_registry
                .execute_phase_with_resources(
                    PipelinePhase::ArtifactCommit,
                    input,
                    PhaseExecutionResources {
                        output_publisher: Some(owner.clone()),
                        source_load,
                        ..PhaseExecutionResources::default()
                    },
                )
                .unwrap()
                .result;
            assert_ne!(result.status, PhaseStatus::Complete);
            assert!(result.output_refs.is_empty());
            assert!(result.artifact_commit.is_none());
        };
        let mut missing = parent_outputs.clone();
        missing.pop();
        let source_hash = source_key(&captured.snapshot().source_versions[0]);
        rejected(
            input_for(&missing, snapshot, unit.clone(), source_hash).unwrap(),
            Some(source_inputs),
        );
        let mut foreign_role = parent_outputs.clone();
        let status_index = foreign_role
            .iter()
            .position(|output| {
                output.output_kind() == &OutputKind::new("SourceTheoremReflexivityKernelStatus")
            })
            .unwrap();
        foreign_role[status_index] = run
            .phase_results
            .values()
            .flatten()
            .flat_map(|row| row.output_refs.iter())
            .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremMetadata"))
            .unwrap()
            .clone();
        rejected(
            input_for(&foreign_role, snapshot, unit.clone(), source_hash).unwrap(),
            Some(source_inputs),
        );
        let WorkUnit::Module { module } = &unit else {
            panic!("module work unit")
        };
        // A differently bound policy needs its own genuine producer registry:
        // the original registry forbids conflicting content for one logical output id.
        let swapped_parent = "reflexivity-artifacts/other-policy";
        fs::create_dir_all(fixture.root.join("alpha").join(swapped_parent)).unwrap();
        let swapped_owner = publisher(snapshot, 0, &unit);
        allow_reflexivity_artifact_outputs(&swapped_owner, &unit, false);
        let swapped_policy = VerifierPolicy::release().with_require_kernel_certificates(false);
        let swapped_registry = reflexivity_artifact_registry(
            swapped_policy.clone(),
            swapped_policy,
            swapped_parent,
            false,
        );
        let (_, mut swapped_submit) = fixture.submit_request(vec![], vec![]);
        swapped_submit.vc_descriptors = vec![VcTaskDescriptor::new(
            VcTaskDescriptorId::new("other-policy"),
            module.clone(),
            VcOrderKey::new("other-policy"),
            vec![],
            vec![EvidenceCandidateId::new("other-policy")],
        )];
        let swapped_submission = CompilerDriver::new(swapped_registry)
            .with_output_publisher(swapped_owner)
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                swapped_submit,
                Default::default(),
            )
            .unwrap();
        let swapped_run = swapped_submission.scheduler_run.as_ref().unwrap();
        let swapped_status = swapped_run
            .phase_results
            .values()
            .flatten()
            .find(|result| {
                result.output_refs.iter().any(|output| {
                    output.output_kind() == &OutputKind::new("SourceTheoremReflexivityKernelStatus")
                })
            })
            .expect("independent genuine policy baseline has sealed status");
        assert_eq!(swapped_status.status, PhaseStatus::Complete);
        assert!(
            swapped_run
                .phase_results
                .values()
                .flatten()
                .any(|result| result.status == PhaseStatus::Complete
                    && result.artifact_commit.is_some())
        );
        let mut genuine_swap = parent_outputs.clone();
        genuine_swap[status_index] = swapped_status.output_refs[0].clone();
        assert!(
            input_for(&genuine_swap, snapshot, unit.clone(), source_hash).is_err(),
            "independent producer output cannot transfer publication authority"
        );
        rejected(
            input_for(
                &parent_outputs,
                snapshot,
                unit.clone(),
                mizar_session::Hash::from_bytes([0x55; 32]),
            )
            .unwrap(),
            Some(source_inputs),
        );
        let wrong_module = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("wrong"),
            ),
        };
        rejected(
            input_for(&parent_outputs, snapshot, wrong_module, source_hash).unwrap(),
            Some(source_inputs),
        );
        assert!(
            input_for(
                &parent_outputs,
                snapshot_id(0xe7),
                unit.clone(),
                source_hash,
            )
            .is_err()
        );
        rejected(
            input_for(&parent_outputs, snapshot, unit.clone(), source_hash).unwrap(),
            None,
        );
        let matching_cancel = CancellationToken {
            snapshot,
            generation: CancellationGeneration::new(1),
            reason: CancellationReason::ExplicitRequest,
        };
        let cancelled = consumer_registry
            .execute_phase_with_resources(
                PipelinePhase::ArtifactCommit,
                input_for(&parent_outputs, snapshot, unit.clone(), source_hash).unwrap(),
                PhaseExecutionResources {
                    cancellation: Some(matching_cancel),
                    output_publisher: Some(owner.clone()),
                    source_load: Some(source_inputs),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        assert_eq!(cancelled.status, PhaseStatus::Cancelled);
        assert!(cancelled.output_refs.is_empty());
        assert!(cancelled.artifact_commit.is_none());
        let duplicate = {
            let mut parents = parent_outputs.clone();
            parents[4] = parents[0].clone();
            input_for(&parents, snapshot, unit.clone(), source_hash)
        };
        assert!(duplicate.is_err());
        let stale_input = input_for(&parent_outputs, snapshot, unit.clone(), source_hash).unwrap();
        owner.mark_obsolete(snapshot).unwrap();
        rejected(stale_input, Some(source_inputs));
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert!(collected.outputs_dropped > 0);
        assert!(input_for(&parent_outputs, snapshot, unit.clone(), source_hash).is_err());
        assert!(
            fs::read_dir(fixture.root.join("alpha").join(direct_parent))
                .unwrap()
                .next()
                .is_none()
        );
        assert!(!artifact.exports.is_empty());
    }
}

#[test]
fn scheduled_reflexivity_kernel_profile_rejects_nonexact_candidates() {
    use mizar_build::task_graph::{
        BackendProfileId, EvidenceCandidateId, TaskKind, VcOrderKey, VcTaskDescriptor,
        VcTaskDescriptorId,
    };
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};

    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing",
        "multiple-evidence",
        "backend-profile",
        "missing-vc-service",
    ] {
        with_native_summary_parent(
            text,
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
                    ("KernelCheck", "SourceTheoremReflexivityKernelStatus"),
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
                if case != "missing-vc-service" {
                    builder.register_source_theorem_vc_generation(
                        GenerationSchemaVersion::new("source-theorem-local-v1"),
                        VcSchemaVersion::new("vc-v1"),
                    );
                }
                builder.register_source_theorem_reflexivity_kernel_check(VerifierPolicy::release());
                let registry = builder.build().unwrap();
                let (_, mut submit) = fixture.submit_request(vec![], vec![]);
                if case != "missing" {
                    let WorkUnit::Module { module } = &old.work_unit else {
                        panic!("module input")
                    };
                    submit.vc_descriptors = vec![VcTaskDescriptor::new(
                        VcTaskDescriptorId::new("opaque-negative-case"),
                        module.clone(),
                        VcOrderKey::new("opaque-order"),
                        if case == "backend-profile" {
                            vec![BackendProfileId::new("opaque-backend")]
                        } else {
                            vec![]
                        },
                        if case == "multiple-evidence" {
                            vec![
                                EvidenceCandidateId::new("opaque-evidence-a"),
                                EvidenceCandidateId::new("opaque-evidence-b"),
                            ]
                        } else {
                            vec![EvidenceCandidateId::new("opaque-evidence")]
                        },
                    )];
                }
                let submission = CompilerDriver::new(registry)
                    .with_output_publisher(owner)
                    .submit_captured(session.clone(), ids, snapshots, submit, Default::default())
                    .unwrap();
                let graph = submission.task_graph.as_ref().unwrap();
                let kernel_tasks = graph
                    .tasks()
                    .iter()
                    .filter(|task| task.kind == TaskKind::KernelCheck)
                    .collect::<Vec<_>>();
                match case {
                    "missing" => assert!(kernel_tasks.is_empty()),
                    "multiple-evidence" => assert_eq!(kernel_tasks.len(), 2),
                    "backend-profile" => assert!(
                        graph
                            .tasks()
                            .iter()
                            .any(|task| task.kind == TaskKind::AtpSolve
                                || task.kind == TaskKind::BackendRun)
                    ),
                    "missing-vc-service" => assert_eq!(kernel_tasks.len(), 1),
                    _ => unreachable!(),
                }
                let status_published = submission.scheduler_run.as_ref().is_some_and(|run| {
                    run.phase_results.values().flatten().any(|row| {
                        row.output_refs.iter().any(|output| {
                            output.output_kind()
                                == &OutputKind::new("SourceTheoremReflexivityKernelStatus")
                        })
                    })
                });
                assert!(
                    !status_published,
                    "{case} must fail closed without a reflexivity status"
                );
                if case == "missing" {
                    assert!(submission.scheduler_run.is_some());
                } else {
                    assert_eq!(
                        submission.status,
                        DriverSubmissionStatus::BlockedByMissingPhaseServices
                    );
                }
            },
        );
    }
}

#[test]
fn scheduled_reflexivity_artifact_edits_match_independent_clean_outputs() {
    use mizar_build::task_graph::{
        EvidenceCandidateId, TaskKind, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_proof::policy::VerifierPolicy;

    let baseline_text =
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let edits = [
        (
            "rename",
            "theorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
        ),
        (
            "whitespace",
            "\n theorem T: for X being set holds X = X proof let X be set; thus X = X; end; \n",
        ),
    ];
    let execute = |fixture: &Fixture,
                   bytes: &[u8],
                   generation: u64,
                   workers: usize,
                   reverse: bool,
                   output_parent: &str,
                   include_discharge: bool,
                   ids: &InMemorySessionIdAllocator,
                   snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>| {
        fixture.write(bytes);
        fs::create_dir_all(fixture.root.join("alpha").join(output_parent)).unwrap();
        let (mut request, mut submit) = fixture.submit_request(vec![], vec![]);
        let plan = mizar_build::planner::produce_build_plan(
            submit.plan_request.clone(),
            submit.workspace_packages.clone(),
            submit.lockfile.clone(),
        )
        .unwrap();
        request.dependency_inputs.lockfile_hash =
            mizar_build::planner::lockfile_content_hash(&plan.lockfile);
        request.dependency_inputs.toolchain = plan.toolchain.clone();
        request.verifier_config = mizar_driver::request::VerifierConfigInput::new(
            mizar_build::planner::workspace_config_content_hash(
                &plan.verifier_config,
                &plan.build_config,
            ),
        );
        request.source_inputs.versions.clear();
        request.generation = BuildRequestGeneration::new(generation);
        let source = SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("main"),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: "src/main.miz".into(),
            },
        };
        let session = request
            .capture_sources(
                ids,
                snapshots,
                vec![(DiskSourceLoader::new(fixture.root.join("alpha")), source)],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, 0, &unit);
        allow_reflexivity_artifact_outputs(&owner, &unit, include_discharge);
        let registry = reflexivity_artifact_registry(
            VerifierPolicy::release(),
            VerifierPolicy::release(),
            output_parent,
            include_discharge,
        );
        assert_eq!(
            registry.descriptors().len(),
            12 + usize::from(include_discharge)
        );
        let cache_input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot,
                session.captured().snapshot().source_versions[0].source_hash,
                vec![],
            ),
        );
        for descriptor in registry.descriptors() {
            for phase in &descriptor.phases {
                let query = registry.cache_key_for_phase(*phase, &cache_input).unwrap();
                assert!(
                    matches!(query.intent, PhaseCacheIntent::NoKey { .. }),
                    "{} {phase:?}",
                    descriptor.service_name
                );
            }
        }
        let WorkUnit::Module { module } = &unit else {
            panic!("module work unit")
        };
        submit.vc_descriptors = vec![VcTaskDescriptor::new(
            VcTaskDescriptorId::new("opaque-reflexivity-artifact"),
            module.clone(),
            VcOrderKey::new("opaque-reflexivity-artifact"),
            vec![],
            vec![EvidenceCandidateId::new("opaque-reflexivity-artifact")],
        )];
        submit.worker_count = workers;
        submit.completion_order = if reverse {
            CompletionOrder::Reverse
        } else {
            CompletionOrder::Canonical
        };
        let submission = CompilerDriver::new(registry)
            .with_output_publisher(owner.clone())
            .submit_captured(session, ids, snapshots, submit, Default::default())
            .unwrap();
        (submission, owner)
    };

    for (include_discharge, case_index, (case, edited_text)) in
        [false, true].into_iter().flat_map(|include_discharge| {
            edits
                .iter()
                .copied()
                .enumerate()
                .map(move |(case_index, case)| (include_discharge, case_index, case))
        })
    {
        let fixture = Fixture::new(baseline_text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (baseline, baseline_owner) = execute(
            &fixture,
            baseline_text.as_bytes(),
            (case_index * 3) as u64,
            1,
            false,
            &format!("reflexivity-artifacts/{case}-baseline"),
            include_discharge,
            &ids,
            &snapshots,
        );
        let baseline_hash = baseline.session.captured().snapshot().source_versions[0].source_hash;

        let (edited, edited_owner) = execute(
            &fixture,
            edited_text.as_bytes(),
            (case_index * 3 + 1) as u64,
            1,
            false,
            &format!("reflexivity-artifacts/{case}-edited"),
            include_discharge,
            &ids,
            &snapshots,
        );
        let edited_snapshot = edited.session.captured().snapshot();
        let edited_hash = edited_snapshot.source_versions[0].source_hash;
        assert_ne!(
            baseline_hash, edited_hash,
            "{case}: edit changes source identity"
        );

        let clean_fixture = Fixture::new(edited_text.as_bytes());
        let clean_ids = InMemorySessionIdAllocator::new();
        let clean_source_ids = InMemorySessionIdAllocator::new();
        // Reserve the baseline capture's source ID so this independent capture
        // has the same local source ordinal as the persistent edit run.
        clean_source_ids.next_source_id(edited_snapshot.id).unwrap();
        let clean_snapshots = SnapshotRegistry::with_allocator(clean_source_ids);
        let (clean, clean_owner) = execute(
            &clean_fixture,
            edited_text.as_bytes(),
            (case_index * 3 + 2) as u64,
            4,
            true,
            &format!("reflexivity-artifacts/{case}-clean"),
            include_discharge,
            &clean_ids,
            &clean_snapshots,
        );
        assert_ne!(
            edited.session.captured().snapshot().id,
            clean.session.captured().snapshot().id,
            "{case}: clean comparison has an independent captured session"
        );
        assert_ne!(
            std::sync::Arc::as_ptr(&edited_owner),
            std::sync::Arc::as_ptr(&clean_owner),
            "{case}: clean comparison has an independent output store"
        );
        assert_eq!(
            edited_snapshot.source_versions[0].source_id,
            clean.session.captured().snapshot().source_versions[0].source_id,
            "{case}: independent capture uses the same local source ordinal"
        );
        assert_eq!(
            edited_snapshot.source_versions[0].source_hash,
            clean.session.captured().snapshot().source_versions[0].source_hash
        );

        for (submission, owner) in [
            (&baseline, &baseline_owner),
            (&edited, &edited_owner),
            (&clean, &clean_owner),
        ] {
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if include_discharge {
                    BuildSessionOutcome::Succeeded
                } else {
                    BuildSessionOutcome::Blocked
                })
            );
            assert_eq!(
                submission.status,
                if include_discharge {
                    DriverSubmissionStatus::SchedulerValidated
                } else {
                    DriverSubmissionStatus::BlockedByMissingPhaseServices
                }
            );
            if include_discharge {
                assert!(submission.missing_services.is_empty());
            } else {
                assert_eq!(
                    submission
                        .missing_services
                        .iter()
                        .map(|missing| missing.phase)
                        .collect::<Vec<_>>(),
                    [PipelinePhase::VcDischarge]
                );
            }
            let graph = submission.task_graph.as_ref().unwrap();
            let run = submission.scheduler_run.as_ref().unwrap();
            assert_eq!(
                run.phase_results.values().flatten().count(),
                12 + usize::from(include_discharge),
                "{case}: all registered services produced actual phase results"
            );
            for task in graph.tasks() {
                let state = run
                    .task_states
                    .iter()
                    .find(|state| state.task_id == task.id)
                    .unwrap()
                    .state;
                assert_eq!(
                    state,
                    if task.kind == TaskKind::VcDischarge && !include_discharge {
                        TaskState::Blocked
                    } else {
                        TaskState::Completed
                    },
                    "{case}: unexpected task state for {:?}",
                    task.kind
                );
            }
            assert!(
                !graph
                    .tasks()
                    .iter()
                    .any(|task| matches!(task.kind, TaskKind::AtpSolve | TaskKind::BackendRun))
            );
            assert!(
                run.diagnostics.is_empty(),
                "{case}: no scheduler diagnostics"
            );
            assert!(
                run.phase_results.values().flatten().all(|row| {
                    row.status == PhaseStatus::Complete
                        && row.cache_observation.is_some()
                        && row.diagnostics.is_empty()
                }),
                "{case}: completed phases retain actual cache observations and no phase diagnostics"
            );
            if include_discharge {
                let refs = run
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter())
                    .collect::<Vec<_>>();
                let core_ref = refs
                    .iter()
                    .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremCore"))
                    .unwrap();
                let vc_ref = refs
                    .iter()
                    .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let discharge_ref = refs
                    .iter()
                    .find(|output| {
                        output.output_kind() == &OutputKind::new("SourceTheoremDischarge")
                    })
                    .unwrap();
                let core = owner
                    .storage()
                    .typed_handle::<mizar_core::core_ir::CoreIr>(core_ref, core_ref.output_kind())
                    .unwrap();
                let core = owner.storage().get(&core).unwrap();
                let vcs = owner
                    .storage()
                    .typed_handle::<mizar_vc::vc_ir::VcSet>(vc_ref, vc_ref.output_kind())
                    .unwrap();
                let vcs = owner.storage().get(&vcs).unwrap();
                let discharge = owner
                    .storage()
                    .typed_handle::<mizar_vc::discharge::DischargeOutput>(
                        discharge_ref,
                        discharge_ref.output_kind(),
                    )
                    .unwrap();
                let discharge = owner.storage().get(&discharge).unwrap();
                assert_eq!(
                    core.source_id(),
                    submission.session.captured().snapshot().source_versions[0].source_id
                );
                assert_eq!(vcs.source(), core.source_id());
                assert_eq!(vcs.vcs().len(), 1);
                assert_eq!(discharge.vc_set().vcs().len(), 1);
                assert_eq!(
                    *discharge,
                    mizar_vc::discharge::try_discharge(mizar_vc::discharge::DischargeInput {
                        vc_set: &vcs,
                        policy: &mizar_vc::discharge::DischargePolicy::default(),
                    },)
                    .unwrap(),
                    "{case}: retained discharge equals native discharge reconstruction"
                );
                assert_eq!(discharge_ref.lineage().parents.len(), 2);
                assert!(discharge_ref.lineage().parents.contains(&core_ref.output()));
                assert!(discharge_ref.lineage().parents.contains(&vc_ref.output()));
                assert_eq!(
                    graph
                        .tasks()
                        .iter()
                        .filter(|task| task.kind == TaskKind::VcDischarge)
                        .count(),
                    1
                );
                assert_eq!(
                    graph
                        .tasks()
                        .iter()
                        .filter(|task| task.kind == TaskKind::KernelCheck)
                        .count(),
                    1
                );
            }
            let artifact_task = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::ArtifactCommit)
                .unwrap();
            assert_eq!(run.phase_results[&artifact_task.id].len(), 1);
            assert_eq!(run.phase_results[&artifact_task.id][0].output_refs.len(), 1);
            assert!(
                run.phase_results[&artifact_task.id][0]
                    .artifact_commit
                    .is_some()
            );
            assert!(run.phase_results.values().flatten().any(|row| {
                row.output_refs.iter().any(|output| {
                    output.output_kind() == &OutputKind::new("SourceTheoremArtifactData")
                })
            }));
        }

        let output = |submission: &mizar_driver::driver::BuildSubmission,
                      owner: &PhaseOutputPublisher,
                      fixture: &Fixture,
                      canonical_snapshot: mizar_session::BuildSnapshotId| {
            let run = submission.scheduler_run.as_ref().unwrap();
            let graph = submission.task_graph.as_ref().unwrap();
            let task = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::ArtifactCommit)
                .unwrap();
            let result = &run.phase_results[&task.id][0];
            let reference = &result.output_refs[0];
            let handle = owner
                .storage()
                .typed_handle::<mizar_artifact::verified_artifact::VerifiedArtifact>(
                    reference,
                    reference.output_kind(),
                )
                .unwrap();
            let artifact = (*owner.storage().get(&handle).unwrap()).clone();
            let commit = result.artifact_commit.as_ref().unwrap();
            let root = fixture
                .root
                .join("alpha")
                .join(&commit.manifest().artifact_root);
            let read = mizar_artifact::manifest::read_manifest_file(
                &root,
                mizar_artifact::manifest::ManifestFileReadOptions {
                    validate_references: true,
                    validate_witness_files: true,
                    validate_development_artifacts: true,
                    expected_manifest_hash: Some(commit.write().artifact_hash),
                },
            )
            .unwrap();
            assert_eq!(read.manifest, *commit.manifest());
            let main_bytes = fs::read(root.join("main.mizir.json")).unwrap();
            assert_eq!(
                main_bytes,
                mizar_artifact::store::canonical_json_bytes(
                    &mizar_artifact::verified_artifact::builtin_verified_artifact_json(&artifact)
                        .unwrap()
                )
            );
            let witness = &artifact.proof_witnesses[0];
            let domain = mizar_artifact::store::artifact_hash_domain(
                witness.witness_artifact_hash.schema_family.clone(),
                witness.witness_artifact_hash.schema_version,
            );
            let witness_read = mizar_artifact::store::read_published_artifact(
                &root,
                &mizar_artifact::store::PublishedArtifactPath::new(&witness.witness_path).unwrap(),
                mizar_artifact::store::PublishedArtifactReadOptions {
                    artifact_hash_domain: Some(&domain),
                    expected_artifact_hash: Some(witness.witness_artifact_hash.digest),
                    ..mizar_artifact::store::PublishedArtifactReadOptions::default()
                },
            )
            .unwrap();
            let wire = mizar_proof::witness_store::read_source_theorem_witness_file(
                &witness_read,
                witness,
            )
            .unwrap();
            assert!(!wire.is_empty());
            let kernel_task = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::KernelCheck)
                .unwrap();
            let [kernel_result] = run.phase_results[&kernel_task.id].as_slice() else {
                panic!("one actual KernelCheck result")
            };
            let [status_ref] = kernel_result.output_refs.as_slice() else {
                panic!("one actual sealed reflexivity status")
            };
            let status_handle = owner
                .storage()
                .typed_handle::<(
                    mizar_core::core_ir::CoreIr,
                    mizar_vc::vc_ir::VcSet,
                    mizar_kernel::checker::KernelCheckResult,
                    mizar_proof::policy::PolicyDecision,
                    mizar_proof::status::ProofStatusProjection,
                    mizar_vc::dependency_slice::DependencySlice,
                )>(status_ref, status_ref.output_kind())
                .unwrap();
            let sealed = owner.storage().get(&status_handle).unwrap();
            let frontend_ref = run
                .phase_results
                .values()
                .flatten()
                .flat_map(|row| row.output_refs.iter())
                .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
                .expect("actual Frontend output");
            let frontend = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    frontend_ref,
                    frontend_ref.output_kind(),
                )
                .unwrap();
            let frontend = owner.storage().get(&frontend).unwrap();
            let ast = frontend.ast.as_ref().expect("captured source parses");
            let WorkUnit::Module { module } = &task.unit else {
                panic!("module ArtifactCommit task")
            };
            let resolver_module = mizar_resolve::resolved_ast::ModuleId::new(
                module.package.clone(),
                module.path.clone(),
            );
            let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                ast,
                &resolver_module,
            )
            .unwrap()
            .expect("genuine source theorem inputs");
            let ((_, slice, draft), expected_file, expected_reference) =
                mizar_proof::status::source_theorem_reflexivity_witness_file_candidate(
                    &arena,
                    &env,
                    submission.session.captured().snapshot(),
                    &VerifierPolicy::release(),
                    &witness.witness_path,
                )
                .unwrap();
            assert_eq!(witness, &expected_reference);
            assert_eq!(witness_read.value, expected_file);
            assert_eq!(slice, sealed.5);
            assert_eq!(draft.payload_bytes(), wire);
            let (scope, labels, resolved) =
                mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
            let typed =
                mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)
                    .unwrap()
                    .2;
            let check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &typed, &scope, &env, &labels, &resolved, None,
            )
            .unwrap();
            let (projected_metadata, projected_wire) =
                mizar_proof::status::source_theorem_reflexivity_trusted_witness_metadata_with_wire(
                    &arena,
                    &check,
                    &env,
                    canonical_snapshot,
                    sealed.1.vcs()[0].id,
                    &VerifierPolicy::release(),
                )
                .unwrap();
            let discharge = if include_discharge {
                let discharge_ref = run
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|row| row.output_refs.iter())
                    .find(|output| {
                        output.output_kind() == &OutputKind::new("SourceTheoremDischarge")
                    })
                    .unwrap();
                let discharge_handle = owner
                    .storage()
                    .typed_handle::<mizar_vc::discharge::DischargeOutput>(
                        discharge_ref,
                        discharge_ref.output_kind(),
                    )
                    .unwrap();
                let discharge = owner.storage().get(&discharge_handle).unwrap();
                Some((*discharge).clone())
            } else {
                None
            };
            (
                artifact,
                read.manifest,
                wire,
                projected_metadata,
                projected_wire,
                discharge,
            )
        };
        let edited_output = output(
            &edited,
            &edited_owner,
            &fixture,
            edited.session.captured().snapshot().id,
        );
        let clean_output = output(
            &clean,
            &clean_owner,
            &clean_fixture,
            edited.session.captured().snapshot().id,
        );
        assert_eq!(
            edited_output.2, edited_output.4,
            "{case}: edited FILE matches owner projection"
        );
        assert_eq!(
            edited_output.2, clean_output.4,
            "{case}: clean FILE content under the edited snapshot identity"
        );
        assert_eq!(
            edited_output.3, clean_output.3,
            "{case}: projected trusted Kernel metadata"
        );
        assert_eq!(
            edited_output.0.source_hash, edited_hash,
            "{case}: edited DATA names captured bytes"
        );
        assert_eq!(
            clean_output.0.source_hash,
            clean.session.captured().snapshot().source_versions[0].source_hash,
            "{case}: clean DATA names its captured bytes"
        );
        // Artifact witness binding hashes, paths, and the derived implementation
        // hash are snapshot-local; every actual FILE is owner-checked above.
        let mut normalized_clean = clean_output.0.clone();
        let edited_witness = &edited_output.0.proof_witnesses[0];
        let clean_witness = &mut normalized_clean.proof_witnesses[0];
        clean_witness.witness_path = edited_witness.witness_path.clone();
        clean_witness.witness_artifact_hash.digest = edited_witness.witness_artifact_hash.digest;
        clean_witness.kernel_acceptance.target_binding_hash.digest =
            edited_witness.kernel_acceptance.target_binding_hash.digest;
        clean_witness.kernel_acceptance.provenance_hash.digest =
            edited_witness.kernel_acceptance.provenance_hash.digest;
        clean_witness.kernel_acceptance.accepted_result_hash.digest =
            edited_witness.kernel_acceptance.accepted_result_hash.digest;
        normalized_clean.implementation_hash =
            normalized_clean.compute_implementation_hash().unwrap();
        assert_eq!(
            edited_output.0, normalized_clean,
            "{case}: retained VerifiedArtifact DATA"
        );

        // The manifest root is output-local; module artifact references carry
        // the same snapshot-local witness identities as the main file.
        let mut normalized_manifest = clean_output.1.clone();
        normalized_manifest.artifact_root = edited_output.1.artifact_root.clone();
        let edited_entry = &edited_output.1.modules[0];
        let clean_entry = &mut normalized_manifest.modules[0];
        clean_entry.artifact_hash.digest = edited_entry.artifact_hash.digest;
        clean_entry.implementation_hash.digest = edited_entry.implementation_hash.digest;
        for (clean, edited) in clean_entry
            .proof_witnesses
            .iter_mut()
            .zip(&edited_entry.proof_witnesses)
        {
            clean.witness_path = edited.witness_path.clone();
            clean.witness_artifact_hash.digest = edited.witness_artifact_hash.digest;
        }
        assert_eq!(
            edited_output.1, normalized_manifest,
            "{case}: validated manifest module entries"
        );
        if include_discharge {
            let edited_discharge = edited_output.5.as_ref().unwrap();
            let clean_discharge = clean_output.5.as_ref().unwrap();
            let clean_vcs = clean_discharge.vc_set();
            let normalized_vcs = mizar_vc::vc_ir::VcSet::try_new(mizar_vc::vc_ir::VcSetParts {
                schema_version: clean_vcs.schema_version().clone(),
                snapshot: edited_discharge.vc_set().snapshot(),
                source: clean_vcs.source(),
                module: clean_vcs.module().clone(),
                generated_formulas: clean_vcs.generated_formulas().to_vec(),
                vcs: clean_vcs.vcs().to_vec(),
                seed_accounting: clean_vcs.seed_accounting().to_vec(),
            })
            .unwrap();
            assert_eq!(
                edited_discharge.vc_set(),
                &normalized_vcs,
                "{case}: retained discharge VC data agrees after snapshot-local set identity normalization"
            );
            assert_eq!(
                edited_discharge.evidence_records(),
                clean_discharge.evidence_records(),
                "{case}: retained discharge evidence records agree"
            );
            assert_eq!(
                edited_discharge.explanations(),
                clean_discharge.explanations(),
                "{case}: retained discharge explanations agree"
            );
        }
    }
}

#[test]
fn scheduled_reflexivity_discharge_admission_requires_the_exact_profile() {
    use mizar_build::task_graph::{
        EvidenceCandidateId, TaskKind, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};

    #[derive(Debug)]
    struct DescriptorService(mizar_driver::registry::PhaseDescriptor);
    impl mizar_driver::registry::PhaseService for DescriptorService {
        fn phase(&self) -> mizar_driver::registry::PhaseDescriptor {
            self.0.clone()
        }
        fn cache_key(
            &self,
            _input: &mizar_driver::registry::PhaseInput,
            _context: &mizar_driver::registry::PhaseCacheContext,
        ) -> PhaseCacheIntent {
            PhaseCacheIntent::NoKey {
                reason: "admission spoof has no cache identity".to_owned(),
            }
        }
        fn execute(
            &self,
            _input: mizar_driver::registry::PhaseInput,
            _context: mizar_driver::registry::PhaseExecutionContext<'_>,
        ) -> mizar_driver::registry::PhaseResult {
            mizar_driver::registry::PhaseResult::complete()
        }
    }

    let source = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let fixture = Fixture::new(source.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, _) = fixture.submit_request(vec![], vec![]);
    draft.source_inputs.versions.clear();
    let input = SourceInput {
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new("main"),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: Edition::new("2025"),
        origin: SourceOriginInput::Disk {
            path: "src/main.miz".into(),
        },
    };
    let session = draft
        .capture_sources(
            &ids,
            &snapshots,
            vec![(DiskSourceLoader::new(fixture.root.join("alpha")), input)],
        )
        .unwrap();
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    let WorkUnit::Module { module } = &unit else {
        unreachable!()
    };

    let nominal = reflexivity_artifact_registry(
        VerifierPolicy::release(),
        VerifierPolicy::release(),
        "reflexivity-artifacts/admission",
        true,
    );
    let exact_discharge = nominal
        .descriptor_for_phase(PipelinePhase::VcDischarge)
        .unwrap()
        .clone();

    let profile = |variant: &str| {
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
                GenerationSchemaVersion::new("source-theorem-local-v1"),
                VcSchemaVersion::new("vc-v1"),
            )
            .register_source_theorem_reflexivity_kernel_check(VerifierPolicy::release());
        match variant {
            "spoofed-concrete-type" => {
                builder.register(DescriptorService(exact_discharge.clone()));
            }
            "mismatched-descriptor" => {
                builder.register(DescriptorService(
                    mizar_driver::registry::PhaseDescriptor::new(
                        "UnexpectedDischargeService",
                        mizar_driver::registry::PhaseOwner::MizarVc,
                        vec![PipelinePhase::VcDischarge],
                        exact_discharge.schema_version.clone(),
                        exact_discharge.output_kind.clone(),
                    )
                    .unwrap(),
                ));
            }
            _ => {
                builder.register_source_theorem_vc_discharge(
                    mizar_vc::discharge::DischargePolicy::default(),
                );
            }
        }
        builder
            .register_source_theorem_reflexivity_artifact_commit(
                VerifierPolicy::release(),
                (
                    "proof-witnesses".to_owned(),
                    "main.mizir.json".to_owned(),
                    "reflexivity-artifacts/admission".to_owned(),
                ),
            )
            .unwrap();
        if variant == "surplus-service" {
            let config = mizar_driver::registry::StageConfig::source_theorem(vec![]);
            builder
                .register_source_theorem_atp_solve(
                    config.discharge_policy,
                    config.logic_profile,
                    config.concrete_format,
                )
                .unwrap();
        }
        builder.build().unwrap()
    };

    for case in [
        "nominal",
        "spoofed-concrete-type",
        "mismatched-descriptor",
        "surplus-service",
        "duplicate-discharge-module",
        "duplicate-kernel-module",
    ] {
        let registry = profile(case);
        let owner = publisher(session.captured().snapshot().id, 0, &unit);
        allow_reflexivity_artifact_outputs(&owner, &unit, true);
        let (_, mut submit) = fixture.submit_request(vec![], vec![]);
        let descriptor = |id: &str, evidence: &[&str]| {
            VcTaskDescriptor::new(
                VcTaskDescriptorId::new(id),
                module.clone(),
                VcOrderKey::new(id),
                vec![],
                evidence
                    .iter()
                    .map(|candidate| EvidenceCandidateId::new(*candidate))
                    .collect(),
            )
        };
        submit.vc_descriptors = match case {
            "duplicate-discharge-module" => vec![
                descriptor("admission-discharge-a", &[]),
                descriptor("admission-discharge-b", &[]),
            ],
            "duplicate-kernel-module" => {
                vec![descriptor(
                    "admission-kernel",
                    &["candidate-a", "candidate-b"],
                )]
            }
            _ => vec![descriptor("admission-single", &["candidate"])],
        };
        let submission = CompilerDriver::new(registry)
            .with_output_publisher(owner)
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                submit,
                Default::default(),
            )
            .unwrap();
        assert!(submission.missing_services.is_empty());
        assert!(submission.scheduler_run.is_some());
        let graph = submission.task_graph.as_ref().unwrap();
        let discharges = graph
            .tasks()
            .iter()
            .filter(|task| task.kind == TaskKind::VcDischarge)
            .collect::<Vec<_>>();
        let kernels = graph
            .tasks()
            .iter()
            .filter(|task| task.kind == TaskKind::KernelCheck)
            .collect::<Vec<_>>();
        assert!(
            !graph
                .tasks()
                .iter()
                .any(|task| { matches!(task.kind, TaskKind::AtpSolve | TaskKind::BackendRun) })
        );
        let has_discharge_output = submission.scheduler_run.as_ref().is_some_and(|run| {
            run.phase_results.values().flatten().any(|row| {
                row.output_refs.iter().any(|output| {
                    output.output_kind() == &OutputKind::new("SourceTheoremDischarge")
                })
            })
        });
        assert_eq!(has_discharge_output, case == "nominal", "{case}");
        assert_eq!(
            discharges.len(),
            if case == "duplicate-discharge-module" {
                2
            } else {
                1
            }
        );
        assert_eq!(
            kernels.len(),
            match case {
                "duplicate-discharge-module" => 0,
                "duplicate-kernel-module" => 2,
                _ => 1,
            }
        );
        for discharge in discharges {
            let mut dependencies = graph
                .tasks()
                .iter()
                .filter(|task| discharge.dependencies.contains(&task.id))
                .map(|task| task.kind)
                .collect::<Vec<_>>();
            dependencies.sort();
            assert_eq!(
                dependencies,
                if case == "nominal" {
                    [TaskKind::CheckAndElaborate, TaskKind::VcGenerate]
                } else {
                    [TaskKind::PackageResolve, TaskKind::VcGenerate]
                },
                "{case}: only the exact profile gains the direct source-check bridge"
            );
        }
    }
}

#[test]
fn scheduled_mapped_citation_publishes_genuine_base_main() {
    use mizar_artifact::direct_by::DirectByTarget;
    use mizar_build::task_graph::{
        EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_proof::policy::VerifierPolicy;

    // Spec §§16.1/16.5.1: the earlier clean theorem is an actual compiler citation.
    let base_main = "theorem Base: for X being set holds X = X proof let X be set; thus X = X; end;\ntheorem Main: for Y being set holds Y = Y proof let Y be set; thus Y = Y by Base; end;\n";
    let chain = "\u{feff}:: captured 😀\r\ntheorem ZBase: for X being set holds X = X proof let X be set; thus X = X; end;\r\ntheorem AMiddle: for Y being set holds Y = Y proof let Y be set; thus Y = Y by ZBase; end;\r\ntheorem Main: for Z being set holds Z = Z proof let Z be set; thus Z = Z by AMiddle; end;\r\n";
    let refused_sources = [
        "open theorem Base: for X being set holds X = X;",
        "assumed theorem Base: for X being set holds X = X;",
        "private theorem Base: for X being set holds X = X proof let X be set; thus X = X; end;",
        "lemma Base: for X being set holds X = X proof let X be set; thus X = X; end;",
        "theorem Base: for X being object holds X = X proof let X be object; thus X = X; end;",
        "theorem Base: for X being set holds X = X proof let X be set; thus X = X by Base; end;",
        "theorem Base: for X being set holds X = X proof let X be set; thus X = X by Main; end;",
    ]
    .into_iter()
    .flat_map(|base| {
        [
            format!("{base}\ntheorem Main: for Y being set holds Y = Y proof let Y be set; thus Y = Y by Base; end;\n"),
            format!("{base}\ntheorem Middle: for Y being set holds Y = Y proof let Y be set; thus Y = Y by Base; end;\ntheorem Main: for Z being set holds Z = Z proof let Z be set; thus Z = Z by Middle; end;\n"),
        ]
    })
    .collect::<Vec<_>>();
    for (text, open, labels, expected_citations, expected_publication) in [
        (
            base_main,
            false,
            &["Base", "Main"][..],
            &[("Main", "Base")][..],
            true,
        ),
        (
            chain,
            true,
            &["ZBase", "AMiddle", "Main"][..],
            &[("AMiddle", "ZBase"), ("Main", "AMiddle")][..],
            true,
        ),
    ]
    .into_iter()
    .chain(
        refused_sources
            .iter()
            .map(|text| (text.as_str(), false, &[][..], &[][..], false)),
    ) {
        // Compiler syntax ranges use loaded text: a leading BOM is removed and
        // each CRLF becomes LF (mizar_session::normalize_source_text).
        let loaded_text = text
            .strip_prefix('\u{feff}')
            .unwrap_or(text)
            .replace("\r\n", "\n");
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut request, mut submit) = fixture.submit_request(vec![], vec![]);
        let plan = mizar_build::planner::produce_build_plan(
            submit.plan_request.clone(),
            submit.workspace_packages.clone(),
            submit.lockfile.clone(),
        )
        .unwrap();
        request.dependency_inputs.lockfile_hash =
            mizar_build::planner::lockfile_content_hash(&plan.lockfile);
        request.dependency_inputs.toolchain = plan.toolchain.clone();
        request.verifier_config = mizar_driver::request::VerifierConfigInput::new(
            mizar_build::planner::workspace_config_content_hash(
                &plan.verifier_config,
                &plan.build_config,
            ),
        );
        request.source_inputs.versions.clear();
        let session = request
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new("main"),
                        normalized_path: fixture.version.normalized_path.clone(),
                        edition: Edition::new("2025"),
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
                    },
                )],
            )
            .unwrap();
        if open {
            fixture.write(&[0xff]);
        }
        let module = mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        );
        let unit = WorkUnit::Module {
            module: module.clone(),
        };
        let owner = publisher(session.captured().snapshot().id, 0, &unit);
        allow_reflexivity_artifact_outputs(&owner, &unit, true);
        fs::create_dir_all(fixture.root.join("alpha/mapped-artifacts")).unwrap();
        submit.vc_descriptors = vec![VcTaskDescriptor::new(
            VcTaskDescriptorId::new("mapped-reflexivity"),
            module,
            VcOrderKey::new("mapped-reflexivity"),
            vec![],
            vec![EvidenceCandidateId::new("mapped-reflexivity")],
        )];
        let mut driver = CompilerDriver::new(reflexivity_artifact_registry(
            VerifierPolicy::release(),
            VerifierPolicy::release(),
            "mapped-artifacts",
            true,
        ))
        .with_output_publisher(owner);
        let submission = driver
            .submit_captured(session, &ids, &snapshots, submit, Default::default())
            .unwrap();
        let stream = driver.events(submission.session.id());
        let publications = stream
            .events()
            .iter()
            .filter_map(|event| match &event.kind {
                mizar_driver::events::BuildEventKind::SnapshotPublished { snapshot } => {
                    Some(snapshot)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if !expected_publication {
            assert!(publications.is_empty(), "refused cited owner: {text}");
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.values().flatten().all(|result| {
                result.artifact_commit.is_none()
                    && result.output_refs.iter().all(|output| {
                        output.output_kind() != &OutputKind::new("SourceTheoremArtifactData")
                    })
            }));
            assert_eq!(
                fs::read_dir(fixture.root.join("alpha/mapped-artifacts"))
                    .unwrap()
                    .count(),
                0,
                "refused direct or transitive cited owner leaves no staged artifacts: {text}"
            );
            continue;
        }
        assert_eq!(
            publications.len(),
            1,
            "{:?}",
            submission.scheduler_run.as_ref().unwrap().task_states
        );
        let published = publications[0];
        let [artifact] = published.artifacts() else {
            panic!("one genuine module artifact")
        };
        assert_eq!(artifact.exports.len(), labels.len());
        assert_eq!(artifact.obligations.len(), labels.len());
        assert_eq!(artifact.proof_witnesses.len(), labels.len());
        assert!(
            artifact
                .obligations
                .iter()
                .all(|row| row.status
                    == mizar_artifact::verified_artifact::ObligationStatus::Accepted)
        );
        assert!(
            artifact.proof_witnesses.iter().all(|row| row.proof_status
                == mizar_artifact::proof_witness::ProofStatus::DischargedBuiltin)
        );
        let json =
            mizar_artifact::verified_artifact::builtin_verified_artifact_json(artifact).unwrap();
        assert_eq!(
            mizar_artifact::verified_artifact::read_builtin_verified_artifact(
                &json,
                Default::default()
            )
            .unwrap(),
            **artifact
        );
        for owner in 0..labels.len() {
            for case in [
                "missing-witness",
                "missing-status",
                "substituted-status",
                "open-status",
            ] {
                let mut changed = json.clone();
                let mizar_artifact::store::CanonicalJson::Object(fields) = &mut changed else {
                    unreachable!()
                };
                let field = if case == "missing-witness" {
                    "proof_witnesses"
                } else {
                    "obligations"
                };
                let mizar_artifact::store::CanonicalJson::Array(rows) =
                    fields.get_mut(field).unwrap()
                else {
                    unreachable!()
                };
                match case {
                    "missing-witness" | "missing-status" => {
                        rows.remove(owner);
                    }
                    "substituted-status" => {
                        let mizar_artifact::store::CanonicalJson::Object(other) =
                            &rows[(owner + 1) % labels.len()]
                        else {
                            unreachable!()
                        };
                        let fingerprint = other["obligation_fingerprint"].clone();
                        let mizar_artifact::store::CanonicalJson::Object(row) = &mut rows[owner]
                        else {
                            unreachable!()
                        };
                        row.insert("obligation_fingerprint".to_owned(), fingerprint);
                    }
                    _ => {
                        let mizar_artifact::store::CanonicalJson::Object(row) = &mut rows[owner]
                        else {
                            unreachable!()
                        };
                        row.insert(
                            "status".to_owned(),
                            mizar_artifact::store::CanonicalJson::string("open"),
                        );
                        row.insert(
                            "accepted_witness_obligation_id".to_owned(),
                            mizar_artifact::store::CanonicalJson::Null,
                        );
                    }
                }
                assert!(matches!(
                    mizar_artifact::verified_artifact::read_builtin_verified_artifact(
                        &changed,
                        Default::default()
                    ),
                    Err(mizar_artifact::verified_artifact::VerifiedArtifactError::WitnessReferenceMismatch { .. })
                ), "{case}: owner {owner} must fail witness/status binding before hash validation");
            }
        }
        assert!(artifact.provenance.captured_source.is_none());
        let metadata = published.source_metadata()[0].as_ref().unwrap();
        assert!(metadata.identity().is_none());
        let citations = metadata.direct_by().expect("genuine mapped citation");
        assert!(citations.complete());
        assert_eq!(citations.examinations().len(), labels.len());
        assert_eq!(citations.occurrences().len(), expected_citations.len());
        let captured = &published.captured().snapshot().source_versions[0];
        assert_eq!(
            citations.captured_source(),
            &(
                captured.source_id,
                captured.origin.clone(),
                captured.source_hash
            )
        );
        for (occurrence, (citing, target_name)) in
            citations.occurrences().iter().zip(expected_citations)
        {
            let DirectByTarget::Theorem(target) = &occurrence.target else {
                panic!("compiler-bound theorem target")
            };
            for (actual, name) in [
                (&target.export, *target_name),
                (&occurrence.citing.as_ref().unwrap().export, *citing),
            ] {
                let expected = artifact
                    .exports
                    .iter()
                    .find(|export| {
                        export.source_range.start_byte
                            == loaded_text.find(&format!("theorem {name}:")).unwrap() as u64
                    })
                    .unwrap();
                assert_eq!(actual, expected);
            }
            assert_eq!(
                &loaded_text[occurrence.target_range.start..occurrence.target_range.end],
                *target_name
            );
            assert_eq!(
                target.publication_hash,
                published.manifests()[0].artifact_hash
            );
        }
        let commit = &published.manifests()[0];
        assert_eq!(
            commit.manifest.modules[0].proof_witnesses.len(),
            labels.len()
        );
        if open {
            assert!(
                !fixture
                    .root
                    .join("alpha")
                    .join(&commit.manifest.artifact_root)
                    .exists()
            );
            continue;
        }
        let read = mizar_artifact::manifest::read_manifest_file(
            fixture
                .root
                .join("alpha")
                .join(&commit.manifest.artifact_root),
            mizar_artifact::manifest::ManifestFileReadOptions {
                validate_references: true,
                validate_witness_files: true,
                validate_development_artifacts: true,
                expected_manifest_hash: Some(commit.artifact_hash),
            },
        )
        .unwrap();
        assert_eq!(read, *commit);
    }
}
