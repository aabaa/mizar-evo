use super::*;
use crate::declarations::DeclarationShellCollector;
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
};
use mizar_syntax::{
    SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind, SyntaxRecoveryKind,
};

#[test]
fn task33r_builds_exact_containing_functor_owner() {
    let (ast, module, resolved, resolver) = task33r_owner_fixture();
    let handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(handoff.surface_fingerprint(), ast.snapshot_text());
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-nested-fraenkel-functor-owner-v1|module=pkg.nested_multi_capture|functor-node={}|symbol={}|definition={}",
            handoff.functor_definition().index(),
            handoff.symbol().fqn().as_str(),
            handoff.definition().index(),
        )
    );
    assert_eq!(
        handoff.dependencies.allocation.shell,
        handoff.declaration_shell()
    );
    assert_eq!(handoff.dependencies.allocation.symbol, *handoff.symbol());
    assert_eq!(
        handoff.dependencies.allocation.definition,
        Some(handoff.definition())
    );
    assert_eq!(
        handoff.dependencies.allocation.contribution,
        handoff.contribution()
    );
    assert_eq!(handoff.dependencies.allocation.origin, *handoff.origin());
    handoff.validate_complete().unwrap();
    handoff.validate_resolver_collection(&resolver).unwrap();

    fn assert_error<T: std::error::Error>() {}
    assert_error::<SourceNestedFraenkelFunctorOwnerError>();
    let displays = [
        (
            SourceNestedFraenkelFunctorOwnerError::InvalidDependency,
            "nested Fraenkel functor owner dependency is invalid",
        ),
        (
            SourceNestedFraenkelFunctorOwnerError::InvalidResolverProfile,
            "nested Fraenkel functor owner resolver profile is invalid",
        ),
        (
            SourceNestedFraenkelFunctorOwnerError::InvalidOwnerCardinality,
            "nested Fraenkel functor owner cardinality is invalid",
        ),
        (
            SourceNestedFraenkelFunctorOwnerError::InvalidOwnerProvenance,
            "nested Fraenkel functor owner provenance is invalid",
        ),
        (
            SourceNestedFraenkelFunctorOwnerError::InvalidSymbolAssociation,
            "nested Fraenkel functor owner symbol association is invalid",
        ),
        (
            SourceNestedFraenkelFunctorOwnerError::InvalidDefinitionAssociation,
            "nested Fraenkel functor owner definition association is invalid",
        ),
        (
            SourceNestedFraenkelFunctorOwnerError::InvalidAssociation,
            "nested Fraenkel functor owner association is invalid",
        ),
    ];
    for (error, display) in displays {
        assert_eq!(error.to_string(), display);
    }

    let alternate_ast =
        task33r_owner_ast_with_shape(ast.source_id, Task33rOwnerShape::AlternateSpelling);
    let (alternate_resolved, alternate_resolver) = task33r_resolver_for(&alternate_ast, &module);
    let alternate = SourceNestedFraenkelFunctorOwnerProducer::build(
        &alternate_ast,
        &module,
        &alternate_resolved,
        &alternate_resolver,
    )
    .unwrap();
    assert_ne!(
        alternate.surface_fingerprint(),
        handoff.surface_fingerprint()
    );
    assert_ne!(alternate.symbol(), handoff.symbol());

    let pressured_ast =
        task33r_owner_ast_with_shape(ast.source_id, Task33rOwnerShape::PrecedingPredicate);
    let (pressured_resolved, pressured_resolver) = task33r_resolver_for(&pressured_ast, &module);
    let pressured = SourceNestedFraenkelFunctorOwnerProducer::build(
        &pressured_ast,
        &module,
        &pressured_resolved,
        &pressured_resolver,
    )
    .unwrap();
    assert!(pressured.definition().index() > 0);
    assert_eq!(
        pressured.dependencies.allocation.definition,
        Some(pressured.definition())
    );
}

#[test]
fn task33r_rejects_dependency_and_resolver_profile_mismatch() {
    let (ast, module, resolved, resolver) = task33r_owner_fixture();
    let wrong_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
    assert_eq!(
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &wrong_module, &resolved, &resolver,)
            .err()
            .unwrap(),
        SourceNestedFraenkelFunctorOwnerError::InvalidDependency
    );

    let foreign_ast = task33r_owner_ast(task33r_foreign_source_id());
    let (foreign_resolved, foreign_resolver) = task33r_resolver_for(&foreign_ast, &module);
    assert_eq!(
        SourceNestedFraenkelFunctorOwnerProducer::build(
            &ast,
            &module,
            &foreign_resolved,
            &foreign_resolver,
        )
        .err()
        .unwrap(),
        SourceNestedFraenkelFunctorOwnerError::InvalidDependency
    );

    let stale_ast =
        task33r_owner_ast_with_shape(ast.source_id, Task33rOwnerShape::StaleDefinitionSpelling);
    let (stale_resolved, stale_resolver) = task33r_resolver_for(&stale_ast, &module);
    assert_eq!(
        SourceNestedFraenkelFunctorOwnerProducer::build(&stale_ast, &module, &resolved, &resolver,)
            .err()
            .unwrap(),
        SourceNestedFraenkelFunctorOwnerError::InvalidDependency
    );
    let handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    assert_eq!(
        handoff.validate_resolver_collection(&stale_resolver),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidDependency)
    );
    assert!(
        SourceNestedFraenkelFunctorOwnerProducer::build(
            &stale_ast,
            &module,
            &stale_resolved,
            &stale_resolver,
        )
        .is_ok()
    );

    let empty_ast = task33r_empty_ast(ast.source_id);
    let empty_resolved = SurfaceResolvedArena::lower(&empty_ast, &module).unwrap();
    let empty_resolver =
        FraenkelGeneratorVariableSourceCollector::new(&empty_ast, &module, &empty_resolved)
            .unwrap()
            .collect()
            .unwrap();
    assert_eq!(
        SourceNestedFraenkelFunctorOwnerProducer::build(
            &empty_ast,
            &module,
            &empty_resolved,
            &empty_resolver,
        )
        .err()
        .unwrap(),
        SourceNestedFraenkelFunctorOwnerError::InvalidResolverProfile
    );

    for shape in [
        Task33rOwnerShape::MissingGenerator,
        Task33rOwnerShape::ExtraGenerator,
        Task33rOwnerShape::DuplicateGenerator,
        Task33rOwnerShape::ReorderedGenerators,
        Task33rOwnerShape::PartialMapper,
        Task33rOwnerShape::OrphanedComprehension,
    ] {
        let malformed_ast = task33r_owner_ast_with_shape(ast.source_id, shape);
        let (malformed_resolved, malformed_resolver) =
            task33r_resolver_for(&malformed_ast, &module);
        assert_eq!(
            SourceNestedFraenkelFunctorOwnerProducer::build(
                &malformed_ast,
                &module,
                &malformed_resolved,
                &malformed_resolver,
            )
            .err()
            .unwrap(),
            SourceNestedFraenkelFunctorOwnerError::InvalidResolverProfile,
            "unexpected disposition for {shape:?}",
        );
    }
}

#[test]
fn task33r_rejects_owner_cardinality_and_provenance() {
    let (ast, module, resolved, resolver) = task33r_owner_fixture();
    assert!(exact_owner_profile(&resolver).is_some());

    let mut handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    handoff.dependencies.shells = DeclarationShellSet::default();
    assert_eq!(
        handoff.validate_complete(),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidDependency)
    );

    let cardinality_ast =
        task33r_owner_ast_with_shape(ast.source_id, Task33rOwnerShape::ExtraFunctor);
    let (cardinality_resolved, cardinality_resolver) =
        task33r_resolver_for(&cardinality_ast, &module);
    assert_eq!(
        SourceNestedFraenkelFunctorOwnerProducer::build(
            &cardinality_ast,
            &module,
            &cardinality_resolved,
            &cardinality_resolver,
        )
        .err()
        .unwrap(),
        SourceNestedFraenkelFunctorOwnerError::InvalidOwnerCardinality
    );

    let recovered_ast =
        task33r_owner_ast_with_shape(ast.source_id, Task33rOwnerShape::RecoveredOwner);
    let (recovered_resolved, recovered_resolver) = task33r_resolver_for(&recovered_ast, &module);
    assert_eq!(
        SourceNestedFraenkelFunctorOwnerProducer::build(
            &recovered_ast,
            &module,
            &recovered_resolved,
            &recovered_resolver,
        )
        .err()
        .unwrap(),
        SourceNestedFraenkelFunctorOwnerError::InvalidResolverProfile
    );

    let wrong_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
    let shells = DeclarationShellCollector::new(&ast, &wrong_module).collect();
    let projections = SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(wrong_module.path().as_str()),
    )
    .extract();
    let profile = exact_owner_profile(&resolver).unwrap();
    let shell = shells
        .declarations()
        .iter()
        .find(|shell| shell.kind() == DeclarationShellKind::FunctorDefinition)
        .unwrap();
    let parent = shells.declaration(shell.parent().unwrap()).unwrap();
    let projection = projections
        .iter()
        .find(|projection| projection.shell() == shell.id())
        .unwrap();
    assert_eq!(
        validate_owner_provenance(shell, parent, &module, &resolved, profile, projection,),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidOwnerProvenance)
    );
}

#[test]
fn task33r_rejects_symbol_definition_and_retained_association_corruption() {
    let (ast, module, resolved, resolver) = task33r_owner_fixture();
    let mut handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    handoff.dependencies.allocation.definition = None;
    assert_eq!(
        handoff.validate_complete(),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidAssociation)
    );

    let handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    let mut allocation = handoff.dependencies.allocation.clone();
    allocation.symbol = SymbolId::new(
        module.clone(),
        LocalSymbolId::new("foreign"),
        FullyQualifiedName::new("pkg.nested_multi_capture.foreign"),
    );
    assert_eq!(
        validate_owner_allocation(
            &handoff.dependencies.symbols,
            &allocation,
            &module,
            ast.source_id,
        ),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidSymbolAssociation)
    );

    let mut allocation = handoff.dependencies.allocation.clone();
    allocation.definition = None;
    assert_eq!(
        validate_owner_allocation(
            &handoff.dependencies.symbols,
            &allocation,
            &module,
            ast.source_id,
        ),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidDefinitionAssociation)
    );

    let mut handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    handoff.association.origin = handoff.association.origin.clone().recovered();
    assert_eq!(
        handoff.validate_complete(),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidAssociation)
    );
}

#[test]
fn task33r_enforces_default_deny_precedence_and_replay() {
    let (ast, module, resolved, resolver) = task33r_owner_fixture();
    let mut handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    handoff.dependencies.version = "stale";
    handoff.association.surface_fingerprint = "stale".to_owned();
    assert_eq!(
        handoff.validate_complete(),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidDependency)
    );

    let mut handoff =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    handoff.dependencies.domain = "stale";
    assert_eq!(
        handoff.validate_complete(),
        Err(SourceNestedFraenkelFunctorOwnerError::InvalidDependency)
    );

    let rebuilt =
        SourceNestedFraenkelFunctorOwnerProducer::build(&ast, &module, &resolved, &resolver)
            .unwrap();
    assert!(
        rebuilt
            == SourceNestedFraenkelFunctorOwnerProducer::build(
                &ast, &module, &resolved, &resolver,
            )
            .unwrap()
    );
}

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
fn selectors_in_distinct_structure_owners_do_not_conflict() {
    let source_id = source_id();
    let ast = task263_distinct_structure_ast(source_id);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let declarations = shells.declarations();

    assert_eq!(declarations.len(), 7);
    assert_eq!(
        declarations[0].kind(),
        DeclarationShellKind::DefinitionBlock
    );
    assert_eq!(
        declarations[1].kind(),
        DeclarationShellKind::StructureDefinition
    );
    assert_eq!(declarations[2].kind(), DeclarationShellKind::StructureField);
    assert_eq!(
        declarations[3].kind(),
        DeclarationShellKind::StructureProperty
    );
    assert_eq!(
        declarations[4].kind(),
        DeclarationShellKind::StructureDefinition
    );
    assert_eq!(declarations[5].kind(), DeclarationShellKind::StructureField);
    assert_eq!(
        declarations[6].kind(),
        DeclarationShellKind::StructureProperty
    );
    assert_eq!(declarations[2].parent(), Some(declarations[1].id()));
    assert_eq!(declarations[3].parent(), Some(declarations[1].id()));
    assert_eq!(declarations[5].parent(), Some(declarations[4].id()));
    assert_eq!(declarations[6].parent(), Some(declarations[4].id()));

    let namespace = NamespacePath::new("main");
    let projections = SignatureProjectionExtractor::new(&ast, &shells, namespace.clone()).extract();
    let reversed = projections.iter().rev().cloned().collect::<Vec<_>>();

    assert_eq!(projections.len(), 6);
    let result = collect(source_id, &shells, &projections);
    let reordered = collect(source_id, &shells, &reversed);

    assert!(result.diagnostics().is_empty());
    assert_eq!(result.diagnostics(), reordered.diagnostics());
    assert_eq!(result.env().symbols().len(), 6);
    assert_eq!(reordered.env().symbols().len(), 6);
    assert_eq!(
        result
            .env()
            .symbols()
            .iter()
            .map(|entry| entry.symbol())
            .collect::<Vec<_>>(),
        reordered
            .env()
            .symbols()
            .iter()
            .map(|entry| entry.symbol())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        result
            .env()
            .symbols()
            .visible_candidates(&namespace, "carrier")
            .len(),
        2
    );
    assert_eq!(
        result
            .env()
            .symbols()
            .visible_candidates(&namespace, "marker")
            .len(),
        2
    );
    let conflicts = definition_conflicts(&result);
    assert!(conflicts.iter().all(|(_, conflict)| conflict.is_none()));
    assert_eq!(conflicts, definition_conflicts(&reordered));
}

#[test]
fn field_and_property_in_same_structure_keep_duplicate_conflict() {
    let source_id = source_id();
    let ast = task263_same_structure_collision_ast(source_id);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let declarations = shells.declarations();

    assert_eq!(declarations.len(), 4);
    assert_eq!(
        declarations[0].kind(),
        DeclarationShellKind::DefinitionBlock
    );
    assert_eq!(
        declarations[1].kind(),
        DeclarationShellKind::StructureDefinition
    );
    assert_eq!(declarations[2].kind(), DeclarationShellKind::StructureField);
    assert_eq!(
        declarations[3].kind(),
        DeclarationShellKind::StructureProperty
    );
    assert_eq!(declarations[2].parent(), Some(declarations[1].id()));
    assert_eq!(declarations[3].parent(), Some(declarations[1].id()));
    assert_eq!(declarations[2].range(), range(source_id, 47, 70));

    let projections =
        SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new("main")).extract();
    let reversed = projections.iter().rev().cloned().collect::<Vec<_>>();

    assert_eq!(projections.len(), 3);
    let result = collect(source_id, &shells, &projections);
    let reordered = collect(source_id, &shells, &reversed);

    assert_eq!(result.diagnostics(), reordered.diagnostics());
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.env().symbols().len(), 3);
    let diagnostic = &result.diagnostics()[0];
    assert_eq!(
        diagnostic.class(),
        SymbolDiagnosticClass::DuplicateDeclaration
    );
    assert_eq!(diagnostic.shell(), Some(declarations[2].id()));
    assert_eq!(diagnostic.range(), range(source_id, 47, 70));
    assert_eq!(diagnostic.spelling(), "duplicate");
    assert_eq!(
        candidate_source_ranges(&result, diagnostic.candidates()),
        vec![range(source_id, 47, 70), range(source_id, 75, 101)]
    );
    assert_eq!(
        result
            .env()
            .definitions()
            .iter()
            .filter(|definition| {
                definition.conflict() == Some(&DeclarationConflictClass::DuplicateSpelling)
            })
            .count(),
        2
    );
    assert_eq!(
        definition_conflicts(&result),
        definition_conflicts(&reordered)
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
fn property_implementation_shells_preserve_symbol_fingerprints_and_context_guards() {
    let source_id = source_id();
    let ast = task264_property_recovery_ast(source_id);
    let module = module_id();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let namespace = NamespacePath::new("main");
    let projections = SignatureProjectionExtractor::new(&ast, &shells, namespace.clone()).extract();

    assert_eq!(shells.declarations().len(), 6);
    assert_eq!(projections.len(), 2);
    let property_shells = shells
        .declarations()
        .iter()
        .filter(|shell| shell.kind() == DeclarationShellKind::PropertyImplementation)
        .collect::<Vec<_>>();
    assert_eq!(property_shells.len(), 3);
    assert!(projections.iter().all(|projection| {
        property_shells
            .iter()
            .all(|shell| shell.id() != projection.shell())
    }));

    let result = collect(source_id, &shells, &projections);
    assert!(result.diagnostics().is_empty());
    assert_eq!(result.env().symbols().len(), 2);
    assert_eq!(result.env().definitions().len(), 2);
    assert!(result.env().registrations().is_empty());
    assert!(result.env().overloads().is_empty());
    assert!(result.env().lexical_summaries().is_empty());
    assert!(result.env().declaration_dependencies().is_empty());
    let theorem = result
        .env()
        .symbols()
        .iter()
        .find(|entry| entry.primary_spelling() == "AfterRecovery")
        .expect("following theorem symbol");
    assert!(
        theorem
            .symbol()
            .local()
            .as_str()
            .contains("owner=theorem#1")
    );
    assert_eq!(
        theorem.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 304, 334))
    );
    assert_eq!(theorem.origin().structural_path(), &[2, 1]);
    let theorem_definition = result
        .env()
        .definitions()
        .iter()
        .find(|entry| entry.kind() == DefinitionKind::Theorem)
        .expect("following theorem definition");
    assert_eq!(theorem_definition.id().index(), 1);
    assert_eq!(theorem_definition.symbol(), theorem.symbol());

    assert_eq!(result.env().contributions().iter().count(), 1);
    let contribution = result.env().contributions().iter().next().unwrap();
    assert_eq!(contribution.id().index(), 0);
    assert_eq!(
        contribution.anchor(),
        &SourceAnchor::Range(range(source_id, 20, 70))
    );
    let expected_symbols = result
        .env()
        .symbols()
        .iter()
        .map(|entry| {
            assert_eq!(entry.contribution().index(), 0);
            entry.symbol().clone()
        })
        .collect::<Vec<_>>();
    let expected_definitions = result
        .env()
        .definitions()
        .iter()
        .map(|entry| {
            assert_eq!(entry.contribution().index(), 0);
            entry.id()
        })
        .collect::<Vec<_>>();
    assert_eq!(contribution.effects().symbols(), expected_symbols);
    assert_eq!(contribution.effects().definitions(), expected_definitions);
    assert!(contribution.effects().overload_groups().is_empty());
    assert!(contribution.effects().registrations().is_empty());
    assert!(contribution.effects().lexical_summaries().is_empty());
    assert!(contribution.effects().labels().is_empty());
    assert!(contribution.effects().namespace_edges().is_empty());
    assert!(contribution.effects().declaration_dependencies().is_empty());
    assert!(contribution.effects().imports().is_empty());
    assert!(contribution.effects().exports().is_empty());
    assert!(contribution.effects().diagnostics().is_empty());

    let mut fabricated = projections.clone();
    fabricated.push(projection(
        property_shells[0].id(),
        namespace,
        "FabricatedProperty",
        SymbolKind::Selector,
        DefinitionKind::Selector,
    ));
    let rejected = collect(source_id, &shells, &fabricated);
    assert_eq!(rejected.env().symbols().len(), 2);
    assert_eq!(rejected.env().definitions().len(), 2);
    assert_eq!(rejected.diagnostics().len(), 1);
    assert_eq!(
        rejected.diagnostics()[0].class(),
        SymbolDiagnosticClass::ContextOnlyShell
    );

    let mut property_only_builder = SurfaceAstBuilder::new(source_id);
    let property = node(
        &mut property_only_builder,
        SurfaceNodeKind::PropertyImplementation,
        source_id,
        0,
        10,
        Vec::new(),
    );
    let root = finish_module(&mut property_only_builder, source_id, vec![property]);
    let property_only_ast = property_only_builder.finish(Some(root), None);
    let property_only_shells =
        DeclarationShellCollector::new(&property_only_ast, &module).collect();
    let property_only = collect(source_id, &property_only_shells, &[]);
    assert!(property_only.diagnostics().is_empty());
    assert!(property_only.env().symbols().is_empty());
    assert!(property_only.env().definitions().is_empty());
    assert!(property_only.env().registrations().is_empty());
    assert!(property_only.env().overloads().is_empty());
    assert!(property_only.env().lexical_summaries().is_empty());
    assert!(property_only.env().declaration_dependencies().is_empty());
    assert_eq!(property_only.env().contributions().iter().count(), 1);
    let property_only_contribution = property_only.env().contributions().iter().next().unwrap();
    assert_eq!(property_only_contribution.id().index(), 0);
    assert_eq!(
        property_only_contribution.anchor(),
        &SourceAnchor::Point {
            source_id,
            offset: 0
        }
    );
    assert!(property_only_contribution.effects().symbols().is_empty());
    assert!(
        property_only_contribution
            .effects()
            .definitions()
            .is_empty()
    );
    assert!(
        property_only_contribution
            .effects()
            .overload_groups()
            .is_empty()
    );
    assert!(
        property_only_contribution
            .effects()
            .registrations()
            .is_empty()
    );
    assert!(
        property_only_contribution
            .effects()
            .lexical_summaries()
            .is_empty()
    );
    assert!(property_only_contribution.effects().labels().is_empty());
    assert!(
        property_only_contribution
            .effects()
            .namespace_edges()
            .is_empty()
    );
    assert!(
        property_only_contribution
            .effects()
            .declaration_dependencies()
            .is_empty()
    );
    assert!(property_only_contribution.effects().imports().is_empty());
    assert!(property_only_contribution.effects().exports().is_empty());
    assert!(
        property_only_contribution
            .effects()
            .diagnostics()
            .is_empty()
    );

    let stable_shell_kinds = [
        (DeclarationShellKind::Placeholder, "placeholder"),
        (DeclarationShellKind::Reserve, "reserve"),
        (DeclarationShellKind::Theorem, "theorem"),
        (DeclarationShellKind::Lemma, "lemma"),
        (DeclarationShellKind::DefinitionBlock, "definition-block"),
        (
            DeclarationShellKind::RegistrationBlock,
            "registration-block",
        ),
        (DeclarationShellKind::ClaimBlock, "claim-block"),
        (
            DeclarationShellKind::AttributeDefinition,
            "attribute-definition",
        ),
        (
            DeclarationShellKind::PredicateDefinition,
            "predicate-definition",
        ),
        (
            DeclarationShellKind::FunctorDefinition,
            "functor-definition",
        ),
        (DeclarationShellKind::ModeDefinition, "mode-definition"),
        (
            DeclarationShellKind::StructureDefinition,
            "structure-definition",
        ),
        (
            DeclarationShellKind::AlgorithmDefinition,
            "algorithm-definition",
        ),
        (
            DeclarationShellKind::AttributeRedefinition,
            "attribute-redefinition",
        ),
        (
            DeclarationShellKind::PredicateRedefinition,
            "predicate-redefinition",
        ),
        (
            DeclarationShellKind::FunctorRedefinition,
            "functor-redefinition",
        ),
        (DeclarationShellKind::NotationAlias, "notation-alias"),
        (DeclarationShellKind::PropertyClause, "property-clause"),
        (DeclarationShellKind::StructureField, "structure-field"),
        (
            DeclarationShellKind::StructureProperty,
            "structure-property",
        ),
        (
            DeclarationShellKind::InheritanceDefinition,
            "inheritance-definition",
        ),
        (
            DeclarationShellKind::FieldRedefinition,
            "field-redefinition",
        ),
        (
            DeclarationShellKind::PropertyRedefinition,
            "property-redefinition",
        ),
        (
            DeclarationShellKind::ExistentialRegistration,
            "existential-registration",
        ),
        (
            DeclarationShellKind::ConditionalRegistration,
            "conditional-registration",
        ),
        (
            DeclarationShellKind::FunctorialRegistration,
            "functorial-registration",
        ),
        (
            DeclarationShellKind::ReductionRegistration,
            "reduction-registration",
        ),
        (
            DeclarationShellKind::VisibilityWrapper,
            "visibility-wrapper",
        ),
        (
            DeclarationShellKind::PropertyImplementation,
            "property-implementation",
        ),
    ];
    for (expected_code, (kind, expected_key)) in stable_shell_kinds.into_iter().enumerate() {
        assert_eq!(declaration_shell_kind_code(kind), expected_code as u32);
        assert_eq!(declaration_shell_kind_key(kind), expected_key);
    }
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

#[derive(Debug, Clone, Copy)]
struct Task263StructureShape {
    structure_range: (usize, usize),
    structure_name: (&'static str, usize),
    field_range: (usize, usize),
    field_name: (&'static str, usize),
    property_range: (usize, usize),
    property_name: (&'static str, usize),
}

fn task263_structure_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    shape: Task263StructureShape,
) -> SurfaceBuilderNodeId {
    let structure_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        shape.structure_name.0,
        range(
            source_id,
            shape.structure_name.1,
            shape.structure_name.1 + shape.structure_name.0.len(),
        ),
    );
    let pattern = node(
        builder,
        SurfaceNodeKind::StructurePattern,
        source_id,
        shape.structure_name.1,
        shape.structure_name.1 + shape.structure_name.0.len(),
        vec![structure_name],
    );
    let field_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        shape.field_name.0,
        range(
            source_id,
            shape.field_name.1,
            shape.field_name.1 + shape.field_name.0.len(),
        ),
    );
    let field = node(
        builder,
        SurfaceNodeKind::StructureField,
        source_id,
        shape.field_range.0,
        shape.field_range.1,
        vec![field_name],
    );
    let property_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        shape.property_name.0,
        range(
            source_id,
            shape.property_name.1,
            shape.property_name.1 + shape.property_name.0.len(),
        ),
    );
    let property = node(
        builder,
        SurfaceNodeKind::StructureProperty,
        source_id,
        shape.property_range.0,
        shape.property_range.1,
        vec![property_name],
    );
    node(
        builder,
        SurfaceNodeKind::StructureDefinition,
        source_id,
        shape.structure_range.0,
        shape.structure_range.1,
        vec![pattern, field, property],
    )
}

fn task263_distinct_structure_ast(source_id: SourceId) -> mizar_syntax::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let base = task263_structure_item(
        &mut builder,
        source_id,
        Task263StructureShape {
            structure_range: (13, 98),
            structure_name: ("Task263Base", 20),
            field_range: (42, 63),
            field_name: ("carrier", 48),
            property_range: (68, 91),
            property_name: ("marker", 77),
        },
    );
    let derived = task263_structure_item(
        &mut builder,
        source_id,
        Task263StructureShape {
            structure_range: (102, 190),
            structure_name: ("Task263Derived", 109),
            field_range: (134, 155),
            field_name: ("carrier", 140),
            property_range: (160, 183),
            property_name: ("marker", 169),
        },
    );
    let definition = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        0,
        191,
        vec![base, derived],
    );
    let root = finish_module(&mut builder, source_id, vec![definition]);
    builder.finish(Some(root), None)
}

fn task263_same_structure_collision_ast(source_id: SourceId) -> mizar_syntax::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let structure = task263_structure_item(
        &mut builder,
        source_id,
        Task263StructureShape {
            structure_range: (13, 108),
            structure_name: ("Task263Collision", 20),
            field_range: (47, 70),
            field_name: ("duplicate", 53),
            property_range: (75, 101),
            property_name: ("duplicate", 84),
        },
    );
    let definition = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        0,
        113,
        vec![structure],
    );
    let root = finish_module(&mut builder, source_id, vec![definition]);
    builder.finish(Some(root), None)
}

fn task264_property_recovery_ast(source_id: SourceId) -> mizar_syntax::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let first_property = node(
        &mut builder,
        SurfaceNodeKind::PropertyImplementation,
        source_id,
        0,
        10,
        Vec::new(),
    );
    let mode = pattern_item(
        &mut builder,
        source_id,
        33,
        SurfaceNodeKind::ModeDefinition,
        SurfaceNodeKind::ModePattern,
        &[(SurfaceTokenKind::Identifier, "Domain")],
    );
    let definition = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        20,
        70,
        vec![mode],
    );
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::MissingTerm,
        range(source_id, 170, 171),
        Vec::new(),
    );
    let recovered_property = node(
        &mut builder,
        SurfaceNodeKind::PropertyImplementation,
        source_id,
        140,
        200,
        vec![recovery],
    );
    let final_property = node(
        &mut builder,
        SurfaceNodeKind::PropertyImplementation,
        source_id,
        220,
        300,
        Vec::new(),
    );
    let theorem_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "AfterRecovery",
        range(source_id, 312, 325),
    );
    let theorem_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 325, 326),
    );
    let theorem_formula = node(
        &mut builder,
        SurfaceNodeKind::FormulaExpression,
        source_id,
        328,
        334,
        Vec::new(),
    );
    let theorem = node(
        &mut builder,
        SurfaceNodeKind::TheoremItem,
        source_id,
        304,
        334,
        vec![theorem_label, theorem_colon, theorem_formula],
    );
    let root = finish_module(
        &mut builder,
        source_id,
        vec![
            first_property,
            definition,
            recovered_property,
            final_property,
            theorem,
        ],
    );
    builder.finish(Some(root), None)
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

fn definition_conflicts(
    result: &SymbolCollectionResult,
) -> Vec<(SymbolId, Option<DeclarationConflictClass>)> {
    result
        .env()
        .definitions()
        .iter()
        .map(|definition| (definition.symbol().clone(), definition.conflict().cloned()))
        .collect()
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

fn task33r_owner_fixture() -> (
    SurfaceAst,
    ModuleId,
    SurfaceResolvedArena,
    FraenkelGeneratorVariableSourceCollection,
) {
    let source_id = source_id();
    let module = ModuleId::new(
        PackageId::new("pkg"),
        ModulePath::new("nested_multi_capture"),
    );
    let ast = task33r_owner_ast(source_id);
    let (resolved, resolver) = task33r_resolver_for(&ast, &module);
    (ast, module, resolved, resolver)
}

fn task33r_resolver_for(
    ast: &SurfaceAst,
    module: &ModuleId,
) -> (
    SurfaceResolvedArena,
    FraenkelGeneratorVariableSourceCollection,
) {
    let resolved = SurfaceResolvedArena::lower(ast, module).unwrap();
    let resolver = FraenkelGeneratorVariableSourceCollector::new(ast, module, &resolved)
        .unwrap()
        .collect()
        .unwrap();
    (resolved, resolver)
}

fn task33r_foreign_source_id() -> SourceId {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "06".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    let _original = allocator.next_source_id(snapshot_id).unwrap();
    allocator.next_source_id(snapshot_id).unwrap()
}

fn task33r_empty_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let root = builder.add_node(SurfaceNodeKind::Root, range(source_id, 0, 1), Vec::new());
    builder.finish(Some(root), None)
}

fn task33r_owner_ast(source_id: SourceId) -> SurfaceAst {
    task33r_owner_ast_with_shape(source_id, Task33rOwnerShape::Exact)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task33rOwnerShape {
    Exact,
    StaleDefinitionSpelling,
    AlternateSpelling,
    MissingGenerator,
    ExtraGenerator,
    DuplicateGenerator,
    ReorderedGenerators,
    PartialMapper,
    OrphanedComprehension,
    ExtraFunctor,
    RecoveredOwner,
    PrecedingPredicate,
}

fn task33r_owner_ast_with_shape(source_id: SourceId, shape: Task33rOwnerShape) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        if shape == Task33rOwnerShape::StaleDefinitionSpelling {
            "definition-stale"
        } else {
            "definition"
        },
        range(source_id, 39, 49),
    );
    let func = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        if shape == Task33rOwnerShape::AlternateSpelling {
            "func-alternate"
        } else {
            "func"
        },
        range(source_id, 52, 56),
    );
    let outer_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "{",
        range(source_id, 93, 94),
    );
    let inner_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "{",
        range(source_id, 95, 96),
    );
    let mapper_x = task33r_identifier(&mut builder, source_id, "x", 98);
    let mapper_y = task33r_identifier(&mut builder, source_id, "y", 101);
    let open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "[",
        range(source_id, 97, 98),
    );
    let comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 100, 101),
    );
    let close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "]",
        range(source_id, 102, 103),
    );
    let application_children = if shape == Task33rOwnerShape::PartialMapper {
        vec![open, mapper_x, close]
    } else {
        vec![open, mapper_x, comma, mapper_y, close]
    };
    let application = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        range(source_id, 97, 103),
        application_children,
    );
    let inner_mapper = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 97, 103),
        vec![application],
    );
    let inner_where = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "where",
        range(source_id, 104, 109),
    );
    let inner_binder = builder.add_token(
        SurfaceTokenKind::Identifier,
        "z",
        range(source_id, 110, 111),
    );
    let inner_is = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 112, 114),
    );
    let inner_type = task33r_type(&mut builder, source_id, 115);
    let inner_segment = builder.add_node(
        SurfaceNodeKind::ComprehensionVariableSegment,
        range(source_id, 110, 129),
        vec![inner_binder, inner_is, inner_type],
    );
    let inner_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 130, 131),
    );
    let inner = builder.add_node(
        SurfaceNodeKind::SetComprehension,
        range(source_id, 95, 131),
        vec![
            inner_open,
            inner_mapper,
            inner_where,
            inner_segment,
            inner_close,
        ],
    );
    let outer_mapper = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 95, 131),
        vec![inner],
    );
    let outer_where = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "where",
        range(source_id, 132, 137),
    );
    let outer_x_binder = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 144, 145),
    );
    let outer_x_is = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 146, 148),
    );
    let outer_x_type = task33r_type(&mut builder, source_id, 149);
    let outer_x_segment = builder.add_node(
        SurfaceNodeKind::ComprehensionVariableSegment,
        range(source_id, 144, 163),
        vec![outer_x_binder, outer_x_is, outer_x_type],
    );
    let outer_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 164, 165),
    );
    let outer_y_binder = builder.add_token(
        SurfaceTokenKind::Identifier,
        "y",
        range(source_id, 165, 166),
    );
    let outer_y_is = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 167, 169),
    );
    let outer_y_type = task33r_type(&mut builder, source_id, 170);
    let outer_y_segment = builder.add_node(
        SurfaceNodeKind::ComprehensionVariableSegment,
        range(source_id, 165, 184),
        vec![outer_y_binder, outer_y_is, outer_y_type],
    );
    let outer_extra_segment = if matches!(
        shape,
        Task33rOwnerShape::ExtraGenerator | Task33rOwnerShape::DuplicateGenerator
    ) {
        let extra_binder = builder.add_token(
            SurfaceTokenKind::Identifier,
            if shape == Task33rOwnerShape::DuplicateGenerator {
                "y"
            } else {
                "q"
            },
            range(source_id, 187, 188),
        );
        let extra_is = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "is",
            range(source_id, 189, 191),
        );
        let extra_type = task33r_type(&mut builder, source_id, 192);
        Some(builder.add_node(
            SurfaceNodeKind::ComprehensionVariableSegment,
            range(source_id, 187, 206),
            vec![extra_binder, extra_is, extra_type],
        ))
    } else {
        None
    };
    let outer_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 185, 186),
    );
    let mut outer_children = vec![outer_open, outer_mapper, outer_where];
    match shape {
        Task33rOwnerShape::MissingGenerator => outer_children.push(outer_x_segment),
        Task33rOwnerShape::ReorderedGenerators => {
            outer_children.extend([outer_y_segment, outer_comma, outer_x_segment]);
        }
        _ => {
            outer_children.extend([outer_x_segment, outer_comma, outer_y_segment]);
            if let Some(extra_segment) = outer_extra_segment {
                let extra_comma = builder.add_token(
                    SurfaceTokenKind::ReservedSymbol,
                    ",",
                    range(source_id, 186, 187),
                );
                outer_children.extend([extra_comma, extra_segment]);
            }
        }
    }
    outer_children.push(outer_close);
    let outer = builder.add_node(
        SurfaceNodeKind::SetComprehension,
        range(source_id, 93, 186),
        outer_children,
    );
    let definiens_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 93, 186),
        vec![outer],
    );
    let definiens = builder.add_node(
        SurfaceNodeKind::TermDefiniens,
        range(source_id, 93, 186),
        vec![definiens_expression],
    );
    let mut functor_children = vec![func, definiens];
    if shape == Task33rOwnerShape::RecoveredOwner {
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::SkippedToken,
            range(source_id, 187, 188),
            Vec::new(),
        );
        functor_children.push(recovery);
    }
    let functor = builder.add_node(
        if shape == Task33rOwnerShape::OrphanedComprehension {
            SurfaceNodeKind::PredicateDefinition
        } else {
            SurfaceNodeKind::FunctorDefinition
        },
        range(source_id, 52, 188),
        functor_children,
    );
    let block_children = vec![definition, functor];
    let block = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source_id, 39, 191),
        block_children,
    );
    let mut root_children = Vec::new();
    if shape == Task33rOwnerShape::PrecedingPredicate {
        let predicate_token = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "pred",
            range(source_id, 1, 5),
        );
        let predicate = builder.add_node(
            SurfaceNodeKind::PredicateDefinition,
            range(source_id, 1, 5),
            vec![predicate_token],
        );
        let predicate_block = builder.add_node(
            SurfaceNodeKind::DefinitionBlockItem,
            range(source_id, 0, 6),
            vec![predicate],
        );
        root_children.push(predicate_block);
    }
    root_children.push(block);
    if shape == Task33rOwnerShape::ExtraFunctor {
        let extra_definition = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "definition",
            range(source_id, 208, 218),
        );
        let extra_func = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "func-extra",
            range(source_id, 219, 229),
        );
        let extra_functor = builder.add_node(
            SurfaceNodeKind::FunctorDefinition,
            range(source_id, 219, 229),
            vec![extra_func],
        );
        let extra_block = builder.add_node(
            SurfaceNodeKind::DefinitionBlockItem,
            range(source_id, 208, 230),
            vec![extra_definition, extra_functor],
        );
        root_children.push(extra_block);
    }
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 221),
        root_children,
    );
    builder.finish(Some(root), None)
}

fn task33r_identifier(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    spelling: &str,
    start: usize,
) -> SurfaceBuilderNodeId {
    let identifier = builder.add_token(
        SurfaceTokenKind::Identifier,
        spelling,
        range(source_id, start, start + 1),
    );
    let reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, start, start + 1),
        vec![identifier],
    );
    builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, start, start + 1),
        vec![reference],
    )
}

fn task33r_type(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
) -> SurfaceBuilderNodeId {
    let element = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "Element",
        range(source_id, start, start + 7),
    );
    let element_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, start, start + 7),
        vec![element],
    );
    let element_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, start, start + 7),
        vec![element_segment],
    );
    let of = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "of",
        range(source_id, start + 8, start + 10),
    );
    let nat = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "NAT",
        range(source_id, start + 11, start + 14),
    );
    let nat_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, start + 11, start + 14),
        vec![nat],
    );
    let nat_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, start + 11, start + 14),
        vec![nat_segment],
    );
    let nat_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, start + 11, start + 14),
        vec![nat_symbol],
    );
    let nat_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, start + 11, start + 14),
        vec![nat_reference],
    );
    let arguments = builder.add_node(
        SurfaceNodeKind::TypeArguments,
        range(source_id, start + 8, start + 14),
        vec![of, nat_expression],
    );
    let head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, start, start + 14),
        vec![element_symbol, arguments],
    );
    builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, start, start + 14),
        vec![head],
    )
}
