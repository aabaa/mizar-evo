use super::*;

#[test]
fn source_candidate_sat_screen_preserves_original_pair_and_fresh_kernel_oracle() {
    use mizar_atp::{
        backend::{
            BackendCandidatePayload, build_source_registration_candidate_inputs as build,
            screen_source_registration_candidate as screen,
        },
        problem::ConcreteFormat,
    };
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus,
    };
    let policy = VerifierPolicy::release().with_instance_search_limits(Some([u64::MAX; 4]));
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        for (renamed_source, base) in [(false, &original), (true, &renamed)] {
            let formal = if renamed_source { "Value" } else { "X" };
            let negative = base.replacen(
                &format!("means {formal} = {formal}"),
                &format!("means not {formal} = {formal}"),
                1,
            );
            assert_ne!(&negative, base);
            for (false_source, text) in [(false, base), (true, &negative)] {
                let (core, _, vcs) = source_candidate_core_and_vcs(multiple, text);
                let before = (core.clone(), vcs.clone());
                for vc in vcs
                    .vcs()
                    .iter()
                    .filter(|vc| vc.status == VcStatus::NeedsAtp)
                {
                    let handoff =
                        build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
                    for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                        let profile = source_candidate_profile("source-sat-screen", format);
                        let original_pair =
                            build(&core, &vcs, vc.id, profile.clone(), format).unwrap();
                        let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) =
                            original_pair.1.payload()
                        else {
                            panic!("actual candidate bytes")
                        };
                        let normal = source_candidate_check_bytes(
                            &handoff,
                            bytes,
                            original_pair.1.target_binding(),
                        )
                        .unwrap();
                        let rejected = false_source && vc.id.index() != 3;
                        assert_eq!(
                            normal,
                            if rejected {
                                KernelCheckStatus::Rejected
                            } else {
                                KernelCheckStatus::Accepted
                            }
                        );
                        let actual = screen(
                            &core,
                            &vcs,
                            vc.id,
                            profile.clone(),
                            format,
                            policy.instance_search_limits(),
                        )
                        .unwrap();
                        assert_eq!(actual, if rejected { None } else { Some(original_pair) });
                        assert_eq!(
                            screen(
                                &core,
                                &vcs,
                                vc.id,
                                profile,
                                format,
                                policy.instance_search_limits()
                            )
                            .unwrap(),
                            actual
                        );
                    }
                }
                assert_eq!((core, vcs), before);
            }
        }
    }
}

#[test]
fn source_candidate_sat_screen_uses_explicit_policy_bounds_and_actual_record_counts() {
    use mizar_atp::{
        backend::{
            build_source_registration_candidate_inputs as build,
            screen_source_registration_candidate as screen,
        },
        problem::ConcreteFormat,
        translator::translate_source_existential_registration,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus,
    };
    let mut substitution_counts = std::collections::BTreeSet::new();
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, _, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let before = (core.clone(), vcs.clone());
        for vc in vcs
            .vcs()
            .iter()
            .filter(|vc| vc.status == VcStatus::NeedsAtp)
        {
            let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
            let envelope = handoff.canonical_evidence();
            let target = envelope.target_vc();
            let mut context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(
                    envelope.kernel_profile().profile_id,
                    ClauseTautologyPolicy::Reject,
                ),
            );
            context.accepted_schema_versions.insert(2);
            let parsed =
                parse_formula_evidence(&source_candidate_wire(&handoff), &context).unwrap();
            substitution_counts.insert(parsed.substitutions().len());
            for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                let profile = source_candidate_profile("source-sat-bounds", format);
                let problem =
                    translate_source_existential_registration(&core, &vcs, vc.id, profile.clone())
                        .unwrap();
                let exact = [
                    u64::try_from(problem.observed_ground_terms().unwrap().len()).unwrap(),
                    u64::try_from(parsed.substitutions().len()).unwrap(),
                    u64::try_from(parsed.equality_instances().len()).unwrap(),
                    1,
                ];
                assert!(exact.into_iter().all(|count| count > 0));
                let pair = build(&core, &vcs, vc.id, profile.clone(), format).unwrap();
                let policy = VerifierPolicy::release();
                assert_eq!(
                    screen(
                        &core,
                        &vcs,
                        vc.id,
                        profile.clone(),
                        format,
                        policy.instance_search_limits()
                    )
                    .unwrap(),
                    None
                );
                for bounds in [exact, [u64::MAX; 4]] {
                    let configured = policy.clone().with_instance_search_limits(Some(bounds));
                    assert_eq!(
                        screen(
                            &core,
                            &vcs,
                            vc.id,
                            profile.clone(),
                            format,
                            configured.instance_search_limits()
                        )
                        .unwrap(),
                        Some(pair.clone())
                    );
                }
                for coordinate in 0..4 {
                    let mut below = exact;
                    below[coordinate] -= 1;
                    let configured = policy.clone().with_instance_search_limits(Some(below));
                    assert_eq!(
                        screen(
                            &core,
                            &vcs,
                            vc.id,
                            profile.clone(),
                            format,
                            configured.instance_search_limits()
                        )
                        .unwrap(),
                        None,
                        "coordinate={coordinate}, exact={exact:?}"
                    );
                }
                let zero = policy.clone().with_instance_search_limits(Some([0; 4]));
                assert_eq!(
                    screen(
                        &core,
                        &vcs,
                        vc.id,
                        profile.clone(),
                        format,
                        zero.instance_search_limits()
                    )
                    .unwrap(),
                    None
                );
            }
        }
        assert_eq!((core, vcs), before);
    }
    assert!(substitution_counts.len() > 1);
}

#[test]
fn source_candidate_sat_screen_preserves_lower_refusals_before_optional_limits() {
    use mizar_atp::{
        backend::{
            build_source_registration_candidate_inputs as build,
            screen_source_registration_candidate as screen,
        },
        problem::*,
    };
    use mizar_core::core_ir::{CoreIr, CoreIrParts};
    use mizar_vc::vc_ir::*;
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, open, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let (foreign, _, foreign_vcs) =
            source_candidate_core_and_vcs(multiple, &text.replace("marked", "foreignmarked"));
        let selected = vcs
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap()
            .id;
        let discharged = vcs
            .vcs()
            .iter()
            .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
            .unwrap()
            .id;
        let absent = VcId::new(vcs.vcs().len());
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
        let id = core.generated().iter().next().unwrap().0;
        parts.generated.get_mut(id).unwrap().evidence.clear();
        let changed_core = CoreIr::try_new(parts).unwrap();
        let unsupported = LogicProfile::try_new(
            "unsupported",
            LogicFragment::TffLike,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::GuardPredicates,
            NativePropertySupport::Unsupported,
            [ConcreteFormat::Tptp].into_iter().collect(),
        )
        .unwrap();
        for mutation in 0..8 {
            let mut parts = VcSetParts {
                schema_version: vcs.schema_version().clone(),
                snapshot: vcs.snapshot(),
                source: vcs.source(),
                module: vcs.module().clone(),
                generated_formulas: vcs.generated_formulas().to_vec(),
                vcs: vcs.vcs().to_vec(),
                seed_accounting: vcs.seed_accounting().to_vec(),
            };
            match mutation {
                0 => parts.vcs[selected.index()].status = VcStatus::Open,
                1 => {
                    parts.vcs[selected.index()].status = vcs.vc(discharged).unwrap().status.clone()
                }
                2 => {
                    parts.vcs[discharged.index()].provenance.pop();
                }
                3 => parts.vcs[discharged.index()].status = VcStatus::Open,
                4 => parts.vcs[selected.index()].source.related.clear(),
                5 => {
                    parts.vcs[selected.index()].premises.pop();
                }
                6 => parts.vcs[selected.index()].goal = vcs.vc(discharged).unwrap().goal,
                _ => parts.generated_formulas[0].shape = VcGeneratedFormulaShape::True,
            }
            let changed_vcs = VcSet::try_new(parts).unwrap();
            for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                let profile = source_candidate_profile("source-sat-refusals", format);
                let other_format = if format == ConcreteFormat::Tptp {
                    ConcreteFormat::SmtLib
                } else {
                    ConcreteFormat::Tptp
                };
                for (c, v, id, p) in [
                    (&core, &open, selected, profile.clone()),
                    (&core, &vcs, discharged, profile.clone()),
                    (&core, &vcs, absent, profile.clone()),
                    (&foreign, &vcs, selected, profile.clone()),
                    (&core, &foreign_vcs, selected, profile.clone()),
                    (&changed_core, &vcs, selected, profile.clone()),
                    (&core, &vcs, selected, unsupported.clone()),
                    (
                        &core,
                        &vcs,
                        selected,
                        source_candidate_profile("wrong-format", other_format),
                    ),
                ]
                .into_iter()
                .filter(|_| mutation == 0)
                .chain([
                    (&core, &changed_vcs, selected, profile.clone()),
                    (&core, &changed_vcs, absent, profile.clone()),
                ]) {
                    let before = (c.clone(), v.clone());
                    let old_error = build(c, v, id, p.clone(), format).unwrap_err();
                    for limits in [None, Some([0; 4]), Some([u64::MAX; 4])] {
                        assert_eq!(
                            screen(c, v, id, p.clone(), format, limits).unwrap_err(),
                            old_error
                        );
                    }
                    assert_eq!((c.clone(), v.clone()), before);
                }
            }
        }
    }
}
