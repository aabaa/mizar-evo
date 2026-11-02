use super::*;

#[test]
fn step5c4_dependent_mode_preserves_actual_parameter_bindings_and_checked_types() {
    use mizar_checker::binding_env::{
        BinderIdentity, BindingKind, BindingLookupResult, BindingLookupSite, BindingTypeSite,
    };
    use mizar_checker::type_checker::{
        DeclarationStatus, FormulaKind, FormulaStatus, NormalizedTypeStatus, TermReference,
        TermStatus, TypeHeadRef, check_source_dependent_mode_types,
    };
    use mizar_checker::typed_ast::{FactProvenance, FactStatus, TypeEntryActual};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_mode_dependent_of_params_001")
        .unwrap();
    let exact = std::fs::read_to_string(&case.source_path).unwrap();
    let renamed = exact
        .replace("let X be set", "let Formal be set")
        .replace("of X is", "of Formal is")
        .replace(
            "for A for m being MemberKind of A holds m = m",
            "for Header being set for q being MemberKind of Header holds q = q",
        )
        .replace("let A;", "let Local be set;")
        .replace("let m be MemberKind of A", "let n be MemberKind of Local")
        .replace("thus m = m", "thus n = n");
    for (text, names, nonreflexive) in [
        (exact.clone(), ["X", "A", "A", "m", "A", "m"], false),
        (renamed, ["Formal", "A", "Header", "q", "Local", "n"], false),
        (
            exact.replace("m = m", "m = A"),
            ["X", "A", "A", "m", "A", "m"],
            true,
        ),
    ] {
        let inputs = || {
            super::super::source_registration_inputs(
                &config.workspace_root,
                case,
                super::super::formula_statement::step5c8_test_frontend(&text),
            )
            .unwrap()
        };
        let (source, typed, symbols) = inputs();
        let output = check_source_dependent_mode_types(&source, &typed, &symbols).unwrap();
        let (replay_source, replay_typed, replay_symbols) = inputs();
        assert_eq!(
            output,
            check_source_dependent_mode_types(&replay_source, &replay_typed, &replay_symbols)
                .unwrap()
        );
        let (bindings, declarations, inferred) = output;
        assert_eq!(bindings.source_id(), source.source_id());
        assert_eq!(bindings.module_id(), source.module());
        assert!(
            bindings.diagnostics().is_empty()
                && declarations.diagnostics().is_empty()
                && inferred.diagnostics().is_empty()
        );
        assert_eq!(bindings.bindings().len(), 6);
        assert_eq!(declarations.declarations().len(), 6);
        assert_eq!(inferred.terms().len(), 6);
        assert_eq!(inferred.formulas().len(), 2);
        assert!(inferred.candidate_sets().is_empty() && inferred.facts().is_empty());
        assert!(declarations.facts().iter().all(|(_, fact)| matches!(
            fact.status,
            FactStatus::Known | FactStatus::Assumed
        ) && matches!(
            fact.provenance,
            FactProvenance::Declared(_) | FactProvenance::Assumed(_)
        )));
        let proof_start = text.find("proof").unwrap();
        let starts = [
            text.find(&format!("let {} be", names[0])).unwrap() + 4,
            text.find(&format!("reserve {} for", names[1])).unwrap() + 8,
            text.find(&format!("for {} ", names[2])).unwrap() + 4,
            text.find(&format!("for {} being", names[3])).unwrap() + 4,
            proof_start
                + text[proof_start..]
                    .find(&format!("let {}", names[4]))
                    .unwrap()
                + 4,
            proof_start
                + text[proof_start..]
                    .find(&format!("let {} be", names[5]))
                    .unwrap()
                + 4,
        ];
        let expected_kinds = [
            BindingKind::DefinitionParameter,
            BindingKind::ReservedVariable,
            BindingKind::QuantifierBinder,
            BindingKind::QuantifierBinder,
            BindingKind::LetBinding,
            BindingKind::LetBinding,
        ];
        let actual = starts.map(|start| {
            bindings
                .bindings()
                .iter()
                .find(|(_, binding)| binding.declaration_range.start == start)
                .unwrap()
                .1
        });
        assert_eq!(
            actual
                .iter()
                .map(|binding| binding.id)
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
        for (index, binding) in actual.iter().enumerate() {
            assert_eq!(binding.spelling, names[index]);
            assert_eq!(binding.kind, expected_kinds[index]);
            assert_eq!(
                binding.declaration_range.end,
                starts[index] + names[index].len()
            );
            assert!(!matches!(
                binding.identity,
                BinderIdentity::Generated { .. }
            ));
            let declaration = declarations
                .declarations()
                .iter()
                .find(|(_, declaration)| declaration.binding == binding.id)
                .unwrap()
                .1;
            assert_eq!(declaration.status, DeclarationStatus::Checked);
            assert!(declaration.deferred.is_empty());
            assert_eq!(declaration.context, binding.owner_context);
            let TypeEntryActual::Known(ty) = declarations
                .type_entries()
                .get(declaration.type_entry.unwrap())
                .unwrap()
                .actual
            else {
                panic!("known declaration type")
            };
            let ty = declarations.normalized_types().get(ty).unwrap();
            assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
            assert_eq!(ty.status, NormalizedTypeStatus::Known);
            assert!(
                ty.args.is_empty()
                    && ty.attributes.positive().is_empty()
                    && ty.attributes.negative().is_empty()
            );
        }
        assert_ne!(actual[0].owner_context, actual[2].owner_context);
        assert_ne!(actual[2].owner_context, actual[4].owner_context);
        let mut argument_sites = Vec::new();
        for (use_index, (argument_binding, member_binding)) in
            [(actual[2], actual[3]), (actual[4], actual[5])]
                .into_iter()
                .enumerate()
        {
            let spelling = format!("MemberKind of {}", argument_binding.spelling);
            let application_start = if use_index == 0 {
                text.find(&spelling).unwrap()
            } else {
                proof_start + text[proof_start..].find(&spelling).unwrap()
            };
            let argument_start = application_start + "MemberKind of ".len();
            let argument = inferred.terms().iter().find(|(_, term)| matches!(typed.node(term.site.node()).unwrap().anchor, SourceAnchor::Range(range) if range.start == argument_start && range.end == argument_start + argument_binding.spelling.len())).unwrap().1;
            argument_sites.push(argument.site.clone());
            assert_eq!(
                argument.reference,
                Some(TermReference::Binding(argument_binding.id))
            );
            assert_eq!(argument.status, TermStatus::Inferred);
            assert!(argument.deferred.is_empty() && argument.candidate_set.is_none());
            assert_ne!(
                argument.reference,
                Some(TermReference::Binding(actual[0].id))
            );
            assert_ne!(
                argument.reference,
                Some(TermReference::Binding(actual[1].id))
            );
            let context = bindings.contexts().get(argument.context).unwrap();
            assert!(context.visible_bindings.contains(&argument_binding.id));
            assert!(!context.visible_bindings.contains(&actual[0].id));
            assert_eq!(
                bindings
                    .lookup(&BindingLookupSite::new(
                        &argument_binding.spelling,
                        argument.context,
                        context.lexical_scope.clone(),
                        argument_start
                    ))
                    .unwrap(),
                BindingLookupResult::Local(argument_binding.id)
            );
            let TypeEntryActual::Known(actual_type) = inferred
                .type_entries()
                .get(argument.type_entry)
                .unwrap()
                .actual
            else {
                panic!("known actual argument")
            };
            assert_eq!(
                inferred.normalized_types().get(actual_type).unwrap().head,
                TypeHeadRef::BuiltinSet
            );
            assert_eq!(
                inferred
                    .normalized_types()
                    .get(argument.expected_type.unwrap())
                    .unwrap()
                    .head,
                TypeHeadRef::BuiltinSet
            );
            let declaration = declarations
                .declarations()
                .iter()
                .find(|(_, declaration)| declaration.binding == member_binding.id)
                .unwrap()
                .1;
            let application_range = SourceRange {
                source_id: source.source_id(),
                start: application_start,
                end: application_start + spelling.len(),
            };
            assert_eq!(
                typed
                    .node(declaration.type_site.as_ref().unwrap().node())
                    .unwrap()
                    .anchor,
                SourceAnchor::Range(application_range)
            );
            assert_eq!(
                member_binding.type_site,
                BindingTypeSite::Source(application_range)
            );
        }
        assert_ne!(argument_sites[0], argument_sites[1]);
        for (_, formula) in inferred.formulas().iter() {
            assert_eq!(formula.kind, FormulaKind::Equality);
            assert_eq!(formula.status, FormulaStatus::Checked);
            assert!(formula.deferred.is_empty() && formula.facts.is_empty());
            let expected = if formula.source_range.start < proof_start {
                [
                    actual[3].id,
                    if nonreflexive {
                        actual[2].id
                    } else {
                        actual[3].id
                    },
                ]
            } else {
                [
                    actual[5].id,
                    if nonreflexive {
                        actual[4].id
                    } else {
                        actual[5].id
                    },
                ]
            };
            assert_eq!(formula.terms.len(), 2);
            for (site, binding) in formula.terms.iter().zip(expected) {
                let term = inferred
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site == *site)
                    .unwrap()
                    .1;
                assert_eq!(term.reference, Some(TermReference::Binding(binding)));
                assert_eq!(term.status, TermStatus::Inferred);
            }
        }
    }
}

#[test]
fn step5c4_dependent_mode_rejects_each_bad_argument_and_corrupt_source_association() {
    use mizar_checker::type_checker::check_source_dependent_mode_types;
    use mizar_checker::typed_ast::{NodeRecoveryState, TypedArena, TypingState};
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_mode_dependent_of_params_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let check = |text: &str| {
        let frontend = super::super::formula_statement::step5c8_test_frontend(text);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control must parse cleanly: {text}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, frontend)
                .unwrap_or_else(|error| {
                    panic!("semantic control must reach checker: {text}: {error}")
                });
        check_source_dependent_mode_types(&source, &typed, &symbols)
    };
    assert!(check(&text).is_ok());
    let uses = text
        .match_indices("MemberKind of A")
        .map(|(start, _)| start)
        .collect::<Vec<_>>();
    assert_eq!(uses.len(), 2);
    for start in &uses {
        for replacement in [
            "MemberKind of X",
            "MemberKind of Missing",
            "MemberKind of A, A",
            "MemberKind",
            "set",
        ] {
            let mut changed = text.clone();
            changed.replace_range(*start..*start + "MemberKind of A".len(), replacement);
            assert!(check(&changed).is_err(), "{changed}");
        }
    }
    for start in &uses {
        for replacement in ["MemberKind of", "MissingMode of A"] {
            let mut malformed = text.clone();
            malformed.replace_range(*start..*start + "MemberKind of A".len(), replacement);
            let frontend = super::super::formula_statement::step5c8_test_frontend(&malformed);
            assert!(!frontend.diagnostics.is_empty(), "{malformed}");
            assert!(
                super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err()
            );
        }
    }
    for (from, to) in [
        ("for A for m", "for A being object for m"),
        ("let A;", "let A be object;"),
        ("reserve A for set", "reserve A for object"),
    ] {
        let changed = text.replace(from, to);
        assert!(check(&changed).is_err(), "{changed}");
    }
    for (from, to) in [
        ("let X be set", "let X be object"),
        ("of X is set", "of A is set"),
        ("of X is set", "of X is object"),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        assert!(check(&changed).is_err(), "{changed}");
    }
    for replacement in ["of X is non empty set", "of X is MissingMode"] {
        let malformed = text.replace("of X is set", replacement);
        let frontend = super::super::formula_statement::step5c8_test_frontend(&malformed);
        assert!(!frontend.diagnostics.is_empty(), "{malformed}");
        assert!(super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    }
    let (definition, rest) = text.split_once("\n\nreserve").unwrap();
    let forward =
        super::super::formula_statement::step5c8_test_frontend(&format!("reserve{rest}\n{definition}"));
    assert!(!forward.diagnostics.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, case, forward).is_err());
    let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
    let ast = frontend.ast.clone().unwrap();
    for start in &uses {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        let mut changed_tokens = 0;
        for node in ast.nodes() {
            let children = node.children.iter().map(|id| rebuilt[id.index()]).collect();
            let id = match &node.kind {
                SurfaceNodeKind::Token(token) => {
                    let spelling = if node.range.start == *start {
                        assert_eq!(token.text.as_ref(), "MemberKind");
                        assert_eq!(node.range.end - node.range.start, "OtherKindX".len());
                        changed_tokens += 1;
                        "OtherKindX".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                kind => builder.add_node(kind.clone(), node.range, children),
            };
            rebuilt.push(id);
        }
        assert_eq!(changed_tokens, 1);
        let mut changed_frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(changed_frontend.diagnostics.is_empty());
        changed_frontend.ast =
            Some(builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None));
        let (changed_source, changed_typed, changed_symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, changed_frontend)
                .unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&changed_source, &changed_symbols)
            .unwrap();
        assert!(
            check_source_dependent_mode_types(&changed_source, &changed_typed, &changed_symbols)
                .is_err()
        );
    }
    let (source, typed, symbols) =
        super::super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
    let argument_nodes = uses.iter().map(|start| typed.iter().find(|(_, node)| node.kind.as_str() == "TermReference" && matches!(node.anchor, SourceAnchor::Range(range) if range.start == start + "MemberKind of ".len())).unwrap().0).collect::<Vec<_>>();
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for mutation in 0..7 {
        let mut nodes = raw.clone();
        let first = argument_nodes[0].index();
        let second = argument_nodes[1].index();
        match mutation {
            0 => nodes[first].resolved_node = nodes[second].resolved_node,
            1 => nodes[second].anchor = nodes[first].anchor.clone(),
            2 => nodes[first].kind = "TypeExpression".into(),
            3 => nodes[second].recovery = NodeRecoveryState::Recovered,
            4 => nodes[first].typing = TypingState::Successful,
            5 => nodes[first].children.clear(),
            6 => nodes[second].children = nodes[first].children.clone(),
            _ => unreachable!(),
        }
        assert_ne!(nodes, raw);
        let changed = TypedArena::try_new(typed.root(), nodes).unwrap();
        assert!(
            check_source_dependent_mode_types(&source, &changed, &symbols).is_err(),
            "typed mutation {mutation}"
        );
    }
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("dependent"));
    let foreign = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
    assert!(check_source_dependent_mode_types(&foreign, &typed, &symbols).is_err());
    let mut foreign_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    foreign_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    let foreign_source = SurfaceResolvedArena::lower(&foreign_ast, symbols.module_id()).unwrap();
    assert!(check_source_dependent_mode_types(&foreign_source, &typed, &symbols).is_err());
    let stale =
        super::super::formula_statement::step5c8_test_frontend(&text.replace("MemberKind", "OtherKind"))
            .ast
            .unwrap();
    let wrong = super::super::resolver_symbol_collection(&config.workspace_root, case, &stale);
    assert!(check_source_dependent_mode_types(&source, &typed, &wrong.env).is_err());
    let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        check_source_dependent_mode_types(
            &source,
            &typed,
            &SymbolEnv::new(symbols.module_id().clone(), indexes)
        )
        .is_err()
    );
}
