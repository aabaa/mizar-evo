use super::*;

#[test]
fn source_theorem_candidate_inputs_bind_real_encoding_and_evidence() {
    use mizar_atp::{
        backend::{
            BackendCandidateEvidence, BackendCandidatePayload, EncodedBackendProblem,
            build_source_theorem_candidate_inputs as build,
            encode_local_variable_reflexivity_names,
        },
        problem::*,
        translator::translate_source_theorem_sequent,
    };
    use mizar_core::core_ir::{CoreFormulaKind, CoreProofNodeKind};
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
        },
        clause::Term,
        formula_evidence::{
            Formula, FormulaEvidenceParseContext, FormulaSource, parse_formula_evidence,
        },
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::policy::{
        CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, PolicyCandidate,
        ProofPolicyEvaluator, VerifierPolicy,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, KernelContextIdentitySource as SourceContext,
            build_source_theorem_discharge_kernel_handoff as handoff,
        },
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcFormulaRef, VcSchemaVersion, VcStatus},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let baseline = step5c14_computation_core(text).unwrap();
    let module = baseline.module_id();
    for (case, text) in [
        text.to_owned(),
        text.replace(" X", " Local"),
        format!(":: 日本語 λ\n{text}"),
        text.replace("T:", "Renamed:"),
        text.to_owned(),
        text.to_owned(),
        text.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, symbols) =
            super::super::super::tests::source_registration_owner_inputs(
                &text,
                module.package().as_str(),
                module.path().as_str(),
                usize::from(case == 4),
            );
        assert!(frontend.diagnostics.is_empty() && symbols.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        if case == 4 {
            assert_ne!(ast.source_id, baseline.source_id());
        }
        let core = with_source_theorem_check(ast, &source, symbols.env(), |check| {
            mizar_core::elaborator::lower_source_theorem_skeletons(check, None)
        })
        .unwrap();
        assert_eq!(core.source_id(), ast.source_id);
        let snapshot = snapshot_id(if case == 5 { 778 } else { 777 });
        let generation =
            GenerationSchemaVersion::new(if case == 5 { "" } else { "atp46-generation" });
        let schema = VcSchemaVersion::new(if case == 5 { "λ\n\0" } else { "atp46-schema" });
        let vcs = generate_source_theorem_vcs(&core, snapshot, &generation, &schema).unwrap();
        let id = vcs.vcs()[0].id;
        let (label, max_steps) = match case {
            5 => ("", 0),
            6 => ("λ\n\0", u32::MAX),
            _ => ("explicit", 64),
        };
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(label),
                max_steps,
            },
        };
        let output = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        })
        .unwrap();
        let before = (core.clone(), vcs.clone(), output.clone(), policy.clone());
        for (fragment, format) in [
            (LogicFragment::Fof, ConcreteFormat::Tptp),
            (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib),
        ] {
            let profile = LogicProfile::try_new(
                "caller-explicit",
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
            let problem = translate_source_theorem_sequent(
                &core,
                &vcs,
                id,
                &output,
                &policy,
                profile.clone(),
            )
            .unwrap();
            let expected_encoded = EncodedBackendProblem::encode(&problem, format).unwrap();
            let old = handoff(&core, &vcs, id, &output, &policy).unwrap();
            let target = old.canonical_evidence().target_vc();
            let kernel_profile = old.canonical_evidence().kernel_profile();
            let tautology = match kernel_profile.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!(),
            };
            let parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(kernel_profile.profile_id, tautology),
            );
            let parsed =
                parse_formula_evidence(&old.encode_local_evidence(None).unwrap(), &parse_context)
                    .unwrap();
            let names = encode_local_variable_reflexivity_names(&parsed).unwrap();
            assert_eq!(names, [vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]]);
            let wire = old.encode_local_evidence(Some(&names)).unwrap();
            let expected = BackendCandidateEvidence::new(
                format!(
                    "source-theorem-candidate-v1:{}",
                    expected_encoded
                        .metadata_hash()
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                ),
                BackendCandidatePayload::FormulaSubstitutionBytes(wire.clone()),
                expected_encoded.target_binding().clone(),
                expected_encoded.input_hash(),
                expected_encoded.provenance_hash(),
                expected_encoded.formula_labels().to_vec(),
                expected_encoded.symbol_bindings().to_vec(),
            )
            .unwrap();
            let proof = core.proofs().iter().next().unwrap().1;
            let CoreProofNodeKind::IntroduceBinder { binder, .. } =
                &core.proof_nodes().get(proof.root).unwrap().kind
            else {
                panic!()
            };
            let CoreFormulaKind::Forall { binders, .. } =
                &core.formulas().get(proof.proposition).unwrap().kind
            else {
                panic!()
            };
            assert_ne!(binder.var, binders[0].var);
            let Some(AtpFormulaTree::Forall { binders, .. }) = problem.conjecture().formula()
            else {
                panic!()
            };
            assert_eq!(binders.len(), 1);
            assert!(problem.symbol_map().iter().any(|row| row.backend_symbol() == binders[0].variable()
                && matches!(row.source(), AtpSymbolSource::GeneratedBinder(source) if source.as_str() == format!("proof-let:{}", binder.var.index()))));
            let [variable] = parsed.variable_manifest() else {
                panic!()
            };
            let [guard] = parsed.formulas() else { panic!() };
            let (Formula::Atom(guard_atom), Formula::Atom(goal_atom)) =
                (&guard.formula, &parsed.final_goal().formula)
            else {
                panic!()
            };
            let term = Term::Variable(variable.variable_id);
            assert_eq!(guard_atom.arguments.as_slice(), std::slice::from_ref(&term));
            assert_eq!(goal_atom.arguments, [term.clone(), term]);
            let context = &output.vc_set().vc(id).unwrap().local_context.entries()[0];
            assert_eq!(
                guard.source,
                FormulaSource::LocalHypothesis {
                    local_context_id: u32::try_from(context.id.index() + 1).unwrap()
                }
            );
            assert_eq!(
                old.canonical_evidence().formula_evidence()[0].producer_formula_ref(),
                context.formula
            );
            assert_eq!(
                old.canonical_evidence().final_goal().producer_formula_ref,
                output.vc_set().vc(id).unwrap().goal
            );
            let actual = build(&core, &vcs, id, &output, &policy, profile.clone(), format).unwrap();
            assert_eq!(actual, (expected_encoded, expected));
            assert_eq!(
                actual,
                build(&core, &vcs, id, &output, &policy, profile.clone(), format).unwrap()
            );
            let (encoded, candidate) = actual;
            let BackendCandidatePayload::FormulaSubstitutionBytes(returned_wire) =
                candidate.payload()
            else {
                panic!()
            };
            let fresh = handoff(&core, &vcs, id, &output, &policy).unwrap();
            let target = fresh.canonical_evidence().target_vc();
            let kernel_profile = fresh.canonical_evidence().kernel_profile();
            let tautology = match kernel_profile.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!(),
            };
            let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(kernel_profile.profile_id, tautology),
            );
            parse_context.accepted_schema_versions.insert(2);
            let parsed = parse_formula_evidence(returned_wire, &parse_context).unwrap();
            let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
            let entries = fresh
                .context_identity()
                .entries()
                .iter()
                .map(|entry| {
                    let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                        panic!()
                    };
                    let SourceContext::LocalHypothesis { local_context_id } = entry.source() else {
                        panic!()
                    };
                    KernelContextIdentityEntry::new(
                        KernelContextIdentitySource::LocalHypothesis { local_context_id },
                        entry.formula_id(),
                        Fingerprint::new(
                            entry.formula_fingerprint().algorithm_id,
                            entry.formula_fingerprint().digest.clone(),
                        ),
                        KernelFormulaProducerRef::Core(formula),
                    )
                })
                .collect();
            let context = FormulaEvidenceContext::with_context_identity(
                Some(fresh.context_identity_hash().as_bytes().to_vec()),
                vec![],
                vec![],
                Some(KernelContextIdentityPayload::new(
                    target.clone(),
                    fresh.canonical_hash(),
                    fresh.context_identity_hash(),
                    entries,
                )),
                ImportedFactContextLimits::default(),
            )
            .unwrap();
            let encoded_problem_matches = matches!(
                candidate.payload(),
                BackendCandidatePayload::FormulaSubstitutionBytes(_)
            ) && candidate.target_binding()
                == encoded.target_binding()
                && candidate.encoded_problem_hash() == encoded.input_hash()
                && candidate.provenance_hash() == encoded.provenance_hash()
                && candidate.formula_label_refs() == encoded.formula_labels()
                && candidate.symbol_binding_refs() == encoded.symbol_bindings();
            assert!(encoded_problem_matches);
            let evaluator = ProofPolicyEvaluator::new(VerifierPolicy::release());
            let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
            assert!(evaluator.can_schedule_kernel_check(
                &PolicyCandidate::UncheckedFormulaSubstitution {
                    encoded_problem_matches
                }
            ));
            let result = check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: &target,
                evidence: &parsed,
                formula_context: Some(&context),
                check_kind: KernelEvidenceCheckKind::ProofObligation,
                policy: KernelCheckPolicy::default(),
                limits: KernelEvidenceCheckLimits::default(),
            });
            assert_eq!(result.status(), KernelCheckStatus::Accepted);
            assert!(!result.policy_taint() && result.used_axioms().is_empty());
            assert_eq!(result.target_vc_fingerprint(), &target);
            let input = KernelPolicyInput::from_kernel_result(&result, origin);
            assert_eq!(input.origin(), origin);
            assert_eq!(
                evaluator
                    .evaluate_candidate(&PolicyCandidate::KernelResult(input))
                    .class,
                CandidatePolicyClass::KernelVerified
            );
        }
        assert!(vcs.vcs().iter().all(|row| row.status == VcStatus::Open));
        assert!(
            output
                .vc_set()
                .vcs()
                .iter()
                .all(|row| row.status == VcStatus::NeedsAtp)
        );
        assert!(
            core.proofs().iter().all(|(_, row)| row.status
                == mizar_core::core_ir::CoreProofStatus::PendingAutomaticProof)
        );
        assert!(
            core.obligation_seeds()
                .iter()
                .all(|(_, row)| row.status == mizar_core::core_ir::ObligationSeedStatus::Active)
        );
        assert_eq!((core, vcs, output, policy), before);
    }
}

#[test]
fn source_theorem_candidate_inputs_preserve_refusals() {
    use mizar_atp::{
        backend::{EncodedBackendProblem, build_source_theorem_candidate_inputs as build},
        problem::*,
        translator::translate_source_theorem_sequent as translate,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        vc_ir::*,
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let core = step5c14_computation_core(text).unwrap();
    let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
    let id = vcs.vcs()[0].id;
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let output = try_discharge(DischargeInput {
        vc_set: &vcs,
        policy: &policy,
    })
    .unwrap();
    let profile = LogicProfile::try_new(
        "source-refusals",
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
    let before = (
        core.clone(),
        vcs.clone(),
        output.clone(),
        policy.clone(),
        profile.clone(),
    );
    let good = build(
        &core,
        &vcs,
        id,
        &output,
        &policy,
        profile.clone(),
        ConcreteFormat::Tptp,
    )
    .unwrap();
    let other_policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("different"),
            max_steps: 0,
        },
    };
    assert_eq!(
        output,
        try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &other_policy
        })
        .unwrap()
    );
    assert_eq!(
        good,
        build(
            &core,
            &vcs,
            id,
            &output,
            &other_policy,
            profile.clone(),
            ConcreteFormat::Tptp
        )
        .unwrap()
    );
    let problem = translate(&core, &vcs, id, &output, &policy, profile.clone()).unwrap();
    let expected = EncodedBackendProblem::encode(&problem, ConcreteFormat::SmtLib).unwrap_err();
    assert_eq!(
        build(
            &core,
            &vcs,
            id,
            &output,
            &policy,
            profile.clone(),
            ConcreteFormat::SmtLib
        )
        .unwrap_err(),
        expected
    );
    let foreign = step5c14_computation_core(&text.replace("T:", "Foreign:")).unwrap();
    let foreign_vcs = generate_core_vcs(&foreign, snapshot_id(777)).unwrap();
    let foreign_output = try_discharge(DischargeInput {
        vc_set: &foreign_vcs,
        policy: &policy,
    })
    .unwrap();
    assert_ne!(foreign_output, output);
    for (current, original, selected, processed) in [
        (&foreign, &vcs, id, &output),
        (&core, &foreign_vcs, id, &output),
        (&core, &vcs, id, &foreign_output),
        (&core, &vcs, VcId::new(usize::MAX), &output),
        (&core, output.vc_set(), id, &output),
    ] {
        let expected = translate(
            current,
            original,
            selected,
            processed,
            &policy,
            profile.clone(),
        )
        .unwrap_err();
        assert_eq!(
            build(
                current,
                original,
                selected,
                processed,
                &policy,
                profile.clone(),
                ConcreteFormat::SmtLib
            )
            .unwrap_err(),
            expected
        );
    }
    for source in [
        text.replacen("theorem", "lemma", 1),
        text.replace("set", "object"),
    ] {
        let unsupported = step5c14_computation_core(&source).unwrap();
        let original = generate_core_vcs(&unsupported, snapshot_id(777)).unwrap();
        assert!(!original.vcs().is_empty());
        let selected = original.vcs()[0].id;
        let expected = translate(
            &unsupported,
            &original,
            selected,
            &output,
            &policy,
            profile.clone(),
        )
        .unwrap_err();
        assert_eq!(
            build(
                &unsupported,
                &original,
                selected,
                &output,
                &policy,
                profile.clone(),
                ConcreteFormat::SmtLib
            )
            .unwrap_err(),
            expected
        );
    }
    let bad = LogicProfile::try_new(
        "bad-profile",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let expected = translate(&core, &vcs, id, &output, &policy, bad.clone()).unwrap_err();
    assert_eq!(
        build(
            &core,
            &vcs,
            id,
            &output,
            &policy,
            bad,
            ConcreteFormat::SmtLib
        )
        .unwrap_err(),
        expected
    );
    assert_eq!((core, vcs, output, policy, profile), before);
}

#[test]
fn source_theorem_candidate_check_matches_independent_pipeline() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::{
        backend::{BackendCandidatePayload, build_source_theorem_candidate_inputs as build},
        problem::*,
    };
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_core::{
        core_ir::{CoreProofStatus, ObligationSeedStatus},
        elaborator::lower_source_theorem_skeletons as lower,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
        },
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{
            CandidatePolicyClass, KernelEvidenceFormat, KernelEvidenceOrigin, KernelPolicyInput,
            PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
        },
        status::{
            check_source_theorem_candidate as verify, check_source_theorem_reflexivity as legacy,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, KernelContextIdentitySource as SourceContext,
            build_source_theorem_discharge_kernel_handoff as handoff,
        },
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcFormulaRef, VcSchemaVersion,
            VcStatus,
        },
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut first_source = None;
    for (case, text) in [
        original.to_owned(),
        original.replace(" X", " Local"),
        format!(":: 日本語 λ\n{original}\n"),
        original.replace("T:", "Renamed:"),
        original.to_owned(),
        original.to_owned(),
        original.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (package, module, skip) = if case == 4 {
            ("another-package", "nested.owners", 7)
        } else {
            ("owner-test", "owners", 0)
        };
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let env = collected.env();
        let source_before = (source.clone(), env.clone());
        assert_eq!(ast.source_id, source.source_id());
        if case == 0 {
            first_source = Some(source.source_id());
        }
        if case == 4 {
            assert_ne!(Some(source.source_id()), first_source);
        }
        let snapshot = snapshot_id(if case == 5 { 778 } else { 777 });
        let generation =
            GenerationSchemaVersion::new(if case == 5 { "" } else { "proof62-generation" });
        let schema = VcSchemaVersion::new(if case == 5 {
            "λ\n\0"
        } else {
            "proof62-schema"
        });
        let discharge_policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(if case == 6 { "λ\n\0" } else { "explicit" }),
                max_steps: if case == 6 { 0 } else { 64 },
            },
        };
        let policy = if case == 0 {
            VerifierPolicy::default()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution])
        } else if case == 6 {
            VerifierPolicy::release().with_require_kernel_certificates(true)
        } else {
            VerifierPolicy::default()
        };
        let inputs_before = (
            generation.clone(),
            schema.clone(),
            discharge_policy.clone(),
            policy.clone(),
        );
        with_source_theorem_check(ast, &source, env, |check| {
            let check_before = format!("{check:?}");
            let (fragment, format) = if case % 2 == 0 {
                (LogicFragment::Fof, ConcreteFormat::Tptp)
            } else {
                (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib)
            };
            let fresh = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source,
                check.typed_ast(),
                check.scope(),
                env,
                check.labels(),
                check.resolved_labels(),
                None,
            )?;
            assert_eq!(check.owners(), fresh.owners());
            let core = lower(&fresh, None)?;
            let vcs = generate(&core, snapshot, &generation, &schema)?;
            assert_eq!(vcs.vcs().len(), 1);
            let id = vcs.vcs()[0].id;
            let discharge = try_discharge(DischargeInput {
                vc_set: &vcs,
                policy: &discharge_policy,
            })
            .map_err(|error| error.to_string())?;
            let profile = LogicProfile::try_new(
                "proof62-explicit",
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
            let (encoded, candidate) = build(
                &core,
                &vcs,
                id,
                &discharge,
                &discharge_policy,
                profile.clone(),
                format,
            )?;
            let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
            let BackendCandidatePayload::FormulaSubstitutionBytes(wire) = candidate.payload()
            else {
                panic!("actual ATP producer returns formula evidence")
            };
            assert!(!wire.is_empty());
            let encoded_problem_matches = candidate.target_binding() == encoded.target_binding()
                && candidate.encoded_problem_hash() == encoded.input_hash()
                && candidate.provenance_hash() == encoded.provenance_hash()
                && candidate.formula_label_refs() == encoded.formula_labels()
                && candidate.symbol_binding_refs() == encoded.symbol_bindings();
            assert!(encoded_problem_matches);
            let evaluator = ProofPolicyEvaluator::new(policy.clone());
            assert!(evaluator.can_schedule_kernel_check(
                &PolicyCandidate::UncheckedFormulaSubstitution {
                    encoded_problem_matches
                }
            ));
            let processed = handoff(&core, &vcs, id, &discharge, &discharge_policy)?;
            let target = processed.canonical_evidence().target_vc();
            let kernel_profile = processed.canonical_evidence().kernel_profile();
            let tautology = match kernel_profile.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!(),
            };
            let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(kernel_profile.profile_id, tautology),
            );
            parse_context.accepted_schema_versions.insert(2);
            let parsed = parse_formula_evidence(wire, &parse_context).unwrap();
            let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
            let entries = processed
                .context_identity()
                .entries()
                .iter()
                .map(|entry| {
                    let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                        panic!()
                    };
                    let SourceContext::LocalHypothesis { local_context_id } = entry.source() else {
                        panic!()
                    };
                    KernelContextIdentityEntry::new(
                        KernelContextIdentitySource::LocalHypothesis { local_context_id },
                        entry.formula_id(),
                        Fingerprint::new(
                            entry.formula_fingerprint().algorithm_id,
                            entry.formula_fingerprint().digest.clone(),
                        ),
                        KernelFormulaProducerRef::Core(formula),
                    )
                })
                .collect();
            let context = FormulaEvidenceContext::with_context_identity(
                Some(processed.context_identity_hash().as_bytes().to_vec()),
                vec![],
                vec![],
                Some(KernelContextIdentityPayload::new(
                    target.clone(),
                    processed.canonical_hash(),
                    processed.context_identity_hash(),
                    entries,
                )),
                ImportedFactContextLimits::default(),
            )
            .unwrap();
            let result = check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: &target,
                evidence: &parsed,
                formula_context: Some(&context),
                check_kind: KernelEvidenceCheckKind::ProofObligation,
                policy: KernelCheckPolicy::default(),
                limits: KernelEvidenceCheckLimits::default(),
            });
            let input = KernelPolicyInput::from_kernel_result(&result, origin);
            assert_eq!(input.origin(), origin);
            let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input));
            assert_eq!(core.source_id(), ast.source_id);
            assert_eq!(core.module_id(), source.module());
            assert_eq!(core.module_id().package().as_str(), package);
            assert_eq!(core.module_id().path().as_str(), module);
            assert_eq!(vcs.source(), ast.source_id);
            assert_eq!(vcs.snapshot(), snapshot);
            assert_eq!(
                vcs.module().as_str(),
                format!(
                    "package={}:{};module={}:{}",
                    package.len(),
                    package,
                    module.len(),
                    module
                )
            );
            assert_eq!(vcs.schema_version(), &schema);
            assert!(!core.proofs().is_empty() && !vcs.seed_accounting().is_empty());
            assert!(
                core.proofs()
                    .iter()
                    .all(|(_, row)| row.status == CoreProofStatus::PendingAutomaticProof)
            );
            for accounting in vcs.seed_accounting() {
                let SeedOriginRef::ExistingCore { seed } = accounting.origin else {
                    panic!()
                };
                assert_eq!(accounting.seed_status, ObligationSeedStatus::Active);
                assert_eq!(
                    core.obligation_seeds().get(seed).unwrap().status,
                    ObligationSeedStatus::Active
                );
            }
            for row in vcs.vcs() {
                assert_eq!(row.status, VcStatus::Open);
                assert_eq!(row.anchor.generation_schema_version, generation);
                assert!(vcs.seed_accounting_for(row.seed.handoff).is_some());
            }
            assert!(
                discharge
                    .vc_set()
                    .vcs()
                    .iter()
                    .all(|row| row.status == VcStatus::NeedsAtp)
            );
            assert_eq!(discharge.vc_set().vc(id).unwrap().id, id);
            assert_eq!(result.status(), KernelCheckStatus::Accepted);
            assert_eq!(
                result.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!result.policy_taint() && result.used_axioms().is_empty());
            assert_eq!(result.target_vc_fingerprint(), &target);
            assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
            if case == 0 {
                assert_eq!(
                    legacy(&source, check, env, snapshot, id, &policy).unwrap_err(),
                    "theorem.proof.kernel_policy_rejected"
                );
            }
            let expected = (core, vcs, discharge, encoded, candidate, result, decision);
            let actual = verify(
                &source,
                check,
                env,
                snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            assert_eq!(actual, expected);
            assert_eq!(
                verify(
                    &source,
                    check,
                    env,
                    snapshot,
                    id,
                    (&generation, &schema, profile, format),
                    (&discharge_policy, &policy)
                )?,
                expected
            );
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), source_before);
        assert_eq!(
            (generation, schema, discharge_policy, policy),
            inputs_before
        );
    }
}

#[test]
fn source_theorem_candidate_check_preserves_policy_and_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::{backend::build_source_theorem_candidate_inputs as build, problem::*};
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_theorem_candidate as verify, check_source_theorem_reflexivity as legacy,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snapshot = snapshot_id(777);
    let generation = GenerationSchemaVersion::new("proof62-errors");
    let schema = VcSchemaVersion::new("proof62-schema");
    let discharge_policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let normal = VerifierPolicy::default();
    let disabled = normal.clone().with_kernel_evidence_formats([]);
    let builtin = normal
        .clone()
        .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
    let profile = LogicProfile::try_new(
        "proof62-errors",
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
        "proof62-bad",
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
    let inputs_before = (
        generation.clone(),
        schema.clone(),
        discharge_policy.clone(),
        normal.clone(),
        disabled.clone(),
        builtin.clone(),
        profile.clone(),
        bad_profile.clone(),
    );
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (case, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        original.replace("set", "object"),
        format!("{original} {}", original.replace("T:", "U:")),
        cited.to_owned(),
        "theorem C: 0 = 0 by computation(steps: 8);".to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for (id, encoding, format) in [
                (VcId::new(0), &profile, ConcreteFormat::Tptp),
                (VcId::new(usize::MAX), &bad_profile, ConcreteFormat::SmtLib),
                (VcId::new(0), &bad_profile, ConcreteFormat::SmtLib),
                (VcId::new(0), &profile, ConcreteFormat::SmtLib),
            ] {
                if case == 0
                    && id == VcId::new(0)
                    && encoding == &profile
                    && format == ConcreteFormat::Tptp
                {
                    continue;
                }
                let expected = (|| {
                    let fresh = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &source,
                        check.typed_ast(),
                        check.scope(),
                        env,
                        check.labels(),
                        check.resolved_labels(),
                        None,
                    )?;
                    let core = lower(&fresh, None)?;
                    let vcs = generate(&core, snapshot, &generation, &schema)?;
                    let discharge = try_discharge(DischargeInput {
                        vc_set: &vcs,
                        policy: &discharge_policy,
                    })
                    .map_err(|error| error.to_string())?;
                    build(
                        &core,
                        &vcs,
                        id,
                        &discharge,
                        &discharge_policy,
                        encoding.clone(),
                        format,
                    )
                })()
                .unwrap_err();
                for policy in [&normal, &disabled] {
                    assert_eq!(
                        verify(
                            &source,
                            check,
                            env,
                            snapshot,
                            id,
                            (&generation, &schema, encoding.clone(), format),
                            (&discharge_policy, policy)
                        )
                        .unwrap_err(),
                        expected
                    );
                }
            }
            if case == 0 {
                let fresh = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &source,
                    check.typed_ast(),
                    check.scope(),
                    env,
                    check.labels(),
                    check.resolved_labels(),
                    None,
                )?;
                let core = lower(&fresh, None)?;
                let vcs = generate(&core, snapshot, &generation, &schema)?;
                let id = vcs.vcs()[0].id;
                let discharge = try_discharge(DischargeInput {
                    vc_set: &vcs,
                    policy: &discharge_policy,
                })
                .map_err(|error| error.to_string())?;
                let prior = build(
                    &core,
                    &vcs,
                    id,
                    &discharge,
                    &discharge_policy,
                    profile.clone(),
                    ConcreteFormat::Tptp,
                )?;
                let other_policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("different"),
                        max_steps: 0,
                    },
                };
                assert_eq!(
                    try_discharge(DischargeInput {
                        vc_set: &vcs,
                        policy: &other_policy
                    })
                    .unwrap(),
                    discharge
                );
                assert_eq!(
                    build(
                        &core,
                        &vcs,
                        id,
                        &discharge,
                        &other_policy,
                        profile.clone(),
                        ConcreteFormat::Tptp
                    )?,
                    prior
                );
                let old = legacy(&source, check, env, snapshot, id, &builtin)?;
                assert_eq!(old.1.class, CandidatePolicyClass::DischargedBuiltin);
                for policy in [&builtin, &disabled] {
                    assert_eq!(
                        verify(
                            &source,
                            check,
                            env,
                            snapshot,
                            id,
                            (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                            (&discharge_policy, policy)
                        )
                        .unwrap_err(),
                        "theorem.candidate.proof.kernel_policy_rejected"
                    );
                }
                assert_eq!(
                    legacy(&source, check, env, snapshot, id, &disabled).unwrap_err(),
                    "theorem.proof.kernel_policy_rejected"
                );
                let (other_frontend, other_source, other) = inputs(
                    &original.replace("T:", "Foreign:"),
                    "owner-test",
                    "owners",
                    7,
                );
                assert!(other_frontend.diagnostics.is_empty() && other.diagnostics().is_empty());
                assert_ne!(other_source.source_id(), source.source_id());
                let foreign_before = (other_source.clone(), other.env().clone());
                for (current, symbols) in [
                    (&source, other.env()),
                    (&other_source, env),
                    (&other_source, other.env()),
                ] {
                    let expected = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        current,
                        check.typed_ast(),
                        check.scope(),
                        symbols,
                        check.labels(),
                        check.resolved_labels(),
                        None,
                    )
                    .unwrap_err();
                    for (selected, encoding, format, policy) in [
                        (id, &profile, ConcreteFormat::Tptp, &normal),
                        (
                            VcId::new(usize::MAX),
                            &bad_profile,
                            ConcreteFormat::SmtLib,
                            &disabled,
                        ),
                    ] {
                        assert_eq!(
                            verify(
                                current,
                                check,
                                symbols,
                                snapshot,
                                selected,
                                (&generation, &schema, encoding.clone(), format),
                                (&discharge_policy, policy)
                            )
                            .unwrap_err(),
                            expected
                        );
                    }
                }
                with_source_theorem_check(
                    other_frontend.ast.as_ref().unwrap(),
                    &other_source,
                    other.env(),
                    |foreign_check| {
                        let expected = SourceVariableSemanticsChecker::check_theorem_skeletons(
                            &source,
                            foreign_check.typed_ast(),
                            foreign_check.scope(),
                            env,
                            foreign_check.labels(),
                            foreign_check.resolved_labels(),
                            None,
                        )
                        .unwrap_err();
                        assert_eq!(
                            verify(
                                &source,
                                foreign_check,
                                env,
                                snapshot,
                                VcId::new(usize::MAX),
                                (
                                    &generation,
                                    &schema,
                                    bad_profile.clone(),
                                    ConcreteFormat::SmtLib
                                ),
                                (&discharge_policy, &disabled)
                            )
                            .unwrap_err(),
                            expected
                        );
                        Ok(())
                    },
                )?;
                assert_eq!((other_source, other.env().clone()), foreign_before);
                assert_eq!(legacy(&source, check, env, snapshot, id, &builtin)?, old);
                assert_eq!(
                    build(
                        &core,
                        &vcs,
                        id,
                        &discharge,
                        &discharge_policy,
                        profile.clone(),
                        ConcreteFormat::Tptp
                    )?,
                    prior
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(
        (
            generation,
            schema,
            discharge_policy,
            normal,
            disabled,
            builtin,
            profile,
            bad_profile
        ),
        inputs_before
    );
}

#[test]
fn source_theorem_candidate_selection_matches_fresh_pipeline() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::problem::*;
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{
            CandidatePolicyClass, KernelEvidenceFormat, KernelEvidenceOrigin, KernelPolicyInput,
            VerifierPolicy,
        },
        selection::{
            CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass,
            ProofWitnessPublication, select_winner,
        },
        status::{
            check_source_theorem_candidate as verify, select_source_theorem_candidate as select,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, SeedOriginRef, VcId, VcSchemaVersion, VcStatus,
        },
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut first_source = None;
    let mut prior = Vec::new();
    for (case, text) in [
        original.to_owned(),
        original.replace(" X", " Local"),
        format!(":: 日本語 λ\n{original}\n"),
        original.replace("T:", "Renamed:"),
        original.to_owned(),
        original.to_owned(),
        original.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (package, module, skip) = if case == 4 {
            ("other-package", "nested.owners", 7)
        } else {
            ("owner-test", "owners", 0)
        };
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let env = collected.env();
        assert_eq!(ast.source_id, source.source_id());
        if case == 0 {
            first_source = Some(source.source_id());
        }
        if case == 4 {
            assert_ne!(Some(source.source_id()), first_source);
        }
        let snapshot = snapshot_id(if case == 5 { 778 } else { 777 });
        let generation =
            GenerationSchemaVersion::new(if case == 5 { "" } else { "proof63-generation" });
        let schema = VcSchemaVersion::new(if case == 5 {
            "λ\n\0"
        } else {
            "proof63-schema"
        });
        let discharge_policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(if case == 6 { "unused λ\0" } else { "explicit" }),
                max_steps: if case == 6 { 0 } else { 64 },
            },
        };
        let policy = match case {
            0 => VerifierPolicy::default()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            1 => VerifierPolicy::default().with_profile_id("current λ\0"),
            2 => VerifierPolicy::default().with_schema_version(2),
            3 => VerifierPolicy::default().with_checker_schema_version(2),
            4 => VerifierPolicy::default().with_instance_search_limits(Some([0, 1, u64::MAX, 3])),
            _ => VerifierPolicy::release(),
        };
        let (fragment, format) = if case % 2 == 0 {
            (LogicFragment::Fof, ConcreteFormat::Tptp)
        } else {
            (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib)
        };
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
        let source_before = (source.clone(), env.clone(), prior.clone());
        let config_before = (
            generation.clone(),
            schema.clone(),
            discharge_policy.clone(),
            profile.clone(),
            policy.clone(),
        );
        with_source_theorem_check(ast, &source, env, |check| {
            let check_before = format!("{check:?}");
            let id = VcId::new(0);
            let raw = verify(
                &source,
                check,
                env,
                snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            let raw_before = raw.clone();
            assert_eq!(raw.0.source_id(), ast.source_id);
            assert_eq!(raw.0.module_id(), source.module());
            assert_eq!(raw.0.module_id().package().as_str(), package);
            assert_eq!(raw.0.module_id().path().as_str(), module);
            assert_eq!(raw.1.source(), ast.source_id);
            assert_eq!(raw.1.snapshot(), snapshot);
            assert_eq!(raw.1.schema_version(), &schema);
            let [vc] = raw.1.vcs() else { panic!() };
            assert_eq!(vc.id, id);
            assert_eq!(vc.anchor.generation_schema_version, generation);
            assert_eq!(vc.status, VcStatus::Open);
            assert!(!raw.0.proofs().is_empty() && !raw.1.seed_accounting().is_empty());
            assert!(
                raw.0
                    .proofs()
                    .iter()
                    .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
            );
            for accounting in raw.1.seed_accounting() {
                let SeedOriginRef::ExistingCore { seed } = accounting.origin else {
                    panic!()
                };
                assert_eq!(accounting.seed_status, ObligationSeedStatus::Active);
                assert_eq!(
                    raw.0.obligation_seeds().get(seed).unwrap().status,
                    ObligationSeedStatus::Active
                );
            }
            assert!(
                raw.2
                    .vc_set()
                    .vcs()
                    .iter()
                    .all(|vc| vc.status == VcStatus::NeedsAtp)
            );
            assert_eq!(raw.2.vc_set().vc(id).unwrap().id, id);
            assert_eq!(raw.3.target_binding(), raw.4.target_binding());
            assert_eq!(raw.3.concrete_format(), format);
            assert_eq!(raw.3.logic_profile_name(), profile.name().as_str());
            assert_eq!(raw.5.status(), KernelCheckStatus::Accepted);
            assert_eq!(
                raw.5.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!raw.5.policy_taint());
            assert_eq!(raw.6.class, CandidatePolicyClass::KernelVerified);
            let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
            let kernel_input = KernelPolicyInput::from_kernel_result(&raw.5, origin);
            assert_eq!(kernel_input.origin(), origin);
            let candidate_id = CandidateSourceId::new(raw.4.candidate_id()).unwrap();
            let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
                candidate_id.clone(),
                &kernel_input,
            )
            .unwrap()
            .with_provenance_hash(raw.4.provenance_hash());
            let evidence = ProofEvidenceSet::new(
                raw.5.target_vc_fingerprint().sort_bytes(),
                raw.3.input_hash(),
                policy.clone(),
            )
            .with_candidates([candidate]);
            let expected_selection = select_winner(&evidence);
            let expected = (
                raw.0.clone(),
                raw.1.clone(),
                raw.2.clone(),
                raw.3.clone(),
                raw.4.clone(),
                raw.5.clone(),
                raw.6.clone(),
                expected_selection.clone(),
            );
            let expected_before = expected.clone();
            let actual = select(
                &source,
                check,
                env,
                snapshot,
                vc.id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            assert_eq!(actual, expected);
            let selected = &actual.7;
            assert_eq!(selected.selected_candidate_id(), Some(&candidate_id));
            assert_eq!(selected.selected_class(), ProofWinnerClass::KernelVerified);
            assert!(selected.trusted_used_axioms_available());
            let meta = selected.reuse_metadata();
            assert_eq!(meta.selected_class(), ProofWinnerClass::KernelVerified);
            assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
            assert_eq!(meta.encoded_problem_hash(), Some(raw.3.input_hash()));
            assert_eq!(
                meta.selected_evidence_hash(),
                kernel_input.accepted_evidence_hash()
            );
            assert!(meta.selected_evidence_hash().is_some());
            assert_eq!(
                meta.accepted_goal_polarity(),
                kernel_input.accepted_goal_polarity()
            );
            assert!(meta.accepted_goal_polarity().is_some());
            assert_eq!(
                meta.selected_candidate_provenance_hash(),
                Some(raw.4.provenance_hash())
            );
            assert_eq!(meta.deterministic_discharge_hash(), None);
            assert_eq!(meta.external_admission_status(), None);
            assert_eq!(meta.selected_proof_witness_hash(), None);
            assert_eq!(
                meta.proof_witness_publication(),
                ProofWitnessPublication::NotApplicable
            );
            assert_eq!(raw, raw_before);
            assert_eq!(expected, expected_before);
            assert_eq!(format!("{check:?}"), check_before);
            prior.push(actual);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&source_before.0, &source_before.1));
        assert_eq!(&prior[..source_before.2.len()], source_before.2);
        assert_eq!(
            (generation, schema, discharge_policy, profile, policy),
            config_before
        );
    }
}

#[test]
fn source_theorem_candidate_selection_preserves_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::problem::*;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_theorem_candidate as verify, select_source_theorem_candidate as select,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snapshot = snapshot_id(777);
    let generation = GenerationSchemaVersion::new("proof63-errors");
    let schema = VcSchemaVersion::new("λ\n\0");
    let discharge_policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let normal = VerifierPolicy::default();
    let disabled = normal.clone().with_kernel_evidence_formats([]);
    let builtin = normal
        .clone()
        .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
    let profile = LogicProfile::try_new(
        "explicit",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let unsupported = LogicProfile::try_new(
        "unsupported",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let config_before = (
        generation.clone(),
        schema.clone(),
        discharge_policy.clone(),
        normal.clone(),
        disabled.clone(),
        builtin.clone(),
        profile.clone(),
        unsupported.clone(),
    );
    for (case, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        original.replace("set", "object"),
        format!("{original} {}", original.replace("T:", "U:")),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            let successful = if case == 0 {
                Some(verify(
                    &source,
                    check,
                    env,
                    snapshot,
                    VcId::new(0),
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &normal),
                )?)
            } else {
                None
            };
            let successful_before = successful.clone();
            let mut first_error = None;
            let config_refusals = [
                (
                    VcId::new(usize::MAX),
                    &profile,
                    ConcreteFormat::Tptp,
                    &normal,
                ),
                (
                    VcId::new(usize::MAX),
                    &unsupported,
                    ConcreteFormat::SmtLib,
                    &disabled,
                ),
                (VcId::new(0), &unsupported, ConcreteFormat::Tptp, &normal),
                (VcId::new(0), &profile, ConcreteFormat::SmtLib, &normal),
                (VcId::new(0), &profile, ConcreteFormat::Tptp, &disabled),
                (VcId::new(0), &profile, ConcreteFormat::Tptp, &builtin),
            ];
            let source_refusal = [(VcId::new(0), &profile, ConcreteFormat::Tptp, &normal)];
            let refusals = if case == 0 {
                config_refusals.as_slice()
            } else {
                source_refusal.as_slice()
            };
            for &(selected, encoding, format, policy) in refusals {
                let expected = verify(
                    &source,
                    check,
                    env,
                    snapshot,
                    selected,
                    (&generation, &schema, encoding.clone(), format),
                    (&discharge_policy, policy),
                )
                .unwrap_err();
                if selected == VcId::new(usize::MAX) {
                    if let Some(previous) = &first_error {
                        assert_eq!(&expected, previous);
                    } else {
                        first_error = Some(expected.clone());
                    }
                }
                let actual = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    selected,
                    (&generation, &schema, encoding.clone(), format),
                    (&discharge_policy, policy),
                )
                .unwrap_err();
                assert_eq!(actual, expected);
            }
            if case == 0 {
                let (other_frontend, other_source, other) = inputs(
                    &original.replace("T:", "Foreign:"),
                    "owner-test",
                    "owners",
                    7,
                );
                assert!(other_frontend.diagnostics.is_empty() && other.diagnostics().is_empty());
                assert_ne!(other_source.source_id(), source.source_id());
                let foreign_before = (other_source.clone(), other.env().clone());
                for (current, symbols) in [
                    (&source, other.env()),
                    (&other_source, env),
                    (&other_source, other.env()),
                ] {
                    let expected = verify(
                        current,
                        check,
                        symbols,
                        snapshot,
                        VcId::new(0),
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge_policy, &normal),
                    )
                    .unwrap_err();
                    assert_eq!(
                        select(
                            current,
                            check,
                            symbols,
                            snapshot,
                            VcId::new(0),
                            (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                            (&discharge_policy, &normal)
                        )
                        .unwrap_err(),
                        expected
                    );
                }
                with_source_theorem_check(
                    other_frontend.ast.as_ref().unwrap(),
                    &other_source,
                    other.env(),
                    |foreign_check| {
                        let expected = verify(
                            &source,
                            foreign_check,
                            env,
                            snapshot,
                            VcId::new(0),
                            (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                            (&discharge_policy, &normal),
                        )
                        .unwrap_err();
                        assert_eq!(
                            select(
                                &source,
                                foreign_check,
                                env,
                                snapshot,
                                VcId::new(0),
                                (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                                (&discharge_policy, &normal)
                            )
                            .unwrap_err(),
                            expected
                        );
                        Ok(())
                    },
                )?;
                assert_eq!((other_source, other.env().clone()), foreign_before);
            }
            assert_eq!(successful, successful_before);
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(
        (
            generation,
            schema,
            discharge_policy,
            normal,
            disabled,
            builtin,
            profile,
            unsupported
        ),
        config_before
    );
}

#[test]
fn source_theorem_candidate_status_matches_existing_projection() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::problem::*;
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{
            CandidatePolicyClass, KernelEvidenceFormat, KernelEvidenceOrigin, KernelPolicyInput,
            VerifierPolicy,
        },
        selection::{
            ProofSelectionSource, ProofWinnerClass, ProofWitnessPublication, VcProofSelection,
            merge_artifact_proof_selections as merge,
        },
        status::{
            ArtifactPublicationGap, ArtifactStatusPublication, ProjectedProofStatus,
            ProofStatusProjectionInput, TrustedUsedAxiomsRef,
            project_source_theorem_candidate_status as project, project_status,
            select_source_theorem_candidate as select,
            source_theorem_obligation_identity_with_schemas as identity,
        },
    };
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness,
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion, VcStatus},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut first_source = None;
    let mut prior = Vec::new();
    for (case, text) in [
        original.to_owned(),
        original.replace(" X", " Local"),
        format!(":: 日本語 λ\n{original}\n"),
        original.replace("T:", "Renamed:"),
        original.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (package, module, skip) = match case {
            2 => ("other-package", "nested.owners", 0),
            4 => ("owner-test", "owners", 7),
            _ => ("owner-test", "owners", 0),
        };
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let env = collected.env();
        assert_eq!(ast.source_id, source.source_id());
        if case == 0 {
            first_source = Some(source.source_id());
        }
        if skip == 7 {
            assert_ne!(Some(source.source_id()), first_source);
        }
        let snapshot = snapshot_id(if case == 3 { 778 } else { 777 });
        let generation = GenerationSchemaVersion::new(if case == 3 { "" } else { "λ\n\0" });
        let schema = VcSchemaVersion::new(if case == 3 { "vc\t\0" } else { "" });
        let discharge_policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(if case == 4 {
                    "unused計算\0"
                } else {
                    "explicit"
                }),
                max_steps: if case == 4 { 0 } else { 64 },
            },
        };
        let policy = match case {
            0 => VerifierPolicy::default()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            1 => VerifierPolicy::default().with_profile_id("current λ\0"),
            2 => VerifierPolicy::default().with_instance_search_limits(Some([0, 1, u64::MAX, 3])),
            _ => VerifierPolicy::release(),
        };
        let (fragment, format) = if case % 2 == 0 {
            (LogicFragment::Fof, ConcreteFormat::Tptp)
        } else {
            (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib)
        };
        let profile = LogicProfile::try_new(
            "status λ\n\0",
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
        let source_before = (source.clone(), env.clone(), prior.clone());
        let config_before = (
            generation.clone(),
            schema.clone(),
            discharge_policy.clone(),
            profile.clone(),
            policy.clone(),
        );
        with_source_theorem_check(ast, &source, env, |check| {
            let check_before = format!("{check:?}");
            let id = VcId::new(0);
            let original_identity =
                identity(&source, check, env, snapshot, id, (&generation, &schema))?;
            let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
            let raw = select(
                &source,
                check,
                env,
                snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            let input = KernelPolicyInput::from_kernel_result(&raw.5, origin);
            let reference = TrustedUsedAxiomsRef::from_kernel_result(&raw.5, origin).unwrap();
            assert_eq!(raw.5.status(), KernelCheckStatus::Accepted);
            assert_eq!(
                raw.5.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!raw.5.policy_taint());
            assert_eq!(raw.6.class, CandidatePolicyClass::KernelVerified);
            assert_eq!(raw.7.selected_class(), ProofWinnerClass::KernelVerified);
            assert_eq!(
                raw.7.selected_candidate_id().unwrap().as_str(),
                raw.4.candidate_id()
            );
            assert_eq!(
                raw.7.reuse_metadata().encoded_problem_hash(),
                Some(raw.3.input_hash())
            );
            assert_eq!(
                raw.7.reuse_metadata().selected_candidate_provenance_hash(),
                Some(raw.4.provenance_hash())
            );
            assert_eq!(
                raw.7.reuse_metadata().proof_witness_publication(),
                ProofWitnessPublication::NotApplicable
            );
            assert_eq!(raw.7.reuse_metadata().selected_proof_witness_hash(), None);
            assert!(raw.5.used_axioms().is_empty());
            assert_eq!(reference.used_axiom_count(), raw.5.used_axioms().len());
            assert_eq!(
                Some(reference.accepted_evidence_hash()),
                input.accepted_evidence_hash()
            );
            let [merged] =
                <[_; 1]>::try_from(merge([VcProofSelection::new(id, raw.7.clone())], []).unwrap())
                    .unwrap();
            assert_eq!(merged.source(), ProofSelectionSource::Portfolio);
            let without_reference = project_status(ProofStatusProjectionInput::new(
                merged.clone(),
                policy.clone(),
                original_identity.0.clone(),
            ))
            .unwrap();
            let expected_projection = project_status(
                ProofStatusProjectionInput::new(
                    merged,
                    policy.clone(),
                    original_identity.0.clone(),
                )
                .with_trusted_used_axioms(reference.clone()),
            )
            .unwrap();
            assert_eq!(without_reference.trusted_used_axioms(), None);
            assert_eq!(
                without_reference
                    .reuse_metadata()
                    .trusted_used_axioms_hash(),
                None
            );
            assert_ne!(
                without_reference
                    .reuse_metadata()
                    .proof_reuse_validation_hash(),
                expected_projection
                    .reuse_metadata()
                    .proof_reuse_validation_hash()
            );
            let before = (
                original_identity.clone(),
                raw.clone(),
                reference.clone(),
                expected_projection.clone(),
            );
            let actual = project(
                &source,
                check,
                env,
                snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            assert_eq!(
                actual,
                (expected_projection.clone(), original_identity.1.clone())
            );
            assert_eq!(actual.0.vc(), id);
            assert_eq!(
                actual.0.obligation_id(),
                original_identity.0.obligation_id()
            );
            assert_eq!(actual.0.selected_class(), ProofWinnerClass::KernelVerified);
            assert_eq!(actual.0.projected_status(), ProjectedProofStatus::Accepted);
            assert_eq!(
                actual.0.artifact_publication(),
                ArtifactStatusPublication::ExternalDependencyGap(
                    ArtifactPublicationGap::MissingKernelVerifiedWitness
                )
            );
            assert_eq!(actual.0.accepted_witness_obligation_id(), None);
            assert_eq!(actual.0.trusted_used_axioms(), Some(&reference));
            let meta = actual.0.reuse_metadata();
            let chosen = raw.7.reuse_metadata();
            let original = &original_identity.0;
            assert_eq!(meta.obligation_anchor(), original.obligation_anchor());
            assert_eq!(
                meta.obligation_fingerprint(),
                original.obligation_fingerprint()
            );
            assert_eq!(meta.vc_fingerprint(), original.vc_fingerprint());
            assert_eq!(
                meta.local_context_fingerprint(),
                original.local_context_fingerprint()
            );
            assert_eq!(
                meta.dependency_slice_fingerprint(),
                original.dependency_slice_fingerprint()
            );
            assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
            assert_eq!(meta.selected_candidate_id(), raw.7.selected_candidate_id());
            assert_eq!(
                meta.selected_evidence_hash(),
                Some(reference.accepted_evidence_hash())
            );
            assert_eq!(
                meta.selected_evidence_hash(),
                chosen.selected_evidence_hash()
            );
            assert_eq!(
                meta.accepted_goal_polarity(),
                input.accepted_goal_polarity()
            );
            assert_eq!(
                meta.selected_candidate_provenance_hash(),
                chosen.selected_candidate_provenance_hash()
            );
            assert_eq!(meta.selection_reason(), chosen.selection_reason());
            assert_eq!(meta.tie_break_key_hash(), chosen.tie_break_key_hash());
            assert_eq!(
                meta.trusted_used_axioms_hash(),
                Some(reference.used_axioms_hash())
            );
            assert_eq!(meta.selected_proof_witness_hash(), None);
            assert_eq!(meta.deterministic_discharge_hash(), None);
            assert_eq!(meta.dependency_compatibility(), None);
            assert_eq!(meta.explanation_ref(), None);
            assert!(!meta.cache_reuse_predicate_complete());
            assert_eq!(raw.0.source_id(), ast.source_id);
            assert_eq!(raw.0.module_id(), source.module());
            assert_eq!(raw.1.source(), ast.source_id);
            assert_eq!(raw.1.snapshot(), snapshot);
            assert_eq!(raw.1.schema_version(), &schema);
            assert_eq!(
                raw.1.vc(id).unwrap().anchor.generation_schema_version,
                generation
            );
            assert!(
                raw.0
                    .proofs()
                    .iter()
                    .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
            );
            assert!(
                raw.0
                    .obligation_seeds()
                    .iter()
                    .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
            );
            assert!(raw.1.vcs().iter().all(|vc| vc.status == VcStatus::Open));
            assert!(
                raw.2
                    .vc_set()
                    .vcs()
                    .iter()
                    .all(|vc| vc.status == VcStatus::NeedsAtp)
            );
            assert_eq!(actual.1.status(), &VcStatus::Open);
            assert!(!actual.1.entries().is_empty() && !actual.1.unknowns().is_empty());
            assert_eq!(
                actual.1.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert_eq!(
                actual.1.fingerprint().hash(),
                meta.dependency_slice_fingerprint()
            );
            assert!(actual.1.requires_cache_miss());
            assert_eq!(
                (original_identity, raw, reference, expected_projection),
                before
            );
            assert_eq!(format!("{check:?}"), check_before);
            prior.push(actual);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&source_before.0, &source_before.1));
        assert_eq!(&prior[..source_before.2.len()], source_before.2);
        assert_eq!(
            (generation, schema, discharge_policy, profile, policy),
            config_before
        );
    }
}

#[test]
fn source_theorem_candidate_status_preserves_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::problem::*;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            project_source_theorem_candidate_status as project,
            select_source_theorem_candidate as select,
            source_theorem_obligation_identity_with_schemas as identity,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snapshot = snapshot_id(777);
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let discharge_policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let normal = VerifierPolicy::default();
    let disabled = normal.clone().with_kernel_evidence_formats([]);
    let builtin = normal
        .clone()
        .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
    let profile = LogicProfile::try_new(
        "explicit",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let unsupported = LogicProfile::try_new(
        "unsupported",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let config_before = (
        generation.clone(),
        schema.clone(),
        discharge_policy.clone(),
        normal.clone(),
        disabled.clone(),
        builtin.clone(),
        profile.clone(),
        unsupported.clone(),
    );
    for (case, text) in [
        original.to_owned(),
        "open theorem O: for X being set holds X = X;".to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            let id = VcId::new(0);
            if case == 0 {
                let original_identity =
                    identity(&source, check, env, snapshot, id, (&generation, &schema))?;
                let raw = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &normal),
                )?;
                let baseline_before = (original_identity.clone(), raw.clone());
                let identity_error = identity(
                    &source,
                    check,
                    env,
                    snapshot,
                    VcId::new(usize::MAX),
                    (&generation, &schema),
                )
                .unwrap_err();
                let policy_error = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &disabled),
                )
                .unwrap_err();
                assert_ne!(identity_error, policy_error);
                assert_eq!(
                    project(
                        &source,
                        check,
                        env,
                        snapshot,
                        VcId::new(usize::MAX),
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge_policy, &disabled)
                    )
                    .unwrap_err(),
                    identity_error
                );
                for (encoding, format, policy) in [
                    (&unsupported, ConcreteFormat::Tptp, &normal),
                    (&profile, ConcreteFormat::SmtLib, &normal),
                    (&profile, ConcreteFormat::Tptp, &disabled),
                    (&profile, ConcreteFormat::Tptp, &builtin),
                ] {
                    let expected = select(
                        &source,
                        check,
                        env,
                        snapshot,
                        id,
                        (&generation, &schema, encoding.clone(), format),
                        (&discharge_policy, policy),
                    )
                    .unwrap_err();
                    assert_eq!(
                        project(
                            &source,
                            check,
                            env,
                            snapshot,
                            id,
                            (&generation, &schema, encoding.clone(), format),
                            (&discharge_policy, policy)
                        )
                        .unwrap_err(),
                        expected
                    );
                }
                for (text, recovered) in [
                    (original.replace("T:", "Foreign:"), false),
                    ("theorem T: for X being set holds X = ;".to_owned(), true),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 7);
                    assert_eq!(!other_frontend.diagnostics.is_empty(), recovered);
                    assert_ne!(source.source_id(), other_source.source_id());
                    let foreign_before = (other_source.clone(), other.env().clone());
                    let (current, symbols) = if recovered {
                        (&other_source, other.env())
                    } else {
                        (&source, other.env())
                    };
                    let expected = identity(
                        current,
                        check,
                        symbols,
                        snapshot,
                        id,
                        (&generation, &schema),
                    )
                    .unwrap_err();
                    assert_eq!(
                        project(
                            current,
                            check,
                            symbols,
                            snapshot,
                            id,
                            (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                            (&discharge_policy, &normal)
                        )
                        .unwrap_err(),
                        expected
                    );
                    assert_eq!((other_source, other.env().clone()), foreign_before);
                }
                assert_eq!((original_identity, raw), baseline_before);
            } else {
                let expected = identity(&source, check, env, snapshot, id, (&generation, &schema))
                    .unwrap_err();
                assert_eq!(
                    project(
                        &source,
                        check,
                        env,
                        snapshot,
                        id,
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge_policy, &normal)
                    )
                    .unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(
        (
            generation,
            schema,
            discharge_policy,
            normal,
            disabled,
            builtin,
            profile,
            unsupported
        ),
        config_before
    );
}

#[test]
fn source_theorem_candidate_metadata_matches_checked_pipeline() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_artifact::{
        proof_witness::KernelAcceptanceMetadata,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::SchemaVersion,
    };
    use mizar_atp::{
        backend::{BackendCandidatePayload, encode_local_variable_reflexivity_names},
        problem::*,
    };
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{KernelCheckStatus, KernelEvidenceCheckKind},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceFormat, KernelEvidenceOrigin,
            KernelPolicyInput, VerifierPolicy,
        },
        selection::{ProofWinnerClass, ProofWitnessPublication},
        status::{
            ArtifactPublicationGap, ArtifactStatusPublication,
            check_source_theorem_candidate_metadata as metadata,
            project_source_theorem_candidate_status as status,
            select_source_theorem_candidate as select,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_theorem_discharge_kernel_handoff as handoff,
        },
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion, VcStatus},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut first_source = None;
    let mut prior = Vec::new();
    for (case, text) in [
        original.to_owned(),
        original.replace(" X", " Local"),
        format!(":: 日本語 λ\n{original}\n"),
        original.replace("T:", "Renamed:"),
        original.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (package, module, skip) = match case {
            2 => ("other-package", "nested.owners", 0),
            4 => ("owner-test", "owners", 7),
            _ => ("owner-test", "owners", 0),
        };
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let env = collected.env();
        assert_eq!(ast.source_id, source.source_id());
        if case == 0 {
            first_source = Some(source.source_id());
        }
        if skip == 7 {
            assert_ne!(Some(source.source_id()), first_source);
        }
        let snapshot = snapshot_id(if case == 3 { 778 } else { 777 });
        let generation = GenerationSchemaVersion::new(if case == 3 { "" } else { "λ\n\0" });
        let schema = VcSchemaVersion::new(if case == 3 { "vc\t\0" } else { "" });
        let discharge_policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(if case == 4 {
                    "unused計算\0"
                } else {
                    "explicit"
                }),
                max_steps: if case == 4 { 0 } else { 64 },
            },
        };
        let policy = match case {
            0 => VerifierPolicy::default()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            1 => VerifierPolicy::default().with_profile_id("current λ\0"),
            2 => VerifierPolicy::default()
                .with_schema_version(2)
                .with_checker_schema_version(7),
            3 => VerifierPolicy::release(),
            _ => VerifierPolicy::default().with_instance_search_limits(Some([0, 1, u64::MAX, 3])),
        };
        let (fragment, format) = if case % 2 == 0 {
            (LogicFragment::Fof, ConcreteFormat::Tptp)
        } else {
            (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib)
        };
        let profile = LogicProfile::try_new(
            "metadata λ\n\0",
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
        let before = (source.clone(), env.clone(), prior.clone());
        let config_before = (
            generation.clone(),
            schema.clone(),
            discharge_policy.clone(),
            profile.clone(),
            policy.clone(),
        );
        with_source_theorem_check(ast, &source, env, |check| {
            let check_before = format!("{check:?}");
            let id = VcId::new(0);
            let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
            let raw = select(
                &source,
                check,
                env,
                snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            let BackendCandidatePayload::FormulaSubstitutionBytes(wire) = raw.4.payload() else {
                panic!()
            };
            assert_eq!(raw.4.target_binding(), raw.3.target_binding());
            assert_eq!(raw.4.encoded_problem_hash(), raw.3.input_hash());
            assert_eq!(raw.4.provenance_hash(), raw.3.provenance_hash());
            assert_eq!(raw.4.formula_label_refs(), raw.3.formula_labels());
            assert_eq!(raw.4.symbol_binding_refs(), raw.3.symbol_bindings());
            assert_eq!(
                raw.7.selected_candidate_id().unwrap().as_str(),
                raw.4.candidate_id()
            );
            let processed = handoff(&raw.0, &raw.1, id, &raw.2, &discharge_policy)?;
            let target = processed.canonical_evidence().target_vc();
            let kernel_profile = processed.canonical_evidence().kernel_profile();
            let tautology = match kernel_profile.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!(),
            };
            let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(kernel_profile.profile_id, tautology),
            );
            parse_context.accepted_schema_versions.insert(2);
            let parsed = parse_formula_evidence(wire, &parse_context).unwrap();
            assert_eq!(parsed.schema_version(), 2);
            assert_eq!(parsed.target_vc(), &parse_context.expected_target_vc);
            let v1 = processed.encode_local_evidence(None)?;
            let v1_parsed = parse_formula_evidence(&v1, &parse_context).unwrap();
            assert_eq!(v1_parsed.schema_version(), 1);
            let names = encode_local_variable_reflexivity_names(&v1_parsed)?;
            assert!(!names.is_empty());
            assert_eq!(parsed.equality_instances().len(), names.len());
            assert_ne!(wire, &v1);
            assert_eq!(wire, &processed.encode_local_evidence(Some(&names))?);
            for name in &names {
                assert!(wire.windows(name.len()).any(|bytes| bytes == name));
            }
            let input = KernelPolicyInput::from_kernel_result(&raw.5, origin);
            assert_eq!(raw.5.status(), KernelCheckStatus::Accepted);
            assert_eq!(
                raw.5.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!raw.5.policy_taint());
            assert_eq!(raw.6.class, CandidatePolicyClass::KernelVerified);
            assert_eq!(raw.7.selected_class(), ProofWinnerClass::KernelVerified);
            assert_eq!(
                input.accepted_goal_polarity(),
                Some(AcceptedGoalPolarity::AssertFalseForRefutation)
            );
            let accepted = input.accepted_evidence_hash().unwrap();
            let components = processed.local_evidence_component_hashes()?;
            let [
                kernel_profile_fingerprint,
                formula_evidence_hash,
                substitution_evidence_hash,
                provenance_hash,
            ] = [
                ("profile", components[0]),
                ("formula", components[1]),
                ("substitutions", components[2]),
                ("provenance", components[3]),
            ]
            .map(|(kind, digest)| {
                ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    format!("mizar-vc/local-evidence-component/{kind}"),
                    SchemaVersion::new(1, 0),
                    digest,
                )
            });
            let acceptance = KernelAcceptanceMetadata {
                kernel_profile_fingerprint,
                verifier_policy_fingerprint: ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    "mizar-proof/verifier-policy",
                    SchemaVersion::new(policy.schema_version(), 0),
                    policy.policy_fingerprint().hash(),
                ),
                checker_schema_version: SchemaVersion::new(policy.checker_schema_version(), 0),
                evidence_schema_version: SchemaVersion::new(parsed.schema_version(), 0),
                target_binding_hash: ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    "mizar-vc/kernel-evidence-handoff-canonical",
                    SchemaVersion::new(1, 0),
                    processed.canonical_hash(),
                ),
                formula_evidence_hash,
                substitution_evidence_hash,
                provenance_hash,
                formula_context_hash: None,
                accepted_result_hash: ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    "mizar-proof/kernel-accepted-evidence",
                    SchemaVersion::new(1, 0),
                    accepted,
                ),
            };
            assert!(processed.formula_context_requirements().is_none());
            assert!(!processed.context_identity().entries().is_empty());
            assert!(processed.context_identity_is_consistent());
            assert_eq!(
                processed.context_identity().hash(),
                processed.context_identity_hash()
            );
            assert_eq!(processed.context_identity().target_vc(), target);
            let existing_status = status(
                &source,
                check,
                env,
                snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            assert_eq!(
                existing_status.0.artifact_publication(),
                ArtifactStatusPublication::ExternalDependencyGap(
                    ArtifactPublicationGap::MissingKernelVerifiedWitness
                )
            );
            assert_eq!(existing_status.0.accepted_witness_obligation_id(), None);
            assert_eq!(raw.7.reuse_metadata().selected_proof_witness_hash(), None);
            assert_eq!(
                raw.7.reuse_metadata().proof_witness_publication(),
                ProofWitnessPublication::NotApplicable
            );
            let expected = (
                raw.0.clone(),
                raw.1.clone(),
                raw.2.clone(),
                raw.3.clone(),
                raw.4.clone(),
                raw.5.clone(),
                raw.6.clone(),
                raw.7.clone(),
                Some(acceptance.clone()),
            );
            let inputs_before = (
                raw.clone(),
                processed.clone(),
                wire.clone(),
                parsed.clone(),
                components,
                acceptance.clone(),
                existing_status.clone(),
            );
            let actual = metadata(
                &source,
                check,
                env,
                snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge_policy, &policy),
            )?;
            assert_eq!(actual, expected);
            assert_eq!(actual.0.source_id(), ast.source_id);
            assert_eq!(actual.0.module_id(), source.module());
            assert_eq!(actual.1.source(), ast.source_id);
            assert_eq!(actual.1.snapshot(), snapshot);
            assert_eq!(actual.1.schema_version(), &schema);
            assert_eq!(
                actual.1.vc(id).unwrap().anchor.generation_schema_version,
                generation
            );
            assert!(
                actual
                    .0
                    .proofs()
                    .iter()
                    .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
            );
            assert!(
                actual
                    .0
                    .obligation_seeds()
                    .iter()
                    .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
            );
            assert!(actual.1.vcs().iter().all(|vc| vc.status == VcStatus::Open));
            assert!(
                actual
                    .2
                    .vc_set()
                    .vcs()
                    .iter()
                    .all(|vc| vc.status == VcStatus::NeedsAtp)
            );
            assert_eq!(
                (
                    raw.clone(),
                    processed,
                    wire.clone(),
                    parsed,
                    components,
                    acceptance,
                    existing_status
                ),
                inputs_before
            );
            assert_eq!(format!("{check:?}"), check_before);
            prior.push(actual);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
        assert_eq!(
            (generation, schema, discharge_policy, profile, policy),
            config_before
        );
    }
}

#[test]
fn source_theorem_candidate_metadata_preserves_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::problem::*;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_theorem_candidate_metadata as metadata,
            select_source_theorem_candidate as select,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snapshot = snapshot_id(777);
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let discharge_policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let normal = VerifierPolicy::default();
    let disabled = normal.clone().with_kernel_evidence_formats([]);
    let builtin = normal
        .clone()
        .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
    let profile = LogicProfile::try_new(
        "explicit",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let unsupported = LogicProfile::try_new(
        "unsupported",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let config_before = (
        generation.clone(),
        schema.clone(),
        discharge_policy.clone(),
        normal.clone(),
        disabled.clone(),
        builtin.clone(),
        profile.clone(),
        unsupported.clone(),
    );
    for (case, text) in [
        original.to_owned(),
        "open theorem O: for X being set holds X = X;".to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            let id = VcId::new(0);
            if case == 0 {
                let raw = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &normal),
                )?;
                let raw_before = raw.clone();
                let absent = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    VcId::new(usize::MAX),
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &normal),
                )
                .unwrap_err();
                let policy_error = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &disabled),
                )
                .unwrap_err();
                assert_ne!(absent, policy_error);
                let combined = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    VcId::new(usize::MAX),
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &disabled),
                )
                .unwrap_err();
                assert_eq!(combined, absent);
                assert_eq!(
                    metadata(
                        &source,
                        check,
                        env,
                        snapshot,
                        VcId::new(usize::MAX),
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge_policy, &disabled)
                    )
                    .unwrap_err(),
                    combined
                );
                for (encoding, format, policy) in [
                    (&unsupported, ConcreteFormat::Tptp, &normal),
                    (&profile, ConcreteFormat::SmtLib, &normal),
                    (&profile, ConcreteFormat::Tptp, &disabled),
                    (&profile, ConcreteFormat::Tptp, &builtin),
                ] {
                    let expected = select(
                        &source,
                        check,
                        env,
                        snapshot,
                        id,
                        (&generation, &schema, encoding.clone(), format),
                        (&discharge_policy, policy),
                    )
                    .unwrap_err();
                    assert_eq!(
                        metadata(
                            &source,
                            check,
                            env,
                            snapshot,
                            id,
                            (&generation, &schema, encoding.clone(), format),
                            (&discharge_policy, policy)
                        )
                        .unwrap_err(),
                        expected
                    );
                }
                let (other_frontend, other_source, other) = inputs(
                    &original.replace("T:", "Foreign:"),
                    "owner-test",
                    "owners",
                    7,
                );
                assert!(other_frontend.diagnostics.is_empty() && other.diagnostics().is_empty());
                assert_ne!(source.source_id(), other_source.source_id());
                let foreign_before = (other_source.clone(), other.env().clone());
                let expected = select(
                    &source,
                    check,
                    other.env(),
                    snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &normal),
                )
                .unwrap_err();
                assert_eq!(
                    metadata(
                        &source,
                        check,
                        other.env(),
                        snapshot,
                        id,
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge_policy, &normal)
                    )
                    .unwrap_err(),
                    expected
                );
                assert_eq!((other_source, other.env().clone()), foreign_before);
                assert_eq!(raw, raw_before);
            } else {
                let expected = select(
                    &source,
                    check,
                    env,
                    snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge_policy, &normal),
                )
                .unwrap_err();
                assert_eq!(
                    metadata(
                        &source,
                        check,
                        env,
                        snapshot,
                        id,
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge_policy, &normal)
                    )
                    .unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(
        (
            generation,
            schema,
            discharge_policy,
            normal,
            disabled,
            builtin,
            profile,
            unsupported
        ),
        config_before
    );
}

#[test]
fn source_theorem_candidate_witness_draft_matches_checked_inputs() {
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessReadOptions, ProofWitnessRef,
            proof_witness_ref_json, read_proof_witness_ref, write_proof_witness_ref,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalJson, SchemaVersion, artifact_hash_domain, canonical_json_bytes},
    };
    use mizar_atp::{
        backend::{BackendCandidatePayload, encode_local_variable_reflexivity_names},
        problem::*,
    };
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::{
        policy::{KernelEvidenceOrigin, VerifierPolicy},
        selection::{ProofWinnerClass, VcProofSelection, merge_artifact_proof_selections as merge},
        status::{
            ArtifactPublicationGap, ArtifactStatusPublication, ProofStatusProjectionInput,
            TrustedUsedAxiomsRef, check_source_theorem_candidate_metadata as checked,
            project_status, source_theorem_candidate_witness_draft as draft,
            source_theorem_obligation_identity_with_schemas as identity,
        },
        witness_store::{ProofWitnessPayloadSchema, ProofWitnessProvenance, stage},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DependencyArtifactRef, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOriginInput,
        ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_theorem_discharge_kernel_handoff as handoff,
        },
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion, VcStatus},
    };
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut prior = Vec::new();
    for (case, text) in [
        original.to_owned(),
        original.replace(" X", " Local"),
        format!(":: 日本語 λ\n{original}\n"),
        original.replace("T:", "Renamed:"),
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(&path, &text).unwrap();
        let (package, module) = if case == 2 {
            ("other-package", "nested.owners")
        } else {
            ("draft-test", "owners")
        };
        let input = SourceInput {
            package_id: PackageId::new(package),
            module_path: ModulePath::new(module),
            normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
            edition: Edition::new("2026"),
            origin: if case % 2 == 0 {
                SourceOriginInput::Disk { path: path.clone() }
            } else {
                SourceOriginInput::OpenBuffer {
                    uri: format!("file://{}", path.display()),
                    expected_version: 1,
                    actual_version: 1,
                    text: text.clone().into(),
                }
            },
        };
        let policy = if case == 1 {
            VerifierPolicy::default().with_profile_id("draft λ\0")
        } else if case == 2 {
            VerifierPolicy::default()
                .with_schema_version(2)
                .with_checker_schema_version(7)
        } else {
            VerifierPolicy::release()
        };
        let dependencies = if case == 3 {
            vec![DependencyArtifactRef::new(
                "caller-recorded/source",
                hash_text(&text),
            )]
        } else {
            vec![]
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    dependencies,
                    hash_text("caller-lock"),
                    ToolchainInfo::new("caller-supplied-draft-toolchain"),
                    policy.policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        let (loaded, captured) = loaded.pop().unwrap();
        assert_eq!(captured, input);
        assert_eq!(snapshot.source_versions[0].source_id, loaded.source_id);
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(source_unit_from_loaded(loaded, path.clone()))
        .unwrap();
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        assert_eq!(ast.source_id, snapshot.source_versions[0].source_id);
        let module = ModuleId::new(input.package_id, input.module_path);
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let generation = GenerationSchemaVersion::new(if case == 3 { "" } else { "λ\n\0" });
        let schema = VcSchemaVersion::new(if case == 3 { "vc\t\0" } else { "" });
        let discharge = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(if case == 3 { "unused" } else { "explicit" }),
                max_steps: if case == 3 { 0 } else { 64 },
            },
        };
        let (fragment, format) = if case % 2 == 0 {
            (LogicFragment::Fof, ConcreteFormat::Tptp)
        } else {
            (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib)
        };
        let profile = LogicProfile::try_new(
            "draft λ\0",
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
        let witness_path = format!("proof-witnesses/caller-{case}/checked.bin");
        let before = (source.clone(), env.clone(), snapshot.clone(), prior.clone());
        let config_before = (
            generation.clone(),
            schema.clone(),
            discharge.clone(),
            profile.clone(),
            policy.clone(),
            witness_path.clone(),
        );
        with_source_theorem_check(ast, &source, env, |check| {
            let check_before = format!("{check:?}");
            let id = VcId::new(0);
            let original = identity(&source, check, env, snapshot.id, id, (&generation, &schema))?;
            let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
            let raw = checked(
                &source,
                check,
                env,
                snapshot.id,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge, &policy),
            )?;
            let acceptance = raw.8.as_ref().unwrap();
            let BackendCandidatePayload::FormulaSubstitutionBytes(wire) = raw.4.payload() else {
                panic!()
            };
            let processed = handoff(&raw.0, &raw.1, id, &raw.2, &discharge)?;
            let target = processed.canonical_evidence().target_vc();
            let actual_profile = processed.canonical_evidence().kernel_profile();
            let tautology = match actual_profile.clause_tautology_policy {
                KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                _ => panic!(),
            };
            let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(actual_profile.profile_id, tautology),
            );
            parse_context.accepted_schema_versions.insert(2);
            let parsed = parse_formula_evidence(wire, &parse_context).unwrap();
            assert_eq!(parsed.schema_version(), 2);
            let v1 =
                parse_formula_evidence(&processed.encode_local_evidence(None)?, &parse_context)
                    .unwrap();
            let names = encode_local_variable_reflexivity_names(&v1)?;
            assert!(!names.is_empty());
            assert_eq!(parsed.equality_instances().len(), names.len());
            assert_eq!(
                acceptance.target_binding_hash.digest,
                processed.canonical_hash()
            );
            assert!(acceptance.formula_context_hash.is_none());
            assert!(!processed.context_identity().entries().is_empty());
            let reference = TrustedUsedAxiomsRef::from_kernel_result(&raw.5, origin).unwrap();
            let [selection] =
                <[_; 1]>::try_from(merge([VcProofSelection::new(id, raw.7.clone())], []).unwrap())
                    .unwrap();
            let projection = project_status(
                ProofStatusProjectionInput::new(selection, policy.clone(), original.0.clone())
                    .with_trusted_used_axioms(reference.clone()),
            )
            .unwrap();
            let provenance = ProofWitnessProvenance::new(
                snapshot.id.fingerprint(),
                "mizar-proof/source-theorem-candidate-witness-draft/v1",
                raw.7.selected_candidate_id().unwrap().clone(),
                origin,
                original.0.vc_fingerprint(),
                original.1.fingerprint().hash(),
                snapshot.dependency_artifact_fingerprint(),
                policy.policy_fingerprint().hash(),
                acceptance.checker_schema_version,
                acceptance.evidence_schema_version,
            )
            .unwrap()
            .with_accepted_result_hash(acceptance.accepted_result_hash.digest)
            .with_trusted_used_axioms_ref(&reference);
            let payload_schema = ProofWitnessPayloadSchema::with_canonical_bytes_required(
                "mizar-proof/source-theorem-candidate-checked-wire",
                acceptance.evidence_schema_version,
                true,
            )
            .unwrap();
            let obligation = ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                "mizar-proof/source-theorem-original-obligation",
                SchemaVersion::new(1, 0),
                original.0.obligation_fingerprint(),
            );
            let wire_hex = wire
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            for name in &names {
                assert!(wire.windows(name.len()).any(|bytes| bytes == name));
            }
            let hashes = [
                acceptance.accepted_result_hash.digest,
                policy.policy_fingerprint().hash(),
            ]
            .map(|hash| {
                hash.as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            });
            let expected_value = CanonicalJson::object([
                (
                    "hash_domain",
                    CanonicalJson::string("mizar-proof/witness-payload/v1"),
                ),
                (
                    "obligation_fingerprint",
                    CanonicalJson::string(obligation.to_artifact_hash_string()),
                ),
                ("payload_bytes_hex", CanonicalJson::string(&wire_hex)),
                (
                    "payload_schema_family",
                    CanonicalJson::string("mizar-proof/source-theorem-candidate-checked-wire"),
                ),
                (
                    "payload_schema_version",
                    CanonicalJson::string(acceptance.evidence_schema_version.to_string()),
                ),
                ("selected_evidence_hash", CanonicalJson::string(&hashes[0])),
                (
                    "verifier_policy_fingerprint",
                    CanonicalJson::string(&hashes[1]),
                ),
            ])
            .unwrap();
            let domain = artifact_hash_domain(payload_schema.family(), payload_schema.version());
            let payload_hash = ArtifactHashRef::new(
                ArtifactHashClass::Artifact,
                payload_schema.family(),
                payload_schema.version(),
                domain.hash(&expected_value, &[]),
            );
            let expected_ref = ProofWitnessRef {
                schema_version: SchemaVersion::new(2, 0),
                obligation_id: original.0.obligation_id().into(),
                obligation_fingerprint: obligation.clone(),
                proof_status: ProofStatus::KernelVerified,
                evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
                witness_path: witness_path.clone(),
                witness_artifact_hash: payload_hash.clone(),
                kernel_acceptance: acceptance.clone(),
            };
            let inputs_before = (
                original.clone(),
                raw.clone(),
                projection.clone(),
                provenance.clone(),
                payload_schema.clone(),
            );
            let actual = draft(
                &source,
                check,
                env,
                &snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge, &policy),
                &witness_path,
            )?;
            assert_eq!((&actual.0, &actual.1), (&projection, &original.1));
            let acquired = &actual.2;
            assert_eq!(acquired.obligation_id(), original.0.obligation_id());
            assert_eq!(acquired.obligation_anchor(), original.0.obligation_anchor());
            assert_eq!(acquired.obligation_fingerprint(), &obligation);
            assert_eq!(acquired.selected_class(), ProofWinnerClass::KernelVerified);
            assert_eq!(
                acquired.selected_evidence_hash(),
                reference.accepted_evidence_hash()
            );
            assert_eq!(acquired.payload_schema(), &payload_schema);
            assert_eq!(acquired.payload_bytes(), wire);
            assert_eq!(acquired.payload_hash_value(), expected_value);
            assert_eq!(acquired.witness_path(), witness_path);
            assert_eq!(acquired.kernel_acceptance(), acceptance);
            assert_eq!(acquired.provenance(), &provenance);
            assert_eq!(
                acquired.provenance().trusted_used_axioms_ref(),
                Some(&reference)
            );
            assert_eq!(actual.0.trusted_used_axioms(), Some(&reference));
            assert_eq!(reference.used_axiom_count(), raw.5.used_axioms().len());
            assert!(raw.5.used_axioms().is_empty());
            assert_eq!(
                actual.0.artifact_publication(),
                ArtifactStatusPublication::ExternalDependencyGap(
                    ArtifactPublicationGap::MissingKernelVerifiedWitness
                )
            );
            assert_eq!(actual.0.accepted_witness_obligation_id(), None);
            assert_eq!(
                actual.0.reuse_metadata().selected_proof_witness_hash(),
                None
            );
            assert!(!actual.0.reuse_metadata().cache_reuse_predicate_complete());
            assert!(actual.1.requires_cache_miss() && !actual.1.is_complete());
            assert!(
                raw.0
                    .proofs()
                    .iter()
                    .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
            );
            assert!(
                raw.0
                    .obligation_seeds()
                    .iter()
                    .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
            );
            assert!(raw.1.vcs().iter().all(|vc| vc.status == VcStatus::Open));
            assert!(
                raw.2
                    .vc_set()
                    .vcs()
                    .iter()
                    .all(|vc| vc.status == VcStatus::NeedsAtp)
            );
            let staged = stage(acquired.clone()).unwrap();
            assert_eq!(staged.staged_payload_hash(), &payload_hash);
            assert_eq!(staged.artifact_reference_candidate(), Some(&expected_ref));
            assert_eq!(staged.provenance(), &provenance);
            let json = proof_witness_ref_json(&expected_ref).unwrap();
            assert_eq!(
                write_proof_witness_ref(staged.artifact_reference_candidate().unwrap()).unwrap(),
                canonical_json_bytes(&json)
            );
            assert_eq!(
                read_proof_witness_ref(&json, ProofWitnessReadOptions::default()).unwrap(),
                expected_ref
            );
            assert!(!root.join(&witness_path).exists());
            assert_eq!(
                (
                    original.clone(),
                    raw.clone(),
                    projection,
                    provenance,
                    payload_schema
                ),
                inputs_before
            );
            assert_eq!(format!("{check:?}"), check_before);
            prior.push(actual);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env, &snapshot), (&before.0, &before.1, &before.2));
        assert_eq!(&prior[..before.3.len()], before.3);
        assert_eq!(
            (generation, schema, discharge, profile, policy, witness_path),
            config_before
        );
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_theorem_candidate_witness_draft_preserves_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_atp::problem::*;
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::{KernelEvidenceFormat, KernelEvidenceOrigin, VerifierPolicy},
        selection::{VcProofSelection, merge_artifact_proof_selections as merge},
        status::{
            ProofStatusProjectionInput, TrustedUsedAxiomsRef,
            check_source_theorem_candidate_metadata as checked, project_status,
            source_theorem_candidate_witness_draft as draft,
            source_theorem_obligation_identity_with_schemas as identity,
        },
        witness_store::ProofWitnessStoreError,
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo,
        WorkspaceRoot, hash_text,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    fs::write(&path, text).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let registry = SnapshotRegistry::new();
    let policy = VerifierPolicy::default();
    let input = SourceInput {
        package_id: PackageId::new("draft-errors"),
        module_path: ModulePath::new("owners"),
        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
        edition: Edition::new("2026"),
        origin: SourceOriginInput::Disk { path: path.clone() },
    };
    let (snapshot, lease, mut loaded) = registry
        .load_snapshot(
            ids.next_request_id().unwrap(),
            (
                WorkspaceRoot::new(root.to_str().unwrap()),
                vec![],
                hash_text("caller-lock"),
                ToolchainInfo::new("caller-toolchain"),
                policy.policy_fingerprint().hash(),
            ),
            vec![(DiskSourceLoader::new(&root), input.clone())],
        )
        .unwrap();
    let (loaded, _) = loaded.pop().unwrap();
    assert_eq!(loaded.source_id, snapshot.source_versions[0].source_id);
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::super::super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run_loaded(source_unit_from_loaded(loaded, path.clone()))
    .unwrap();
    let ast = frontend.ast.as_ref().unwrap();
    let module = ModuleId::new(input.package_id, input.module_path);
    let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected =
        SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str()))
            .collect(&module);
    let env = collected.env();
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let discharge = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let disabled = policy.clone().with_kernel_evidence_formats([]);
    let builtin = policy
        .clone()
        .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
    let profile = LogicProfile::try_new(
        "explicit",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let before = (
        source.clone(),
        env.clone(),
        snapshot.clone(),
        generation.clone(),
        schema.clone(),
        discharge.clone(),
        policy.clone(),
        profile.clone(),
    );
    with_source_theorem_check(ast, &source, env, |check| {
        let check_before = format!("{check:?}");
        let id = VcId::new(0);
        let original = identity(&source, check, env, snapshot.id, id, (&generation, &schema))?;
        let raw = checked(
            &source,
            check,
            env,
            snapshot.id,
            id,
            (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
            (&discharge, &policy),
        )?;
        let reference = TrustedUsedAxiomsRef::from_kernel_result(
            &raw.5,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
        )
        .unwrap();
        let [selection] =
            <[_; 1]>::try_from(merge([VcProofSelection::new(id, raw.7.clone())], []).unwrap())
                .unwrap();
        let expected_projection = project_status(
            ProofStatusProjectionInput::new(selection, policy.clone(), original.0.clone())
                .with_trusted_used_axioms(reference),
        )
        .unwrap();
        let baseline = draft(
            &source,
            check,
            env,
            &snapshot,
            id,
            (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
            (&discharge, &policy),
            "proof-witnesses/checked.bin",
        )?;
        assert_eq!(
            (&baseline.0, &baseline.1),
            (&expected_projection, &original.1)
        );
        let baseline_before = (original.clone(), raw.clone(), baseline.clone());
        for (path, reason) in [
            ("", "path must start with proof-witnesses/"),
            (
                "proof-witnesses/",
                "path must contain a child segment after proof-witnesses/",
            ),
            (
                "proof-witnesses/a\\b",
                "path must be artifact-root relative and use / separators",
            ),
            (
                "proof-witnesses/../checked.bin",
                "path must not contain empty, ., .., or drive-prefix segments",
            ),
        ] {
            let expected = ProofWitnessStoreError::InvalidWitnessPath {
                reason: reason.into(),
            }
            .to_string();
            assert_eq!(
                draft(
                    &source,
                    check,
                    env,
                    &snapshot,
                    id,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge, &policy),
                    path
                )
                .unwrap_err(),
                expected
            );
        }
        let absent = identity(
            &source,
            check,
            env,
            snapshot.id,
            VcId::new(usize::MAX),
            (&generation, &schema),
        )
        .unwrap_err();
        assert_eq!(
            draft(
                &source,
                check,
                env,
                &snapshot,
                VcId::new(usize::MAX),
                (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                (&discharge, &disabled),
                ""
            )
            .unwrap_err(),
            absent
        );
        for (format, current) in [
            (ConcreteFormat::SmtLib, &policy),
            (ConcreteFormat::Tptp, &disabled),
            (ConcreteFormat::Tptp, &builtin),
        ] {
            let expected = checked(
                &source,
                check,
                env,
                snapshot.id,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge, current),
            )
            .unwrap_err();
            assert_eq!(
                draft(
                    &source,
                    check,
                    env,
                    &snapshot,
                    id,
                    (&generation, &schema, profile.clone(), format),
                    (&discharge, current),
                    "proof-witnesses/checked.bin"
                )
                .unwrap_err(),
                expected
            );
        }
        let (_, other_source, other) =
            inputs(&text.replace("T:", "Foreign:"), "draft-errors", "owners", 7);
        let expected = identity(
            &source,
            check,
            other.env(),
            snapshot.id,
            id,
            (&generation, &schema),
        )
        .unwrap_err();
        assert_eq!(
            draft(
                &source,
                check,
                other.env(),
                &snapshot,
                id,
                (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                (&discharge, &policy),
                "proof-witnesses/checked.bin"
            )
            .unwrap_err(),
            expected
        );
        assert_ne!(source.source_id(), other_source.source_id());
        let (unsupported_frontend, unsupported_source, unsupported_symbols) = inputs(
            "open theorem O: for X being set holds X = X;",
            "draft-errors",
            "owners",
            7,
        );
        with_source_theorem_check(
            unsupported_frontend.ast.as_ref().unwrap(),
            &unsupported_source,
            unsupported_symbols.env(),
            |unsupported_check| {
                let expected = identity(
                    &unsupported_source,
                    unsupported_check,
                    unsupported_symbols.env(),
                    snapshot.id,
                    id,
                    (&generation, &schema),
                )
                .unwrap_err();
                assert_eq!(
                    draft(
                        &unsupported_source,
                        unsupported_check,
                        unsupported_symbols.env(),
                        &snapshot,
                        id,
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge, &policy),
                        "proof-witnesses/checked.bin"
                    )
                    .unwrap_err(),
                    expected
                );
                Ok(())
            },
        )?;
        assert_eq!((original, raw, baseline), baseline_before);
        assert_eq!(format!("{check:?}"), check_before);
        Ok(())
    })
    .unwrap();
    assert_eq!(
        (
            source,
            env.clone(),
            snapshot.clone(),
            generation,
            schema,
            discharge,
            policy,
            profile
        ),
        before
    );
    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_theorem_candidate_witness_file_matches_transport() {
    use mizar_artifact::{
        proof_witness::{EvidenceKind, ProofStatus, ProofWitnessRef},
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            CanonicalJson, PublishedArtifactPath, PublishedArtifactReadOptions, SchemaVersion,
            artifact_hash_domain, canonical_json_bytes, read_published_artifact,
            write_published_artifact,
        },
    };
    use mizar_atp::problem::*;
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            ArtifactPublicationGap, ArtifactStatusPublication,
            source_theorem_candidate_witness_draft as draft,
            source_theorem_candidate_witness_file_candidate as candidate,
        },
        witness_store::{read_source_theorem_witness_file as recover, stage},
    };
    use mizar_resolve::{resolved_ast::ModuleId, symbols::source_theorem_module_original_inputs};
    use mizar_session::{
        DependencyArtifactRef, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOriginInput,
        ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    for (case, text) in [original.to_owned(), original.replace(" X", " Local")]
        .into_iter()
        .enumerate()
    {
        fs::write(&path, &text).unwrap();
        let policy = if case == 0 {
            VerifierPolicy::default()
        } else {
            VerifierPolicy::release()
                .with_profile_id("file λ\0")
                .with_checker_schema_version(7)
        };
        let input = SourceInput {
            package_id: PackageId::new("candidate-file"),
            module_path: ModulePath::new("owners"),
            normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
            edition: Edition::new("2026"),
            origin: if case == 0 {
                SourceOriginInput::Disk { path: path.clone() }
            } else {
                SourceOriginInput::OpenBuffer {
                    uri: format!("file://{}", path.display()),
                    expected_version: 1,
                    actual_version: 1,
                    text: text.clone().into(),
                }
            },
        };
        let dependencies = if case == 0 {
            vec![]
        } else {
            vec![DependencyArtifactRef::new(
                "caller-recorded/source",
                hash_text(&text),
            )]
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    dependencies,
                    hash_text("caller-lock"),
                    ToolchainInfo::new("caller-file-test"),
                    policy.policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        let (loaded, received) = loaded.pop().unwrap();
        assert_eq!(received, input);
        assert_eq!(loaded.source_id, snapshot.source_versions[0].source_id);
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(source_unit_from_loaded(loaded, path.clone()))
        .unwrap();
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        assert_eq!(ast.source_id, snapshot.source_versions[0].source_id);
        let module = ModuleId::new(input.package_id, input.module_path);
        let (source, symbols) = source_theorem_module_original_inputs(ast, &module)
            .unwrap()
            .unwrap();
        let generation = GenerationSchemaVersion::new(if case == 0 { "λ\n\0" } else { "" });
        let schema = VcSchemaVersion::new(if case == 0 { "" } else { "vc\t\0" });
        let discharge = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(if case == 0 { "explicit" } else { "unused" }),
                max_steps: if case == 0 { 64 } else { 0 },
            },
        };
        let (fragment, format) = if case == 0 {
            (LogicFragment::Fof, ConcreteFormat::Tptp)
        } else {
            (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib)
        };
        let profile = LogicProfile::try_new(
            "file λ\0",
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
        let witness_path = format!("proof-witnesses/file-{case}/checked.json");
        let before = (
            source.clone(),
            symbols.clone(),
            snapshot.clone(),
            generation.clone(),
            schema.clone(),
            discharge.clone(),
            profile.clone(),
            policy.clone(),
        );
        with_source_theorem_check(ast, &source, &symbols, |check| {
            let check_before = format!("{check:?}");
            let id = VcId::new(0);
            let baseline = draft(
                &source,
                check,
                &symbols,
                &snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge, &policy),
                &witness_path,
            )?;
            let generic = stage(baseline.2.clone()).unwrap();
            assert_eq!(
                generic.staged_payload_hash().digest,
                artifact_hash_domain(
                    baseline.2.payload_schema().family(),
                    baseline.2.payload_schema().version()
                )
                .hash(&baseline.2.payload_hash_value(), &[])
            );
            let d = &baseline.2;
            let m = d.kernel_acceptance();
            let mut acceptance = std::collections::BTreeMap::new();
            for (key, hash) in [
                ("kernel_profile_fingerprint", &m.kernel_profile_fingerprint),
                (
                    "verifier_policy_fingerprint",
                    &m.verifier_policy_fingerprint,
                ),
                ("target_binding_hash", &m.target_binding_hash),
                ("formula_evidence_hash", &m.formula_evidence_hash),
                ("substitution_evidence_hash", &m.substitution_evidence_hash),
                ("provenance_hash", &m.provenance_hash),
                ("accepted_result_hash", &m.accepted_result_hash),
            ] {
                acceptance.insert(
                    key.to_owned(),
                    CanonicalJson::string(hash.to_artifact_hash_string()),
                );
            }
            acceptance.insert(
                "checker_schema_version".into(),
                CanonicalJson::string(m.checker_schema_version.to_string()),
            );
            acceptance.insert(
                "evidence_schema_version".into(),
                CanonicalJson::string(m.evidence_schema_version.to_string()),
            );
            acceptance.insert(
                "formula_context_hash".into(),
                m.formula_context_hash
                    .as_ref()
                    .map_or(CanonicalJson::Null, |hash| {
                        CanonicalJson::string(hash.to_artifact_hash_string())
                    }),
            );
            let wire_hex = d
                .payload_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let file = CanonicalJson::object([
                ("schema_version", CanonicalJson::string("1.0")),
                (
                    "schema_family",
                    CanonicalJson::string("mizar-proof/source-theorem-witness-file"),
                ),
                ("obligation_id", CanonicalJson::string(d.obligation_id())),
                (
                    "obligation_fingerprint",
                    CanonicalJson::string(d.obligation_fingerprint().to_artifact_hash_string()),
                ),
                (
                    "payload_schema_family",
                    CanonicalJson::string("mizar-proof/source-theorem-candidate-checked-wire"),
                ),
                ("payload_schema_version", CanonicalJson::string("2.0")),
                ("payload_bytes_hex", CanonicalJson::string(wire_hex)),
                ("kernel_acceptance", CanonicalJson::Object(acceptance)),
            ])
            .unwrap();
            let domain = artifact_hash_domain(
                "mizar-proof/source-theorem-witness-file",
                SchemaVersion::new(1, 0),
            );
            let file_hash = ArtifactHashRef::new(
                ArtifactHashClass::Artifact,
                "mizar-proof/source-theorem-witness-file",
                SchemaVersion::new(1, 0),
                domain.hash(&file, &[]),
            );
            let expected_ref = ProofWitnessRef {
                schema_version: SchemaVersion::new(2, 0),
                obligation_id: d.obligation_id().into(),
                obligation_fingerprint: d.obligation_fingerprint().clone(),
                proof_status: ProofStatus::KernelVerified,
                evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
                witness_path: witness_path.clone(),
                witness_artifact_hash: file_hash.clone(),
                kernel_acceptance: m.clone(),
            };
            assert_ne!(generic.staged_payload_hash(), &file_hash);
            let baseline_before = baseline.clone();
            let actual = candidate(
                &source,
                check,
                &symbols,
                &snapshot,
                id,
                (&generation, &schema, profile.clone(), format),
                (&discharge, &policy),
                &witness_path,
            )?;
            assert_eq!(
                actual,
                (baseline.clone(), file.clone(), expected_ref.clone())
            );
            let transport = root.join("unpublished-files");
            let published_path = PublishedArtifactPath::new(&witness_path).unwrap();
            let written =
                write_published_artifact(&transport, &published_path, &actual.1, &domain, &[])
                    .unwrap();
            assert_eq!(written.artifact_hash, file_hash.digest);
            let read = read_published_artifact(
                &transport,
                &published_path,
                PublishedArtifactReadOptions {
                    artifact_hash_domain: Some(&domain),
                    expected_artifact_hash: Some(file_hash.digest),
                    ..PublishedArtifactReadOptions::default()
                },
            )
            .unwrap();
            assert_eq!(read.value, file);
            assert_eq!(
                fs::read(transport.join(&witness_path)).unwrap(),
                canonical_json_bytes(&file)
            );
            assert_eq!(recover(&read, &actual.2).unwrap(), d.payload_bytes());
            let staged = actual
                .0
                .2
                .clone()
                .stage_source_theorem_file(&read, &actual.2)?;
            assert_eq!(staged.obligation_id(), generic.obligation_id());
            assert_eq!(staged.obligation_anchor(), generic.obligation_anchor());
            assert_eq!(
                staged.obligation_fingerprint(),
                generic.obligation_fingerprint()
            );
            assert_eq!(staged.selected_class(), generic.selected_class());
            assert_eq!(
                staged.selected_evidence_hash(),
                generic.selected_evidence_hash()
            );
            assert_eq!(staged.witness_path(), generic.witness_path());
            assert_eq!(staged.provenance(), generic.provenance());
            assert_eq!(
                staged.payload_schema().family(),
                "mizar-proof/source-theorem-witness-file"
            );
            assert_eq!(staged.payload_schema().version(), SchemaVersion::new(1, 0));
            assert!(staged.payload_schema().canonical_bytes_required());
            assert_eq!(staged.staged_payload_hash(), &file_hash);
            assert_eq!(staged.artifact_reference_candidate(), Some(&expected_ref));
            let reference = d.provenance().trusted_used_axioms_ref().unwrap();
            assert_eq!(
                staged.provenance().trusted_used_axioms_ref(),
                Some(reference)
            );
            assert_eq!(
                staged.provenance().trusted_used_axiom_ref_hash(),
                Some(reference.used_axioms_hash())
            );
            assert_eq!(baseline.0.trusted_used_axioms(), Some(reference));
            assert_eq!(
                baseline.0.artifact_publication(),
                ArtifactStatusPublication::ExternalDependencyGap(
                    ArtifactPublicationGap::MissingKernelVerifiedWitness
                )
            );
            assert_eq!(baseline.0.accepted_witness_obligation_id(), None);
            assert_eq!(
                baseline.0.reuse_metadata().selected_proof_witness_hash(),
                None
            );
            assert!(baseline.1.requires_cache_miss());
            assert_eq!(baseline, baseline_before);
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            (
                source,
                symbols,
                snapshot.clone(),
                generation,
                schema,
                discharge,
                profile,
                policy
            ),
            before
        );
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_theorem_candidate_witness_file_rejects_cross_associations() {
    use mizar_artifact::{
        proof_witness::{EvidenceKind, ProofStatus, ProofWitnessRef},
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            CanonicalJson, MinorVersionPolicy, PublishedArtifactPath, PublishedArtifactReadOptions,
            SchemaVersion, SchemaVersionSupport, artifact_hash_domain, read_published_artifact,
            write_published_artifact,
        },
    };
    use mizar_atp::problem::*;
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            source_theorem_candidate_witness_draft as draft,
            source_theorem_candidate_witness_file_candidate as candidate,
            source_theorem_reflexivity_witness_file_candidate as builtin,
        },
        witness_store::{ProofWitnessDraft, read_source_theorem_witness_file as recover},
    };
    use mizar_resolve::{resolved_ast::ModuleId, symbols::source_theorem_module_original_inputs};
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo,
        WorkspaceRoot, hash_text,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcId, VcSchemaVersion},
    };
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let policy = VerifierPolicy::default();
    let changed_policy = policy.clone().with_profile_id("other-current-policy");
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let discharge = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let profile = LogicProfile::try_new(
        "explicit",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let witness_path = "proof-witnesses/errors/checked.json";
    let mut captured = Vec::new();
    for current in [text.to_owned(), text.replace("T:", "Foreign:")] {
        fs::write(&path, &current).unwrap();
        let input = SourceInput {
            package_id: PackageId::new("file-errors"),
            module_path: ModulePath::new("owners"),
            normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
            edition: Edition::new("2026"),
            origin: SourceOriginInput::Disk { path: path.clone() },
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    vec![],
                    hash_text("caller-lock"),
                    ToolchainInfo::new("caller-file-test"),
                    policy.policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        let (loaded, _) = loaded.pop().unwrap();
        assert_eq!(loaded.source_id, snapshot.source_versions[0].source_id);
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(source_unit_from_loaded(loaded, path.clone()))
        .unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(input.package_id, input.module_path);
        let (source, symbols) = source_theorem_module_original_inputs(ast, &module)
            .unwrap()
            .unwrap();
        captured.push((snapshot, lease, frontend, source, symbols));
    }
    let (snapshot, _, frontend, source, symbols) = &captured[0];
    let (normal, other_policy, genuine_builtin) =
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), source, symbols, |check| {
            let normal = draft(
                source,
                check,
                symbols,
                snapshot,
                VcId::new(0),
                (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                (&discharge, &policy),
                witness_path,
            )?;
            let other_policy = draft(
                source,
                check,
                symbols,
                snapshot,
                VcId::new(0),
                (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                (&discharge, &changed_policy),
                witness_path,
            )?;
            let genuine_builtin = builtin(source, symbols, snapshot, &policy, witness_path)?;
            let disabled = policy.clone().with_kernel_evidence_formats([]);
            for (selected, current, path) in [
                (VcId::new(0), &policy, ""),
                (VcId::new(0), &disabled, witness_path),
                (VcId::new(usize::MAX), &disabled, ""),
            ] {
                let expected = draft(
                    source,
                    check,
                    symbols,
                    snapshot,
                    selected,
                    (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                    (&discharge, current),
                    path,
                )
                .unwrap_err();
                assert_eq!(
                    candidate(
                        source,
                        check,
                        symbols,
                        snapshot,
                        selected,
                        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                        (&discharge, current),
                        path
                    )
                    .unwrap_err(),
                    expected
                );
            }
            Ok((normal, other_policy, genuine_builtin))
        })
        .unwrap();
    let (snapshot, _, frontend, source, symbols) = &captured[1];
    let other_source =
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), source, symbols, |check| {
            draft(
                source,
                check,
                symbols,
                snapshot,
                VcId::new(0),
                (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
                (&discharge, &policy),
                witness_path,
            )
        })
        .unwrap();
    let domain = artifact_hash_domain(
        "mizar-proof/source-theorem-witness-file",
        SchemaVersion::new(1, 0),
    );
    let assemble = |d: &ProofWitnessDraft| {
        let m = d.kernel_acceptance();
        let mut metadata = std::collections::BTreeMap::new();
        for (key, hash) in [
            ("kernel_profile_fingerprint", &m.kernel_profile_fingerprint),
            (
                "verifier_policy_fingerprint",
                &m.verifier_policy_fingerprint,
            ),
            ("target_binding_hash", &m.target_binding_hash),
            ("formula_evidence_hash", &m.formula_evidence_hash),
            ("substitution_evidence_hash", &m.substitution_evidence_hash),
            ("provenance_hash", &m.provenance_hash),
            ("accepted_result_hash", &m.accepted_result_hash),
        ] {
            metadata.insert(
                key.to_owned(),
                CanonicalJson::string(hash.to_artifact_hash_string()),
            );
        }
        metadata.insert(
            "checker_schema_version".into(),
            CanonicalJson::string(m.checker_schema_version.to_string()),
        );
        metadata.insert(
            "evidence_schema_version".into(),
            CanonicalJson::string(m.evidence_schema_version.to_string()),
        );
        metadata.insert(
            "formula_context_hash".into(),
            m.formula_context_hash
                .as_ref()
                .map_or(CanonicalJson::Null, |hash| {
                    CanonicalJson::string(hash.to_artifact_hash_string())
                }),
        );
        let file = CanonicalJson::object([
            ("schema_version", CanonicalJson::string("1.0")),
            (
                "schema_family",
                CanonicalJson::string("mizar-proof/source-theorem-witness-file"),
            ),
            ("obligation_id", CanonicalJson::string(d.obligation_id())),
            (
                "obligation_fingerprint",
                CanonicalJson::string(d.obligation_fingerprint().to_artifact_hash_string()),
            ),
            (
                "payload_schema_family",
                CanonicalJson::string(d.payload_schema().family()),
            ),
            (
                "payload_schema_version",
                CanonicalJson::string(d.payload_schema().version().to_string()),
            ),
            (
                "payload_bytes_hex",
                CanonicalJson::string(
                    d.payload_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>(),
                ),
            ),
            ("kernel_acceptance", CanonicalJson::Object(metadata)),
        ])
        .unwrap();
        let reference = ProofWitnessRef {
            schema_version: SchemaVersion::new(2, 0),
            obligation_id: d.obligation_id().into(),
            obligation_fingerprint: d.obligation_fingerprint().clone(),
            proof_status: ProofStatus::KernelVerified,
            evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
            witness_path: d.witness_path().into(),
            witness_artifact_hash: ArtifactHashRef::new(
                ArtifactHashClass::Artifact,
                "mizar-proof/source-theorem-witness-file",
                SchemaVersion::new(1, 0),
                domain.hash(&file, &[]),
            ),
            kernel_acceptance: m.clone(),
        };
        (file, reference)
    };
    let (normal_file, normal_ref) = assemble(&normal.2);
    let (policy_file, policy_ref) = assemble(&other_policy.2);
    let (source_file, source_ref) = assemble(&other_source.2);
    let transport = root.join("unpublished-files");
    let published_path = PublishedArtifactPath::new(witness_path).unwrap();
    let read_options = PublishedArtifactReadOptions {
        artifact_hash_domain: Some(&domain),
        ..PublishedArtifactReadOptions::default()
    };
    let before = (
        normal.clone(),
        other_policy.clone(),
        other_source.clone(),
        genuine_builtin.clone(),
    );
    for (product, file, reference) in [
        (&normal, &normal_file, &normal_ref),
        (&other_policy, &policy_file, &policy_ref),
        (&other_source, &source_file, &source_ref),
        (&genuine_builtin.0, &genuine_builtin.1, &genuine_builtin.2),
    ] {
        write_published_artifact(&transport, &published_path, file, &domain, &[]).unwrap();
        let read = read_published_artifact(&transport, &published_path, read_options).unwrap();
        assert_eq!(
            recover(&read, reference).unwrap(),
            product.2.payload_bytes()
        );
        let staged = product
            .2
            .clone()
            .stage_source_theorem_file(&read, reference)
            .unwrap();
        assert_eq!(staged.artifact_reference_candidate(), Some(reference));
        assert_eq!(staged.provenance(), product.2.provenance());
    }
    for (product, file, reference, family, version, status) in [
        (
            &normal,
            &normal_file,
            &normal_ref,
            "mizar-proof/source-theorem-reflexivity-checked-wire",
            3,
            ProofStatus::DischargedBuiltin,
        ),
        (
            &genuine_builtin.0,
            &genuine_builtin.1,
            &genuine_builtin.2,
            "mizar-proof/source-theorem-candidate-checked-wire",
            2,
            ProofStatus::KernelVerified,
        ),
    ] {
        let CanonicalJson::Object(mut fields) = file.clone() else {
            panic!()
        };
        fields.insert(
            "payload_schema_family".into(),
            CanonicalJson::string(family),
        );
        let value = CanonicalJson::Object(fields);
        let mut relabeled = reference.clone();
        relabeled.schema_version = SchemaVersion::new(version, 0);
        relabeled.proof_status = status;
        relabeled.witness_artifact_hash.digest = domain.hash(&value, &[]);
        write_published_artifact(&transport, &published_path, &value, &domain, &[]).unwrap();
        let read = read_published_artifact(&transport, &published_path, read_options).unwrap();
        assert_eq!(
            recover(&read, &relabeled).unwrap(),
            product.2.payload_bytes()
        );
        assert_eq!(
            product
                .2
                .clone()
                .stage_source_theorem_file(&read, &relabeled)
                .unwrap_err(),
            "source theorem witness file staging mismatch"
        );
    }
    let CanonicalJson::Object(fields) = &normal_file else {
        panic!()
    };
    let support = SchemaVersionSupport::new(
        "mizar-proof/source-theorem-witness-file",
        1,
        0,
        MinorVersionPolicy::UpToSupported,
    );
    let mut mutations = Vec::new();
    for (key, value) in [
        (
            "payload_schema_family",
            CanonicalJson::string("unknown/wire"),
        ),
        ("payload_schema_version", CanonicalJson::string("1.0")),
        ("payload_bytes_hex", CanonicalJson::string("0g")),
        ("kernel_acceptance", CanonicalJson::Null),
    ] {
        let mut changed = fields.clone();
        changed.insert(key.into(), value);
        mutations.push(CanonicalJson::Object(changed));
    }
    let mut missing = fields.clone();
    missing.remove("obligation_id");
    mutations.push(CanonicalJson::Object(missing));
    let mut incompatible = fields.clone();
    incompatible.insert("schema_version".into(), CanonicalJson::string("2.0"));
    incompatible.insert("payload_bytes_hex".into(), CanonicalJson::Null);
    mutations.push(CanonicalJson::Object(incompatible));
    for value in mutations {
        let mut expected_ref = normal_ref.clone();
        expected_ref.witness_artifact_hash.digest = domain.hash(&value, &[]);
        write_published_artifact(&transport, &published_path, &value, &domain, &[]).unwrap();
        let read = read_published_artifact(&transport, &published_path, read_options).unwrap();
        let error = recover(&read, &expected_ref).unwrap_err();
        if let CanonicalJson::Object(fields) = &value
            && fields.get("schema_version") == Some(&CanonicalJson::string("2.0"))
        {
            assert_eq!(
                error,
                support
                    .check_at_path(Some("2.0"), witness_path)
                    .unwrap_err()
                    .to_string()
            );
        }
        assert_eq!(
            normal
                .2
                .clone()
                .stage_source_theorem_file(&read, &expected_ref)
                .unwrap_err(),
            error
        );
    }
    write_published_artifact(&transport, &published_path, &normal_file, &domain, &[]).unwrap();
    let read = read_published_artifact(&transport, &published_path, read_options).unwrap();
    let mut wrong_hash = read.clone();
    wrong_hash.artifact_hash = None;
    let error = recover(&wrong_hash, &normal_ref).unwrap_err();
    assert_eq!(
        normal
            .2
            .clone()
            .stage_source_theorem_file(&wrong_hash, &normal_ref)
            .unwrap_err(),
        error
    );
    for case in 0..3 {
        let mut wrong_ref = normal_ref.clone();
        match case {
            0 => wrong_ref.witness_path = "proof-witnesses/foreign.json".into(),
            1 => wrong_ref.obligation_id.push_str("-foreign"),
            2 => wrong_ref.witness_artifact_hash.digest = hash_text("foreign-file"),
            _ => unreachable!(),
        }
        let error = recover(&read, &wrong_ref).unwrap_err();
        assert_eq!(
            normal
                .2
                .clone()
                .stage_source_theorem_file(&read, &wrong_ref)
                .unwrap_err(),
            error
        );
    }
    for (file, reference) in [(&policy_file, &policy_ref), (&source_file, &source_ref)] {
        write_published_artifact(&transport, &published_path, file, &domain, &[]).unwrap();
        let read = read_published_artifact(&transport, &published_path, read_options).unwrap();
        assert!(recover(&read, reference).is_ok());
        assert_eq!(
            normal
                .2
                .clone()
                .stage_source_theorem_file(&read, reference)
                .unwrap_err(),
            "source theorem witness file staging mismatch"
        );
    }
    let mut wire_fields = fields.clone();
    wire_fields.insert("payload_bytes_hex".into(), CanonicalJson::string("00"));
    let wrong_wire = CanonicalJson::Object(wire_fields);
    let mut coherent_ref = normal_ref.clone();
    coherent_ref.witness_artifact_hash.digest = domain.hash(&wrong_wire, &[]);
    write_published_artifact(&transport, &published_path, &wrong_wire, &domain, &[]).unwrap();
    let read = read_published_artifact(&transport, &published_path, read_options).unwrap();
    assert_eq!(recover(&read, &coherent_ref).unwrap(), [0]);
    assert_eq!(
        normal
            .2
            .clone()
            .stage_source_theorem_file(&read, &coherent_ref)
            .unwrap_err(),
        "source theorem witness file staging mismatch"
    );
    assert_eq!(
        (normal, other_policy, other_source, genuine_builtin),
        before
    );
    for (snapshot, lease, _, _, _) in captured {
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}
