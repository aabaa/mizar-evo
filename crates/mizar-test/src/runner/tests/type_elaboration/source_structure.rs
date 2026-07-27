use super::{
    SourceStructureRouteOutput, SyntheticSourceStructureDependencies, source_structure_output,
    source_structure_output_with_mutation, synthetic_source_structure_output,
    synthetic_source_structure_output_with_mutation,
};

#[test]
fn task254_real_route_publishes_exact_aggregate_and_preserves_final_ownership() {
    let (ast, module, shells, symbols) = task254_real_ast();
    let first = source_structure_output(&ast, module.clone(), &shells, &symbols)
        .expect("Task 254 exact selector")
        .unwrap_or_else(|error| panic!("Task 254 real route failed: {error}"));
    let second = source_structure_output(&ast, module, &shells, &symbols)
        .expect("Task 254 exact selector should be deterministic")
        .unwrap_or_else(|error| panic!("Task 254 repeated route failed: {error}"));
    assert_task254_real_oracle(&first);
    assert_task254_real_binding_contexts(&first);
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task254_real_member_container_and_term_key_corruption_fails_atomically() {
    let (ast, module, shells, symbols) = task254_real_ast();
    let corruptions: [fn(&mut mizar_checker::source_structure::SourceStructureHandoffInput); 5] = [
        |input| input.terms[0].source_range = input.terms[1].source_range,
        |input| input.members[0].site = input.members[6].site.clone(),
        |input| input.members[6].site = input.members[8].site.clone(),
        |input| input.field_updates[0].site = input.members[7].site.clone(),
        |input| input.requests.swap(0, 1),
    ];
    for corrupt in corruptions {
        let error =
            source_structure_output_with_mutation(&ast, module.clone(), &shells, &symbols, corrupt)
                .expect("corruption must not change exact selection")
                .expect_err("corrupt Task 254 transaction must fail atomically");
        assert!(
            error.contains("structure") || error.contains("field update"),
            "{error}"
        );
    }
    assert!(
        source_structure_output(&ast, module, &shells, &symbols)
            .expect("uncorrupted exact selector")
            .is_ok()
    );
}

#[test]
fn task254_exact_selector_excludes_every_other_active_type_case() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 254 isolation plan should build");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_structure_output(&ast, resolver.module, &resolver.shells, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        ["fail_type_elaboration_local_structure_term_gap_001"]
    );
}

#[test]
fn task254_synthetic_entry_points_keep_dependency_and_mutation_boundaries_explicit() {
    let (ast, module, _, symbols) = task254_real_ast();
    let binding_env = task254_module_binding_env(&ast, module.clone());
    let roots = task254_outer_structure_roots(&ast);
    let output = synthetic_source_structure_output(
        &ast,
        module.clone(),
        binding_env.clone(),
        &symbols,
        &roots,
        None::<SyntheticSourceStructureDependencies>,
        &BTreeSet::new(),
    )
    .expect("synthetic Task 254 route");
    assert_task254_real_oracle(&output);
    let error = synthetic_source_structure_output_with_mutation(
        &ast,
        module,
        binding_env,
        &symbols,
        &roots,
        None::<SyntheticSourceStructureDependencies>,
        &BTreeSet::new(),
        |input| {
            input.members[0].role =
                mizar_checker::source_structure::SourceStructureMemberRole::Selector
        },
    )
    .expect_err("synthetic key/role substitution must fail");
    assert!(error.contains("structure"), "{error}");
}

#[test]
fn task254_synthetic_application_fingerprint_root_and_unrelated_matrix() {
    let (ast, module, _, symbols) = task254_real_ast();
    let roots = task254_outer_structure_roots(&ast);
    let numeral_three = task254_term_expression_with_spelling(&ast, "3");
    let positive_dependencies =
        task254_bare_application_dependencies(&ast, &module, &symbols, numeral_three, Some("3"));
    let positive = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module.clone()),
        &symbols,
        &roots,
        Some(positive_dependencies.clone()),
        &BTreeSet::new(),
    )
    .expect("Task 254 should consume a Task 253 root application");
    let structure = positive.typed_ast.source_structure().expect("Task 254");
    assert!(structure.application_fingerprint().is_some());
    assert_eq!(
        positive
            .typed_ast
            .source_application()
            .expect("targeted Task 253")
            .debug_text(),
        structure
            .application_fingerprint()
            .expect("Task 254 application fingerprint")
    );
    assert_eq!(
        positive.typed_ast.source_application(),
        positive.resolved.source_application()
    );
    assert_eq!(
        positive.typed_ast.source_structure(),
        positive.resolved.source_structure()
    );
    assert_eq!(
        positive.typed_ast.source_term(),
        positive.resolved.source_term()
    );
    assert_eq!(
        structure
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Application(_)
            ))
            .count(),
        1
    );
    assert_eq!(
        positive
            .typed_ast
            .source_term()
            .expect("Task 252")
            .terms()
            .len(),
        7
    );

    let unrelated_roots = [roots[0], roots[2]];
    let unrelated = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &symbols,
        &unrelated_roots,
        Some(positive_dependencies),
        &BTreeSet::new(),
    )
    .expect("unrelated Task 253 may coexist with Task 254");
    assert_eq!(
        unrelated
            .typed_ast
            .source_structure()
            .expect("Task 254")
            .application_fingerprint(),
        None
    );
    assert!(unrelated.typed_ast.source_application().is_some());
    assert_eq!(
        unrelated.typed_ast.source_application(),
        unrelated.resolved.source_application()
    );
    assert_eq!(
        unrelated.typed_ast.source_structure(),
        unrelated.resolved.source_structure()
    );
    assert_eq!(
        unrelated.typed_ast.source_term(),
        unrelated.resolved.source_term()
    );
    assert_eq!(
        unrelated
            .typed_ast
            .source_term()
            .expect("Task 252")
            .terms()
            .len(),
        7
    );
}

#[test]
fn task254_synthetic_structure_syntax_matrix_preserves_written_shape() {
    let (real_ast, module, _, symbols) = task254_real_ast();
    let source = real_ast.source_id;
    let mut syntax = Task254SyntheticSyntax::new(source);

    let zero = syntax.constructor(Vec::new(), false);
    let repeated = {
        let one = syntax.primary("1");
        let two = syntax.primary("2");
        let three = syntax.primary("3");
        syntax.constructor(
            vec![("marker", one), ("carrier", two), ("marker", three)],
            false,
        )
    };
    let nested_value = {
        let child = syntax.constructor(Vec::new(), false);
        syntax.constructor(vec![("carrier", child)], false)
    };
    let selector_chain = {
        let base = syntax.constructor(Vec::new(), false);
        let inner = syntax.selector(base, "carrier", Vec::new());
        let primary = syntax.primary("4");
        let structure = syntax.constructor(Vec::new(), false);
        syntax.selector(inner, "marker", vec![primary, structure])
    };
    let zero_selector_call = {
        let base = syntax.constructor(Vec::new(), false);
        syntax.selector_zero_call(base, "carrier")
    };
    let update = {
        let base = syntax.constructor(Vec::new(), false);
        let primary = syntax.primary("5");
        let replacement = syntax.constructor(Vec::new(), false);
        let repeated_value = syntax.primary("6");
        syntax.update(
            base,
            vec![
                (vec!["start", "x"], primary),
                (vec!["marker"], replacement),
                (vec!["start", "x"], repeated_value),
            ],
        )
    };
    syntax.gap(4);
    let wrapped_core = syntax.constructor(Vec::new(), false);
    let inner_wrapper = syntax.wrapper(wrapped_core);
    let wrapped = syntax.wrapper(inner_wrapper);
    let wrapped_core_range = syntax.range(wrapped_core);
    let root_specs = [
        (syntax.range(zero), SurfaceNodeKind::StructureConstructor),
        (
            syntax.range(repeated),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(nested_value),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(selector_chain),
            SurfaceNodeKind::SelectorAccess,
        ),
        (
            syntax.range(zero_selector_call),
            SurfaceNodeKind::SelectorAccess,
        ),
        (syntax.range(update), SurfaceNodeKind::StructureUpdate),
        (syntax.range(wrapped), SurfaceNodeKind::ParenthesizedTerm),
    ];
    let ast = syntax.finish(vec![
        zero,
        repeated,
        nested_value,
        selector_chain,
        zero_selector_call,
        update,
        wrapped,
    ]);
    let roots =
        root_specs.map(|(range, kind)| task254_node_with_range_and_kind(&ast, range, &kind));
    let wrapped_core_index = task254_node_with_range_and_kind(
        &ast,
        wrapped_core_range,
        &SurfaceNodeKind::StructureConstructor,
    );
    let degraded = BTreeSet::from([wrapped_core_index]);
    let output = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &symbols,
        &roots,
        None,
        &degraded,
    )
    .expect("Task 254 synthetic syntax matrix");
    let handoff = output.typed_ast.source_structure().expect("Task 254");

    assert!(handoff.terms().iter().any(|(_, term)| {
        term.kind() == mizar_checker::source_structure::SourceStructureTermKind::Constructor
            && handoff.members().iter().all(|(_, member)| {
                member.term()
                    != mizar_checker::source_structure::SourceStructureTermId::new(
                        term.source_ordinal(),
                    )
            })
    }));
    assert_eq!(
        handoff
            .members()
            .iter()
            .filter(|(_, member)| member.spelling() == "marker")
            .count(),
        4
    );
    assert!(handoff.members().iter().any(|(_, member)| {
        member.spelling() == "x"
            && member.role()
                == mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment
            && member.parent().is_some()
    }));
    assert_eq!(
        handoff
            .field_updates()
            .iter()
            .filter(|(_, update)| update.spelling().starts_with("start . x :="))
            .count(),
        2
    );
    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Structure(_)
            ))
            .count(),
        7
    );
    assert!(handoff.edges().iter().any(|(_, edge)| {
        edge.role() == mizar_checker::source_structure::SourceStructureEdgeRole::SelectorArgument
            && matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Primary(_)
            )
    }));
    assert_eq!(handoff.wrappers().len(), 2);
    assert_eq!(
        handoff
            .wrappers()
            .iter()
            .map(|(_, wrapper)| wrapper.ordinal())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(handoff.terms().iter().any(|(_, term)| {
        term.recovery() == mizar_checker::source_structure::SourceStructureRecovery::Degraded
    }));
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
    );
}

#[test]
fn task254_synthetic_task253_targets_cover_selector_argument_and_update_value() {
    let (real_ast, module, _, symbols) = task254_real_ast();
    for (spelling, expected_role, root_kind) in [
        (
            "91",
            mizar_checker::source_structure::SourceStructureEdgeRole::SelectorArgument,
            SurfaceNodeKind::SelectorAccess,
        ),
        (
            "92",
            mizar_checker::source_structure::SourceStructureEdgeRole::UpdateValue,
            SurfaceNodeKind::StructureUpdate,
        ),
    ] {
        let mut syntax = Task254SyntheticSyntax::new(real_ast.source_id);
        let base = syntax.constructor(Vec::new(), false);
        let application = syntax.primary(spelling);
        let application_range = syntax.range(application);
        let root = match root_kind {
            SurfaceNodeKind::SelectorAccess => syntax.selector(base, "carrier", vec![application]),
            SurfaceNodeKind::StructureUpdate => {
                syntax.update(base, vec![(vec!["carrier"], application)])
            }
            _ => unreachable!("Task 254 target matrix root kind"),
        };
        let root_range = syntax.range(root);
        let ast = syntax.finish(vec![root]);
        let application = task254_node_with_range_and_kind(
            &ast,
            application_range,
            &SurfaceNodeKind::TermExpression,
        );
        let root = task254_node_with_range_and_kind(&ast, root_range, &root_kind);
        let dependencies = task254_bare_application_dependencies(
            &ast,
            &module,
            &symbols,
            application,
            Some(spelling),
        );
        let output = synthetic_source_structure_output(
            &ast,
            module.clone(),
            task254_module_binding_env(&ast, module.clone()),
            &symbols,
            &[root],
            Some(dependencies),
            &BTreeSet::new(),
        )
        .expect("Task 254 Task 253 target matrix");
        let handoff = output.typed_ast.source_structure().expect("Task 254");
        assert!(handoff.application_fingerprint().is_some());
        assert!(handoff.edges().iter().any(|(_, edge)| {
            edge.role() == expected_role
                && matches!(
                    edge.target(),
                    mizar_checker::source_structure::SourceStructureTarget::Application(_)
                )
        }));
        assert_eq!(
            output.typed_ast.source_application(),
            output.resolved.source_application()
        );
        assert_eq!(
            output.typed_ast.source_structure(),
            output.resolved.source_structure()
        );
        assert_eq!(
            output.typed_ast.source_term(),
            output.resolved.source_term()
        );
    }
}

#[test]
fn task254_imported_root_provenance_matrix_reaches_source_structure_producer() {
    let (real_ast, module, _, _) = task254_real_ast();
    let mut syntax = Task254SyntheticSyntax::new(real_ast.source_id);
    let constructor = syntax.constructor_named("ImportedPair", Vec::new(), false);
    let constructor_range = syntax.range(constructor);
    let ast = syntax.finish(vec![constructor]);
    let root = task254_node_with_range_and_kind(
        &ast,
        constructor_range,
        &SurfaceNodeKind::StructureConstructor,
    );
    let valid_symbols =
        task254_imported_structure_symbols(&ast, &module, Task254ImportedRootCorruption::None);
    let valid = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module.clone()),
        &valid_symbols,
        &[root],
        None::<SyntheticSourceStructureDependencies>,
        &BTreeSet::new(),
    )
    .expect("valid imported Task 254 root");
    let imported_root = valid
        .typed_ast
        .source_structure()
        .expect("Task 254")
        .roots()
        .iter()
        .next()
        .map(|(_, root)| root)
        .expect("imported Task 254 root row");
    assert_eq!(
        imported_root.symbol().module().path().as_str(),
        "task254.imported"
    );
    assert_eq!(
        valid.typed_ast.source_structure(),
        valid.resolved.source_structure()
    );

    for corruption in [
        Task254ImportedRootCorruption::ContributionKind,
        Task254ImportedRootCorruption::ContributionSource,
        Task254ImportedRootCorruption::ContributionModule,
        Task254ImportedRootCorruption::ContributionRange,
        Task254ImportedRootCorruption::SymbolEffect,
        Task254ImportedRootCorruption::Visibility,
        Task254ImportedRootCorruption::ExportStatus,
        Task254ImportedRootCorruption::Namespace,
        Task254ImportedRootCorruption::AuthenticatedImportModule,
        Task254ImportedRootCorruption::AuthenticatedImportEffect,
    ] {
        let validation_symbols = task254_imported_structure_symbols(&ast, &module, corruption);
        let error =
            super::type_elaboration::synthetic_source_structure_output_with_validation_symbols(
                &ast,
                module.clone(),
                task254_module_binding_env(&ast, module.clone()),
                &valid_symbols,
                &validation_symbols,
                &[root],
                None::<SyntheticSourceStructureDependencies>,
                &BTreeSet::new(),
            )
            .expect_err("corrupt imported Task 254 provenance must fail in the producer");
        assert!(
            error.contains("structure") && error.contains("root"),
            "{corruption:?}: {error}"
        );
    }
}

#[test]
fn task254_synthetic_whole_subtree_exclusion_matrix_is_exact() {
    let (real_ast, module, _, symbols) = task254_real_ast();
    let mut syntax = Task254SyntheticSyntax::new(real_ast.source_id);
    let type_arguments = syntax.constructor(Vec::new(), true);
    let set_enumeration_value = syntax.opaque_term(SurfaceNodeKind::SetEnumeration, "{ 1 }");
    let set_enumeration = syntax.constructor(vec![("carrier", set_enumeration_value)], false);
    let comprehension_value = syntax.opaque_term(SurfaceNodeKind::SetComprehension, "{ x }");
    let comprehension = syntax.constructor(vec![("carrier", comprehension_value)], false);
    let choice_value = syntax.opaque_term(SurfaceNodeKind::ChoiceTerm, "the set");
    let choice = syntax.constructor(vec![("carrier", choice_value)], false);
    let qua_value = syntax.opaque_term(SurfaceNodeKind::QuaExpression, "x qua set");
    let qua = syntax.constructor(vec![("carrier", qua_value)], false);
    let template_value = syntax.opaque_term(SurfaceNodeKind::TemplateArgument, "template");
    let template_descendant = syntax.constructor(vec![("carrier", template_value)], false);
    let template_child = syntax.constructor(Vec::new(), false);
    let template_child_range = syntax.range(template_child);
    let template_ancestor = syntax.template_ancestor(template_child);
    let application_child = syntax.constructor(Vec::new(), false);
    let application_child_range = syntax.range(application_child);
    let reverse_application = syntax.application_ancestor(application_child);
    let root_specs = [
        (
            syntax.range(type_arguments),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(set_enumeration),
            SurfaceNodeKind::StructureConstructor,
        ),
        (
            syntax.range(comprehension),
            SurfaceNodeKind::StructureConstructor,
        ),
        (syntax.range(choice), SurfaceNodeKind::StructureConstructor),
        (syntax.range(qua), SurfaceNodeKind::StructureConstructor),
        (
            syntax.range(template_descendant),
            SurfaceNodeKind::StructureConstructor,
        ),
        (template_child_range, SurfaceNodeKind::StructureConstructor),
        (
            application_child_range,
            SurfaceNodeKind::StructureConstructor,
        ),
    ];
    let reverse_application_range = syntax.range(reverse_application);
    let ast = syntax.finish(vec![
        type_arguments,
        set_enumeration,
        comprehension,
        choice,
        qua,
        template_descendant,
        template_ancestor,
        reverse_application,
    ]);
    let roots =
        root_specs.map(|(range, kind)| task254_node_with_range_and_kind(&ast, range, &kind));
    let application_site = task254_node_with_range_and_kind(
        &ast,
        reverse_application_range,
        &SurfaceNodeKind::ApplicationTerm,
    );
    let dependencies =
        task254_bare_application_dependencies(&ast, &module, &symbols, application_site, None);
    let output = synthetic_source_structure_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &symbols,
        &roots,
        Some(dependencies),
        &BTreeSet::new(),
    )
    .expect("Task 254 excluded subtree matrix");
    let handoff = output.typed_ast.source_structure().expect("Task 254");
    assert!(handoff.terms().is_empty());
    assert!(handoff.members().is_empty());
    assert!(handoff.field_updates().is_empty());
    assert!(handoff.edges().is_empty());
    assert_eq!(handoff.application_fingerprint(), None);
    assert!(output.typed_ast.source_application().is_some());
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
    );
}

fn assert_task254_real_oracle(output: &SourceStructureRouteOutput) {
    let handoff = output
        .typed_ast
        .source_structure()
        .expect("Task 254 handoff");
    let primary = output.typed_ast.source_term().expect("Task 252 handoff");
    assert_eq!(
        (
            handoff.terms().len(),
            handoff.wrappers().len(),
            handoff.roots().len(),
            handoff.members().len(),
            handoff.field_updates().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (5, 0, 3, 9, 2, 10, 26)
    );
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (8, 0, 8)
    );
    assert_eq!(handoff.application_fingerprint(), None);
    assert_eq!(
        handoff.primary_term_fingerprint(),
        primary.debug_text().as_str()
    );
    assert_eq!(
        handoff
            .terms()
            .iter()
            .map(|(id, term)| (id.index(), term.source_ordinal(), term.kind()))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
            (
                1,
                1,
                mizar_checker::source_structure::SourceStructureTermKind::SelectorAccess,
            ),
            (
                2,
                2,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
            (
                3,
                3,
                mizar_checker::source_structure::SourceStructureTermKind::FunctionalUpdate,
            ),
            (
                4,
                4,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
        ]
    );
    assert_eq!(
        handoff
            .members()
            .iter()
            .map(|(_, member)| (member.spelling(), member.role()))
            .collect::<Vec<_>>(),
        [
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::Selector,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment,
            ),
            (
                "carrier",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                "marker",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
        ]
    );
    for (_, member) in handoff.members().iter() {
        let node = output
            .typed_ast
            .nodes()
            .node(member.site().node())
            .expect("member arena node");
        let expected = match member.role() {
            mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment => {
                "source.term.structure.member.constructor-assignment"
            }
            mizar_checker::source_structure::SourceStructureMemberRole::Selector => {
                "source.term.structure.member.selector"
            }
            mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment => {
                "source.term.structure.member.update-path-segment"
            }
            role => panic!("unexpected member role: {role:?}"),
        };
        assert_eq!(node.kind.as_str(), expected);
    }
    for (_, update) in handoff.field_updates().iter() {
        assert_eq!(
            output
                .typed_ast
                .nodes()
                .node(update.site().node())
                .expect("FieldUpdate arena node")
                .kind
                .as_str(),
            "source.term.structure.field-update"
        );
    }
    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Primary(_)
            ))
            .count(),
        8
    );
    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_structure::SourceStructureTarget::Structure(_)
            ))
            .count(),
        2
    );
    assert_eq!(output.typed_ast.source_application(), None);
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
}

fn assert_task254_real_binding_contexts(output: &SourceStructureRouteOutput) {
    let handoff = output
        .typed_ast
        .source_structure()
        .expect("Task 254 handoff");
    let primary = output.typed_ast.source_term().expect("Task 252 handoff");
    assert_eq!(output.binding_env.contexts().len(), 2);
    assert_eq!(output.binding_env.bindings().len(), 2);
    assert!(matches!(
        output
            .binding_env
            .contexts()
            .get(mizar_checker::binding_env::BindingContextId::new(0))
            .expect("Task 254 module context")
            .owner,
        mizar_checker::binding_env::BindingContextOwner::Module
    ));
    assert!(matches!(
        output
            .binding_env
            .contexts()
            .get(mizar_checker::binding_env::BindingContextId::new(1))
            .expect("Task 254 definition context")
            .owner,
        mizar_checker::binding_env::BindingContextOwner::DeclarationShell(_)
    ));
    assert!(handoff.terms().iter().all(|(_, term)| term.context()
        == mizar_checker::binding_env::BindingContextId::new(1)));
    assert!(primary.terms().iter().all(|(_, term)| term.context()
        == mizar_checker::binding_env::BindingContextId::new(1)));
}

fn task254_real_ast() -> (
    SurfaceAst,
    ResolverModuleId,
    mizar_resolve::declarations::DeclarationShellSet,
    SymbolEnv,
) {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 254 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == "fail_type_elaboration_local_structure_term_gap_001")
        .expect("Task 254 case should remain active");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .unwrap_or_else(|error| panic!("Task 254 frontend failed: {error}"));
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.expect("Task 254 AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let module = resolver.module;
    let shells = resolver.shells;
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
    (ast, module, shells, symbols)
}

fn task254_outer_structure_roots(ast: &SurfaceAst) -> Vec<usize> {
    let mut parents = vec![None; ast.nodes().len()];
    for (parent, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parents[child.index()] = Some(parent);
        }
    }
    ast.nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            matches!(
                node.kind,
                SurfaceNodeKind::StructureConstructor
                    | SurfaceNodeKind::SelectorAccess
                    | SurfaceNodeKind::StructureUpdate
            )
        })
        .filter(|(index, _)| {
            let mut cursor = parents[*index];
            while let Some(parent) = cursor {
                if matches!(
                    ast.nodes()[parent].kind,
                    SurfaceNodeKind::StructureConstructor
                        | SurfaceNodeKind::SelectorAccess
                        | SurfaceNodeKind::StructureUpdate
                ) {
                    return false;
                }
                cursor = parents[parent];
            }
            true
        })
        .map(|(index, _)| index)
        .collect()
}

fn task254_module_binding_env(
    ast: &SurfaceAst,
    module: ResolverModuleId,
) -> mizar_checker::binding_env::BindingEnv {
    let mut contexts = mizar_checker::binding_env::BindingContextTable::new();
    let context = contexts.insert(mizar_checker::binding_env::BindingContextDraft {
        owner: mizar_checker::binding_env::BindingContextOwner::Module,
        parent: None,
        layer: mizar_checker::binding_env::BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: mizar_checker::binding_env::BindingContextRecovery::Normal,
    });
    assert_eq!(
        context,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    mizar_checker::binding_env::BindingEnv::try_new(mizar_checker::binding_env::BindingEnvParts {
        source_id: ast.source_id,
        module_id: module,
        contexts,
        bindings: mizar_checker::binding_env::BindingTable::new(),
        diagnostics: mizar_checker::binding_env::BindingDiagnosticTable::new(),
    })
    .expect("synthetic Task 254 binding env")
}

fn task254_term_expression_with_spelling(ast: &SurfaceAst, spelling: &str) -> usize {
    ast.nodes()
        .iter()
        .enumerate()
        .find(|(_, node)| {
            matches!(node.kind, SurfaceNodeKind::TermExpression)
                && task254_subtree_tokens(ast, node) == [spelling]
        })
        .map(|(index, _)| index)
        .unwrap_or_else(|| panic!("Task 254 term expression `{spelling}`"))
}

fn task254_bare_application_dependencies(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    application_node: usize,
    excluded_primary_spelling: Option<&str>,
) -> SyntheticSourceStructureDependencies {
    use mizar_checker::{
        source_application::{
            SourceFunctorApplicationForm, SourceFunctorApplicationHandoffInput,
            SourceFunctorApplicationId, SourceFunctorApplicationInput,
            SourceFunctorApplicationKind, SourceFunctorApplicationProducer,
            SourceFunctorApplicationRecovery, SourceFunctorCandidateId,
            SourceFunctorCandidateInput, SourceFunctorHeadSite, SourceFunctorTypeRequestInput,
            SourceFunctorTypeRequestKind,
        },
        source_term::{
            SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermId,
            SourcePrimaryTermInput, SourcePrimaryTermKind, SourcePrimaryTermProducer,
            SourcePrimaryTermRecovery, SourcePrimaryTermRole,
        },
        typed_ast::{
            NodeRecoveryState, TypedArena, TypedNode, TypedNodeId, TypedSiteRef, TypingState,
        },
    };
    use mizar_session::SourceAnchor;

    let application = &ast.nodes()[application_node];
    let application_children = application
        .children
        .iter()
        .filter_map(|child| {
            ast.node(*child)
                .filter(|node| !matches!(node.kind, SurfaceNodeKind::Token(_)))
                .map(|_| child.index())
        })
        .collect::<Vec<_>>();
    let [head_node] = application_children.as_slice() else {
        panic!("bare Task 253 site must have one structural head");
    };
    assert_eq!(ast.nodes()[*head_node].range, application.range);
    let application_spelling = task254_subtree_tokens(ast, application).join(" ");

    let mut numeral_nodes = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::NumeralTerm))
        .filter(|(_, node)| {
            excluded_primary_spelling
                .is_none_or(|excluded| task254_subtree_tokens(ast, node).as_slice() != [excluded])
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    numeral_nodes.sort_by_key(|index| ast.nodes()[*index].range.start);

    let mut typed_nodes = ast
        .nodes()
        .iter()
        .map(|node| {
            TypedNode::new("source.surface.unowned", SourceAnchor::Range(node.range))
                .with_children(
                    node.children
                        .iter()
                        .map(|child| TypedNodeId::new(child.index()))
                        .collect(),
                )
                .with_typing(TypingState::Unknown)
                .with_recovery(if node.recovered {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                })
        })
        .collect::<Vec<_>>();
    for numeral in &numeral_nodes {
        typed_nodes[*numeral].kind = "source.term.numeral".into();
    }
    typed_nodes[application_node].kind = "source.term.functor-application.symbolic".into();
    typed_nodes[*head_node].kind = "source.term.functor-head.single".into();
    let arena = TypedArena::try_new(
        ast.root().map(|root| TypedNodeId::new(root.index())),
        typed_nodes,
    )
    .expect("Task 254 synthetic dependency arena");
    let binding_env = task254_module_binding_env(ast, module.clone());
    let primary_input = SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: numeral_nodes
            .iter()
            .enumerate()
            .map(|(source_ordinal, node)| {
                let source = &ast.nodes()[*node];
                SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(*node)),
                    source_range: source.range,
                    source_ordinal,
                    context: mizar_checker::binding_env::BindingContextId::new(0),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: task254_subtree_tokens(ast, source).join(" "),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }
            })
            .collect(),
        references: Vec::new(),
        numeric_type_requests: numeral_nodes
            .iter()
            .enumerate()
            .map(|(request_ordinal, node)| {
                let source = &ast.nodes()[*node];
                SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(request_ordinal),
                    owner: TypedSiteRef::Node(TypedNodeId::new(*node)),
                    source_range: source.range,
                    spelling: task254_subtree_tokens(ast, source).join(" "),
                    request_ordinal,
                }
            })
            .collect(),
    };
    let primary = SourcePrimaryTermProducer::build(primary_input, &binding_env, &arena)
        .expect("Task 254 synthetic Task 252 dependency");
    let candidate = symbols
        .symbols()
        .iter()
        .filter(|entry| entry.kind() == SymbolKind::Functor)
        .filter_map(|entry| {
            let SourceAnchor::Range(range) = entry.origin().anchor() else {
                return None;
            };
            (range.end <= application.range.start).then_some((range.start, entry))
        })
        .min_by_key(|(start, _)| *start)
        .map(|(_, entry)| entry)
        .expect("source-preceding synthetic Task 253 candidate");
    let application_id = SourceFunctorApplicationId::new(0);
    let candidate_id = SourceFunctorCandidateId::new(0);
    let application_input = SourceFunctorApplicationHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        applications: vec![SourceFunctorApplicationInput {
            site: TypedSiteRef::Node(TypedNodeId::new(application_node)),
            source_range: application.range,
            source_ordinal: 0,
            context: mizar_checker::binding_env::BindingContextId::new(0),
            recovery: SourceFunctorApplicationRecovery::Normal,
            spelling: application_spelling.clone(),
            kind: SourceFunctorApplicationKind::Symbolic,
            form: SourceFunctorApplicationForm::Bare,
            head_ordinal: 0,
            head: SourceFunctorHeadSite::Single {
                site: TypedSiteRef::Node(TypedNodeId::new(*head_node)),
                source_range: ast.nodes()[*head_node].range,
                spelling: application_spelling,
            },
        }],
        wrappers: Vec::new(),
        candidates: vec![SourceFunctorCandidateInput {
            application: application_id,
            ordinal: 0,
            symbol: candidate.symbol().clone(),
            contribution: candidate.contribution(),
        }],
        arguments: Vec::new(),
        type_requests: vec![
            SourceFunctorTypeRequestInput {
                application: application_id,
                candidate: Some(candidate_id),
                request_ordinal: 0,
                kind: SourceFunctorTypeRequestKind::CandidateSignature,
            },
            SourceFunctorTypeRequestInput {
                application: application_id,
                candidate: None,
                request_ordinal: 1,
                kind: SourceFunctorTypeRequestKind::ApplicationResultType,
            },
        ],
    };
    let application = SourceFunctorApplicationProducer::build(
        application_input,
        symbols,
        &binding_env,
        &primary,
        &arena,
    )
    .expect("Task 254 synthetic Task 253 dependency");
    SyntheticSourceStructureDependencies {
        arena,
        primary,
        application: Some(application),
    }
}

fn task254_subtree_tokens<'a>(
    ast: &'a SurfaceAst,
    node: &'a mizar_syntax::SurfaceNode,
) -> Vec<&'a str> {
    let mut tokens = Vec::new();
    fn collect<'a>(
        ast: &'a SurfaceAst,
        node: &'a mizar_syntax::SurfaceNode,
        tokens: &mut Vec<&'a str>,
    ) {
        if let Some(token) = node.token_text() {
            tokens.push(token);
            return;
        }
        for child in &node.children {
            if let Some(child) = ast.node(*child) {
                collect(ast, child, tokens);
            }
        }
    }
    collect(ast, node, &mut tokens);
    tokens
}

fn task254_node_with_range_and_kind(
    ast: &SurfaceAst,
    range: SourceRange,
    kind: &SurfaceNodeKind,
) -> usize {
    let matches = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| node.range == range && &node.kind == kind)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [index] = matches.as_slice() else {
        panic!("expected one synthetic {kind:?} at {range:?}");
    };
    *index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task254ImportedRootCorruption {
    None,
    ContributionKind,
    ContributionSource,
    ContributionModule,
    ContributionRange,
    SymbolEffect,
    Visibility,
    ExportStatus,
    Namespace,
    AuthenticatedImportModule,
    AuthenticatedImportEffect,
}

fn task254_imported_structure_symbols(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    corruption: Task254ImportedRootCorruption,
) -> SymbolEnv {
    let dependency = ResolverModuleId::new(
        module.package().clone(),
        ModulePath::new("task254.imported"),
    );
    let constructor_range = ast
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
        .expect("Task 254 imported constructor")
        .range;
    let valid_provenance_range = SourceRange {
        source_id: ast.source_id,
        start: 1,
        end: 5,
    };
    let contribution_range = if corruption == Task254ImportedRootCorruption::ContributionRange {
        SourceRange {
            source_id: ast.source_id,
            start: constructor_range.start + 1,
            end: constructor_range.end + 1,
        }
    } else {
        valid_provenance_range
    };
    let contribution_module = if corruption == Task254ImportedRootCorruption::ContributionModule {
        module.clone()
    } else {
        dependency.clone()
    };
    let contribution_source = if corruption == Task254ImportedRootCorruption::ContributionSource {
        let allocator = InMemorySessionIdAllocator::new();
        allocator
            .next_source_id(snapshot_id(254))
            .expect("Task 254 first drift source allocation");
        allocator
            .next_source_id(snapshot_id(253))
            .expect("Task 254 distinct drift source allocation")
    } else {
        ast.source_id
    };
    if corruption == Task254ImportedRootCorruption::ContributionSource {
        assert_ne!(
            contribution_source, ast.source_id,
            "Task 254 drift source must differ"
        );
    }
    let contribution_kind = if corruption == Task254ImportedRootCorruption::ContributionKind {
        ContributionKind::LocalSource {
            source_id: ast.source_id,
        }
    } else {
        ContributionKind::ImportedSource {
            source_id: contribution_source,
        }
    };

    let import_origin = SemanticOrigin::new(
        ast.source_id,
        module.clone(),
        SourceAnchor::Range(valid_provenance_range),
        vec![0],
    );
    let mut import_nodes = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let import_owner = import_nodes
        .push(mizar_resolve::resolved_ast::ResolvedNode::new(
            SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            import_origin.clone(),
        ))
        .expect("Task 254 imported owner");
    let mut resolved_imports = mizar_resolve::resolved_ast::ResolvedImports::new();
    let import = resolved_imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
        import_owner,
        valid_provenance_range,
        "import task254.imported;",
        None,
        mizar_resolve::resolved_ast::ImportResolution::Resolved(dependency.clone()),
        import_origin,
    ));

    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        contribution_module,
        contribution_kind,
        SourceAnchor::Range(contribution_range),
    );
    let symbol = ResolverSymbolId::new(
        dependency.clone(),
        LocalSymbolId::new("Structure/ImportedPair/0"),
        FullyQualifiedName::new("task254.imported::ImportedPair/0"),
    );
    let namespace = if corruption == Task254ImportedRootCorruption::Namespace {
        NamespacePath::new("task254.drift")
    } else {
        NamespacePath::new(module.path().as_str())
    };
    let visibility = if corruption == Task254ImportedRootCorruption::Visibility {
        Visibility::Private
    } else {
        Visibility::Public
    };
    let export_status = if corruption == Task254ImportedRootCorruption::ExportStatus {
        ExportStatus::LocalOnly
    } else {
        ExportStatus::Exported
    };
    indexes.symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Structure,
            namespace,
            "ImportedPair",
            SemanticOrigin::new(
                ast.source_id,
                dependency.clone(),
                SourceAnchor::Range(valid_provenance_range),
                vec![0],
            ),
            contribution,
        )
        .with_visibility(visibility)
        .with_export_status(export_status),
    );
    if corruption != Task254ImportedRootCorruption::SymbolEffect {
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
    }
    let authenticated_module =
        if corruption == Task254ImportedRootCorruption::AuthenticatedImportModule {
            module.clone()
        } else {
            dependency
        };
    indexes
        .imports
        .insert(mizar_resolve::env::ImportIndexEntry::new(
            import,
            Some(authenticated_module),
            None,
            contribution,
        ));
    if corruption != Task254ImportedRootCorruption::AuthenticatedImportEffect {
        indexes.contributions.add_import(contribution, import);
    }
    SymbolEnv::new(module.clone(), indexes)
}

struct Task254SyntheticSyntax {
    source: SourceId,
    next: usize,
    builder: SurfaceAstBuilder,
}

impl Task254SyntheticSyntax {
    fn new(source: SourceId) -> Self {
        Self {
            source,
            next: 1_000,
            builder: SurfaceAstBuilder::new(source),
        }
    }

    fn token(&mut self, kind: SurfaceTokenKind, spelling: &str) -> SurfaceBuilderNodeId {
        let start = self.next;
        let end = start + spelling.len().max(1);
        self.next = end + 128;
        self.builder.add_token(
            kind,
            spelling,
            SourceRange {
                source_id: self.source,
                start,
                end,
            },
        )
    }

    fn token_at(
        &mut self,
        kind: SurfaceTokenKind,
        spelling: &str,
        cursor: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let start = *cursor;
        let end = start + spelling.len().max(1);
        *cursor = end + 1;
        self.next = self.next.max(end + 128);
        self.builder.add_token(
            kind,
            spelling,
            SourceRange {
                source_id: self.source,
                start,
                end,
            },
        )
    }

    fn node(
        &mut self,
        kind: SurfaceNodeKind,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let start = children
            .iter()
            .filter_map(|child| self.builder.node_range(*child))
            .map(|range| range.start)
            .min()
            .expect("synthetic children");
        let end = children
            .iter()
            .filter_map(|child| self.builder.node_range(*child))
            .map(|range| range.end)
            .max()
            .expect("synthetic children");
        self.builder.add_node(
            kind,
            SourceRange {
                source_id: self.source,
                start,
                end,
            },
            children,
        )
    }

    fn primary(&mut self, spelling: &str) -> SurfaceBuilderNodeId {
        let token = self.token(SurfaceTokenKind::Numeral, spelling);
        let numeral = self.node(SurfaceNodeKind::NumeralTerm, vec![token]);
        self.node(SurfaceNodeKind::TermExpression, vec![numeral])
    }

    fn opaque_term(&mut self, kind: SurfaceNodeKind, spelling: &str) -> SurfaceBuilderNodeId {
        let token = self.token(SurfaceTokenKind::Identifier, spelling);
        let opaque = self.node(kind, vec![token]);
        self.node(SurfaceNodeKind::TermExpression, vec![opaque])
    }

    fn range(&self, node: SurfaceBuilderNodeId) -> SourceRange {
        self.builder.node_range(node).expect("synthetic node range")
    }

    fn gap(&mut self, width: usize) {
        self.next += width;
    }

    fn qualified_structure_named(&mut self, spelling: &str) -> SurfaceBuilderNodeId {
        let token = self.token(SurfaceTokenKind::Identifier, spelling);
        let segment = self.node(SurfaceNodeKind::PathSegment, vec![token]);
        self.node(SurfaceNodeKind::QualifiedSymbol, vec![segment])
    }

    fn constructor(
        &mut self,
        fields: Vec<(&str, SurfaceBuilderNodeId)>,
        type_arguments: bool,
    ) -> SurfaceBuilderNodeId {
        self.constructor_named("Task254Pair", fields, type_arguments)
    }

    fn constructor_named(
        &mut self,
        spelling: &str,
        fields: Vec<(&str, SurfaceBuilderNodeId)>,
        type_arguments: bool,
    ) -> SurfaceBuilderNodeId {
        if fields.is_empty() {
            let root = self.qualified_structure_named(spelling);
            let mut children = vec![root];
            if type_arguments {
                let of = self.token(SurfaceTokenKind::ReservedWord, "of");
                let set = self.token(SurfaceTokenKind::Identifier, "set");
                children.push(self.node(SurfaceNodeKind::TypeArguments, vec![of, set]));
            }
            children.push(self.token(SurfaceTokenKind::ReservedSymbol, "("));
            children.push(self.token(SurfaceTokenKind::ReservedSymbol, ")"));
            return self.node(SurfaceNodeKind::StructureConstructor, children);
        }

        let first_value_start = self.range(fields[0].1).start;
        let first_label = fields[0].0;
        let prefix_width = spelling.len().max(1)
            + usize::from(type_arguments) * ("of".len() + "set".len() + 2)
            + "(".len()
            + first_label.len().max(1)
            + ":".len()
            + 6;
        let mut cursor = first_value_start
            .checked_sub(prefix_width)
            .expect("synthetic constructor prefix range");
        let root_token = self.token_at(SurfaceTokenKind::Identifier, spelling, &mut cursor);
        let root_segment = self.node(SurfaceNodeKind::PathSegment, vec![root_token]);
        let root = self.node(SurfaceNodeKind::QualifiedSymbol, vec![root_segment]);
        let mut children = vec![root];
        if type_arguments {
            let of = self.token_at(SurfaceTokenKind::ReservedWord, "of", &mut cursor);
            let set = self.token_at(SurfaceTokenKind::Identifier, "set", &mut cursor);
            children.push(self.node(SurfaceNodeKind::TypeArguments, vec![of, set]));
        }
        children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, "(", &mut cursor));
        for (ordinal, (label, value)) in fields.into_iter().enumerate() {
            let value_range = self.range(value);
            if ordinal > 0 {
                children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ",", &mut cursor));
            }
            let label = self.token_at(SurfaceTokenKind::Identifier, label, &mut cursor);
            let colon = self.token_at(SurfaceTokenKind::ReservedSymbol, ":", &mut cursor);
            assert!(
                cursor <= value_range.start,
                "synthetic constructor label must precede its value"
            );
            let field = self.node(SurfaceNodeKind::FieldArgument, vec![label, colon, value]);
            children.push(field);
            cursor = value_range.end + 1;
        }
        children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ")", &mut cursor));
        self.node(SurfaceNodeKind::StructureConstructor, children)
    }

    fn selector(
        &mut self,
        base: SurfaceBuilderNodeId,
        member: &str,
        arguments: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let base_range = self.range(base);
        let mut cursor = base_range.end + 1;
        let dot = self.token_at(SurfaceTokenKind::ReservedSymbol, ".", &mut cursor);
        let member = self.token_at(SurfaceTokenKind::Identifier, member, &mut cursor);
        let mut children = vec![base, dot, member];
        if !arguments.is_empty() {
            children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, "(", &mut cursor));
            let argument_count = arguments.len();
            for (ordinal, argument) in arguments.into_iter().enumerate() {
                let argument_range = self.range(argument);
                assert!(
                    cursor <= argument_range.start,
                    "synthetic selector member must precede its arguments"
                );
                children.push(argument);
                if ordinal + 1 < argument_count {
                    cursor = argument_range.end + 1;
                    children.push(self.token_at(
                        SurfaceTokenKind::ReservedSymbol,
                        ",",
                        &mut cursor,
                    ));
                } else {
                    cursor = argument_range.end + 1;
                }
            }
            children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ")", &mut cursor));
        }
        self.node(SurfaceNodeKind::SelectorAccess, children)
    }

    fn selector_zero_call(
        &mut self,
        base: SurfaceBuilderNodeId,
        member: &str,
    ) -> SurfaceBuilderNodeId {
        let dot = self.token(SurfaceTokenKind::ReservedSymbol, ".");
        let member = self.token(SurfaceTokenKind::Identifier, member);
        let open = self.token(SurfaceTokenKind::ReservedSymbol, "(");
        let close = self.token(SurfaceTokenKind::ReservedSymbol, ")");
        self.node(
            SurfaceNodeKind::SelectorAccess,
            vec![base, dot, member, open, close],
        )
    }

    fn update(
        &mut self,
        base: SurfaceBuilderNodeId,
        updates: Vec<(Vec<&str>, SurfaceBuilderNodeId)>,
    ) -> SurfaceBuilderNodeId {
        let mut cursor = self.range(base).end + 1;
        let with = self.token_at(SurfaceTokenKind::ReservedWord, "with", &mut cursor);
        let open = self.token_at(SurfaceTokenKind::ReservedSymbol, "(", &mut cursor);
        let mut children = vec![base, with, open];
        for (ordinal, (path, value)) in updates.into_iter().enumerate() {
            if ordinal > 0 {
                children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ",", &mut cursor));
            }
            let mut field_children = Vec::new();
            let path_count = path.len();
            for (segment_ordinal, segment) in path.into_iter().enumerate() {
                field_children.push(self.token_at(
                    SurfaceTokenKind::Identifier,
                    segment,
                    &mut cursor,
                ));
                if segment_ordinal + 1 < path_count {
                    field_children.push(self.token_at(
                        SurfaceTokenKind::ReservedSymbol,
                        ".",
                        &mut cursor,
                    ));
                }
            }
            let assign = self.token_at(SurfaceTokenKind::ReservedSymbol, ":=", &mut cursor);
            field_children.push(assign);
            let value_range = self.range(value);
            assert!(
                cursor <= value_range.start,
                "synthetic update path must precede its replacement"
            );
            field_children.push(value);
            let start = self
                .builder
                .node_range(field_children[0])
                .expect("synthetic update path start")
                .start;
            let end = self
                .builder
                .node_range(value)
                .expect("synthetic update value")
                .end;
            children.push(self.builder.add_node(
                SurfaceNodeKind::FieldUpdate,
                SourceRange {
                    source_id: self.source,
                    start,
                    end,
                },
                field_children,
            ));
            cursor = value_range.end + 1;
        }
        children.push(self.token_at(SurfaceTokenKind::ReservedSymbol, ")", &mut cursor));
        self.node(SurfaceNodeKind::StructureUpdate, children)
    }

    fn wrapper(&mut self, child: SurfaceBuilderNodeId) -> SurfaceBuilderNodeId {
        let child_range = self
            .builder
            .node_range(child)
            .expect("synthetic wrapper child");
        let mut open_cursor = child_range.start.saturating_sub(2);
        let open = self.token_at(
            SurfaceTokenKind::ReservedSymbol,
            "(",
            &mut open_cursor,
        );
        let mut close_cursor = child_range.end + 1;
        let close = self.token_at(
            SurfaceTokenKind::ReservedSymbol,
            ")",
            &mut close_cursor,
        );
        self.builder.add_node(
            SurfaceNodeKind::ParenthesizedTerm,
            SourceRange {
                source_id: self.source,
                start: self
                    .builder
                    .node_range(open)
                    .expect("synthetic wrapper open")
                    .start,
                end: self
                    .builder
                    .node_range(close)
                    .expect("synthetic wrapper close")
                    .end,
            },
            vec![open, child, close],
        )
    }

    fn template_ancestor(&mut self, child: SurfaceBuilderNodeId) -> SurfaceBuilderNodeId {
        let range = self.range(child);
        self.builder
            .add_node(SurfaceNodeKind::TemplateArgument, range, vec![child])
    }

    fn application_ancestor(&mut self, child: SurfaceBuilderNodeId) -> SurfaceBuilderNodeId {
        let range = self.range(child);
        self.builder
            .add_node(SurfaceNodeKind::ApplicationTerm, range, vec![child])
    }

    fn finish(mut self, roots: Vec<SurfaceBuilderNodeId>) -> SurfaceAst {
        let end = roots
            .iter()
            .filter_map(|root| self.builder.node_range(*root))
            .map(|range| range.end)
            .max()
            .unwrap_or(self.next);
        let root = self.builder.add_node(
            SurfaceNodeKind::Root,
            SourceRange {
                source_id: self.source,
                start: 0,
                end: end + 1,
            },
            roots,
        );
        self.builder.finish(Some(root), None)
    }
}
