use super::*;
use crate::declarations::DeclarationShellCollector;
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
};
use mizar_syntax::{
    SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind, SyntaxRecoveryKind,
};

#[test]
fn registers_opaque_symbols_definitions_and_contribution_effects() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::PredicateDefinition),
            test_item(10, SurfaceNodeKind::FunctorDefinition),
            visible_test_item(20, "private", SurfaceNodeKind::ModeDefinition),
        ],
    );
    let namespace = NamespacePath::new("main");
    let projections = vec![
        projection(
            shells.declarations()[0].id(),
            namespace.clone(),
            "P",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        ),
        projection(
            shells.declarations()[1].id(),
            namespace.clone(),
            "F",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("_ + _")
        .with_arity(2),
        projection(
            shells.declarations()[2].id(),
            namespace.clone(),
            "M",
            SymbolKind::Mode,
            DefinitionKind::Mode,
        ),
    ];

    let result = collect(source_id, &shells, &projections);
    let env = result.env();

    assert!(result.diagnostics().is_empty());
    assert_eq!(env.symbols().len(), 3);
    assert_eq!(env.definitions().len(), 3);
    assert_eq!(env.symbols().visible_candidates(&namespace, "P").len(), 1);
    assert_eq!(
        env.symbols().visible_candidates(&namespace, "F")[0].notation_spelling(),
        Some("_ + _")
    );
    assert_eq!(
        env.symbols().visible_candidates(&namespace, "M")[0].visibility(),
        Visibility::Private
    );
    assert_eq!(
        env.symbols().visible_candidates(&namespace, "M")[0].export_status(),
        ExportStatus::LocalOnly
    );
    let effects = env.contributions().iter().next().unwrap().effects();
    assert_eq!(effects.symbols().len(), 3);
    assert_eq!(effects.definitions().len(), 3);
}

#[test]
fn duplicate_detection_marks_represented_kind_families_in_order() {
    let source_id = source_id();
    let cases = duplicate_cases();
    let shells = shells_for(source_id, duplicate_case_items(&cases));
    let namespace = NamespacePath::new("main");
    let mut projections = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let first = shells.declarations()[index * 2].id();
        let second = shells.declarations()[index * 2 + 1].id();
        projections.push(case.projection(second, namespace.clone()));
        projections.push(case.projection(first, namespace.clone()));
    }

    let result = collect(source_id, &shells, &projections);

    assert_eq!(result.diagnostics().len(), cases.len());
    for (diagnostic, case) in result.diagnostics().iter().zip(cases.iter()) {
        assert_eq!(
            diagnostic.class(),
            SymbolDiagnosticClass::DuplicateDeclaration
        );
        assert_eq!(diagnostic.spelling(), case.spelling);
        assert_eq!(diagnostic.candidates().len(), 2);
    }
    let conflicts = result
        .env()
        .definitions()
        .iter()
        .filter_map(|entry| entry.conflict())
        .collect::<Vec<_>>();
    assert_eq!(conflicts.len(), cases.len() * 2);
    assert!(
        conflicts
            .iter()
            .all(|conflict| **conflict == DeclarationConflictClass::DuplicateSpelling)
    );
    assert_eq!(
        result
            .env()
            .contributions()
            .iter()
            .next()
            .unwrap()
            .effects()
            .diagnostics()
            .len(),
        cases.len()
    );
}

#[test]
fn overloadable_candidates_form_groups_and_illegal_groups_get_diagnostics() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::FunctorDefinition),
            test_item(10, SurfaceNodeKind::FunctorDefinition),
            test_item(20, SurfaceNodeKind::PredicateDefinition),
            test_item(30, SurfaceNodeKind::PredicateDefinition),
        ],
    );
    let namespace = NamespacePath::new("main");
    let projections = vec![
        projection(
            shells.declarations()[0].id(),
            namespace.clone(),
            "F",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1),
        projection(
            shells.declarations()[1].id(),
            namespace.clone(),
            "F",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1),
        projection(
            shells.declarations()[2].id(),
            namespace.clone(),
            "BadLeft",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        )
        .with_notation_spelling("_ bad _")
        .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
        projection(
            shells.declarations()[3].id(),
            namespace,
            "BadRight",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        )
        .with_notation_spelling("_ bad _")
        .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
    ];

    let result = collect(source_id, &shells, &projections);

    assert_eq!(result.env().overloads().len(), 2);
    assert_eq!(
        result.diagnostics()[0].class(),
        SymbolDiagnosticClass::IllegalOverloadGroup
    );
    let illegal = result
        .env()
        .overloads()
        .iter()
        .find(|group| group.key().spelling() == "_ bad _")
        .unwrap();
    assert_eq!(illegal.diagnostics(), &[result.diagnostics()[0].id()]);
    let legal = result
        .env()
        .overloads()
        .iter()
        .find(|group| group.key().spelling() == "F")
        .unwrap();
    assert_eq!(legal.candidates().len(), 2);
    assert!(legal.diagnostics().is_empty());
}

#[test]
fn same_signature_functor_conflicts_get_specific_internal_class() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::FunctorDefinition),
            test_item(10, SurfaceNodeKind::FunctorDefinition),
        ],
    );
    let namespace = NamespacePath::new("main");
    let projections = vec![
        projection(
            shells.declarations()[0].id(),
            namespace.clone(),
            "gauge ( _ )",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key(
            "let x be set",
            "gauge ( _ )",
            "set",
        )),
        projection(
            shells.declarations()[1].id(),
            namespace,
            "gauge ( _ )",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key(
            "let x be set",
            "gauge ( _ )",
            "round set",
        )),
    ];

    let result = collect(source_id, &shells, &projections);

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].class(),
        SymbolDiagnosticClass::SameSignatureReturnConflict
    );
    let conflicts = result
        .env()
        .definitions()
        .iter()
        .filter_map(|entry| entry.conflict())
        .collect::<Vec<_>>();
    assert_eq!(conflicts.len(), 2);
    assert!(
        conflicts
            .iter()
            .all(|conflict| **conflict == DeclarationConflictClass::SameSignatureReturnConflict)
    );
}

#[test]
fn same_signature_same_return_functors_get_definition_conflict_class() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::FunctorDefinition),
            test_item(10, SurfaceNodeKind::FunctorDefinition),
        ],
    );
    let namespace = NamespacePath::new("main");
    let projections = vec![
        projection(
            shells.declarations()[0].id(),
            namespace.clone(),
            "gauge ( _ )",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key(
            "let x be set",
            "gauge ( _ )",
            "set",
        )),
        projection(
            shells.declarations()[1].id(),
            namespace,
            "gauge ( _ )",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key(
            "let x be set",
            "gauge ( _ )",
            "set",
        )),
    ];

    let permuted = vec![projections[1].clone(), projections[0].clone()];
    let result = collect(source_id, &shells, &projections);
    let reordered = collect(source_id, &shells, &permuted);

    assert_eq!(result.diagnostics(), reordered.diagnostics());
    assert_eq!(
        result.env().snapshot_text(),
        reordered.env().snapshot_text()
    );
    assert_eq!(result.diagnostics().len(), 1);
    let diagnostic = &result.diagnostics()[0];
    assert_eq!(
        diagnostic.class(),
        SymbolDiagnosticClass::SameSignatureDefinitionConflict
    );
    assert_eq!(diagnostic.shell(), Some(shells.declarations()[0].id()));
    assert_eq!(diagnostic.range(), shells.declarations()[0].range());
    assert_eq!(
        candidate_source_ranges(&result, diagnostic.candidates()),
        shells
            .declarations()
            .iter()
            .map(DeclarationShell::range)
            .collect::<Vec<_>>()
    );
    assert!(result.env().definitions().iter().all(|definition| {
        definition.conflict() == Some(&DeclarationConflictClass::SameSignatureDefinitionConflict)
    }));
    assert_eq!(
        result
            .env()
            .snapshot_text()
            .matches("conflict=same_signature_definition_conflict")
            .count(),
        2
    );
}

#[test]
fn same_return_conflict_candidates_keep_source_order_past_lexical_ordinal_ten() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        (0..12)
            .map(|index| test_item(index * 10, SurfaceNodeKind::FunctorDefinition))
            .collect(),
    );
    let namespace = NamespacePath::new("main");
    let projections = shells
        .declarations()
        .iter()
        .map(|shell| {
            projection(
                shell.id(),
                namespace.clone(),
                "gauge ( _ )",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling("gauge ( _ )")
            .with_overload_policy(SymbolOverloadPolicy::Overloadable)
            .with_arity(1)
            .with_functor_signature_key(functor_signature_key("let x be set", "gauge ( _ )", "set"))
        })
        .collect::<Vec<_>>();
    let permuted = projections.iter().rev().cloned().collect::<Vec<_>>();

    let result = collect(source_id, &shells, &projections);
    let reordered = collect(source_id, &shells, &permuted);

    assert_eq!(result.diagnostics(), reordered.diagnostics());
    assert_eq!(result.diagnostics().len(), 1);
    let diagnostic = &result.diagnostics()[0];
    assert_eq!(
        diagnostic.class(),
        SymbolDiagnosticClass::SameSignatureDefinitionConflict
    );
    assert_eq!(diagnostic.shell(), Some(shells.declarations()[0].id()));
    assert_eq!(diagnostic.range(), shells.declarations()[0].range());
    assert_eq!(diagnostic.candidates().len(), 12);
    assert_eq!(
        candidate_source_ranges(&result, diagnostic.candidates()),
        shells
            .declarations()
            .iter()
            .map(DeclarationShell::range)
            .collect::<Vec<_>>()
    );
}

#[test]
fn parser_backed_functor_signature_conflict_uses_extracted_return_types() {
    let source_id = source_id();
    let ast = parser_backed_same_signature_conflict_ast(source_id, &["set"], &["round", "set"]);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let namespace = NamespacePath::new("main");
    let projections = SignatureProjectionExtractor::new(&ast, &shells, namespace.clone()).extract();
    let functor_keys = projections
        .iter()
        .filter(|projection| projection.symbol_kind() == SymbolKind::Functor)
        .map(|projection| {
            projection
                .functor_signature_key
                .as_ref()
                .expect("parser-backed functor projection should carry a signature key")
        })
        .collect::<Vec<_>>();

    assert_eq!(functor_keys.len(), 2);
    assert!(
        functor_keys
            .iter()
            .all(|key| key.argument_context == "definition let x be set end")
    );
    assert!(functor_keys.iter().all(|key| key.pattern == "gauge ( x )"));
    assert!(functor_keys.iter().all(|key| key.arity.is_none()));
    assert_eq!(
        functor_keys
            .iter()
            .map(|key| key.return_type.as_str())
            .collect::<Vec<_>>(),
        vec!["set", "round set"]
    );

    let result = collect(source_id, &shells, &projections);

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].class(),
        SymbolDiagnosticClass::SameSignatureReturnConflict
    );
    let conflicts = result
        .env()
        .definitions()
        .iter()
        .filter_map(|entry| entry.conflict())
        .collect::<Vec<_>>();
    assert_eq!(conflicts.len(), 2);
    assert!(
        conflicts
            .iter()
            .all(|conflict| **conflict == DeclarationConflictClass::SameSignatureReturnConflict)
    );
}

#[test]
fn parser_backed_same_signature_same_return_functors_conflict() {
    let source_id = source_id();
    let ast = parser_backed_same_signature_conflict_ast(source_id, &["set"], &["set"]);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let projections =
        SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new("main")).extract();

    let result = collect(source_id, &shells, &projections);

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].class(),
        SymbolDiagnosticClass::SameSignatureDefinitionConflict
    );
    assert_eq!(result.diagnostics()[0].candidates().len(), 2);
    assert!(result.env().definitions().iter().all(|definition| {
        definition.conflict() == Some(&DeclarationConflictClass::SameSignatureDefinitionConflict)
    }));
}

#[test]
fn same_return_conflict_requires_the_exact_ordinary_functor_argument_key() {
    struct NearMiss {
        name: &'static str,
        left: SymbolDeclarationProjection,
        right: SymbolDeclarationProjection,
    }

    let source_id = source_id();
    let cases = 9;
    let shells = shells_for(
        source_id,
        (0..cases * 2)
            .map(|index| test_item(index * 10, SurfaceNodeKind::FunctorDefinition))
            .collect(),
    );
    let namespace = NamespacePath::new("main");
    let exact =
        |shell, namespace: NamespacePath, spelling: &str, context: &str, pattern: &str, arity| {
            projection(
                shell,
                namespace,
                spelling,
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling(pattern)
            .with_overload_policy(SymbolOverloadPolicy::Overloadable)
            .with_arity(arity)
            .with_functor_signature_key(FunctorSignatureKey {
                argument_context: context.to_owned(),
                pattern: pattern.to_owned(),
                arity: Some(arity),
                return_type: "set".to_owned(),
            })
        };
    let mut near_misses = Vec::new();
    let mut pair = |name, left, right| near_misses.push(NearMiss { name, left, right });
    pair(
        "spelling",
        exact(
            shells.declarations()[0].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        exact(
            shells.declarations()[1].id(),
            namespace.clone(),
            "other",
            "ctx",
            "gauge ( _ )",
            1,
        ),
    );
    pair(
        "pattern",
        exact(
            shells.declarations()[2].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        exact(
            shells.declarations()[3].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge [ _ ]",
            1,
        ),
    );
    pair(
        "argument context",
        exact(
            shells.declarations()[4].id(),
            namespace.clone(),
            "gauge",
            "left",
            "gauge ( _ )",
            1,
        ),
        exact(
            shells.declarations()[5].id(),
            namespace.clone(),
            "gauge",
            "right",
            "gauge ( _ )",
            1,
        ),
    );
    pair(
        "arity",
        exact(
            shells.declarations()[6].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        exact(
            shells.declarations()[7].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            2,
        ),
    );
    pair(
        "namespace",
        exact(
            shells.declarations()[8].id(),
            NamespacePath::new("left"),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        exact(
            shells.declarations()[9].id(),
            NamespacePath::new("right"),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
    );
    pair(
        "both kinds",
        exact(
            shells.declarations()[10].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        projection(
            shells.declarations()[11].id(),
            namespace.clone(),
            "gauge",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key("ctx", "gauge ( _ )", "set")),
    );
    pair(
        "symbol kind",
        exact(
            shells.declarations()[12].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        projection(
            shells.declarations()[13].id(),
            namespace.clone(),
            "gauge",
            SymbolKind::Predicate,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key("ctx", "gauge ( _ )", "set")),
    );
    pair(
        "definition kind",
        exact(
            shells.declarations()[14].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        projection(
            shells.declarations()[15].id(),
            namespace.clone(),
            "gauge",
            SymbolKind::Functor,
            DefinitionKind::Predicate,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key("ctx", "gauge ( _ )", "set")),
    );
    pair(
        "nonordinary",
        exact(
            shells.declarations()[16].id(),
            namespace.clone(),
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        ),
        exact(
            shells.declarations()[17].id(),
            namespace,
            "gauge",
            "ctx",
            "gauge ( _ )",
            1,
        )
        .with_overload_policy(SymbolOverloadPolicy::NonOverloadable),
    );

    for near_miss in near_misses {
        let result = collect(source_id, &shells, &[near_miss.left, near_miss.right]);
        assert!(
            result.diagnostics().is_empty(),
            "{} near miss unexpectedly conflicted: {:?}",
            near_miss.name,
            result.diagnostics()
        );
    }
}

#[test]
fn mixed_return_group_keeps_one_return_conflict_in_canonical_order() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::FunctorDefinition),
            test_item(10, SurfaceNodeKind::FunctorDefinition),
            test_item(20, SurfaceNodeKind::FunctorDefinition),
        ],
    );
    let namespace = NamespacePath::new("main");
    let make = |index: usize, return_type: &str| {
        projection(
            shells.declarations()[index].id(),
            namespace.clone(),
            "gauge ( _ )",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("gauge ( _ )")
        .with_overload_policy(SymbolOverloadPolicy::Overloadable)
        .with_arity(1)
        .with_functor_signature_key(functor_signature_key(
            "let x be set",
            "gauge ( _ )",
            return_type,
        ))
    };
    let canonical = vec![make(0, "set"), make(1, "set"), make(2, "round set")];
    let permuted = vec![
        canonical[2].clone(),
        canonical[0].clone(),
        canonical[1].clone(),
    ];

    let first = collect(source_id, &shells, &canonical);
    let second = collect(source_id, &shells, &permuted);

    assert_eq!(first.diagnostics(), second.diagnostics());
    assert_eq!(first.env().snapshot_text(), second.env().snapshot_text());
    assert_eq!(first.diagnostics().len(), 1);
    let diagnostic = &first.diagnostics()[0];
    assert_eq!(
        diagnostic.class(),
        SymbolDiagnosticClass::SameSignatureReturnConflict
    );
    assert_eq!(diagnostic.shell(), Some(shells.declarations()[0].id()));
    assert_eq!(diagnostic.range(), shells.declarations()[0].range());
    assert_eq!(
        candidate_source_ranges(&first, diagnostic.candidates()),
        shells
            .declarations()
            .iter()
            .map(DeclarationShell::range)
            .collect::<Vec<_>>()
    );
    assert!(first.env().definitions().iter().all(|definition| {
        definition.conflict() == Some(&DeclarationConflictClass::SameSignatureReturnConflict)
    }));
}

#[test]
fn recovered_same_return_functor_does_not_cascade_a_signature_conflict() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 1, 2),
        Vec::new(),
    );
    let recovered = node(
        &mut builder,
        SurfaceNodeKind::FunctorDefinition,
        source_id,
        0,
        5,
        vec![recovery],
    );
    let clean = node(
        &mut builder,
        SurfaceNodeKind::FunctorDefinition,
        source_id,
        10,
        15,
        Vec::new(),
    );
    let root = finish_module(&mut builder, source_id, vec![recovered, clean]);
    let ast = builder.finish(Some(root), None);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let namespace = NamespacePath::new("main");
    let make =
        |index: usize| {
            projection(
                shells.declarations()[index].id(),
                namespace.clone(),
                "gauge ( _ )",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling("gauge ( _ )")
            .with_overload_policy(SymbolOverloadPolicy::Overloadable)
            .with_arity(1)
            .with_functor_signature_key(functor_signature_key("let x be set", "gauge ( _ )", "set"))
        };

    let result = collect(source_id, &shells, &[make(0), make(1)]);

    assert!(result.diagnostics().is_empty());
    assert_eq!(
        result
            .env()
            .definitions()
            .iter()
            .filter_map(|definition| definition.conflict())
            .collect::<Vec<_>>(),
        vec![&DeclarationConflictClass::RecoveredShell]
    );
}

#[test]
fn diagnostics_are_sorted_by_range_class_spelling_and_stable_ids() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::DefinitionBlockItem),
            test_item(0, SurfaceNodeKind::PredicateDefinition),
            test_item(0, SurfaceNodeKind::PredicateDefinition),
            test_item(0, SurfaceNodeKind::AttributeDefinition),
            test_item(0, SurfaceNodeKind::AttributeDefinition),
            test_item(70, SurfaceNodeKind::FunctorDefinition),
            test_item(80, SurfaceNodeKind::FunctorDefinition),
        ],
    );
    let namespace = NamespacePath::new("main");
    let projections = vec![
        projection(
            shells.declarations()[6].id(),
            namespace.clone(),
            "IllegalRight",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("_ ? _")
        .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
        projection(
            shells.declarations()[2].id(),
            namespace.clone(),
            "BDup",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        ),
        projection(
            shells.declarations()[4].id(),
            namespace.clone(),
            "ADup",
            SymbolKind::Attribute,
            DefinitionKind::Attribute,
        ),
        projection(
            shells.declarations()[0].id(),
            namespace.clone(),
            "Context",
            SymbolKind::Structure,
            DefinitionKind::Structure,
        ),
        projection(
            shells.declarations()[5].id(),
            namespace.clone(),
            "IllegalLeft",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("_ ? _")
        .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
        projection(
            shells.declarations()[1].id(),
            namespace.clone(),
            "BDup",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        ),
        projection(
            shells.declarations()[3].id(),
            namespace,
            "ADup",
            SymbolKind::Attribute,
            DefinitionKind::Attribute,
        ),
    ];

    let result = collect(source_id, &shells, &projections);
    let diagnostics = result.diagnostics();

    assert_eq!(diagnostics.len(), 4);
    assert_eq!(
        diagnostics
            .iter()
            .map(SymbolDiagnostic::class)
            .collect::<Vec<_>>(),
        vec![
            SymbolDiagnosticClass::ContextOnlyShell,
            SymbolDiagnosticClass::DuplicateDeclaration,
            SymbolDiagnosticClass::DuplicateDeclaration,
            SymbolDiagnosticClass::IllegalOverloadGroup,
        ]
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(SymbolDiagnostic::spelling)
            .collect::<Vec<_>>(),
        vec!["Context", "ADup", "BDup", "_ ? _"]
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.id().index())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.range().start)
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 70]
    );
}

#[test]
fn symbol_identity_includes_namespace_notation_arity_and_explicit_slot() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::FunctorDefinition),
            test_item(10, SurfaceNodeKind::FunctorDefinition),
        ],
    );
    let projections = vec![
        projection(
            shells.declarations()[0].id(),
            NamespacePath::new("left"),
            "Op",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("_ + _")
        .with_arity(2)
        .with_identity_slot("member:0"),
        projection(
            shells.declarations()[1].id(),
            NamespacePath::new("right"),
            "Op",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("_ * _")
        .with_arity(2)
        .with_identity_slot("member:1"),
    ];

    let result = collect(source_id, &shells, &projections);
    let locals = result
        .env()
        .symbols()
        .iter()
        .map(|entry| entry.symbol().local().as_str())
        .collect::<Vec<_>>();

    assert_eq!(locals.len(), 2);
    assert_ne!(locals[0], locals[1]);
    assert!(locals.iter().any(|local| local.contains("namespace=left")));
    assert!(locals.iter().any(|local| local.contains("namespace=right")));
    assert!(locals.iter().any(|local| local.contains("notation=_ + _")));
    assert!(locals.iter().any(|local| local.contains("notation=_ * _")));
    assert!(locals.iter().all(|local| local.contains("arity=2")));
    assert!(locals.iter().any(|local| local.contains("slot=member\\c0")));
    assert!(locals.iter().any(|local| local.contains("slot=member\\c1")));
}

#[test]
fn registration_projection_populates_symbol_definition_and_registration_indexes() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![test_item(0, SurfaceNodeKind::ConditionalRegistration)],
    );
    let projection = SymbolDeclarationProjection::new(
        shells.declarations()[0].id(),
        NamespacePath::new("main"),
        "Reg",
        SymbolKind::Registration,
    )
    .with_registration_kind(RegistrationKind::Cluster);

    let result = collect(source_id, &shells, &[projection]);
    let env = result.env();

    assert_eq!(env.symbols().len(), 1);
    assert_eq!(env.definitions().len(), 1);
    assert_eq!(env.registrations().len(), 1);
    let symbol = env.symbols().iter().next().unwrap();
    let definition = env.definitions().iter().next().unwrap();
    let registration = env.registrations().iter().next().unwrap();
    assert_eq!(definition.symbol(), symbol.symbol());
    assert_eq!(definition.kind(), DefinitionKind::Registration);
    assert_eq!(registration.symbol(), Some(symbol.symbol()));
    assert_eq!(registration.kind(), RegistrationKind::Cluster);
    let effects = env.contributions().iter().next().unwrap().effects();
    assert_eq!(effects.symbols(), &[symbol.symbol().clone()]);
    assert_eq!(effects.definitions(), &[definition.id()]);
    assert_eq!(effects.registrations().len(), 1);
    assert_eq!(effects.registrations(), &[registration.id()]);
    assert_eq!(symbol.contribution(), definition.contribution());
    assert_eq!(symbol.contribution(), registration.contribution());
}

#[test]
fn recovered_shells_stay_local_and_malformed_without_panicking() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 1, 2),
        Vec::new(),
    );
    let predicate = node(
        &mut builder,
        SurfaceNodeKind::PredicateDefinition,
        source_id,
        0,
        5,
        vec![recovery],
    );
    let root = finish_module(&mut builder, source_id, vec![predicate]);
    let ast = builder.finish(Some(root), None);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let projection = projection(
        shells.declarations()[0].id(),
        NamespacePath::new("main"),
        "Recovered",
        SymbolKind::Predicate,
        DefinitionKind::Predicate,
    );

    let result = collect(source_id, &shells, &[projection]);
    let symbol = result
        .env()
        .symbols()
        .visible_candidates(&NamespacePath::new("main"), "Recovered")[0];

    assert_eq!(symbol.export_status(), ExportStatus::LocalOnly);
    assert!(matches!(
        symbol.signature(),
        Some(SignatureShell::Malformed { class }) if class == "recovered-shell"
    ));
    assert_eq!(
        result.env().definitions().iter().next().unwrap().conflict(),
        Some(&DeclarationConflictClass::RecoveredShell)
    );
    assert!(result.env().lexical_summaries().is_empty());
}

#[test]
fn recovered_symbols_do_not_cascade_duplicate_or_overload_diagnostics() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let duplicate_recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 1, 2),
        Vec::new(),
    );
    let overload_recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 21, 22),
        Vec::new(),
    );
    let recovered_predicate = node(
        &mut builder,
        SurfaceNodeKind::PredicateDefinition,
        source_id,
        0,
        5,
        vec![duplicate_recovery],
    );
    let clean_predicate = node(
        &mut builder,
        SurfaceNodeKind::PredicateDefinition,
        source_id,
        10,
        15,
        Vec::new(),
    );
    let recovered_functor = node(
        &mut builder,
        SurfaceNodeKind::FunctorDefinition,
        source_id,
        20,
        25,
        vec![overload_recovery],
    );
    let clean_functor = node(
        &mut builder,
        SurfaceNodeKind::FunctorDefinition,
        source_id,
        30,
        35,
        Vec::new(),
    );
    let root = finish_module(
        &mut builder,
        source_id,
        vec![
            recovered_predicate,
            clean_predicate,
            recovered_functor,
            clean_functor,
        ],
    );
    let ast = builder.finish(Some(root), None);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let namespace = NamespacePath::new("main");
    let projections = vec![
        projection(
            shells.declarations()[0].id(),
            namespace.clone(),
            "P",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        ),
        projection(
            shells.declarations()[1].id(),
            namespace.clone(),
            "P",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        ),
        projection(
            shells.declarations()[2].id(),
            namespace.clone(),
            "BadRecovered",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("_ bad _")
        .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
        projection(
            shells.declarations()[3].id(),
            namespace,
            "BadClean",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        )
        .with_notation_spelling("_ bad _")
        .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
    ];

    let result = collect(source_id, &shells, &projections);
    let conflicts = result
        .env()
        .definitions()
        .iter()
        .filter_map(|entry| entry.conflict())
        .collect::<Vec<_>>();

    assert!(result.diagnostics().is_empty());
    assert_eq!(result.env().overloads().len(), 1);
    let overload = result.env().overloads().iter().next().unwrap();
    assert_eq!(overload.candidates().len(), 2);
    assert!(overload.diagnostics().is_empty());
    assert_eq!(
        conflicts,
        vec![
            &DeclarationConflictClass::RecoveredShell,
            &DeclarationConflictClass::RecoveredShell
        ]
    );
    assert!(
        result
            .env()
            .contributions()
            .iter()
            .next()
            .unwrap()
            .effects()
            .diagnostics()
            .is_empty()
    );
}

#[test]
fn recovered_context_only_shells_do_not_emit_context_diagnostics() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 1, 2),
        Vec::new(),
    );
    let registration_block = node(
        &mut builder,
        SurfaceNodeKind::RegistrationBlockItem,
        source_id,
        10,
        20,
        Vec::new(),
    );
    let definition_block = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        0,
        30,
        vec![recovery, registration_block],
    );
    let root = finish_module(&mut builder, source_id, vec![definition_block]);
    let ast = builder.finish(Some(root), None);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let child = shells
        .declarations()
        .iter()
        .find(|shell| shell.kind() == DeclarationShellKind::RegistrationBlock)
        .unwrap();
    let projections = vec![projection(
        child.id(),
        NamespacePath::new("main"),
        "RecoveredContext",
        SymbolKind::Registration,
        DefinitionKind::Registration,
    )];

    let result = collect(source_id, &shells, &projections);

    assert!(result.diagnostics().is_empty());
    assert!(result.env().symbols().is_empty());
}

#[test]
fn context_parent_visibility_and_recovery_propagate_to_child_symbols() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let private_marker = visibility_marker(&mut builder, source_id, 0, "private");
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 12, 13),
        Vec::new(),
    );
    let predicate = node(
        &mut builder,
        SurfaceNodeKind::PredicateDefinition,
        source_id,
        20,
        25,
        Vec::new(),
    );
    let definition_block = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        10,
        30,
        vec![recovery, predicate],
    );
    let visible_block = node(
        &mut builder,
        SurfaceNodeKind::VisibleItem,
        source_id,
        0,
        30,
        vec![private_marker, definition_block],
    );
    let root = finish_module(&mut builder, source_id, vec![visible_block]);
    let ast = builder.finish(Some(root), None);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let child = shells
        .declarations()
        .iter()
        .find(|shell| shell.kind() == DeclarationShellKind::PredicateDefinition)
        .unwrap();
    let projection = projection(
        child.id(),
        NamespacePath::new("main"),
        "InheritedContext",
        SymbolKind::Predicate,
        DefinitionKind::Predicate,
    );

    let result = collect(source_id, &shells, &[projection]);
    let symbol = result
        .env()
        .symbols()
        .visible_candidates(&NamespacePath::new("main"), "InheritedContext")[0];

    assert_eq!(symbol.visibility(), Visibility::Private);
    assert_eq!(symbol.export_status(), ExportStatus::LocalOnly);
    assert!(symbol.origin().is_recovered());
    assert!(matches!(
        symbol.signature(),
        Some(SignatureShell::Malformed { class }) if class == "recovered-shell"
    ));
    assert_eq!(
        result.env().definitions().iter().next().unwrap().conflict(),
        Some(&DeclarationConflictClass::RecoveredShell)
    );
    assert!(result.env().lexical_summaries().is_empty());
}

#[test]
fn context_only_shells_do_not_fabricate_symbol_identities() {
    let source_id = source_id();
    let shells = shells_for(
        source_id,
        vec![
            test_item(0, SurfaceNodeKind::DefinitionBlockItem),
            visible_test_item(10, "public", SurfaceNodeKind::FunctorDefinition),
        ],
    );
    let projections = vec![
        projection(
            shells.declarations()[0].id(),
            NamespacePath::new("main"),
            "Block",
            SymbolKind::Structure,
            DefinitionKind::Structure,
        ),
        projection(
            shells.declarations()[1].id(),
            NamespacePath::new("main"),
            "VisibleFunctor",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        ),
    ];

    let result = collect(source_id, &shells, &projections);

    assert_eq!(result.env().symbols().len(), 1);
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].class(),
        SymbolDiagnosticClass::ContextOnlyShell
    );
    assert_eq!(
        result
            .env()
            .symbols()
            .iter()
            .next()
            .unwrap()
            .export_status(),
        ExportStatus::Exported
    );
}

#[test]
fn parser_backed_extractor_projects_represented_signature_families() {
    let source_id = source_id();
    let ast = parser_backed_signature_ast(source_id);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let namespace = NamespacePath::new("main");
    let projections = SignatureProjectionExtractor::new(&ast, &shells, namespace.clone()).extract();

    assert_projection(&projections, SymbolKind::Theorem, "T1");
    assert_projection(&projections, SymbolKind::Lemma, "L1");
    assert_projection(&projections, SymbolKind::Attribute, "empty");
    assert_projection(&projections, SymbolKind::Attribute, "ranked");
    assert_projection(&projections, SymbolKind::Predicate, "x R y");
    assert_projection(&projections, SymbolKind::Functor, "F");
    assert_projection(&projections, SymbolKind::Mode, "Element");
    assert_projection(&projections, SymbolKind::Structure, "Carrier");
    assert_projection(&projections, SymbolKind::Selector, "carrier");
    assert_projection(&projections, SymbolKind::Selector, "property");
    assert_projection(&projections, SymbolKind::Algorithm, "verified");
    assert_projection(&projections, SymbolKind::Synonym, "++");
    assert_projection(&projections, SymbolKind::Antonym, "--");
    assert_projection(&projections, SymbolKind::Redefinition, "R2");
    assert_projection(&projections, SymbolKind::Redefinition, "attr-red");
    assert_projection(&projections, SymbolKind::Redefinition, "func-red");
    assert_projection(&projections, SymbolKind::Redefinition, "field-red");
    assert_projection(&projections, SymbolKind::Redefinition, "property-red");
    assert_projection(&projections, SymbolKind::Attribute, "symmetry");
    assert_projection(&projections, SymbolKind::Registration, "Reg");
    assert_projection(&projections, SymbolKind::Registration, "ExistsReg");
    assert_projection(&projections, SymbolKind::Registration, "FunctorialReg");
    assert_projection(&projections, SymbolKind::Registration, "ReduceReg");

    for projection in &projections {
        assert!(matches!(
            projection.signature(),
            Some(SignatureShell::Opaque { schema, .. }) if schema == "parser-signature-v1"
        ));
    }

    let predicate = projections
        .iter()
        .find(|projection| projection.symbol_kind() == SymbolKind::Predicate)
        .unwrap();
    assert_eq!(predicate.notation_spelling(), Some("x R y"));
    assert_eq!(predicate.arity(), None);
    assert!(matches!(
        predicate.signature(),
        Some(SignatureShell::Opaque { schema, payload })
            if schema == "parser-signature-v1"
                && payload.contains("node=PredicateDefinition")
                && payload.contains("roles=PredicatePattern")
                && payload.contains("TemplateParameter")
    ));

    let parameterized_attribute = projections
        .iter()
        .find(|projection| {
            projection.symbol_kind() == SymbolKind::Attribute
                && projection.primary_spelling() == "ranked"
        })
        .unwrap();
    assert_eq!(
        parameterized_attribute.notation_spelling(),
        Some("2 - ranked")
    );

    let result = collect(source_id, &shells, &projections);
    assert!(
        result
            .env()
            .lexical_summaries()
            .visible_candidates(&namespace, "x R y")
            .iter()
            .any(|entry| entry.kind() == LexicalSummaryKind::Notation)
    );
    assert!(
        result
            .env()
            .lexical_summaries()
            .visible_candidates(&namespace, "ranked")
            .iter()
            .any(|entry| entry.kind() == LexicalSummaryKind::Notation)
    );
    assert!(
        result
            .env()
            .lexical_summaries()
            .visible_candidates(&namespace, "2 - ranked")
            .is_empty()
    );
    assert!(
        result
            .env()
            .lexical_summaries()
            .visible_candidates(&namespace, "carrier")
            .is_empty()
    );
    assert!(
        result
            .env()
            .lexical_summaries()
            .visible_candidates(&namespace, "symmetry")
            .is_empty()
    );
    assert!(
        result
            .env()
            .lexical_summaries()
            .visible_candidates(&namespace, "verified")
            .is_empty()
    );
    assert!(
        result
            .env()
            .lexical_summaries()
            .visible_candidates(&namespace, "T1")
            .is_empty()
    );
    assert!(
        result
            .env()
            .registrations()
            .iter()
            .any(|entry| entry.kind() == RegistrationKind::Cluster)
    );
    assert!(
        result
            .env()
            .registrations()
            .iter()
            .any(|entry| entry.kind() == RegistrationKind::Reduction)
    );
}

#[test]
fn parser_backed_recovered_projection_uses_malformed_signature() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 0, 1),
        Vec::new(),
    );
    let pattern_tokens = token_sequence(
        &mut builder,
        source_id,
        2,
        &[(SurfaceTokenKind::UserSymbol, "Broken")],
    );
    let pattern = node(
        &mut builder,
        SurfaceNodeKind::PredicatePattern,
        source_id,
        2,
        8,
        pattern_tokens,
    );
    let predicate = node(
        &mut builder,
        SurfaceNodeKind::PredicateDefinition,
        source_id,
        0,
        8,
        vec![recovery, pattern],
    );
    let root = finish_module(&mut builder, source_id, vec![predicate]);
    let ast = builder.finish(Some(root), None);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let projections =
        SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new("main")).extract();

    assert!(matches!(
        projections[0].signature(),
        Some(SignatureShell::Opaque { schema, .. }) if schema == "parser-signature-v1"
    ));
    let result = collect(source_id, &shells, &projections);
    let symbol = result.env().symbols().iter().next().unwrap();

    assert!(matches!(
        symbol.signature(),
        Some(SignatureShell::Malformed { class }) if class == "recovered-shell"
    ));
    assert_eq!(
        result.env().definitions().iter().next().unwrap().conflict(),
        Some(&DeclarationConflictClass::RecoveredShell)
    );
    assert!(result.env().lexical_summaries().is_empty());
}

#[derive(Debug, Clone)]
struct DuplicateCase {
    item_kind: SurfaceNodeKind,
    spelling: &'static str,
    symbol_kind: SymbolKind,
    definition_kind: Option<DefinitionKind>,
    registration_kind: Option<RegistrationKind>,
}

impl DuplicateCase {
    fn projection(
        &self,
        shell: DeclarationShellId,
        namespace: NamespacePath,
    ) -> SymbolDeclarationProjection {
        let projection =
            SymbolDeclarationProjection::new(shell, namespace, self.spelling, self.symbol_kind);
        if let Some(registration_kind) = self.registration_kind {
            projection.with_registration_kind(registration_kind)
        } else {
            projection.with_definition_kind(
                self.definition_kind
                    .expect("non-registration duplicate case has a definition kind"),
            )
        }
    }
}

fn duplicate_cases() -> Vec<DuplicateCase> {
    vec![
        duplicate_case(
            SurfaceNodeKind::PredicateDefinition,
            "DupPredicate",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        ),
        duplicate_case(
            SurfaceNodeKind::FunctorDefinition,
            "DupFunctor",
            SymbolKind::Functor,
            DefinitionKind::Functor,
        ),
        duplicate_case(
            SurfaceNodeKind::ModeDefinition,
            "DupMode",
            SymbolKind::Mode,
            DefinitionKind::Mode,
        ),
        duplicate_case(
            SurfaceNodeKind::AttributeDefinition,
            "DupAttribute",
            SymbolKind::Attribute,
            DefinitionKind::Attribute,
        ),
        duplicate_case(
            SurfaceNodeKind::StructureDefinition,
            "DupStructure",
            SymbolKind::Structure,
            DefinitionKind::Structure,
        ),
        duplicate_case(
            SurfaceNodeKind::TheoremItem,
            "DupTheorem",
            SymbolKind::Theorem,
            DefinitionKind::Theorem,
        ),
        duplicate_case(
            SurfaceNodeKind::LemmaItem,
            "DupLemma",
            SymbolKind::Lemma,
            DefinitionKind::Lemma,
        ),
        duplicate_case(
            SurfaceNodeKind::AlgorithmDefinition,
            "DupAlgorithm",
            SymbolKind::Algorithm,
            DefinitionKind::Algorithm,
        ),
        duplicate_case(
            SurfaceNodeKind::NotationAlias,
            "DupSynonym",
            SymbolKind::Synonym,
            DefinitionKind::Synonym,
        ),
        duplicate_case(
            SurfaceNodeKind::NotationAlias,
            "DupAntonym",
            SymbolKind::Antonym,
            DefinitionKind::Antonym,
        ),
        duplicate_case(
            SurfaceNodeKind::PredicateRedefinition,
            "DupRedefinition",
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
        ),
        duplicate_case(
            SurfaceNodeKind::StructureField,
            "DupSelector",
            SymbolKind::Selector,
            DefinitionKind::Selector,
        ),
        DuplicateCase {
            item_kind: SurfaceNodeKind::ConditionalRegistration,
            spelling: "DupRegistration",
            symbol_kind: SymbolKind::Registration,
            definition_kind: None,
            registration_kind: Some(RegistrationKind::Cluster),
        },
    ]
}

const fn duplicate_case(
    item_kind: SurfaceNodeKind,
    spelling: &'static str,
    symbol_kind: SymbolKind,
    definition_kind: DefinitionKind,
) -> DuplicateCase {
    DuplicateCase {
        item_kind,
        spelling,
        symbol_kind,
        definition_kind: Some(definition_kind),
        registration_kind: None,
    }
}

fn duplicate_case_items(cases: &[DuplicateCase]) -> Vec<TestItem> {
    cases
        .iter()
        .enumerate()
        .flat_map(|(index, case)| {
            let start = index * 20;
            [
                test_item(start, case.item_kind.clone()),
                test_item(start + 10, case.item_kind.clone()),
            ]
        })
        .collect()
}

fn assert_projection(
    projections: &[SymbolDeclarationProjection],
    kind: SymbolKind,
    spelling: &str,
) {
    assert!(
        projections
            .iter()
            .any(|projection| projection.symbol_kind() == kind
                && projection.primary_spelling() == spelling),
        "missing {kind:?} projection named {spelling}; projections={projections:?}"
    );
}

fn parser_backed_signature_ast(source_id: SourceId) -> mizar_syntax::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let items = vec![
        label_item(
            &mut builder,
            source_id,
            0,
            SurfaceNodeKind::TheoremItem,
            "T1",
        ),
        label_item(
            &mut builder,
            source_id,
            20,
            SurfaceNodeKind::LemmaItem,
            "L1",
        ),
        pattern_item(
            &mut builder,
            source_id,
            40,
            SurfaceNodeKind::AttributeDefinition,
            SurfaceNodeKind::AttributePattern,
            &[(SurfaceTokenKind::Identifier, "empty")],
        ),
        pattern_item(
            &mut builder,
            source_id,
            500,
            SurfaceNodeKind::AttributeDefinition,
            SurfaceNodeKind::AttributePattern,
            &[
                (SurfaceTokenKind::Numeral, "2"),
                (SurfaceTokenKind::ReservedSymbol, "-"),
                (SurfaceTokenKind::UserSymbol, "ranked"),
            ],
        ),
        templated_pattern_item(
            &mut builder,
            source_id,
            60,
            SurfaceNodeKind::PredicateDefinition,
            SurfaceNodeKind::PredicatePattern,
            &[
                (SurfaceTokenKind::Identifier, "x"),
                (SurfaceTokenKind::UserSymbol, "R"),
                (SurfaceTokenKind::Identifier, "y"),
            ],
            "T",
        ),
        pattern_item(
            &mut builder,
            source_id,
            90,
            SurfaceNodeKind::FunctorDefinition,
            SurfaceNodeKind::FunctorPattern,
            &[(SurfaceTokenKind::Identifier, "F")],
        ),
        pattern_item(
            &mut builder,
            source_id,
            110,
            SurfaceNodeKind::ModeDefinition,
            SurfaceNodeKind::ModePattern,
            &[(SurfaceTokenKind::Identifier, "Element")],
        ),
        structure_item(&mut builder, source_id, 135),
        algorithm_item(&mut builder, source_id, 190, "verified"),
        notation_alias_item(&mut builder, source_id, 230, "synonym", "++"),
        notation_alias_item(&mut builder, source_id, 250, "antonym", "--"),
        redefinition_item(
            &mut builder,
            source_id,
            270,
            SurfaceNodeKind::PredicateRedefinition,
            "R2",
        ),
        redefinition_item(
            &mut builder,
            source_id,
            290,
            SurfaceNodeKind::AttributeRedefinition,
            "attr-red",
        ),
        redefinition_item(
            &mut builder,
            source_id,
            310,
            SurfaceNodeKind::FunctorRedefinition,
            "func-red",
        ),
        redefinition_item(
            &mut builder,
            source_id,
            330,
            SurfaceNodeKind::FieldRedefinition,
            "field-red",
        ),
        redefinition_item(
            &mut builder,
            source_id,
            350,
            SurfaceNodeKind::PropertyRedefinition,
            "property-red",
        ),
        property_clause_item(&mut builder, source_id, 375, "symmetry"),
        label_item(
            &mut builder,
            source_id,
            395,
            SurfaceNodeKind::ExistentialRegistration,
            "ExistsReg",
        ),
        label_item(
            &mut builder,
            source_id,
            420,
            SurfaceNodeKind::ConditionalRegistration,
            "Reg",
        ),
        label_item(
            &mut builder,
            source_id,
            440,
            SurfaceNodeKind::FunctorialRegistration,
            "FunctorialReg",
        ),
        label_item(
            &mut builder,
            source_id,
            470,
            SurfaceNodeKind::ReductionRegistration,
            "ReduceReg",
        ),
    ];
    let root = finish_module(&mut builder, source_id, items);
    builder.finish(Some(root), None)
}

fn parser_backed_same_signature_conflict_ast(
    source_id: SourceId,
    left_return_type: &[&str],
    right_return_type: &[&str],
) -> mizar_syntax::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let items = vec![
        definition_block_with_functor(
            &mut builder,
            source_id,
            0,
            Some("public"),
            "GaugeADef",
            left_return_type,
        ),
        definition_block_with_functor(
            &mut builder,
            source_id,
            80,
            Some("private"),
            "GaugeBDef",
            right_return_type,
        ),
    ];
    let root = finish_module(&mut builder, source_id, items);
    builder.finish(Some(root), None)
}

fn definition_block_with_functor(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    visibility: Option<&str>,
    label: &str,
    return_type: &[&str],
) -> SurfaceBuilderNodeId {
    let definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, start, start + 10),
    );
    let let_statement = let_statement(builder, source_id, start + 11);
    let functor = visible_functor_definition_item(
        builder,
        source_id,
        start + 30,
        visibility,
        label,
        return_type,
    );
    let end_start = builder
        .node_range(functor)
        .expect("fresh definition content should have a source range")
        .end
        + 2;
    let end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, end_start, end_start + 3),
    );
    node(
        builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        start,
        end_start + 3,
        vec![definition, let_statement, functor, end],
    )
}

fn visible_functor_definition_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    visibility: Option<&str>,
    label: &str,
    return_type: &[&str],
) -> SurfaceBuilderNodeId {
    let Some(visibility) = visibility else {
        return functor_definition_item(builder, source_id, start, label, return_type);
    };
    let marker = visibility_marker(builder, source_id, start, visibility);
    let target_start = start + visibility.len() + 1;
    let target = functor_definition_item(builder, source_id, target_start, label, return_type);
    let target_end = builder
        .node_range(target)
        .expect("fresh functor node should have a source range")
        .end;
    node(
        builder,
        SurfaceNodeKind::VisibleItem,
        source_id,
        start,
        target_end,
        vec![marker, target],
    )
}

fn let_statement(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
) -> SurfaceBuilderNodeId {
    let tokens = token_sequence(
        builder,
        source_id,
        start,
        &[
            (SurfaceTokenKind::ReservedWord, "let"),
            (SurfaceTokenKind::Identifier, "x"),
            (SurfaceTokenKind::ReservedWord, "be"),
            (SurfaceTokenKind::Identifier, "set"),
        ],
    );
    node(
        builder,
        SurfaceNodeKind::LetStatement,
        source_id,
        start,
        start + 12,
        tokens,
    )
}

fn functor_definition_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    label: &str,
    return_type: &[&str],
) -> SurfaceBuilderNodeId {
    let func = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "func",
        range(source_id, start, start + 4),
    );
    let label_start = start + 5;
    let label = builder.add_token(
        SurfaceTokenKind::Identifier,
        label,
        range(source_id, label_start, label_start + label.len()),
    );
    let colon_start = label_start + 10;
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, colon_start, colon_start + 1),
    );
    let pattern_start = colon_start + 2;
    let pattern_tokens = token_sequence(
        builder,
        source_id,
        pattern_start,
        &[
            (SurfaceTokenKind::Identifier, "gauge"),
            (SurfaceTokenKind::ReservedSymbol, "("),
            (SurfaceTokenKind::Identifier, "x"),
            (SurfaceTokenKind::ReservedSymbol, ")"),
        ],
    );
    let pattern = node(
        builder,
        SurfaceNodeKind::FunctorPattern,
        source_id,
        pattern_start,
        pattern_start + 12,
        pattern_tokens,
    );
    let arrow_start = pattern_start + 13;
    let arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, arrow_start, arrow_start + 2),
    );
    let type_start = arrow_start + 3;
    let type_tokens = return_type
        .iter()
        .scan(type_start, |cursor, text| {
            let token_start = *cursor;
            let token_end = token_start + text.len();
            *cursor = token_end + 1;
            Some(builder.add_token(
                SurfaceTokenKind::Identifier,
                *text,
                range(source_id, token_start, token_end),
            ))
        })
        .collect::<Vec<_>>();
    let type_end = type_tokens
        .last()
        .map(|id| {
            builder
                .node_range(*id)
                .expect("fresh token should have a source range")
                .end
        })
        .unwrap_or(type_start);
    let type_expression = node(
        builder,
        SurfaceNodeKind::TypeExpression,
        source_id,
        type_start,
        type_end,
        type_tokens,
    );
    node(
        builder,
        SurfaceNodeKind::FunctorDefinition,
        source_id,
        start,
        type_end,
        vec![func, label, colon, pattern, arrow, type_expression],
    )
}

fn label_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    kind: SurfaceNodeKind,
    label: &str,
) -> SurfaceBuilderNodeId {
    let label_token = builder.add_token(
        SurfaceTokenKind::Identifier,
        label,
        range(source_id, start, start + label.len()),
    );
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, start + label.len(), start + label.len() + 1),
    );
    node(
        builder,
        kind,
        source_id,
        start,
        start + label.len() + 1,
        vec![label_token, colon],
    )
}

fn pattern_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    item_kind: SurfaceNodeKind,
    pattern_kind: SurfaceNodeKind,
    tokens: &[(SurfaceTokenKind, &str)],
) -> SurfaceBuilderNodeId {
    let token_nodes = token_sequence(builder, source_id, start, tokens);
    let end = start + token_nodes.len() * 2 + 1;
    let pattern = node(builder, pattern_kind, source_id, start, end, token_nodes);
    node(builder, item_kind, source_id, start, end, vec![pattern])
}

fn templated_pattern_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    item_kind: SurfaceNodeKind,
    pattern_kind: SurfaceNodeKind,
    tokens: &[(SurfaceTokenKind, &str)],
    template_name: &str,
) -> SurfaceBuilderNodeId {
    let token_nodes = token_sequence(builder, source_id, start, tokens);
    let pattern_end = start + token_nodes.len() * 2 + 1;
    let pattern = node(
        builder,
        pattern_kind,
        source_id,
        start,
        pattern_end,
        token_nodes,
    );
    let template_start = pattern_end + 1;
    let template_token = builder.add_token(
        SurfaceTokenKind::Identifier,
        template_name,
        range(
            source_id,
            template_start,
            template_start + template_name.len(),
        ),
    );
    let template = node(
        builder,
        SurfaceNodeKind::TemplateParameter,
        source_id,
        template_start,
        template_start + template_name.len(),
        vec![template_token],
    );
    node(
        builder,
        item_kind,
        source_id,
        start,
        template_start + template_name.len(),
        vec![pattern, template],
    )
}

fn structure_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
) -> SurfaceBuilderNodeId {
    let pattern_tokens = token_sequence(
        builder,
        source_id,
        start,
        &[(SurfaceTokenKind::Identifier, "Carrier")],
    );
    let pattern = node(
        builder,
        SurfaceNodeKind::StructurePattern,
        source_id,
        start,
        start + 7,
        pattern_tokens,
    );
    let field_tokens = token_sequence(
        builder,
        source_id,
        start + 10,
        &[(SurfaceTokenKind::Identifier, "carrier")],
    );
    let field = node(
        builder,
        SurfaceNodeKind::StructureField,
        source_id,
        start + 10,
        start + 17,
        field_tokens,
    );
    let property_tokens = token_sequence(
        builder,
        source_id,
        start + 20,
        &[(SurfaceTokenKind::Identifier, "property")],
    );
    let property = node(
        builder,
        SurfaceNodeKind::StructureProperty,
        source_id,
        start + 20,
        start + 28,
        property_tokens,
    );
    node(
        builder,
        SurfaceNodeKind::StructureDefinition,
        source_id,
        start,
        start + 30,
        vec![pattern, field, property],
    )
}

fn algorithm_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    name: &str,
) -> SurfaceBuilderNodeId {
    let algorithm = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "algorithm",
        range(source_id, start, start + 9),
    );
    let name_start = start + 10;
    let name = builder.add_token(
        SurfaceTokenKind::Identifier,
        name,
        range(source_id, name_start, name_start + name.len()),
    );
    node(
        builder,
        SurfaceNodeKind::AlgorithmDefinition,
        source_id,
        start,
        name_start + 8,
        vec![algorithm, name],
    )
}

fn notation_alias_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    keyword: &str,
    spelling: &str,
) -> SurfaceBuilderNodeId {
    let keyword_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        keyword,
        range(source_id, start, start + keyword.len()),
    );
    let pattern_start = start + keyword.len() + 1;
    let pattern_tokens = token_sequence(
        builder,
        source_id,
        pattern_start,
        &[(SurfaceTokenKind::UserSymbol, spelling)],
    );
    let pattern = node(
        builder,
        SurfaceNodeKind::NotationPattern,
        source_id,
        pattern_start,
        pattern_start + spelling.len(),
        pattern_tokens,
    );
    node(
        builder,
        SurfaceNodeKind::NotationAlias,
        source_id,
        start,
        pattern_start + spelling.len(),
        vec![keyword_token, pattern],
    )
}

fn property_clause_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    spelling: &str,
) -> SurfaceBuilderNodeId {
    let token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        spelling,
        range(source_id, start, start + spelling.len()),
    );
    node(
        builder,
        SurfaceNodeKind::PropertyClause,
        source_id,
        start,
        start + spelling.len(),
        vec![token],
    )
}

fn redefinition_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    kind: SurfaceNodeKind,
    spelling: &str,
) -> SurfaceBuilderNodeId {
    let token = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        spelling,
        range(source_id, start, start + spelling.len()),
    );
    node(
        builder,
        kind,
        source_id,
        start,
        start + spelling.len(),
        vec![token],
    )
}

fn token_sequence(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    tokens: &[(SurfaceTokenKind, &str)],
) -> Vec<SurfaceBuilderNodeId> {
    tokens
        .iter()
        .scan(start, |cursor, (kind, text)| {
            let token_start = *cursor;
            let token_end = token_start + text.len();
            *cursor = token_end + 1;
            Some(builder.add_token(*kind, *text, range(source_id, token_start, token_end)))
        })
        .collect()
}

fn collect(
    source_id: SourceId,
    shells: &DeclarationShellSet,
    projections: &[SymbolDeclarationProjection],
) -> SymbolCollectionResult {
    let module = module_id();
    SymbolCollector::new(source_id, &module, shells, projections).collect()
}

fn candidate_source_ranges(
    result: &SymbolCollectionResult,
    candidates: &[SymbolId],
) -> Vec<SourceRange> {
    candidates
        .iter()
        .map(|candidate| {
            let definition = result
                .env()
                .definitions()
                .by_symbol(candidate)
                .expect("diagnostic candidate should retain its definition");
            match definition.origin().anchor() {
                SourceAnchor::Range(range) => *range,
                _ => panic!("test definition should retain a source range"),
            }
        })
        .collect()
}

fn projection(
    shell: DeclarationShellId,
    namespace: NamespacePath,
    spelling: &str,
    symbol_kind: SymbolKind,
    definition_kind: DefinitionKind,
) -> SymbolDeclarationProjection {
    SymbolDeclarationProjection::new(shell, namespace, spelling, symbol_kind)
        .with_definition_kind(definition_kind)
}

fn shells_for(source_id: SourceId, items: Vec<TestItem>) -> DeclarationShellSet {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let item_nodes = items
        .into_iter()
        .map(|item| item.build(&mut builder, source_id))
        .collect();
    let root = finish_module(&mut builder, source_id, item_nodes);
    let ast = builder.finish(Some(root), None);
    DeclarationShellCollector::new(&ast, &module_id()).collect()
}

enum TestItem {
    Node {
        start: usize,
        kind: SurfaceNodeKind,
    },
    Visible {
        start: usize,
        spelling: &'static str,
        target_kind: SurfaceNodeKind,
    },
}

impl TestItem {
    fn build(self, builder: &mut SurfaceAstBuilder, source_id: SourceId) -> SurfaceBuilderNodeId {
        match self {
            Self::Node { start, kind } => {
                node(builder, kind, source_id, start, start + 5, Vec::new())
            }
            Self::Visible {
                start,
                spelling,
                target_kind,
            } => {
                let marker = visibility_marker(builder, source_id, start, spelling);
                let target_start = start + spelling.len() + 1;
                let target = node(
                    builder,
                    target_kind,
                    source_id,
                    target_start,
                    target_start + 5,
                    Vec::new(),
                );
                node(
                    builder,
                    SurfaceNodeKind::VisibleItem,
                    source_id,
                    start,
                    target_start + 5,
                    vec![marker, target],
                )
            }
        }
    }
}

const fn test_item(start: usize, kind: SurfaceNodeKind) -> TestItem {
    TestItem::Node { start, kind }
}

const fn visible_test_item(
    start: usize,
    spelling: &'static str,
    target_kind: SurfaceNodeKind,
) -> TestItem {
    TestItem::Visible {
        start,
        spelling,
        target_kind,
    }
}

fn visibility_marker(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    spelling: &str,
) -> SurfaceBuilderNodeId {
    let token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        spelling,
        range(source_id, start, start + spelling.len()),
    );
    node(
        builder,
        SurfaceNodeKind::VisibilityMarker,
        source_id,
        start,
        start + spelling.len(),
        vec![token],
    )
}

fn finish_module(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    items: Vec<SurfaceBuilderNodeId>,
) -> SurfaceBuilderNodeId {
    let item_list = node(builder, SurfaceNodeKind::ItemList, source_id, 0, 200, items);
    let unit = node(
        builder,
        SurfaceNodeKind::CompilationUnit,
        source_id,
        0,
        200,
        vec![item_list],
    );
    node(
        builder,
        SurfaceNodeKind::Root,
        source_id,
        0,
        200,
        vec![unit],
    )
}

fn node(
    builder: &mut SurfaceAstBuilder,
    kind: SurfaceNodeKind,
    source_id: SourceId,
    start: usize,
    end: usize,
    children: Vec<SurfaceBuilderNodeId>,
) -> SurfaceBuilderNodeId {
    builder.add_node(kind, range(source_id, start, end), children)
}

fn module_id() -> ModuleId {
    ModuleId::new(PackageId::new("app"), ModulePath::new("main"))
}

fn source_id() -> SourceId {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "05".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id)
        .unwrap()
}

fn functor_signature_key(
    argument_context: &str,
    pattern: &str,
    return_type: &str,
) -> FunctorSignatureKey {
    FunctorSignatureKey {
        argument_context: argument_context.to_owned(),
        pattern: pattern.to_owned(),
        arity: Some(1),
        return_type: return_type.to_owned(),
    }
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
