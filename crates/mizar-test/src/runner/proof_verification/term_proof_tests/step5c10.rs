use super::*;

#[test]
fn step5c10_mapped_sources_reach_frozen_phases() {
    use mizar_proof::policy::CandidatePolicyClass;
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    for (ordinal, (id, _, key)) in super::super::super::formula_statement::STEP5C10_CASES
        .into_iter()
        .enumerate()
    {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let output = run_frontend(&config.workspace_root, case, ordinal).unwrap();
        assert!(
            output.diagnostics.is_empty(),
            "{id}: {:?}",
            output.diagnostics
        );
        let ast = output.ast.unwrap();
        let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
        assert!(
            resolver.detail_keys.is_empty(),
            "{id}: {:?}",
            resolver.detail_keys
        );
        let phase = case.expectation.expected_phase.unwrap();
        let result = theorem_ast_output(
            &ast,
            &resolver.module,
            &resolver.env,
            phase,
            snapshot_id(ordinal),
            None,
        );
        if let Some(key) = key {
            assert_eq!(result.unwrap_err(), key, "{id}");
            continue;
        }
        let (classes, vcs) = result.unwrap_or_else(|error| panic!("{id}: {error}"));
        match phase {
            PipelinePhase::StatementCheck => {
                assert_eq!(
                    classes,
                    [
                        CandidatePolicyClass::OpenAllowed,
                        CandidatePolicyClass::AssumedByPolicy
                    ]
                );
                assert!(vcs.is_none());
            }
            PipelinePhase::VcGeneration => {
                assert!(classes.is_empty());
                let vcs = vcs.unwrap();
                assert_eq!(vcs.source(), ast.source_id);
                assert_eq!(vcs.snapshot(), snapshot_id(ordinal));
                assert_eq!(vcs.vcs().len(), 2);
                let mut context_sizes = Vec::new();
                for vc in vcs.vcs() {
                    assert_eq!(vc.status, mizar_vc::vc_ir::VcStatus::Open);
                    assert!(
                        vc.local_context
                            .entries()
                            .iter()
                            .all(|entry| entry.formula.is_some() && !entry.provenance.is_empty())
                    );
                    assert!(!vc.provenance.is_empty());
                    context_sizes.push(vc.local_context.entries().len());
                }
                context_sizes.sort();
                assert_eq!(context_sizes, [1, 2]);
                assert_eq!(
                    theorem_ast_output(
                        &ast,
                        &resolver.module,
                        &resolver.env,
                        phase,
                        snapshot_id(ordinal),
                        None,
                    )
                    .unwrap()
                    .1,
                    Some(vcs)
                );
            }
            _ => panic!("unexpected success phase"),
        }
        let run = run_proof_verification_case(
            &config.workspace_root,
            &config.workspace_root.join("tests"),
            case,
            ordinal,
        );
        assert_eq!(
            run.status,
            super::super::super::ProofVerificationCaseStatus::Passed,
            "{:?}",
            run.failure
        );
    }
}

#[test]
fn step5c10_skeleton_errors_require_the_actual_thesis_and_repair() {
    let valid = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    assert!(check_theorem_source(valid, PipelinePhase::VcGeneration).is_ok());
    assert!(
        check_theorem_source(
            &valid.replace(
                "proof let X be set; thus X = X;",
                "proof let Y be set; thus Y = Y;"
            ),
            PipelinePhase::VcGeneration
        )
        .is_ok()
    );
    for (source, key) in [
        (
            valid.replace("thus X = X;", "assume A: X = X; thus X = X;"),
            "theorems.skeleton.assumption_without_antecedent",
        ),
        (
            valid.replace("thus X = X;", "thus not X = X;"),
            "theorems.skeleton.conclusion_mismatch",
        ),
        (
            valid.replace("thus X = X;", ""),
            "theorems.skeleton.incomplete_proof",
        ),
    ] {
        assert_eq!(
            check_theorem_source(&source, PipelinePhase::StatementCheck).unwrap_err(),
            key
        );
    }
    for source in [
        valid.replace("let X be set", "let X be object"),
        valid.replace("thus", "hence"),
        valid.replace("thus X = X;", "thus X = X; thus X = X;"),
        valid.replace("for X being set holds", "for X being set st X = X holds"),
        valid.replace("thus X = X", "thus Y = Y"),
    ] {
        assert!(
            check_theorem_source(&source, PipelinePhase::StatementCheck).is_err(),
            "{source}"
        );
    }
}

#[test]
fn step5c10_citations_are_prior_clean_owners_not_names() {
    let source = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let renamed = source
        .replace("Lem1", "Earlier")
        .replace("UseLem1", "Later")
        .replace("X", "Bound");
    assert_eq!(
        check_theorem_source(&renamed, PipelinePhase::VcGeneration)
            .unwrap()
            .1
            .unwrap()
            .vcs()
            .len(),
        2
    );
    for source in [
        source.replace("by Lem1", "by Missing"),
        source.replace("by Lem1", "by UseLem1"),
        source.replace("thus X = X;", "thus X = X by UseLem1;"),
    ] {
        assert_eq!(
            check_theorem_source(&source, PipelinePhase::Resolve).unwrap_err(),
            "theorems.reference.unknown_label",
            "{source}"
        );
    }
    for source in [
        source.replace("by Lem1", "by Lem1, Lem1"),
        source.replace("lemma Lem1", "assumed lemma Lem1"),
        source
            .replacen("being set", "being object", 1)
            .replacen("be set", "be object", 1),
    ] {
        assert!(
            check_theorem_source(&source, PipelinePhase::VcGeneration).is_err(),
            "{source}"
        );
    }
}

#[test]
fn step5c10_status_projection_never_supplies_a_clean_dependency() {
    use mizar_proof::policy::{
        CandidatePolicyClass, PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
    };
    let source = "open theorem O: for X being set holds X = X; assumed theorem A: for X being set holds X = X;";
    let (classes, vcs) = check_theorem_source(source, PipelinePhase::StatementCheck).unwrap();
    assert_eq!(
        classes,
        [
            CandidatePolicyClass::OpenAllowed,
            CandidatePolicyClass::AssumedByPolicy
        ]
    );
    assert!(vcs.is_none());
    assert!(check_theorem_source(source, PipelinePhase::VcGeneration).is_err());
    for status in ["open", "assumed"] {
        let dependency = format!(
            "{status} theorem A: for X being set holds X = X; theorem T: for X being set holds X = X proof let X be set; thus X = X by A; end;"
        );
        assert!(check_theorem_source(&dependency, PipelinePhase::StatementCheck).is_err());
        let justified = format!(
            "{status} theorem T: for X being set holds X = X proof let X be set; thus X = X; end;"
        );
        assert!(check_theorem_source(&justified, PipelinePhase::StatementCheck).is_err());
    }
    for policy in [VerifierPolicy::development(), VerifierPolicy::release()] {
        let evaluator = ProofPolicyEvaluator::new(policy);
        for candidate in [
            PolicyCandidate::OpenObligation,
            PolicyCandidate::PolicyAssumption,
        ] {
            let decision = evaluator.evaluate_candidate(&candidate);
            assert!(!decision.can_schedule_kernel_check);
            assert!(decision.kernel_evidence_check_kind.is_none());
            assert!(!matches!(
                decision.class,
                CandidatePolicyClass::KernelVerified
                    | CandidatePolicyClass::DischargedBuiltin
                    | CandidatePolicyClass::KernelCheckable
            ));
        }
    }
}

#[test]
fn step5c10_theorem_receipts_reject_substituted_resolution() {
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_resolve::{
        env::NamespacePath,
        labels::{LabelProjection, LabelProjectionData, LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::SurfaceResolvedArena,
    };
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::super::formula_statement::STEP5C10_CASES[0].0)
        .unwrap();
    let source = fs::read_to_string(&case.source_path)
        .unwrap()
        .replace("by Lem1", "by Missing");
    let ast = parse(&source);
    let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &resolver.module, &resolver.env),
    )
    .unwrap();
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::super::super::type_elaboration::step5c8_formula_typed_ast(
        &ast,
        &resolver.module,
        &resolver.env,
        &scope,
        &bindings,
        true,
    )
    .unwrap();
    let arena = SurfaceResolvedArena::lower(&ast, &resolver.module).unwrap();
    let namespace = NamespacePath::new(resolver.module.path().as_str());
    let labels = ProofLabelSourceCollector::new(
        &ast,
        &resolver.module,
        namespace.clone(),
        resolver.env.symbols().iter().next().unwrap().contribution(),
        &arena,
    )
    .unwrap()
    .collect_with_theorem_owners(&resolver.env)
    .unwrap();
    let genuine = LabelResolver::new(labels.projections()).resolve(
        &resolver.module,
        &namespace,
        labels.references(),
    );
    assert_eq!(
        SourceVariableSemanticsChecker::check_theorem_skeletons(
            &arena,
            &typed,
            &scope,
            &resolver.env,
            &labels,
            &genuine,
            None,
        )
        .unwrap_err(),
        "theorems.reference.unknown_label"
    );
    let owner = labels
        .projections()
        .iter()
        .find(|label| label.primary_spelling() == "Lem1")
        .unwrap();
    let renamed = LabelProjection::current_module(
        LabelProjectionData {
            origin_path: owner.origin_path().clone(),
            module: owner.module().clone(),
            namespace: owner.namespace().clone(),
            primary_spelling: "Missing".into(),
            kind: owner.kind(),
            declaration_range: owner.declaration_range(),
            origin: owner.origin().clone(),
            contribution: owner.contribution(),
        },
        match owner.source() {
            mizar_resolve::labels::LabelProjectionSource::CurrentModule {
                visible_after_ordinal,
                ..
            } => *visible_after_ordinal,
            _ => panic!("expected local theorem label"),
        },
    );
    let substituted =
        LabelResolver::new(&[renamed]).resolve(&resolver.module, &namespace, labels.references());
    assert!(!substituted.has_unresolved());
    assert_eq!(
        SourceVariableSemanticsChecker::check_theorem_skeletons(
            &arena,
            &typed,
            &scope,
            &resolver.env,
            &labels,
            &substituted,
            None,
        )
        .unwrap_err(),
        "theorems.source.invalid"
    );
}

#[test]
fn step5c10_theorem_receipts_reject_foreign_source_identity() {
    use mizar_session::SessionIdAllocator;
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::super::formula_statement::STEP5C10_CASES[0].0)
        .unwrap();
    let mut ast = parse(&fs::read_to_string(&case.source_path).unwrap());
    let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
    let ids = mizar_session::InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(777)).unwrap();
    ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
    assert!(
        theorem_ast_output(
            &ast,
            &resolver.module,
            &resolver.env,
            PipelinePhase::VcGeneration,
            snapshot_id(777),
            None,
        )
        .is_err()
    );
}
