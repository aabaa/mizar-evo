use super::*;

#[test]
fn step5c5_predicate_arguments_preserve_order_bindings_polarity_and_real_rejections() {
    use super::super::type_elaboration::{
        step5c3_functor_argument_detail_keys, step5c5_predicate_argument_detail_keys,
    };
    use mizar_checker::overload_resolution::{
        CandidateDeclarationKind, CandidateRejectionReason, CandidateViabilityStatus,
        OverloadResultStatus, OverloadSiteKind,
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, TypeHeadRef, check_source_distinct_loci_overloads,
    };
    use mizar_resolve::{env::SymbolKind, names::resolve_template_formal};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_pred_argument_type_mismatch_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let valid = text
        .replace("X being set", "X being PredBox")
        .replace("X be set", "X be PredBox");
    let renamed = text
        .replace("PredBox", "Container")
        .replace("MatchesDef", "RelatesDef")
        .replace("BadMatch1", "UseRelates")
        .replace("matches", "relates")
        .replace("let P, Q", "let Left, Right")
        .replace("P relates Q", "Left relates Right")
        .replace("P.d", "Left.value")
        .replace("Q.d", "Right.value")
        .replace("field d", "field value")
        .replace("X", "Actual");
    let mut variants = Vec::new();
    for mask in 0..4 {
        let mut changed = text.clone();
        if mask & 1 != 0 {
            changed = changed.replace("X being set", "X being PredBox");
        }
        if mask & 2 != 0 {
            changed = changed.replace("X be set", "X be PredBox");
        }
        variants.push((
            changed,
            [
                if mask & 1 == 0 { vec![0, 1] } else { vec![] },
                if mask & 2 == 0 { vec![0, 1] } else { vec![] },
            ],
        ));
    }
    variants.extend([
        (renamed, [vec![0, 1], vec![0, 1]]),
        (
            valid.replace("X matches X", "X.d matches X"),
            [vec![0], vec![0]],
        ),
        (
            valid.replace("X matches X", "X matches X.d"),
            [vec![1], vec![1]],
        ),
        (
            text.replace("X matches X", "X does not matches X"),
            [vec![0, 1], vec![0, 1]],
        ),
        (
            valid.replace("X matches X", "X does not matches X"),
            [vec![], vec![]],
        ),
    ]);
    for (variant, rejected_indices) in variants {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        let outputs =
            check_source_distinct_loci_overloads(&source, &symbols, &typed, true, None).unwrap();
        assert_eq!(
            outputs,
            check_source_distinct_loci_overloads(&source, &symbols, &typed, true, None).unwrap()
        );
        assert!(check_source_distinct_loci_overloads(&source, &symbols, &typed, false, None).is_err());
        assert!(step5c3_functor_argument_detail_keys(&source, &symbols, &typed).is_err());
        let rejected_sites = rejected_indices
            .iter()
            .filter(|indices| !indices.is_empty())
            .count();
        assert_eq!(
            step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).unwrap(),
            if rejected_sites == 0 {
                vec![]
            } else {
                vec!["predicates.application.argument_type_mismatch".to_owned()]
            }
        );
        let (normalization, collection, expansion, viability, graphs, selection, _) = outputs;
        assert!(normalization.diagnostics().is_empty());
        assert!(collection.diagnostics().is_empty());
        assert!(expansion.diagnostics().is_empty());
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 2);
        assert_eq!(expansion.candidates().len(), 2);
        assert_eq!(viability.decisions().len(), 2);
        assert_eq!(viability.diagnostics().len(), rejected_sites);
        assert_eq!(graphs.graphs().len(), 2);
        assert_eq!(selection.results().len(), 2);
        assert!(selection.inserted_views().is_empty());
        let predicate = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Predicate)
            .unwrap();
        let structure = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Structure)
            .unwrap();
        let selector = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Selector)
            .unwrap();
        assert!(
            matches!(selector.origin().anchor(), SourceAnchor::Range(range) if variant[range.start..range.end].contains("field "))
        );
        let (_, pattern) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &K::PredicatePattern)
            .unwrap();
        let formals = [
            resolve_template_formal(&source, pattern.children()[0]).unwrap(),
            resolve_template_formal(&source, pattern.children()[2]).unwrap(),
        ];
        assert_ne!(formals[0], formals[1]);
        let renamed = variant.contains("let Left, Right");
        let formal_offsets = if renamed {
            [
                variant.find("let Left").unwrap() + 4,
                variant.find(", Right be").unwrap() + 2,
            ]
        } else {
            [
                variant.find("let P").unwrap() + 4,
                variant.find(", Q be").unwrap() + 2,
            ]
        };
        for (formal, offset) in formals.iter().zip(formal_offsets) {
            assert!(
                matches!(source.arena().node(*formal).unwrap().origin().anchor(), SourceAnchor::Range(range) if range.start == offset)
            );
        }
        let actual_name = if renamed { "Actual" } else { "X" };
        let binder_offsets = [
            variant.find(&format!("for {actual_name}")).unwrap() + 4,
            variant.find(&format!("let {actual_name}")).unwrap() + 4,
        ];
        let mut occurrences = Vec::new();
        let mut binders = Vec::new();
        for (index, (site_id, site)) in collection.sites().iter().enumerate() {
            assert_eq!(site.kind, OverloadSiteKind::PredicateApplication);
            let application = source
                .arena()
                .node(
                    typed
                        .node(site.owner.node())
                        .unwrap()
                        .resolved_node
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(application.kind(), &K::PredicateApplication);
            assert_eq!(
                SourceAnchor::Range(site.source_range),
                *application.origin().anchor()
            );
            if variant.contains(" not ") {
                assert!(variant[site.source_range.start..site.source_range.end].contains(" not "));
            }
            let segment = source.arena().node(application.children()[0]).unwrap();
            assert_eq!(segment.kind(), &K::PredicateSegment);
            let actual_slots = [segment.children()[0], *segment.children().last().unwrap()];
            assert_eq!(
                site.arguments
                    .iter()
                    .map(|site| site.node().index())
                    .collect::<Vec<_>>(),
                actual_slots.map(|id| source.arena().node(id).unwrap().children()[0].index())
            );
            for argument in &site.arguments {
                let node = source
                    .arena()
                    .node(typed.node(argument.node()).unwrap().resolved_node.unwrap())
                    .unwrap();
                let reference = if node.kind() == &K::SelectorAccess {
                    source.arena().node(node.children()[0]).unwrap()
                } else {
                    node
                };
                assert_eq!(reference.kind(), &K::TermReference);
                let binder = resolve_template_formal(&source, reference.children()[0]).unwrap();
                assert!(!formals.contains(&binder));
                assert!(
                    matches!(source.arena().node(binder).unwrap().origin().anchor(), SourceAnchor::Range(range) if range.start == binder_offsets[index])
                );
                binders.push(binder);
                occurrences.push(argument.clone());
            }
            let (_, candidate) = expansion
                .candidates()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(
                candidate.declaration_kind,
                CandidateDeclarationKind::Predicate
            );
            assert_eq!(&candidate.symbol, predicate.symbol());
            assert_eq!(candidate.ordinary_root, candidate.symbol);
            assert_eq!(
                SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                *predicate.origin().anchor()
            );
            assert!(candidate.result.is_none());
            assert_eq!(candidate.parameters.len(), 2);
            for parameter in &candidate.parameters {
                let parameter = normalization.normalized_types().get(*parameter).unwrap();
                assert_eq!(
                    parameter.head,
                    TypeHeadRef::Structure(structure.symbol().clone())
                );
                assert_eq!(parameter.status, NormalizedTypeStatus::Known);
            }
            let (_, decision) = viability
                .decisions()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(decision.source_candidate, candidate.id);
            let (_, result) = selection
                .results()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert!(result.diagnostics.is_empty());
            if rejected_indices[index].is_empty() {
                assert!(
                    matches!(&decision.status, CandidateViabilityStatus::Viable { views } if views.len() == 2 && views.iter().all(|view| view.actual == candidate.parameters[view.argument_index] && view.target == view.actual))
                );
                assert!(
                    matches!(&result.status, OverloadResultStatus::Resolved { exposed_result: None, refinements, inserted_views, .. } if refinements.is_empty() && inserted_views.is_empty())
                );
            } else {
                let CandidateViabilityStatus::Rejected { reasons } = &decision.status else {
                    panic!("{:?}", decision.status)
                };
                assert_eq!(
                    reasons
                        .iter()
                        .map(|reason| reason.argument_index)
                        .collect::<Vec<_>>(),
                    rejected_indices[index]
                );
                for reason in reasons {
                    assert_eq!(reason.reason, CandidateRejectionReason::MissingEvidence);
                    assert_eq!(
                        reason.target,
                        Some(candidate.parameters[reason.argument_index])
                    );
                    let actual = normalization
                        .normalized_types()
                        .get(reason.actual.unwrap())
                        .unwrap();
                    assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
                    assert_eq!(actual.status, NormalizedTypeStatus::Known);
                }
                assert_eq!(decision.diagnostics.len(), 1);
                assert!(decision.output_candidate.is_none());
                assert!(
                    matches!(&result.status, OverloadResultStatus::NoMatch { rejected } if rejected.is_empty())
                );
            }
        }
        for (index, occurrence) in occurrences.iter().enumerate() {
            assert!(!occurrences[..index].contains(occurrence));
        }
        assert_eq!(binders[0], binders[1]);
        assert_eq!(binders[2], binders[3]);
        assert_ne!(binders[0], binders[2]);
    }
}

#[test]
fn step5c5_predicate_argument_mismatch_rejects_other_errors_and_forged_inputs() {
    use super::super::type_elaboration::{
        step5c3_functor_argument_detail_keys, step5c5_predicate_argument_detail_keys,
    };
    use mizar_checker::typed_ast::{TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_pred_argument_type_mismatch_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let inputs = |text: &str| {
        let frontend = super::super::formula_statement::step5c8_test_frontend(text);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control must parse cleanly: {text}: {:?}",
            frontend.diagnostics
        );
        super::super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap()
    };
    for (from, to) in [
        ("P.d = Q.d", "P.d = Q"),
        ("P.d = Q.d", "X.d = Q.d"),
        ("P.d = Q.d", "P.missing = Q.d"),
        ("P.d = Q.d", "P.d = Q.missing"),
        ("P matches Q", "Q matches P"),
        ("P matches Q", "P matches P"),
        ("P, Q be PredBox", "P, Q be set"),
        ("field d -> set", "field d -> object"),
        ("holds X matches X", "holds P matches X"),
        ("thus X matches X", "thus X matches Q"),
        ("X matches X", "X matches X matches X"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        let (source, typed, symbols) = inputs(&changed);
        assert!(
            step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).is_err(),
            "{changed}"
        );
    }
    let widening = text
        .replace("P, Q be PredBox", "P, Q be object")
        .replace("P.d = Q.d", "P = Q");
    let (source, typed, symbols) = inputs(&widening);
    assert!(step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).is_err());
    let (source, typed, symbols) = inputs(&text);
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let argument_ids = typed.iter().filter(|(_, node)| node.kind.as_str() == "TermReference" && matches!(node.anchor, SourceAnchor::Range(range) if range.start > text.find("theorem").unwrap())).map(|(id, _)| id).collect::<Vec<_>>();
    assert_eq!(argument_ids.len(), 4);
    for argument in &argument_ids {
        for mutation in 0..3 {
            let mut nodes = raw.clone();
            let other = argument_ids
                .iter()
                .find(|id| *id != argument)
                .unwrap()
                .index();
            match mutation {
                0 => nodes[argument.index()].resolved_node = nodes[other].resolved_node,
                1 => nodes[argument.index()].anchor = nodes[other].anchor.clone(),
                2 => nodes[argument.index()].typing = TypingState::Successful,
                _ => unreachable!(),
            }
            assert!(
                step5c5_predicate_argument_detail_keys(
                    &source,
                    &symbols,
                    &TypedArena::try_new(typed.root(), nodes).unwrap()
                )
                .is_err()
            );
        }
    }
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let offsets = text
        .match_indices("X matches X")
        .map(|(offset, _)| offset + 2)
        .collect::<Vec<_>>();
    assert_eq!(offsets.len(), 2);
    for target in offsets.into_iter().map(Some).chain([None]) {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changed = 0;
        for node in ast.nodes() {
            let mut children = node
                .children
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect::<Vec<_>>();
            let id = match &node.kind {
                K::Token(token) => {
                    let spelling = if Some(node.range.start) == target {
                        assert_eq!(token.text.as_ref(), "matches");
                        changed += 1;
                        "unknown".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                kind => {
                    if target.is_none() && kind == &K::ItemList {
                        assert_eq!(children.len(), 3);
                        children.swap(0, 1);
                        changed += 1;
                    }
                    builder.add_node(kind.clone(), node.range, children)
                }
            };
            rebuilt.push(id);
        }
        assert_eq!(changed, 1);
        let mut frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        frontend.ast = Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (changed_source, changed_typed, changed_symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&changed_source, &changed_symbols)
            .unwrap();
        assert!(
            step5c5_predicate_argument_detail_keys(
                &changed_source,
                &changed_symbols,
                &changed_typed
            )
            .is_err()
        );
    }
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("predicate"));
    assert!(
        step5c5_predicate_argument_detail_keys(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    let mut foreign_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    foreign_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    assert!(
        step5c5_predicate_argument_detail_keys(
            &SurfaceResolvedArena::lower(&foreign_ast, symbols.module_id()).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    let (_, _, other_symbols) = inputs(&text.replace("PredBox", "OtherBox"));
    assert!(step5c5_predicate_argument_detail_keys(&source, &other_symbols, &typed).is_err());
    let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        step5c5_predicate_argument_detail_keys(
            &source,
            &SymbolEnv::new(symbols.module_id().clone(), indexes),
            &typed
        )
        .is_err()
    );
    let theorem = text.find("theorem BadMatch1:").unwrap();
    let duplicate = format!("{text}\n{}", &text[theorem..]);
    let frontend = super::super::formula_statement::step5c8_test_frontend(&duplicate);
    assert!(frontend.diagnostics.is_empty());
    let result = super::super::resolver_symbol_collection(
        &config.workspace_root,
        case,
        frontend.ast.as_ref().unwrap(),
    );
    assert!(!result.detail_keys.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let frontend = super::super::formula_statement::step5c8_test_frontend(
        &text.replace("thus X matches X;", "thus X does matches X;"),
    );
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let functor_case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_argument_type_mismatch_functor_001")
        .unwrap();
    let frontend = super::super::formula_statement::step5c8_test_frontend(
        &std::fs::read_to_string(&functor_case.source_path).unwrap(),
    );
    let (source, typed, symbols) =
        super::super::source_registration_inputs(&config.workspace_root, functor_case, frontend).unwrap();
    assert_eq!(
        step5c3_functor_argument_detail_keys(&source, &symbols, &typed).unwrap(),
        vec!["types.application.argument_type_mismatch"]
    );
    assert!(step5c5_predicate_argument_detail_keys(&source, &symbols, &typed).is_err());
}
#[test]
fn step5c5_functor_property_checks_body_bindings_and_retains_only_a_pending_obligation() {
    use mizar_checker::{
        binding_env::BindingKind,
        type_checker::{
            NormalizedTypeStatus, TermFormulaChecker, TermKind, TermReference, TermStatus,
            TypeHeadRef,
        },
        typed_ast::{InitialObligationKind, InitialObligationStatus, TypeEntryActual, TypeStatus},
    };
    let config = step5c11_config();
    let (case, text) = step5c5_functor_property_case();
    for (variant, body_order) in [
        (text.clone(), [0, 1]),
        (text.replace("PairJoinDef", "UnionRule"), [0, 1]),
        (text.replace("X", "Left").replace("Y", "Right"), [0, 1]),
        (text.replace(r"\*\", r"\+\"), [0, 1]),
        (text.replace("{X, Y}", "{Y, X}"), [1, 0]),
        (text.replace(r"X \*\ Y ->", r"Y \*\ X ->"), [0, 1]),
        (text.replace("{X, Y}", "{X, X}"), [0, 0]),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        let result =
            TermFormulaChecker::check_source_functor_property(&source, &symbols, &typed).unwrap();
        assert_eq!(
            result,
            TermFormulaChecker::check_source_functor_property(&source, &symbols, &typed).unwrap()
        );
        let (bindings, checked, obligations) = result;
        assert!(bindings.diagnostics().is_empty() && checked.diagnostics().is_empty());
        assert!(
            checked.formulas().is_empty()
                && checked.facts().is_empty()
                && checked.candidate_sets().is_empty()
        );
        let formals = bindings
            .bindings()
            .iter()
            .map(|(_, entry)| entry)
            .collect::<Vec<_>>();
        assert_eq!(formals.len(), 2);
        for formal in &formals {
            assert_eq!(formal.kind, BindingKind::DefinitionParameter);
            assert_eq!(
                &variant[formal.declaration_range.start..formal.declaration_range.end],
                formal.spelling
            );
            assert!(formal.declaration_range.end < variant.find("func ").unwrap());
        }
        assert_ne!(formals[0].id, formals[1].id);
        assert_ne!(formals[0].identity, formals[1].identity);
        let terms = checked
            .terms()
            .iter()
            .map(|(_, term)| term)
            .collect::<Vec<_>>();
        assert_eq!(terms.len(), 3);
        for term in &terms {
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty() && term.candidate_set.is_none());
            let TypeEntryActual::Known(id) =
                checked.type_entries().get(term.type_entry).unwrap().actual
            else {
                panic!("body type must be known")
            };
            assert_eq!(
                checked.type_entries().get(term.type_entry).unwrap().status,
                TypeStatus::Known
            );
            let normalized = checked.normalized_types().get(id).unwrap();
            assert_eq!(normalized.status, NormalizedTypeStatus::Known);
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        }
        let elements = terms
            .iter()
            .filter(|term| term.kind == TermKind::Variable)
            .collect::<Vec<_>>();
        assert_eq!(elements.len(), 2);
        assert_ne!(elements[0].site, elements[1].site);
        for (element, formal_index) in elements.iter().zip(body_order) {
            assert_eq!(
                element.reference,
                Some(TermReference::Binding(formals[formal_index].id))
            );
            let SourceAnchor::Range(range) = typed.node(element.site.node()).unwrap().anchor else {
                panic!("real body occurrence")
            };
            assert_eq!(
                &variant[range.start..range.end],
                formals[formal_index].spelling
            );
        }
        let body = terms
            .iter()
            .find(|term| term.kind == TermKind::SetEnumeration)
            .unwrap();
        assert!(body.reference.is_none());
        let body_type = checked.type_entries().get(body.type_entry).unwrap();
        let TypeEntryActual::Known(actual_type) = body_type.actual else {
            unreachable!()
        };
        assert_eq!(body_type.expected, Some(actual_type));
        assert_eq!(body.expected_type, Some(actual_type));
        let SourceAnchor::Range(body_range) = typed.node(body.site.node()).unwrap().anchor else {
            unreachable!()
        };
        assert!(variant[body_range.start..body_range.end].starts_with('{'));
        for element in elements {
            let SourceAnchor::Range(range) = typed.node(element.site.node()).unwrap().anchor else {
                unreachable!()
            };
            assert!(body_range.start < range.start && range.end < body_range.end);
        }
        assert_eq!(obligations.len(), 1);
        let (_, obligation) = obligations.iter().next().unwrap();
        assert_eq!(
            obligation.kind,
            InitialObligationKind::FunctorPropertyCorrectness
        );
        assert_eq!(obligation.status, InitialObligationStatus::Pending);
        assert!(obligation.assumptions.is_empty());
        let (definition_id, definition) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &mizar_syntax::SurfaceNodeKind::FunctorDefinition)
            .unwrap();
        let functor = symbols
            .symbols()
            .iter()
            .find(|entry| entry.origin().anchor() == definition.origin().anchor())
            .unwrap();
        let property_symbol = symbols
            .symbols()
            .iter()
            .find(|entry| entry.origin().anchor() == &SourceAnchor::Range(obligation.source_range))
            .unwrap();
        let pattern = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &mizar_syntax::SurfaceNodeKind::FunctorPattern)
            .unwrap()
            .1;
        let mut loci = Vec::new();
        for child in pattern.children() {
            let node = source.arena().node(*child).unwrap();
            if let mizar_syntax::SurfaceNodeKind::Token(token) = node.kind()
                && let Some(formal) = formals
                    .iter()
                    .find(|formal| formal.spelling == token.text.as_ref())
            {
                let declaration = source
                    .arena()
                    .iter()
                    .find(|(_, node)| {
                        node.origin().anchor() == &SourceAnchor::Range(formal.declaration_range)
                    })
                    .unwrap()
                    .0;
                loci.push(declaration.index());
            }
        }
        assert_eq!(loci.len(), 2);
        assert_ne!(loci[0], loci[1]);
        let return_type = definition
            .children()
            .iter()
            .find(|id| {
                source.arena().node(**id).unwrap().kind()
                    == &mizar_syntax::SurfaceNodeKind::TypeExpression
            })
            .unwrap();
        assert_eq!(
            obligation.goal.as_str(),
            format!(
                "source.functor.property.request:functor={}:property={}:loci={},{}:body={}:type={}",
                functor.symbol().fqn().as_str(),
                obligation.owner.node().index(),
                loci[0],
                loci[1],
                body.site.node().index(),
                return_type.index()
            )
        );
        assert_eq!(
            obligation.provenance.as_str(),
            format!(
                "source.functor.property:owner={}:node={}",
                property_symbol.symbol().fqn().as_str(),
                obligation.owner.node().index()
            )
        );
        assert_ne!(definition_id.index(), obligation.owner.node().index());
        let property = typed.node(obligation.owner.node()).unwrap();
        assert_eq!(property.kind.as_str(), "PropertyClause");
        assert_eq!(
            property.anchor,
            SourceAnchor::Range(obligation.source_range)
        );
        assert!(
            variant[obligation.source_range.start..obligation.source_range.end]
                .starts_with("commutativity")
        );
    }
}

#[test]
fn step5c5_functor_property_rejects_unsupported_bodies_and_foreign_owners() {
    use mizar_checker::type_checker::TermFormulaChecker;
    let config = step5c11_config();
    let (case, text) = step5c5_functor_property_case();
    for (before, after) in [
        (r#""\\*\\""#, r#""\\+\\""#),
        ("{X, Y}", "{Missing, Y}"),
        ("{X, Y}", "{X, Missing}"),
        ("{X, Y}", "[X, Y]"),
        ("{X, Y}", "{X}"),
        ("X \\*\\ Y ->", "X \\*\\ X ->"),
        ("-> set", "-> object"),
        ("X, Y be set", "X, Y be object"),
        ("commutativity", "idempotence"),
        (
            "commutativity\n  proof\n    thus thesis;\n  end;",
            "commutativity;",
        ),
        ("  commutativity\n  proof\n    thus thesis;\n  end;\n", ""),
    ] {
        let changed = text.replace(before, after);
        assert_ne!(changed, text);
        let frontend = super::super::formula_statement::step5c8_test_frontend(&changed);
        assert!(
            frontend.diagnostics.is_empty(),
            "{before} => {after}: {:?}",
            frontend.diagnostics
        );
        let outcome = super::super::source_registration_inputs(&config.workspace_root, &case, frontend)
            .and_then(|(source, typed, symbols)| {
                TermFormulaChecker::check_source_functor_property(&source, &symbols, &typed)
            });
        assert!(outcome.is_err(), "unsupported {before} => {after}");
    }
    let duplicate_binding = text.replace("let X, Y", "let X, X");
    let frontend = super::super::formula_statement::step5c8_test_frontend(&duplicate_binding);
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
    let duplicate = format!("{text}\n{}", text.split("infix_operator").next().unwrap());
    let frontend = super::super::formula_statement::step5c8_test_frontend(&duplicate);
    assert!(frontend.diagnostics.is_empty());
    let resolution = super::super::resolver_symbol_collection(
        &config.workspace_root,
        &case,
        frontend.ast.as_ref().unwrap(),
    );
    assert!(!resolution.detail_keys.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
    let malformed = text.replace("{X, Y}", "{X, Y");
    let frontend = super::super::formula_statement::step5c8_test_frontend(&malformed);
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
}

#[test]
fn step5c5_functor_property_rejects_forged_source_typed_and_symbol_inputs() {
    use mizar_checker::{
        type_checker::TermFormulaChecker,
        typed_ast::{TypedArena, TypingState},
    };
    use mizar_resolve::{env::SymbolEnv, resolved_ast::SurfaceResolvedArena};
    let config = step5c11_config();
    let (case, text) = step5c5_functor_property_case();
    let inputs = |text: &str| {
        let frontend = super::super::formula_statement::step5c8_test_frontend(text);
        assert!(frontend.diagnostics.is_empty());
        super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap()
    };
    let (source, typed, symbols) = inputs(&text);
    let (foreign_source, _, foreign_symbols) = inputs(&text.replace("PairJoinDef", "AnotherRule"));
    assert!(
        TermFormulaChecker::check_source_functor_property(&foreign_source, &symbols, &typed)
            .is_err()
    );
    assert!(
        TermFormulaChecker::check_source_functor_property(&source, &foreign_symbols, &typed)
            .is_err()
    );
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("property"));
    assert!(
        TermFormulaChecker::check_source_functor_property(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed
        )
        .is_err()
    );
    for mutation in 0..3 {
        let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
        match mutation {
            0 => indexes.definitions = Default::default(),
            1 => indexes.symbols = Default::default(),
            2 => indexes.contributions = Default::default(),
            _ => unreachable!(),
        }
        assert!(
            TermFormulaChecker::check_source_functor_property(
                &source,
                &SymbolEnv::new(symbols.module_id().clone(), indexes),
                &typed
            )
            .is_err()
        );
    }
    let original_nodes = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let targets = typed
        .iter()
        .filter(|(_, node)| {
            matches!(
                node.kind.as_str(),
                "TermReference" | "PropertyClause" | "FunctorPattern" | "OperatorDeclaration"
            )
        })
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 5);
    for target in targets {
        for mutation in 0..3 {
            let mut nodes = original_nodes.clone();
            match mutation {
                0 => nodes[target.index()].kind = "ForeignOwner".into(),
                1 => nodes[target.index()].resolved_node = None,
                2 => nodes[target.index()].typing = TypingState::Successful,
                _ => unreachable!(),
            }
            let forged = TypedArena::try_new(typed.root(), nodes).unwrap();
            assert!(
                TermFormulaChecker::check_source_functor_property(&source, &symbols, &forged)
                    .is_err(),
                "{target:?}/{mutation}"
            );
        }
    }
}

#[test]
fn step5c5_dependent_return_tracks_actual_substitution_body_and_pending_clauses() {
    use mizar_checker::{binding_env::BindingKind, type_checker::*, typed_ast::*};
    let (case, text) = step5c5_dependent_return_case();
    let (first, second) = text.split_once("\n\ndefinition\n").unwrap();
    for variant in [
        text.clone(),
        text.replace("MemberKind5Def", "ModeLabel"),
        text.replace("MemberKind5 of", "ResultKind of"),
        text.replace("IdemDef", "FunctorLabel"),
        text.replace("idembox", "copybox"),
        format!("{}\n\ndefinition\n{second}", first.replace("X", "Formal")),
        format!("{first}\n\ndefinition\n{}", second.replace("X", "Actual")),
        text.replace("it = X", "it = it"),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{:?}",
            frontend.diagnostics
        );
        let (source, nodes, symbols) =
            super::super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
                .unwrap();
        let output =
            TermFormulaChecker::check_source_dependent_functor_types(&source, &symbols, &nodes)
                .unwrap();
        assert_eq!(
            output,
            TermFormulaChecker::check_source_dependent_functor_types(&source, &symbols, &nodes)
                .unwrap()
        );
        let (bindings, inference, requests) = output;
        let formals = bindings
            .bindings()
            .iter()
            .map(|(_, entry)| entry)
            .collect::<Vec<_>>();
        assert_eq!(formals.len(), 2);
        assert_ne!(formals[0].identity, formals[1].identity);
        assert_ne!(formals[0].owner_context, formals[1].owner_context);
        for formal in &formals {
            assert_eq!(formal.kind, BindingKind::DefinitionParameter);
            assert_eq!(
                &variant[formal.declaration_range.start..formal.declaration_range.end],
                formal.spelling
            );
        }
        assert!(
            inference.diagnostics().is_empty()
                && inference.facts().is_empty()
                && inference.candidate_sets().is_empty()
        );
        assert_eq!(inference.terms().len(), 3);
        assert_eq!(inference.formulas().len(), 1);
        let terms = inference
            .terms()
            .iter()
            .map(|(_, term)| term)
            .collect::<Vec<_>>();
        let (_, formula) = inference.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_ne!(formula.terms[0], formula.terms[1]);
        let left = terms
            .iter()
            .find(|term| term.site == formula.terms[0])
            .unwrap();
        let right = terms
            .iter()
            .find(|term| term.site == formula.terms[1])
            .unwrap();
        let argument = terms
            .iter()
            .find(|term| !formula.terms.contains(&term.site))
            .unwrap();
        assert_eq!(left.kind, TermKind::It);
        assert_eq!(
            argument.reference,
            Some(TermReference::Binding(formals[1].id))
        );
        let repeated_it = variant.contains("it = it");
        assert_eq!(
            right.kind,
            if repeated_it {
                TermKind::It
            } else {
                TermKind::Variable
            }
        );
        assert_eq!(
            right.reference,
            (!repeated_it).then_some(TermReference::Binding(formals[1].id))
        );
        for term in &terms {
            assert_eq!(term.context, formals[1].owner_context);
            assert_eq!(term.status, TermStatus::Inferred);
            let entry = inference.type_entries().get(term.type_entry).unwrap();
            let TypeEntryActual::Known(actual) = entry.actual else {
                panic!("known")
            };
            assert_eq!(entry.status, TypeStatus::Known);
            let ty = inference.normalized_types().get(actual).unwrap();
            assert_eq!(ty.status, NormalizedTypeStatus::Known);
            assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
            assert!(
                ty.args.is_empty()
                    && ty.attributes.positive().is_empty()
                    && ty.attributes.negative().is_empty()
            );
            if term.site == argument.site {
                assert_eq!(entry.expected, Some(actual));
            }
            let node = nodes.node(term.site.node()).unwrap();
            assert_eq!(
                node.kind.as_str(),
                if term.kind == TermKind::It {
                    "ItTerm"
                } else {
                    "TermReference"
                }
            );
        }
        assert_eq!(requests.len(), 2);
        for ((_, request), (spelling, kind)) in requests.iter().zip([
            ("existence", InitialObligationKind::FunctorExistence),
            ("uniqueness", InitialObligationKind::FunctorUniqueness),
        ]) {
            assert_eq!(request.status, InitialObligationStatus::Pending);
            assert_eq!(request.kind, kind);
            assert!(request.assumptions.is_empty());
            assert_eq!(
                &variant[request.source_range.start..request.source_range.end],
                format!("{spelling};")
            );
            assert_eq!(
                nodes.node(request.owner.node()).unwrap().anchor,
                SourceAnchor::Range(request.source_range)
            );
            assert!(request.goal.as_str().contains(&format!(
                "substitution={}->{}",
                formals[0].id.index(),
                formals[1].id.index()
            )));
            assert!(
                request
                    .goal
                    .as_str()
                    .contains(&format!("body={}", formula.site.node().index()))
            );
            assert!(
                request
                    .provenance
                    .as_str()
                    .contains(&format!("kind={spelling}"))
            );
        }
        assert!(
            TypedAst::try_new(TypedAstParts {
                source_id: source.source_id(),
                module_id: source.module().clone(),
                resolved_root: Some(source.arena().root()),
                source_context: None,
                source_type: None,
                source_attribute: None,
                nodes,
                contexts: LocalTypeContextTable::new(),
                types: TypeTable::new(),
                facts: TypeFactTable::new(),
                coercions: CoercionTable::new(),
                initial_obligations: requests,
                diagnostics: TypeDiagnosticTable::new()
            })
            .is_err(),
            "generic installation must retain its functor-family boundary"
        );
    }
}

#[test]
fn step5c5_dependent_return_rejects_guard_and_unsupported_source_shapes() {
    use mizar_checker::type_checker::*;
    let (case, text) = step5c5_dependent_return_case();
    let (first, second) = text.split_once("\n\ndefinition\n").unwrap();
    let object = format!(
        "{first}\n\ndefinition\n{}",
        second.replace("let X be set", "let X be object")
    );
    let frontend = super::super::formula_statement::step5c8_test_frontend(&object);
    assert!(
        frontend.diagnostics.is_empty(),
        "object is a well-formed guard mismatch"
    );
    let (source, nodes, symbols) =
        super::super::source_registration_inputs(&step5c11_config().workspace_root, &case, frontend)
            .unwrap();
    let object_token=source.arena().iter().find(|(_,node)|matches!(node.kind(),mizar_syntax::SurfaceNodeKind::Token(token) if token.text.as_ref()=="object")).unwrap();
    let SourceAnchor::Range(range) = object_token.1.origin().anchor() else {
        panic!("source range")
    };
    let known = TypeNormalizer::default().normalize(
        &symbols,
        [TypeExpressionInput::new(
            mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(object_token.0.index()),
            ),
            *range,
            "object",
            TypeHeadInput::BuiltinObject,
        )],
    );
    assert!(known.diagnostics().is_empty());
    assert!(
        known
            .normalized_types()
            .iter()
            .all(|(_, ty)| ty.status == NormalizedTypeStatus::Known
                && ty.head == TypeHeadRef::BuiltinObject)
    );
    assert!(
        TermFormulaChecker::check_source_dependent_functor_types(&source, &symbols, &nodes)
            .is_err()
    );
    for (from, to) in [
        ("-> MemberKind5 of X", "-> Missing of X"),
        ("-> MemberKind5 of X", "-> set"),
        ("-> MemberKind5 of X", "-> MemberKind5"),
        ("-> MemberKind5 of X", "-> MemberKind5 of X, X"),
        ("-> MemberKind5 of X", "-> MemberKind5 of Missing"),
        ("it = X", "it = Missing"),
        ("it = X", "X = X"),
        ("it = X", "it = the set"),
        ("idembox X ->", "idembox Missing ->"),
        ("MemberKind5 of X is set", "MemberKind5 of Missing is set"),
        ("MemberKind5 of X is set", "MemberKind5 of X is object"),
        ("means it = X", "equals X"),
        ("  existence;\n", ""),
        ("  uniqueness;\n", ""),
        ("existence;\n  uniqueness;", "uniqueness;\n  existence;"),
        ("uniqueness;", "uniqueness; uniqueness;"),
        ("existence;", "existence proof thus thesis; end;"),
        ("func IdemDef:", "assume X = X; func IdemDef:"),
        ("it = X", "it ="),
        ("let X be set;\n  func", "let X,Y be set;\n  func"),
    ] {
        let changed = text.replacen(from, to, 1);
        assert_ne!(changed, text, "missing mutation {from}");
        let result = super::super::source_registration_inputs(
            &step5c11_config().workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(&changed),
        )
        .and_then(|(source, nodes, symbols)| {
            TermFormulaChecker::check_source_dependent_functor_types(&source, &symbols, &nodes)
        });
        assert!(result.is_err(), "accepted {from} -> {to}");
    }
    for changed in [
        format!("definition\n{second}\n{first}"),
        format!("{text}\n{text}"),
    ] {
        let result = super::super::source_registration_inputs(
            &step5c11_config().workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(&changed),
        )
        .and_then(|(source, nodes, symbols)| {
            TermFormulaChecker::check_source_dependent_functor_types(&source, &symbols, &nodes)
        });
        assert!(result.is_err(), "forward mode or extra definitions");
    }
}

#[test]
fn step5c5_dependent_return_rejects_foreign_source_environment_and_neutral_nodes() {
    use mizar_checker::{type_checker::TermFormulaChecker, typed_ast::*};
    use mizar_resolve::{env::SymbolEnv, resolved_ast::SurfaceResolvedArena};
    let (case, text) = step5c5_dependent_return_case();
    let inputs = |text: &str| {
        super::super::source_registration_inputs(
            &step5c11_config().workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(text),
        )
        .unwrap()
    };
    let (source, nodes, symbols) = inputs(&text);
    let (other, _, foreign) = inputs(&text.replace("IdemDef", "ForeignLabel"));
    assert!(
        TermFormulaChecker::check_source_dependent_functor_types(&other, &symbols, &nodes).is_err()
    );
    assert!(
        TermFormulaChecker::check_source_dependent_functor_types(&source, &foreign, &nodes)
            .is_err()
    );
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let module = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("dependent"));
    assert!(
        TermFormulaChecker::check_source_dependent_functor_types(
            &SurfaceResolvedArena::lower(&ast, &module).unwrap(),
            &symbols,
            &nodes
        )
        .is_err()
    );
    for mutation in 0..3 {
        let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
        match mutation {
            0 => indexes.definitions = Default::default(),
            1 => indexes.symbols = Default::default(),
            _ => indexes.contributions = Default::default(),
        }
        assert!(
            TermFormulaChecker::check_source_dependent_functor_types(
                &source,
                &SymbolEnv::new(symbols.module_id().clone(), indexes),
                &nodes
            )
            .is_err()
        );
    }
    for (target, _) in nodes.iter().filter(|(_, node)| {
        matches!(
            node.kind.as_str(),
            "ItTerm" | "TypeArguments" | "FunctorPattern" | "CorrectnessCondition"
        )
    }) {
        for mutation in 0..5 {
            let mut raw = nodes
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            match mutation {
                0 => raw[target.index()].kind = "ForeignOwner".into(),
                1 => raw[target.index()].recovery = NodeRecoveryState::Recovered,
                2 => raw[target.index()].children.clear(),
                3 => raw[target.index()].resolved_node = None,
                _ => raw[target.index()].anchor = raw[nodes.root().unwrap().index()].anchor.clone(),
            }
            let changed = TypedArena::try_new(nodes.root(), raw).unwrap();
            assert!(
                TermFormulaChecker::check_source_dependent_functor_types(
                    &source, &symbols, &changed
                )
                .is_err(),
                "{target:?}/{mutation}"
            );
        }
    }
}

#[test]
fn step5c5_narrower_registration_retains_real_choice_nonempty_and_kernel_leaves() {
    use mizar_checker::registration_resolution::*;
    use mizar_core::core_ir::*;
    use mizar_vc::vc_ir::*;
    let text = include_str!("../../../../../tests/miz/pass/predicates/pass_type_elaboration_pred_redefine_narrower_loci_001.miz");
    let (source, nodes, symbols) = step5c5_narrower_inputs(text).unwrap();
    let checked = check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
    assert_eq!(checked.validations().len(), 1);
    assert_eq!(checked.database().pending().len(), 1);
    assert!(checked.database().activated().is_empty());
    let choices = checked.choice_terms().unwrap();
    assert_eq!(choices.terms().len(), 2);
    assert_eq!(choices.type_sites().len(), 2);
    for ((_, choice), gate) in choices.terms().iter().zip(checked.choice_gates().unwrap().iter()) {
        assert_eq!(gate.owner(), choice.site());
        assert_eq!(gate.source_range(), choice.source_range());
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(gate.base_evidence_kind(), Some(ExistentialGateBaseEvidenceKind::BuiltinSet));
        assert!(gate.registration().is_none() && gate.attributes().is_empty());
    }
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    assert_eq!(core.definitions().len(), 1);
    assert_eq!(core.proofs().len(), 1);
    assert_eq!(core.generated().len(), 1);
    assert_eq!(core.obligation_seeds().len(), 2);
    let witnesses = core.terms().iter().filter(|(_, t)| matches!(t.kind, CoreTermKind::Apply { .. })).collect::<Vec<_>>();
    assert_eq!(witnesses.len(), 2);
    assert_eq!(witnesses[0].1.kind, witnesses[1].1.kind);
    assert_ne!(witnesses[0].1.source, witnesses[1].1.source);
    let (_, nonempty) = core.obligation_seeds().iter().find(|(_, seed)| seed.kind == ObligationSeedKind::GeneratedNonEmptiness).unwrap();
    assert_eq!(nonempty.status, ObligationSeedStatus::Active);
    assert!(nonempty.context.is_empty());
    assert!(matches!(core.formulas().get(nonempty.goal.unwrap()).unwrap().kind, CoreFormulaKind::Exists { .. }));
    let vcs = step5c3_registration_vcs(&core).unwrap();
    assert_eq!(vcs.vcs().len(), 2);
    assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open));
    assert_eq!(vcs.seed_accounting().len(), 2);
    assert!(vcs.seed_accounting().iter().any(|row| matches!(row.mapping,
        SeedVcMapping::NoConcreteVc { reason: SeedNoVcReason::BuiltinSetInhabitation { .. } })));
    assert!(vcs.seed_accounting().iter().any(|row| matches!(&row.mapping,
        SeedVcMapping::Expanded { vcs, .. } if vcs.len() == 2 && vcs[0].expansion_index == 0 && vcs[1].expansion_index == 1)));
    for (index, vc) in vcs.vcs().iter().enumerate() {
        let handoff = mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
        assert!(handoff.targets_vc(&vcs, vc.id).unwrap());
        assert_eq!(handoff.canonical_evidence().substitutions().len(), index);
        assert!(step5c3_check_registration_handoff(&handoff, "clean").unwrap().sat_check_report().is_some());
        for mutation in ["wire", "goal-bytes", "context", "provenance", "polarity", "resource"] {
            assert!(step5c3_check_registration_handoff(&handoff, mutation).is_err(), "leaf {index}: {mutation}");
        }
        if index == 1 {
            for mutation in ["missing-substitution", "substitution-source", "substitution-actual"] {
                assert!(step5c3_check_registration_handoff(&handoff, mutation).is_err(), "{mutation}");
            }
        }
    }
    let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
        super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).unwrap();
    assert_eq!(database.activated().len(), 1);
    assert!(database.pending().is_empty() && database.rejected().is_empty());
    let active = database.activated().iter().next().unwrap();
    let validation = &checked.validations()[0];
    assert_eq!(active.pattern().as_str(), format!("{:?}", validation.pattern()));
    assert_eq!(active.correctness().as_str(), validation.correctness_provenance().as_str());
    let registration = symbols.registrations().iter().find(|entry|
        entry.origin().anchor() == &nodes.node(validation.owner().node()).unwrap().anchor).unwrap();
    assert_eq!(active.source().origin(), registration.origin());
    assert!(active.fingerprint().is_some());
    mizar_checker::type_checker::check_source_predicate_redefinition_types(&source, &nodes, &symbols, &database).unwrap();
    assert!(mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
        super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release().with_kernel_evidence_formats([])).is_err());
    let false_body = text.replacen("means X = X", "means not X = X", 1);
    let (source, nodes, symbols) = step5c5_narrower_inputs(&false_body).unwrap();
    let checked = check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let vcs = step5c3_registration_vcs(&core).unwrap();
    for (index, vc) in vcs.vcs().iter().enumerate() {
        let handoff = mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
        assert_eq!(step5c3_check_registration_handoff(&handoff, "clean").is_ok(), index == 0);
    }
    assert!(mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
        super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).is_err());
}

#[test]
fn step5c5_narrower_rejects_missing_support_roots_and_malformed_coherence() {
    use mizar_checker::{registration_resolution::check_source_existential_registration_proof as seal,
        type_checker::check_source_predicate_redefinition_types as check, typed_ast::*};
    let text = include_str!("../../../../../tests/miz/pass/predicates/pass_type_elaboration_pred_redefine_narrower_loci_001.miz");
    for (from, to) in [
        ("by N2Def;", ";"), ("by N2Def;", "by EqvDef;"),
        ("take the set;", "take the n2marked set;"),
        ("let X, Y be n2marked set;", "let X, Y be set;"),
        ("pred EqvDef: X eqv Y", "pred EqvDef: eqv X"),
        ("means Y = X;", "means Missing = X;"),
        ("  coherence\n  proof\n    thus thesis;\n  end;", ""),
        ("  coherence\n", "  coherence;\n  coherence\n"),
        ("thus thesis;", "thus X = X;"),
        ("coherence\n", "coherence with N2Def\n"),
    ] {
        let changed = text.replacen(from, to, 1);
        assert_ne!(changed, text);
        let result = step5c5_narrower_inputs(&changed).and_then(|(source, nodes, symbols)| {
            let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
                super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release())?;
            check(&source, &nodes, &symbols, &database)
        });
        assert!(result.is_err(), "{from} -> {to}");
    }
    let blocks = text.split("\n\n").collect::<Vec<_>>();
    assert_eq!(blocks.len(), 4);
    let changed = [blocks[1], blocks[0], blocks[2], blocks[3]].join("\n\n");
    assert!(step5c5_narrower_inputs(&changed).and_then(|(source, nodes, symbols)| {
        let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
            super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release())?;
        check(&source, &nodes, &symbols, &database)
    }).is_err());
    let duplicate_root = format!("{}\n\n{}\n\n{}\n\n{}\n\n{}", blocks[0], blocks[1], blocks[2], blocks[2].replace("EqvDef", "OtherDef"), blocks[3]);
    assert!(step5c5_narrower_inputs(&duplicate_root).is_err(), "duplicate ordinary signatures reject during resolution before target inference");
    let (source, nodes, symbols) = step5c5_narrower_inputs(text).unwrap();
    let pending = seal(&source, &nodes, &symbols).unwrap();
    assert!(check(&source, &nodes, &symbols, pending.database()).is_err());
    let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
        super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).unwrap();
    let (foreign_source, foreign_nodes, foreign_symbols) = step5c5_narrower_inputs(&text.replace("n2marked", "foreignmarked")).unwrap();
    let foreign_database = mizar_proof::status::prove_source_existential_registration(&foreign_source, &foreign_nodes, &foreign_symbols,
        super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).unwrap();
    assert!(check(&source, &nodes, &symbols, &foreign_database).is_err());
    assert!(check(&source, &nodes, &foreign_symbols, &database).is_err());
    assert!(check(&source, &foreign_nodes, &symbols, &database).is_err());
    assert!(check(&foreign_source, &nodes, &symbols, &database).is_err());
    for kind in ["TakeStatement", "TypeExpression", "BuiltinPredicateApplication", "CoherenceCondition"] {
        {
            let (target, _) = nodes.iter().find(|(_, node)| node.kind.as_str() == kind).unwrap();
            let mut raw = nodes.iter().map(|(_, node)| node.clone()).collect::<Vec<_>>();
            raw[target.index()].recovery = NodeRecoveryState::Recovered;
            let altered = TypedArena::try_new(nodes.root(), raw).unwrap();
            assert!(check(&source, &altered, &symbols, &database).is_err(), "{kind}");
        }
    }
}

#[test]
fn step5c5_narrower_preserves_scoped_operands_shared_type_and_pending_schema() {
    use mizar_checker::{binding_env::BindingKind, registration_resolution::ExistentialGateStatus,
        type_checker::{check_source_predicate_redefinition_types as check, NormalizedTypeStatus}, typed_ast::*};
    let text = include_str!("../../../../../tests/miz/pass/predicates/pass_type_elaboration_pred_redefine_narrower_loci_001.miz");
    let blocks = text.split("\n\n").collect::<Vec<_>>();
    let mut variants = vec![text.to_owned(), text.replace("N2Def", "AttributeLabel")
        .replace("N2MarkedExists", "InhabitationLabel").replace("Eqv2Def", "NewLabel")
        .replace("EqvDef", "RootLabel").replace("n2marked", "tagged").replace("eqv", "related")];
    for index in [0, 2, 3] {
        let mut renamed = blocks.iter().map(|block| (*block).to_owned()).collect::<Vec<_>>();
        renamed[index] = renamed[index].replace("X", "Left").replace("Y", "Right");
        variants.push(renamed.join("\n\n"));
    }
    variants.push(text.replacen("means X = Y;", "means Y = X;", 1));
    variants.push(text.replacen("means Y = X;", "means X = Y;", 1));
    for variant in variants {
        let (source, nodes, symbols) = step5c5_narrower_inputs(&variant).unwrap();
        let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
            super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).unwrap();
        let (bindings, inference, gates, typed) = check(&source, &nodes, &symbols, &database).unwrap();
        let formals = bindings.bindings().iter().map(|(_, entry)| entry).collect::<Vec<_>>();
        assert_eq!(formals.len(), 4);
        for (index, formal) in formals.iter().enumerate() {
            assert!(formals[index + 1..].iter().all(|other| formal.identity != other.identity));
        }
        assert_eq!(formals[0].type_site, formals[1].type_site);
        assert_eq!(formals[2].type_site, formals[3].type_site);
        assert_ne!(formals[0].owner_context, formals[2].owner_context);
        for entry in &formals {
            assert_eq!(entry.kind, BindingKind::DefinitionParameter);
            assert_eq!(&variant[entry.declaration_range.start..entry.declaration_range.end], entry.spelling);
        }
        let gate = gates.iter().next().unwrap();
        assert_eq!(gates.iter().count(), 1);
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        let active = database.activated().iter().next().unwrap();
        assert_eq!(gate.registration(), Some(active.id()));
        assert_eq!(gate.pattern(), active.pattern());
        assert_eq!(gate.attributes().len(), 1);
        assert!(gate.diagnostics().is_empty());
        assert_eq!(inference.terms().len(), 4);
        assert_eq!(inference.formulas().len(), 2);
        assert!(inference.diagnostics().is_empty() && inference.candidate_sets().is_empty());
        for (index, (_, formula)) in inference.formulas().iter().enumerate() {
            assert_eq!(formula.kind, FormulaKind::Equality);
            assert_eq!(formula.status, FormulaStatus::Checked);
            assert_eq!(formula.terms.len(), 2);
            for site in &formula.terms {
                let term = inference.terms().iter().find_map(|(_, row)| (&row.site == site).then_some(row)).unwrap();
                let TermReference::Binding(binding) = term.reference.as_ref().unwrap() else { panic!("bound variable") };
                let formal = formals[index * 2..index * 2 + 2].iter().find(|formal| formal.id == *binding).unwrap();
                assert_eq!(term.context, formal.owner_context);
                assert_eq!(term.kind, TermKind::Variable);
                assert_eq!(term.status, TermStatus::Inferred);
                let node = nodes.node(site.node()).unwrap();
                let SourceAnchor::Range(range) = node.anchor else { panic!("source range") };
                assert_eq!(&variant[range.start..range.end], formal.spelling);
                let entry = inference.type_entries().get(term.type_entry).unwrap();
                let TypeEntryActual::Known(actual) = entry.actual else { panic!("known type") };
                let ty = inference.normalized_types().get(actual).unwrap();
                assert_eq!(ty.status, NormalizedTypeStatus::Known);
                assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
                assert!(ty.args.is_empty() && ty.attributes.negative().is_empty());
                assert_eq!(ty.attributes.positive().len(), usize::from(index == 1));
            }
        }
        assert!(typed.source_term().is_some() && typed.source_atomic_formula().is_some());
        assert!(typed.source_predicate_definition().is_none());
        assert_eq!(typed.initial_obligations().len(), 1);
        let (_, request) = typed.initial_obligations().iter().next().unwrap();
        assert_eq!(request.kind, InitialObligationKind::PredicateRedefinitionCoherence);
        assert_eq!(request.status, InitialObligationStatus::Pending);
        assert!(request.assumptions.is_empty());
        assert!(variant[request.source_range.start..request.source_range.end].starts_with("coherence"));
        let substitution = [(formals[0].id, formals[2].id), (formals[1].id, formals[3].id)];
        assert!(request.goal.as_str().contains(&format!("substitution={substitution:?}")));
        for (index, (_, formula)) in inference.formulas().iter().enumerate() {
            let operands = formula.terms.iter().map(|site| {
                let term = inference.terms().iter().find_map(|(_, row)| (&row.site == site).then_some(row)).unwrap();
                let Some(TermReference::Binding(binding)) = term.reference else { panic!("binding") };
                binding
            }).collect::<Vec<_>>();
            let key = if index == 0 { "original-body" } else { "new-body" };
            assert!(request.goal.as_str().contains(&format!("{key}={index}:{operands:?}")));
        }
        let (_, narrow_term) = inference.terms().iter().nth(2).unwrap();
        let TypeEntryActual::Known(narrow_type) = inference.type_entries().get(narrow_term.type_entry).unwrap().actual else { panic!("known") };
        let narrow_type = inference.normalized_types().get(narrow_type).unwrap();
        let closure = narrow_type.attributes.positive().iter().map(|attribute| attribute.symbol.clone()).collect::<std::collections::BTreeSet<_>>();
        let guards = [(formals[2].id, &narrow_type.head, &closure),
            (formals[3].id, &narrow_type.head, &closure)];
        assert!(request.goal.as_str().contains(&format!("guards={guards:?}")));
        assert_eq!(typed.source_term().unwrap().terms().len(), 4);
        assert_eq!(typed.source_atomic_formula().unwrap().formulas().len(), 2);
    }
}

#[test]
fn step5c5_narrower_rejects_coherent_token_kind_forgery() {
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let text = include_str!("../../../../../tests/miz/pass/predicates/pass_type_elaboration_pred_redefine_narrower_loci_001.miz");
    let config = step5c11_config();
    let case = build_test_plan(&config).unwrap().cases.into_iter()
        .find(|case| case.id.0 == "pass_type_elaboration_pred_redefine_narrower_loci_001").unwrap();
    let ast = super::super::formula_statement::step5c8_test_frontend(text).ast.unwrap();
    for target in [text.rfind("thus"), text.rfind("proof"), text.rfind("set;"), text.rfind("thesis"), None] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changed = 0;
        for node in ast.nodes() {
            let children = node.children.iter().map(|id| rebuilt[id.index()]).collect();
            let id = match &node.kind {
                K::Token(token) => {
                    let kind = if Some(node.range.start) == target {
                        changed += 1;
                        assert_ne!(token.kind, mizar_syntax::SurfaceTokenKind::Identifier);
                        mizar_syntax::SurfaceTokenKind::Identifier
                    } else { token.kind };
                    builder.add_token(kind, token.text.clone(), node.range)
                }
                K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Thesis) if target.is_none() => {
                    changed += 1;
                    builder.add_node(K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Contradiction), node.range, children)
                }
                kind => builder.add_node(kind.clone(), node.range, children),
            };
            rebuilt.push(id);
        }
        assert_eq!(changed, 1);
        let mut frontend = super::super::formula_statement::step5c8_test_frontend(text);
        frontend.ast = Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (source, typed, symbols) = super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&source, &symbols).unwrap();
        assert!(mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &typed, &symbols).is_err(), "forged source node at {target:?}");
    }
}

#[test]
fn step5c5_narrower_rejects_equal_attributed_signatures_and_typed_graph_mutations() {
    use mizar_checker::{type_checker::check_source_predicate_redefinition_types as check, typed_ast::*};
    let text = include_str!("../../../../../tests/miz/pass/predicates/pass_type_elaboration_pred_redefine_narrower_loci_001.miz");
    let blocks = text.split("\n\n").collect::<Vec<_>>();
    for changed in [
        text.replacen("let X, Y be set;", "let X, Y be n2marked set;", 1),
        text.replacen("pred EqvDef: X eqv Y", "pred EqvDef: X other Y", 1),
        [blocks[0], blocks[1], blocks[3], blocks[2]].join("\n\n"),
    ] {
        let (source, nodes, symbols) = step5c5_narrower_inputs(&changed).unwrap();
        let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
            super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).unwrap();
        assert!(check(&source, &nodes, &symbols, &database).is_err(), "target inference rejects equal signatures, wrong head, or later root after genuine proof");
    }

    let (source, nodes, symbols) = step5c5_narrower_inputs(text).unwrap();
    let database = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
        super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release()).unwrap();
    let (_, inference, _, typed) = check(&source, &nodes, &symbols, &database).unwrap();
    let (_, request) = typed.initial_obligations().iter().next().unwrap();
    let (_, body) = inference.formulas().iter().next().unwrap();
    let SourceAnchor::Range(body_range) = typed.nodes().node(body.site.node()).unwrap().anchor else { panic!("body range") };
    for mutation in 0..3 {
        let mut obligations = InitialObligationTable::new();
        obligations.insert(InitialObligationDraft {
            kind: request.kind,
            owner: if mutation == 1 { body.site.clone() } else { request.owner.clone() },
            source_range: if mutation == 2 { body_range } else { request.source_range },
            assumptions: request.assumptions.clone(), goal: request.goal.clone(),
            provenance: request.provenance.clone(), status: request.status,
        });
        let reconstructed = TypedAst::try_new(TypedAstParts {
            source_id: source.source_id(), module_id: source.module().clone(), resolved_root: typed.resolved_root(),
            source_context: None, source_type: None, source_attribute: None, nodes: typed.nodes().clone(),
            contexts: typed.contexts().clone(), types: typed.types().clone(), facts: typed.facts().clone(),
            coercions: typed.coercions().clone(), initial_obligations: obligations, diagnostics: typed.diagnostics().clone(),
        });
        assert_eq!(reconstructed.is_ok(), mutation == 0, "coherence obligation owner/range mutation {mutation}");
    }
    let primary = typed.source_term().unwrap();
    let atomic = typed.source_atomic_formula().unwrap();
    for formula in [false, true] {
        let sites = if formula {
            inference.formulas().iter().map(|(_, row)| row.site.clone()).collect::<Vec<_>>()
        } else {
            inference.terms().iter().map(|(_, row)| row.site.clone()).collect::<Vec<_>>()
        };
        for mutation in [false, true] {
            let mut raw = typed.nodes().iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
            if mutation {
                raw[sites[0].node().index()].recovery = NodeRecoveryState::Recovered;
            } else {
                raw[sites[0].node().index()].anchor = raw[sites[1].node().index()].anchor.clone();
            }
            let altered = TypedAst::try_new(TypedAstParts {
                source_id: source.source_id(), module_id: source.module().clone(), resolved_root: typed.resolved_root(),
                source_context: None, source_type: None, source_attribute: None,
                nodes: TypedArena::try_new(typed.nodes().root(), raw).unwrap(),
                contexts: typed.contexts().clone(), types: typed.types().clone(), facts: typed.facts().clone(),
                coercions: typed.coercions().clone(), initial_obligations: typed.initial_obligations().clone(),
                diagnostics: typed.diagnostics().clone(),
            }).unwrap();
            let installed = altered.with_source_term(primary.clone());
            if formula {
                assert!(installed.unwrap().with_source_atomic_formula(atomic.clone()).is_err(), "atomic range/recovery mutation {mutation}");
            } else {
                assert!(installed.is_err(), "primary range/recovery mutation {mutation}");
            }
        }
    }
}
