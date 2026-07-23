#[test]
fn active_source_binding_context_fixture_preserves_the_final_checker_handoff() {
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
    let plan = build_test_plan(&config).expect("Task 248 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_source_binding_context_shadowing_001"
        })
        .expect("Task 248 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 248 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.expect("Task 248 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let shells = resolver.shells.clone();
    let [reserve_shell, definition_shell] = shells.declarations() else {
        panic!("Task 248 fixture should have exactly two resolver shells");
    };
    let reserve_node = ast.node(reserve_shell.node_id()).expect("reserve source item");
    let definition_node = ast
        .node(definition_shell.node_id())
        .expect("definition source item");
    let parameter_ids = structural_child_ids(&ast, definition_node);
    let [parameter_id] = parameter_ids.as_slice() else {
        panic!("definition should have exactly one parameter");
    };
    let parameter_node = ast.node(*parameter_id).expect("definition parameter");
    let reserve_name_range = sole_token_range(&ast, reserve_node, "x");
    let reserve_type_range = sole_token_range(&ast, reserve_node, "set");
    let parameter_name_range = sole_token_range(&ast, parameter_node, "x");
    let parameter_type_range = sole_token_range(&ast, parameter_node, "set");
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let first = source_binding_context_output(
        &ast,
        resolver.module.clone(),
        &shells,
        &symbols,
    )
    .expect("Task 248 source shape should select the bounded route")
    .expect("Task 248 source payload should be valid");
    let second = source_binding_context_output(
        &ast,
        resolver.module,
        &shells,
        &symbols,
    )
    .expect("Task 248 source shape should remain selected")
    .expect("Task 248 repeated source payload should remain valid");

    let handoff = first
        .typed_ast
        .source_context()
        .expect("TypedAst should own the final Task 248 handoff");
    assert_eq!(handoff.items().len(), 2);
    assert_eq!(handoff.declarations().len(), 2);
    assert_eq!(handoff.binding_env().bindings().len(), 2);
    assert_eq!(handoff.binding_env().contexts().len(), 2);
    assert_eq!(handoff.local_contexts(), first.typed_ast.contexts());
    let reserve_item = handoff
        .items()
        .get(mizar_checker::source_context::SourceItemId::new(0))
        .expect("reserve item row");
    let definition_item = handoff
        .items()
        .get(mizar_checker::source_context::SourceItemId::new(1))
        .expect("definition item row");
    assert_eq!(reserve_item.shell, reserve_shell.id());
    assert_eq!(reserve_item.shell_ordinal, 0);
    assert_eq!(
        reserve_item.role,
        mizar_checker::source_context::SourceItemRole::Reserve
    );
    assert_eq!(reserve_item.source_range, reserve_node.range);
    assert_eq!(reserve_item.parent, None);
    assert_eq!(
        reserve_item.visibility,
        mizar_checker::source_context::SourceItemVisibility::Unspecified
    );
    assert_eq!(reserve_item.local_scope, None);
    assert_eq!(reserve_item.predecessor, None);
    assert_eq!(definition_item.shell, definition_shell.id());
    assert_eq!(definition_item.shell_ordinal, 1);
    assert_eq!(
        definition_item.role,
        mizar_checker::source_context::SourceItemRole::DefinitionBlock
    );
    assert_eq!(definition_item.source_range, definition_node.range);
    assert_eq!(definition_item.parent, None);
    assert_eq!(
        definition_item.visibility,
        mizar_checker::source_context::SourceItemVisibility::Unspecified
    );
    assert_eq!(
        definition_item.predecessor,
        Some(mizar_checker::source_context::SourceItemId::new(0))
    );
    let reserve = handoff
        .declarations()
        .get(mizar_checker::source_context::SourceDeclarationId::new(0))
        .expect("reserve declaration row");
    let parameter = handoff
        .declarations()
        .get(mizar_checker::source_context::SourceDeclarationId::new(1))
        .expect("definition parameter declaration row");
    assert_ne!(reserve.binding, parameter.binding);
    assert_eq!(reserve.item, reserve_item.id);
    assert_eq!(reserve.source_ordinal, 0);
    assert_eq!(reserve.spelling, "x");
    assert_eq!(reserve.declaration_range, reserve_name_range);
    assert_eq!(reserve.written_type_range, reserve_type_range);
    assert_eq!(
        reserve.role,
        mizar_checker::source_context::SourceBindingSiteRole::ReserveDefault
    );
    assert_eq!(reserve.binding_context, reserve_item.binding_context);
    assert_eq!(reserve.local_context, reserve_item.local_context);
    assert_eq!(reserve.shadowed_binding, None);
    assert_eq!(reserve.predecessor, None);
    assert_eq!(parameter.item, definition_item.id);
    assert_eq!(parameter.source_ordinal, 1);
    assert_eq!(parameter.spelling, "x");
    assert_eq!(parameter.declaration_range, parameter_name_range);
    assert_eq!(parameter.written_type_range, parameter_type_range);
    let mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter { local } =
        &parameter.role
    else {
        panic!("parameter declaration should retain resolver-shaped local provenance");
    };
    assert_eq!(local.spelling(), "x");
    assert_eq!(local.declaration_range(), parameter_name_range);
    assert_eq!(local.visible_after_ordinal(), 1);
    assert_eq!(definition_item.local_scope.as_ref(), Some(local.scope()));
    assert_eq!(parameter.binding_context, definition_item.binding_context);
    assert_eq!(parameter.local_context, definition_item.local_context);
    assert_eq!(parameter.shadowed_binding, Some(reserve.binding));
    assert_eq!(
        parameter.predecessor,
        Some(mizar_checker::source_context::SourceDeclarationId::new(0))
    );

    let reserve_binding = handoff
        .binding_env()
        .bindings()
        .get(reserve.binding)
        .expect("reserve binding row");
    let parameter_binding = handoff
        .binding_env()
        .bindings()
        .get(parameter.binding)
        .expect("parameter binding row");
    assert_eq!(reserve_binding.spelling, "x");
    assert_eq!(
        reserve_binding.kind,
        mizar_checker::binding_env::BindingKind::ReservedVariable
    );
    assert_eq!(
        reserve_binding.status,
        mizar_checker::binding_env::BindingStatus::Reserved
    );
    assert_eq!(reserve_binding.owner_context, reserve_item.binding_context);
    assert_eq!(reserve_binding.declaration_range, reserve_name_range);
    assert_eq!(reserve_binding.visible_after_ordinal, 0);
    assert_eq!(
        reserve_binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(reserve_type_range)
    );
    assert_eq!(parameter_binding.spelling, "x");
    assert_eq!(
        parameter_binding.kind,
        mizar_checker::binding_env::BindingKind::DefinitionParameter
    );
    assert_eq!(
        parameter_binding.status,
        mizar_checker::binding_env::BindingStatus::Active
    );
    assert_eq!(
        parameter_binding.owner_context,
        definition_item.binding_context
    );
    assert_eq!(parameter_binding.declaration_range, parameter_name_range);
    assert_eq!(parameter_binding.visible_after_ordinal, 1);
    assert_eq!(
        parameter_binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(parameter_type_range)
    );
    assert_eq!(
        parameter_binding.identity,
        mizar_checker::binding_env::BinderIdentity::ResolverLocal {
            scope: local.scope().clone(),
            ordinal: 1,
            declaration_range: parameter_name_range,
        }
    );

    let module_context = handoff
        .binding_env()
        .contexts()
        .get(reserve_item.binding_context)
        .expect("module binding context");
    let declaration_context = handoff
        .binding_env()
        .contexts()
        .get(definition_item.binding_context)
        .expect("definition binding context");
    assert_eq!(
        module_context.owner,
        mizar_checker::binding_env::BindingContextOwner::Module
    );
    assert_eq!(module_context.parent, None);
    assert_eq!(module_context.bindings, vec![reserve.binding]);
    assert_eq!(module_context.visible_bindings, vec![reserve.binding]);
    assert_eq!(
        declaration_context.owner,
        mizar_checker::binding_env::BindingContextOwner::DeclarationShell(definition_shell.id())
    );
    assert_eq!(declaration_context.parent, Some(reserve_item.binding_context));
    assert_eq!(declaration_context.lexical_scope.as_ref(), Some(local.scope()));
    assert_eq!(declaration_context.bindings, vec![parameter.binding]);
    assert_eq!(
        declaration_context.visible_bindings,
        vec![reserve.binding, parameter.binding]
    );

    let module_link = handoff
        .context_links()
        .get(reserve_item.binding_context)
        .expect("module context link");
    let declaration_link = handoff
        .context_links()
        .get(definition_item.binding_context)
        .expect("definition context link");
    assert_eq!(module_link.local_context, reserve_item.local_context);
    assert_eq!(module_link.item, None);
    assert_eq!(
        declaration_link.local_context,
        definition_item.local_context
    );
    assert_eq!(declaration_link.item, Some(definition_item.id));
    assert_source_site(
        &first.typed_ast,
        &reserve_item.site,
        reserve_node.range,
        reserve_item.local_context,
    );
    assert_source_site(
        &first.typed_ast,
        &definition_item.site,
        definition_node.range,
        definition_item.local_context,
    );
    assert_source_site(
        &first.typed_ast,
        &reserve.site,
        reserve_name_range,
        reserve.local_context,
    );
    assert_source_site(
        &first.typed_ast,
        &parameter.site,
        parameter_name_range,
        parameter.local_context,
    );
    assert!(matches!(
        handoff.binding_env().lookup(
            &mizar_checker::binding_env::BindingLookupSite::new(
                "x",
                reserve_item.binding_context,
                None,
                2,
            )
        ),
        Ok(mizar_checker::binding_env::BindingLookupResult::Local(id)) if id == reserve.binding
    ));
    assert!(matches!(
        handoff.binding_env().lookup(
            &mizar_checker::binding_env::BindingLookupSite::new(
                "x",
                definition_item.binding_context,
                Some(mizar_resolve::names::LocalTermScope::new(vec![1, 0])),
                2,
            )
        ),
        Ok(mizar_checker::binding_env::BindingLookupResult::Local(id)) if id == parameter.binding
    ));
    assert_eq!(first.resolved.source_context(), Some(handoff));
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert_eq!(first.typed_ast.nodes().len(), 5);
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert!(!first.typed_ast.debug_text().is_empty());
    assert!(!first.resolved.debug_text().is_empty());

    let baseline = source_input_from_handoff(handoff);
    let first_projection = complete_source_projection(baseline.clone());
    let second_projection = complete_source_projection(baseline.clone());
    assert_eq!(first_projection, second_projection);
    assert_eq!(first_projection.handoff(), handoff);
    assert_eq!(
        first_projection.handoff().debug_text(),
        second_projection.handoff().debug_text()
    );
    assert!(!first_projection.handoff().debug_text().is_empty());

    let mut corrupt = baseline.clone();
    corrupt.items.pop();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnknownBindingShell { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items.push(corrupt.items[1].clone());
    corrupt.items[2].shell_ordinal = 2;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::DuplicateShell { index: 2 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings.pop();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::PartialItem { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings.push(corrupt.bindings[1].clone());
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleBindingOrdinal { index: 2 },
    );

    let mut corrupt = baseline.clone();
    let mut duplicate = corrupt.bindings[1].clone();
    duplicate.source_ordinal = 2;
    duplicate.site = mizar_checker::typed_ast::TypedSiteRef::Role {
        node: reserve_item.site.node(),
        role: mizar_checker::typed_ast::TypeRole::new("duplicate-definition-parameter"),
    };
    let mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter { local } =
        &mut duplicate.role
    else {
        panic!("baseline second binding should be a definition parameter");
    };
    *local = mizar_resolve::names::LocalTermBinding::new(
        "x",
        local.scope().clone(),
        duplicate.declaration_range,
        2,
    );
    corrupt.bindings.push(duplicate);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::DuplicateSameScopeBinding { index: 2 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items.swap(0, 1);
    corrupt.items[0].shell_ordinal = 0;
    corrupt.items[1].shell_ordinal = 1;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::ReorderedItems { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings.swap(0, 1);
    corrupt.bindings[0].source_ordinal = 0;
    corrupt.bindings[1].source_ordinal = 1;
    let declaration_range = corrupt.bindings[0].declaration_range;
    let mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter { local } =
        &mut corrupt.bindings[0].role
    else {
        panic!("reordered first binding should be a definition parameter");
    };
    *local = mizar_resolve::names::LocalTermBinding::new(
        "x",
        local.scope().clone(),
        declaration_range,
        0,
    );
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::ReorderedBindings { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[1].shell_ordinal = 8;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleShellOrdinal { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].source_ordinal = 8;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleBindingOrdinal { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[1].module_id = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task248"),
        mizar_session::ModulePath::new("other"),
    );
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::ModuleMismatch { index: 1 },
    );

    let unrelated_source = task248_other_source_id();
    let mut corrupt = baseline.clone();
    corrupt.items[1].source_range.source_id = unrelated_source;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::ItemSourceMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].declaration_range.source_id = unrelated_source;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::BindingSourceMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].written_type_range.start = definition_node.range.end + 1;
    corrupt.bindings[1].written_type_range.end = definition_node.range.end + 2;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::BindingRangeMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[1].source_range.end = parameter_name_range.start;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::BindingRangeMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].declaration_range.start = definition_node.range.end + 1;
    corrupt.bindings[1].declaration_range.end = definition_node.range.end + 2;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::BindingRangeMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[1].parent = Some(corrupt.items[1].shell);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::InvalidParent { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].source_range.end = corrupt.items[1].source_range.end;
    corrupt.items[1].parent = Some(corrupt.items[0].shell);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::InvalidParent { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[1].local_scope = Some(mizar_resolve::names::LocalTermScope::new(vec![9]));
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleLocalIdentity { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].context_owner =
        mizar_checker::source_context::SourceBindingContextOwner::Module;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RoleMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[0].role =
        mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
            local: mizar_resolve::names::LocalTermBinding::new(
                "x",
                mizar_resolve::names::LocalTermScope::new(Vec::new()),
                corrupt.bindings[0].declaration_range,
                0,
            ),
        };
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RoleMismatch { index: 0 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].role =
        mizar_checker::source_context::SourceBindingSiteRole::ReserveDefault;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RoleMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[1].visibility =
        mizar_checker::source_context::SourceItemVisibility::Public;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedVisibility { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].site = corrupt.bindings[0].site.clone();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::DuplicateTypedSite,
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].recovery =
        mizar_checker::binding_env::BindingRecoveryState::Recovered;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RecoveredBinding { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[1].recovery =
        mizar_checker::source_context::SourceItemRecovery::Recovered;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RecoveredItemClaimsBinding { index: 1 },
    );

    let mut corrupt = baseline.clone();
    let stale_range = corrupt.bindings[1].declaration_range;
    let mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter { local } =
        &mut corrupt.bindings[1].role
    else {
        panic!("baseline second binding should be a definition parameter");
    };
    *local = mizar_resolve::names::LocalTermBinding::new(
        "x",
        local.scope().clone(),
        mizar_session::SourceRange {
            start: stale_range.start + 1,
            ..stale_range
        },
        1,
    );
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleLocalIdentity { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].spelling = "y".to_owned();
    let different_spelling_range = corrupt.bindings[1].declaration_range;
    let mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter { local } =
        &mut corrupt.bindings[1].role
    else {
        panic!("baseline second binding should be a definition parameter");
    };
    *local = mizar_resolve::names::LocalTermBinding::new(
        "y",
        local.scope().clone(),
        different_spelling_range,
        1,
    );
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::MissingRequiredShadow,
    );

    let mut unsupported_rereserve = baseline.clone();
    unsupported_rereserve.items[1].role =
        mizar_checker::source_context::SourceItemRole::Reserve;
    unsupported_rereserve.items[1].local_scope = None;
    unsupported_rereserve.bindings[1].context_owner =
        mizar_checker::source_context::SourceBindingContextOwner::Module;
    unsupported_rereserve.bindings[1].role =
        mizar_checker::source_context::SourceBindingSiteRole::ReserveDefault;
    assert_source_context_error(
        unsupported_rereserve,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut recovered = baseline.clone();
    recovered.items[1].recovery =
        mizar_checker::source_context::SourceItemRecovery::Recovered;
    recovered.bindings.pop();
    let first_recovery = mizar_checker::source_context::SourceBindingContextProducer::build(
        recovered.clone(),
    )
    .expect("recovered-empty input should be supported");
    let second_recovery =
        mizar_checker::source_context::SourceBindingContextProducer::build(recovered)
            .expect("recovered-empty input should be deterministic");
    assert_eq!(first_recovery, second_recovery);
    let mizar_checker::source_context::SourceBindingContextBuild::Incomplete(incomplete) =
        &first_recovery
    else {
        panic!("recovered-empty input must not publish a complete handoff");
    };
    assert_eq!(incomplete.recovered_shell(), definition_shell.id());
    assert_eq!(
        incomplete.recovered_context(),
        mizar_checker::binding_env::BindingContextId::new(1)
    );
    assert_eq!(
        incomplete.diagnostic(),
        mizar_checker::binding_env::BindingDiagnosticId::new(0)
    );
    let diagnostic = incomplete
        .binding_env()
        .diagnostics()
        .get(incomplete.diagnostic())
        .expect("recovery diagnostic");
    assert_eq!(diagnostic.source_range, Some(definition_node.range));
    assert_eq!(
        diagnostic.class,
        mizar_checker::binding_env::BindingDiagnosticClass::RecoveredContextBoundary
    );
    assert_eq!(
        diagnostic.severity,
        mizar_checker::binding_env::BindingDiagnosticSeverity::Error
    );
    assert_eq!(
        diagnostic.message_key,
        "checker.binding.source_context.recovered"
    );
    assert_eq!(
        diagnostic.recovery,
        mizar_checker::binding_env::BindingDiagnosticRecovery::Recovery
    );
    assert_eq!(
        first_recovery.into_complete(),
        Err(mizar_checker::source_context::SourceContextError::IncompleteRecovery)
    );

    let valid_handoff = complete_source_projection(baseline.clone()).into_handoff();
    assert!(matches!(
        typed_ast_with_source_context(
            &first.typed_ast,
            valid_handoff.clone(),
            mizar_checker::typed_ast::LocalTypeContextTable::new(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidNodeContext { .. })
    ));

    let mut mismatched_contexts = mizar_checker::typed_ast::LocalTypeContextTable::new();
    for (id, context) in valid_handoff.local_contexts().iter() {
        mismatched_contexts.insert(mizar_checker::typed_ast::LocalTypeContextDraft {
            owner: if id.index() == 0 {
                mizar_checker::typed_ast::TypedSiteRef::Role {
                    node: definition_item.site.node(),
                    role: mizar_checker::typed_ast::TypeRole::new("wrong-module-owner"),
                }
            } else {
                context.owner.clone()
            },
            parent: context.parent,
            layer: context.layer,
            bindings: context.bindings.clone(),
            introduced_assumptions: context.introduced_assumptions.clone(),
            visible_facts: context.visible_facts.clone(),
            recovery: context.recovery,
        });
    }
    assert_eq!(
        typed_ast_with_source_context(
            &first.typed_ast,
            valid_handoff.clone(),
            mismatched_contexts,
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceContext)
    );

    let mut wrong_module_site = baseline.clone();
    wrong_module_site.module_site = mizar_checker::typed_ast::TypedSiteRef::Role {
        node: definition_item.site.node(),
        role: mizar_checker::typed_ast::TypeRole::new("wrong-module-site"),
    };
    let wrong_module_handoff = complete_source_projection(wrong_module_site).into_handoff();
    assert_eq!(
        typed_ast_with_source_context(
            &first.typed_ast,
            wrong_module_handoff.clone(),
            wrong_module_handoff.local_contexts().clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceContext)
    );

    let mut wrong_declaration_site = baseline.clone();
    wrong_declaration_site.bindings[1].site =
        mizar_checker::typed_ast::TypedSiteRef::Role {
            node: first
                .typed_ast
                .nodes()
                .root()
                .expect("typed root for invalid-site corruption"),
            role: mizar_checker::typed_ast::TypeRole::new("wrong-declaration-site"),
        };
    let wrong_declaration_handoff =
        complete_source_projection(wrong_declaration_site).into_handoff();
    assert_eq!(
        typed_ast_with_source_context(
            &first.typed_ast,
            wrong_declaration_handoff.clone(),
            wrong_declaration_handoff.local_contexts().clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceContext)
    );

    assert!(typed_ast_with_source_context(
        &first.typed_ast,
        valid_handoff.clone(),
        valid_handoff.local_contexts().clone(),
    )
    .is_ok());

    assert!(source_binding_context_token_shape_is_exact(
        &["reserve", "x", "for", "set", ";"],
        &["definition", "let", "x", "be", "set", ";", "end", ";"],
    ));
    for (reserve_tokens, definition_tokens) in [
        (
            vec!["reserve", "y", "for", "set", ";"],
            vec!["definition", "let", "x", "be", "set", ";", "end", ";"],
        ),
        (
            vec!["reserve", "x", "for", "object", ";"],
            vec!["definition", "let", "x", "be", "set", ";", "end", ";"],
        ),
        (
            vec!["reserve", "x", "for", "set", ";"],
            vec!["definition", "let", "y", "be", "set", ";", "end", ";"],
        ),
        (
            vec!["reserve", "x", "for", "set", ";"],
            vec!["definition", "let", "x", "be", "object", ";", "end", ";"],
        ),
        (
            vec!["reserve", "x", "for", "set", ";"],
            vec![
                "definition",
                "let",
                "x",
                "be",
                "set",
                ";",
                "let",
                "y",
                "be",
                "set",
                ";",
                "end",
                ";",
            ],
        ),
    ] {
        assert!(!source_binding_context_token_shape_is_exact(
            &reserve_tokens,
            &definition_tokens,
        ));
    }
}

fn sole_token_range(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    spelling: &str,
) -> mizar_session::SourceRange {
    let mut ranges = Vec::new();
    collect_token_ranges(ast, node, spelling, &mut ranges);
    let [range] = ranges.as_slice() else {
        panic!("expected one `{spelling}` token, got {ranges:?}");
    };
    *range
}

fn collect_token_ranges(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    spelling: &str,
    ranges: &mut Vec<mizar_session::SourceRange>,
) {
    if node.token_text() == Some(spelling) {
        ranges.push(node.range);
    }
    for child in &node.children {
        if let Some(child) = ast.node(*child) {
            collect_token_ranges(ast, child, spelling, ranges);
        }
    }
}

fn assert_source_site(
    typed_ast: &mizar_checker::typed_ast::TypedAst,
    site: &mizar_checker::typed_ast::TypedSiteRef,
    range: mizar_session::SourceRange,
    context: mizar_checker::typed_ast::LocalTypeContextId,
) {
    let node = typed_ast
        .nodes()
        .node(site.node())
        .expect("source-context site should reference a typed node");
    assert_eq!(node.anchor, mizar_session::SourceAnchor::Range(range));
    assert_eq!(node.links.context, Some(context));
}

fn source_input_from_handoff(
    handoff: &mizar_checker::source_context::SourceBindingContextHandoff,
) -> mizar_checker::source_context::SourceBindingContextInput {
    let module_site = handoff
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        .expect("module local context")
        .owner
        .clone();
    let items = handoff
        .items()
        .iter()
        .map(|(_, item)| mizar_checker::source_context::SourceItemInput {
            shell: item.shell,
            shell_ordinal: item.shell_ordinal,
            role: item.role,
            module_id: handoff.module_id().clone(),
            source_range: item.source_range,
            parent: item.parent,
            visibility: item.visibility,
            site: item.site.clone(),
            local_scope: item.local_scope.clone(),
            recovery: item.recovery,
        })
        .collect();
    let bindings = handoff
        .declarations()
        .iter()
        .map(|(_, declaration)| {
            let item = handoff
                .items()
                .get(declaration.item)
                .expect("declaration item");
            let binding = handoff
                .binding_env()
                .bindings()
                .get(declaration.binding)
                .expect("declaration binding");
            let context_owner = if matches!(
                declaration.role,
                mizar_checker::source_context::SourceBindingSiteRole::ReserveDefault
            ) {
                mizar_checker::source_context::SourceBindingContextOwner::Module
            } else {
                mizar_checker::source_context::SourceBindingContextOwner::Shell(item.shell)
            };
            mizar_checker::source_context::SourceBindingSiteInput {
                shell: item.shell,
                context_owner,
                source_ordinal: declaration.source_ordinal,
                spelling: declaration.spelling.clone(),
                declaration_range: declaration.declaration_range,
                written_type_range: declaration.written_type_range,
                site: declaration.site.clone(),
                role: declaration.role.clone(),
                recovery: binding.recovery,
            }
        })
        .collect();
    mizar_checker::source_context::SourceBindingContextInput {
        source_id: handoff.source_id(),
        module_id: handoff.module_id().clone(),
        module_site,
        items,
        bindings,
    }
}

fn complete_source_projection(
    input: mizar_checker::source_context::SourceBindingContextInput,
) -> mizar_checker::source_context::SourceBindingContextProjection {
    mizar_checker::source_context::SourceBindingContextProducer::build(input)
        .expect("valid Task 248 source projection")
        .into_complete()
        .expect("complete Task 248 source projection")
}

fn assert_source_context_error(
    input: mizar_checker::source_context::SourceBindingContextInput,
    expected: mizar_checker::source_context::SourceContextError,
) {
    assert_eq!(
        mizar_checker::source_context::SourceBindingContextProducer::build(input),
        Err(expected)
    );
}

fn typed_ast_with_source_context(
    template: &mizar_checker::typed_ast::TypedAst,
    source_context: mizar_checker::source_context::SourceBindingContextHandoff,
    contexts: mizar_checker::typed_ast::LocalTypeContextTable,
) -> Result<mizar_checker::typed_ast::TypedAst, mizar_checker::typed_ast::TypedAstError> {
    mizar_checker::typed_ast::TypedAst::try_new(mizar_checker::typed_ast::TypedAstParts {
        source_id: template.source_id(),
        module_id: template.module_id().clone(),
        resolved_root: template.resolved_root(),
        source_context: Some(source_context),
        nodes: template.nodes().clone(),
        contexts,
        types: mizar_checker::typed_ast::TypeTable::new(),
        facts: mizar_checker::typed_ast::TypeFactTable::new(),
        coercions: mizar_checker::typed_ast::CoercionTable::new(),
        initial_obligations: mizar_checker::typed_ast::InitialObligationTable::new(),
        diagnostics: mizar_checker::typed_ast::TypeDiagnosticTable::new(),
    })
}

fn task248_other_source_id() -> mizar_session::SourceId {
    use mizar_session::SessionIdAllocator as _;
    let snapshot = mizar_session::BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "42".repeat(32)
    ))
    .expect("valid Task 248 corruption snapshot");
    let allocator = mizar_session::InMemorySessionIdAllocator::new();
    allocator
        .next_source_id(snapshot)
        .expect("first Task 248 corruption source id");
    allocator
        .next_source_id(snapshot)
        .expect("distinct Task 248 corruption source id")
}
