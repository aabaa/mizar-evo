use super::*;
use crate::{expectation::PipelinePhase, staged_model::Stage};

#[test]
fn checker278_real_ordinary_root_survives_final_projection_and_rerun() {
    let config = step5c11_config();
    let case = step5c11_plan()
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::STEP5C13_OVERLOAD_IDS[2])
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let run = |text: &str| {
        super::super::source_ordinary_root(
            &config.workspace_root,
            case,
            super::super::formula_statement::step5c8_test_frontend(text),
        )
    };
    let first = run(&text).unwrap();
    assert_eq!(first, run(&text).unwrap());
    let renamed = text
        .replace("ovroot", "renamed_root")
        .replace("OvRoot", "RenamedRoot");
    let second = run(&format!(":: alternate source\n{renamed}")).unwrap();
    assert_eq!(second.resolved_overloads().len(), 1);
    assert!(second.source_overload().is_some());
    for invalid in [
        text.replace("reserve x for set", "reserve x for object"),
        text.replace("ovroot(x) = x", "ovroot(x,x) = x"),
    ] {
        assert!(run(&invalid).is_err(), "{invalid}");
    }
}

#[test]
fn checker278_ordinary_root_admission_and_trace_credit_are_explicit() {
    let config = step5c11_config();
    let plan = step5c11_plan();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::STEP5C13_OVERLOAD_IDS[2])
        .unwrap();
    assert!(super::super::step5c13_overload_admitted(
        &config.workspace_root,
        case
    ));
    for mutation in 0..4 {
        let mut changed = case.clone();
        match mutation {
            0 => changed.expectation.stage = Stage::TypeElaboration,
            1 => changed.expectation.tags.clear(),
            2 => changed.expectation.domain.push_str("_other"),
            _ => changed.expectation.expected_phase = Some(PipelinePhase::TypeCheck),
        }
        assert!(!super::super::step5c13_overload_admitted(
            &config.workspace_root,
            &changed
        ));
    }
    let trace =
        std::fs::read_to_string(config.workspace_root.join("tests/coverage/spec_trace.toml"))
            .unwrap();
    assert!(
        !trace.contains(
            &case
                .expectation
                .source
                .to_string_lossy()
                .replace(".miz", ".expect.toml")
        )
    );
}

#[test]
fn step5c13_real_source_selects_from_both_roots_at_both_sites_by_actual_type() {
    use mizar_checker::overload_resolution::{
        CandidateViabilityStatus, ExposedResultSource, OverloadResultStatus,
    };
    use mizar_checker::type_checker::{TypeHeadRef, check_source_distinct_loci_overloads};
    let case = step5c13_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (source_text, structure) in [
        (text.clone(), false),
        (
            text.replace("ovbox", "other_box")
                .replace("OvBox", "OtherBox")
                .replace("X", "A")
                .replace("B be", "C be")
                .replace("other_box B", "other_box C")
                .replace("B.d", "C.d")
                .replace(" d ", " value ")
                .replace(".d", ".value"),
            false,
        ),
        (
            text.replace("for X being set", "for X being OvBox")
                .replace("  let X be set;\n  thus", "  let X be OvBox;\n  thus")
                .replace("ovbox X = X", "ovbox X = X.d"),
            true,
        ),
    ] {
        let (source, typed, symbols) = step5c13_inputs(&source_text).unwrap();
        let outputs =
            check_source_distinct_loci_overloads(&source, &symbols, &typed, false, None).unwrap();
        assert_eq!(
            outputs,
            check_source_distinct_loci_overloads(&source, &symbols, &typed, false, None).unwrap()
        );
        let (normalization, collection, expansion, viability, graphs, selection, _) = outputs;
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 4);
        assert_eq!(expansion.candidates().len(), 4);
        assert_eq!(viability.decisions().len(), 4);
        assert_eq!(
            viability
                .decisions()
                .iter()
                .filter(|(_, row)| matches!(row.status, CandidateViabilityStatus::Viable { .. }))
                .count(),
            2
        );
        assert_eq!(
            viability
                .decisions()
                .iter()
                .filter(|(_, row)| matches!(row.status, CandidateViabilityStatus::Rejected { .. }))
                .count(),
            2
        );
        assert_eq!(graphs.graphs().len(), 2);
        assert_eq!(selection.results().len(), 2);
        assert!(selection.inserted_views().is_empty());
        let mut declarations = symbols
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Functor)
            .collect::<Vec<_>>();
        declarations.sort_by_key(|entry| match entry.origin().anchor() {
            mizar_session::SourceAnchor::Range(r) => r.start,
            _ => panic!("missing range"),
        });
        assert_eq!(declarations.len(), 2);
        let structure_symbol = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Structure)
            .unwrap()
            .symbol();
        let applications = source
            .arena()
            .iter()
            .filter(|(_, node)| {
                matches!(
                    node.kind(),
                    mizar_syntax::ast::SurfaceNodeKind::PrefixExpression(_)
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(applications.len(), 2);
        for (index, (_, output)) in collection.sites().iter().enumerate() {
            let (id, application) = applications[index];
            assert_eq!(
                output.owner,
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(id.index())
                )
            );
            assert_eq!(
                mizar_session::SourceAnchor::Range(output.source_range),
                *application.origin().anchor()
            );
            assert_eq!(
                output.arguments,
                vec![mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(application.children()[1].index())
                )]
            );
        }
        for (id, site) in collection.sites().iter() {
            assert!(site.expected.is_none());
            let candidates = collection
                .candidates()
                .iter()
                .filter(|(_, row)| row.site == id)
                .map(|(_, row)| row)
                .collect::<Vec<_>>();
            assert_eq!(candidates.len(), 2);
            assert_ne!(candidates[0].ordinary_root, candidates[1].ordinary_root);
            assert_ne!(candidates[0].parameters, candidates[1].parameters);
            for candidate in candidates {
                assert_eq!(candidate.symbol, candidate.ordinary_root);
                assert!(candidate.coherence.is_none());
                assert!(candidate.template.is_none());
                let declaration = declarations[candidate.provenance.declaration_order];
                assert_eq!(&candidate.symbol, declaration.symbol());
                assert_eq!(
                    mizar_session::SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                    *declaration.origin().anchor()
                );
                assert_eq!(candidate.parameters.len(), 1);
                let expected = if candidate.provenance.declaration_order == 0 {
                    TypeHeadRef::BuiltinSet
                } else {
                    TypeHeadRef::Structure(structure_symbol.clone())
                };
                assert_eq!(
                    normalization
                        .normalized_types()
                        .get(candidate.parameters[0])
                        .unwrap()
                        .head,
                    expected
                );
                assert_eq!(
                    normalization
                        .normalized_types()
                        .get(candidate.result.unwrap())
                        .unwrap()
                        .head,
                    TypeHeadRef::BuiltinSet
                );
                assert!(candidate.provenance.source_range.unwrap().end <= site.source_range.start);
            }
        }
        for (_, result) in selection.results().iter() {
            let OverloadResultStatus::Resolved {
                root,
                exposed_result: Some(exposed),
                refinements,
                inserted_views,
            } = &result.status
            else {
                panic!("{:?}", result.status)
            };
            let selected = graphs.candidates().get(*root).unwrap();
            assert_eq!(
                &selected.symbol,
                declarations[usize::from(structure)].symbol()
            );
            assert!(
                refinements.is_empty() && inserted_views.is_empty() && exposed.evidence.is_empty()
            );
            assert_eq!(exposed.source, ExposedResultSource::SelectedRoot);
            assert_eq!(exposed.result, selected.result);
            assert_eq!(
                normalization
                    .normalized_types()
                    .get(exposed.result.unwrap())
                    .unwrap()
                    .head,
                TypeHeadRef::BuiltinSet
            );
            let parameter = &normalization
                .normalized_types()
                .get(selected.parameters[0])
                .unwrap()
                .head;
            if structure {
                assert!(matches!(parameter, TypeHeadRef::Structure(_)));
            } else {
                assert_eq!(parameter, &TypeHeadRef::BuiltinSet);
            }
        }
    }
    let report = super::super::run_advanced_semantics_corpus(&step5c11_config()).unwrap();
    assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
    assert_eq!(report.results.len(), 12);
    assert_eq!(report.passed_count(), 12);
}

#[test]
fn step5c13_rejects_source_signature_selector_binding_and_order_corruption() {
    use mizar_checker::type_checker::check_source_distinct_loci_overloads;
    let text = std::fs::read_to_string(step5c13_case().source_path).unwrap();
    for (before, after) in [
        (
            "func Ov1Def: ovbox X -> set equals X",
            "func Ov1Def: ovbox X -> OvBox equals X",
        ),
        ("let B be OvBox", "let B be set"),
        ("field d -> set", "field d -> object"),
        ("equals B.d", "equals B.missing"),
        ("equals B.d", "equals X.d"),
        ("equals X;", "equals Y;"),
        ("ovbox B -> set", "ovbox X -> set"),
        ("let X be set;\n  thus", "let Y be set;\n  thus"),
        ("thus ovbox X = X", "thus ovbox Y = X"),
        ("holds ovbox X = X", "holds ovbox Y = X"),
        ("thus ovbox X = X", "thus X = X"),
        ("coherence;", "existence;"),
        ("func Ov2Def:", "redefine func Ov2Def:"),
    ] {
        assert!(text.contains(before));
        let input = step5c13_inputs(&text.replace(before, after));
        assert!(
            input
                .and_then(
                    |(source, typed, symbols)| check_source_distinct_loci_overloads(
                        &source, &symbols, &typed, false, None
                    )
                )
                .is_err(),
            "{before} -> {after}"
        );
    }
    let split = text.find("theorem OvUse1:").unwrap();
    let first_end = text.find("end;").unwrap() + 4;
    for changed in [
        format!("{}\n{}", &text[split..], &text[..split]),
        format!("{}\n{}", &text[..first_end], text),
        text[first_end..].to_owned(),
    ] {
        assert!(
            step5c13_inputs(&changed)
                .and_then(|(s, t, e)| check_source_distinct_loci_overloads(&s, &e, &t, false, None))
                .is_err()
        );
    }
}

#[test]
fn step5c13_authenticates_complete_source_environment_and_neutral_projection() {
    use mizar_checker::type_checker::check_source_distinct_loci_overloads;
    use mizar_checker::typed_ast::{NodeRecoveryState, TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let case = step5c13_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    let (source, typed, symbols) =
        super::super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
            .unwrap();
    check_source_distinct_loci_overloads(&source, &symbols, &typed, false, None).unwrap();
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for mutation in 0..7 {
        let mut nodes = raw.clone();
        let mut root = typed.root();
        match mutation {
            0 => nodes[0].kind = "Unrelated".into(),
            1 => nodes[0].resolved_node = nodes[1].resolved_node,
            2 => nodes[0].anchor = nodes[1].anchor.clone(),
            3 => nodes[0].recovery = NodeRecoveryState::Recovered,
            4 => nodes[0].typing = TypingState::Successful,
            5 => nodes.last_mut().unwrap().children.clear(),
            6 => root = None,
            _ => unreachable!(),
        }
        let changed = TypedArena::try_new(root, nodes).unwrap();
        assert!(
            check_source_distinct_loci_overloads(&source, &symbols, &changed, false, None).is_err(),
            "mutation {mutation}"
        );
    }
    let foreign = ResolverModuleId::new(PackageId::new("other"), ModulePath::new("other"));
    assert!(
        check_source_distinct_loci_overloads(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed,
            false, None
        )
        .is_err()
    );
    let (_, _, changed_env) = step5c13_inputs(&text.replace("ovbox", "otherbox")).unwrap();
    assert!(check_source_distinct_loci_overloads(&source, &changed_env, &typed, false, None).is_err());
}

#[test]
fn step5c13_admission_reserves_both_rows_and_all_stage_aliases() {
    let config = step5c11_config();
    let original = step5c13_case();
    assert!(super::super::step5c13_overload_admitted(
        &config.workspace_root,
        &original
    ));
    for original in [original.clone(), step5c13_registration_case()] {
        for mutation in 0..22 {
            let mut case = original.clone();
            match mutation {
                0 => case.id.0.push_str("_extra"),
                1 => case.expectation.id.0.push_str("_extra"),
                2 => case.source_path = case.source_path.with_file_name("wrong.miz"),
                3 => {
                    case.expectation_path =
                        case.expectation_path.with_file_name("wrong.expect.toml")
                }
                4 => case.expectation.source = PathBuf::from("wrong.miz"),
                5 => case.expectation.stage = crate::staged_model::Stage::TypeElaboration,
                6 => {
                    case.expectation.expected_phase =
                        Some(crate::expectation::PipelinePhase::TypeCheck)
                }
                7 => {
                    case.expectation.expected_outcome = if case.expectation.expected_outcome
                        == crate::expectation::ExpectedOutcome::Fail
                    {
                        crate::expectation::ExpectedOutcome::Pass
                    } else {
                        crate::expectation::ExpectedOutcome::Fail
                    }
                }
                8 => case.expectation.tags.clear(),
                9 => case.expectation.tags.push("extra".into()),
                10 => case.expectation.diagnostic_codes.push("E-UNRELATED".into()),
                11 => case.expectation.stable_detail_key = Some("wrong.detail".into()),
                12 => {
                    case.expectation.kind =
                        if case.expectation.kind == crate::expectation::TestKind::Fail {
                            crate::expectation::TestKind::Pass
                        } else {
                            crate::expectation::TestKind::Fail
                        }
                }
                13 => case.expectation.rejection_reason = Some("wrong.reason".into()),
                14 => case.expectation.domain = "wrong".into(),
                15 => case.expectation.schema_version = 2,
                16 => case.expectation.spec_refs[0].0.push_str("_wrong"),
                17 => case.expectation.profiles.push("wrong".into()),
                18 => case.expectation.ast_profile = Some("wrong".into()),
                19 => case.expectation.snapshots = Some("wrong".into()),
                20 => case.expectation.diagnostic_payloads.push("wrong".into()),
                21 => {
                    case.expectation.failure_category =
                        if case.expectation.failure_category.is_some() {
                            None
                        } else {
                            Some("wrong".into())
                        }
                }
                _ => unreachable!(),
            }
            assert!(
                !super::super::step5c13_overload_admitted(&config.workspace_root, &case),
                "mutation {mutation}"
            );
        }
    }
    let plan = build_test_plan(&config).unwrap();
    let negative = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::STEP5C13_OVERLOAD_IDS[1])
        .unwrap();
    assert!(super::super::step5c13_overload_admitted(
        &config.workspace_root,
        negative
    ));
    for source_case in [&original, negative] {
        for (stage, phase, tag) in [
            (
                crate::staged_model::Stage::ParseOnly,
                crate::expectation::PipelinePhase::Parse,
                "active_parse_only",
            ),
            (
                crate::staged_model::Stage::DeclarationSymbol,
                crate::expectation::PipelinePhase::Resolve,
                "active_declaration_symbol",
            ),
            (
                crate::staged_model::Stage::TypeElaboration,
                crate::expectation::PipelinePhase::TypeCheck,
                "active_type_elaboration",
            ),
            (
                crate::staged_model::Stage::FormulaStatement,
                crate::expectation::PipelinePhase::StatementCheck,
                "active_formula_statement",
            ),
            (
                crate::staged_model::Stage::ProofVerification,
                crate::expectation::PipelinePhase::VcGeneration,
                "active_proof_verification",
            ),
        ] {
            for alias in [false, true] {
                let mut case = source_case.clone();
                case.expectation.stage = stage;
                case.expectation.expected_phase = Some(phase);
                case.expectation.tags = vec![tag.into()];
                if alias {
                    case.id.0 = "unrelated".into();
                    case.source_path = case.source_path.with_file_name("unrelated.miz");
                    case.expectation_path = case
                        .expectation_path
                        .with_file_name("unrelated.expect.toml");
                }
                assert!(!super::super::is_active_parse_only(&case));
                assert!(!super::super::is_active_declaration_symbol(&case));
                assert!(!super::super::is_active_type_elaboration(&case));
                assert!(!super::super::formula_statement::is_active_formula_statement(
                    &config.workspace_root,
                    &case
                ));
                assert!(!super::super::is_active_proof_verification(&case));
                assert!(!super::super::step5c13_overload_admitted(
                    &config.workspace_root,
                    &case
                ));
            }
        }
    }
    assert!(
        super::super::validate_step5c11_registration_inventory(&config.workspace_root, &plan).is_empty()
    );
    for original in [original, negative.clone()] {
        let mut missing = plan.clone();
        missing.cases.retain(|case| case.id != original.id);
        let mut duplicate = plan.clone();
        duplicate.cases.push(original);
        for invalid in [missing, duplicate] {
            assert!(
                super::super::validate_step5c11_registration_inventory(&config.workspace_root, &invalid)
                    .iter()
                    .any(|d| d.code.0 == "E-ADVANCED-SEMANTICS-INVENTORY")
            );
        }
    }
}

#[test]
fn step5c13_checks_each_coherently_corrupted_source_callee() {
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let case = step5c13_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    for target in [2, 3] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut occurrence = 0;
        let mut changed_callee = None;
        for (index, node) in ast.nodes().iter().enumerate() {
            let children = node
                .children
                .iter()
                .map(|child| rebuilt[child.index()])
                .collect();
            let id = match &node.kind {
                K::Token(token) => {
                    let changed = token.text.as_ref() == "ovbox" && {
                        occurrence += 1;
                        occurrence - 1 == target
                    };
                    if changed {
                        changed_callee = Some(index);
                    }
                    builder.add_token(
                        token.kind,
                        if changed {
                            "wrong".into()
                        } else {
                            token.text.clone()
                        },
                        node.range,
                    )
                }
                K::PrefixExpression(operator)
                    if node.children.first().map(|id| id.index()) == changed_callee =>
                {
                    let mut operator = operator.clone();
                    operator.spelling = "wrong".into();
                    builder.add_node(K::PrefixExpression(operator), node.range, children)
                }
                kind => builder.add_node(kind.clone(), node.range, children),
            };
            rebuilt.push(id);
        }
        assert_eq!(occurrence, 4);
        let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
        let env =
            super::super::resolver_symbol_collection(&step5c11_config().workspace_root, &case, &changed);
        assert!(env.detail_keys.is_empty());
        let resolved = SurfaceResolvedArena::lower(&changed, &env.module).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&resolved, &env.env).unwrap();
        let typed = mizar_checker::typed_ast::TypedArena::try_new(
            Some(mizar_checker::typed_ast::TypedNodeId::new(
                resolved.arena().root().index(),
            )),
            resolved
                .arena()
                .iter()
                .map(|(id, node)| {
                    mizar_checker::typed_ast::TypedNode::new(
                        format!("{:?}", node.kind()),
                        node.origin().anchor().clone(),
                    )
                    .with_resolved_node(id)
                    .with_children(
                        node.children()
                            .iter()
                            .map(|child| mizar_checker::typed_ast::TypedNodeId::new(child.index()))
                            .collect(),
                    )
                })
                .collect(),
        )
        .unwrap();
        assert!(
            mizar_checker::type_checker::check_source_distinct_loci_overloads(
                &resolved, &env.env, &typed, false, None
            )
            .is_err(),
            "callee {target}"
        );
    }
}

#[test]
fn step5c13_attributed_consumer_derives_four_gates_six_widenings_and_two_ambiguities() {
    use mizar_checker::{
        overload_resolution::*, registration_resolution::*, type_checker::*, typed_ast::*,
    };
    let original = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let renamed = original
        .replace("OAADef", "LeftDefinition")
        .replace("OBBDef", "RightDefinition")
        .replace("oamarked", "leftmarked")
        .replace("obmarked", "rightmarked")
        .replace("OAExists", "LeftExists")
        .replace("OBExists", "RightExists")
        .replace("OABExists", "BothExist")
        .replace("Ov3Def", "FirstOverload")
        .replace("Ov4Def", "SecondOverload")
        .replace("ovpick", "choosebox")
        .replace("OvBad1", "Consumer")
        .replace("X", "Value");
    for text in [original, renamed] {
        let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
        let database = mizar_proof::status::prove_source_existential_registration(
            &source,
            &nodes,
            &symbols,
            super::super::shared::snapshot_id(0),
            &mizar_proof::policy::VerifierPolicy::release(),
        )
        .unwrap();
        let output =
            check_source_distinct_loci_overloads(&source, &symbols, &nodes, false, Some(&database))
                .unwrap();
        assert_eq!(
            output,
            check_source_distinct_loci_overloads(&source, &symbols, &nodes, false, Some(&database))
                .unwrap()
        );
        let (normalization, collection, expansion, viability, graphs, selection, details) = output;
        let (gates, coercions) = details.unwrap();
        assert_eq!(gates.len(), 4);
        assert_eq!(coercions.coercions().len(), 6);
        assert!(coercions.diagnostics().is_empty() && coercions.initial_obligations().is_empty());
        assert_eq!(
            coercions.normalized_types(),
            normalization.normalized_types()
        );
        let mut gate_sizes = Vec::new();
        for gate in gates.iter() {
            assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
            assert_eq!(
                nodes.node(gate.owner().node()).unwrap().anchor,
                SourceAnchor::Range(gate.source_range())
            );
            assert!(gate.base_evidence_kind().is_none() && gate.facts().is_empty());
            let active = database
                .activated()
                .iter()
                .find(|active| Some(active.id()) == gate.registration())
                .unwrap();
            assert_eq!(gate.pattern(), active.pattern());
            let SourceAnchor::Range(declaration) = active.source().origin().anchor() else {
                panic!("source registration")
            };
            assert!(declaration.end < gate.source_range().start);
            let entry = normalization
                .type_entries()
                .iter()
                .find(|(_, entry)| entry.owner == *gate.owner())
                .unwrap()
                .1;
            let TypeEntryActual::Known(ty) = entry.actual else {
                panic!("known gate type")
            };
            let mut expected = normalization
                .normalized_types()
                .get(ty)
                .unwrap()
                .attributes
                .positive()
                .iter()
                .map(|attribute| RegistrationAttributeKey::new(format!("{:?}", attribute.symbol)))
                .collect::<Vec<_>>();
            expected.sort();
            assert_eq!(gate.attributes(), expected.as_slice());
            gate_sizes.push(gate.attributes().len());
        }
        gate_sizes.sort();
        assert_eq!(gate_sizes, [1, 1, 2, 2]);
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 4);
        let mut argument_coercions = std::collections::BTreeSet::new();
        for (_, decision) in viability.decisions().iter() {
            let CandidateViabilityStatus::Viable { views } = &decision.status else {
                panic!("{:?}", decision.status)
            };
            let [view] = views.as_slice() else {
                panic!("unary")
            };
            assert_eq!(view.kind, ArgumentViewKind::CoercionWidening);
            let row = coercions.coercions().get(view.coercion.unwrap()).unwrap();
            let candidate = expansion
                .candidates()
                .get(decision.source_candidate)
                .unwrap();
            let call = collection.sites().get(candidate.site).unwrap();
            assert_eq!(row.site, call.arguments[0]);
            assert_eq!((row.from, row.to), (Some(view.actual), view.target));
            assert_eq!(view.target, candidate.parameters[0]);
            assert_eq!(view.facts, row.supporting_facts);
            assert!(argument_coercions.insert(row.id));
        }
        assert_eq!(argument_coercions.len(), 4);
        let mut expected_bodies = std::collections::BTreeMap::new();
        for (_, block) in source
            .arena()
            .iter()
            .filter(|(_, node)| format!("{:?}", node.kind()) == "DefinitionBlockItem")
        {
            let definition = source.arena().node(block.children()[2]).unwrap();
            if format!("{:?}", definition.kind()) != "FunctorDefinition" {
                continue;
            }
            let parameter = source.arena().node(block.children()[1]).unwrap();
            let segment = source.arena().node(parameter.children()[1]).unwrap();
            let mut expected_types = Vec::new();
            for id in [segment.children()[2], definition.children()[5]] {
                let owner = TypedSiteRef::Node(TypedNodeId::new(id.index()));
                let entry = normalization
                    .type_entries()
                    .iter()
                    .find(|(_, entry)| entry.owner == owner)
                    .unwrap()
                    .1;
                let TypeEntryActual::Known(ty) = entry.actual else {
                    panic!("known definition type")
                };
                expected_types.push(ty);
            }
            let body = source.arena().node(definition.children()[7]).unwrap();
            let expression = source.arena().node(body.children()[0]).unwrap();
            let owner = TypedSiteRef::Node(TypedNodeId::new(expression.children()[0].index()));
            assert!(
                expected_bodies
                    .insert(owner, (Some(expected_types[0]), expected_types[1]))
                    .is_none()
            );
        }
        assert_eq!(expected_bodies.len(), 2);
        let mut body_count = 0;
        for (id, row) in coercions.coercions().iter() {
            assert_eq!(row.status, CoercionStatus::Candidate);
            assert!(row.obligation.is_none());
            let from = normalization
                .normalized_types()
                .get(row.from.unwrap())
                .unwrap();
            let target = normalization.normalized_types().get(row.to).unwrap();
            assert_eq!(from.head, TypeHeadRef::BuiltinSet);
            assert_eq!(target.head, from.head);
            assert_eq!(from.status, NormalizedTypeStatus::Known);
            assert_eq!(target.status, NormalizedTypeStatus::Known);
            assert!(target.attributes.positive().iter().all(|want| {
                from.attributes
                    .positive()
                    .iter()
                    .any(|have| have.symbol == want.symbol && have.args == want.args)
            }));
            for fact in &row.supporting_facts {
                let fact = coercions.facts().get(*fact).unwrap();
                assert_eq!(fact.subject, row.site);
                assert_eq!(fact.status, FactStatus::Known);
                assert!(matches!(fact.provenance, FactProvenance::Builtin(_)));
            }
            if !argument_coercions.contains(&id) {
                body_count += 1;
                assert_eq!(expected_bodies.remove(&row.site), Some((row.from, row.to)));
                assert_eq!(from.attributes.positive().len(), 1);
                assert!(target.attributes.positive().is_empty());
            }
        }
        assert_eq!(body_count, 2);
        assert!(expected_bodies.is_empty());
        for (_, graph) in graphs.graphs().iter() {
            assert_eq!(graph.nodes.len(), 2);
            assert_ne!(graph.nodes[0].ordinary_root, graph.nodes[1].ordinary_root);
            assert_eq!(graph.comparisons.len(), 1);
            assert_eq!(
                graph.comparisons[0].status,
                SpecificityComparisonOutcome::Incomparable
            );
            assert!(graph.edges.is_empty() && graph.diagnostics.is_empty());
        }
        assert!(selection.results().iter().all(|(_,result)|matches!(&result.status,OverloadResultStatus::Ambiguous{candidates} if candidates.len()==2)));
        let missing = SpecificityGraphOutput::build(&viability, []);
        let blocked = OverloadSelectionOutput::resolve(&missing, []);
        assert!(
            blocked
                .results()
                .iter()
                .all(|(_, result)| matches!(result.status, OverloadResultStatus::Blocked { .. }))
        );
        for status in [
            ViabilityCoercionStatus::PendingObligation,
            ViabilityCoercionStatus::Blocked,
            ViabilityCoercionStatus::Rejected,
            ViabilityCoercionStatus::MissingEvidence,
        ] {
            let inputs = viability.decisions().iter().map(|(_, decision)| {
                let CandidateViabilityStatus::Viable { views } = &decision.status else {
                    unreachable!()
                };
                let view = &views[0];
                CandidateViabilityInput {
                    candidate: decision.source_candidate,
                    arguments: vec![ArgumentViabilityEvidence::Coercion {
                        actual: view.actual,
                        target: view.target,
                        coercion: view.coercion.unwrap(),
                        kind: ViabilityCoercionKind::Widening,
                        status,
                        facts: view.facts.clone(),
                        path: None,
                    }],
                }
            });
            let rejected = CandidateViabilityOutput::filter(&expansion, inputs);
            assert!(rejected.decisions().iter().all(|(_, decision)| !matches!(
                decision.status,
                CandidateViabilityStatus::Viable { .. }
            )));
        }
    }
}

#[test]
fn step5c13_attributed_consumer_uses_each_calls_actual_type() {
    use mizar_checker::{
        overload_resolution::*, type_checker::check_source_distinct_loci_overloads,
    };
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    for (from, to) in [
        (
            "for X being oamarked obmarked set",
            "for X being oamarked set",
        ),
        ("let X be oamarked obmarked set", "let X be obmarked set"),
    ] {
        let changed = text.replace(from, to);
        let (source, nodes, symbols) = step5c13_registration_inputs(&changed).unwrap();
        let database = mizar_proof::status::prove_source_existential_registration(
            &source,
            &nodes,
            &symbols,
            super::super::shared::snapshot_id(0),
            &mizar_proof::policy::VerifierPolicy::release(),
        )
        .unwrap();
        let (_, collection, _, viability, graphs, selection, details) =
            check_source_distinct_loci_overloads(&source, &symbols, &nodes, false, Some(&database))
                .unwrap();
        assert_eq!(details.unwrap().1.coercions().len(), 4);
        assert_eq!(
            viability
                .decisions()
                .iter()
                .filter(|(_, decision)| matches!(
                    decision.status,
                    CandidateViabilityStatus::Rejected { .. }
                ))
                .count(),
            1
        );
        assert_eq!(
            selection
                .results()
                .iter()
                .filter(|(_, result)| matches!(
                    result.status,
                    OverloadResultStatus::Ambiguous { .. }
                ))
                .count(),
            1
        );
        let (_, resolved) = selection
            .results()
            .iter()
            .find(|(_, result)| matches!(result.status, OverloadResultStatus::Resolved { .. }))
            .unwrap();
        let OverloadResultStatus::Resolved {
            root,
            exposed_result: Some(exposed),
            ..
        } = &resolved.status
        else {
            panic!("selected declaration")
        };
        assert_eq!(
            exposed.result,
            graphs.candidates().get(*root).unwrap().result
        );
        let call = collection.sites().get(resolved.site).unwrap();
        assert_eq!(
            call.source_range.start,
            changed
                .find(if from.starts_with("for") {
                    "ovpick X = X\nproof"
                } else {
                    "ovpick X = X;"
                })
                .unwrap()
        );
        let diagnostic = viability
            .decisions()
            .iter()
            .find_map(|(_, decision)| match &decision.status {
                CandidateViabilityStatus::Rejected { reasons } => Some(&reasons[0]),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            diagnostic.reason,
            CandidateRejectionReason::RejectedEvidence
        );
    }
}

#[test]
fn step5c13_attributed_consumer_rejects_unproved_foreign_or_changed_inputs() {
    use mizar_checker::{
        registration_resolution::check_source_existential_registration_proof,
        type_checker::check_source_distinct_loci_overloads, typed_ast::*,
    };
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let duplicate = text.replace(
        "let X be oamarked set;\n  func Ov3Def: ovpick X -> set equals X;",
        "let Y be obmarked set;\n  func Ov3Def: ovpick Y -> set equals Y;",
    );
    assert_ne!(duplicate, text);
    let (duplicate_source, duplicate_nodes, duplicate_symbols) =
        step5c13_registration_inputs(&duplicate).unwrap();
    let duplicate_database = mizar_proof::status::prove_source_existential_registration(
        &duplicate_source,
        &duplicate_nodes,
        &duplicate_symbols,
        super::super::shared::snapshot_id(0),
        &mizar_proof::policy::VerifierPolicy::release(),
    )
    .unwrap();
    assert!(
        check_source_distinct_loci_overloads(
            &duplicate_source,
            &duplicate_symbols,
            &duplicate_nodes,
            false,
            Some(&duplicate_database)
        )
        .is_err()
    );
    for (from, to) in [
        ("let X be oamarked set", "let X be set"),
        ("let X be obmarked set", "let X be set"),
        ("for X being oamarked obmarked set", "for X being set"),
        ("let X be oamarked obmarked set", "let X be set"),
        ("holds ovpick X = X", "holds ovpick Missing = X"),
        ("thus ovpick X = X", "thus ovpick Missing = X"),
        ("holds ovpick X = X", "holds missing X = X"),
        ("thus ovpick X = X", "thus missing X = X"),
        ("holds ovpick X = X", "holds ovpick X = Missing"),
        ("thus ovpick X = X", "thus ovpick X = Missing"),
        (
            "func Ov3Def: ovpick X -> set equals X",
            "func Ov3Def: ovpick X -> set equals the set",
        ),
        (
            "func Ov4Def: ovpick X -> set equals X",
            "func Ov4Def: ovpick X -> object equals X",
        ),
        ("func Ov4Def:", "func Ov3Def:"),
        ("by OAADef, OBBDef;", "by OAADef;"),
        (
            "attr OAADef: X is oamarked means X = X",
            "attr OAADef: X is oamarked means not X = X",
        ),
        (
            "cluster OABExists: oamarked obmarked set",
            "cluster OABExists: oamarked set",
        ),
    ] {
        let changed = text.replacen(from, to, 1);
        assert_ne!(changed, text);
        let result = step5c13_registration_inputs(&changed).and_then(|(source, nodes, symbols)| {
            let database = mizar_proof::status::prove_source_existential_registration(
                &source,
                &nodes,
                &symbols,
                super::super::shared::snapshot_id(0),
                &mizar_proof::policy::VerifierPolicy::release(),
            )?;
            check_source_distinct_loci_overloads(&source, &symbols, &nodes, false, Some(&database))
        });
        assert!(result.is_err(), "accepted {from}->{to}");
    }
    let (attributes, rest) = text.split_once("registration\n").unwrap();
    let (registration, tail) = rest.split_once("\n\ndefinition\n").unwrap();
    for changed in [
        format!("{attributes}definition\n{tail}\nregistration\n{registration}"),
        format!("{text}\ntheorem Extra: for X being set holds X = X;\n"),
    ] {
        let result = step5c13_registration_inputs(&changed).and_then(|(source, nodes, symbols)| {
            let database = mizar_proof::status::prove_source_existential_registration(
                &source,
                &nodes,
                &symbols,
                super::super::shared::snapshot_id(0),
                &mizar_proof::policy::VerifierPolicy::release(),
            )?;
            check_source_distinct_loci_overloads(&source, &symbols, &nodes, false, Some(&database))
        });
        assert!(result.is_err(), "late registration or extra source item");
    }
    let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
    let database = mizar_proof::status::prove_source_existential_registration(
        &source,
        &nodes,
        &symbols,
        super::super::shared::snapshot_id(0),
        &mizar_proof::policy::VerifierPolicy::release(),
    )
    .unwrap();
    assert!(
        check_source_distinct_loci_overloads(&source, &symbols, &nodes, true, Some(&database))
            .is_err()
    );
    let pending = check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
    assert!(
        check_source_distinct_loci_overloads(
            &source,
            &symbols,
            &nodes,
            false,
            Some(pending.database())
        )
        .is_err()
    );
    let (other, other_nodes, other_symbols) =
        step5c13_registration_inputs(&text.replace("oamarked", "othermarked")).unwrap();
    let other_database = mizar_proof::status::prove_source_existential_registration(
        &other,
        &other_nodes,
        &other_symbols,
        super::super::shared::snapshot_id(0),
        &mizar_proof::policy::VerifierPolicy::release(),
    )
    .unwrap();
    assert!(
        check_source_distinct_loci_overloads(
            &source,
            &symbols,
            &nodes,
            false,
            Some(&other_database)
        )
        .is_err()
    );
    assert!(
        check_source_distinct_loci_overloads(
            &source,
            &other_symbols,
            &nodes,
            false,
            Some(&database)
        )
        .is_err()
    );
    for (target, _) in nodes.iter().filter(|(_, node)| {
        matches!(
            node.kind.as_str(),
            "AttributeChain" | "FunctorPattern" | "LetStatement"
        )
    }) {
        for mutation in 0..4 {
            let mut raw = nodes
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            match mutation {
                0 => raw[target.index()].resolved_node = None,
                1 => raw[target.index()].anchor = raw[nodes.root().unwrap().index()].anchor.clone(),
                2 => raw[target.index()].children.clear(),
                _ => raw[target.index()].recovery = NodeRecoveryState::Recovered,
            }
            let changed = TypedArena::try_new(nodes.root(), raw).unwrap();
            assert!(
                check_source_distinct_loci_overloads(
                    &source,
                    &symbols,
                    &changed,
                    false,
                    Some(&database)
                )
                .is_err()
            );
        }
    }
    assert!(
        mizar_proof::status::prove_source_existential_registration(
            &source,
            &nodes,
            &symbols,
            super::super::shared::snapshot_id(0),
            &mizar_proof::policy::VerifierPolicy::release().with_kernel_evidence_formats([])
        )
        .is_err()
    );
}

#[test]
fn step5c13_source_registration_proves_three_full_patterns_atomically() {
    use mizar_checker::registration_resolution::*;
    use mizar_core::core_ir::{CoreTermKind, ObligationSeedKind};
    use mizar_vc::vc_ir::{SeedNoVcReason, SeedVcMapping, VcStatus};
    let original = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let renamed = original.replace("OAADef", "LeftDefinition")
        .replace("OBBDef", "RightDefinition").replace("oamarked", "leftmarked")
        .replace("obmarked", "rightmarked").replace("OAExists", "LeftExists")
        .replace("OBExists", "RightExists").replace("OABExists", "BothExist")
        .replace("Ov3Def", "FirstOverload").replace("Ov4Def", "SecondOverload")
        .replace("ovpick", "selectvalue").replace("OvBad1", "LaterTheorem")
        .replace("X", "Value");
    for (index, text) in [original, renamed].into_iter().enumerate() {
        let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
        let checked = check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
        assert_eq!(checked.validations().len(), 3);
        assert_eq!(checked.database().pending().len(), 3);
        assert!(checked.database().activated().is_empty());
        let choices = checked.choice_terms().unwrap();
        assert_eq!(choices.terms().len(), 6);
        assert_eq!(choices.type_sites().len(), 6);
        assert_eq!(choices.requests().len(), 12);
        let gates = checked.choice_gates().unwrap();
        assert_eq!(gates.len(), 6);
        for ((_, choice), gate) in choices.terms().iter().zip(gates.iter()) {
            assert_eq!(gate.owner(), choice.site());
            assert_eq!(gate.source_range(), choice.source_range());
            assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
            assert_eq!(gate.base_evidence_kind(), Some(ExistentialGateBaseEvidenceKind::BuiltinSet));
            assert_eq!(gate.base_evidence_coverage(), Some(ExistentialGateBaseEvidenceCoverage::Builtin));
            assert!(gate.registration().is_none());
            assert!(gate.attributes().is_empty() && gate.facts().is_empty());
        }
        let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
        assert_eq!(core.definitions().len(), 2);
        assert_eq!(core.proofs().len(), 3);
        assert_eq!(core.generated().len(), 3);
        assert_eq!(core.obligation_seeds().len(), 6);
        let witnesses = core.terms().iter().filter(|(_, t)| matches!(t.kind, CoreTermKind::Apply { .. }))
            .collect::<Vec<_>>();
        assert_eq!(witnesses.len(), 6);
        for pair in witnesses.as_chunks::<2>().0 {
            assert_eq!(pair[0].1.kind, pair[1].1.kind);
            assert_ne!(pair[0].1.source, pair[1].1.source);
        }
        assert_ne!(witnesses[0].1.kind, witnesses[2].1.kind);
        assert_ne!(witnesses[2].1.kind, witnesses[4].1.kind);
        assert_ne!(witnesses[0].1.kind, witnesses[4].1.kind);
        assert_eq!(core.obligation_seeds().iter().filter(|(_, s)| s.kind == ObligationSeedKind::GeneratedNonEmptiness).count(), 3);
        let vcs = step5c3_registration_vcs(&core).unwrap();
        assert_eq!(vcs.vcs().len(), 6);
        assert!(vcs.vcs().iter().all(|v| v.status == VcStatus::Open));
        assert_eq!(vcs.seed_accounting().len(), 6);
        assert_eq!(vcs.seed_accounting().iter().filter(|r| matches!(&r.mapping,
            SeedVcMapping::Expanded { vcs, .. } if vcs.len() == 2 && vcs[0].expansion_index == 0 && vcs[1].expansion_index == 1)).count(), 3);
        let builtin_origins = vcs.seed_accounting().iter().filter_map(|r| match r.mapping {
            SeedVcMapping::NoConcreteVc { reason: SeedNoVcReason::BuiltinSetInhabitation { origin } } => Some(origin),
            _ => None,
        }).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(builtin_origins.len(), 3);
        let mut handoff_text = String::new();
        for (vc, substitution_count) in vcs.vcs().iter().zip([0, 1, 0, 1, 0, 2]) {
            let handoff = mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
            handoff_text.push_str(&handoff.debug_text());
            assert!(handoff.targets_vc(&vcs, vc.id).unwrap());
            assert_eq!(handoff.canonical_evidence().substitutions().len(), substitution_count);
            assert!(step5c3_check_registration_handoff(&handoff, "clean").unwrap().sat_check_report().is_some());
            for mutation in ["wire", "goal-bytes", "context", "provenance", "polarity", "resource"] {
                assert!(step5c3_check_registration_handoff(&handoff, mutation).is_err(), "{:?}: {mutation}", vc.id);
            }
            if substitution_count > 0 {
                for mutation in ["missing-substitution", "substitution-source", "substitution-actual"] {
                    assert!(step5c3_check_registration_handoff(&handoff, mutation).is_err(), "{:?}: {mutation}", vc.id);
                }
            }
            if substitution_count == 2 {
                for mutation in ["missing-second-substitution", "swapped-substitution-source"] {
                    assert!(step5c3_check_registration_handoff(&handoff, mutation).is_err(), "{mutation}");
                }
            }
        }
        if index == 0 {
            for (path, actual) in [
                ("core/step5c13_source_registration.core_ir.snap", core.debug_text()),
                ("vc/step5c13_source_registration.vc_ir.snap", vcs.debug_text()),
                ("vc/step5c13_source_registration.kernel_handoff.snap", handoff_text),
            ] {
                assert_eq!(actual, std::fs::read_to_string(step5c11_config().workspace_root.join("tests/snapshots").join(path)).unwrap(), "{path}");
            }
        }
        let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
            super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).unwrap();
        assert_eq!(database.activated().len(), 3);
        assert!(database.pending().is_empty() && database.rejected().is_empty());
        for validation in checked.validations() {
            let active = database.activated().iter().find(|entry| entry.pattern().as_str() == format!("{:?}", validation.pattern())).unwrap();
            assert_eq!(active.correctness().as_str(), validation.correctness_provenance().as_str());
            let source_entry = symbols.registrations().iter().find(|entry|
                entry.origin().anchor() == &nodes.node(validation.owner().node()).unwrap().anchor).unwrap();
            assert_eq!(active.source().origin(), source_entry.origin());
            assert!(active.fingerprint().is_some());
        }
        assert!(checked.database().activated().is_empty());
        assert_eq!(checked.database().pending().len(), 3);
        let policy = mizar_proof::policy::VerifierPolicy::release().with_kernel_evidence_formats([]);
        assert!(mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
            super::super::shared::snapshot_id(0), &policy).is_err());
    }
}

#[test]
fn step5c13_source_registration_rejects_partial_foreign_and_malformed_proofs() {
    use mizar_checker::registration_resolution::check_source_existential_registration_proof as check;
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    for (from, to) in [
        ("by OAADef, OBBDef;", "by OAADef;"),
        ("by OAADef, OBBDef;", "by OBBDef, OAADef;"),
        ("by OAADef, OBBDef;", "by OAADef, OAADef;"),
        ("by OAADef;", "by OBBDef;"),
        ("by OAADef;", "by Missing;"),
        ("thus the set is oamarked obmarked", "thus the set is oamarked"),
        ("thus the set is oamarked obmarked", "thus the set is oamarked set"),
        ("thus the set is oamarked obmarked", "thus the set is obmarked oamarked"),
        ("thus the set is oamarked obmarked", "thus the set is oamarked oamarked"),
        ("for X being oamarked obmarked set", "for X being oamarked obmarked"),
        ("cluster OABExists: oamarked obmarked set", "cluster OABExists: oamarked set"),
        ("take the set;", "take the oamarked set;"),
        ("holds ovpick X = X", "holds ovpick Absent = X"),
    ] {
        let changed = text.replacen(from, to, 1);
        assert_ne!(changed, text);
        assert!(super::super::formula_statement::step5c8_test_frontend(&changed).diagnostics.is_empty(), "semantic control: {from} -> {to}");
        let (source, nodes, symbols) = step5c13_registration_inputs(&changed).unwrap();
        assert!(check(&source, &nodes, &symbols).is_err(), "{from} -> {to}");
    }
    for changed in [text.replacen("take the set;", "take ;", 1), text.replacen("holds ovpick X = X", "holds missing X = X", 1)] {
        assert!(!super::super::formula_statement::step5c8_test_frontend(&changed).diagnostics.is_empty());
        assert!(step5c13_registration_inputs(&changed).is_err());
    }
    let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
    let other = text.replace("oamarked", "foreignmarked");
    let (foreign_source, foreign_nodes, foreign_symbols) = step5c13_registration_inputs(&other).unwrap();
    assert!(check(&source, &nodes, &foreign_symbols).is_err());
    assert!(check(&source, &foreign_nodes, &symbols).is_err());
    assert!(check(&foreign_source, &nodes, &symbols).is_err());
    for definition in ["oamarked", "obmarked"] {
        let changed = text.replace(&format!("X is {definition} means X = X"), &format!("X is {definition} means not X = X"));
        assert_ne!(changed, text);
        let (source, nodes, symbols) = step5c13_registration_inputs(&changed).unwrap();
        let pending = check(&source, &nodes, &symbols).unwrap();
        assert_eq!(pending.database().pending().len(), 3);
        assert!(mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
            super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).is_err());
        assert!(pending.database().activated().is_empty());
    }
}

#[test]
fn step5c13_source_registration_rejects_shared_conjunct_and_parent_corruption() {
    use mizar_core::core_ir::*;
    use mizar_vc::vc_ir::*;
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
    let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let parents = core.obligation_seeds().iter().filter(|(_, row)| row.kind == ObligationSeedKind::CheckerInitial).collect::<Vec<_>>();
    let definitions = core.definitions().iter().collect::<Vec<_>>();
    let origins = core.generated().iter().collect::<Vec<_>>();
    let CoreFormulaKind::Exists { body: union, .. } = core.formulas().get(parents[2].1.goal.unwrap()).unwrap().kind else { panic!("union") };
    let CoreFormulaKind::And(conjuncts) = &core.formulas().get(union).unwrap().kind else { panic!("conjuncts") };
    for mutation in 0..8 {
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

        match mutation {
            0 => parts.formulas.get_mut(union).unwrap().kind = CoreFormulaKind::And(vec![conjuncts[0], conjuncts[1], conjuncts[1]]),
            1 => parts.definitions.get_mut(definitions[1].0).unwrap().params[0].var = CoreVarId::new(999),
            2 => parts.generated.get_mut(origins[2].0).unwrap().functor = origins[0].1.functor.clone(),
            3 => parts.obligation_seeds.get_mut(parents[2].0).unwrap().core_refs.retain(|r| *r != CoreNodeRef::Definition(definitions[1].0)),
            4 => {
                for (_, node) in parts.proof_nodes.iter_mut() {
                    match &mut node.kind {
                        CoreProofNodeKind::Step { justification, .. } => justification.citations.reverse(),
                        CoreProofNodeKind::TerminalGoal { citations, .. } => citations.reverse(),
                        _ => (),
                    }
                }
            }
            5 => parts.obligation_seeds.get_mut(parents[2].0).unwrap().goal = parents[0].1.goal,
            6 => parts.definitions.get_mut(definitions[1].0).unwrap().source = definitions[0].1.source.clone(),
            7 => {
                let foreign = parents[0].1.core_refs.iter().find(|r| matches!(r, CoreNodeRef::ObligationSeed(_))).unwrap();
                let own = parts.obligation_seeds.get_mut(parents[2].0).unwrap().core_refs.iter_mut().find(|r| matches!(r, CoreNodeRef::ObligationSeed(_))).unwrap();
                *own = foreign.clone();
            }
            _ => unreachable!(),
        }
        parts.source_map.formula_sources = parts.formulas.iter().map(|(id, row)| (id, row.source.clone())).collect();
        parts.source_map.generated_sources = parts.generated.iter().map(|(id, row)| (id, row.source.clone())).collect();
        parts.source_map.obligation_sources = parts.obligation_seeds.iter().map(|(id, row)| (id, row.source.clone())).collect();
        parts.source_map.definition_sources = parts.definitions.iter().map(|(id, row)| (id, row.source.clone())).collect();
        let altered = CoreIr::try_new(parts).unwrap_or_else(|error| panic!("Core mutation {mutation} must remain structurally valid: {error}"));
        assert!(step5c3_registration_vcs(&altered).is_err(), "Core mutation {mutation}");
    }
    let vcs = step5c3_registration_vcs(&core).unwrap();
    for mutation in 0..4 {
        let mut parts = VcSetParts {
            schema_version: vcs.schema_version().clone(), snapshot: vcs.snapshot(), source: vcs.source(), module: vcs.module().clone(),
            generated_formulas: vcs.generated_formulas().to_vec(), vcs: vcs.vcs().to_vec(), seed_accounting: vcs.seed_accounting().to_vec(),
        };
        match mutation {
            0 => { parts.vcs[5].premises.pop(); }
            1 => parts.vcs[5].goal = parts.vcs[1].goal,
            2 => {
                let rows = parts.seed_accounting.iter_mut().filter(|row| matches!(row.mapping, SeedVcMapping::Expanded { .. })).collect::<Vec<_>>();
                let mut rows = rows.into_iter();
                let first = rows.next().unwrap().mapping.clone();
                rows.next_back().unwrap().mapping = first;
            }
            3 => parts.vcs[5].premises.swap(1, 2),
            _ => unreachable!(),
        }
        if mutation == 2 {
            let actual_handoff = parts.seed_accounting.iter().rfind(|row| matches!(row.mapping, SeedVcMapping::Expanded { .. })).unwrap().handoff;
            assert_eq!(VcSet::try_new(parts).unwrap_err(), VcIrError::VcMappedFromWrongSeed {
                vc: vcs.vcs()[0].id, expected_handoff: vcs.vcs()[0].seed.handoff, actual_handoff,
            });
            continue;
        }
        let altered = VcSet::try_new(parts).unwrap_or_else(|error| panic!("VC mutation {mutation} must remain structurally valid: {error}"));
        for vc in altered.vcs() {
            assert!(mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(&core, &altered, vc.id).is_err(), "VC mutation {mutation}");
        }
    }
}
