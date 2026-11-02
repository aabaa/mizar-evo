use super::*;

#[test]
fn contextual_computation_inputs_parse_but_fail_actual_source_scope() {
    use mizar_resolve::{
        names::{
            SourceVariableScopeError as Error, SourceVariableScopeInput as Input,
            SourceVariableScopeResolver as Resolver,
        },
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_syntax::SurfaceNodeKind as K;

    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::super::formula_statement::STEP5C10_CASES[0].0)
        .unwrap();
    for (argument, owner) in [
        ("0 = 0", None),
        ("not 0 = 0", None),
        ("thesis", Some("thus")),
        ("(thesis)", Some("hence")),
    ] {
        let justification = format!("by computation(calc({argument}));");
        let text = match owner {
            Some(owner) => {
                format!("theorem T: 0 = 0 proof {owner} thesis {justification} end;")
            }
            None => format!("theorem T: 0 = 0 {justification}"),
        };
        let frontend = super::super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(
            frontend.diagnostics.is_empty(),
            "{text}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        let application = ast
            .node_views()
            .filter(|node| matches!(node.kind(), K::ComputationJustification))
            .flat_map(|node| node.child_views())
            .find(|node| matches!(node.kind(), K::ApplicationTerm))
            .unwrap();
        let formulas = application
            .child_views()
            .filter(|node| matches!(node.kind(), K::FormulaExpression))
            .collect::<Vec<_>>();
        assert_eq!(formulas.len(), 1, "{text}");
        assert_eq!(
            &text[formulas[0].range().start..formulas[0].range().end],
            argument
        );

        let resolver = resolver_symbol_collection(&config.workspace_root, case, ast);
        assert!(
            resolver.detail_keys.is_empty(),
            "{text}: {:?}",
            resolver.detail_keys
        );
        let source = SurfaceResolvedArena::lower(ast, &resolver.module).unwrap();
        let scope =
            Resolver::resolve_proof_occurrences(Input::new(ast, source.module(), &resolver.env));
        assert_eq!(scope.unwrap_err(), Error::InvalidShape, "{text}");
        assert_eq!(
            step5c14_computation_core(&text).unwrap_err(),
            "InvalidShape",
            "{text}"
        );
    }
}

#[test]
fn source_theorem_formula_payloads_preserve_actual_locals_and_content() {
    use mizar_core::{
        binder_normalization::{
            BinderContext, CanonicalFormulaKind, CanonicalTermKind, CanonicalVar,
            NormalizedVarClass, NormalizedVarSort, canonical_formula, normalize_core_formula,
        },
        core_ir::{CoreFormulaKind, CoreProofNodeKind},
        elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
    };
    use mizar_vc::vc_ir::VcFormulaRef;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut baseline = None;
    for source in [
        original.to_owned(),
        "theorem Renamed: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;".to_owned(),
        format!(":: leading trivia\n{}\n", original.replace(" proof ", "\nproof\n")),
    ] {
        let (core, payloads) = with_step5c14_theorem_check(&source, |check| {
            let core = lower_source_theorem_skeletons(check, None)?;
            let before = core.clone();
            let proof = core.proofs().iter().next().unwrap().1;
            let CoreFormulaKind::Forall { binders, .. } =
                &core.formulas().get(proof.proposition).unwrap().kind
            else {
                panic!()
            };
            let CoreProofNodeKind::IntroduceBinder { binder: local, .. } =
                &core.proof_nodes().get(proof.root).unwrap().kind
            else {
                panic!()
            };
            assert_ne!(binders[0].var, local.var);
            assert_eq!(binders[0].role.as_str(), "quantifier");
            assert_eq!(local.role.as_str(), "proof-let");
            let guard = local.ty_guard.unwrap();
            let goal = core.obligation_seeds().iter().next().unwrap().1.goal.unwrap();
            let vcs = generate_core_vcs(&core, snapshot_id(777))?;
            assert_eq!(vcs.vcs().len(), 1);
            assert_eq!(vcs.vcs()[0].goal, VcFormulaRef::Core(goal));
            assert_eq!(
                vcs.vcs()[0].local_context.entries()[0].formula,
                Some(VcFormulaRef::Core(guard))
            );
            let mut context = BinderContext::new();
            for binder in [&binders[0], local] {
                context.declare_variable(
                    binder.var,
                    NormalizedVarClass::Free,
                    binder.role.clone(),
                    NormalizedVarSort::Term,
                );
            }
            for (id, var) in [(guard, local.var), (binders[0].ty_guard.unwrap(), binders[0].var)] {
                let row = core.formulas().get(id).unwrap();
                let normalized = normalize_core_formula(&core, id, &context, &row.source)
                    .map_err(|e| format!("{e:?}"))?;
                assert_eq!(normalized.free_variables, [var].into_iter().collect());
                let canonical = canonical_formula(&normalized, &context, &row.source)
                    .map_err(|e| format!("{e:?}"))?;
                let CanonicalFormulaKind::TypePred { subject, .. } = canonical.kind() else {
                    panic!()
                };
                assert_eq!(subject.kind(), &CanonicalTermKind::Var(CanonicalVar::Free(var)));
            }
            let mut prefix = b"mizar-core/source-theorem-formula/v1\0".to_vec();
            for value in [
                core.module_id().package().as_str(),
                core.module_id().path().as_str(),
                "proof-let",
            ] {
                prefix.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
                prefix.extend_from_slice(value.as_bytes());
            }
            let mut guard_bytes = prefix.clone();
            guard_bytes.extend_from_slice(&[3, 0, 0, 0, 3, b's', b'e', b't', 2, 0, 0, 0, 0]);
            let mut goal_bytes = prefix;
            goal_bytes.extend_from_slice(&[2, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0]);
            let expected = [(guard, guard_bytes.clone()), (goal, goal_bytes.clone())]
                .into_iter()
                .collect();
            for roots in [[guard, goal], [goal, guard], [guard, goal]] {
                assert_eq!(source_theorem_formula_payloads(check, &core, &roots)?, expected);
            }
            for (id, bytes) in [(guard, guard_bytes.clone()), (goal, goal_bytes.clone())] {
                assert_eq!(
                    source_theorem_formula_payloads(check, &core, &[id])?,
                    [(id, bytes)].into_iter().collect()
                );
            }
            assert!(source_theorem_formula_payloads(check, &core, &[])?.is_empty());
            assert_eq!(core, before);
            Ok((core, [guard_bytes, goal_bytes]))
        }).unwrap();
        if let Some((previous, bytes)) = &baseline {
            assert_ne!(&core, previous);
            assert_eq!(&payloads, bytes);
        } else {
            baseline = Some((core, payloads));
        }
    }
}

#[test]
fn source_theorem_formula_payloads_refuse_foreign_roots_and_profiles() {
    use mizar_core::{
        core_ir::*,
        elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let invalid = "unsupported source theorem formula payload";
    for source in [
        original.to_owned(),
        "open theorem O: for X being set holds X = X;".to_owned(),
        original.replacen("theorem", "lemma", 1),
        original.replace("set", "object"),
        format!("{original} {}", original.replace("T:", "U:")),
        "theorem C: 0 = 0 by computation(steps: 8);".to_owned(),
        include_str!(
            "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ).to_owned(),
    ] {
        with_step5c14_theorem_check(&source, |check| {
            if source.starts_with("open theorem") {
                let lower_error = lower_source_theorem_skeletons(check, None).unwrap_err();
                assert_eq!(lower_error, "theorems.core.status_unsupported");
                let foreign = step5c14_computation_core(
                    "theorem C: 0 = 0 by computation(steps: 8);",
                )?;
                assert!(foreign.proof_nodes().iter().any(|(_, node)| {
                    matches!(node.kind, CoreProofNodeKind::ComputationGoal { .. })
                }));
                let roots = foreign.obligation_seeds().iter().flat_map(|(_, seed)| {
                    seed.context.iter().copied().chain(seed.goal)
                }).collect::<Vec<_>>();
                assert!(!roots.is_empty());
                for requested in [roots.as_slice(), &[]] {
                    assert_eq!(
                        source_theorem_formula_payloads(check, &foreign, requested).unwrap_err(),
                        lower_error
                    );
                }
                return Ok(());
            }
            let core = lower_source_theorem_skeletons(check, None)?;
            let before = core.clone();
            let roots = core.obligation_seeds().iter().flat_map(|(_, seed)| {
                seed.context.iter().copied().chain(seed.goal)
            }).collect::<Vec<_>>();
            assert!(!roots.is_empty());
            if source != original {
                if source.contains("lemma T:") {
                    assert_eq!(core.items().iter().next().unwrap().1.kind, CoreItemKind::Lemma);
                }
                if source.contains("being object") {
                    assert_eq!(core.items().len(), 1);
                    assert_eq!(core.proof_nodes().len(), 2);
                    assert_eq!(roots.len(), 2);
                    let CoreFormulaKind::TypePred { ty, .. } =
                        &core.formulas().get(roots[0]).unwrap().kind
                    else {
                        panic!()
                    };
                    assert_eq!(ty.as_str(), "object");
                    assert_eq!(
                        source_theorem_formula_payloads(check, &core, &roots[1..]).unwrap_err(),
                        invalid
                    );
                }
                assert_eq!(source_theorem_formula_payloads(check, &core, &roots).unwrap_err(), invalid);
                assert_eq!(source_theorem_formula_payloads(check, &core, &[]).unwrap_err(), invalid);
            } else {
                let proof = core.proofs().iter().next().unwrap().1;
                let CoreFormulaKind::Forall { binders, body } =
                    &core.formulas().get(proof.proposition).unwrap().kind
                else {
                    panic!()
                };
                for refused in [
                    vec![roots[0], roots[0]],
                    vec![CoreFormulaId::new(usize::MAX)],
                    vec![proof.proposition],
                    vec![binders[0].ty_guard.unwrap()],
                    vec![*body],
                    vec![roots[0], *body],
                ] {
                    assert_eq!(source_theorem_formula_payloads(check, &core, &refused).unwrap_err(), invalid);
                }
                let foreign = step5c14_computation_core(&source.replace("T:", "Different:"))?;
                for requested in [roots.as_slice(), &[]] {
                    assert_eq!(source_theorem_formula_payloads(check, &foreign, requested).unwrap_err(), invalid);
                }
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
                let CoreFormulaKind::Equals { left, .. } =
                    core.formulas().get(roots[1]).unwrap().kind
                else {
                    panic!()
                };
                parts.terms.get_mut(left).unwrap().kind = CoreTermKind::Var(binders[0].var);
                let mutated = CoreIr::try_new(parts).unwrap();
                assert_ne!(mutated, core);
                for requested in [roots.as_slice(), &[]] {
                    assert_eq!(source_theorem_formula_payloads(check, &mutated, requested).unwrap_err(), invalid);
                }
            }
            assert_eq!(core, before);
            Ok(())
        }).unwrap();
    }
}

#[test]
fn source_theorem_core_semantic_payload_preserves_original_and_local_content() {
    use mizar_core::{
        binder_normalization::{
            BinderContext, CanonicalFormulaKind as F, CanonicalTermKind as T, CanonicalVar,
            canonical_formula, normalize_core_formula,
        },
        core_ir::*,
        elaborator::{
            lower_source_theorem_skeletons, source_theorem_core_semantic_payload,
            source_theorem_formula_payloads,
        },
    };
    use std::collections::BTreeSet;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut baseline = None;
    for (source, same_owner) in [
        (original.to_owned(), true),
        ("theorem T: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;".to_owned(), true),
        (format!(":: trivia\n{}\n", original.replace(" proof ", "\nproof\n")), true),
        (original.replace("T:", "Other:"), false),
    ] {
        let (core, bytes) = with_step5c14_theorem_check(&source, |check| {
            let check_before = format!("{check:?}");
            let core = lower_source_theorem_skeletons(check, None)?;
            let before = core.clone();
            assert_eq!(core.items().len(), 1);
            assert_eq!(core.proofs().len(), 1);
            assert_eq!(core.proof_nodes().len(), 2);
            assert_eq!(core.obligation_seeds().len(), 1);
            assert_eq!(core.formulas().len(), 5);
            assert_eq!(core.terms().len(), 6);
            assert!(core.definitions().is_empty());
            assert!(core.algorithms().is_empty());
            assert!(core.algorithm_statements().is_empty());
            assert!(core.generated().is_empty());
            assert!(core.diagnostics().is_empty());
            let (item_id, item) = core.items().iter().next().unwrap();
            let (proof_id, proof) = core.proofs().iter().next().unwrap();
            let (seed_id, seed) = core.obligation_seeds().iter().next().unwrap();
            assert_eq!(item.kind, CoreItemKind::Theorem);
            assert_eq!(item.status, CoreItemStatus::Valid);
            assert!(item.dependencies.is_empty());
            assert!(item.diagnostics.is_empty());
            assert_eq!(proof.item, item_id);
            assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
            assert_eq!(seed.owner, item_id);
            assert_eq!(seed.kind, ObligationSeedKind::TheoremProof);
            assert_eq!(seed.status, ObligationSeedStatus::Active);
            assert!(seed.label.is_none());
            assert!(seed.diagnostics.is_empty());
            assert_eq!(item.symbol.module(), core.module_id());
            assert!(!item.visibility.as_str().is_empty());
            assert_eq!(seed.local_path.as_str(), format!("proof/{}", item.symbol.fqn().as_str()));
            assert_eq!(seed.semantic_origin.as_str(), format!("{}.proof", item.symbol.fqn().as_str()));
            let root = core.proof_nodes().get(proof.root).unwrap();
            let CoreProofNodeKind::IntroduceBinder { binder: local, child } = &root.kind else {
                panic!()
            };
            let terminal = core.proof_nodes().get(*child).unwrap();
            let CoreProofNodeKind::TerminalGoal { obligation, citations } = &terminal.kind else {
                panic!()
            };
            assert_eq!(*obligation, seed_id);
            assert!(citations.is_empty());
            assert!(root.diagnostics.is_empty());
            assert!(terminal.diagnostics.is_empty());
            let guard = local.ty_guard.unwrap();
            let goal = seed.goal.unwrap();
            assert_eq!(seed.context, [guard]);
            let proposition = core.formulas().get(proof.proposition).unwrap();
            let CoreFormulaKind::Forall { binders, body } = &proposition.kind else {
                panic!()
            };
            assert_eq!(binders.len(), 1);
            assert_ne!(binders[0].var, local.var);
            assert_eq!(binders[0].role.as_str(), "quantifier");
            assert_eq!(local.role.as_str(), "proof-let");
            let original_guard = binders[0].ty_guard.unwrap();
            let formulas = [proof.proposition, original_guard, *body, guard, goal]
                .into_iter().collect::<BTreeSet<_>>();
            assert_eq!(formulas.len(), 5);
            assert_eq!(formulas, core.formulas().iter().map(|(id, _)| id).collect());
            let mut terms = BTreeSet::new();
            for (id, var) in [(original_guard, binders[0].var), (guard, local.var)] {
                let CoreFormulaKind::TypePred { subject, ty } = &core.formulas().get(id).unwrap().kind else {
                    panic!()
                };
                assert_eq!(ty.as_str(), "set");
                assert_eq!(core.terms().get(*subject).unwrap().kind, CoreTermKind::Var(var));
                assert!(terms.insert(*subject));
            }
            for (id, var) in [(*body, binders[0].var), (goal, local.var)] {
                let CoreFormulaKind::Equals { left, right } = &core.formulas().get(id).unwrap().kind else {
                    panic!()
                };
                for term in [*left, *right] {
                    assert_eq!(core.terms().get(term).unwrap().kind, CoreTermKind::Var(var));
                    assert!(terms.insert(term));
                }
            }
            assert_eq!(terms, core.terms().iter().map(|(id, _)| id).collect());
            let mut refs = vec![CoreNodeRef::Item(item_id), CoreNodeRef::Proof(proof_id),
                CoreNodeRef::ProofNode(*child), CoreNodeRef::Formula(guard), CoreNodeRef::Formula(goal)];
            refs.sort();
            assert_eq!(seed.core_refs, refs);
            let context = BinderContext::new();
            let normalized = normalize_core_formula(&core, proof.proposition, &context, &proposition.source)
                .map_err(|e| format!("{e:?}"))?;
            assert!(normalized.free_variables.is_empty());
            let canonical = canonical_formula(&normalized, &context, &proposition.source)
                .map_err(|e| format!("{e:?}"))?;
            let F::Forall { binders, body } = canonical.kind() else { panic!() };
            assert_eq!(binders.len(), 1);
            assert_eq!(binders[0].role.as_str(), "quantifier");
            let F::TypePred { subject, ty } = binders[0].ty_guard.as_ref().unwrap().kind() else { panic!() };
            assert_eq!(ty.as_str(), "set");
            let F::Equals { left, right } = body.kind() else { panic!() };
            for term in [subject, left, right] {
                assert_eq!(term.kind(), &T::Var(CanonicalVar::Bound(0)));
            }
            // Independent grammar vectors precede the new operation.
            let mut prefix = b"mizar-core/source-theorem-formula/v1\0".to_vec();
            for text in [core.module_id().package().as_str(), core.module_id().path().as_str(), "proof-let"] {
                prefix.extend_from_slice(&u32::try_from(text.len()).unwrap().to_be_bytes());
                prefix.extend_from_slice(text.as_bytes());
            }
            let mut guard_bytes = prefix.clone();
            guard_bytes.extend_from_slice(&[3, 0, 0, 0, 3, b's', b'e', b't', 2, 0, 0, 0, 0]);
            let mut goal_bytes = prefix;
            goal_bytes.extend_from_slice(&[2, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0]);
            let expected_locals = [(guard, guard_bytes.clone()), (goal, goal_bytes.clone())].into_iter().collect();
            assert_eq!(source_theorem_formula_payloads(check, &core, &[guard, goal])?, expected_locals);
            let mut tree = vec![7, 0, 0, 0, 1, 0, 0, 0, 10];
            tree.extend_from_slice(b"quantifier");
            tree.extend_from_slice(&[1, 3, 0, 0, 0, 3, b's', b'e', b't', 1, 0, 0, 0, 0,
                2, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
            let mut expected = b"mizar-core/source-theorem-core-semantics/v1\0".to_vec();
            for field in [
                core.module_id().package().as_str().as_bytes(),
                core.module_id().path().as_str().as_bytes(),
                item.symbol.module().package().as_str().as_bytes(),
                item.symbol.module().path().as_str().as_bytes(),
                item.symbol.local().as_str().as_bytes(), item.symbol.fqn().as_str().as_bytes(),
                item.visibility.as_str().as_bytes(), tree.as_slice(), guard_bytes.as_slice(),
                goal_bytes.as_slice(), seed.local_path.as_str().as_bytes(), seed.semantic_origin.as_str().as_bytes(),
            ] {
                expected.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                expected.extend_from_slice(field);
            }
            let bytes = source_theorem_core_semantic_payload(check, &core)?;
            assert_eq!(bytes, expected);
            assert_eq!(source_theorem_core_semantic_payload(check, &core)?, bytes);
            assert_eq!(core, before);
            assert_eq!(format!("{check:?}"), check_before);
            Ok((core, bytes))
        }).unwrap();
        if let Some((previous, previous_bytes)) = &baseline {
            assert_ne!(&core, previous);
            if same_owner {
                assert_eq!(&bytes, previous_bytes);
            } else {
                assert_ne!(&bytes, previous_bytes);
            }
        } else {
            baseline = Some((core, bytes));
        }
    }
}

#[test]
fn source_theorem_core_semantic_payload_preserves_genuine_refusals() {
    use mizar_core::{
        core_ir::*,
        elaborator::{
            lower_source_theorem_skeletons, source_theorem_core_semantic_payload,
            source_theorem_formula_payloads,
        },
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let foreign = step5c14_computation_core(&original.replace("T:", "Foreign:")).unwrap();
    for source in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        original.replace("set", "object"),
        "open theorem O: for X being set holds X = X;".to_owned(),
    ] {
        with_step5c14_theorem_check(&source, |check| {
            let check_before = format!("{check:?}");
            let core = if source.starts_with("open theorem") {
                assert_eq!(
                    lower_source_theorem_skeletons(check, None).unwrap_err(),
                    "theorems.core.status_unsupported"
                );
                foreign.clone()
            } else {
                let lowered = lower_source_theorem_skeletons(check, None)?;
                if source == original {
                    assert_ne!(lowered, foreign);
                    assert_ne!(
                        lowered.items().iter().next().unwrap().1.symbol,
                        foreign.items().iter().next().unwrap().1.symbol
                    );
                    foreign.clone()
                } else {
                    let (_, item) = lowered.items().iter().next().unwrap();
                    if source.starts_with("lemma") {
                        assert_eq!(item.kind, CoreItemKind::Lemma);
                    } else {
                        let proof = lowered.proofs().iter().next().unwrap().1;
                        let CoreFormulaKind::Forall { binders, .. } =
                            &lowered.formulas().get(proof.proposition).unwrap().kind
                        else {
                            panic!()
                        };
                        let seed = lowered.obligation_seeds().iter().next().unwrap().1;
                        for id in [binders[0].ty_guard.unwrap(), seed.context[0]] {
                            let CoreFormulaKind::TypePred { ty, .. } =
                                &lowered.formulas().get(id).unwrap().kind
                            else {
                                panic!()
                            };
                            assert_eq!(ty.as_str(), "object");
                        }
                    }
                    lowered
                }
            };
            let before = core.clone();
            let roots = core
                .obligation_seeds()
                .iter()
                .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
                .collect::<Vec<_>>();
            let expected = source_theorem_formula_payloads(check, &core, &roots).unwrap_err();
            assert_eq!(
                expected,
                if source.starts_with("open theorem") {
                    "theorems.core.status_unsupported"
                } else {
                    "unsupported source theorem formula payload"
                }
            );
            assert_eq!(
                source_theorem_core_semantic_payload(check, &core).unwrap_err(),
                expected
            );
            assert_eq!(core, before);
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
    }
}

#[test]
fn source_theorem_file_staging_matches_genuine_file_and_legacy_boundaries() -> Result<(), String> {
    use mizar_artifact::store::SchemaVersion;
    use mizar_proof::{
        status::{ArtifactPublicationGap, ProjectedProofStatus},
        witness_store::stage,
    };
    use mizar_vc::{dependency_slice::DependencySliceCompleteness, vc_ir::VcStatus};

    with_source_theorem_file_staging_inputs(|_, actual, read, legacy| {
        let draft = &actual.0.2;
        let old_again = stage(draft.clone()).unwrap();
        assert_eq!(&old_again, legacy);
        assert_eq!(legacy.payload_schema(), draft.payload_schema());
        assert_ne!(
            legacy.staged_payload_hash(),
            &actual.2.witness_artifact_hash
        );
        let staged = draft.clone().stage_source_theorem_file(read, &actual.2)?;
        assert_eq!(staged.obligation_id(), legacy.obligation_id());
        assert_eq!(staged.obligation_anchor(), legacy.obligation_anchor());
        assert_eq!(
            staged.obligation_fingerprint(),
            legacy.obligation_fingerprint()
        );
        assert_eq!(staged.selected_class(), legacy.selected_class());
        assert_eq!(
            staged.selected_evidence_hash(),
            legacy.selected_evidence_hash()
        );
        assert_eq!(staged.witness_path(), legacy.witness_path());
        assert_eq!(staged.provenance(), legacy.provenance());
        assert_eq!(
            staged.payload_schema().family(),
            "mizar-proof/source-theorem-witness-file"
        );
        assert_eq!(staged.payload_schema().version(), SchemaVersion::new(1, 0));
        assert!(staged.payload_schema().canonical_bytes_required());
        assert_eq!(
            staged.staged_payload_hash(),
            &actual.2.witness_artifact_hash
        );
        assert_eq!(staged.artifact_reference_candidate(), Some(&actual.2));
        assert_eq!(
            staged,
            draft.clone().stage_source_theorem_file(read, &actual.2)?
        );
        assert_eq!(
            actual.0.0.projected_status(),
            ProjectedProofStatus::Accepted
        );
        assert_eq!(
            actual.0.0.artifact_publication().gap(),
            Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
        );
        assert_eq!(actual.0.1.status(), &VcStatus::Open);
        assert_eq!(
            actual.0.1.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(actual.0.1.requires_cache_miss());
        assert!(!actual.0.0.reuse_metadata().cache_reuse_predicate_complete());
        Ok(())
    })
}

#[test]
fn source_theorem_file_staging_rejects_genuine_cross_associations() -> Result<(), String> {
    let mut products = Vec::new();
    with_source_theorem_file_staging_inputs(|_, actual, read, _| {
        products.push((actual.0.2.clone(), read.clone(), actual.2.clone()));
        Ok(())
    })?;
    for index in [2, 3, 4, 5] {
        let (baseline_draft, baseline_read, baseline_reference) = &products[0];
        let (foreign_draft, foreign_read, foreign_reference) = &products[index];
        assert_eq!(
            mizar_proof::witness_store::read_source_theorem_witness_file(
                baseline_read,
                baseline_reference,
            )?,
            baseline_draft.payload_bytes(),
        );
        assert_eq!(
            mizar_proof::witness_store::read_source_theorem_witness_file(
                foreign_read,
                foreign_reference,
            )?,
            foreign_draft.payload_bytes(),
        );
        assert!(
            baseline_draft.obligation_id() != foreign_draft.obligation_id()
                || baseline_draft.obligation_fingerprint()
                    != foreign_draft.obligation_fingerprint()
                || baseline_draft.witness_path() != foreign_draft.witness_path()
                || baseline_draft.kernel_acceptance() != foreign_draft.kernel_acceptance()
                || baseline_draft.payload_bytes() != foreign_draft.payload_bytes()
        );
        assert_eq!(
            baseline_draft
                .clone()
                .stage_source_theorem_file(foreign_read, foreign_reference)
                .unwrap_err(),
            "source theorem witness file staging mismatch",
        );
        assert_eq!(
            foreign_draft
                .clone()
                .stage_source_theorem_file(baseline_read, baseline_reference)
                .unwrap_err(),
            "source theorem witness file staging mismatch",
        );
    }
    Ok(())
}

#[test]
fn source_theorem_file_staging_preserves_reader_refusals_and_inputs() -> Result<(), String> {
    use mizar_artifact::store::{CanonicalJson, SchemaVersion, artifact_hash_domain};
    use mizar_proof::witness_store::read_source_theorem_witness_file as recover;
    use mizar_session::hash_text;

    with_source_theorem_file_staging_inputs(|index, actual, read, _| {
        if index != 0 {
            return Ok(());
        }
        let draft = &actual.0.2;
        let mut missing_hash = read.clone();
        missing_hash.artifact_hash = None;
        let mut incompatible_version = read.clone();
        let CanonicalJson::Object(fields) = &mut incompatible_version.value else {
            panic!("genuine file object")
        };
        fields.insert("schema_version".into(), CanonicalJson::string("2.0"));
        for invalid in [missing_hash, incompatible_version] {
            let old = recover(&invalid, &actual.2).unwrap_err();
            assert_eq!(
                draft
                    .clone()
                    .stage_source_theorem_file(&invalid, &actual.2)
                    .unwrap_err(),
                old,
            );
        }
        let mut wrong_reference = actual.2.clone();
        wrong_reference.witness_path = "proof-witnesses/foreign.json".into();
        let old = recover(read, &wrong_reference).unwrap_err();
        assert_eq!(
            draft
                .clone()
                .stage_source_theorem_file(read, &wrong_reference)
                .unwrap_err(),
            old,
        );
        let domain = artifact_hash_domain(
            "mizar-proof/source-theorem-witness-file",
            SchemaVersion::new(1, 0),
        );
        for kind in ["wire", "identity", "metadata"] {
            let mut edited = read.clone();
            let mut reference = actual.2.clone();
            let CanonicalJson::Object(fields) = &mut edited.value else {
                panic!("genuine file object")
            };
            match kind {
                "wire" => {
                    fields.insert("payload_bytes_hex".into(), CanonicalJson::string("00"));
                }
                "identity" => {
                    reference.obligation_id.push_str("-foreign");
                    fields.insert(
                        "obligation_id".into(),
                        CanonicalJson::string(reference.obligation_id.as_str()),
                    );
                }
                "metadata" => {
                    reference
                        .kernel_acceptance
                        .verifier_policy_fingerprint
                        .digest = hash_text("foreign-staging-policy");
                    let Some(CanonicalJson::Object(metadata)) = fields.get_mut("kernel_acceptance")
                    else {
                        panic!("genuine acceptance object")
                    };
                    metadata.insert(
                        "verifier_policy_fingerprint".into(),
                        CanonicalJson::string(
                            reference
                                .kernel_acceptance
                                .verifier_policy_fingerprint
                                .to_artifact_hash_string(),
                        ),
                    );
                }
                _ => unreachable!(),
            }
            let hash = domain.hash(&edited.value, &[]);
            edited.artifact_hash = Some(hash);
            reference.witness_artifact_hash.digest = hash;
            let recovered = recover(&edited, &reference)?;
            if kind == "wire" {
                assert_ne!(recovered, draft.payload_bytes());
            } else {
                assert_eq!(recovered, draft.payload_bytes());
            }
            assert_eq!(
                draft
                    .clone()
                    .stage_source_theorem_file(&edited, &reference)
                    .unwrap_err(),
                "source theorem witness file staging mismatch",
            );
        }
        Ok(())
    })
}
