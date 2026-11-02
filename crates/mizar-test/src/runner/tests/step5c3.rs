use super::*;

#[test]
fn step5c3_functor_argument_mismatch_tracks_both_real_candidates_and_actual_bindings() {
    use super::super::type_elaboration::step5c3_functor_argument_detail_keys;
    use mizar_checker::overload_resolution::{
        ArgumentViabilityEvidence, CandidateRejectionReason, CandidateViabilityInput,
        CandidateViabilityOutput, CandidateViabilityStatus, OverloadResultStatus,
        OverloadSelectionOutput, SpecificityGraphOutput,
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, TypeHeadRef, check_source_distinct_loci_overloads,
    };
    use mizar_checker::typed_ast::{TypedNodeId, TypedSiteRef};
    use mizar_resolve::{env::SymbolKind, names::resolve_template_formal};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_argument_type_mismatch_functor_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let renamed = text
        .replace("WBox2", "Container")
        .replace("wget", "extract")
        .replace("WGetDef", "ExtractDef")
        .replace("WidenBad1", "UseExtract")
        .replace(" d ", " value ")
        .replace(".d", ".value")
        .replace("B", "Formal")
        .replace("X", "Actual");
    for (variant, mask) in (0..4)
        .map(|mask| (text.clone(), mask))
        .chain([(renamed, 0)])
    {
        let mut variant = variant;
        if mask & 1 != 0 {
            variant = variant.replace(
                "for X being set holds wget X = X",
                "for X being WBox2 holds wget X = X.d",
            );
        }
        if mask & 2 != 0 {
            variant = variant.replace(
                "let X be set;\n  thus wget X = X",
                "let X be WBox2;\n  thus wget X = X.d",
            );
        }
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
        let keys = step5c3_functor_argument_detail_keys(&source, &symbols, &typed).unwrap();
        assert_eq!(
            keys,
            if mask == 3 {
                Vec::<String>::new()
            } else {
                vec!["types.application.argument_type_mismatch".into()]
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
        assert_eq!(graphs.graphs().len(), 2);
        assert_eq!(selection.results().len(), 2);
        assert!(selection.inserted_views().is_empty());
        let constructor = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Functor)
            .unwrap();
        let structure = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Structure)
            .unwrap();
        let binder_offsets = if variant.contains("Actual") {
            [
                variant.find("for Actual").unwrap() + 4,
                variant.find("let Actual").unwrap() + 4,
            ]
        } else {
            [
                variant.find("for X").unwrap() + 4,
                variant.find("let X").unwrap() + 4,
            ]
        };
        let mut binders = Vec::new();
        for (index, (site_id, site)) in collection.sites().iter().enumerate() {
            let application = typed
                .node(site.owner.node())
                .unwrap()
                .resolved_node
                .unwrap();
            let application_node = source.arena().node(application).unwrap();
            assert_eq!(
                SourceAnchor::Range(site.source_range),
                *application_node.origin().anchor()
            );
            assert_eq!(
                site.arguments,
                vec![TypedSiteRef::Node(TypedNodeId::new(
                    application_node.children()[1].index()
                ))]
            );
            let argument = source.arena().node(application_node.children()[1]).unwrap();
            assert_eq!(
                argument.kind(),
                &mizar_syntax::SurfaceNodeKind::TermReference
            );
            let binder = resolve_template_formal(&source, argument.children()[0]).unwrap();
            assert!(
                matches!(source.arena().node(binder).unwrap().origin().anchor(), SourceAnchor::Range(range) if range.start == binder_offsets[index])
            );
            binders.push(binder);
            let (_, candidate) = expansion
                .candidates()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(&candidate.symbol, constructor.symbol());
            assert_eq!(candidate.ordinary_root, candidate.symbol);
            assert_eq!(
                SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                *constructor.origin().anchor()
            );
            assert_eq!(candidate.parameters.len(), 1);
            let parameter = normalization
                .normalized_types()
                .get(candidate.parameters[0])
                .unwrap();
            assert_eq!(
                parameter.head,
                TypeHeadRef::Structure(structure.symbol().clone())
            );
            assert_eq!(parameter.status, NormalizedTypeStatus::Known);
            assert_eq!(
                normalization
                    .normalized_types()
                    .get(candidate.result.unwrap())
                    .unwrap()
                    .head,
                TypeHeadRef::BuiltinSet
            );
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
            if mask & (1 << index) == 0 {
                let CandidateViabilityStatus::Rejected { reasons } = &decision.status else {
                    panic!("{:?}", decision.status)
                };
                let [reason] = reasons.as_slice() else {
                    panic!("{reasons:?}")
                };
                assert_eq!(reason.argument_index, 0);
                assert_eq!(reason.reason, CandidateRejectionReason::MissingEvidence);
                assert_eq!(reason.target, Some(parameter.id));
                let actual = normalization
                    .normalized_types()
                    .get(reason.actual.unwrap())
                    .unwrap();
                assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
                assert_eq!(actual.status, NormalizedTypeStatus::Known);
                assert!(
                    matches!(&result.status, OverloadResultStatus::NoMatch { rejected } if rejected.is_empty())
                );
                assert!(decision.output_candidate.is_none());
            } else {
                let CandidateViabilityStatus::Viable { views } = &decision.status else {
                    panic!("{:?}", decision.status)
                };
                assert_eq!(views.len(), 1);
                assert_eq!(views[0].actual, parameter.id);
                assert_eq!(views[0].target, parameter.id);
                assert!(matches!(
                    result.status,
                    OverloadResultStatus::Resolved { .. }
                ));
            }
        }
        assert_ne!(binders[0], binders[1]);
        if mask == 0 && variant == text {
            // The same coarse pipeline failure also occurs for the reverse pair;
            // its actual/target heads, rather than NoMatch, identify its meaning.
            let mut candidates = collection
                .candidates()
                .iter()
                .map(
                    |(_, row)| mizar_checker::overload_resolution::OverloadCandidateInput {
                        site: row.site_key.clone(),
                        symbol: row.symbol.clone(),
                        ordinary_root: row.ordinary_root.clone(),
                        declaration_kind: row.declaration_kind.clone(),
                        parameters: vec![row.result.unwrap()],
                        result: row.result,
                        origin: row.origin.clone(),
                        template: None,
                        coherence: None,
                        provenance: row.provenance.clone(),
                    },
                )
                .collect::<Vec<_>>();
            let structure_type = expansion.candidates().iter().next().unwrap().1.parameters[0];
            let sites = collection.sites().iter().map(|(_, row)| {
                mizar_checker::overload_resolution::OverloadSiteInput {
                    key: row.key.clone(),
                    owner: row.owner.clone(),
                    source_range: row.source_range,
                    kind: row.kind.clone(),
                    name: row.name.clone(),
                    arguments: row.arguments.clone(),
                    expected: None,
                    source_qua: Vec::new(),
                    recovery: row.recovery.clone(),
                }
            });
            let other = mizar_checker::overload_resolution::OverloadCollectionOutput::collect(
                sites,
                candidates.drain(..),
            );
            let expanded =
                mizar_checker::overload_resolution::TemplateExpansionOutput::expand(&other);
            let filtered = CandidateViabilityOutput::filter(
                &expanded,
                expanded
                    .candidates()
                    .iter()
                    .map(|(candidate, _)| CandidateViabilityInput {
                        candidate,
                        arguments: vec![ArgumentViabilityEvidence::Exact {
                            actual: structure_type,
                        }],
                    }),
            );
            for (_, row) in filtered.decisions().iter() {
                assert!(
                    matches!(&row.status, CandidateViabilityStatus::Rejected { reasons } if reasons[0].reason == CandidateRejectionReason::MissingEvidence && reasons[0].actual == Some(structure_type))
                );
            }
            let selected =
                OverloadSelectionOutput::resolve(&SpecificityGraphOutput::build(&filtered, []), []);
            assert!(
                selected
                    .results()
                    .iter()
                    .all(|(_, row)| matches!(row.status, OverloadResultStatus::NoMatch { .. }))
            );
        }
    }
}

#[test]
fn step5c3_functor_argument_mismatch_never_credits_unsupported_source_or_forged_identity() {
    use super::super::type_elaboration::step5c3_functor_argument_detail_keys;
    use mizar_checker::typed_ast::{TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_argument_type_mismatch_functor_001")
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
        ("equals B.d", "equals B"),
        ("equals B.d", "equals X.d"),
        ("equals B.d", "equals B.missing"),
        ("wget B -> set", "wget B -> object"),
        ("field d -> set", "field d -> object"),
        ("holds wget X = X", "holds wget B = X"),
        ("thus wget X = X", "thus wget B = X"),
        ("holds wget X = X", "holds wget X = B"),
        ("thus wget X = X", "thus wget X = B"),
        ("let B be WBox2", "let B be set"),
        ("coherence;", "existence;"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        let (source, typed, symbols) = inputs(&changed);
        assert!(
            step5c3_functor_argument_detail_keys(&source, &symbols, &typed).is_err(),
            "{changed}"
        );
    }
    let legal_widening = text
        .replace("let B be WBox2", "let B be object")
        .replace("wget B -> set equals B.d", "wget B -> object equals B");
    let (source, typed, symbols) = inputs(&legal_widening);
    assert!(step5c3_functor_argument_detail_keys(&source, &symbols, &typed).is_err());
    let (source, typed, symbols) = inputs(&text);
    let applications = typed
        .iter()
        .filter(|(_, node)| node.kind.as_str().starts_with("PrefixExpression("))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(applications.len(), 2);
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for application in applications {
        let argument = typed.node(application).unwrap().children[1].index();
        for mutation in 0..4 {
            let mut nodes = raw.clone();
            match mutation {
                0 => nodes[argument].anchor = nodes[application.index()].anchor.clone(),
                1 => nodes[argument].resolved_node = nodes[application.index()].resolved_node,
                2 => nodes[argument].typing = TypingState::Successful,
                3 => nodes[argument].children.clear(),
                _ => unreachable!(),
            }
            let changed = TypedArena::try_new(typed.root(), nodes).unwrap();
            assert!(step5c3_functor_argument_detail_keys(&source, &symbols, &changed).is_err());
        }
    }
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let call_offsets = text
        .match_indices("wget X")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(call_offsets.len(), 2);
    for target in call_offsets.into_iter().map(Some).chain([None]) {
        use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changes = 0;
        for node in ast.nodes() {
            let mut children = node
                .children
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect::<Vec<_>>();
            let id = match &node.kind {
                K::Token(token) => {
                    let spelling = if Some(node.range.start) == target {
                        assert_eq!(token.text.as_ref(), "wget");
                        changes += 1;
                        "oops".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                K::PrefixExpression(operator) if Some(node.range.start) == target => {
                    let mut operator = operator.clone();
                    operator.spelling = "oops".into();
                    builder.add_node(K::PrefixExpression(operator), node.range, children)
                }
                kind => {
                    if target.is_none() && kind == &K::ItemList {
                        assert_eq!(children.len(), 3);
                        children.swap(0, 1);
                        changes += 1;
                    }
                    builder.add_node(kind.clone(), node.range, children)
                }
            };
            rebuilt.push(id);
        }
        assert_eq!(changes, 1);
        let mut frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        frontend.ast = Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (changed_source, changed_typed, changed_symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&changed_source, &changed_symbols)
            .unwrap();
        assert!(
            step5c3_functor_argument_detail_keys(&changed_source, &changed_symbols, &changed_typed)
                .is_err()
        );
    }
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("argument"));
    let foreign = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
    assert!(step5c3_functor_argument_detail_keys(&foreign, &symbols, &typed).is_err());
    let mut other_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    other_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    let foreign_source = SurfaceResolvedArena::lower(&other_ast, symbols.module_id()).unwrap();
    assert!(step5c3_functor_argument_detail_keys(&foreign_source, &symbols, &typed).is_err());
    let (_, _, other_symbols) = inputs(&text.replace("WBox2", "Container"));
    assert!(step5c3_functor_argument_detail_keys(&source, &other_symbols, &typed).is_err());
    let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        step5c3_functor_argument_detail_keys(
            &source,
            &SymbolEnv::new(symbols.module_id().clone(), indexes),
            &typed
        )
        .is_err()
    );
    let theorem_start = text.find("theorem WidenBad1:").unwrap();
    let unrelated = format!("{text}\n{}", &text[theorem_start..]);
    let frontend = super::super::formula_statement::step5c8_test_frontend(&unrelated);
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let result = super::super::resolver_symbol_collection(
        &config.workspace_root,
        case,
        frontend.ast.as_ref().unwrap(),
    );
    assert!(!result.detail_keys.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let malformed = text.replace("thus wget X = X;", "thus wget X = ;");
    let frontend = super::super::formula_statement::step5c8_test_frontend(&malformed);
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
}
#[test]
fn step5c3_widening_checks_both_real_calls_and_gates_after_fresh_proof() {
    use mizar_checker::{
        overload_resolution::*, registration_resolution::*, type_checker::*, typed_ast::*,
    };
    let original = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let renamed = original
        .replace("WMDef", "MarkDefinition")
        .replace("wmarked", "tagged")
        .replace("WMarkedExists", "TaggedExists")
        .replace("WBoxDef", "BoxDefinition")
        .replace("wbox", "container")
        .replace("WidenArg1", "Consumer")
        .replace("X", "Value");
    for text in [original, renamed] {
        let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
        let database = mizar_proof::status::prove_source_existential_registration(
            &source,
            &nodes,
            &symbols,
            super::super::shared::snapshot_id(0),
            &mizar_proof::policy::VerifierPolicy::release(),
        )
        .unwrap();
        let output =
            check_source_attribute_widening_types(&source, &nodes, &symbols, &database).unwrap();
        assert_eq!(
            output,
            check_source_attribute_widening_types(&source, &nodes, &symbols, &database).unwrap()
        );
        let (gates, coercions, collection, viability) = output;
        let active = database.activated().iter().next().unwrap();
        let gates = gates.iter().collect::<Vec<_>>();
        assert_eq!(gates.len(), 2);
        assert_ne!(gates[0].owner(), gates[1].owner());
        for gate in &gates {
            assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
            assert_eq!(gate.registration(), Some(active.id()));
            assert_eq!(gate.pattern(), active.pattern());
            assert!(gate.base_evidence_kind().is_none() && gate.base_evidence_coverage().is_none());
            assert!(gate.facts().is_empty() && gate.diagnostics().is_empty());
            let node = nodes.node(gate.owner().node()).unwrap();
            assert_eq!(node.kind.as_str(), "TypeExpression");
            assert_eq!(
                node.anchor,
                mizar_session::SourceAnchor::Range(gate.source_range())
            );
            let spelling = &text[gate.source_range().start..gate.source_range().end];
            assert!(
                spelling == "wmarked set" || spelling == "tagged set",
                "{spelling}"
            );
        }
        assert!(coercions.diagnostics().is_empty() && coercions.initial_obligations().is_empty());
        assert_eq!(coercions.facts().len(), 2);
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 2);
        let mut bindings = std::collections::BTreeSet::new();
        for ((_, site), (_, decision)) in
            collection.sites().iter().zip(viability.decisions().iter())
        {
            assert_eq!(site.arguments.len(), 1);
            let arg = &site.arguments[0];
            let typed_arg = nodes.node(arg.node()).unwrap();
            let resolved_arg = source
                .arena()
                .node(typed_arg.resolved_node.unwrap())
                .unwrap();
            bindings.insert(
                mizar_resolve::names::resolve_template_formal(&source, resolved_arg.children()[0])
                    .unwrap(),
            );
            let row = coercions
                .coercions()
                .iter()
                .find_map(|(_, row)| (&row.site == arg).then_some(row))
                .unwrap();
            assert_eq!(row.status, CoercionStatus::Candidate);
            assert_eq!(row.kind, CoercionKind::Widening);
            let actual = row.from.unwrap();
            assert_ne!(actual, row.to);
            let ty = coercions.normalized_types().get(actual).unwrap();
            let target = coercions.normalized_types().get(row.to).unwrap();
            assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
            assert_eq!(ty.status, NormalizedTypeStatus::Known);
            assert_eq!(ty.attributes.positive().len(), 1);
            assert!(ty.attributes.negative().is_empty() && ty.args.is_empty());
            assert_eq!(target.head, ty.head);
            assert!(
                target.attributes.positive().is_empty()
                    && target.attributes.negative().is_empty()
                    && target.args.is_empty()
            );
            let [fact] = row.supporting_facts.as_slice() else {
                panic!("one support")
            };
            let fact = coercions.facts().get(*fact).unwrap();
            assert_eq!(&fact.subject, arg);
            assert_eq!(fact.status, FactStatus::Known);
            assert!(matches!(fact.provenance, FactProvenance::Builtin(_)));
            let CandidateViabilityStatus::Viable { views } = &decision.status else {
                panic!("viable")
            };
            assert_eq!(views.len(), 1);
            assert_eq!(views[0].kind, ArgumentViewKind::FactWidening);
            assert_eq!((views[0].actual, views[0].target), (actual, row.to));
            assert_eq!(views[0].facts, row.supporting_facts);
            let candidate = collection
                .candidates()
                .get(decision.source_candidate)
                .unwrap();
            assert_eq!(candidate.parameters, [row.to]);
            assert_eq!(candidate.result, Some(row.to));
        }
        assert_eq!(
            bindings.len(),
            2,
            "equal spellings retain distinct resolver binders"
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        for mutation in 0..6 {
            let inputs = expansion.candidates().iter().map(|(id, candidate)| {
                let site = collection.sites().get(candidate.site).unwrap();
                let row = coercions
                    .coercions()
                    .iter()
                    .find_map(|(_, row)| (row.site == site.arguments[0]).then_some(row))
                    .unwrap();
                CandidateViabilityInput {
                    candidate: id,
                    arguments: vec![ArgumentViabilityEvidence::FactWidening {
                        actual: row.from.unwrap(),
                        target: if mutation == 5 {
                            row.from.unwrap()
                        } else {
                            row.to
                        },
                        facts: if mutation == 0 {
                            Vec::new()
                        } else {
                            row.supporting_facts.clone()
                        },
                        status: match mutation {
                            1 => ViabilityFactStatus::PendingObligation,
                            2 => ViabilityFactStatus::Degraded,
                            3 => ViabilityFactStatus::Rejected,
                            4 => ViabilityFactStatus::OutOfScopeAssumption,
                            _ => ViabilityFactStatus::Consumable,
                        },
                    }],
                }
            });
            let rejected = CandidateViabilityOutput::filter(&expansion, inputs);
            assert!(
                rejected.decisions().iter().all(|(_, decision)| !matches!(
                    decision.status,
                    CandidateViabilityStatus::Viable { .. }
                )),
                "mutation {mutation}"
            );
        }
        for mutation in 0..5 {
            let candidate = ExistentialGateCandidate::new(
                active.id(),
                active.pattern().clone(),
                active.correctness().clone(),
                active.evidence().clone(),
                active.trigger().clone(),
                gates[0].attributes().to_vec(),
            )
            .with_fingerprint(active.fingerprint().unwrap().clone());
            let candidate = match mutation {
                0 => candidate.with_fingerprint("foreign"),
                1 => candidate.with_correctness("foreign"),
                2 => candidate.with_activation_evidence("foreign"),
                3 => candidate.with_trigger("foreign"),
                _ => candidate.with_attributes([RegistrationAttributeKey::new("foreign")]),
            };
            let input = ExistentialGateInput::new(
                gates[0].owner().clone(),
                gates[0].source_range(),
                active.pattern().clone(),
                active.trigger().clone(),
                gates[0].attributes().to_vec(),
            )
            .with_candidates([candidate]);
            let rejected = ExistentialGateOutput::evaluate(&database, [input]);
            assert!(
                rejected
                    .iter()
                    .all(|gate| gate.status() != ExistentialGateStatus::Satisfied)
            );
        }
    }
}

#[test]
fn step5c3_widening_rejects_changed_consumers_and_foreign_inputs() {
    use mizar_checker::{
        registration_resolution::*, type_checker::check_source_attribute_widening_types as check,
        typed_ast::*,
    };
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    for (from, to) in [
        ("holds wbox X = X", "holds wbox Missing = X"),
        ("thus wbox X = X", "thus wbox Missing = X"),
        ("holds wbox X = X", "holds X = X"),
        ("thus wbox X = X", "thus X = X"),
        ("holds wbox X = X", "holds wbox X = wbox X"),
        ("thus wbox X = X", "thus wbox X = wbox X"),
        ("holds wbox X = X", "holds missing X = X"),
        ("thus wbox X = X", "thus missing X = X"),
        ("for X being wmarked set", "for X being set"),
        ("let X be wmarked set", "let X be set"),
        ("for X being wmarked set", "for X being non wmarked set"),
        ("let X be wmarked set", "let X be non wmarked set"),
        (
            "func WBoxDef: wbox X -> set",
            "func WBoxDef: wbox X -> object",
        ),
        ("-> set equals X", "-> set equals the set"),
        ("func WBoxDef: wbox X", "func WBoxDef: wbox Missing"),
        ("let X be set;\n  func", "let X be object;\n  func"),
        ("by WBoxDef;", "by WMDef;"),
        ("by WBoxDef;", ";"),
        ("means X = X", "means not X = X"),
        ("by WMDef;", "by WBoxDef;"),
        ("take the set;", "take the wmarked set;"),
        ("by WMDef;", ";"),
        ("coherence;", "coherence; coherence;"),
        ("holds wbox X = X", "holds wbox = X"),
    ] {
        let changed = text.replacen(from, to, 1);
        assert_ne!(changed, text, "missing mutation {from}");
        let result = step5c3_registration_inputs(&changed).and_then(|(source, nodes, symbols)| {
            let database = mizar_proof::status::prove_source_existential_registration(
                &source,
                &nodes,
                &symbols,
                super::super::shared::snapshot_id(0),
                &mizar_proof::policy::VerifierPolicy::release(),
            )?;
            check(&source, &nodes, &symbols, &database)
        });
        assert!(result.is_err(), "accepted {from} -> {to}");
    }
    let (first, rest) = text.split_once("registration\n").unwrap();
    let (registration, tail) = rest.split_once("\n\ndefinition\n").unwrap();
    for changed in [
        format!("{first}definition\n{tail}\nregistration\n{registration}"),
        format!("{text}\ntheorem Extra: for X being set holds X = X;\n"),
    ] {
        let rejected =
            step5c3_registration_inputs(&changed).and_then(|(source, nodes, symbols)| {
                let database = mizar_proof::status::prove_source_existential_registration(
                    &source,
                    &nodes,
                    &symbols,
                    super::super::shared::snapshot_id(0),
                    &mizar_proof::policy::VerifierPolicy::release(),
                )?;
                check(&source, &nodes, &symbols, &database)
            });
        assert!(rejected.is_err(), "extra item or late registration");
    }
    let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
    let database = mizar_proof::status::prove_source_existential_registration(
        &source,
        &nodes,
        &symbols,
        super::super::shared::snapshot_id(0),
        &mizar_proof::policy::VerifierPolicy::release(),
    )
    .unwrap();
    let pending = check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
    assert!(check(&source, &nodes, &symbols, pending.database()).is_err());
    let (foreign_source, foreign_nodes, foreign_symbols) =
        step5c3_registration_inputs(&text.replace("wmarked", "othermarked")).unwrap();
    let foreign_database = mizar_proof::status::prove_source_existential_registration(
        &foreign_source,
        &foreign_nodes,
        &foreign_symbols,
        super::super::shared::snapshot_id(0),
        &mizar_proof::policy::VerifierPolicy::release(),
    )
    .unwrap();
    assert!(check(&source, &nodes, &symbols, &foreign_database).is_err());
    assert!(check(&source, &foreign_nodes, &symbols, &database).is_err());
    assert!(check(&source, &nodes, &foreign_symbols, &database).is_err());
    let target = nodes
        .iter()
        .find(|(_, node)| node.kind.as_str() == "LetStatement")
        .unwrap()
        .0;
    for mutation in 0..5 {
        let mut raw = nodes
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match mutation {
            0 => raw[target.index()].typing = TypingState::Successful,
            1 => raw[target.index()].recovery = NodeRecoveryState::Recovered,
            2 => raw[target.index()].children.reverse(),
            3 => {
                raw[target.index()].resolved_node = raw[nodes.root().unwrap().index()].resolved_node
            }
            _ => raw[target.index()].anchor = raw[nodes.root().unwrap().index()].anchor.clone(),
        }
        let altered = TypedArena::try_new(nodes.root(), raw).unwrap();
        assert!(
            check(&source, &altered, &symbols, &database).is_err(),
            "neutral mutation {mutation}"
        );
    }
}

#[test]
fn step5c3_source_registration_preserves_real_choices_accounting_and_full_baselines() {
    use mizar_checker::registration_resolution::*;
    use mizar_core::core_ir::{
        CoreFormulaKind, CoreTermKind, ObligationSeedKind, ObligationSeedStatus,
    };
    use mizar_vc::vc_ir::{SeedNoVcReason, SeedVcMapping, VcStatus};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let renamed = text
        .replace("WMDef", "MarkDefinition")
        .replace("wmarked", "tagged")
        .replace("WMarkedExists", "TaggedExists")
        .replace("WBoxDef", "BoxDefinition")
        .replace("wbox", "container")
        .replace("WidenArg1", "Consumer")
        .replace("X", "Value");
    for (index, text) in [text, renamed].iter().enumerate() {
        let (source, nodes, symbols) = step5c3_registration_inputs(text).unwrap();
        let checked =
            check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
        assert!(checked.database().activated().is_empty());
        assert_eq!(checked.database().pending().len(), 1);
        assert_eq!(checked.validations().len(), 1);
        let choices = checked.choice_terms().unwrap();
        assert_eq!(choices.terms().len(), 2);
        assert_eq!(choices.type_sites().len(), 2);
        assert_eq!(choices.requests().len(), 4);
        let gates = checked.choice_gates().unwrap();
        assert_eq!(gates.len(), 2);
        for ((_, choice), gate) in choices.terms().iter().zip(gates.iter()) {
            assert_eq!(gate.source_range(), choice.source_range());
            assert_eq!(gate.owner(), choice.site());
            assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
            assert_eq!(
                gate.base_evidence_kind(),
                Some(ExistentialGateBaseEvidenceKind::BuiltinSet)
            );
            assert_eq!(
                gate.base_evidence_coverage(),
                Some(ExistentialGateBaseEvidenceCoverage::Builtin)
            );
            assert!(gate.registration().is_none());
            assert!(gate.attributes().is_empty() && gate.facts().is_empty());
        }
        let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
        let witnesses = core
            .terms()
            .iter()
            .filter(|(_, term)| matches!(term.kind, CoreTermKind::Apply { .. }))
            .collect::<Vec<_>>();
        assert_eq!(witnesses.len(), 2);
        assert_ne!(witnesses[0].0, witnesses[1].0);
        assert_ne!(witnesses[0].1.source, witnesses[1].1.source);
        assert_eq!(witnesses[0].1.kind, witnesses[1].1.kind);
        let (_, nonempty) = core
            .obligation_seeds()
            .iter()
            .find(|(_, seed)| seed.kind == ObligationSeedKind::GeneratedNonEmptiness)
            .unwrap();
        assert_eq!(nonempty.status, ObligationSeedStatus::Active);
        assert!(nonempty.context.is_empty());
        assert!(matches!(
            core.formulas().get(nonempty.goal.unwrap()).unwrap().kind,
            CoreFormulaKind::Exists { .. }
        ));
        let vcs = step5c3_registration_vcs(&core).unwrap();
        assert_eq!(vcs.vcs().len(), 2);
        assert!(vcs.vcs().iter().all(|vc| vc.status == VcStatus::Open));
        assert_eq!(vcs.seed_accounting().len(), 2);
        assert!(vcs.seed_accounting().iter().any(|row| matches!(
            row.mapping,
            SeedVcMapping::NoConcreteVc {
                reason: SeedNoVcReason::BuiltinSetInhabitation { .. }
            }
        ) && row.seed_status
            == ObligationSeedStatus::Active));
        assert!(vcs.seed_accounting().iter().any(|row| matches!(&row.mapping, SeedVcMapping::Expanded { vcs, .. } if vcs.len() == 2 && vcs[0].expansion_index == 0 && vcs[1].expansion_index == 1)));
        let mut handoff_text = String::new();
        for vc in vcs.vcs() {
            let handoff =
                mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(
                    &core, &vcs, vc.id,
                )
                .unwrap();
            assert!(handoff.targets_vc(&vcs, vc.id).unwrap());
            assert_eq!(
                handoff.canonical_evidence().substitutions().len(),
                vc.id.index()
            );
            assert!(step5c3_check_registration_handoff(&handoff, "clean").is_ok());
            handoff_text.push_str(&handoff.debug_text());
        }
        if index == 0 {
            for (path, actual) in [
                (
                    "core/step5c3_source_registration.core_ir.snap",
                    core.debug_text(),
                ),
                (
                    "vc/step5c3_source_registration.vc_ir.snap",
                    vcs.debug_text(),
                ),
                (
                    "vc/step5c3_source_registration.kernel_handoff.snap",
                    handoff_text,
                ),
            ] {
                assert_eq!(
                    actual,
                    std::fs::read_to_string(
                        step5c11_config()
                            .workspace_root
                            .join("tests/snapshots")
                            .join(path)
                    )
                    .unwrap(),
                    "{path}"
                );
            }
        }
        let database = mizar_proof::status::prove_source_existential_registration(
            &source,
            &nodes,
            &symbols,
            super::super::shared::snapshot_id(0),
            &mizar_proof::policy::VerifierPolicy::release(),
        )
        .unwrap();
        let active = database.activated().iter().next().unwrap();
        assert_eq!(database.activated().len(), 1);
        assert!(database.pending().is_empty() && database.rejected().is_empty());
        assert_eq!(
            active.pattern().as_str(),
            format!("{:?}", checked.validations()[0].pattern())
        );
        assert_eq!(
            active.source().origin(),
            symbols.registrations().iter().next().unwrap().origin()
        );
        assert_eq!(
            active.correctness().as_str(),
            checked.validations()[0].correctness_provenance().as_str()
        );
        assert!(active.fingerprint().is_some());
        assert!(
            checked.database().activated().is_empty(),
            "proof does not mutate earlier pending input"
        );
        let policy =
            mizar_proof::policy::VerifierPolicy::release().with_kernel_evidence_formats([]);
        assert!(
            mizar_proof::status::prove_source_existential_registration(
                &source,
                &nodes,
                &symbols,
                super::super::shared::snapshot_id(0),
                &policy
            )
            .is_err()
        );
        let RegistrationValidationPattern::Existential { attributes, .. } =
            checked.validations()[0].pattern()
        else {
            panic!("existential");
        };
        let candidate = ExistentialGateCandidate::new(
            active.id(),
            active.pattern().clone(),
            active.correctness().clone(),
            active.evidence().clone(),
            active.trigger().clone(),
            attributes.clone(),
        )
        .with_fingerprint(active.fingerprint().unwrap().clone());
        let site = nodes
            .iter()
            .find(|(_, node)| node.kind.as_str() == "Theorem")
            .map(|(id, _)| id)
            .unwrap_or(nodes.root().unwrap());
        let range = mizar_session::SourceRange {
            source_id: source.source_id(),
            start: text.len() - 2,
            end: text.len() - 1,
        };
        for (pattern, expected) in [
            (active.pattern().clone(), true),
            (RegistrationPatternKey::new("builtin.set"), false),
        ] {
            let gate = ExistentialGateOutput::evaluate(
                &database,
                [ExistentialGateInput::new(
                    mizar_checker::typed_ast::TypedSiteRef::Node(site),
                    range,
                    pattern,
                    active.trigger().clone(),
                    attributes.clone(),
                )
                .with_candidates([candidate.clone()])],
            );
            assert_eq!(
                gate.iter().next().unwrap().status() == ExistentialGateStatus::Satisfied,
                expected
            );
        }
    }
}

#[test]
fn step5c3_source_registration_rejects_invalid_source_and_foreign_neutral_inputs() {
    use mizar_checker::{
        registration_resolution::check_source_existential_registration_proof as check,
        typed_ast::{NodeRecoveryState, TypedArena, TypingState},
    };
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    for (from, to) in [
        ("by WMDef;", ";"),
        ("by WMDef;", "by WBoxDef;"),
        ("by WMDef;", "by Missing;"),
        ("take the set;", "take the wmarked set;"),
        ("thus the set is wmarked", "thus the wmarked set is wmarked"),
        ("take the set;", ""),
        (
            "take the set;\n    thus the set is wmarked by WMDef;",
            "thus the set is wmarked by WMDef;\n    take the set;",
        ),
        ("holds wbox X = X", "holds wbox Absent = X"),
    ] {
        let changed = text.replacen(from, to, 1);
        assert_ne!(changed, text);
        let frontend = super::super::formula_statement::step5c8_test_frontend(&changed);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control {from}: {:?}",
            frontend.diagnostics
        );
        let rejected = step5c3_registration_inputs(&changed)
            .map_or(true, |(source, nodes, symbols)| {
                check(&source, &nodes, &symbols).is_err()
            });
        assert!(rejected, "accepted source mutation {from} -> {to}");
    }
    let (definition, rest) = text.split_once("registration\n").unwrap();
    let (registration, later) = rest.split_once("\n\ndefinition\n").unwrap();
    let reordered = format!("registration\n{registration}\n{definition}\ndefinition\n{later}");
    assert!(
        step5c3_registration_inputs(&reordered).map_or(true, |(source, nodes, symbols)| check(
            &source, &nodes, &symbols
        )
        .is_err())
    );
    for malformed in [
        text.replace("take the set;", "take ;"),
        text.replace("take the set;", "assume contradiction; take the set;"),
        text.replace("holds wbox X = X", "holds missing X = X"),
    ] {
        assert!(
            !super::super::formula_statement::step5c8_test_frontend(&malformed)
                .diagnostics
                .is_empty()
        );
        assert!(step5c3_registration_inputs(&malformed).is_err());
    }
    let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
    let (_, _, foreign_symbols) =
        step5c3_registration_inputs(&text.replace("wmarked", "marked")).unwrap();
    assert!(check(&source, &nodes, &foreign_symbols).is_err());
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("registration"));
    assert!(
        check(
            &SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap(),
            &nodes,
            &symbols
        )
        .is_err()
    );
    let mut foreign_ast = ast;
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    foreign_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    assert!(
        check(
            &SurfaceResolvedArena::lower(&foreign_ast, source.module()).unwrap(),
            &nodes,
            &symbols
        )
        .is_err()
    );
    let target = nodes
        .iter()
        .find(|(_, node)| node.kind.as_str() == "TakeStatement")
        .unwrap()
        .0;
    for mutation in 0..5 {
        let mut raw = nodes
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match mutation {
            0 => raw[target.index()].typing = TypingState::Successful,
            1 => raw[target.index()].recovery = NodeRecoveryState::Recovered,
            2 => raw[target.index()].children.clear(),
            3 => {
                raw[target.index()].resolved_node = raw[nodes.root().unwrap().index()].resolved_node
            }
            4 => raw[target.index()].anchor = raw[nodes.root().unwrap().index()].anchor.clone(),
            _ => unreachable!(),
        }
        let altered = TypedArena::try_new(nodes.root(), raw).unwrap();
        assert!(
            check(&source, &altered, &symbols).is_err(),
            "neutral mutation {mutation}"
        );
    }
    let negative = text.replacen("means X = X", "means not X = X", 1);
    let (source, nodes, symbols) = step5c3_registration_inputs(&negative).unwrap();
    let checked = check(&source, &nodes, &symbols).unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let vcs = step5c3_registration_vcs(&core).unwrap();
    for vc in vcs.vcs() {
        let handoff = mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(
            &core, &vcs, vc.id,
        )
        .unwrap();
        assert_eq!(
            step5c3_check_registration_handoff(&handoff, "clean").is_ok(),
            vc.id.index() == 0
        );
    }
    assert!(
        mizar_proof::status::prove_source_existential_registration(
            &source,
            &nodes,
            &symbols,
            super::super::shared::snapshot_id(0),
            &mizar_proof::policy::VerifierPolicy::release()
        )
        .is_err()
    );
    assert!(checked.database().activated().is_empty());
}

#[test]
fn step5c3_source_registration_normal_wire_rejects_context_polarity_and_substitution_changes() {
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
    let checked =
        mizar_checker::registration_resolution::check_source_existential_registration_proof(
            &source, &nodes, &symbols,
        )
        .unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let vcs = step5c3_registration_vcs(&core).unwrap();
    for vc in vcs.vcs() {
        let handoff = mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(
            &core, &vcs, vc.id,
        )
        .unwrap();
        let result = step5c3_check_registration_handoff(&handoff, "clean").unwrap();
        assert!(result.sat_check_report().is_some());
        for mutation in [
            "wire",
            "goal-bytes",
            "context",
            "provenance",
            "polarity",
            "resource",
        ] {
            assert!(
                step5c3_check_registration_handoff(&handoff, mutation).is_err(),
                "leaf {:?}: {mutation}",
                vc.id
            );
        }
        if vc.id.index() == 1 {
            let missing = step5c3_check_registration_handoff(&handoff, "missing-equality-instance").unwrap_err();
            assert!(missing.contains("InvalidSatRefutation"), "{missing}");
            let changed_actual = step5c3_check_registration_handoff(&handoff, "substitution-actual").unwrap_err();
            assert!(changed_actual.contains("InvalidSatRefutation"), "{changed_actual}");
            for mutation in [
                "missing-substitution",
                "substitution-source",
            ] {
                assert!(
                    step5c3_check_registration_handoff(&handoff, mutation).is_err(),
                    "{mutation}"
                );
            }
        }
    }
}

#[test]
fn step5c3_source_registration_rejects_coherent_core_and_vc_corruption() {
    use mizar_core::core_ir::*;
    use mizar_vc::vc_ir::*;
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
    let checked =
        mizar_checker::registration_resolution::check_source_existential_registration_proof(
            &source, &nodes, &symbols,
        )
        .unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let (origin_id, _) = core.generated().iter().next().unwrap();
    let (definition_id, definition) = core.definitions().iter().next().unwrap();
    let (parent_id, parent) = core
        .obligation_seeds()
        .iter()
        .find(|(_, seed)| seed.kind == ObligationSeedKind::CheckerInitial)
        .unwrap();
    let (nonempty_id, nonempty) = core
        .obligation_seeds()
        .iter()
        .find(|(_, seed)| seed.kind == ObligationSeedKind::GeneratedNonEmptiness)
        .unwrap();
    let witness = core
        .terms()
        .iter()
        .find(|(_, term)| matches!(term.kind, CoreTermKind::Apply { .. }))
        .unwrap()
        .0;
    let witnesses = core
        .terms()
        .iter()
        .filter(|(_, term)| matches!(term.kind, CoreTermKind::Apply { .. }))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    let CoreFormulaKind::Exists {
        body: nonempty_body,
        ..
    } = core.formulas().get(nonempty.goal.unwrap()).unwrap().kind
    else {
        panic!("nonempty");
    };
    let (_, proof) = core.proofs().iter().next().unwrap();
    let step = core
        .proof_nodes()
        .iter()
        .find(|(_, node)| matches!(node.kind, CoreProofNodeKind::Step { .. }))
        .unwrap()
        .0;
    for mutation in 0..25 {
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
            0 => parts.generated.get_mut(origin_id).unwrap().key = "choice:attributed.set".into(),
            1 => {
                parts.generated.get_mut(origin_id).unwrap().owner = definition.owner.item().unwrap()
            }
            2 => parts
                .generated
                .get_mut(origin_id)
                .unwrap()
                .params
                .push(definition.params[0].var),
            3 => {
                parts.generated.get_mut(origin_id).unwrap().functor =
                    Some(definition.symbol.clone())
            }
            4 => parts.generated.get_mut(origin_id).unwrap().evidence.clear(),
            5 => {
                parts.terms.get_mut(witnesses[1]).unwrap().kind = CoreTermKind::Apply {
                    functor: definition.symbol.clone(),
                    args: vec![],
                }
            }
            6 => parts.terms.get_mut(witness).unwrap().source = proof.source.clone(),
            7 => {
                parts.formulas.get_mut(nonempty_body).unwrap().kind = CoreFormulaKind::Atom {
                    predicate: definition.symbol.clone(),
                    args: vec![witness],
                }
            }
            8 => {
                let CoreFormulaKind::Exists { binders, .. } =
                    &mut parts.formulas.get_mut(nonempty.goal.unwrap()).unwrap().kind
                else {
                    unreachable!();
                };
                binders[0].ty_guard = definition.params[0].ty_guard;
            }
            9 => {
                let CoreFormulaKind::Exists { binders, .. } =
                    &mut parts.formulas.get_mut(nonempty.goal.unwrap()).unwrap().kind
                else {
                    unreachable!();
                };
                binders[0].var = definition.params[0].var;
            }
            10 => {
                parts.obligation_seeds.get_mut(nonempty_id).unwrap().status =
                    ObligationSeedStatus::Deferred
            }
            11 => {
                parts.obligation_seeds.get_mut(nonempty_id).unwrap().source = parent.source.clone()
            }
            12 => parts
                .obligation_seeds
                .get_mut(parent_id)
                .unwrap()
                .context
                .push(nonempty_body),
            13 => {
                parts
                    .obligation_seeds
                    .get_mut(parent_id)
                    .unwrap()
                    .core_refs
                    .pop();
            }
            14 => {
                let CoreProofNodeKind::Step { justification, .. } =
                    &mut parts.proof_nodes.get_mut(step).unwrap().kind
                else {
                    unreachable!();
                };
                justification.citations.clear();
            }
            15 => {
                parts
                    .terms
                    .insert(core.terms().get(witness).unwrap().clone());
            }
            16 => {
                parts
                    .formulas
                    .insert(core.formulas().get(nonempty_body).unwrap().clone());
            }
            17 => {
                let mut duplicate = parent.clone();
                duplicate.local_path = "extra".into();
                parts.obligation_seeds.insert(duplicate);
            }
            18 => {
                parts.definitions.get_mut(definition_id).unwrap().params[0].ty_guard =
                    Some(nonempty_body)
            }
            19 => {
                let DefinitionBody::Formula(body) = definition.body else {
                    unreachable!();
                };
                let CoreFormulaKind::Equals { right, .. } =
                    &mut parts.formulas.get_mut(body).unwrap().kind
                else {
                    unreachable!();
                };
                *right = witness;
            }
            20 => parts.generated.get_mut(origin_id).unwrap().source = parent.source.clone(),
            21 => {
                parts.generated.get_mut(origin_id).unwrap().kind =
                    GeneratedOriginKind::FraenkelComprehension
            }
            22 => parts
                .obligation_seeds
                .get_mut(nonempty_id)
                .unwrap()
                .context
                .push(parent.goal.unwrap()),
            23 => {
                let CoreFormulaKind::Exists { body, .. } =
                    &mut parts.formulas.get_mut(nonempty.goal.unwrap()).unwrap().kind
                else {
                    unreachable!();
                };
                *body = parent.goal.unwrap();
            }
            24 => {
                for (_, node) in parts.proof_nodes.iter_mut() {
                    let citations = match &mut node.kind {
                        CoreProofNodeKind::Step { justification, .. } => {
                            &mut justification.citations
                        }
                        CoreProofNodeKind::TerminalGoal { citations, .. } => citations,
                        _ => continue,
                    };
                    for citation in citations {
                        if matches!(citation, CoreCitation::Label(label)
                            if label.as_str().starts_with("definition:"))
                        {
                            *citation =
                                CoreCitation::Label(CoreLabelRef::new("definition:foreign"));
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        parts.source_map.term_sources = parts
            .terms
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.formula_sources = parts
            .formulas
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.generated_sources = parts
            .generated
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.proof_sources = parts
            .proof_nodes
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        parts.source_map.obligation_sources = parts
            .obligation_seeds
            .iter()
            .map(|(id, row)| (id, row.source.clone()))
            .collect();
        let altered = CoreIr::try_new(parts).unwrap_or_else(|error| {
            panic!("mutation {mutation} should be structurally valid: {error}")
        });
        assert!(
            step5c3_registration_vcs(&altered).is_err(),
            "accepted Core mutation {mutation}"
        );
    }
    let vcs = step5c3_registration_vcs(&core).unwrap();
    for mutation in 0..11 {
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
            0 => parts.vcs[1].goal = parts.vcs[0].goal,
            1 => {
                let goal = parts.vcs[1].goal;
                parts.vcs[1]
                    .premises
                    .push(PremiseRef::GeneratedFact { formula: goal });
            }
            2 => parts.generated_formulas[0].shape = VcGeneratedFormulaShape::True,
            3 => parts.vcs[0].status = VcStatus::NeedsAtp,
            4 => parts.vcs[0].premises.clear(),
            5 => parts
                .seed_accounting
                .retain(|row| matches!(row.mapping, SeedVcMapping::Expanded { .. })),
            6 => {
                let row = parts
                    .seed_accounting
                    .iter_mut()
                    .find(|row| matches!(row.mapping, SeedVcMapping::NoConcreteVc { .. }))
                    .unwrap();
                row.mapping = SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::BuiltinSetInhabitation {
                        origin: GeneratedOriginId::new(origin_id.index() + 1),
                    },
                };
            }
            7 => {
                let row = parts
                    .seed_accounting
                    .iter_mut()
                    .find(|row| matches!(row.mapping, SeedVcMapping::Expanded { .. }))
                    .unwrap();
                let SeedVcMapping::Expanded { vcs, .. } = &mut row.mapping else {
                    unreachable!();
                };
                vcs[0].vc = VcId::new(1);
                vcs[1].vc = VcId::new(0);
            }
            8 => {
                parts.vcs[1].local_context = LocalContext::try_new(
                    vec![ContextEntry {
                        id: ContextEntryId::new(0),
                        sort_key: CanonicalSortKey::new("injected"),
                        kind: ContextEntryKind::ProofAssumption,
                        formula: Some(parts.vcs[1].goal),
                        provenance: vec![],
                    }],
                    vec![],
                )
                .unwrap()
            }
            9 => {
                parts.vcs[0].anchor.generation_schema_version =
                    GenerationSchemaVersion::new("foreign")
            }
            10 => parts.vcs[1].source.related.clear(),
            _ => unreachable!(),
        }
        let altered = VcSet::try_new(parts).unwrap_or_else(|error| {
            panic!("mutation {mutation} should be structurally valid: {error}")
        });
        assert!(
            mizar_vc::kernel_evidence_handoff::build_source_existential_kernel_handoff(
                &core,
                &altered,
                VcId::new(1)
            )
            .is_err(),
            "accepted VC mutation {mutation}"
        );
    }
}
