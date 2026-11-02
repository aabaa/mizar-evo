use super::*;

#[test]
fn step5c6_functor_synonym_types_preserve_original_root_locus_order_and_eight_binders() {
    use mizar_checker::overload_resolution::{CandidateViabilityStatus, OverloadSiteKind};
    use mizar_checker::type_checker::{TypeHeadRef, check_source_functor_synonym_types};
    use mizar_checker::typed_ast::{TypeEntryActual, TypedNodeId, TypedSiteRef};
    use mizar_resolve::{
        env::{RelationKind, SymbolKind},
        names::resolve_template_formal,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_synonym_functor_001")
        .unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let blocks = text.split("\n\n").collect::<Vec<_>>();
    let (header, proof) = blocks[2].split_once("proof").unwrap();
    let renamed = format!(
        "{}\n\n{}\n\n{}proof{}",
        blocks[0].replace("X", "BaseL").replace("Y", "BaseR"),
        blocks[1].replace("X", "AliasL").replace("Y", "AliasR"),
        header.replace("X", "HeaderL").replace("Y", "HeaderR"),
        proof.replace("X", "LocalL").replace("Y", "LocalR")
    )
    .replace("synbase", "original")
    .replace("synalt", "alternate")
    .replace("SynBaseDef", "OriginalDef")
    .replace("SynUse1", "UseAlias");
    for (variant, permutation, actual_order) in [
        (text.clone(), [0, 1], [[0, 1], [0, 1]]),
        (renamed.clone(), [0, 1], [[0, 1], [0, 1]]),
        (
            renamed.replace(
                "synonym AliasL alternate AliasR",
                "synonym AliasR alternate AliasL",
            ),
            [1, 0],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("synonym X synalt Y", "synonym Y synalt X"),
            [1, 0],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("for X synbase Y", "for Y synbase X"),
            [1, 0],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("holds X synalt Y", "holds Y synalt X"),
            [0, 1],
            [[1, 0], [0, 1]],
        ),
        (
            text.replace("thus X synalt Y", "thus Y synalt X"),
            [0, 1],
            [[0, 1], [1, 0]],
        ),
        (
            text.replace("holds X synalt Y", "holds X synalt X")
                .replace("thus X synalt Y", "thus X synalt X"),
            [0, 1],
            [[0, 0], [0, 0]],
        ),
        (
            text.replace("equals X;", "equals Y;"),
            [0, 1],
            [[0, 1], [0, 1]],
        ),
        (
            text.replace("= X\nproof", "= Y\nproof")
                .replace("= X by", "= Y by"),
            [0, 1],
            [[0, 1], [0, 1]],
        ),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        let output = check_source_functor_synonym_types(
            &source,
            &symbols,
            &typed,
            mizar_resolve::env::RelationKind::Synonym,
        )
        .unwrap();
        assert_eq!(
            output,
            check_source_functor_synonym_types(
                &source,
                &symbols,
                &typed,
                mizar_resolve::env::RelationKind::Synonym
            )
            .unwrap()
        );
        let (normalization, collection, viability) = output;
        assert!(normalization.diagnostics().is_empty());
        assert!(collection.diagnostics().is_empty());
        assert!(viability.diagnostics().is_empty());
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 2);
        assert_eq!(viability.decisions().len(), 2);
        let original = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Functor)
            .unwrap();
        let alias = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Synonym)
            .unwrap();
        assert_ne!(original.symbol(), alias.symbol());
        assert_eq!(alias.relations().len(), 1);
        assert_eq!(alias.relations()[0].kind(), RelationKind::Synonym);
        assert_eq!(alias.relations()[0].target(), original.symbol());
        let mut groups = source
            .arena()
            .iter()
            .filter(|(_, node)| {
                matches!(
                    node.kind(),
                    K::QualifiedVariableSegment | K::QuantifierVariableSegment
                )
            })
            .map(|(_, node)| [node.children()[0], node.children()[2]])
            .collect::<Vec<_>>();
        groups.sort_by_key(|group| {
            match source.arena().node(group[0]).unwrap().origin().anchor() {
                SourceAnchor::Range(range) => range.start,
                _ => unreachable!(),
            }
        });
        assert_eq!(groups.len(), 4);
        let identities = groups
            .iter()
            .flatten()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(identities.len(), 8);
        let (_, original_pattern) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &K::FunctorPattern)
            .unwrap();
        assert_eq!(
            [
                resolve_template_formal(&source, original_pattern.children()[0]).unwrap(),
                resolve_template_formal(&source, original_pattern.children()[2]).unwrap()
            ],
            groups[0]
        );
        for (_, pattern) in source
            .arena()
            .iter()
            .filter(|(_, node)| node.kind() == &K::NotationPattern)
        {
            let binding_pair = [
                resolve_template_formal(&source, pattern.children()[0]).unwrap(),
                resolve_template_formal(&source, pattern.children()[2]).unwrap(),
            ];
            assert_ne!(binding_pair[0], binding_pair[1]);
            assert!(binding_pair.iter().all(|id| groups[1].contains(id)));
        }
        let (_, definition) = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &K::FunctorDefinition)
            .unwrap();
        let return_site = TypedSiteRef::Node(TypedNodeId::new(definition.children()[5].index()));
        let declared_result = normalization
            .type_entries()
            .iter()
            .find(|(_, entry)| entry.owner == return_site)
            .unwrap()
            .1
            .actual;
        let mut occurrences = Vec::new();
        for (index, (site_id, site)) in collection.sites().iter().enumerate() {
            assert_eq!(site.kind, OverloadSiteKind::FunctorApplication);
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
            assert!(matches!(application.kind(), K::InfixExpression(_)));
            assert_eq!(
                SourceAnchor::Range(site.source_range),
                *application.origin().anchor()
            );
            let written = [application.children()[0], application.children()[2]];
            assert_ne!(written[0], written[1]);
            assert_eq!(
                site.arguments,
                permutation.map(|slot| TypedSiteRef::Node(TypedNodeId::new(written[slot].index())))
            );
            for (slot, reference) in written.iter().enumerate() {
                let node = source.arena().node(*reference).unwrap();
                assert_eq!(node.kind(), &K::TermReference);
                assert_eq!(
                    resolve_template_formal(&source, node.children()[0]).unwrap(),
                    groups[index + 2][actual_order[index][slot]]
                );
                occurrences.push(*reference);
            }
            let (_, candidate) = collection
                .candidates()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(&candidate.symbol, original.symbol());
            assert_eq!(candidate.ordinary_root, candidate.symbol);
            assert_eq!(
                SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                *original.origin().anchor()
            );
            assert_eq!(
                declared_result,
                TypeEntryActual::Known(candidate.result.unwrap())
            );
            assert_eq!(candidate.parameters.len(), 2);
            for ty in candidate.parameters.iter().chain(candidate.result.iter()) {
                assert_eq!(
                    normalization.normalized_types().get(*ty).unwrap().head,
                    TypeHeadRef::BuiltinSet
                );
            }
            let (_, decision) = viability
                .decisions()
                .iter()
                .find(|(_, row)| row.site == site_id)
                .unwrap();
            assert_eq!(decision.source_candidate, candidate.id);
            let CandidateViabilityStatus::Viable { views } = &decision.status else {
                panic!("{:?}", decision.status)
            };
            assert_eq!(
                views
                    .iter()
                    .map(|view| view.argument_index)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            assert!(views.iter().all(|view| view.actual
                == candidate.parameters[view.argument_index]
                && view.target == view.actual
                && view.facts.is_empty()
                && view.coercion.is_none()));
        }
        assert_eq!(
            occurrences
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
    }
}

#[test]
fn step5c6_functor_synonym_types_reject_unrelated_failures_and_forged_relations() {
    use mizar_checker::type_checker::check_source_functor_synonym_types;
    use mizar_checker::typed_ast::{TypedArena, TypingState};
    use mizar_resolve::{
        env::{RelationKind, RelationMetadata, SymbolIndex, SymbolKind},
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_syntax::ast::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_type_elaboration_synonym_functor_001")
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
        ("equals X;", "equals Missing;"),
        ("equals X;", "equals (X);"),
        ("-> set", "-> object"),
        ("being set", "being object"),
        ("let X, Y be set;\n  thus", "let X, Y be object;\n  thus"),
        ("holds X synalt Y", "holds Missing synalt Y"),
        ("thus X synalt Y", "thus X synalt Missing"),
        ("holds X synalt Y = X", "holds X synalt Y = Missing"),
        ("= X by", "= Missing by"),
        ("by SynBaseDef", "by SynUse1"),
        ("coherence;", "existence;"),
        (
            "synonym X synalt Y for X synbase Y",
            "synonym X synalt X for X synbase Y",
        ),
    ] {
        let changed = text.replace(from, to);
        assert_ne!(changed, text);
        let (source, typed, symbols) = inputs(&changed);
        assert!(
            check_source_functor_synonym_types(
                &source,
                &symbols,
                &typed,
                mizar_resolve::env::RelationKind::Synonym
            )
            .is_err(),
            "{changed}"
        );
    }
    let (source, typed, symbols) = inputs(&text);
    let original = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Functor)
        .unwrap();
    let alias = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Synonym)
        .unwrap();
    for relations in [
        vec![],
        vec![RelationMetadata::new(
            RelationKind::Antonym,
            original.symbol().clone(),
        )],
        vec![RelationMetadata::new(
            RelationKind::Synonym,
            alias.symbol().clone(),
        )],
    ] {
        let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
        indexes.symbols = SymbolIndex::new();
        for entry in symbols.symbols().iter() {
            indexes.symbols.insert(if entry.symbol() == alias.symbol() {
                entry.clone().with_relations(relations.clone())
            } else {
                entry.clone()
            });
        }
        assert!(
            check_source_functor_synonym_types(
                &source,
                &SymbolEnv::new(symbols.module_id().clone(), indexes),
                &typed,
                mizar_resolve::env::RelationKind::Synonym
            )
            .is_err()
        );
    }
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let calls = typed
        .iter()
        .filter(|(_, node)| node.kind.as_str().starts_with("InfixExpression("))
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    for call in &calls {
        for argument in [raw[call.index()].children[0], raw[call.index()].children[2]] {
            for mutation in 0..3 {
                let mut nodes = raw.clone();
                match mutation {
                    0 => nodes[argument.index()].resolved_node = nodes[call.index()].resolved_node,
                    1 => nodes[argument.index()].anchor = nodes[call.index()].anchor.clone(),
                    2 => nodes[argument.index()].typing = TypingState::Successful,
                    _ => unreachable!(),
                }
                assert!(
                    check_source_functor_synonym_types(
                        &source,
                        &symbols,
                        &TypedArena::try_new(typed.root(), nodes).unwrap(),
                        mizar_resolve::env::RelationKind::Synonym
                    )
                    .is_err()
                );
            }
        }
    }
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let head_offsets = calls
        .iter()
        .map(|id| match raw[raw[id.index()].children[1].index()].anchor {
            SourceAnchor::Range(range) => range.start,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    for target in head_offsets.into_iter().map(Some).chain([None]) {
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
                        assert_eq!(token.text.as_ref(), "synalt");
                        changed += 1;
                        "absent".into()
                    } else {
                        token.text.clone()
                    };
                    builder.add_token(token.kind, spelling, node.range)
                }
                K::InfixExpression(operator)
                    if node
                        .children
                        .get(1)
                        .is_some_and(|id| Some(ast.node(*id).unwrap().range.start) == target) =>
                {
                    let mut operator = operator.clone();
                    operator.spelling = "absent".into();
                    builder.add_node(K::InfixExpression(operator), node.range, children)
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
        assert!(
            check_source_functor_synonym_types(
                &changed_source,
                &changed_symbols,
                &changed_typed,
                mizar_resolve::env::RelationKind::Synonym
            )
            .is_err()
        );
    }
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("synonym"));
    assert!(
        check_source_functor_synonym_types(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed,
            mizar_resolve::env::RelationKind::Synonym
        )
        .is_err()
    );
    let mut other_ast = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    other_ast.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    assert!(
        check_source_functor_synonym_types(
            &SurfaceResolvedArena::lower(&other_ast, symbols.module_id()).unwrap(),
            &symbols,
            &typed,
            mizar_resolve::env::RelationKind::Synonym
        )
        .is_err()
    );
    let (_, _, other_symbols) = inputs(&text.replace("synalt", "otheralias"));
    assert!(
        check_source_functor_synonym_types(
            &source,
            &other_symbols,
            &typed,
            mizar_resolve::env::RelationKind::Synonym
        )
        .is_err()
    );
    let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.definitions = Default::default();
    assert!(
        check_source_functor_synonym_types(
            &source,
            &SymbolEnv::new(symbols.module_id().clone(), indexes),
            &typed,
            mizar_resolve::env::RelationKind::Synonym
        )
        .is_err()
    );
    let theorem = text.find("theorem SynUse1:").unwrap();
    let frontend =
        super::super::formula_statement::step5c8_test_frontend(&format!("{text}\n{}", &text[theorem..]));
    assert!(frontend.diagnostics.is_empty());
    assert!(
        !super::super::resolver_symbol_collection(
            &config.workspace_root,
            case,
            frontend.ast.as_ref().unwrap()
        )
        .detail_keys
        .is_empty()
    );
    assert!(super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
    let frontend = super::super::formula_statement::step5c8_test_frontend(&text.replace("= X by", "= by"));
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, case, frontend).is_err());
}

#[test]
fn step5c6_alias_relation_selects_actual_target_and_rejects_unsupported_sources() {
    use mizar_checker::type_checker::check_source_functor_synonym_types;
    use mizar_resolve::env::{RelationKind, SymbolKind};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    for (id, original_kind, alias_kind, profile, wrong_kind) in [
        (
            "pass_type_elaboration_synonym_functor_001",
            SymbolKind::Functor,
            SymbolKind::Synonym,
            RelationKind::Synonym,
            "definition\n let X, Y be set;\n pred SynBaseDef: X synbase Y means X = Y;\nend;",
        ),
        (
            "pass_type_elaboration_antonym_predicate_001",
            SymbolKind::Predicate,
            SymbolKind::Antonym,
            RelationKind::Antonym,
            "definition\n let X, Y be set;\n func SynBaseDef: X synbase Y -> set equals X;\n coherence;\nend;",
        ),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path)
            .unwrap()
            .replace("ApartDef", "SynBaseDef")
            .replace("apartfrom", "synbase")
            .replace("closeto", "synalt");
        let blocks = text.split("\n\n").collect::<Vec<_>>();
        let second = blocks[0]
            .replace("SynBaseDef", "SecondDef")
            .replace("synbase", "secondbase");
        let two_targets = format!(
            "{}\n\n{}\n\n{}\n\n{}",
            blocks[0],
            second,
            blocks[1].replace("synbase", "secondbase"),
            blocks[2]
        );
        let frontend = super::super::formula_statement::step5c8_test_frontend(&two_targets);
        assert!(
            frontend.diagnostics.is_empty(),
            "{:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, frontend).unwrap();
        let mut originals = symbols
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == original_kind)
            .collect::<Vec<_>>();
        originals.sort_by_key(|entry| match entry.origin().anchor() {
            SourceAnchor::Range(range) => range.start,
            _ => unreachable!(),
        });
        assert_eq!(originals.len(), 2);
        assert_ne!(originals[0].symbol(), originals[1].symbol());
        let alias = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == alias_kind)
            .unwrap();
        assert_eq!(alias.relations().len(), 1);
        assert_eq!(alias.relations()[0].kind(), profile);
        assert_eq!(alias.relations()[0].target(), originals[1].symbol());
        assert!(check_source_functor_synonym_types(&source, &symbols, &typed, profile).is_err());
        for variant in [
            format!(
                "{}\n{}\n{}",
                blocks[0],
                blocks[0].replace("SynBaseDef", "DuplicateDef"),
                blocks[1]
            ),
            format!(
                "{}\n{}",
                blocks[0],
                blocks[1].replace("for X synbase Y", "for X missing Y")
            ),
            format!("{}\n{}", blocks[1], blocks[0]),
            format!("{wrong_kind}\n{}", blocks[1]),
            format!(
                "{}\n{}\n{}",
                blocks[0],
                blocks[1],
                blocks[1]
                    .replace("synalt", "thirdname")
                    .replace("synbase", "synalt")
            ),
            format!(
                "{}\n{}",
                blocks[0],
                blocks[1]
                    .replace("X, Y be set", "X, Y, Z be set")
                    .replace("for X synbase Y", "for X synbase Z")
            ),
            format!(
                "{}\n{}",
                blocks[0],
                blocks[1]
                    .replace("X, Y be set", "X, Y, Z be set")
                    .replace("X synalt Y", "synalt(X, Y, Z)")
            ),
        ]
        .into_iter()
        .chain((profile == RelationKind::Antonym).then(|| {
            format!(
                "import parser.type_fixtures;\n{}",
                blocks[1]
                    .replace("be set", "be object")
                    .replace("synbase", "divides")
            )
        })) {
            let frontend = super::super::formula_statement::step5c8_test_frontend(&variant);
            assert!(
                frontend.diagnostics.is_empty(),
                "{variant}: {:?}",
                frontend.diagnostics
            );
            let result = super::super::resolver_symbol_collection(
                &config.workspace_root,
                case,
                frontend.ast.as_ref().unwrap(),
            );
            if profile == RelationKind::Antonym {
                assert!(result.detail_keys.iter().all(|key| {
                    key != "declaration_symbol.notation.synonym_loci_mismatch"
                }));
            }
            if variant.starts_with("import ") {
                let imported = super::super::import_fixtures::augment_type_elaboration_import_summaries(
                    frontend.ast.as_ref().unwrap(),
                    &result.module,
                    result.env.clone(),
                );
                assert!(imported.symbols().iter().any(|entry| {
                    entry.kind() == SymbolKind::Predicate
                        && entry.origin().module_id().path().as_str() == "parser.type_fixtures"
                }));
            }
            let last_alias = result
                .env
                .symbols()
                .iter()
                .filter(|entry| entry.kind() == alias_kind)
                .max_by_key(|entry| match entry.origin().anchor() {
                    SourceAnchor::Range(range) => range.start,
                    _ => unreachable!(),
                })
                .unwrap();
            assert!(
                last_alias.relations().is_empty(),
                "{variant}: {:?}",
                last_alias.relations()
            );
        }
    }
}

#[test]
fn step5c6_antonym_preserves_inversion_real_bindings_and_result_free_ordered_calls() {
    use mizar_checker::overload_resolution::{
        CandidateDeclarationKind, CandidateViabilityStatus, OverloadSiteKind,
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, TypeHeadRef, check_source_functor_synonym_types,
    };
    use mizar_checker::typed_ast::{TypedNodeId, TypedSiteRef};
    use mizar_resolve::{
        env::{RelationKind, SymbolKind},
        names::resolve_template_formal,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let case = step5c6_antonym_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let grouped = text
        .replace(
            "for X being set holds X closeto X",
            "for A, B being set holds A closeto B",
        )
        .replace(
            "let X be set;\n  thus X closeto X",
            "let C, D be set;\n  thus C closeto D",
        );
    let blocks = text.split("\n\n").collect::<Vec<_>>();
    let (header, proof) = blocks[2].split_once("proof").unwrap();
    let renamed = format!(
        "{}\n\n{}\n\n{}proof{}",
        blocks[0].replace("X", "BaseL").replace("Y", "BaseR"),
        blocks[1].replace("X", "AliasL").replace("Y", "AliasR"),
        header.replace("X", "Header"),
        proof.replace("X", "Local")
    )
    .replace("ApartDef", "OriginalDef")
    .replace("apartfrom", "original")
    .replace("closeto", "alternate")
    .replace("AntUse1", "UseAlias");
    for (variant, permutation, binding_count, body_same) in [
        (text.clone(), [0, 1], 6, false),
        (renamed, [0, 1], 6, false),
        (grouped.clone(), [0, 1], 8, false),
        (
            grouped.replace("antonym X closeto Y", "antonym Y closeto X"),
            [1, 0],
            8,
            false,
        ),
        (
            grouped.replace("for X apartfrom Y", "for Y apartfrom X"),
            [1, 0],
            8,
            false,
        ),
        (text.replace("not X = Y", "not X = X"), [0, 1], 6, true),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "{variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        let output =
            check_source_functor_synonym_types(&source, &symbols, &typed, RelationKind::Antonym)
                .unwrap();
        assert_eq!(
            output,
            check_source_functor_synonym_types(&source, &symbols, &typed, RelationKind::Antonym)
                .unwrap()
        );
        for profile in [RelationKind::Synonym, RelationKind::Redefinition] {
            assert!(
                check_source_functor_synonym_types(&source, &symbols, &typed, profile).is_err()
            );
        }
        let (normalization, collection, viability) = output;
        assert!(normalization.diagnostics().is_empty());
        assert!(collection.diagnostics().is_empty());
        assert!(viability.diagnostics().is_empty());
        assert_eq!(collection.sites().len(), 2);
        assert_eq!(collection.candidates().len(), 2);
        assert_eq!(viability.decisions().len(), 2);
        let original = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Predicate)
            .unwrap();
        let alias = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == SymbolKind::Antonym)
            .unwrap();
        assert_ne!(original.symbol(), alias.symbol());
        assert_eq!(alias.relations().len(), 1);
        assert_eq!(alias.relations()[0].kind(), RelationKind::Antonym);
        assert_eq!(alias.relations()[0].target(), original.symbol());
        let mut groups = source.arena().iter().filter(|(_, node)| matches!(node.kind(), K::QualifiedVariableSegment | K::QuantifierVariableSegment))
            .map(|(_, node)| node.children().iter().copied().filter(|id| matches!(source.arena().node(*id).unwrap().kind(), K::Token(token) if token.kind == mizar_syntax::SurfaceTokenKind::Identifier)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        groups.sort_by_key(|group| {
            match source.arena().node(group[0]).unwrap().origin().anchor() {
                SourceAnchor::Range(range) => range.start,
                _ => unreachable!(),
            }
        });
        assert_eq!(groups.len(), 4);
        assert_eq!(
            groups
                .iter()
                .flatten()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            binding_count
        );
        let equality = source
            .arena()
            .iter()
            .find(|(_, node)| node.kind() == &K::BuiltinPredicateApplication)
            .unwrap()
            .1;
        let body = [equality.children()[0], equality.children()[2]]
            .map(|id| source.arena().node(id).unwrap().children()[0]);
        assert_ne!(body[0], body[1]);
        let body_binders = body.map(|id| {
            resolve_template_formal(&source, source.arena().node(id).unwrap().children()[0])
                .unwrap()
        });
        assert_eq!(
            body_binders,
            [groups[0][0], groups[0][usize::from(!body_same)]]
        );
        assert!(source.arena().iter().any(
            |(_, node)| matches!(node.kind(), K::Token(token) if token.text.as_ref() == "not")
        ));
        let mut occurrences = Vec::new();
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
            let [segment] = application.children() else {
                panic!("one actual segment required")
            };
            let segment = source.arena().node(*segment).unwrap();
            assert_eq!(segment.kind(), &K::PredicateSegment);
            let written = [segment.children()[0], segment.children()[2]]
                .map(|id| source.arena().node(id).unwrap().children()[0]);
            assert_ne!(written[0], written[1]);
            assert_eq!(
                site.arguments,
                permutation.map(|slot| TypedSiteRef::Node(TypedNodeId::new(written[slot].index())))
            );
            for (slot, id) in written.iter().enumerate() {
                let reference = source.arena().node(*id).unwrap();
                assert_eq!(reference.kind(), &K::TermReference);
                assert_eq!(
                    resolve_template_formal(&source, reference.children()[0]).unwrap(),
                    groups[index + 2][slot.min(groups[index + 2].len() - 1)]
                );
                occurrences.push(*id);
            }
            let (_, candidate) = collection
                .candidates()
                .iter()
                .find(|(_, candidate)| candidate.site == site_id)
                .unwrap();
            assert_eq!(
                candidate.declaration_kind,
                CandidateDeclarationKind::Predicate
            );
            assert_eq!(&candidate.symbol, original.symbol());
            assert_eq!(candidate.ordinary_root, candidate.symbol);
            assert_eq!(
                SourceAnchor::Range(candidate.provenance.source_range.unwrap()),
                *original.origin().anchor()
            );
            assert!(candidate.result.is_none());
            assert_eq!(candidate.parameters.len(), 2);
            for ty in &candidate.parameters {
                let ty = normalization.normalized_types().get(*ty).unwrap();
                assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
                assert_eq!(ty.status, NormalizedTypeStatus::Known);
            }
            let (_, decision) = viability
                .decisions()
                .iter()
                .find(|(_, decision)| decision.site == site_id)
                .unwrap();
            assert_eq!(decision.source_candidate, candidate.id);
            let CandidateViabilityStatus::Viable { views } = &decision.status else {
                panic!("{:?}", decision.status)
            };
            assert_eq!(views.len(), 2);
            for (slot, view) in views.iter().enumerate() {
                assert_eq!(view.argument_index, slot);
                assert_eq!(view.actual, candidate.parameters[slot]);
                assert_eq!(view.target, view.actual);
                assert!(view.facts.is_empty());
                assert!(view.coercion.is_none());
            }
        }
        assert_eq!(
            occurrences
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
    }
}

#[test]
fn step5c6_antonym_rejects_unsupported_source_profiles_and_lower_stage_errors() {
    use mizar_checker::type_checker::check_source_functor_synonym_types;
    use mizar_resolve::env::RelationKind;
    let config = step5c11_config();
    let case = step5c6_antonym_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    for (from, to) in [
        ("not X = Y", "X = Y"),
        ("not X = Y", "not Missing = Y"),
        ("not X = Y", "not X = Missing"),
        ("holds X closeto X", "holds Missing closeto X"),
        ("holds X closeto X", "holds X closeto Missing"),
        ("thus X closeto X", "thus Missing closeto X"),
        ("thus X closeto X", "thus X closeto Missing"),
        ("being set", "being object"),
        ("let X be set", "let X be object"),
        ("antonym X closeto Y", "antonym X closeto X"),
        ("for X apartfrom Y", "for X apartfrom X"),
        ("holds X closeto X", "holds X does not closeto X"),
        ("thus X closeto X", "thus X does not closeto X"),
        ("holds X closeto X", "holds X closeto X closeto X"),
        ("thus X closeto X", "thus X closeto X closeto X"),
    ] {
        let variant = text.replace(from, to);
        assert_ne!(variant, text);
        let frontend = super::super::formula_statement::step5c8_test_frontend(&variant);
        assert!(
            frontend.diagnostics.is_empty(),
            "semantic control must parse: {variant}: {:?}",
            frontend.diagnostics
        );
        let (source, typed, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        assert!(
            check_source_functor_synonym_types(&source, &symbols, &typed, RelationKind::Antonym)
                .is_err(),
            "{variant}"
        );
    }
    let duplicate = format!("{text}\n{}", text.split("\n\n").next().unwrap());
    let frontend = super::super::formula_statement::step5c8_test_frontend(&duplicate);
    assert!(frontend.diagnostics.is_empty());
    assert!(
        !super::super::resolver_symbol_collection(
            &config.workspace_root,
            &case,
            frontend.ast.as_ref().unwrap()
        )
        .detail_keys
        .is_empty()
    );
    assert!(super::super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
    let frontend =
        super::super::formula_statement::step5c8_test_frontend(&text.replace("not X = Y", "not X ="));
    assert!(!frontend.diagnostics.is_empty());
    assert!(super::super::source_registration_inputs(&config.workspace_root, &case, frontend).is_err());
}

#[test]
fn step5c6_antonym_authenticates_relations_source_environment_and_both_calls() {
    use mizar_checker::type_checker::check_source_functor_synonym_types;
    use mizar_checker::typed_ast::{TypedArena, TypedNodeKind, TypingState};
    use mizar_resolve::{
        env::{RelationKind, RelationMetadata, SymbolIndex, SymbolKind},
        resolved_ast::SurfaceResolvedArena,
    };
    let config = step5c11_config();
    let case = step5c6_antonym_case();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let inputs = |text: &str| {
        let frontend = super::super::formula_statement::step5c8_test_frontend(text);
        assert!(frontend.diagnostics.is_empty());
        super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap()
    };
    let (source, typed, symbols) = inputs(&text);
    let original = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Predicate)
        .unwrap();
    let alias = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Antonym)
        .unwrap();
    for relations in [
        vec![],
        vec![RelationMetadata::new(
            RelationKind::Synonym,
            original.symbol().clone(),
        )],
        vec![RelationMetadata::new(
            RelationKind::Antonym,
            alias.symbol().clone(),
        )],
    ] {
        let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
        indexes.symbols = SymbolIndex::new();
        for entry in symbols.symbols().iter() {
            indexes.symbols.insert(if entry.symbol() == alias.symbol() {
                entry.clone().with_relations(relations.clone())
            } else {
                entry.clone()
            });
        }
        assert!(
            check_source_functor_synonym_types(
                &source,
                &SymbolEnv::new(symbols.module_id().clone(), indexes),
                &typed,
                RelationKind::Antonym
            )
            .is_err()
        );
    }
    for part in 0..3 {
        let mut indexes = super::super::import_fixtures::clone_symbol_env_indexes(&symbols);
        match part {
            0 => indexes.symbols = Default::default(),
            1 => indexes.definitions = Default::default(),
            2 => indexes.contributions = Default::default(),
            _ => unreachable!(),
        }
        assert!(
            check_source_functor_synonym_types(
                &source,
                &SymbolEnv::new(symbols.module_id().clone(), indexes),
                &typed,
                RelationKind::Antonym
            )
            .is_err()
        );
    }
    let (_, _, foreign_symbols) = inputs(&text.replace("closeto", "otheralias"));
    assert!(
        check_source_functor_synonym_types(
            &source,
            &foreign_symbols,
            &typed,
            RelationKind::Antonym
        )
        .is_err()
    );
    let ast = super::super::formula_statement::step5c8_test_frontend(&text)
        .ast
        .unwrap();
    let foreign = ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("antonym"));
    assert!(
        check_source_functor_synonym_types(
            &SurfaceResolvedArena::lower(&ast, &foreign).unwrap(),
            &symbols,
            &typed,
            RelationKind::Antonym
        )
        .is_err()
    );
    let raw = typed
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let refs = typed
        .iter()
        .filter(|(_, node)| node.kind.as_str() == "TermReference")
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(refs.len(), 6);
    for id in refs {
        for mutation in 0..3 {
            let mut nodes = raw.clone();
            match mutation {
                0 => {
                    nodes[id.index()].resolved_node =
                        typed.node(typed.root().unwrap()).unwrap().resolved_node
                }
                1 => nodes[id.index()].typing = TypingState::Successful,
                2 => nodes[id.index()].kind = TypedNodeKind::new("Forged"),
                _ => unreachable!(),
            }
            assert!(
                check_source_functor_synonym_types(
                    &source,
                    &symbols,
                    &TypedArena::try_new(typed.root(), nodes).unwrap(),
                    RelationKind::Antonym
                )
                .is_err()
            );
        }
    }
}
