const TASK248_TWO_PARAMETER_SOURCE: &str = concat!(
    "definition\n",
    "  let x be set;\n",
    "  let y be set;\n",
    "  assume x = x;\n",
    "  pred Task259PredicateDefinition: x task259_rel y means x = y;\n",
    "  symmetry by computation(steps: 1);\n",
    "end;\n",
);

const TASK248_TWO_PARAMETER_EXCLUDED_VARIANT: &str = concat!(
    "definition\n",
    "  let x be set;\n",
    "  let y be set;\n",
    "  assume y = y;\n",
    "  pred Task259PredicateDefinition: y task248_alt x means y = x;\n",
    "  symmetry by computation(steps: 2);\n",
    "end;\n",
);

const TASK248_TWO_PARAMETER_DEBUG: &str = concat!(
    "source-binding-context-debug-v1\n",
    "module: task248.two_parameter_profile\n",
    "item#0 shell=0 ordinal=0 role=definition-block range=0..164 parent=none context=1 local_context=1 predecessor=none\n",
    "declaration#0 item=0 binding=0 ordinal=0 role=definition-parameter range=17..18 type_range=22..25 context=1 local_context=1 shadowed=none predecessor=none\n",
    "declaration#1 item=0 binding=1 ordinal=1 role=definition-parameter range=33..34 type_range=38..41 context=1 local_context=1 shadowed=none predecessor=0\n",
    "context-link#0 binding_context=0 local_context=0 item=module\n",
    "context-link#1 binding_context=1 local_context=1 item=0\n",
);

#[test]
fn task248_two_parameter_definition_profile_publishes_dense_context() {
    assert_eq!(TASK248_TWO_PARAMETER_SOURCE.len(), 165);
    let (ast, module, shells, symbols, diagnostics) =
        task248_two_parameter_frontend(TASK248_TWO_PARAMETER_SOURCE, 0);
    assert_eq!(diagnostics, 0);
    assert_eq!(ast.nodes().len(), 71);
    assert_eq!(ast.root().map(mizar_syntax::SurfaceNodeId::index), Some(70));
    assert_eq!(
        ast.root().and_then(|root| ast.node(root)).map(|node| node.range),
        Some(task248_two_parameter_range(ast.source_id, 0, 164))
    );
    assert_eq!(shells.declarations().len(), 3);

    let definition = task248_two_parameter_definition_node(&ast);
    let (nodes, sites) = task248_two_parameter_arena(&ast, definition);
    let first = source_two_parameter_definition_context_projection(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        definition,
        &nodes,
        sites,
    )
    .expect("exact Task 248 Profile B should project");
    let (_, replay_sites) = task248_two_parameter_arena(&ast, definition);
    let second = source_two_parameter_definition_context_projection(
        &ast,
        module,
        &shells,
        &symbols,
        definition,
        &nodes,
        replay_sites,
    )
    .expect("exact Task 248 Profile B replay should project");
    assert_eq!(first, second);

    let handoff = first.handoff();
    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id().path().as_str(), "task248.two_parameter_profile");
    assert_eq!(handoff.binding_env().source_id(), ast.source_id);
    assert_eq!(handoff.binding_env().module_id(), handoff.module_id());
    assert_eq!(handoff.items().len(), 1);
    assert_eq!(handoff.declarations().len(), 2);
    assert_eq!(handoff.binding_env().contexts().len(), 2);
    assert_eq!(handoff.binding_env().bindings().len(), 2);
    assert_eq!(handoff.local_contexts().len(), 2);
    assert_eq!(handoff.context_links().len(), 2);
    assert!(handoff.binding_env().diagnostics().is_empty());

    let item = handoff
        .items()
        .get(mizar_checker::source_context::SourceItemId::new(0))
        .expect("Profile B definition item");
    assert_eq!(
        item.id,
        mizar_checker::source_context::SourceItemId::new(0)
    );
    assert_eq!(
        item.role,
        mizar_checker::source_context::SourceItemRole::DefinitionBlock
    );
    assert_eq!(item.shell.index(), 0);
    assert_eq!(item.shell_ordinal, 0);
    assert_eq!(
        item.source_range,
        task248_two_parameter_range(ast.source_id, 0, 164)
    );
    assert_eq!(item.parent, None);
    assert_eq!(
        item.visibility,
        mizar_checker::source_context::SourceItemVisibility::Unspecified
    );
    assert_eq!(
        item.site,
        mizar_checker::typed_ast::TypedSiteRef::Node(
            mizar_checker::typed_ast::TypedNodeId::new(2)
        )
    );
    assert_eq!(item.binding_context.index(), 1);
    assert_eq!(item.local_context.index(), 1);
    assert_eq!(
        item.local_scope.as_ref().map(|scope| scope.path()),
        Some(&[0][..])
    );
    assert_eq!(
        item.recovery,
        mizar_checker::source_context::SourceItemRecovery::Normal
    );
    assert_eq!(item.predecessor, None);

    let declarations = handoff
        .declarations()
        .iter()
        .map(|(_, declaration)| declaration)
        .collect::<Vec<_>>();
    assert_eq!(
        declarations
            .iter()
            .map(|declaration| declaration.spelling.as_str())
            .collect::<Vec<_>>(),
        ["x", "y"]
    );
    for (index, declaration) in declarations.iter().enumerate() {
        assert_eq!(
            declaration.id,
            mizar_checker::source_context::SourceDeclarationId::new(index)
        );
        assert_eq!(
            declaration.item,
            mizar_checker::source_context::SourceItemId::new(0)
        );
        assert_eq!(
            declaration.binding,
            mizar_checker::binding_env::BindingId::new(index)
        );
        assert_eq!(declaration.source_ordinal, index);
        assert_eq!(
            declaration.declaration_range,
            task248_two_parameter_range(ast.source_id, 17 + 16 * index, 18 + 16 * index)
        );
        assert_eq!(
            declaration.written_type_range,
            task248_two_parameter_range(ast.source_id, 22 + 16 * index, 25 + 16 * index)
        );
        assert_eq!(
            declaration.site,
            mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(index)
            )
        );
        let mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
            local,
        } = &declaration.role
        else {
            panic!("Profile B declaration {index} should retain its resolver-local role");
        };
        assert_eq!(local.spelling(), ["x", "y"][index]);
        assert_eq!(local.scope().path(), &[0]);
        assert_eq!(local.declaration_range(), declaration.declaration_range);
        assert_eq!(local.visible_after_ordinal(), index);
        assert_eq!(declaration.binding_context.index(), 1);
        assert_eq!(declaration.local_context.index(), 1);
        assert_eq!(declaration.shadowed_binding, None);
    }
    assert_eq!(declarations[0].predecessor, None);
    assert_eq!(
        declarations[1].predecessor,
        Some(mizar_checker::source_context::SourceDeclarationId::new(0))
    );

    let bindings = handoff
        .binding_env()
        .bindings()
        .iter()
        .map(|(_, binding)| binding)
        .collect::<Vec<_>>();
    for (index, binding) in bindings.iter().enumerate() {
        assert_eq!(binding.id, mizar_checker::binding_env::BindingId::new(index));
        assert_eq!(binding.spelling, ["x", "y"][index]);
        assert_eq!(
            binding.kind,
            mizar_checker::binding_env::BindingKind::DefinitionParameter
        );
        assert_eq!(
            binding.status,
            mizar_checker::binding_env::BindingStatus::Active
        );
        assert_eq!(binding.owner_context.index(), 1);
        assert_eq!(binding.declaration_range, declarations[index].declaration_range);
        assert_eq!(binding.visible_after_ordinal, index);
        assert!(binding.captured.identities().is_empty());
        assert_eq!(
            binding.recovery,
            mizar_checker::binding_env::BindingRecoveryState::Normal
        );
        assert!(binding.diagnostics.is_empty());
        let mizar_checker::binding_env::BinderIdentity::ResolverLocal {
            scope,
            ordinal,
            declaration_range,
        } = &binding.identity
        else {
            panic!("Profile B binding {index} should retain resolver-local identity");
        };
        assert_eq!(scope.path(), &[0]);
        assert_eq!(*ordinal, index);
        assert_eq!(*declaration_range, declarations[index].declaration_range);
        assert_eq!(
            binding.type_site,
            mizar_checker::binding_env::BindingTypeSite::Source(
                declarations[index].written_type_range
            )
        );
    }

    let module_context = handoff
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("Profile B module context");
    let definition_context = handoff
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("Profile B definition context");
    assert_eq!(
        module_context.id,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    assert_eq!(
        module_context.owner,
        mizar_checker::binding_env::BindingContextOwner::Module
    );
    assert_eq!(module_context.parent, None);
    assert_eq!(
        module_context.layer,
        mizar_checker::binding_env::BindingContextLayer::Module
    );
    assert_eq!(module_context.lexical_scope, None);
    assert!(module_context.bindings.is_empty());
    assert!(module_context.visible_bindings.is_empty());
    assert_eq!(
        module_context.recovery,
        mizar_checker::binding_env::BindingContextRecovery::Normal
    );
    assert_eq!(
        definition_context.id,
        mizar_checker::binding_env::BindingContextId::new(1)
    );
    assert_eq!(
        definition_context.owner,
        mizar_checker::binding_env::BindingContextOwner::DeclarationShell(item.shell)
    );
    assert_eq!(
        definition_context.parent,
        Some(mizar_checker::binding_env::BindingContextId::new(0))
    );
    assert_eq!(
        definition_context.layer,
        mizar_checker::binding_env::BindingContextLayer::Declaration
    );
    assert_eq!(
        definition_context
            .bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        definition_context
            .visible_bindings
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        definition_context.lexical_scope.as_ref().map(|scope| scope.path()),
        Some(&[0][..])
    );
    assert_eq!(
        definition_context.recovery,
        mizar_checker::binding_env::BindingContextRecovery::Normal
    );

    let module_local = handoff
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(0))
        .expect("Profile B module local context");
    let definition_local = handoff
        .local_contexts()
        .get(mizar_checker::typed_ast::LocalTypeContextId::new(1))
        .expect("Profile B definition local context");
    assert_eq!(
        module_local.id,
        mizar_checker::typed_ast::LocalTypeContextId::new(0)
    );
    assert_eq!(
        module_local.owner,
        mizar_checker::typed_ast::TypedSiteRef::Node(
            mizar_checker::typed_ast::TypedNodeId::new(3)
        )
    );
    assert_eq!(module_local.parent, None);
    assert_eq!(
        module_local.layer,
        mizar_checker::typed_ast::TypeContextLayer::Module
    );
    assert!(module_local.bindings.is_empty());
    assert!(module_local.introduced_assumptions.is_empty());
    assert!(module_local.visible_facts.is_empty());
    assert_eq!(
        module_local.recovery,
        mizar_checker::typed_ast::ContextRecoveryState::Normal
    );
    assert_eq!(
        definition_local.id,
        mizar_checker::typed_ast::LocalTypeContextId::new(1)
    );
    assert_eq!(
        definition_local.owner,
        mizar_checker::typed_ast::TypedSiteRef::Node(
            mizar_checker::typed_ast::TypedNodeId::new(2)
        )
    );
    assert_eq!(
        definition_local.parent,
        Some(mizar_checker::typed_ast::LocalTypeContextId::new(0))
    );
    assert_eq!(
        definition_local.layer,
        mizar_checker::typed_ast::TypeContextLayer::Declaration
    );
    assert_eq!(
        definition_local.bindings,
        [
            mizar_checker::typed_ast::BindingTypeRef::Site(
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(0)
                )
            ),
            mizar_checker::typed_ast::BindingTypeRef::Site(
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(1)
                )
            ),
        ]
    );
    assert!(definition_local.introduced_assumptions.is_empty());
    assert!(definition_local.visible_facts.is_empty());
    assert_eq!(
        definition_local.recovery,
        mizar_checker::typed_ast::ContextRecoveryState::Normal
    );

    let links = handoff
        .context_links()
        .iter()
        .map(|(_, link)| link)
        .collect::<Vec<_>>();
    assert_eq!(
        links[0].binding_context,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    assert_eq!(
        links[0].local_context,
        mizar_checker::typed_ast::LocalTypeContextId::new(0)
    );
    assert_eq!(links[0].item, None);
    assert_eq!(
        links[1].binding_context,
        mizar_checker::binding_env::BindingContextId::new(1)
    );
    assert_eq!(
        links[1].local_context,
        mizar_checker::typed_ast::LocalTypeContextId::new(1)
    );
    assert_eq!(
        links[1].item,
        Some(mizar_checker::source_context::SourceItemId::new(0))
    );
    assert_eq!(handoff.debug_text(), TASK248_TWO_PARAMETER_DEBUG);
}

#[test]
fn task248_two_parameter_definition_profile_rejects_corruption() {
    let (ast, module, shells, symbols, _) =
        task248_two_parameter_frontend(TASK248_TWO_PARAMETER_SOURCE, 10);
    let definition = task248_two_parameter_definition_node(&ast);
    let (nodes, sites) = task248_two_parameter_arena(&ast, definition);
    let projection = source_two_parameter_definition_context_projection(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        definition,
        &nodes,
        sites,
    )
    .expect("Profile B baseline");
    let baseline = source_input_from_handoff(projection.handoff());

    let mut corrupt = baseline.clone();
    corrupt.items.clear();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::MissingItems,
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings.clear();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::PartialItem { index: 0 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings.pop();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut corrupt = baseline.clone();
    let mut third = corrupt.bindings[1].clone();
    third.source_ordinal = 2;
    third.spelling = "z".to_owned();
    third.declaration_range = task248_two_parameter_range(ast.source_id, 40, 41);
    third.written_type_range = task248_two_parameter_range(ast.source_id, 38, 41);
    third.site = mizar_checker::typed_ast::TypedSiteRef::Role {
        node: corrupt.module_site.node(),
        role: mizar_checker::typed_ast::TypeRole::new("task248.third-parameter"),
    };
    third.role = mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
        local: mizar_resolve::names::LocalTermBinding::new(
            "z",
            mizar_resolve::names::LocalTermScope::new(vec![0]),
            third.declaration_range,
            2,
        ),
    };
    corrupt.bindings.push(third);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut corrupt = baseline.clone();
    let first_range = corrupt.bindings[0].declaration_range;
    corrupt.bindings[1].declaration_range = first_range;
    corrupt.bindings[1].role =
        mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
            local: mizar_resolve::names::LocalTermBinding::new(
                "y",
                mizar_resolve::names::LocalTermScope::new(vec![0]),
                first_range,
                1,
            ),
        };
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].written_type_range = corrupt.bindings[0].written_type_range;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].recovery =
        mizar_checker::source_context::SourceItemRecovery::Recovered;
    corrupt.bindings.clear();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].recovery =
        mizar_checker::source_context::SourceItemRecovery::Recovered;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RecoveredItemClaimsBinding { index: 0 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].recovery =
        mizar_checker::binding_env::BindingRecoveryState::Recovered;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RecoveredBinding { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].spelling = "x".to_owned();
    corrupt.bindings[1].role =
        mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
            local: mizar_resolve::names::LocalTermBinding::new(
                "x",
                mizar_resolve::names::LocalTermScope::new(vec![0]),
                corrupt.bindings[1].declaration_range,
                1,
            ),
        };
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::DuplicateSameScopeBinding { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].source_ordinal = 7;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleBindingOrdinal { index: 1 },
    );

    let mut corrupt = baseline.clone();
    let declaration_range = corrupt.bindings[1].declaration_range;
    corrupt.bindings[1].role =
        mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
            local: mizar_resolve::names::LocalTermBinding::new(
                "y",
                mizar_resolve::names::LocalTermScope::new(vec![0]),
                declaration_range,
                7,
            ),
        };
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleLocalIdentity { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].declaration_range =
        task248_two_parameter_range(ast.source_id, 16, 17);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::ReorderedBindings { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].shell = shells.declarations()[1].id();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnknownBindingShell { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].context_owner =
        mizar_checker::source_context::SourceBindingContextOwner::Module;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::RoleMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].spelling.clear();
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::EmptyBindingSpelling { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].module_id = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task248"),
        mizar_session::ModulePath::new("task248.other"),
    );
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::ModuleMismatch { index: 0 },
    );

    let unrelated_source = task248_other_source_id();
    let mut corrupt = baseline.clone();
    corrupt.items[0].source_range.source_id = unrelated_source;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::ItemSourceMismatch { index: 0 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].written_type_range.source_id = unrelated_source;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::BindingSourceMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.bindings[1].written_type_range =
        task248_two_parameter_range(ast.source_id, 165, 166);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::BindingRangeMismatch { index: 1 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].shell_ordinal = 1;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::StaleShellOrdinal { index: 0 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].parent = Some(corrupt.items[0].shell);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::InvalidParent { index: 0 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].local_scope = None;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::InvalidItemContext { index: 0 },
    );

    let mut corrupt = baseline.clone();
    corrupt.items[0].visibility =
        mizar_checker::source_context::SourceItemVisibility::Public;
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedVisibility { index: 0 },
    );

    let mut corrupt = baseline.clone();
    let mut duplicate = corrupt.items[0].clone();
    duplicate.shell_ordinal = 1;
    duplicate.site = mizar_checker::typed_ast::TypedSiteRef::Role {
        node: corrupt.module_site.node(),
        role: mizar_checker::typed_ast::TypeRole::new("task248.duplicate-shell"),
    };
    corrupt.items.push(duplicate);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::DuplicateShell { index: 1 },
    );

    for (left, right) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)] {
        let mut corrupt = baseline.clone();
        let duplicate_site = task248_source_input_site(&corrupt, left);
        task248_set_source_input_site(&mut corrupt, right, duplicate_site);
        assert_source_context_error(
            corrupt,
            mizar_checker::source_context::SourceContextError::DuplicateTypedSite,
        );
    }

    let mut corrupt = baseline.clone();
    corrupt.items[0].role = mizar_checker::source_context::SourceItemRole::Reserve;
    corrupt.items[0].local_scope = None;
    for binding in &mut corrupt.bindings {
        binding.context_owner = mizar_checker::source_context::SourceBindingContextOwner::Module;
        binding.role = mizar_checker::source_context::SourceBindingSiteRole::ReserveDefault;
    }
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut corrupt = baseline.clone();
    let mut extra_item = corrupt.items[0].clone();
    extra_item.shell = shells.declarations()[1].id();
    extra_item.shell_ordinal = 1;
    extra_item.site = mizar_checker::typed_ast::TypedSiteRef::Role {
        node: corrupt.module_site.node(),
        role: mizar_checker::typed_ast::TypeRole::new("task248.extra-item"),
    };
    corrupt.items.push(extra_item);
    let mut extra_binding = corrupt.bindings[1].clone();
    extra_binding.shell = shells.declarations()[1].id();
    extra_binding.context_owner =
        mizar_checker::source_context::SourceBindingContextOwner::Shell(
            shells.declarations()[1].id(),
        );
    extra_binding.source_ordinal = 2;
    extra_binding.spelling = "z".to_owned();
    extra_binding.declaration_range = task248_two_parameter_range(ast.source_id, 50, 51);
    extra_binding.written_type_range = task248_two_parameter_range(ast.source_id, 52, 55);
    extra_binding.site = mizar_checker::typed_ast::TypedSiteRef::Role {
        node: corrupt.module_site.node(),
        role: mizar_checker::typed_ast::TypeRole::new("task248.extra-binding"),
    };
    extra_binding.role =
        mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
            local: mizar_resolve::names::LocalTermBinding::new(
                "z",
                mizar_resolve::names::LocalTermScope::new(vec![0]),
                extra_binding.declaration_range,
                2,
            ),
        };
    corrupt.bindings.push(extra_binding);
    assert_source_context_error(
        corrupt,
        mizar_checker::source_context::SourceContextError::UnsupportedTaskShape,
    );

    let mut substitution = baseline;
    for (index, spelling) in ["a", "b"].into_iter().enumerate() {
        substitution.bindings[index].spelling = spelling.to_owned();
        let range = substitution.bindings[index].declaration_range;
        substitution.bindings[index].role =
            mizar_checker::source_context::SourceBindingSiteRole::DefinitionParameter {
                local: mizar_resolve::names::LocalTermBinding::new(
                    spelling,
                    mizar_resolve::names::LocalTermScope::new(vec![0]),
                    range,
                    index,
                ),
            };
    }
    assert!(matches!(
        mizar_checker::source_context::SourceBindingContextProducer::build(substitution),
        Ok(mizar_checker::source_context::SourceBindingContextBuild::Complete(_))
    ));
}

#[test]
fn task248_two_parameter_definition_extractor_is_default_deny() {
    let (ast, module, shells, symbols, _) =
        task248_two_parameter_frontend(TASK248_TWO_PARAMETER_SOURCE, 20);
    let definition = task248_two_parameter_definition_node(&ast);
    let (nodes, baseline_sites) = task248_two_parameter_arena(&ast, definition);

    assert!(
        source_binding_context_output(&ast, module.clone(), &shells, &symbols).is_none(),
        "the active Profile-A selector must remain dormant for Profile B"
    );
    assert!(
        source_binding_context_detail_keys(&ast, module.clone(), &shells, &symbols).is_none(),
        "no active detail key may select Profile B"
    );
    let baseline = source_two_parameter_definition_context_projection(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        definition,
        &nodes,
        baseline_sites,
    )
    .expect("baseline Task 248 Profile B projection");

    let (_, wrong_module_sites) = task248_two_parameter_arena(&ast, definition);
    let wrong_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task248"),
        mizar_session::ModulePath::new("task248.other"),
    );
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            wrong_module,
            &shells,
            &symbols,
            definition,
            &nodes,
            wrong_module_sites,
        ),
        Err(
            "two-parameter source binding context uses another symbol module".to_owned()
        )
    );

    let cross_shell_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task248"),
        mizar_session::ModulePath::new("task248.cross_shell"),
    );
    let cross_shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &cross_shell_module)
            .collect();
    let (_, cross_shell_sites) = task248_two_parameter_arena(&ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &cross_shells,
            &symbols,
            definition,
            &nodes,
            cross_shell_sites,
        ),
        Err("source binding context shell 0 is inconsistent".to_owned())
    );

    let wrong_symbols = mizar_resolve::env::SymbolEnv::new(
        mizar_resolve::resolved_ast::ModuleId::new(
            mizar_session::PackageId::new("task248"),
            mizar_session::ModulePath::new("task248.other"),
        ),
        mizar_resolve::env::SymbolEnvIndexes::default(),
    );
    let (_, wrong_symbol_sites) = task248_two_parameter_arena(&ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &shells,
            &wrong_symbols,
            definition,
            &nodes,
            wrong_symbol_sites,
        ),
        Err(
            "two-parameter source binding context uses another symbol module".to_owned()
        )
    );

    let first_parameter = surface_nodes_with_kind(
        &ast,
        mizar_syntax::SurfaceNodeKind::DefinitionParameter,
    )[0]
        .0;
    let (_, wrong_node_sites) = task248_two_parameter_arena(&ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            first_parameter,
            &nodes,
            wrong_node_sites,
        ),
        Err(
            "two-parameter source binding context definition identity is not exact".to_owned()
        )
    );

    let definition_node = ast.node(definition).expect("Task 248 definition node");
    let definition_shell = &shells.declarations()[0];
    validate_source_context_shell_for_test(
        definition_shell,
        0,
        mizar_resolve::declarations::DeclarationShellKind::DefinitionBlock,
        &module,
        definition,
        definition_node,
    )
    .expect("baseline real resolver shell should validate");
    assert_task248_shell_validation_error(
        definition_shell,
        1,
        mizar_resolve::declarations::DeclarationShellKind::DefinitionBlock,
        &module,
        definition,
        definition_node,
    );
    assert_task248_shell_validation_error(
        definition_shell,
        0,
        mizar_resolve::declarations::DeclarationShellKind::Reserve,
        &module,
        definition,
        definition_node,
    );
    assert_task248_shell_validation_error(
        definition_shell,
        0,
        mizar_resolve::declarations::DeclarationShellKind::DefinitionBlock,
        &module,
        first_parameter,
        definition_node,
    );
    let root_node = ast
        .root()
        .and_then(|root| ast.node(root))
        .expect("Task 248 root node");
    assert_task248_shell_validation_error(
        definition_shell,
        0,
        mizar_resolve::declarations::DeclarationShellKind::DefinitionBlock,
        &module,
        definition,
        root_node,
    );

    let leading_source = format!("\n{TASK248_TWO_PARAMETER_SOURCE}");
    let (_leading_ast, leading_module, leading_shells, _, _) =
        task248_two_parameter_frontend(&leading_source, 90);
    let leading_shell = leading_shells
        .declarations()
        .iter()
        .find(|shell| {
            shell.kind() == mizar_resolve::declarations::DeclarationShellKind::DefinitionBlock
        })
        .expect("leading-newline source definition shell");
    assert_task248_shell_validation_error(
        leading_shell,
        leading_shell.ordinal(),
        leading_shell.kind(),
        &leading_module,
        leading_shell.node_id(),
        definition_node,
    );

    let inner_shell = &shells.declarations()[1];
    assert!(inner_shell.parent().is_some());
    let inner_node = ast
        .node(inner_shell.node_id())
        .expect("nested predicate shell node");
    assert_task248_shell_validation_error(
        inner_shell,
        inner_shell.ordinal(),
        inner_shell.kind(),
        &module,
        inner_shell.node_id(),
        inner_node,
    );

    let (visible_ast, visible_module, visible_shells, _, _) =
        task248_two_parameter_frontend("public theorem Visible: thesis;\n", 91);
    let visible_shell = visible_shells
        .declarations()
        .first()
        .expect("public theorem shell");
    let visible_node = visible_ast
        .node(visible_shell.node_id())
        .expect("public theorem node");
    assert_ne!(
        visible_shell.visibility().state(),
        mizar_resolve::declarations::DeclarationShellVisibilityState::Unspecified
    );
    assert_task248_shell_validation_error(
        visible_shell,
        visible_shell.ordinal(),
        visible_shell.kind(),
        &visible_module,
        visible_shell.node_id(),
        visible_node,
    );

    let (recovered_ast, recovered_module, recovered_shells, _, _) =
        task248_two_parameter_frontend("theorem Broken: ;\n", 92);
    let recovered_shell = recovered_shells
        .declarations()
        .first()
        .expect("malformed theorem shell");
    let recovered_node = recovered_ast
        .node(recovered_shell.node_id())
        .expect("malformed theorem node");
    assert!(recovered_shell.recovered());
    assert_eq!(
        recovered_shell.visibility().state(),
        mizar_resolve::declarations::DeclarationShellVisibilityState::Unspecified
    );
    assert_task248_shell_validation_error(
        recovered_shell,
        recovered_shell.ordinal(),
        recovered_shell.kind(),
        &recovered_module,
        recovered_shell.node_id(),
        recovered_node,
    );

    let (_, _, empty_shells, _, _) = task248_two_parameter_frontend("", 93);
    let (_, empty_shell_sites) = task248_two_parameter_arena(&ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &empty_shells,
            &symbols,
            definition,
            &nodes,
            empty_shell_sites,
        ),
        Err(
            "two-parameter source binding context requires one top-level declaration shell"
                .to_owned()
        )
    );
    let doubled_source =
        format!("{TASK248_TWO_PARAMETER_SOURCE}\ndefinition\nend;\n");
    let (_, _, doubled_shells, _, _) = task248_two_parameter_frontend(&doubled_source, 94);
    let (_, doubled_shell_sites) = task248_two_parameter_arena(&ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &doubled_shells,
            &symbols,
            definition,
            &nodes,
            doubled_shell_sites,
        ),
        Err(
            "two-parameter source binding context requires one top-level declaration shell"
                .to_owned()
        )
    );
    let (_, _, export_shells, _, _) =
        task248_two_parameter_frontend("export Demo;\n", 95);
    assert_eq!(export_shells.exports().len(), 1);
    let (_, export_sites) = task248_two_parameter_arena(&ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &export_shells,
            &symbols,
            definition,
            &nodes,
            export_sites,
        ),
        Err("two-parameter source binding context definition item is not exact".to_owned())
    );

    for (ordinal, source) in [
        TASK248_TWO_PARAMETER_SOURCE.replace("let x be set;", "let z be set;"),
        TASK248_TWO_PARAMETER_SOURCE.replace("let y be set;", "let x be set;"),
        TASK248_TWO_PARAMETER_SOURCE.replace("let y be set;", "assume y = y;"),
        TASK248_TWO_PARAMETER_SOURCE.replace("assume x = x;", "let z be set;"),
        TASK248_TWO_PARAMETER_SOURCE.replace("let x be set;", "let x be Set;"),
        TASK248_TWO_PARAMETER_SOURCE.replace("  let y be set;\n", ""),
        TASK248_TWO_PARAMETER_SOURCE.replace(
            "  assume x = x;\n",
            "  let z be set;\n  assume x = x;\n",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_symbols, _) =
            task248_two_parameter_frontend(&source, 100 + ordinal);
        let Some(near_definition) = surface_nodes_with_kind(
            &near_ast,
            mizar_syntax::SurfaceNodeKind::DefinitionBlockItem,
        )
        .first()
        .map(|(id, _)| *id)
        else {
            continue;
        };
        let (near_nodes, near_sites) =
            task248_two_parameter_arena_for_source(near_ast.source_id);
        assert!(
            source_two_parameter_definition_context_projection(
                &near_ast,
                near_module,
                &near_shells,
                &near_symbols,
                near_definition,
                &near_nodes,
                near_sites,
            )
            .is_err(),
            "near source {ordinal} unexpectedly selected Profile B"
        );
    }

    for (mutation, expected) in [
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::RootMissing,
            "two-parameter source binding context root disappeared",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::RootRecovery,
            "two-parameter source binding context root is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::RootRange,
            "two-parameter source binding context root is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::CompilationItemListMissing,
            "two-parameter source binding context item list is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::CompilationItemListEmpty,
            "two-parameter source binding context requires one top-level source item",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::CompilationItemListDuplicated,
            "two-parameter source binding context requires one top-level source item",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionKind,
            "two-parameter source binding context definition item is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionRecovery,
            "two-parameter source binding context definition item is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionRange,
            "two-parameter source binding context definition item is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildMissing,
            "two-parameter source binding context requires two leading parameters",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildReordered,
            "two-parameter source binding context parameter x is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildDuplicated,
            "two-parameter source binding context parameter y is not exact",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildThird,
            "two-parameter source binding context contains an additional or non-leading parameter",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildNonLeading,
            "two-parameter source binding context contains an additional or non-leading parameter",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildNonDirectParameter,
            "source binding context requires one `let` token",
        ),
        (
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildNestedParameter,
            "two-parameter source binding context parameter x requires one segment",
        ),
    ] {
        assert_task248_authentication_mutation_error(
            &ast, &module, &shells, &symbols, definition, &nodes, mutation, expected,
        );
    }
    for index in 0..3 {
        assert_task248_authentication_mutation_error(
            &ast,
            &module,
            &shells,
            &symbols,
            definition,
            &nodes,
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionTokenText {
                index,
            },
            "two-parameter source binding context definition item is not exact",
        );
    }

    for (index, spelling) in ["x", "y"].into_iter().enumerate() {
        let parameter_error =
            format!("two-parameter source binding context parameter {spelling} is not exact");
        for mutation in [
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterNodeId { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterKind { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterRange { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterRecovery { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterLetText { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterLetRange { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSemicolonText {
                index,
            },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSemicolonRange {
                index,
            },
        ] {
            assert_task248_authentication_mutation_error(
                &ast,
                &module,
                &shells,
                &symbols,
                definition,
                &nodes,
                mutation,
                &parameter_error,
            );
        }

        let segment_error =
            format!("two-parameter source binding context segment {spelling} is not exact");
        for mutation in [
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentKind { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentRecovery { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentNameText { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentNameRange { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentBeText { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentBeRange { index },
        ] {
            assert_task248_authentication_mutation_error(
                &ast,
                &module,
                &shells,
                &symbols,
                definition,
                &nodes,
                mutation,
                &segment_error,
            );
        }
        assert_task248_authentication_mutation_error(
            &ast,
            &module,
            &shells,
            &symbols,
            definition,
            &nodes,
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentChildCardinality {
                index,
            },
            &format!(
                "two-parameter source binding context parameter {spelling} requires one written type"
            ),
        );
        for mutation in [
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSegmentMissing {
                index,
            },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSegmentDuplicated {
                index,
            },
        ] {
            assert_task248_authentication_mutation_error(
                &ast,
                &module,
                &shells,
                &symbols,
                definition,
                &nodes,
                mutation,
                &format!(
                    "two-parameter source binding context parameter {spelling} requires one segment"
                ),
            );
        }

        let type_shape_error =
            format!("two-parameter source binding context type {spelling} has the wrong shape");
        for mutation in [
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeKind { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeRange { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeChildCardinality {
                index,
            },
        ] {
            assert_task248_authentication_mutation_error(
                &ast,
                &module,
                &shells,
                &symbols,
                definition,
                &nodes,
                mutation,
                &type_shape_error,
            );
        }

        for mutation in [
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadKind { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadRecovery { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadRange { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeTokenText { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeTokenRange { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeTokenRecovery { index },
        ] {
            assert_task248_authentication_mutation_error(
                &ast,
                &module,
                &shells,
                &symbols,
                definition,
                &nodes,
                mutation,
                "two-parameter source binding context type is not bare set",
            );
        }
        assert_task248_authentication_mutation_error(
            &ast,
            &module,
            &shells,
            &symbols,
            definition,
            &nodes,
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadChildCardinality {
                index,
            },
            "two-parameter source binding context type head is not bare",
        );

        let extracted_error =
            format!("two-parameter source binding context type {spelling} is not builtin set");
        for mutation in [
            SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeRange {
                index,
            },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeSpelling {
                index,
            },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeHead { index },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeAttributes {
                index,
            },
        ] {
            assert_task248_authentication_mutation_error(
                &ast,
                &module,
                &shells,
                &symbols,
                definition,
                &nodes,
                mutation,
                &extracted_error,
            );
        }

        assert_task248_authentication_mutation_error(
            &ast,
            &module,
            &shells,
            &symbols,
            definition,
            &nodes,
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedSourceOrdinal {
                index,
            },
            &format!("source binding {index} has a stale ordinal"),
        );
        for mutation in [
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalSpelling {
                index,
            },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalRange {
                index,
            },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalScope {
                index,
            },
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalVisibleOrdinal {
                index,
            },
        ] {
            assert_task248_authentication_mutation_error(
                &ast,
                &module,
                &shells,
                &symbols,
                definition,
                &nodes,
                mutation,
                &format!("source binding {index} has stale local identity"),
            );
        }
    }
    assert_task248_authentication_mutation_error(
        &ast,
        &module,
        &shells,
        &symbols,
        definition,
        &nodes,
        SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedScope,
        "two-parameter source binding context scope is not [0]",
    );

    for node_index in 0..4 {
        let role = ["parameter 0", "parameter 1", "definition", "module"][node_index];
        for mutation in [
            Task248ArenaMutation::Anchor(node_index),
            Task248ArenaMutation::Context(node_index),
            Task248ArenaMutation::Recovered(node_index),
            Task248ArenaMutation::Degraded(node_index),
        ] {
            let (mutated_nodes, mutated_sites) =
                task248_two_parameter_mutated_arena(ast.source_id, mutation);
            assert_eq!(
                source_two_parameter_definition_context_projection(
                    &ast,
                    module.clone(),
                    &shells,
                    &symbols,
                    definition,
                    &mutated_nodes,
                    mutated_sites,
                ),
                Err(format!(
                    "two-parameter source binding context {role} site is not exact"
                )),
                "arena mutation {mutation:?} returned the wrong authentication stage"
            );
        }
    }

    let no_root_nodes = mizar_checker::typed_ast::TypedArena::try_new(
        None,
        nodes
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>(),
    )
    .expect("structurally valid Task 248 arena without a root");
    let (_, no_root_sites) = task248_two_parameter_arena(&ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            definition,
            &no_root_nodes,
            no_root_sites,
        ),
        Err("two-parameter source binding context arena has no root".to_owned())
    );

    let (non_root_nodes, non_root_sites) =
        task248_two_parameter_mutated_arena(ast.source_id, Task248ArenaMutation::NonModuleRoot);
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            definition,
            &non_root_nodes,
            non_root_sites,
        ),
        Err(
            "two-parameter source binding context module site is not the arena root".to_owned()
        )
    );

    let (_, mut role_site) = task248_two_parameter_arena(&ast, definition);
    role_site.module = mizar_checker::typed_ast::TypedSiteRef::Role {
        node: role_site.module.node(),
        role: mizar_checker::typed_ast::TypeRole::new("task248.not-root-node-site"),
    };
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            definition,
            &nodes,
            role_site,
        ),
        Err(
            "two-parameter source binding context module site is not the arena root".to_owned()
        )
    );

    let (_, mut missing_site) = task248_two_parameter_arena(&ast, definition);
    missing_site.parameters[1] =
        mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(99));
    assert_eq!(
        source_two_parameter_definition_context_projection(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            definition,
            &nodes,
            missing_site,
        ),
        Err(
            "two-parameter source binding context parameter 1 site does not resolve".to_owned()
        )
    );

    for (left, right, expected_role) in [
        (1, 2, "definition"),
        (1, 3, "definition"),
        (2, 3, "parameter 0"),
    ] {
        let (_, mut crossed_sites) = task248_two_parameter_arena(&ast, definition);
        task248_swap_context_sites(&mut crossed_sites, left, right);
        assert_eq!(
            source_two_parameter_definition_context_projection(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                definition,
                &nodes,
                crossed_sites,
            ),
            Err(format!(
                "two-parameter source binding context {expected_role} site is not exact"
            ))
        );
    }

    for (left, right) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)] {
        let (_, mut duplicate_sites) = task248_two_parameter_arena(&ast, definition);
        let duplicate = task248_context_site(&duplicate_sites, left);
        task248_set_context_site(&mut duplicate_sites, right, duplicate);
        assert_eq!(
            source_two_parameter_definition_context_projection(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                definition,
                &nodes,
                duplicate_sites,
            ),
            Err("two-parameter source binding context sites are not distinct".to_owned())
        );
    }

    assert_eq!(
        TASK248_TWO_PARAMETER_SOURCE.len(),
        TASK248_TWO_PARAMETER_EXCLUDED_VARIANT.len()
    );
    let (variant_ast, variant_module, variant_shells, variant_symbols, diagnostics) =
        task248_two_parameter_frontend(TASK248_TWO_PARAMETER_EXCLUDED_VARIANT, 200);
    assert_eq!(diagnostics, 0);
    let variant_definition = task248_two_parameter_definition_node(&variant_ast);
    let (variant_nodes, variant_sites) =
        task248_two_parameter_arena(&variant_ast, variant_definition);
    let variant = source_two_parameter_definition_context_projection(
        &variant_ast,
        variant_module,
        &variant_shells,
        &variant_symbols,
        variant_definition,
        &variant_nodes,
        variant_sites,
    )
    .expect("normal excluded descendants must not affect Task 248");
    assert_eq!(variant, baseline);
    assert_eq!(variant.handoff().debug_text(), TASK248_TWO_PARAMETER_DEBUG);
}

#[test]
fn task248_two_parameter_definition_installation_is_transactional_and_deterministic() {
    let (ast, module, shells, symbols, _) =
        task248_two_parameter_frontend(TASK248_TWO_PARAMETER_SOURCE, 30);
    let definition = task248_two_parameter_definition_node(&ast);
    let (nodes, sites) = task248_two_parameter_arena(&ast, definition);
    let first_projection = source_two_parameter_definition_context_projection(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        definition,
        &nodes,
        sites,
    )
    .expect("Profile B projection");
    let (_, replay_sites) = task248_two_parameter_arena(&ast, definition);
    let second_projection = source_two_parameter_definition_context_projection(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        definition,
        &nodes,
        replay_sites,
    )
    .expect("Profile B replay projection");

    let mut invalid_nodes = nodes
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    invalid_nodes[1].anchor =
        mizar_session::SourceAnchor::Range(task248_two_parameter_range(ast.source_id, 34, 35));
    let invalid_nodes = mizar_checker::typed_ast::TypedArena::try_new(
        nodes.root(),
        invalid_nodes,
    )
    .expect("structurally valid corrupted arena");
    assert!(matches!(
        task248_two_parameter_typed_ast(
            ast.source_id,
            module.clone(),
            invalid_nodes,
            first_projection.clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceContext)
    ));

    let first = task248_two_parameter_typed_ast(
        ast.source_id,
        module.clone(),
        nodes.clone(),
        first_projection,
    )
    .expect("valid Profile B typed installation");
    let second = task248_two_parameter_typed_ast(
        ast.source_id,
        module,
        nodes,
        second_projection,
    )
    .expect("deterministic Profile B typed replay");
    assert_eq!(first, second);
    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.debug_text().matches(TASK248_TWO_PARAMETER_DEBUG).count(), 1);
    let typed_prefix = concat!(
        "typed-ast-debug-v1\n",
        "module: task248::task248.two_parameter_profile\n",
        "root: node#3\n",
        "resolved_root: <none>\n",
    );
    assert!(
        first
            .debug_text()
            .starts_with(&format!("{typed_prefix}{TASK248_TWO_PARAMETER_DEBUG}"))
    );

    let first_resolved =
        assemble_empty_resolved_typed_ast(&first, Vec::new()).expect("Profile B final assembly");
    let second_resolved =
        assemble_empty_resolved_typed_ast(&second, Vec::new()).expect("Profile B final replay");
    assert_eq!(first_resolved, second_resolved);
    assert_eq!(first_resolved.debug_text(), second_resolved.debug_text());
    assert_eq!(
        first_resolved
            .debug_text()
            .matches(TASK248_TWO_PARAMETER_DEBUG)
            .count(),
        1
    );
    let final_prefix = concat!(
        "resolved-typed-ast-debug-v1\n",
        "module: \"task248\"::\"task248.two_parameter_profile\"\n",
        "root: resolved_node#3\n",
    );
    assert!(
        first_resolved
            .debug_text()
            .starts_with(&format!("{final_prefix}{TASK248_TWO_PARAMETER_DEBUG}")),
        "{}",
        first_resolved.debug_text()
    );
    assert_eq!(first.source_context(), first_resolved.source_context());
    assert_task248_two_parameter_downstream_empty(&first, &first_resolved);
}

fn task248_source_input_site(
    input: &mizar_checker::source_context::SourceBindingContextInput,
    index: usize,
) -> mizar_checker::typed_ast::TypedSiteRef {
    match index {
        0 => input.module_site.clone(),
        1 => input.items[0].site.clone(),
        2 | 3 => input.bindings[index - 2].site.clone(),
        _ => panic!("Task 248 source input site index {index} is out of range"),
    }
}

fn task248_set_source_input_site(
    input: &mut mizar_checker::source_context::SourceBindingContextInput,
    index: usize,
    site: mizar_checker::typed_ast::TypedSiteRef,
) {
    match index {
        0 => input.module_site = site,
        1 => input.items[0].site = site,
        2 | 3 => input.bindings[index - 2].site = site,
        _ => panic!("Task 248 source input site index {index} is out of range"),
    }
}

fn task248_context_site(
    sites: &SourceTwoParameterDefinitionContextSites,
    index: usize,
) -> mizar_checker::typed_ast::TypedSiteRef {
    match index {
        0 => sites.module.clone(),
        1 => sites.definition.clone(),
        2 | 3 => sites.parameters[index - 2].clone(),
        _ => panic!("Task 248 context site index {index} is out of range"),
    }
}

fn task248_set_context_site(
    sites: &mut SourceTwoParameterDefinitionContextSites,
    index: usize,
    site: mizar_checker::typed_ast::TypedSiteRef,
) {
    match index {
        0 => sites.module = site,
        1 => sites.definition = site,
        2 | 3 => sites.parameters[index - 2] = site,
        _ => panic!("Task 248 context site index {index} is out of range"),
    }
}

fn task248_swap_context_sites(
    sites: &mut SourceTwoParameterDefinitionContextSites,
    left: usize,
    right: usize,
) {
    let left_site = task248_context_site(sites, left);
    let right_site = task248_context_site(sites, right);
    task248_set_context_site(sites, left, right_site);
    task248_set_context_site(sites, right, left_site);
}

fn assert_task248_shell_validation_error(
    shell: &mizar_resolve::declarations::DeclarationShell,
    ordinal: usize,
    kind: mizar_resolve::declarations::DeclarationShellKind,
    module: &mizar_resolve::resolved_ast::ModuleId,
    node_id: mizar_syntax::SurfaceNodeId,
    node: &mizar_syntax::SurfaceNode,
) {
    assert_eq!(
        validate_source_context_shell_for_test(
            shell, ordinal, kind, module, node_id, node
        ),
        Err(format!(
            "source binding context shell {ordinal} is inconsistent"
        ))
    );
}

// Rationale: this test helper mirrors the frozen seven-argument seam and adds one expected error.
#[allow(clippy::too_many_arguments)]
fn assert_task248_authentication_mutation_error(
    ast: &mizar_syntax::SurfaceAst,
    module: &mizar_resolve::resolved_ast::ModuleId,
    shells: &mizar_resolve::declarations::DeclarationShellSet,
    symbols: &mizar_resolve::env::SymbolEnv,
    definition: mizar_syntax::SurfaceNodeId,
    nodes: &mizar_checker::typed_ast::TypedArena,
    mutation: SourceTwoParameterDefinitionContextAuthenticationMutation,
    expected: &str,
) {
    let (_, sites) = task248_two_parameter_arena(ast, definition);
    assert_eq!(
        source_two_parameter_definition_context_projection_with_authentication_mutation(
            ast,
            module.clone(),
            shells,
            symbols,
            definition,
            nodes,
            sites,
            mutation,
        ),
        Err(expected.to_owned()),
        "authentication mutation {mutation:?} returned the wrong failure"
    );
}

fn task248_two_parameter_frontend(
    source: &str,
    ordinal: usize,
) -> (
    mizar_syntax::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::declarations::DeclarationShellSet,
    mizar_resolve::env::SymbolEnv,
    usize,
) {
    let (ast, _, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(source, 248_000 + ordinal);
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task248"),
        mizar_session::ModulePath::new("task248.two_parameter_profile"),
    );
    let shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module).collect();
    let projections = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        mizar_resolve::env::NamespacePath::new(module.path().as_str()),
    )
    .extract();
    let symbols =
        mizar_resolve::symbols::SymbolCollector::new(ast.source_id, &module, &shells, &projections)
            .collect()
            .into_env();
    (ast, module, shells, symbols, diagnostics)
}

fn task248_two_parameter_definition_node(
    ast: &mizar_syntax::SurfaceAst,
) -> mizar_syntax::SurfaceNodeId {
    let definitions =
        surface_nodes_with_kind(ast, mizar_syntax::SurfaceNodeKind::DefinitionBlockItem);
    let [(definition, node)] = definitions.as_slice() else {
        panic!("expected exactly one definition block, got {definitions:?}");
    };
    assert_eq!(definition.index(), 67);
    assert_eq!(node.range, task248_two_parameter_range(ast.source_id, 0, 164));
    *definition
}

fn task248_two_parameter_arena(
    ast: &mizar_syntax::SurfaceAst,
    definition: mizar_syntax::SurfaceNodeId,
) -> (
    mizar_checker::typed_ast::TypedArena,
    SourceTwoParameterDefinitionContextSites,
) {
    let parameters =
        surface_nodes_with_kind(ast, mizar_syntax::SurfaceNodeKind::DefinitionParameter);
    let [(first_id, first), (second_id, second)] = parameters.as_slice() else {
        panic!("expected two definition parameters, got {parameters:?}");
    };
    let definition_node = ast.node(definition).expect("definition node");
    assert_eq!([first_id.index(), second_id.index()], [41, 45]);
    assert_eq!(
        [first.range, second.range],
        [
            task248_two_parameter_range(ast.source_id, 13, 26),
            task248_two_parameter_range(ast.source_id, 29, 42),
        ]
    );
    task248_two_parameter_arena_for_source_with_ranges(
        ast.source_id,
        definition_node.range,
        [
            sole_token_range(ast, first, "x"),
            sole_token_range(ast, second, "y"),
        ],
        Some(mizar_checker::typed_ast::TypedNodeId::new(3)),
    )
}

fn task248_two_parameter_arena_for_source(
    source_id: mizar_session::SourceId,
) -> (
    mizar_checker::typed_ast::TypedArena,
    SourceTwoParameterDefinitionContextSites,
) {
    task248_two_parameter_arena_for_source_with_ranges(
        source_id,
        task248_two_parameter_range(source_id, 0, 164),
        [
            task248_two_parameter_range(source_id, 17, 18),
            task248_two_parameter_range(source_id, 33, 34),
        ],
        Some(mizar_checker::typed_ast::TypedNodeId::new(3)),
    )
}

fn task248_two_parameter_arena_for_source_with_ranges(
    source_id: mizar_session::SourceId,
    definition_range: mizar_session::SourceRange,
    parameter_ranges: [mizar_session::SourceRange; 2],
    root: Option<mizar_checker::typed_ast::TypedNodeId>,
) -> (
    mizar_checker::typed_ast::TypedArena,
    SourceTwoParameterDefinitionContextSites,
) {
    let mut builder = mizar_checker::typed_ast::TypedArenaBuilder::new();
    for range in parameter_ranges {
        builder
            .push(
                mizar_checker::typed_ast::TypedNode::new(
                    "source.definition.parameter",
                    mizar_session::SourceAnchor::Range(range),
                )
                .with_typing(mizar_checker::typed_ast::TypingState::Unknown)
                .with_recovery(mizar_checker::typed_ast::NodeRecoveryState::Normal)
                .with_links(mizar_checker::typed_ast::TypedNodeLinks {
                    context: Some(mizar_checker::typed_ast::LocalTypeContextId::new(1)),
                    ..mizar_checker::typed_ast::TypedNodeLinks::default()
                }),
            )
            .expect("parameter node");
    }
    builder
        .push(
            mizar_checker::typed_ast::TypedNode::new(
                "source.definition",
                mizar_session::SourceAnchor::Range(definition_range),
            )
            .with_children(vec![
                mizar_checker::typed_ast::TypedNodeId::new(0),
                mizar_checker::typed_ast::TypedNodeId::new(1),
            ])
            .with_typing(mizar_checker::typed_ast::TypingState::Unknown)
            .with_recovery(mizar_checker::typed_ast::NodeRecoveryState::Normal)
            .with_links(mizar_checker::typed_ast::TypedNodeLinks {
                context: Some(mizar_checker::typed_ast::LocalTypeContextId::new(1)),
                ..mizar_checker::typed_ast::TypedNodeLinks::default()
            }),
        )
        .expect("definition node");
    builder
        .push(
            mizar_checker::typed_ast::TypedNode::new(
                "source.module",
                mizar_session::SourceAnchor::Range(task248_two_parameter_range(
                    source_id, 0, 164,
                )),
            )
            .with_children(vec![mizar_checker::typed_ast::TypedNodeId::new(2)])
            .with_typing(mizar_checker::typed_ast::TypingState::Unknown)
            .with_recovery(mizar_checker::typed_ast::NodeRecoveryState::Normal)
            .with_links(mizar_checker::typed_ast::TypedNodeLinks {
                context: Some(mizar_checker::typed_ast::LocalTypeContextId::new(0)),
                ..mizar_checker::typed_ast::TypedNodeLinks::default()
            }),
        )
        .expect("module node");
    let nodes = builder.finish(root).expect("Profile B typed arena");
    (
        nodes,
        SourceTwoParameterDefinitionContextSites {
            module: mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(3),
            ),
            definition: mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(2),
            ),
            parameters: [
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(0),
                ),
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(1),
                ),
            ],
        },
    )
}

#[derive(Debug, Clone, Copy)]
enum Task248ArenaMutation {
    Anchor(usize),
    Context(usize),
    Recovered(usize),
    Degraded(usize),
    NonModuleRoot,
}

fn task248_two_parameter_mutated_arena(
    source_id: mizar_session::SourceId,
    mutation: Task248ArenaMutation,
) -> (
    mizar_checker::typed_ast::TypedArena,
    SourceTwoParameterDefinitionContextSites,
) {
    let (baseline, sites) = task248_two_parameter_arena_for_source(source_id);
    let mut nodes = baseline
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let root = match mutation {
        Task248ArenaMutation::Anchor(index) => {
            nodes[index].anchor = mizar_session::SourceAnchor::Range(
                task248_two_parameter_range(source_id, 90 + index, 91 + index),
            );
            baseline.root()
        }
        Task248ArenaMutation::Context(index) => {
            nodes[index].links.context =
                Some(mizar_checker::typed_ast::LocalTypeContextId::new(9));
            baseline.root()
        }
        Task248ArenaMutation::Recovered(index) => {
            nodes[index].recovery = mizar_checker::typed_ast::NodeRecoveryState::Recovered;
            baseline.root()
        }
        Task248ArenaMutation::Degraded(index) => {
            nodes[index].recovery = mizar_checker::typed_ast::NodeRecoveryState::Degraded;
            baseline.root()
        }
        Task248ArenaMutation::NonModuleRoot => {
            Some(mizar_checker::typed_ast::TypedNodeId::new(2))
        }
    };
    (
        mizar_checker::typed_ast::TypedArena::try_new(root, nodes)
            .expect("structurally valid Task 248 arena mutation"),
        sites,
    )
}

fn task248_two_parameter_typed_ast(
    source_id: mizar_session::SourceId,
    module: mizar_resolve::resolved_ast::ModuleId,
    nodes: mizar_checker::typed_ast::TypedArena,
    projection: mizar_checker::source_context::SourceBindingContextProjection,
) -> Result<mizar_checker::typed_ast::TypedAst, mizar_checker::typed_ast::TypedAstError> {
    let source_context = projection.into_handoff();
    let contexts = source_context.local_contexts().clone();
    mizar_checker::typed_ast::TypedAst::try_new(mizar_checker::typed_ast::TypedAstParts {
        source_id,
        module_id: module,
        resolved_root: None,
        source_context: Some(source_context),
        source_type: None,
        source_attribute: None,
        nodes,
        contexts,
        types: mizar_checker::typed_ast::TypeTable::new(),
        facts: mizar_checker::typed_ast::TypeFactTable::new(),
        coercions: mizar_checker::typed_ast::CoercionTable::new(),
        initial_obligations: mizar_checker::typed_ast::InitialObligationTable::new(),
        diagnostics: mizar_checker::typed_ast::TypeDiagnosticTable::new(),
    })
}

fn assert_task248_two_parameter_downstream_empty(
    typed: &mizar_checker::typed_ast::TypedAst,
    resolved: &mizar_checker::resolved_typed_ast::ResolvedTypedAst,
) {
    assert!(typed.source_context().is_some());
    assert!(typed.source_type().is_none());
    assert!(typed.source_attribute().is_none());
    assert!(typed.source_evidence().is_none());
    assert!(typed.source_term().is_none());
    assert!(typed.source_application().is_none());
    assert!(typed.source_structure().is_none());
    assert!(typed.source_set_term().is_none());
    assert!(typed.source_atomic_formula().is_none());
    assert!(typed.source_composite_formula().is_none());
    assert!(typed.source_formula_composition().is_none());
    assert!(typed.source_condition_formula_composition().is_none());
    assert!(typed.source_predicate_chain_composition().is_none());
    assert!(typed.source_statement().is_none());
    assert!(typed.source_statement_references().is_none());
    assert!(typed.source_statement_witnesses().is_none());
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.diagnostics().is_empty());

    assert!(resolved.source_context().is_some());
    assert!(resolved.source_type().is_none());
    assert!(resolved.source_attribute().is_none());
    assert!(resolved.source_evidence().is_none());
    assert!(resolved.source_term().is_none());
    assert!(resolved.source_application().is_none());
    assert!(resolved.source_structure().is_none());
    assert!(resolved.source_set_term().is_none());
    assert!(resolved.source_atomic_formula().is_none());
    assert!(resolved.source_composite_formula().is_none());
    assert!(resolved.source_formula_composition().is_none());
    assert!(resolved.source_condition_formula_composition().is_none());
    assert!(resolved.source_predicate_chain_composition().is_none());
    assert!(resolved.source_statement().is_none());
    assert!(resolved.source_statement_references().is_none());
    assert!(resolved.source_statement_witnesses().is_none());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.collection_candidates().is_empty());
    assert!(resolved.expanded_candidates().is_empty());
    assert!(resolved.template_expansions().is_empty());
    assert!(resolved.viable_candidates().is_empty());
    assert!(resolved.viability_decisions().is_empty());
    assert!(resolved.specificity_graphs().is_empty());
    assert!(resolved.resolved_overloads().is_empty());
    assert!(resolved.inserted_coercions().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
}

fn task248_two_parameter_range(
    source_id: mizar_session::SourceId,
    start: usize,
    end: usize,
) -> mizar_session::SourceRange {
    mizar_session::SourceRange {
        source_id,
        start,
        end,
    }
}

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
    let first_core_context = mizar_core::elaborator::prepare_core_context(
        mizar_core::elaborator::CoreContextInput::new(
            mizar_core::elaborator::ResolvedTypedAstSummary::new(
                handoff.source_id(),
                handoff.module_id().clone(),
            ),
        ),
    )
    .expect("Task 248 first empty Core context");
    let replay_source_context = second
        .resolved
        .source_context()
        .expect("Task 248 replay source context");
    let second_core_context = mizar_core::elaborator::prepare_core_context(
        mizar_core::elaborator::CoreContextInput::new(
            mizar_core::elaborator::ResolvedTypedAstSummary::new(
                replay_source_context.source_id(),
                replay_source_context.module_id().clone(),
            ),
        ),
    )
    .expect("Task 248 replay empty Core context");
    let first_core = mizar_core::elaborator::SourceBindingCoreContextProducer::build(
        first_core_context,
        handoff.binding_env().clone(),
    )
    .expect("Task 248 first source-binding Core handoff");
    let second_binding_env = replay_source_context.binding_env().clone();
    let second_core = mizar_core::elaborator::SourceBindingCoreContextProducer::build(
        second_core_context,
        second_binding_env,
    )
    .expect("Task 248 replay source-binding Core handoff");
    assert!(first_core == second_core);
    assert_eq!(first_core.variables().len(), 2);
    let core_rows = first_core.variables().iter().collect::<Vec<_>>();
    assert_eq!(core_rows.len(), 2);
    assert_eq!(
        core_rows[0].0,
        mizar_checker::binding_env::BindingId::new(0)
    );
    assert_eq!(
        core_rows[1].0,
        mizar_checker::binding_env::BindingId::new(1)
    );
    assert_eq!(
        core_rows[0].1.core_var(),
        mizar_core::core_ir::CoreVarId::new(0)
    );
    assert_eq!(
        core_rows[1].1.core_var(),
        mizar_core::core_ir::CoreVarId::new(1)
    );
    assert!(first_core.context().item_registry().items().is_empty());
    assert!(first_core.context().diagnostics().is_empty());
    assert!(first_core.context().worklist().entries().is_empty());
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
    assert_eq!(core_rows[0].0, reserve.binding);
    assert_eq!(core_rows[0].1.binding(), reserve.binding);
    assert_eq!(core_rows[1].0, parameter.binding);
    assert_eq!(core_rows[1].1.binding(), parameter.binding);
    for (binding, row, role, declaration_range) in [
        (
            reserve.binding,
            core_rows[0].1,
            "reserved-variable",
            reserve_name_range,
        ),
        (
            parameter.binding,
            core_rows[1].1,
            "definition-parameter",
            parameter_name_range,
        ),
    ] {
        let var = row.core_var();
        assert_eq!(
            first_core.context().binder_context().variable_classes.get(&var),
            Some(&mizar_core::binder_normalization::NormalizedVarClass::Free)
        );
        assert_eq!(
            first_core.context().binder_context().variable_sorts.get(&var),
            Some(&mizar_core::binder_normalization::NormalizedVarSort::Term)
        );
        assert_eq!(
            first_core.context().binder_context().variable_roles.get(&var),
            Some(&mizar_core::core_ir::CoreVarRole::new(role))
        );
        assert!(matches!(
            first_core.context().binder_type_facts().get(&var),
            Some(facts) if facts.is_empty()
        ));
        let record = first_core
            .context()
            .binder_sources()
            .get(var)
            .expect("Task 248 Core binder source");
        assert_eq!(
            record.source.anchor,
            mizar_core::core_ir::CoreSourceAnchor::SourceRange(declaration_range)
        );
        let expected_key = format!(
            "source-binding-core-variable-v1.binding.{}",
            binding.index()
        );
        assert_eq!(record.source.provenance.len(), 1);
        assert_eq!(
            record.source.provenance[0].phase,
            mizar_core::core_ir::CoreProvenancePhase::Checker
        );
        assert_eq!(record.source.provenance[0].key.as_str(), expected_key);
        assert_eq!(record.provenance.as_slice(), record.source.provenance);
    }

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
    let source_type = first
        .typed_ast
        .source_type()
        .expect("Task 248 route should co-install the Task 249 source-type handoff");
    assert_eq!(source_type.applications().len(), 2);
    assert_eq!(source_type.expressions().len(), 2);
    assert!(source_type.arguments().is_empty());
    for (index, (_, application)) in source_type.applications().iter().enumerate() {
        assert_eq!(application.binding().index(), index);
        assert_eq!(application.source_ordinal(), index);
        let expression = source_type
            .expressions()
            .get(application.root())
            .expect("Task 248 source-type root");
        assert_eq!(
            expression.form(),
            mizar_checker::source_type::SourceTypeApplicationForm::Bare
        );
        assert_eq!(
            expression.head(),
            &mizar_checker::source_type::SourceTypeHead::BuiltinSet
        );
    }
    assert_eq!(first.resolved.source_type(), Some(source_type));
    let source_type_input = source_type_input_from_handoff(source_type);
    for mutation in [
        Task249DefinitionBindingMutation::Kind,
        Task249DefinitionBindingMutation::Status,
        Task249DefinitionBindingMutation::Layer,
        Task249DefinitionBindingMutation::LexicalScope,
    ] {
        let corrupted =
            task249_definition_env_with_mutation(handoff.binding_env(), mutation);
        assert!(
            matches!(
                mizar_checker::source_type::SourceTypeProducer::build(
                    source_type_input.clone(),
                    &corrupted,
                    &symbols,
                    first.typed_ast.nodes(),
                ),
                Err(mizar_checker::source_type::SourceTypeError::InvalidBinding { .. })
            ),
            "Task 249 accepted definition binding mutation {mutation:?}"
        );
    }
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
        source_type: None,
        source_attribute: None,
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

#[derive(Clone, Copy, Debug)]
enum Task249DefinitionBindingMutation {
    Kind,
    Status,
    Layer,
    LexicalScope,
}

fn task249_definition_env_with_mutation(
    original: &mizar_checker::binding_env::BindingEnv,
    mutation: Task249DefinitionBindingMutation,
) -> mizar_checker::binding_env::BindingEnv {
    let mut contexts = mizar_checker::binding_env::BindingContextTable::new();
    for (_, context) in original.contexts().iter() {
        let is_declaration = matches!(
            context.owner,
            mizar_checker::binding_env::BindingContextOwner::DeclarationShell(_)
        );
        let mut draft = mizar_checker::binding_env::BindingContextDraft {
            owner: context.owner.clone(),
            parent: context.parent,
            layer: context.layer,
            lexical_scope: context.lexical_scope.clone(),
            bindings: context.bindings.clone(),
            visible_bindings: context.visible_bindings.clone(),
            recovery: context.recovery,
        };
        if is_declaration {
            match mutation {
                Task249DefinitionBindingMutation::Layer => {
                    draft.layer = mizar_checker::binding_env::BindingContextLayer::Proof;
                }
                Task249DefinitionBindingMutation::LexicalScope => {
                    draft.lexical_scope =
                        Some(mizar_resolve::names::LocalTermScope::new(vec![99]));
                }
                _ => {}
            }
        }
        contexts.insert(draft);
    }

    let mut bindings = mizar_checker::binding_env::BindingTable::new();
    for (_, binding) in original.bindings().iter() {
        let is_definition =
            binding.kind == mizar_checker::binding_env::BindingKind::DefinitionParameter;
        let mut draft = mizar_checker::binding_env::BindingDraft {
            spelling: binding.spelling.clone(),
            kind: binding.kind,
            identity: binding.identity.clone(),
            owner_context: binding.owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.visible_after_ordinal,
            type_site: binding.type_site.clone(),
            status: binding.status,
            captured: binding.captured.clone(),
            diagnostics: binding.diagnostics.clone(),
            recovery: binding.recovery,
        };
        if is_definition {
            match mutation {
                Task249DefinitionBindingMutation::Kind => {
                    draft.kind = mizar_checker::binding_env::BindingKind::QuantifierBinder;
                }
                Task249DefinitionBindingMutation::Status => {
                    draft.status = mizar_checker::binding_env::BindingStatus::Reserved;
                }
                _ => {}
            }
        }
        bindings.insert(draft);
    }

    mizar_checker::binding_env::BindingEnv::try_new(
        mizar_checker::binding_env::BindingEnvParts {
            source_id: original.source_id(),
            module_id: original.module_id().clone(),
            contexts,
            bindings,
            diagnostics: original.diagnostics().clone(),
        },
    )
    .unwrap_or_else(|error| {
        panic!(
            "Task 249 definition corruption {mutation:?} should remain valid at the generic env boundary: {error:?}"
        )
    })
}
