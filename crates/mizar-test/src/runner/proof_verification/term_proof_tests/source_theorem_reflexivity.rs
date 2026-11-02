use super::*;

#[test]
fn source_theorem_reflexivity_checks_genuine_original_targets() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_core::{
        control_flow::{build_control_flow_ir, build_obligation_seed_handoff},
        elaborator::lower_source_theorem_skeletons as lower,
    };
    use mizar_kernel::{
        checker::{KernelCheckStatus, KernelEvidenceCheckKind},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{CandidatePolicyClass, VerifierPolicy},
        status::{
            check_source_theorem_reflexivity as verify,
            source_theorem_canonical_inputs as canonical,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::{
        generator::{
            CoreGenerationCandidateSet as Candidates, CoreGenerationInput as Generation,
            VcNormalizationInput as Normalization, generate_source_theorem_vcs as generate,
        },
        kernel_evidence_handoff::build_source_theorem_kernel_handoff as project,
        vc_ir::*,
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut original_source = None;
    let mut prior = Vec::new();
    for (text, package, module, skip) in [
        (original.to_owned(), "owner-test", "owners", 0),
        (
            original.replace(
                "for X being set holds X = X",
                "for Bound being set holds Bound = Bound",
            ),
            "owner-test",
            "owners",
            0,
        ),
        (
            original.replace(
                "let X be set; thus X = X",
                "let Local be set; thus Local = Local",
            ),
            "owner-test",
            "owners",
            0,
        ),
        (
            format!(":: 日本語 λ\n{original}\n"),
            "owner-test",
            "owners",
            0,
        ),
        (original.to_owned(), "owner-test", "owners", 7),
        (original.to_owned(), "other-package", "owners", 0),
        (original.to_owned(), "owner-test", "nested.owners", 0),
    ] {
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let symbols = collected.env();
        let before = (source.clone(), symbols.clone(), prior.clone());
        if skip == 7 {
            assert_ne!(Some(source.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(source.source_id());
        }
        with_source_theorem_check(ast, &source, symbols, |check| {
            let check_before = format!("{check:?}");
            let core = lower(check, None)?;
            let generation_schema = GenerationSchemaVersion::new("source-theorem-local-v1");
            let vc_schema = VcSchemaVersion::new("vc-v1");
            for ordinal in [777, 778] {
                let snapshot = snapshot_id(ordinal);
                // Independent original generic construction, with the Proof schemas.
                let flow = build_control_flow_ir(&core);
                let handoff = build_obligation_seed_handoff(&core, &flow);
                let intake = SeedIntakeTable::try_from_handoff(&handoff)
                    .map_err(|error| error.to_string())?;
                let package = core.module_id().package().as_str();
                let path = core.module_id().path().as_str();
                let module = VcModuleRef::new(format!(
                    "package={}:{};module={}:{}",
                    package.len(),
                    package,
                    path.len(),
                    path
                ));
                let candidates = Candidates::try_from_seed_intake(Generation {
                    schema_version: &generation_schema,
                    module: &module,
                    intake: &intake,
                    handoff: &handoff,
                    flow_output: Some(&flow),
                })
                .map_err(|error| error.to_string())?;
                let expected = Candidates::try_normalize(Normalization {
                    schema_version: &vc_schema,
                    snapshot,
                    source: core.source_id(),
                    candidates: &candidates,
                })
                .map_err(|error| error.to_string())?;
                let graph = generate(&core, snapshot, &generation_schema, &vc_schema)?;
                assert_eq!(graph, expected);
                assert_eq!(graph.vcs().len(), 1);
                let vc = &expected.vcs()[0];
                assert_eq!(vc.status, VcStatus::Open);
                let original_handoff = project(&core, &expected, vc.id)?;
                let target = original_handoff.canonical_evidence().target_vc();
                let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
                let policy = VerifierPolicy::default();
                let old = canonical(check, snapshot, vc.id)?;
                let old_identity = identity(&source, check, symbols, snapshot, vc.id)?;
                assert_eq!(old_identity.1, old.3);
                let immutable = (
                    core.clone(),
                    graph.clone(),
                    expected.clone(),
                    policy.clone(),
                );
                let actual = verify(&source, check, symbols, snapshot, vc.id, &policy)?;
                assert_eq!(actual.0.status(), KernelCheckStatus::Accepted);
                assert_eq!(actual.0.target_vc_fingerprint(), &target);
                assert_eq!(
                    actual.0.evidence_check_kind(),
                    Some(KernelEvidenceCheckKind::ProofObligation)
                );
                assert!(!actual.0.policy_taint());
                assert!(actual.0.used_axioms().is_empty());
                assert_eq!(actual.1.class, CandidatePolicyClass::DischargedBuiltin);
                assert_eq!(
                    actual,
                    verify(&source, check, symbols, snapshot, vc.id, &policy)?
                );
                assert_eq!(old, canonical(check, snapshot, vc.id)?);
                assert_eq!(
                    old_identity,
                    identity(&source, check, symbols, snapshot, vc.id)?
                );
                assert_eq!(
                    (&core, &graph, &expected, &policy),
                    (&immutable.0, &immutable.1, &immutable.2, &immutable.3)
                );
                prior.push(actual);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, symbols), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
    }
}

#[test]
fn source_theorem_reflexivity_enforces_actual_builtin_policy_formats() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::check_source_theorem_reflexivity as verify,
    };
    use mizar_vc::vc_ir::VcId;
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let (frontend, source, collected) = inputs(text, "owner-test", "owners", 0);
    assert!(frontend.diagnostics.is_empty());
    assert!(collected.diagnostics().is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let symbols = collected.env();
    let before = (source.clone(), symbols.clone());
    with_source_theorem_check(ast, &source, symbols, |check| {
        let check_before = format!("{check:?}");
        for policy in [
            VerifierPolicy::default(),
            VerifierPolicy::release().with_require_kernel_certificates(true),
            VerifierPolicy::development()
                .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
            VerifierPolicy::default().with_kernel_evidence_formats([]),
            VerifierPolicy::release()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
        ] {
            let before = policy.clone();
            let actual = verify(
                &source,
                check,
                symbols,
                snapshot_id(777),
                VcId::new(0),
                &policy,
            );
            if policy
                .kernel_evidence_formats()
                .contains(&KernelEvidenceFormat::BuiltinKernelEvidence)
            {
                let (result, decision) = actual?;
                assert_eq!(result.status(), KernelCheckStatus::Accepted);
                assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
            } else {
                assert_eq!(actual.unwrap_err(), "theorem.proof.kernel_policy_rejected");
            }
            assert_eq!(policy, before);
        }
        assert_eq!(format!("{check:?}"), check_before);
        Ok(())
    })
    .unwrap();
    assert_eq!((source, symbols.clone()), before);
}

#[test]
fn source_theorem_reflexivity_preserves_upstream_errors_and_inputs() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_checker::type_checker::SourceVariableSemanticsChecker as Checker;
    use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
    use mizar_proof::{policy::VerifierPolicy, status::check_source_theorem_reflexivity as verify};
    use mizar_vc::{
        kernel_evidence_handoff::build_source_theorem_kernel_handoff as project, vc_ir::VcId,
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snapshot = snapshot_id(777);
    let absent = VcId::new(usize::MAX);
    let disabled = VerifierPolicy::default().with_kernel_evidence_formats([]);
    let policy_before = disabled.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    // The unchanged alternate-schema generator supplies only VC69 error oracles here;
    // no alternate-schema graph supplies an expected successful target.
    for text in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ] {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let symbols = collected.env();
        let before = (source.clone(), symbols.clone());
        with_source_theorem_check(ast, &source, symbols, |check| {
            let check_before = format!("{check:?}");
            if text == original {
                let policy = VerifierPolicy::default();
                let prior = verify(&source, check, symbols, snapshot, VcId::new(0), &policy)?;
                let prior_before = prior.clone();
                let core = lower(check, None)?;
                let vcs = generate_core_vcs(&core, snapshot)?;
                assert!(!vcs.vcs().is_empty());
                let expected = project(&core, &vcs, absent).unwrap_err();
                assert_eq!(
                    verify(&source, check, symbols, snapshot, absent, &disabled).unwrap_err(),
                    expected
                );
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, collected.env()),
                    ] {
                        let expected = Checker::check_theorem_skeletons(
                            current,
                            check.typed_ast(),
                            check.scope(),
                            symbols,
                            check.labels(),
                            check.resolved_labels(),
                            None,
                        )
                        .unwrap_err();
                        for selected in [VcId::new(0), absent] {
                            assert_eq!(
                                verify(current, check, symbols, snapshot, selected, &disabled)
                                    .unwrap_err(),
                                expected
                            );
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(prior, prior_before);
                assert_eq!(
                    prior,
                    verify(&source, check, symbols, snapshot, VcId::new(0), &policy)?
                );
            } else {
                let core = lower(check, None);
                let expected = match &core {
                    Err(error) => error.clone(),
                    Ok(core) => {
                        let vcs = generate_core_vcs(core, snapshot)?;
                        assert!(!vcs.vcs().is_empty());
                        assert!(vcs.vc(VcId::new(0)).is_some());
                        project(core, &vcs, VcId::new(0)).unwrap_err()
                    }
                };
                assert_eq!(
                    verify(&source, check, symbols, snapshot, VcId::new(0), &disabled).unwrap_err(),
                    expected
                );
                let expected = match &core {
                    Err(error) => error.clone(),
                    Ok(core) => {
                        project(core, &generate_core_vcs(core, snapshot)?, absent).unwrap_err()
                    }
                };
                assert_eq!(
                    verify(&source, check, symbols, snapshot, absent, &disabled).unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, symbols.clone()), before);
    }
    assert_eq!(disabled, policy_before);
}

#[test]
fn source_theorem_reflexivity_selection_matches_fresh_actual_evidence() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_kernel::checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind};
    use mizar_proof::{
        policy::{
            CandidatePolicyClass as Class, KernelEvidenceFormat as Format,
            KernelEvidenceOrigin as Origin, KernelPolicyInput as Input, VerifierPolicy as Policy,
        },
        selection::{
            CandidateSourceId as Id, ProofEvidenceCandidate as Candidate, ProofEvidenceSet as Set,
            ProofWinnerClass as Winner, ProofWitnessPublication as Publication, select_winner,
        },
        status::{
            check_source_theorem_reflexivity as verify, select_source_theorem_reflexivity as select,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut original_source = None;
    let mut prior = Vec::new();
    let bound = original.replace(
        "for X being set holds X = X",
        "for Bound being set holds Bound = Bound",
    );
    let local = original.replace(
        "let X be set; thus X = X",
        "let Local be set; thus Local = Local",
    );
    let trivia = format!(":: 日本語 λ\n{original}\n");
    for (index, (text, package, module, skip)) in [
        (original.to_owned(), "owner-test", "owners", 0),
        (bound, "owner-test", "owners", 0),
        (local, "owner-test", "owners", 0),
        (trivia, "owner-test", "owners", 0),
        (original.to_owned(), "owner-test", "owners", 7),
        (original.to_owned(), "other-package", "owners", 0),
        (original.to_owned(), "owner-test", "nested.owners", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, s, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (s.clone(), env.clone(), prior.clone());
        if skip == 7 {
            assert_ne!(Some(s.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(s.source_id());
        }
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, env, |check| {
            let check_before = format!("{check:?}");
            for (policy_index, policy) in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("theorem-selection"),
                Policy::default().with_schema_version(2),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 6 } else { 1 })
            .enumerate()
            {
                let snap = snapshot_id(777 + usize::from(index != 0 || policy_index != 0));
                let vc = VcId::new(0);
                let policy_before = policy.clone();
                let raw = verify(&s, check, env, snap, vc, &policy)?;
                let raw_before = raw.clone();
                assert_eq!(raw.0.status(), Status::Accepted);
                assert_eq!(raw.0.evidence_check_kind(), Some(Kind::ProofObligation));
                assert!(!raw.0.policy_taint());
                assert!(raw.0.used_axioms().is_empty());
                assert_eq!(raw.1.class, Class::DischargedBuiltin);
                let target = raw.0.target_vc_fingerprint().sort_bytes();
                let hex: String = target.iter().map(|byte| format!("{byte:02x}")).collect();
                let id = Id::new(format!("source-theorem-reflexivity-v1:{hex}")).unwrap();
                let input = Input::from_kernel_result(&raw.0, Origin::BuiltinDischarge);
                let candidate = Candidate::from_trusted_kernel_input(id.clone(), &input).unwrap();
                let expected = select_winner(
                    &Set::new(target, None, policy.clone()).with_candidates([candidate]),
                );
                let chosen = select(&s, check, env, snap, vc, &policy)?;
                let chosen_before = chosen.clone();
                assert_eq!(chosen, expected);
                assert_eq!(chosen.selected_candidate_id(), Some(&id));
                assert_eq!(chosen.selected_class(), Winner::DischargedBuiltin);
                assert!(chosen.trusted_used_axioms_available());
                let meta = chosen.reuse_metadata();
                assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
                let hash = input.accepted_evidence_hash().unwrap();
                let polarity = input.accepted_goal_polarity().unwrap();
                assert_eq!(meta.selected_evidence_hash(), Some(hash));
                assert_eq!(meta.accepted_goal_polarity(), Some(polarity));
                assert_eq!(meta.encoded_problem_hash(), None);
                assert_eq!(meta.selected_candidate_provenance_hash(), None);
                assert_eq!(meta.deterministic_discharge_hash(), None);
                assert_eq!(meta.selected_proof_witness_hash(), None);
                assert_eq!(
                    meta.proof_witness_publication(),
                    Publication::ExternalDependencyGap
                );
                assert_eq!(chosen, select(&s, check, env, snap, vc, &policy)?);
                assert_eq!(raw, verify(&s, check, env, snap, vc, &policy)?);
                assert_eq!(
                    (&raw, &chosen, &policy),
                    (&raw_before, &chosen_before, &policy_before)
                );
                prior.push((raw, chosen));
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&s, env), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
    }
}

#[test]
fn source_theorem_reflexivity_selection_preserves_actual_failures() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            check_source_theorem_reflexivity as verify, select_source_theorem_reflexivity as select,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snap = snapshot_id(777);
    let absent = VcId::new(usize::MAX);
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for text in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ] {
        let (frontend, s, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (s.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, env, |check| {
            let check_before = format!("{check:?}");
            for (index, policy) in policies.iter().enumerate() {
                for vc in [VcId::new(0), absent] {
                    if text == original && index == 0 && vc != absent {
                        continue;
                    }
                    let expected = verify(&s, check, env, snap, vc, policy).unwrap_err();
                    assert_eq!(
                        select(&s, check, env, snap, vc, policy).unwrap_err(),
                        expected
                    );
                }
            }
            if text == original {
                let vc = VcId::new(0);
                let policy = &policies[0];
                let prior = (
                    verify(&s, check, env, snap, vc, policy)?,
                    select(&s, check, env, snap, vc, policy)?,
                );
                let prior_before = prior.clone();
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, env) in [
                        (&other_source, other.env()),
                        (&s, other.env()),
                        (&other_source, collected.env()),
                    ] {
                        for vc in [VcId::new(0), absent] {
                            let expected =
                                verify(current, check, env, snap, vc, &policies[1]).unwrap_err();
                            assert_eq!(
                                select(current, check, env, snap, vc, &policies[1]).unwrap_err(),
                                expected
                            );
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(prior, prior_before);
                assert_eq!(
                    prior,
                    (
                        verify(&s, check, env, snap, vc, policy)?,
                        select(&s, check, env, snap, vc, policy)?
                    )
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((s, env.clone()), before);
    }
    assert_eq!(policies, policies_before);
}

#[test]
fn source_theorem_reflexivity_status_matches_existing_projection_and_full_slice() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        selection::{
            ProofWinnerClass as Winner, VcProofSelection, merge_artifact_proof_selections as merge,
        },
        status::{
            ArtifactPublicationGap as Gap, ArtifactStatusPublication as Publication,
            ProjectedProofStatus as Status, ProofStatusProjectionInput as Input,
            project_source_theorem_reflexivity_status as project, project_status,
            select_source_theorem_reflexivity as select,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness as Completeness,
        vc_ir::{VcId, VcKind, VcStatus},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let bound = original.replace(
        "for X being set holds X = X",
        "for Bound being set holds Bound = Bound",
    );
    let local = original.replace(
        "let X be set; thus X = X",
        "let Local be set; thus Local = Local",
    );
    let trivia = format!(":: 日本語 λ\n{original}\n");
    let mut original_source = None;
    let mut prior = Vec::new();
    for (index, (text, package, module, skip)) in [
        (original.to_owned(), "owner-test", "owners", 0),
        (bound, "owner-test", "owners", 0),
        (local, "owner-test", "owners", 0),
        (trivia, "owner-test", "owners", 0),
        (original.to_owned(), "owner-test", "owners", 7),
        (original.to_owned(), "other-package", "owners", 0),
        (original.to_owned(), "owner-test", "nested.owners", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone(), prior.clone());
        if skip == 7 {
            assert_ne!(Some(source.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(source.source_id());
        }
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for (policy_index, policy) in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("theorem-status"),
                Policy::default().with_schema_version(2),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 6 } else { 1 })
            .enumerate()
            {
                let snap = snapshot_id(777 + usize::from(index != 0 || policy_index != 0));
                let vc = VcId::new(0);
                let policy_before = policy.clone();
                let original = identity(&source, check, env, snap, vc)?;
                let chosen = select(&source, check, env, snap, vc, &policy)?;
                let inputs_before = (original.clone(), chosen.clone());
                assert_eq!(chosen.selected_class(), Winner::DischargedBuiltin);
                assert!(chosen.trusted_used_axioms_available());
                let [merged] = <[_; 1]>::try_from(
                    merge([], [VcProofSelection::new(vc, chosen.clone())]).unwrap(),
                )
                .unwrap();
                let expected =
                    project_status(Input::new(merged, policy.clone(), original.0.clone())).unwrap();
                let actual = project(&source, check, env, snap, vc, &policy)?;
                let actual_before = actual.clone();
                assert_eq!(actual.0, expected);
                assert_eq!(actual.0.vc(), vc);
                assert_eq!(actual.0.obligation_id(), original.0.obligation_id());
                assert_eq!(actual.0.selected_class(), Winner::DischargedBuiltin);
                assert_eq!(actual.0.projected_status(), Status::Accepted);
                assert_eq!(
                    actual.0.artifact_publication(),
                    Publication::ExternalDependencyGap(Gap::DischargedBuiltinWitnessUnsupported)
                );
                assert_eq!(actual.0.accepted_witness_obligation_id(), None);
                assert_eq!(actual.0.trusted_used_axioms(), None);
                assert_eq!(actual.0.explanation_ref(), None);
                let meta = actual.0.reuse_metadata();
                let id = &original.0;
                let selected = chosen.reuse_metadata();
                assert_eq!(meta.obligation_anchor(), id.obligation_anchor());
                assert_eq!(meta.obligation_fingerprint(), id.obligation_fingerprint());
                assert_eq!(meta.vc_fingerprint(), id.vc_fingerprint());
                assert_eq!(
                    meta.local_context_fingerprint(),
                    id.local_context_fingerprint()
                );
                assert_eq!(
                    meta.dependency_slice_fingerprint(),
                    id.dependency_slice_fingerprint()
                );
                assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
                assert_eq!(meta.selected_candidate_id(), chosen.selected_candidate_id());
                assert_eq!(
                    meta.selected_evidence_hash(),
                    selected.selected_evidence_hash()
                );
                assert!(meta.selected_evidence_hash().is_some());
                assert_eq!(
                    meta.accepted_goal_polarity(),
                    selected.accepted_goal_polarity()
                );
                assert!(meta.accepted_goal_polarity().is_some());
                assert_eq!(meta.trusted_used_axioms_hash(), None);
                assert_eq!(meta.selected_proof_witness_hash(), None);
                assert_eq!(meta.deterministic_discharge_hash(), None);
                assert_eq!(meta.selected_candidate_provenance_hash(), None);
                assert_eq!(meta.dependency_compatibility(), None);
                assert_eq!(meta.explanation_ref(), None);
                assert!(!meta.cache_reuse_predicate_complete());
                assert_eq!(actual.1, original.1);
                assert_eq!(actual.1.vc(), vc);
                assert_eq!(actual.1.kind(), &VcKind::TheoremProofStep);
                assert_eq!(actual.1.status(), &VcStatus::Open);
                assert_eq!(actual.1.entries(), original.1.entries());
                assert!(!actual.1.entries().is_empty());
                assert_eq!(actual.1.unknowns(), original.1.unknowns());
                assert!(!actual.1.unknowns().is_empty());
                assert_eq!(actual.1.completeness(), Completeness::IncompleteUncacheable);
                assert_eq!(actual.1.fingerprint(), original.1.fingerprint());
                assert_eq!(
                    actual.1.fingerprint().hash(),
                    meta.dependency_slice_fingerprint()
                );
                assert!(actual.1.requires_cache_miss());
                assert_eq!(actual, project(&source, check, env, snap, vc, &policy)?);
                assert_eq!(original, identity(&source, check, env, snap, vc)?);
                assert_eq!(chosen, select(&source, check, env, snap, vc, &policy)?);
                assert_eq!((&original, &chosen), (&inputs_before.0, &inputs_before.1));
                assert_eq!((&actual, &policy), (&actual_before, &policy_before));
                prior.push(actual);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
    }
}

#[test]
fn source_theorem_reflexivity_status_preserves_actual_error_order() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            project_source_theorem_reflexivity_status as project,
            select_source_theorem_reflexivity as select,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snap = snapshot_id(777);
    let absent = VcId::new(usize::MAX);
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (case, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for (policy_index, policy) in policies.iter().enumerate() {
                for vc in [VcId::new(0), absent] {
                    let actual_identity = identity(&source, check, env, snap, vc);
                    let expected = match actual_identity {
                        Err(error) => {
                            let selection_error =
                                select(&source, check, env, snap, vc, policy).unwrap_err();
                            if case == 3 {
                                assert_eq!(error, "unsupported source theorem formula payload");
                                assert_eq!(selection_error, "theorem.handoff.invalid_projection");
                                assert_ne!(error, selection_error);
                            }
                            error
                        }
                        Ok(original) => {
                            assert_eq!(case, 0);
                            assert_eq!(vc, VcId::new(0));
                            assert!(original.1.requires_cache_miss());
                            if policy_index == 0 {
                                continue;
                            }
                            let error = select(&source, check, env, snap, vc, policy).unwrap_err();
                            assert_eq!(original, identity(&source, check, env, snap, vc)?);
                            error
                        }
                    };
                    assert_eq!(
                        project(&source, check, env, snap, vc, policy).unwrap_err(),
                        expected
                    );
                }
            }
            if case == 0 {
                let vc = VcId::new(0);
                let original_identity = identity(&source, check, env, snap, vc)?;
                let prior = project(&source, check, env, snap, vc, &policies[0])?;
                let prior_before = prior.clone();
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, env),
                    ] {
                        for selected in [vc, absent] {
                            let expected =
                                identity(current, check, symbols, snap, selected).unwrap_err();
                            select(current, check, symbols, snap, selected, &policies[1])
                                .unwrap_err();
                            assert_eq!(
                                project(current, check, symbols, snap, selected, &policies[1])
                                    .unwrap_err(),
                                expected
                            );
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(original_identity, identity(&source, check, env, snap, vc)?);
                assert_eq!(prior, project(&source, check, env, snap, vc, &policies[0])?);
                assert_eq!(prior, prior_before);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(policies, policies_before);
}

#[test]
fn source_theorem_reflexivity_metadata_matches_actual_wire_and_fields() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_artifact::{
        proof_witness::KernelAcceptanceMetadata,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::SchemaVersion,
    };
    use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass as Class, KernelEvidenceFormat as Format,
            KernelEvidenceOrigin as Origin, KernelPolicyInput, VerifierPolicy as Policy,
        },
        status::{
            check_source_theorem_reflexivity as verify,
            check_source_theorem_reflexivity_metadata as metadata,
            project_source_theorem_reflexivity_status as status,
            select_source_theorem_reflexivity as select,
        },
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_theorem_kernel_handoff as project,
        },
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcStatus},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let bound = original.replace(
        "for X being set holds X = X",
        "for Bound being set holds Bound = Bound",
    );
    let local = original.replace(
        "let X be set; thus X = X",
        "let Local be set; thus Local = Local",
    );
    let trivia = format!(":: 日本語 λ\n{original}\n");
    let mut original_source = None;
    let mut prior = Vec::new();
    for (index, (text, package, module, skip)) in [
        (original.to_owned(), "owner-test", "owners", 0),
        (bound, "owner-test", "owners", 0),
        (local, "owner-test", "owners", 0),
        (trivia, "owner-test", "owners", 0),
        (original.to_owned(), "owner-test", "owners", 7),
        (original.to_owned(), "other-package", "owners", 0),
        (original.to_owned(), "owner-test", "nested.owners", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone(), prior.clone());
        if skip == 7 {
            assert_ne!(Some(source.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(source.source_id());
        }
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for policy in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("theorem-metadata"),
                Policy::default().with_schema_version(2),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 6 } else { 1 })
            {
                for ordinal in [777, 778] {
                    let snap = snapshot_id(ordinal);
                    let vc = VcId::new(0);
                    let policy_before = policy.clone();
                    let raw = verify(&source, check, env, snap, vc, &policy)?;
                    let raw_before = raw.clone();
                    let core = lower(check, None)?;
                    let graph = generate(
                        &core,
                        snap,
                        &GenerationSchemaVersion::new("source-theorem-local-v1"),
                        &VcSchemaVersion::new("vc-v1"),
                    )?;
                    assert_eq!(graph.vcs()[0].status, VcStatus::Open);
                    let handoff = project(&core, &graph, vc)?;
                    let envelope = handoff.canonical_evidence();
                    let target = envelope.target_vc();
                    let profile = envelope.kernel_profile();
                    let tautology = match profile.clause_tautology_policy {
                        KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                        KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                        _ => panic!("unsupported actual theorem profile"),
                    };
                    let mut context = FormulaEvidenceParseContext::v1(
                        Fingerprint::new(target.algorithm_id, target.digest.clone()),
                        KernelProfileRecord::v1(profile.profile_id, tautology),
                    );
                    let v1 = handoff.encode_local_evidence(None)?;
                    let parsed = parse_formula_evidence(&v1, &context).unwrap();
                    assert_eq!(parsed.schema_version(), 1);
                    let names =
                        mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed)?;
                    assert!(!names.is_empty());
                    let wire = handoff.encode_local_evidence(Some(&names))?;
                    assert_ne!(wire, v1);
                    context.accepted_schema_versions.insert(2);
                    let named = parse_formula_evidence(&wire, &context).unwrap();
                    assert_eq!(named.schema_version(), 2);
                    assert_eq!(named.equality_instances().len(), names.len());
                    for name in &names {
                        assert!(wire.windows(name.len()).any(|bytes| bytes == name));
                    }
                    let components = handoff.local_evidence_component_hashes()?;
                    let input =
                        KernelPolicyInput::from_kernel_result(&raw.0, Origin::BuiltinDischarge);
                    assert_eq!(raw.0.status(), Status::Accepted);
                    assert_eq!(raw.0.evidence_check_kind(), Some(Kind::ProofObligation));
                    assert!(!raw.0.policy_taint());
                    assert_eq!(raw.1.class, Class::DischargedBuiltin);
                    assert_eq!(
                        input.accepted_goal_polarity(),
                        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                    );
                    let accepted = input.accepted_evidence_hash().unwrap();
                    let families = [
                        "mizar-vc/local-evidence-component/profile",
                        "mizar-vc/local-evidence-component/formula",
                        "mizar-vc/local-evidence-component/substitutions",
                        "mizar-vc/local-evidence-component/provenance",
                        "mizar-proof/verifier-policy",
                        "mizar-vc/kernel-evidence-handoff-canonical",
                        "mizar-proof/kernel-accepted-evidence",
                    ];
                    let digests = [
                        components[0],
                        components[1],
                        components[2],
                        components[3],
                        policy.policy_fingerprint().hash(),
                        handoff.canonical_hash(),
                        accepted,
                    ];
                    let [
                        profile,
                        formula,
                        substitutions,
                        provenance,
                        verifier,
                        target,
                        result,
                    ] = std::array::from_fn(|i| ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: families[i].into(),
                        schema_version: SchemaVersion::new(
                            if i == 4 { policy.schema_version() } else { 1 },
                            0,
                        ),
                        digest: digests[i],
                    });
                    let expected = KernelAcceptanceMetadata {
                        kernel_profile_fingerprint: profile,
                        verifier_policy_fingerprint: verifier,
                        checker_schema_version: SchemaVersion::new(
                            policy.checker_schema_version(),
                            0,
                        ),
                        evidence_schema_version: SchemaVersion::new(named.schema_version(), 0),
                        target_binding_hash: target,
                        formula_evidence_hash: formula,
                        substitution_evidence_hash: substitutions,
                        provenance_hash: provenance,
                        formula_context_hash: None,
                        accepted_result_hash: result,
                    };
                    let chosen = select(&source, check, env, snap, vc, &policy)?;
                    let projected = status(&source, check, env, snap, vc, &policy)?;
                    let old_before = (chosen.clone(), projected.clone());
                    let actual = metadata(&source, check, env, snap, vc, &policy)?;
                    assert_eq!((&actual.0, &actual.1), (&raw.0, &raw.1));
                    assert_eq!(actual.2.as_ref(), Some(&expected));
                    assert_eq!(actual.3, wire);
                    assert_eq!(parse_formula_evidence(&actual.3, &context).unwrap(), named);
                    assert_eq!(
                        chosen.reuse_metadata().selected_evidence_hash(),
                        Some(accepted)
                    );
                    assert!(projected.1.requires_cache_miss());
                    assert!(
                        !projected
                            .0
                            .reuse_metadata()
                            .cache_reuse_predicate_complete()
                    );
                    assert_eq!(actual, metadata(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(raw, verify(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(chosen, select(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(projected, status(&source, check, env, snap, vc, &policy)?);
                    assert_eq!((chosen, projected), old_before);
                    assert_eq!(raw, raw_before);
                    assert_eq!(policy, policy_before);
                    prior.push(actual);
                }
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
    }
}

#[test]
fn source_theorem_reflexivity_metadata_preserves_compatibility_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            check_source_theorem_reflexivity as verify,
            check_source_theorem_reflexivity_metadata as metadata,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snap = snapshot_id(777);
    let absent = VcId::new(usize::MAX);
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for text in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ] {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for (index, policy) in policies.iter().enumerate() {
                for vc in [VcId::new(0), absent] {
                    if text == original && index == 0 && vc != absent {
                        continue;
                    }
                    let expected = verify(&source, check, env, snap, vc, policy).unwrap_err();
                    assert_eq!(
                        metadata(&source, check, env, snap, vc, policy).unwrap_err(),
                        expected
                    );
                }
            }
            if text == original {
                let vc = VcId::new(0);
                let policy = &policies[0];
                let prior = (
                    verify(&source, check, env, snap, vc, policy)?,
                    metadata(&source, check, env, snap, vc, policy)?,
                );
                let prior_before = prior.clone();
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, env),
                    ] {
                        for policy in &policies {
                            for selected in [vc, absent] {
                                let expected =
                                    verify(current, check, symbols, snap, selected, policy)
                                        .unwrap_err();
                                assert_eq!(
                                    metadata(current, check, symbols, snap, selected, policy)
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(prior, prior_before);
                assert_eq!(
                    prior,
                    (
                        verify(&source, check, env, snap, vc, policy)?,
                        metadata(&source, check, env, snap, vc, policy)?,
                    )
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(policies, policies_before);
}
#[test]
fn source_theorem_reflexivity_witness_matches_original_identity_and_checked_wire() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessReadOptions, ProofWitnessRef,
            builtin_proof_witness_ref_json, read_builtin_proof_witness_ref,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalHashDomain as Domain, CanonicalJson as J, HashClass, SchemaVersion},
    };
    use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::selection::{
        CandidateSourceId as Id, ProofEvidenceCandidate as Candidate, ProofEvidenceSet as Set,
        ProofWinnerClass, ProofWitnessPublication, select_winner,
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass as Class, KernelEvidenceFormat as Format,
            KernelEvidenceOrigin as Origin, KernelPolicyInput, VerifierPolicy as Policy,
        },
        status::{
            check_source_theorem_reflexivity as verify,
            check_source_theorem_reflexivity_metadata as metadata,
            project_source_theorem_reflexivity_status as status,
            project_source_theorem_reflexivity_witness as witness,
            select_source_theorem_reflexivity as select,
            source_theorem_canonical_inputs as canonical,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_theorem_kernel_handoff as project,
        },
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcStatus},
    };
    let id = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let bound = id.replace(
        "for X being set holds X = X",
        "for Bound being set holds Bound = Bound",
    );
    let local = id.replace(
        "let X be set; thus X = X",
        "let Local be set; thus Local = Local",
    );
    let trivia = format!(":: 日本語 λ\n{id}\n");
    let mut original_source = None;
    let mut prior = Vec::new();
    for (index, (text, package, module, skip)) in [
        (id.to_owned(), "owner-test", "owners", 0),
        (bound, "owner-test", "owners", 0),
        (local, "owner-test", "owners", 0),
        (trivia, "owner-test", "owners", 0),
        (id.to_owned(), "owner-test", "owners", 7),
        (id.to_owned(), "other-package", "owners", 0),
        (id.to_owned(), "owner-test", "nested.owners", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, s, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let e = collected.env();
        let before = (s.clone(), e.clone(), prior.clone());
        if skip == 7 {
            assert_ne!(Some(s.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(s.source_id());
        }
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, e, |c| {
            let check_before = format!("{c:?}");
            for p in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("theorem-metadata"),
                Policy::default().with_schema_version(2),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 6 } else { 1 })
            {
                for ordinal in [777, 778] {
                    let sn = snapshot_id(ordinal);
                    let vc = VcId::new(0);
                    let policy_before = p.clone();
                    let id = identity(&s, c, e, sn, vc)?;
                    let content = canonical(c, sn, vc)?;
                    let r = verify(&s, c, e, sn, vc, &p)?;
                    let old = metadata(&s, c, e, sn, vc, &p)?;
                    assert_eq!((&old.0, &old.1), (&r.0, &r.1));
                    let core = lower(c, None)?;
                    let graph = generate(
                        &core,
                        sn,
                        &GenerationSchemaVersion::new("source-theorem-local-v1"),
                        &VcSchemaVersion::new("vc-v1"),
                    )?;
                    assert_eq!(graph.vcs()[0].status, VcStatus::Open);
                    let handoff = project(&core, &graph, vc)?;
                    let envelope = handoff.canonical_evidence();
                    let target = envelope.target_vc();
                    let profile = envelope.kernel_profile();
                    let tautology = match profile.clause_tautology_policy {
                        KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                        KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                        _ => panic!("unsupported actual theorem profile"),
                    };
                    let mut context = FormulaEvidenceParseContext::v1(
                        Fingerprint::new(target.algorithm_id, target.digest.clone()),
                        KernelProfileRecord::v1(profile.profile_id, tautology),
                    );
                    let v1 = handoff.encode_local_evidence(None)?;
                    let parsed = parse_formula_evidence(&v1, &context).unwrap();
                    assert_eq!(parsed.schema_version(), 1);
                    let names =
                        mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed)?;
                    assert!(!names.is_empty());
                    let wire = handoff.encode_local_evidence(Some(&names))?;
                    assert_ne!(wire, v1);
                    context.accepted_schema_versions.insert(2);
                    let named = parse_formula_evidence(&wire, &context).unwrap();
                    assert_eq!(named.schema_version(), 2);
                    assert_eq!(named.equality_instances().len(), names.len());
                    for name in &names {
                        assert!(wire.windows(name.len()).any(|bytes| bytes == name));
                    }
                    let input =
                        KernelPolicyInput::from_kernel_result(&r.0, Origin::BuiltinDischarge);
                    assert_eq!(r.0.status(), Status::Accepted);
                    assert_eq!(r.0.evidence_check_kind(), Some(Kind::ProofObligation));
                    assert!(!r.0.policy_taint());
                    assert_eq!(r.1.class, Class::DischargedBuiltin);
                    assert_eq!(
                        input.accepted_goal_polarity(),
                        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                    );
                    let accepted = input.accepted_evidence_hash().unwrap();
                    let m = old.2.clone().unwrap();
                    assert_eq!(m.accepted_result_hash.digest, accepted);
                    assert_eq!(m.target_binding_hash.digest, handoff.canonical_hash());
                    assert_eq!(
                        m.checker_schema_version,
                        SchemaVersion::new(p.checker_schema_version(), 0)
                    );
                    assert_eq!(
                        m.evidence_schema_version,
                        SchemaVersion::new(named.schema_version(), 0)
                    );
                    assert_eq!(m.formula_context_hash, None);
                    let choice = select(&s, c, e, sn, vc, &p)?;
                    let state = status(&s, c, e, sn, vc, &p)?;
                    assert_eq!(id.0.obligation_id(), id.0.obligation_anchor().as_str());
                    assert_eq!(id.0.vc_fingerprint(), content.0.hash());
                    assert_eq!(id.0.local_context_fingerprint(), content.2.hash());
                    assert_eq!(
                        id.0.dependency_slice_fingerprint(),
                        content.3.fingerprint().hash()
                    );
                    assert_eq!(id.1, content.3);
                    assert!(!id.1.entries().is_empty());
                    assert!(!id.1.unknowns().is_empty());
                    assert!(!id.1.is_complete());
                    assert!(id.1.requires_cache_miss());
                    let checked_target = mizar_kernel::rejection::TargetVcFingerprint::new(
                        target.algorithm_id,
                        target.digest.clone(),
                    );
                    let target = r.0.target_vc_fingerprint().sort_bytes();
                    let target_hex: String = target.iter().map(|b| format!("{b:02x}")).collect();
                    let candidate_id =
                        Id::new(format!("source-theorem-reflexivity-v1:{target_hex}")).unwrap();
                    let candidate =
                        Candidate::from_trusted_kernel_input(candidate_id, &input).unwrap();
                    let expected_selection = select_winner(
                        &Set::new(target, None, p.clone()).with_candidates([candidate]),
                    );
                    assert_eq!(choice, expected_selection);
                    assert_eq!(choice.selected_class(), ProofWinnerClass::DischargedBuiltin);
                    let meta = choice.reuse_metadata();
                    assert_eq!(
                        meta.accepted_goal_polarity(),
                        input.accepted_goal_polarity()
                    );
                    assert_eq!(meta.encoded_problem_hash(), None);
                    assert_eq!(meta.selected_candidate_provenance_hash(), None);
                    assert_eq!(meta.deterministic_discharge_hash(), None);
                    assert_eq!(meta.selected_proof_witness_hash(), None);
                    assert_eq!(
                        meta.proof_witness_publication(),
                        ProofWitnessPublication::ExternalDependencyGap
                    );
                    let payload_family = "mizar-proof/source-theorem-reflexivity-witness-payload";
                    let wire_hex: String = wire.iter().map(|b| format!("{b:02x}")).collect();
                    let payload_hash = ArtifactHashRef {
                        class: ArtifactHashClass::Artifact,
                        schema_family: payload_family.into(),
                        schema_version: SchemaVersion::new(1, 0),
                        digest: Domain::new(
                            HashClass::Artifact,
                            payload_family,
                            SchemaVersion::new(1, 0),
                        )
                        .hash(&J::string(wire_hex), &[]),
                    };
                    let identity_hashes = [
                        id.0.obligation_fingerprint(),
                        handoff.context_identity_hash(),
                    ]
                    .map(|h| {
                        h.as_bytes()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    });
                    let [original_hash, local_context] = identity_hashes;
                    let keys = [
                        "original_obligation_fingerprint",
                        "checked_target_bytes",
                        "local_context_identity_hash",
                        "kernel_profile_fingerprint",
                        "verifier_policy_fingerprint",
                        "checker_schema_version",
                        "evidence_schema_version",
                        "target_binding_hash",
                        "formula_evidence_hash",
                        "substitution_evidence_hash",
                        "provenance_hash",
                        "formula_context_hash",
                        "accepted_result_hash",
                        "accepted_goal_polarity",
                        "witness_artifact_hash",
                    ];
                    let values = [
                        original_hash,
                        target_hex,
                        local_context,
                        m.kernel_profile_fingerprint.to_artifact_hash_string(),
                        m.verifier_policy_fingerprint.to_artifact_hash_string(),
                        m.checker_schema_version.to_string(),
                        m.evidence_schema_version.to_string(),
                        m.target_binding_hash.to_artifact_hash_string(),
                        m.formula_evidence_hash.to_artifact_hash_string(),
                        m.substitution_evidence_hash.to_artifact_hash_string(),
                        m.provenance_hash.to_artifact_hash_string(),
                        m.accepted_result_hash.to_artifact_hash_string(),
                        input.accepted_goal_polarity().unwrap().as_str().into(),
                        payload_hash.to_artifact_hash_string(),
                    ]
                    .map(J::string);
                    let mut values = values.into_iter().collect::<Vec<_>>();
                    values.insert(11, J::null());
                    let object = J::object(keys.into_iter().zip(values)).unwrap();
                    let family = "mizar-proof/source-theorem-reflexivity-obligation";
                    let expected_ref = ProofWitnessRef {
                        schema_version: SchemaVersion::new(3, 0),
                        obligation_id: id.0.obligation_id().into(),
                        obligation_fingerprint: ArtifactHashRef {
                            class: ArtifactHashClass::Interface,
                            schema_family: family.into(),
                            schema_version: SchemaVersion::new(1, 0),
                            digest: Domain::new(
                                HashClass::Interface,
                                family,
                                SchemaVersion::new(1, 0),
                            )
                            .hash(&object, &[]),
                        },
                        proof_status: ProofStatus::DischargedBuiltin,
                        evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
                        witness_path: format!(
                            "proof-witnesses/source-theorem-reflexivity-v1/{}.bin",
                            payload_hash
                                .digest
                                .as_bytes()
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>()
                        ),
                        witness_artifact_hash: payload_hash,
                        kernel_acceptance: m.clone(),
                    };
                    assert_ne!(
                        expected_ref.obligation_fingerprint.digest,
                        id.0.obligation_fingerprint()
                    );
                    assert_ne!(handoff.context_identity_hash(), handoff.canonical_hash());
                    assert_eq!(r.0.target_vc_fingerprint(), &checked_target);
                    assert_eq!(old.3, wire);
                    let actual = witness(&s, c, e, sn, vc, &p)?;
                    assert_eq!(actual, (expected_selection, expected_ref, wire.clone()));
                    let json = builtin_proof_witness_ref_json(&actual.1).unwrap();
                    assert_eq!(
                        read_builtin_proof_witness_ref(&json, ProofWitnessReadOptions::default())
                            .unwrap(),
                        actual.1
                    );
                    assert_eq!(actual, witness(&s, c, e, sn, vc, &p)?);
                    assert_eq!(id, identity(&s, c, e, sn, vc)?);
                    assert_eq!(old, metadata(&s, c, e, sn, vc, &p)?);
                    assert_eq!(r, verify(&s, c, e, sn, vc, &p)?);
                    assert_eq!(choice, select(&s, c, e, sn, vc, &p)?);
                    assert_eq!(state, status(&s, c, e, sn, vc, &p)?);
                    assert_eq!(state.1, id.1);
                    assert!(!state.0.reuse_metadata().cache_reuse_predicate_complete());
                    assert_eq!(p, policy_before);
                    prior.push(actual);
                }
            }
            assert_eq!(format!("{c:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&s, e), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
    }
}
#[test]
fn source_theorem_reflexivity_witness_hashes_bind_all_participants() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_artifact::{
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalHashDomain as Domain, CanonicalJson as J, HashClass, SchemaVersion},
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::{
        policy::{KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        status::{
            check_source_theorem_reflexivity_metadata as metadata,
            project_source_theorem_reflexivity_witness as witness,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs,
        kernel_evidence_handoff::build_source_theorem_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let (frontend, s, collected) = inputs(text, "owner-test", "owners", 0);
    with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, collected.env(), |c| {
        let sn = snapshot_id(777);
        let vc = VcId::new(0);
        let p = VerifierPolicy::default();
        let id = identity(&s, c, collected.env(), sn, vc)?;
        let old = metadata(&s, c, collected.env(), sn, vc, &p)?;
        let accepted = old.2.as_ref().unwrap();
        let core = mizar_core::elaborator::lower_source_theorem_skeletons(c, None)?;
        let graph = generate_source_theorem_vcs(
            &core,
            sn,
            &GenerationSchemaVersion::new("source-theorem-local-v1"),
            &VcSchemaVersion::new("vc-v1"),
        )?;
        let handoff = build_source_theorem_kernel_handoff(&core, &graph, vc)?;
        let input =
            KernelPolicyInput::from_kernel_result(&old.0, KernelEvidenceOrigin::BuiltinDischarge);
        let payload_family = "mizar-proof/source-theorem-reflexivity-witness-payload";
        let payload_domain = Domain::new(
            HashClass::Artifact,
            payload_family,
            SchemaVersion::new(1, 0),
        );
        let wire_hex: String = old.3.iter().map(|b| format!("{b:02x}")).collect();
        let payload = ArtifactHashRef {
            class: ArtifactHashClass::Artifact,
            schema_family: payload_family.into(),
            schema_version: SchemaVersion::new(1, 0),
            digest: payload_domain.hash(&J::string(wire_hex), &[]),
        };
        let identity_hashes = [
            id.0.obligation_fingerprint(),
            handoff.context_identity_hash(),
        ]
        .map(|h| {
            h.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        });
        let [original_hash, local_context] = identity_hashes;
        let keys = [
            "original_obligation_fingerprint",
            "checked_target_bytes",
            "local_context_identity_hash",
            "kernel_profile_fingerprint",
            "verifier_policy_fingerprint",
            "checker_schema_version",
            "evidence_schema_version",
            "target_binding_hash",
            "formula_evidence_hash",
            "substitution_evidence_hash",
            "provenance_hash",
            "formula_context_hash",
            "accepted_result_hash",
            "accepted_goal_polarity",
            "witness_artifact_hash",
        ];
        let values = [
            original_hash,
            old.0
                .target_vc_fingerprint()
                .sort_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            local_context,
            accepted
                .kernel_profile_fingerprint
                .to_artifact_hash_string(),
            accepted
                .verifier_policy_fingerprint
                .to_artifact_hash_string(),
            accepted.checker_schema_version.to_string(),
            accepted.evidence_schema_version.to_string(),
            accepted.target_binding_hash.to_artifact_hash_string(),
            accepted.formula_evidence_hash.to_artifact_hash_string(),
            accepted
                .substitution_evidence_hash
                .to_artifact_hash_string(),
            accepted.provenance_hash.to_artifact_hash_string(),
            accepted.accepted_result_hash.to_artifact_hash_string(),
            input.accepted_goal_polarity().unwrap().as_str().into(),
            payload.to_artifact_hash_string(),
        ]
        .map(J::string);
        let mut values = values.into_iter().collect::<Vec<_>>();
        values.insert(11, J::null());
        let object = J::object(keys.into_iter().zip(values)).unwrap();
        let domain = Domain::new(
            HashClass::Interface,
            "mizar-proof/source-theorem-reflexivity-obligation",
            SchemaVersion::new(1, 0),
        );
        let m = domain.hash(&object, &[]);
        let actual = witness(&s, c, collected.env(), sn, vc, &p)?;
        assert_eq!(actual.1.witness_artifact_hash, payload);
        assert_eq!(actual.1.obligation_fingerprint.digest, m);
        assert_eq!(actual.2, old.3);
        let J::Object(fields) = &object else {
            panic!("object");
        };
        assert_eq!(fields.len(), 15);
        // Mutations exercise only the structural hash grammar, never a forged Kernel result.
        for (key, value) in fields {
            let mut changed = fields.clone();
            changed.insert(
                key.clone(),
                match value {
                    J::String(text) => J::string(format!("{text}0")),
                    J::Null => J::string("non-null"),
                    _ => panic!("unexpected participant type"),
                },
            );
            assert_ne!(domain.hash(&J::Object(changed), &[]), m, "{key}");
        }
        assert_eq!(accepted.formula_context_hash, None);
        assert_ne!(
            handoff.context_identity_hash(),
            accepted.target_binding_hash.digest
        );
        assert_ne!(
            handoff.context_identity_hash(),
            id.0.local_context_fingerprint()
        );
        let v1 = handoff.encode_local_evidence(None)?;
        assert_ne!(old.3, v1);
        let v1_hex: String = v1.iter().map(|b| format!("{b:02x}")).collect();
        assert_ne!(payload_domain.hash(&J::string(v1_hex), &[]), payload.digest);
        let envelope = handoff.canonical_evidence();
        let target = envelope.target_vc();
        let profile = envelope.kernel_profile();
        assert_eq!(
            profile.clause_tautology_policy,
            mizar_vc::kernel_evidence_handoff::KernelClauseTautologyPolicy::Reject
        );
        let context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(target.algorithm_id, target.digest.clone()),
            KernelProfileRecord::v1(profile.profile_id, ClauseTautologyPolicy::Reject),
        );
        let parsed = parse_formula_evidence(&v1, &context).unwrap();
        let mut names = mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed)?;
        assert_eq!(handoff.encode_local_evidence(Some(&names))?, old.3);
        assert!(!names.is_empty());
        names[0][0] ^= 1;
        let changed = handoff.encode_local_evidence(Some(&names))?;
        assert_ne!(changed, old.3);
        let hex: String = changed.iter().map(|b| format!("{b:02x}")).collect();
        assert_ne!(payload_domain.hash(&J::string(hex), &[]), payload.digest);
        let mut changed = old.3.clone();
        changed.push(0);
        let hex: String = changed.iter().map(|b| format!("{b:02x}")).collect();
        assert_ne!(payload_domain.hash(&J::string(hex), &[]), payload.digest);
        Ok(())
    })
    .unwrap();
}
#[test]
fn source_theorem_reflexivity_witness_preserves_identity_first_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            check_source_theorem_reflexivity as verify,
            check_source_theorem_reflexivity_metadata as metadata,
            project_source_theorem_reflexivity_witness as witness,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snap = snapshot_id(777);
    let absent = VcId::new(usize::MAX);
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for text in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ] {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for (index, policy) in policies.iter().enumerate() {
                for vc in [VcId::new(0), absent] {
                    if text == original && index == 0 && vc != absent {
                        continue;
                    }
                    let check_error = metadata(&source, check, env, snap, vc, policy).unwrap_err();
                    let expected = match identity(&source, check, env, snap, vc) {
                        Err(_) => identity(&source, check, env, snap, vc).unwrap_err(),
                        Ok(_) => check_error,
                    };
                    assert_eq!(
                        witness(&source, check, env, snap, vc, policy).unwrap_err(),
                        expected
                    );
                }
            }
            if text == original {
                let vc = VcId::new(0);
                let policy = &policies[0];
                let prior = (
                    verify(&source, check, env, snap, vc, policy)?,
                    metadata(&source, check, env, snap, vc, policy)?,
                    witness(&source, check, env, snap, vc, policy)?,
                    identity(&source, check, env, snap, vc)?,
                );
                let prior_before = prior.clone();
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, env),
                    ] {
                        for policy in &policies {
                            for selected in [vc, absent] {
                                let expected =
                                    identity(current, check, symbols, snap, selected).unwrap_err();
                                metadata(current, check, symbols, snap, selected, policy)
                                    .unwrap_err();
                                assert_eq!(
                                    witness(current, check, symbols, snap, selected, policy)
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(prior, prior_before);
                assert_eq!(
                    prior,
                    (
                        verify(&source, check, env, snap, vc, policy)?,
                        metadata(&source, check, env, snap, vc, policy)?,
                        witness(&source, check, env, snap, vc, policy)?,
                        identity(&source, check, env, snap, vc)?,
                    )
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(policies, policies_before);
}

#[test]
fn source_theorem_reflexivity_used_axioms_status_matches_real_kernel_and_original_slice() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceFormat as Format,
            KernelEvidenceOrigin as Origin, KernelPolicyInput, VerifierPolicy as Policy,
        },
        selection::{
            ProofWinnerClass as Winner, VcProofSelection, merge_artifact_proof_selections as merge,
        },
        status::{
            ArtifactPublicationGap as Gap, ArtifactStatusPublication as Publication,
            ProjectedProofStatus as Status, ProofStatusProjectionInput as Input,
            TrustedUsedAxiomsRef, check_source_theorem_reflexivity as verify,
            project_source_theorem_reflexivity_status as legacy,
            project_source_theorem_reflexivity_used_axioms_status as project, project_status,
            select_source_theorem_reflexivity as select,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness as Completeness,
        vc_ir::{VcId, VcStatus},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let bound = original.replace(
        "for X being set holds X = X",
        "for Bound being set holds Bound = Bound",
    );
    let local = original.replace(
        "let X be set; thus X = X",
        "let Local be set; thus Local = Local",
    );
    let trivia = format!(":: 日本語 λ\n{original}\n");
    let mut original_source = None;
    let mut cases = 0;
    for (index, (text, package, module, skip)) in [
        (original.to_owned(), "owner-test", "owners", 0),
        (bound, "owner-test", "owners", 0),
        (local, "owner-test", "owners", 0),
        (trivia, "owner-test", "owners", 0),
        (original.to_owned(), "owner-test", "owners", 7),
        (original.to_owned(), "other-package", "owners", 0),
        (original.to_owned(), "owner-test", "nested.owners", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        if skip == 7 {
            assert_ne!(Some(source.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(source.source_id());
        }
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for policy in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("theorem-used-axioms"),
                Policy::default().with_schema_version(2),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 6 } else { 1 })
            {
                for ordinal in [777, 778] {
                    let snap = snapshot_id(ordinal);
                    let vc = VcId::new(0);
                    let policy_before = policy.clone();
                    let original = identity(&source, check, env, snap, vc)?;
                    let origin = Origin::BuiltinDischarge;
                    let checked = verify(&source, check, env, snap, vc, &policy)?;
                    let chosen = select(&source, check, env, snap, vc, &policy)?;
                    let old = legacy(&source, check, env, snap, vc, &policy)?;
                    let input = KernelPolicyInput::from_kernel_result(&checked.0, origin);
                    let reference =
                        TrustedUsedAxiomsRef::from_kernel_result(&checked.0, origin).unwrap();
                    assert_eq!(checked.0.status(), KernelCheckStatus::Accepted);
                    assert!(!checked.0.policy_taint());
                    assert_eq!(
                        checked.0.evidence_check_kind(),
                        Some(KernelEvidenceCheckKind::ProofObligation)
                    );
                    assert_eq!(checked.1.class, CandidatePolicyClass::DischargedBuiltin);
                    assert_eq!(reference.used_axiom_count(), checked.0.used_axioms().len());
                    assert!(checked.0.used_axioms().is_empty());
                    assert_eq!(
                        Some(reference.accepted_evidence_hash()),
                        input.accepted_evidence_hash()
                    );
                    assert_eq!(chosen.selected_class(), Winner::DischargedBuiltin);
                    assert!(chosen.trusted_used_axioms_available());
                    let [merged] = <[_; 1]>::try_from(
                        merge([], [VcProofSelection::new(vc, chosen.clone())]).unwrap(),
                    )
                    .unwrap();
                    let expected_legacy = project_status(Input::new(
                        merged.clone(),
                        policy.clone(),
                        original.0.clone(),
                    ))
                    .unwrap();
                    let expected = project_status(
                        Input::new(merged, policy.clone(), original.0.clone())
                            .with_trusted_used_axioms(reference.clone()),
                    )
                    .unwrap();
                    assert_eq!(old, (expected_legacy, original.1.clone()));
                    assert_eq!(old.0.trusted_used_axioms(), None);
                    assert_eq!(old.0.reuse_metadata().trusted_used_axioms_hash(), None);
                    let actual = project(&source, check, env, snap, vc, &policy)?;
                    assert_eq!(actual, (expected, original.1.clone()));
                    assert_eq!(actual.0.vc(), vc);
                    assert_eq!(actual.0.obligation_id(), original.0.obligation_id());
                    assert_eq!(actual.0.selected_class(), Winner::DischargedBuiltin);
                    assert_eq!(actual.0.projected_status(), Status::Accepted);
                    assert_eq!(
                        actual.0.artifact_publication(),
                        Publication::ExternalDependencyGap(
                            Gap::DischargedBuiltinWitnessUnsupported
                        )
                    );
                    assert_eq!(actual.0.trusted_used_axioms(), Some(&reference));
                    assert_eq!(actual.0.accepted_witness_obligation_id(), None);
                    assert_eq!(actual.0.explanation_ref(), None);
                    let meta = actual.0.reuse_metadata();
                    let id = &original.0;
                    assert_eq!(meta.obligation_anchor(), id.obligation_anchor());
                    assert_eq!(meta.obligation_fingerprint(), id.obligation_fingerprint());
                    assert_eq!(meta.vc_fingerprint(), id.vc_fingerprint());
                    assert_eq!(
                        meta.local_context_fingerprint(),
                        id.local_context_fingerprint()
                    );
                    assert_eq!(
                        meta.dependency_slice_fingerprint(),
                        id.dependency_slice_fingerprint()
                    );
                    assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
                    assert_eq!(meta.selected_candidate_id(), chosen.selected_candidate_id());
                    assert_eq!(
                        meta.selected_evidence_hash(),
                        Some(reference.accepted_evidence_hash())
                    );
                    assert_eq!(
                        meta.selected_evidence_hash(),
                        chosen.reuse_metadata().selected_evidence_hash()
                    );
                    assert_eq!(
                        meta.accepted_goal_polarity(),
                        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                    );
                    assert_eq!(
                        meta.accepted_goal_polarity(),
                        input.accepted_goal_polarity()
                    );
                    assert_eq!(
                        meta.trusted_used_axioms_hash(),
                        Some(reference.used_axioms_hash())
                    );
                    assert_eq!(meta.selected_proof_witness_hash(), None);
                    assert_eq!(meta.deterministic_discharge_hash(), None);
                    assert_eq!(meta.selected_candidate_provenance_hash(), None);
                    assert_eq!(meta.dependency_compatibility(), None);
                    assert_eq!(meta.explanation_ref(), None);
                    assert!(!meta.cache_reuse_predicate_complete());
                    assert_eq!(actual.1.status(), &VcStatus::Open);
                    assert!(!actual.1.entries().is_empty());
                    assert!(!actual.1.unknowns().is_empty());
                    assert_eq!(actual.1.completeness(), Completeness::IncompleteUncacheable);
                    assert_eq!(
                        actual.1.fingerprint().hash(),
                        meta.dependency_slice_fingerprint()
                    );
                    assert!(actual.1.requires_cache_miss());
                    assert_eq!(actual, project(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(original, identity(&source, check, env, snap, vc)?);
                    assert_eq!(checked, verify(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(chosen, select(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(old, legacy(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(policy, policy_before);
                    cases += 1;
                }
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&before.0, &before.1));
    }
    assert_eq!(cases, 24);
}

#[test]
fn source_theorem_reflexivity_used_axioms_status_preserves_identity_first_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            check_source_theorem_reflexivity as verify,
            project_source_theorem_reflexivity_status as legacy,
            project_source_theorem_reflexivity_used_axioms_status as project,
            select_source_theorem_reflexivity as select,
            source_theorem_obligation_identity as identity,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snap = snapshot_id(777);
    let absent = VcId::new(usize::MAX);
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (case, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            let vc = VcId::new(0);
            let policy = &policies[0];
            let prior = if case == 0 {
                Some((
                    identity(&source, check, env, snap, vc)?,
                    verify(&source, check, env, snap, vc, policy)?,
                    select(&source, check, env, snap, vc, policy)?,
                    legacy(&source, check, env, snap, vc, policy)?,
                    project(&source, check, env, snap, vc, policy)?,
                ))
            } else {
                None
            };
            for (index, policy) in policies.iter().enumerate() {
                for selected in [vc, absent] {
                    if case == 0 && index == 0 && selected == vc {
                        continue;
                    }
                    let expected = match identity(&source, check, env, snap, selected) {
                        Err(_) => {
                            let error = identity(&source, check, env, snap, selected).unwrap_err();
                            let check_error =
                                verify(&source, check, env, snap, selected, policy).unwrap_err();
                            if case == 3 {
                                assert_eq!(error, "unsupported source theorem formula payload");
                                assert_eq!(check_error, "theorem.handoff.invalid_projection");
                                assert_ne!(error, check_error);
                            }
                            error
                        }
                        Ok(original) => {
                            let check_error =
                                verify(&source, check, env, snap, selected, policy).unwrap_err();
                            assert_eq!((case, selected), (0, vc));
                            assert_eq!(check_error, "theorem.proof.kernel_policy_rejected");
                            assert_eq!(original, identity(&source, check, env, snap, selected)?);
                            check_error
                        }
                    };
                    assert_eq!(
                        project(&source, check, env, snap, selected, policy).unwrap_err(),
                        expected
                    );
                }
            }
            if let Some(prior) = prior {
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, env),
                    ] {
                        for policy in &policies {
                            for selected in [vc, absent] {
                                let expected =
                                    identity(current, check, symbols, snap, selected).unwrap_err();
                                verify(current, check, symbols, snap, selected, policy)
                                    .unwrap_err();
                                assert_eq!(
                                    project(current, check, symbols, snap, selected, policy)
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(
                    prior,
                    (
                        identity(&source, check, env, snap, vc)?,
                        verify(&source, check, env, snap, vc, policy)?,
                        select(&source, check, env, snap, vc, policy)?,
                        legacy(&source, check, env, snap, vc, policy)?,
                        project(&source, check, env, snap, vc, policy)?,
                    )
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(policies, policies_before);
}

#[test]
fn source_theorem_reflexivity_trusted_witness_metadata_matches_fresh_kernel() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_kernel::checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind};
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity as Polarity, CandidatePolicyClass as Class,
            KernelEvidenceFormat as Format, KernelEvidenceOrigin as Origin, KernelPolicyInput,
            VerifierPolicy as Policy,
        },
        selection::{ProofWinnerClass as Winner, TrustedKernelEvidence},
        status::{
            check_source_theorem_reflexivity_metadata as metadata,
            source_theorem_reflexivity_trusted_witness_metadata as trusted,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut original_source = None;
    for (index, (text, package, module, skip)) in [
        (original.to_owned(), "owner-test", "owners", 0),
        (
            original.replace(
                "for X being set holds X = X",
                "for Bound being set holds Bound = Bound",
            ),
            "owner-test",
            "owners",
            0,
        ),
        (
            original.replace(
                "let X be set; thus X = X",
                "let Local be set; thus Local = Local",
            ),
            "owner-test",
            "owners",
            0,
        ),
        (
            format!(":: 日本語 λ\n{original}\n"),
            "owner-test",
            "owners",
            0,
        ),
        (original.to_owned(), "owner-test", "owners", 7),
        (original.to_owned(), "other-package", "owners", 0),
        (original.to_owned(), "owner-test", "nested.owners", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        if skip == 7 {
            assert_ne!(Some(source.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(source.source_id());
        }
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for policy in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("theorem-trusted-metadata"),
                Policy::default().with_schema_version(2),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 6 } else { 1 })
            {
                let policy_before = policy.clone();
                for ordinal in [777, 778] {
                    let snap = snapshot_id(ordinal);
                    let vc = VcId::new(0);
                    let origin = Origin::BuiltinDischarge;
                    let (result, decision, acceptance, _wire) =
                        metadata(&source, check, env, snap, vc, &policy)?;
                    assert_eq!(result.status(), Status::Accepted);
                    assert_eq!(result.evidence_check_kind(), Some(Kind::ProofObligation));
                    assert!(!result.policy_taint());
                    assert_eq!(decision.class, Class::DischargedBuiltin);
                    let acceptance = acceptance.unwrap();
                    let input = KernelPolicyInput::from_kernel_result(&result, origin);
                    let evidence = TrustedKernelEvidence::from_policy_input(&input).unwrap();
                    let actual = trusted(&source, check, env, snap, vc, &policy)?;
                    assert_eq!(actual.kernel_acceptance(), &acceptance);
                    assert_eq!(actual.trusted_kernel_evidence(), &evidence);
                    assert_eq!(actual.kernel_evidence_origin(), origin);
                    assert_eq!(actual.selected_class(), Winner::DischargedBuiltin);
                    assert_eq!(
                        Some(actual.accepted_evidence_hash()),
                        input.accepted_evidence_hash()
                    );
                    assert_eq!(
                        actual.accepted_evidence_hash(),
                        acceptance.accepted_result_hash.digest
                    );
                    assert_eq!(
                        actual.trusted_kernel_evidence().accepted_goal_polarity(),
                        Polarity::AssertFalseForRefutation
                    );
                    assert_eq!(
                        Some(actual.trusted_kernel_evidence().accepted_goal_polarity()),
                        input.accepted_goal_polarity()
                    );
                    assert_eq!(actual, trusted(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(policy, policy_before);
                }
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&before.0, &before.1));
    }
}

#[test]
fn source_theorem_reflexivity_trusted_witness_metadata_preserves_upstream_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            check_source_theorem_reflexivity_metadata as metadata,
            source_theorem_reflexivity_trusted_witness_metadata as trusted,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snap = snapshot_id(777);
    let vc = VcId::new(0);
    let absent = VcId::new(usize::MAX);
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (case, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            let prior = if case == 0 {
                Some(trusted(&source, check, env, snap, vc, &policies[0])?)
            } else {
                None
            };
            for (index, policy) in policies
                .iter()
                .take(if case == 0 { 3 } else { 1 })
                .enumerate()
            {
                for selected in [vc, absent].into_iter().take(if case == 0 { 2 } else { 1 }) {
                    if case == 0 && index == 0 && selected == vc {
                        continue;
                    }
                    let expected =
                        metadata(&source, check, env, snap, selected, policy).unwrap_err();
                    assert_eq!(
                        trusted(&source, check, env, snap, selected, policy).unwrap_err(),
                        expected
                    );
                }
            }
            if let Some(prior) = prior {
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, env),
                    ] {
                        let policy = &policies[0];
                        let expected =
                            metadata(current, check, symbols, snap, vc, policy).unwrap_err();
                        assert_eq!(
                            trusted(current, check, symbols, snap, vc, policy).unwrap_err(),
                            expected
                        );
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(prior, trusted(&source, check, env, snap, vc, &policies[0])?);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(policies, policies_before);
}

#[test]
fn source_theorem_reflexivity_trusted_witness_metadata_with_wire_matches_same_check() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::{
        policy::{
            CandidatePolicyClass as Class, KernelEvidenceFormat as Format,
            KernelEvidenceOrigin as Origin, KernelPolicyInput, VerifierPolicy as Policy,
        },
        selection::TrustedKernelEvidence,
        status::{
            check_source_theorem_reflexivity_metadata as metadata,
            source_theorem_reflexivity_trusted_witness_metadata as trusted,
            source_theorem_reflexivity_trusted_witness_metadata_with_wire as trusted_wire,
        },
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_theorem_kernel_handoff as project,
        },
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut prior = Vec::new();
    for (index, (text, package, module, skip)) in [
        (original.to_owned(), "owner-test", "owners", 0),
        (
            original.replace(
                "for X being set holds X = X",
                "for Bound being set holds Bound = Bound",
            ),
            "owner-test",
            "owners",
            0,
        ),
        (
            original.replace(
                "let X be set; thus X = X",
                "let Local be set; thus Local = Local",
            ),
            "owner-test",
            "owners",
            0,
        ),
        (
            format!(":: 日本語 λ\n{original}\n"),
            "owner-test",
            "owners",
            0,
        ),
        (original.to_owned(), "owner-test", "owners", 7),
        (original.to_owned(), "other-package", "owners", 0),
        (original.to_owned(), "owner-test", "nested.owners", 0),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone(), prior.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for policy in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("theorem-trusted-metadata"),
                Policy::default().with_schema_version(2),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 6 } else { 1 })
            {
                let policy_before = policy.clone();
                for ordinal in [777, 778] {
                    let snap = snapshot_id(ordinal);
                    let vc = VcId::new(0);
                    let origin = Origin::BuiltinDischarge;
                    let expected = metadata(&source, check, env, snap, vc, &policy);
                    let expected_before = expected.clone();
                    let (result, decision, acceptance, wire) =
                        expected.as_ref().map_err(Clone::clone)?;
                    assert_eq!(result.status(), Status::Accepted);
                    assert_eq!(result.evidence_check_kind(), Some(Kind::ProofObligation));
                    assert!(!result.policy_taint());
                    assert_eq!(decision.class, Class::DischargedBuiltin);
                    let acceptance = acceptance.as_ref().unwrap();
                    let input = KernelPolicyInput::from_kernel_result(result, origin);
                    let evidence = TrustedKernelEvidence::from_policy_input(&input).unwrap();
                    let actual = trusted_wire(&source, check, env, snap, vc, &policy)?;
                    assert_eq!(&actual.1, wire);
                    let core = lower(check, None)?;
                    let graph = generate(
                        &core,
                        snap,
                        &GenerationSchemaVersion::new("source-theorem-local-v1"),
                        &VcSchemaVersion::new("vc-v1"),
                    )?;
                    let handoff = project(&core, &graph, vc)?;
                    let envelope = handoff.canonical_evidence();
                    let target = envelope.target_vc();
                    let profile = envelope.kernel_profile();
                    let tautology = match profile.clause_tautology_policy {
                        KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                        KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                        _ => panic!("unsupported actual theorem profile"),
                    };
                    let mut context = FormulaEvidenceParseContext::v1(
                        Fingerprint::new(target.algorithm_id, target.digest.clone()),
                        KernelProfileRecord::v1(profile.profile_id, tautology),
                    );
                    context.accepted_schema_versions.insert(2);
                    let parsed = parse_formula_evidence(&actual.1, &context).unwrap();
                    assert_eq!(parsed.schema_version(), 2);
                    assert_eq!(
                        acceptance.evidence_schema_version,
                        mizar_artifact::store::SchemaVersion::new(parsed.schema_version(), 0)
                    );
                    assert_eq!(actual.0.kernel_acceptance(), acceptance);
                    assert_eq!(actual.0.trusted_kernel_evidence(), &evidence);
                    assert_eq!(actual.0.kernel_evidence_origin(), origin);
                    assert_eq!(
                        trusted(&source, check, env, snap, vc, &policy),
                        Ok(actual.0.clone())
                    );
                    assert_eq!(
                        actual,
                        trusted_wire(&source, check, env, snap, vc, &policy)?
                    );
                    assert_eq!(expected, expected_before);
                    assert_eq!(policy, policy_before);
                    prior.push(actual);
                }
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
    }
}

#[test]
fn source_theorem_reflexivity_trusted_witness_metadata_with_wire_preserves_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            check_source_theorem_reflexivity_metadata as metadata,
            source_theorem_reflexivity_trusted_witness_metadata as trusted,
            source_theorem_reflexivity_trusted_witness_metadata_with_wire as trusted_wire,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snap = snapshot_id(777);
    let vc = VcId::new(0);
    let absent = VcId::new(usize::MAX);
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (case, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            let prior = if case == 0 {
                let raw = metadata(&source, check, env, snap, vc, &policies[0]);
                raw.as_ref().map_err(Clone::clone)?;
                Some((
                    raw,
                    trusted_wire(&source, check, env, snap, vc, &policies[0])?,
                    trusted(&source, check, env, snap, vc, &policies[0]),
                ))
            } else {
                None
            };
            for (index, policy) in policies
                .iter()
                .take(if case == 0 { 3 } else { 1 })
                .enumerate()
            {
                for selected in [vc, absent].into_iter().take(if case == 0 { 2 } else { 1 }) {
                    if case == 0 && index == 0 && selected == vc {
                        continue;
                    }
                    let expected =
                        metadata(&source, check, env, snap, selected, policy).unwrap_err();
                    assert_eq!(
                        trusted_wire(&source, check, env, snap, selected, policy).unwrap_err(),
                        expected
                    );
                    assert_eq!(
                        trusted(&source, check, env, snap, selected, policy).unwrap_err(),
                        expected
                    );
                }
            }
            if let Some(prior) = prior {
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, env),
                    ] {
                        for policy in &policies {
                            for selected in [vc, absent] {
                                let expected =
                                    metadata(current, check, symbols, snap, selected, policy)
                                        .unwrap_err();
                                assert_eq!(
                                    trusted_wire(current, check, symbols, snap, selected, policy)
                                        .unwrap_err(),
                                    expected
                                );
                                assert_eq!(
                                    trusted(current, check, symbols, snap, selected, policy)
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(
                    prior,
                    (
                        metadata(&source, check, env, snap, vc, &policies[0]),
                        trusted_wire(&source, check, env, snap, vc, &policies[0])?,
                        trusted(&source, check, env, snap, vc, &policies[0]),
                    )
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!(policies, policies_before);
}

#[test]
fn source_theorem_reflexivity_witness_draft_matches_captured_inputs() {
    use mizar_artifact::store::{
        CanonicalJson, HashClass, PublishedArtifactPath, PublishedArtifactReadOptions,
        artifact_hash_domain, canonical_json_bytes, read_published_artifact,
        write_published_artifact,
    };
    use mizar_artifact::{
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::SchemaVersion,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::witness_store::stage;
    use mizar_proof::{
        policy::{
            KernelEvidenceFormat as Format, KernelEvidenceOrigin as Origin,
            VerifierPolicy as Policy,
        },
        status::{
            ArtifactPublicationGap, ProjectedProofStatus,
            project_source_theorem_reflexivity_status as status,
            source_theorem_obligation_identity as identity,
            source_theorem_reflexivity_trusted_witness_metadata_with_wire as trusted_wire,
            source_theorem_reflexivity_witness_draft as draft,
        },
        witness_store::{ProofWitnessDraft, ProofWitnessPayloadSchema, ProofWitnessProvenance},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DependencyArtifactRef, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOrigin,
        SourceOriginInput, ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness,
        vc_ir::{VcId, VcStatus},
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
    // Raw file content and supplied toolchain/config metadata do not attest Build profiles or runtime.
    let lockfile_hash = hash_text(
        &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"))
            .unwrap(),
    );
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut captures = Vec::new();
    for (index, text) in [
        original.to_owned(),
        original.to_owned(),
        original.replace(
            "for X being set holds X = X",
            "for Bound being set holds Bound = Bound",
        ),
        original.replace(
            "let X be set; thus X = X",
            "let Local be set; thus Local = Local",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(&path, &text).unwrap();
        let input = SourceInput {
            package_id: PackageId::new("draft-test"),
            module_path: ModulePath::new("owners"),
            normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
            edition: Edition::new("2026"),
            origin: if index % 2 == 0 {
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
        // A supplied nonempty reference is metadata only; the zero-import source remains incomplete.
        let dependencies = if index == 3 {
            vec![DependencyArtifactRef::new(
                "caller-recorded/source",
                hash_text(&text),
            )]
        } else {
            Vec::new()
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    dependencies,
                    lockfile_hash,
                    ToolchainInfo::new("caller-supplied-draft-test-toolchain"),
                    Policy::default().policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        assert_eq!(lease.snapshot, snapshot.id);
        let (loaded, captured_input) = loaded.pop().unwrap();
        assert_eq!(captured_input, input);
        let version = &snapshot.source_versions[0];
        assert_eq!(version.source_id, loaded.source_id);
        assert_eq!(version.source_hash, loaded.source_hash);
        assert_eq!(version.origin, loaded.origin);
        assert_eq!(matches!(version.origin, SourceOrigin::Disk), index % 2 == 0);
        let unit = source_unit_from_loaded(loaded, path.clone());
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(unit.clone())
        .unwrap();
        assert_eq!(frontend.source, unit);
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        assert_eq!(ast.source_id, version.source_id);
        let module = ModuleId::new(input.package_id.clone(), input.module_path.clone());
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        assert_eq!(source.source_id(), version.source_id);
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone(), snapshot.clone());
        with_source_theorem_check(ast, &source, env, |check| {
            let check_before = format!("{check:?}");
            for policy in [
                Policy::default(),
                Policy::release().with_require_kernel_certificates(true),
                Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                Policy::default().with_profile_id("draft-policy-variation"),
                Policy::default().with_checker_schema_version(2),
            ]
            .into_iter()
            .take(if index == 0 { 5 } else { 1 })
            {
                let witness_path = format!("proof-witnesses/caller-{index}/checked.bin");
                let inputs_before = (policy.clone(), witness_path.clone());
                let vc = VcId::new(0);
                let (original_id, original_slice) = identity(&source, check, env, snapshot.id, vc)?;
                let (projection, _) = status(&source, check, env, snapshot.id, vc, &policy)?;
                let (trusted, wire) = trusted_wire(&source, check, env, snapshot.id, vc, &policy)?;
                let acceptance = trusted.kernel_acceptance();
                let reuse = projection.reuse_metadata();
                let provenance = ProofWitnessProvenance::new(
                    snapshot.id.fingerprint(),
                    "mizar-proof/source-theorem-reflexivity-witness-draft/v1",
                    reuse.selected_candidate_id().unwrap().clone(),
                    Origin::BuiltinDischarge,
                    original_id.vc_fingerprint(),
                    original_slice.fingerprint().hash(),
                    snapshot.dependency_artifact_fingerprint(),
                    policy.policy_fingerprint().hash(),
                    acceptance.checker_schema_version,
                    acceptance.evidence_schema_version,
                )
                .unwrap()
                .with_accepted_result_hash(acceptance.accepted_result_hash.digest);
                let payload_schema = ProofWitnessPayloadSchema::with_canonical_bytes_required(
                    "mizar-proof/source-theorem-reflexivity-checked-wire",
                    acceptance.evidence_schema_version,
                    true,
                )
                .unwrap();
                let expected_draft = ProofWitnessDraft::new(
                    original_id.obligation_id(),
                    original_id.obligation_anchor().clone(),
                    ArtifactHashRef::new(
                        ArtifactHashClass::Interface,
                        "mizar-proof/source-theorem-original-obligation",
                        SchemaVersion::new(1, 0),
                        original_id.obligation_fingerprint(),
                    ),
                    &projection,
                    trusted,
                    payload_schema,
                    wire.clone(),
                    witness_path.clone(),
                    provenance,
                )
                .unwrap();
                let expected = (projection, original_slice, expected_draft);
                let actual = draft(&source, check, env, &snapshot, vc, &policy, &witness_path)?;
                assert_eq!(actual, expected);
                assert_eq!(
                    actual,
                    draft(&source, check, env, &snapshot, vc, &policy, &witness_path)?
                );
                let reuse = actual.0.reuse_metadata();
                assert_eq!(actual.0.obligation_id(), original_id.obligation_id());
                assert_eq!(
                    original_id.obligation_id(),
                    original_id.obligation_anchor().as_str()
                );
                assert!(
                    original_id
                        .obligation_id()
                        .starts_with("source-theorem-candidate-anchor-v1:")
                );
                assert_eq!(reuse.obligation_anchor(), original_id.obligation_anchor());
                assert_eq!(
                    reuse.obligation_fingerprint(),
                    original_id.obligation_fingerprint()
                );
                assert_eq!(reuse.vc_fingerprint(), original_id.vc_fingerprint());
                assert_eq!(
                    reuse.local_context_fingerprint(),
                    original_id.local_context_fingerprint()
                );
                assert_eq!(
                    reuse.dependency_slice_fingerprint(),
                    original_id.dependency_slice_fingerprint()
                );
                assert_eq!(actual.1.status(), &VcStatus::Open);
                assert_eq!(
                    actual.1.completeness(),
                    DependencySliceCompleteness::IncompleteUncacheable
                );
                assert!(actual.1.requires_cache_miss());
                assert_eq!(actual.0.projected_status(), ProjectedProofStatus::Accepted);
                assert_eq!(
                    actual.0.artifact_publication().gap(),
                    Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
                );
                assert!(reuse.selected_proof_witness_hash().is_none());
                assert!(reuse.deterministic_discharge_hash().is_none());
                assert!(!reuse.cache_reuse_predicate_complete());
                assert!(actual.0.trusted_used_axioms().is_none());
                assert!(actual.2.provenance().trusted_used_axioms_ref().is_none());
                assert!(actual.2.provenance().advisory_backend_ref().is_none());
                assert_eq!(actual.2.payload_bytes(), wire);
                assert_eq!(actual.2.witness_path(), witness_path);
                assert!(!root.join(&witness_path).exists());
                assert_eq!((policy, witness_path), inputs_before);
                let draft_before = actual.2.clone();
                let staged = stage(actual.2.clone()).unwrap();
                let mut payload_hex = String::new();
                for byte in &wire {
                    payload_hex.push_str(&format!("{byte:02x}"));
                }
                let mut evidence_hex = String::new();
                for byte in actual.2.selected_evidence_hash().as_bytes() {
                    evidence_hex.push_str(&format!("{byte:02x}"));
                }
                let mut policy_hex = String::new();
                let policy_fingerprint = actual.2.provenance().verifier_policy_fingerprint();
                for byte in policy_fingerprint.as_bytes() {
                    policy_hex.push_str(&format!("{byte:02x}"));
                }
                let expected_value = CanonicalJson::Object(std::collections::BTreeMap::from([
                    (
                        "hash_domain".to_owned(),
                        CanonicalJson::String("mizar-proof/witness-payload/v1".to_owned()),
                    ),
                    (
                        "obligation_fingerprint".to_owned(),
                        CanonicalJson::String(
                            actual.2.obligation_fingerprint().to_artifact_hash_string(),
                        ),
                    ),
                    (
                        "payload_bytes_hex".to_owned(),
                        CanonicalJson::String(payload_hex),
                    ),
                    (
                        "payload_schema_family".to_owned(),
                        CanonicalJson::String(actual.2.payload_schema().family().to_owned()),
                    ),
                    (
                        "payload_schema_version".to_owned(),
                        CanonicalJson::String(actual.2.payload_schema().version().to_string()),
                    ),
                    (
                        "selected_evidence_hash".to_owned(),
                        CanonicalJson::String(evidence_hex),
                    ),
                    (
                        "verifier_policy_fingerprint".to_owned(),
                        CanonicalJson::String(policy_hex),
                    ),
                ]));
                let mut value = actual.2.payload_hash_value();
                assert_eq!(value, expected_value);
                let hash = staged.staged_payload_hash();
                let schema = actual.2.payload_schema();
                let domain = artifact_hash_domain(schema.family(), schema.version());
                assert_eq!(hash.class, ArtifactHashClass::Artifact);
                assert_eq!(hash.schema_family, schema.family());
                assert_eq!(hash.schema_version, schema.version());
                assert_eq!(domain.class(), HashClass::Artifact);
                assert_eq!(
                    domain,
                    artifact_hash_domain(&hash.schema_family, hash.schema_version)
                );
                assert_eq!(domain.hash(&expected_value, &[]), hash.digest);
                assert!(staged.artifact_reference_candidate().is_none());
                let transport_root = root.join("unpublished-witness-hash-values");
                let transport_path = PublishedArtifactPath::new(actual.2.witness_path()).unwrap();
                let expected_bytes = canonical_json_bytes(&expected_value);
                for _ in 0..2 {
                    assert_eq!(actual.2.payload_hash_value(), expected_value);
                    let written = write_published_artifact(
                        &transport_root,
                        &transport_path,
                        &value,
                        &domain,
                        &[],
                    )
                    .unwrap();
                    assert_eq!(written.path, transport_path);
                    assert_eq!(written.artifact_hash, hash.digest);
                    let read = read_published_artifact(
                        &transport_root,
                        &transport_path,
                        PublishedArtifactReadOptions {
                            artifact_hash_domain: Some(&domain),
                            expected_artifact_hash: Some(hash.digest),
                            ..PublishedArtifactReadOptions::default()
                        },
                    )
                    .unwrap();
                    assert_eq!(read.path, transport_path);
                    assert_eq!(read.value, expected_value);
                    assert_eq!(read.artifact_hash, Some(hash.digest));
                    assert_eq!(
                        fs::read(transport_root.join(actual.2.witness_path())).unwrap(),
                        expected_bytes
                    );
                }
                let CanonicalJson::Object(object) = &mut value else {
                    unreachable!("the whole expected value is an object")
                };
                object.clear();
                assert_eq!(actual.2, draft_before);
                assert_eq!(actual.2.payload_hash_value(), expected_value);
                assert!(!root.join(actual.2.witness_path()).exists());
                captures.push(snapshot.id);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone(), snapshot.clone()), before);
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    // Native origin alone need not change canonical identity. Each oracle used the retained record.
    assert_eq!(captures[0], captures[5]);
    assert_ne!(captures[0], *captures.last().unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_theorem_reflexivity_witness_draft_preserves_errors() {
    use mizar_artifact::{
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::SchemaVersion,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::{
            KernelEvidenceFormat as Format, KernelEvidenceOrigin as Origin,
            VerifierPolicy as Policy,
        },
        status::{
            check_source_theorem_reflexivity_metadata as metadata,
            project_source_theorem_reflexivity_status as status,
            source_theorem_obligation_identity as identity,
            source_theorem_reflexivity_trusted_witness_metadata_with_wire as trusted_wire,
            source_theorem_reflexivity_witness_draft as draft,
        },
        witness_store::{ProofWitnessDraft, ProofWitnessPayloadSchema, ProofWitnessProvenance},
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
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
        Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
    ];
    let policies_before = policies.clone();
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let lockfile_hash = hash_text(
        &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"))
            .unwrap(),
    );
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let mut cases = Vec::new();
    for (index, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
        cited.to_owned(),
        original.replace("theorem T:", "theorem Foreign:"),
        "theorem T: for X being set holds X = ;".to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(&path, &text).unwrap();
        let input = SourceInput {
            package_id: PackageId::new("draft-errors"),
            module_path: ModulePath::new("owners"),
            edition: Edition::new("2026"),
            normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
            origin: SourceOriginInput::Disk { path: path.clone() },
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    Vec::new(),
                    lockfile_hash,
                    ToolchainInfo::new("caller-supplied-draft-test-toolchain"),
                    policies[0].policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        let (loaded, captured_input) = loaded.pop().unwrap();
        assert_eq!(captured_input, input);
        assert_eq!(loaded.source_id, snapshot.source_versions[0].source_id);
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(source_unit_from_loaded(loaded, path.clone()))
        .unwrap();
        assert_eq!(!frontend.diagnostics.is_empty(), index == 6);
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(input.package_id, input.module_path);
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        cases.push((snapshot, lease, frontend, source, collected));
    }
    for (index, (snapshot, _, frontend, source, collected)) in cases.iter().take(5).enumerate() {
        let env = collected.env();
        let before = (source.clone(), env.clone(), snapshot.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), source, env, |check| {
            let check_before = format!("{check:?}");
            let vc = VcId::new(0);
            let absent = VcId::new(usize::MAX);
            for (policy_index, policy) in policies.iter().enumerate() {
                for selected in [vc, absent] {
                    if index == 0 && policy_index == 0 && selected == vc {
                        continue;
                    }
                    // Identity wins even over disabled formats, absent VC and an invalid path.
                    let expected = match identity(source, check, env, snapshot.id, selected) {
                        Err(error) => error,
                        Ok(_) => {
                            metadata(source, check, env, snapshot.id, selected, policy).unwrap_err()
                        }
                    };
                    assert_eq!(
                        draft(source, check, env, snapshot, selected, policy, "").unwrap_err(),
                        expected
                    );
                }
            }
            if index == 0 {
                let policy = &policies[0];
                let (original_id, original_slice) = identity(source, check, env, snapshot.id, vc)?;
                let (projection, _) = status(source, check, env, snapshot.id, vc, policy)?;
                let (trusted, wire) = trusted_wire(source, check, env, snapshot.id, vc, policy)?;
                let acceptance = trusted.kernel_acceptance();
                let provenance = ProofWitnessProvenance::new(
                    snapshot.id.fingerprint(),
                    "mizar-proof/source-theorem-reflexivity-witness-draft/v1",
                    projection
                        .reuse_metadata()
                        .selected_candidate_id()
                        .unwrap()
                        .clone(),
                    Origin::BuiltinDischarge,
                    original_id.vc_fingerprint(),
                    original_slice.fingerprint().hash(),
                    snapshot.dependency_artifact_fingerprint(),
                    policy.policy_fingerprint().hash(),
                    acceptance.checker_schema_version,
                    acceptance.evidence_schema_version,
                )
                .unwrap()
                .with_accepted_result_hash(acceptance.accepted_result_hash.digest);
                let schema = ProofWitnessPayloadSchema::with_canonical_bytes_required(
                    "mizar-proof/source-theorem-reflexivity-checked-wire",
                    acceptance.evidence_schema_version,
                    true,
                )
                .unwrap();
                let obligation = ArtifactHashRef::new(
                    ArtifactHashClass::Interface,
                    "mizar-proof/source-theorem-original-obligation",
                    SchemaVersion::new(1, 0),
                    original_id.obligation_fingerprint(),
                );
                let witness_path = "";
                let expected = ProofWitnessDraft::new(
                    original_id.obligation_id(),
                    original_id.obligation_anchor().clone(),
                    obligation.clone(),
                    &projection,
                    trusted.clone(),
                    schema.clone(),
                    wire.clone(),
                    witness_path,
                    provenance.clone(),
                )
                .unwrap_err()
                .to_string();
                assert_eq!(
                    draft(source, check, env, snapshot, vc, policy, witness_path).unwrap_err(),
                    expected
                );
                let path = "proof-witnesses/errors/prior.bin";
                let expected = ProofWitnessDraft::new(
                    original_id.obligation_id(),
                    original_id.obligation_anchor().clone(),
                    obligation,
                    &projection,
                    trusted,
                    schema,
                    wire,
                    path,
                    provenance,
                )
                .unwrap();
                let prior = draft(source, check, env, snapshot, vc, policy, path)?;
                assert_eq!(
                    (&prior.0, &prior.1, &prior.2),
                    (&projection, &original_slice, &expected)
                );
                for (_, _, _, foreign, symbols) in cases.iter().skip(3) {
                    let foreign_before = (foreign.clone(), symbols.env().clone());
                    for (current, symbols) in [
                        (foreign, symbols.env()),
                        (source, symbols.env()),
                        (foreign, env),
                    ] {
                        for policy in &policies {
                            for selected in [vc, absent] {
                                let expected =
                                    identity(current, check, symbols, snapshot.id, selected)
                                        .unwrap_err();
                                assert_eq!(
                                    draft(current, check, symbols, snapshot, selected, policy, "")
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                    }
                    assert_eq!((foreign.clone(), symbols.env().clone()), foreign_before);
                }
                assert_eq!(
                    prior,
                    draft(source, check, env, snapshot, vc, policy, path)?
                );
                assert!(!root.join(path).exists());
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source.clone(), env.clone(), snapshot.clone()), before);
    }
    assert_eq!(policies, policies_before);
    for (snapshot, lease, _, _, _) in cases {
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_theorem_reflexivity_original_graph_matches_genuine_check() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_core::{
        core_ir::{CoreProofStatus, ObligationSeedStatus},
        elaborator::lower_source_theorem_skeletons as lower,
    };
    use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
    use mizar_proof::{
        policy::{CandidatePolicyClass, VerifierPolicy},
        status::{
            check_source_theorem_reflexivity as compatibility,
            check_source_theorem_reflexivity_with_original_graph as verify,
        },
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, SeedOriginRef, VcId, VcSchemaVersion, VcStatus},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut original_source = None;
    for (text, module, skip) in [
        (original.to_owned(), "owners", 0),
        (original.replace("X", "Bound"), "owners", 0),
        (format!(":: 日本語 λ\n{original}\n"), "owners", 0),
        (original.to_owned(), "nested.owners", 7),
    ] {
        let (frontend, source, collected) = inputs(&text, "owner-test", module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        if skip == 7 {
            assert_ne!(Some(source.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(source.source_id());
        }
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for (ordinal, policy) in [
                (777, VerifierPolicy::default()),
                (
                    778,
                    VerifierPolicy::release().with_require_kernel_certificates(true),
                ),
            ] {
                let snapshot = snapshot_id(ordinal);
                let selected = VcId::new(0);
                let policy_before = policy.clone();
                let core = lower(check, None)?;
                let vcs = generate(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("source-theorem-local-v1"),
                    &VcSchemaVersion::new("vc-v1"),
                )?;
                let (result, decision) =
                    compatibility(&source, check, env, snapshot, selected, &policy)?;
                assert_eq!(core.source_id(), source.source_id());
                assert_eq!(core.module_id(), source.module());
                assert_eq!(core.module_id().package().as_str(), "owner-test");
                assert_eq!(core.module_id().path().as_str(), module);
                assert_eq!(vcs.source(), source.source_id());
                assert_eq!(vcs.snapshot(), snapshot_id(ordinal));
                assert_eq!(
                    vcs.module().as_str(),
                    format!("package=10:owner-test;module={}:{}", module.len(), module)
                );
                assert_eq!(vcs.schema_version().as_str(), "vc-v1");
                assert!(!core.proofs().is_empty() && !vcs.vcs().is_empty());
                assert!(
                    core.proofs().iter().all(|(_, proof)| {
                        proof.status == CoreProofStatus::PendingAutomaticProof
                    })
                );
                assert!(!vcs.seed_accounting().is_empty());
                for accounting in vcs.seed_accounting() {
                    assert_eq!(accounting.seed_status, ObligationSeedStatus::Active);
                    let SeedOriginRef::ExistingCore { seed } = accounting.origin else {
                        panic!("genuine theorem seed comes from original Core")
                    };
                    assert_eq!(
                        core.obligation_seeds().get(seed).unwrap().status,
                        ObligationSeedStatus::Active
                    );
                }
                for vc in vcs.vcs() {
                    assert_eq!(vc.status, VcStatus::Open);
                    assert_eq!(
                        vc.anchor.generation_schema_version.as_str(),
                        "source-theorem-local-v1"
                    );
                    assert!(vcs.seed_accounting_for(vc.seed.handoff).is_some());
                }
                assert_eq!(vcs.vc(selected).unwrap().id, VcId::new(0));
                assert_eq!(result.status(), KernelCheckStatus::Accepted);
                assert_eq!(
                    result.evidence_check_kind(),
                    Some(KernelEvidenceCheckKind::ProofObligation)
                );
                assert!(!result.policy_taint());
                assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                let expected = (core, vcs, result, decision);
                let actual = verify(&source, check, env, snapshot, selected, &policy)?;
                assert_eq!(actual, expected);
                assert_eq!(
                    verify(&source, check, env, snapshot, selected, &policy)?,
                    expected
                );
                assert_eq!((snapshot, selected), (snapshot_id(ordinal), VcId::new(0)));
                assert_eq!(policy, policy_before);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
}

#[test]
fn source_theorem_reflexivity_original_graph_preserves_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            check_source_theorem_reflexivity as compatibility,
            check_source_theorem_reflexivity_with_original_graph as verify,
        },
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snapshot = snapshot_id(777);
    let selected = VcId::new(0);
    let absent = VcId::new(usize::MAX);
    let normal = VerifierPolicy::default();
    let disabled = normal.clone().with_kernel_evidence_formats([]);
    let policies_before = (normal.clone(), disabled.clone());
    for (case, text) in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            for (id, policy) in [
                (selected, &normal),
                (absent, &normal),
                (selected, &disabled),
                (absent, &disabled),
            ] {
                if case == 0 && id == selected && policy == &normal {
                    continue;
                }
                let expected =
                    compatibility(&source, check, env, snapshot, id, policy).unwrap_err();
                assert_eq!(
                    verify(&source, check, env, snapshot, id, policy).unwrap_err(),
                    expected
                );
            }
            if case == 0 {
                let prior = compatibility(&source, check, env, snapshot, selected, &normal)?;
                let (other_frontend, other_source, other) = inputs(
                    &original.replace("T:", "Foreign:"),
                    "owner-test",
                    "owners",
                    7,
                );
                assert!(other_frontend.diagnostics.is_empty());
                assert!(other.diagnostics().is_empty());
                assert_ne!(other_source.source_id(), source.source_id());
                let other_before = (other_source.clone(), other.env().clone());
                for (current, symbols) in [
                    (&source, other.env()),
                    (&other_source, env),
                    (&other_source, other.env()),
                ] {
                    let expected =
                        compatibility(current, check, symbols, snapshot, selected, &normal)
                            .unwrap_err();
                    assert_eq!(
                        verify(current, check, symbols, snapshot, selected, &normal).unwrap_err(),
                        expected
                    );
                    assert_eq!(
                        compatibility(current, check, symbols, snapshot, absent, &disabled)
                            .unwrap_err(),
                        expected
                    );
                    assert_eq!(
                        verify(current, check, symbols, snapshot, absent, &disabled).unwrap_err(),
                        expected
                    );
                }
                assert_eq!((other_source, other.env().clone()), other_before);
                assert_eq!(
                    compatibility(&source, check, env, snapshot, selected, &normal)?,
                    prior
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!((normal, disabled), policies_before);
    assert_eq!((snapshot, selected), (snapshot_id(777), VcId::new(0)));
}
