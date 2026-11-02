use super::*;

#[test]
fn source_theorem_canonical_components_exact_content_and_edits() {
    use mizar_core::elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads};
    use mizar_vc::{
        generator::{CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput},
        vc_ir::*,
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    // Independent frozen grammar vectors prepared before this API, not producer output.
    let vectors = [
        (
            VC_SCHEMA.to_owned(),
            GENERATION_SCHEMA.to_owned(),
            [
                "0b1995287247ab14546781d55ed0bf31d69fb5059a99cb121ef58294dcdc466f",
                "803374a93f4ee93612cf149c760849274ad5b2c9b3996c93fd38a84ecee63874",
                "73e14c7080ab133d7635e954d0776de272ddd550a92395f04546f46796db5766",
            ],
        ),
        (
            format!("{VC_SCHEMA}-changed"),
            GENERATION_SCHEMA.to_owned(),
            [
                "d1dacac4b96f867781b298e90217f090dcefa8c2821906b8c5cd4bb71761bf1e",
                "f769d4420f35013b4fe2493ea72271f39985b5cc58a87b87d61c2498c8ed6eb4",
                "d95ff07e4cfc0ee6ffc3d2804e3069c58e8cff06ecb4db7bb601b1ae0b1e62ee",
            ],
        ),
        (
            VC_SCHEMA.to_owned(),
            format!("{GENERATION_SCHEMA}-changed"),
            [
                "4ba3de656ac4560c67537c459027a63d47b39e41067cdf1455b237901337e65f",
                "4c17538a0e5331db9a12a7c9aeddf615bfc4cbfda6e707886c0287c2f0261183",
                "e778cc5f716882654fd614030c761f56654537fe6ba00d4726efef5eff2ffad1",
            ],
        ),
    ];
    for source in [
        original.to_owned(),
        "theorem T: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;".to_owned(),
        format!(":: leading trivia\n{}\n", original.replace(" proof ", "\nproof\n")),
        original.replace("T:", "U:"),
    ] {
        with_step5c14_theorem_check(&source, |check| {
            let core = lower_source_theorem_skeletons(check, None)?;
            let core_before = core.clone();
            let seed = core.obligation_seeds().iter().next().unwrap().1;
            let roots = [seed.context[0], seed.goal.unwrap()];
            let formulas = source_theorem_formula_payloads(check, &core, &roots)?;
            let formulas_before = formulas.clone();
            let vcs = generate_core_vcs(&core, snapshot_id(777))?;
            let vcs_before = vcs.clone();
            let vc = &vcs.vcs()[0];
            let anchor = vc.anchor.clone();
            assert!(!anchor.is_complete());
            assert_eq!(vcs.canonical_vc_fingerprint(vc.id), None);
            assert_eq!(vcs.local_context_fingerprint(vc.id), None);
            let baseline = vcs.source_theorem_canonical_components(vc.id, &core, &formulas)?;
            assert_eq!(
                baseline,
                vcs.source_theorem_canonical_components(vc.id, &core, &formulas)?
            );
            let actual = [baseline.0.hash(), baseline.1, baseline.2.hash()].map(|hash| {
                hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
            });
            assert_eq!(actual, vectors[0].2);
            let fresh = generate_core_vcs(&core, snapshot_id(778))?;
            assert_ne!(fresh.snapshot(), vcs.snapshot());
            assert_eq!(
                fresh.source_theorem_canonical_components(fresh.vcs()[0].id, &core, &formulas)?,
                baseline
            );
            if source == original {
                for (vc_schema, generation_schema, expected) in &vectors[1..] {
                    // Regenerate candidates and normalize with the actual alternate schemas.
                    let flow = mizar_core::control_flow::build_control_flow_ir(&core);
                    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
                    let intake = SeedIntakeTable::try_from_handoff(&handoff)
                        .map_err(|error| error.to_string())?;
                    let package = core.module_id().package().as_str();
                    let path = core.module_id().path().as_str();
                    let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                        schema_version: &GenerationSchemaVersion::new(generation_schema.clone()),
                        module: &VcModuleRef::new(format!(
                            "package={}:{};module={}:{}", package.len(), package, path.len(), path
                        )),
                        intake: &intake,
                        handoff: &handoff,
                        flow_output: Some(&flow),
                    }).map_err(|error| error.to_string())?;
                    let changed = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
                        schema_version: &VcSchemaVersion::new(vc_schema.clone()),
                        snapshot: vcs.snapshot(),
                        source: core.source_id(),
                        candidates: &candidates,
                    }).map_err(|error| error.to_string())?;
                    let components = changed.source_theorem_canonical_components(
                        changed.vcs()[0].id, &core, &formulas
                    )?;
                    let hashes = [components.0.hash(), components.1, components.2.hash()];
                    for (hash, original) in hashes.iter().zip([
                        baseline.0.hash(), baseline.1, baseline.2.hash()
                    ]) {
                        assert_ne!(*hash, original);
                    }
                    let actual = hashes.map(|hash| {
                        hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                    });
                    assert_eq!(&actual, expected);
                }
            }
            assert_eq!(vc.anchor, anchor);
            assert_eq!(core, core_before);
            assert_eq!(formulas, formulas_before);
            assert_eq!(vcs, vcs_before);
            Ok(())
        }).unwrap();
    }
}

#[test]
fn source_theorem_canonical_components_refuse_maps_and_whole_inputs() {
    use mizar_core::{
        core_ir::*,
        elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
    };
    use mizar_vc::{kernel_evidence_handoff::build_source_theorem_kernel_handoff, vc_ir::*};
    let source = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    with_step5c14_theorem_check(source, |check| {
        let core = lower_source_theorem_skeletons(check, None)?;
        let core_before = core.clone();
        let seed = core.obligation_seeds().iter().next().unwrap().1;
        let guard = seed.context[0];
        let goal = seed.goal.unwrap();
        let formulas = source_theorem_formula_payloads(check, &core, &[guard, goal])?;
        let formulas_before = formulas.clone();
        let vcs = generate_core_vcs(&core, snapshot_id(777))?;
        let vcs_before = vcs.clone();
        let selected = vcs.vcs()[0].id;
        let proof = core.proofs().iter().next().unwrap().1;
        let CoreFormulaKind::Forall { binders, .. } =
            &core.formulas().get(proof.proposition).unwrap().kind
        else {
            panic!()
        };
        let original_guard = binders[0].ty_guard.unwrap();
        assert_ne!(original_guard, guard);
        let invalid = "unsupported source theorem canonical components";
        for mutation in 0..7 {
            let mut changed = formulas.clone();
            match mutation {
                0 => changed.clear(),
                1 => {
                    changed.remove(&guard);
                }
                2 => {
                    changed.remove(&goal);
                }
                3 => {
                    changed.get_mut(&guard).unwrap().clear();
                }
                4 => {
                    changed.get_mut(&goal).unwrap().clear();
                }
                5 => {
                    changed.insert(original_guard, formulas[&guard].clone());
                }
                6 => {
                    let bytes = changed.remove(&guard).unwrap();
                    changed.insert(original_guard, bytes);
                }
                _ => unreachable!(),
            }
            assert_eq!(
                vcs.source_theorem_canonical_components(selected, &core, &changed)
                    .unwrap_err(),
                invalid,
                "map mutation {mutation}"
            );
        }
        let foreign = step5c14_computation_core(&source.replace("T:", "Different:"))?;
        let foreign_vcs = generate_core_vcs(&foreign, vcs.snapshot())?;
        for (actual_core, actual_vcs, id) in [
            (&core, &vcs, VcId::new(usize::MAX)),
            (&foreign, &vcs, selected),
            (&core, &foreign_vcs, selected),
        ] {
            let upstream =
                build_source_theorem_kernel_handoff(actual_core, actual_vcs, id).unwrap_err();
            for map in [&formulas, &std::collections::BTreeMap::new()] {
                assert_eq!(
                    actual_vcs
                        .source_theorem_canonical_components(id, actual_core, map)
                        .unwrap_err(),
                    upstream
                );
            }
        }
        for mutation in 0..10 {
            let mut parts = VcSetParts {
                schema_version: vcs.schema_version().clone(),
                snapshot: vcs.snapshot(),
                source: vcs.source(),
                module: vcs.module().clone(),
                generated_formulas: vcs.generated_formulas().to_vec(),
                vcs: vcs.vcs().to_vec(),
                seed_accounting: vcs.seed_accounting().to_vec(),
            };
            let mut entries = parts.vcs[0].local_context.entries().to_vec();
            let mut policies = Vec::new();
            match mutation {
                0 => parts.vcs[0].status = VcStatus::NeedsAtp,
                1 => entries[0].kind = ContextEntryKind::CurrentThesis,
                2 => entries[0].formula = Some(VcFormulaRef::Core(original_guard)),
                3 => entries[0].provenance.clear(),
                4 => {
                    entries[0].provenance[0].core.as_mut().unwrap().key =
                        "foreign-checker-role".into()
                }
                5 => entries[0].sort_key = CanonicalSortKey::new("foreign-source-ordinal"),
                6 => policies.push(VerifierPolicyInput {
                    sort_key: CanonicalSortKey::new("foreign-policy"),
                    key: PolicyKey::new("foreign-policy"),
                    value: PolicyValue::new("enabled"),
                }),
                7 => parts.vcs[0].premises.clear(),
                8 => {
                    parts.vcs[0].proof_hint = Some(ProofHint {
                        citations: Vec::new(),
                        unfold_requests: Vec::new(),
                        premise_restrictions: Vec::new(),
                        solver: None,
                        max_axioms: None,
                        timeout: None,
                        computation: None,
                        provenance: Vec::new(),
                    })
                }
                9 => parts.vcs[0].provenance.clear(),
                _ => unreachable!(),
            }
            parts.vcs[0].local_context = LocalContext::try_new(entries, policies).unwrap();
            let changed = VcSet::try_new(parts).unwrap();
            assert_ne!(changed, vcs, "whole-input mutation {mutation}");
            let upstream =
                build_source_theorem_kernel_handoff(&core, &changed, selected).unwrap_err();
            assert_eq!(
                changed
                    .source_theorem_canonical_components(selected, &core, &formulas)
                    .unwrap_err(),
                upstream,
                "whole-input mutation {mutation}"
            );
        }
        // Explicitly malformed, source-unbacked negative: change only the seed provenance key.
        // Original Core56 bytes below are negative operands, not authenticated to this Core.
        let mut parts = CoreIrParts {
            source_id: core.source_id(),
            module_id: core.module_id().clone(),
            items: core.items().clone(),
            terms: core.terms().clone(),
            formulas: core.formulas().clone(),
            definitions: core.definitions().clone(),
            proofs: core.proofs().clone(),
            proof_nodes: core.proof_nodes().clone(),
            algorithms: core.algorithms().clone(),
            algorithm_statements: core.algorithm_statements().clone(),
            generated: core.generated().clone(),
            obligation_seeds: core.obligation_seeds().clone(),
            source_map: core.source_map().clone(),
            diagnostics: core.diagnostics().clone(),
        };
        let seed_id = core.obligation_seeds().iter().next().unwrap().0;
        let provenance = &mut parts.obligation_seeds.get_mut(seed_id).unwrap().provenance;
        assert_eq!(provenance.len(), 1);
        assert_eq!(provenance[0].phase, CoreProvenancePhase::Checker);
        provenance[0].key = "checker/theorem/foreign/skeleton/terminal".into();
        let malformed = CoreIr::try_new(parts).unwrap();
        assert_ne!(malformed, core);
        assert_eq!(malformed.source_map(), core.source_map());
        let malformed_vcs = generate_core_vcs(&malformed, vcs.snapshot())?;
        let id = malformed_vcs.vcs()[0].id;
        assert!(build_source_theorem_kernel_handoff(&malformed, &malformed_vcs, id).is_ok());
        assert_eq!(
            malformed_vcs
                .source_theorem_canonical_components(id, &malformed, &formulas)
                .unwrap_err(),
            invalid
        );
        assert_eq!(core, core_before);
        assert_eq!(formulas, formulas_before);
        assert_eq!(vcs, vcs_before);
        Ok(())
    })
    .unwrap();
}

#[test]
fn source_theorem_canonical_inputs_match_genuine_content_and_full_slices() {
    use mizar_core::elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads};
    use mizar_proof::status::source_theorem_canonical_inputs;
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        generator::{CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput},
        vc_ir::*,
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    // Independent fixed-schema literals frozen before the facade implementation.
    let expected = [
        "770f0ab609b0f647adef9a5d0a96b8ca81c755bed1fa0c91e4bd9021e5625563",
        "e2753e241cf06d7bbc31e1886154a430469c93bcae47a724e4a21a77e46e45d9",
        "2d1ad8e890a278757aef06df254c0035e637e14ad636fa53e887fddb72db8aaa",
    ];
    for source in [
        original.to_owned(),
        "theorem T: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;".to_owned(),
        format!(":: leading trivia\n{}\n", original.replace(" proof ", "\nproof\n")),
        original.replace("T:", "U:"),
    ] {
        with_step5c14_theorem_check(&source, |check| {
            let check_before = format!("{check:?}");
            let owners_before = check.owners().to_vec();
            assert_eq!(check.owners().len(), 1);
            assert_eq!(check.owners()[0].0.spelling, if source.contains("U:") { "U" } else { "T" });
            let core = lower_source_theorem_skeletons(check, None)?;
            let roots = core.obligation_seeds().iter().flat_map(|(_, seed)| {
                seed.context.iter().copied().chain(seed.goal)
            }).collect::<Vec<_>>();
            let formulas = source_theorem_formula_payloads(check, &core, &roots)?;
            for ordinal in [777, 778] {
                let snapshot = snapshot_id(ordinal);
                // Independently generate the original graph, without relabeling a container.
                let flow = mizar_core::control_flow::build_control_flow_ir(&core);
                let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
                let intake = SeedIntakeTable::try_from_handoff(&handoff)
                    .map_err(|error| error.to_string())?;
                let package = core.module_id().package().as_str();
                let path = core.module_id().path().as_str();
                let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                    schema_version: &GenerationSchemaVersion::new("source-theorem-local-v1"),
                    module: &VcModuleRef::new(format!(
                        "package={}:{};module={}:{}", package.len(), package, path.len(), path
                    )),
                    intake: &intake,
                    handoff: &handoff,
                    flow_output: Some(&flow),
                }).map_err(|error| error.to_string())?;
                let vcs = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
                    schema_version: &VcSchemaVersion::new("vc-v1"),
                    snapshot,
                    source: core.source_id(),
                    candidates: &candidates,
                }).map_err(|error| error.to_string())?;
                assert_eq!(vcs.snapshot(), snapshot);
                assert_eq!(vcs.vcs().len(), 1);
                let vc = &vcs.vcs()[0];
                let anchor_before = vc.anchor.clone();
                assert_eq!(vc.status, VcStatus::Open);
                assert_eq!(vc.kind, VcKind::TheoremProofStep);
                assert!(!vc.anchor.is_complete());
                assert_eq!(vcs.canonical_vc_fingerprint(vc.id), None);
                assert_eq!(vcs.local_context_fingerprint(vc.id), None);
                let components = vcs.source_theorem_canonical_components(vc.id, &core, &formulas)?;
                let slices = try_compute_dependency_slices(DependencySliceInput {
                    vc_set: &vcs,
                    discharge_output: None,
                }).map_err(|error| error.to_string())?;
                let slice = slices.slice_for(vc.id).unwrap();
                let actual = source_theorem_canonical_inputs(check, snapshot, vc.id)?;
                assert_eq!((actual.0, actual.1, actual.2), components);
                let hashes = [actual.0.hash(), actual.1, actual.2.hash()].map(|hash| {
                    hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                });
                assert_eq!(hashes, expected);
                assert_eq!(&actual.3, slice);
                assert_eq!(actual.3.vc(), slice.vc());
                assert_eq!(actual.3.kind(), slice.kind());
                assert_eq!(actual.3.status(), slice.status());
                assert_eq!(actual.3.entries(), slice.entries());
                assert_eq!(actual.3.unknowns(), slice.unknowns());
                assert_eq!(actual.3.completeness(), slice.completeness());
                assert_eq!(actual.3.fingerprint(), slice.fingerprint());
                assert_eq!(actual.3.kind(), &VcKind::TheoremProofStep);
                assert_eq!(actual.3.status(), &VcStatus::Open);
                assert!(!actual.3.entries().is_empty());
                assert!(!actual.3.unknowns().is_empty());
                assert_eq!(actual.3.completeness(), DependencySliceCompleteness::IncompleteUncacheable);
                assert!(!actual.3.is_complete());
                assert!(actual.3.requires_cache_miss());
                assert_eq!(actual, source_theorem_canonical_inputs(check, snapshot, vc.id)?);
                assert_eq!(vc.anchor, anchor_before);
            }
            assert_eq!(check.owners(), owners_before);
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        }).unwrap();
    }
}

#[test]
fn source_theorem_canonical_inputs_preserve_upstream_errors_before_selection() {
    use mizar_core::{
        core_ir::{CoreItemKind, CoreProofNodeKind},
        elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
    };
    use mizar_proof::status::source_theorem_canonical_inputs;
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let absent = VcId::new(usize::MAX);
    with_step5c14_theorem_check(original, |check| {
        let check_before = format!("{check:?}");
        let core = lower_source_theorem_skeletons(check, None)?;
        let roots = core
            .obligation_seeds()
            .iter()
            .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
            .collect::<Vec<_>>();
        let formulas = source_theorem_formula_payloads(check, &core, &roots)?;
        // VC69 checks target absence before schema/replay, so the old generator is sufficient.
        let vcs = generate_core_vcs(&core, snapshot_id(777))?;
        let expected = vcs
            .source_theorem_canonical_components(absent, &core, &formulas)
            .unwrap_err();
        assert_eq!(
            source_theorem_canonical_inputs(check, snapshot_id(777), absent).unwrap_err(),
            expected
        );
        assert_eq!(format!("{check:?}"), check_before);
        Ok(())
    })
    .unwrap();
    for source in [
        "open theorem O: for X being set holds X = X;".to_owned(),
        original.replacen("theorem", "lemma", 1),
        "theorem C: 0 = 0 by computation(steps: 8);".to_owned(),
    ] {
        with_step5c14_theorem_check(&source, |check| {
            let check_before = format!("{check:?}");
            let expected = if source.starts_with("open theorem") {
                let error = lower_source_theorem_skeletons(check, None).unwrap_err();
                assert_eq!(error, "theorems.core.status_unsupported");
                error
            } else {
                let core = lower_source_theorem_skeletons(check, None)?;
                if source.starts_with("lemma") {
                    assert_eq!(
                        core.items().iter().next().unwrap().1.kind,
                        CoreItemKind::Lemma
                    );
                } else {
                    assert!(
                        core.obligation_seeds()
                            .iter()
                            .all(|(_, seed)| seed.context.is_empty())
                    );
                    assert!(core.proof_nodes().iter().any(|(_, node)| {
                        matches!(node.kind, CoreProofNodeKind::ComputationGoal { .. })
                    }));
                }
                let roots = core
                    .obligation_seeds()
                    .iter()
                    .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
                    .collect::<Vec<_>>();
                assert!(!roots.is_empty());
                source_theorem_formula_payloads(check, &core, &roots).unwrap_err()
            };
            for selected in [VcId::new(0), absent] {
                assert_eq!(
                    source_theorem_canonical_inputs(check, snapshot_id(777), selected).unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
    }
}
