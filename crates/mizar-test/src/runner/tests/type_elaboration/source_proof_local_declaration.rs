use super::type_elaboration::{
    SOURCE_STATEMENT_B3M1_TEXT as TASK269B_SOURCE_TEXT,
    SOURCE_STATEMENT_B3N_TEXT as TASK269A_SOURCE_TEXT, SourceProofLocalDeclarationRouteMutation,
    SourceProofLocalDeclarationRouteOutput, source_proof_local_declaration_output,
    source_proof_local_declaration_output_with_mutation,
    source_statement_output_with_source as task269a_lower_output,
    source_statement_transport_detail_keys as task269a_legacy_detail_keys,
};

#[test]
fn task269a_exact_frontend_binding_transaction_and_debug_are_stable() {
    assert_eq!(TASK269A_SOURCE_TEXT.len(), 107);
    assert_eq!(
        sha256_text(TASK269A_SOURCE_TEXT),
        "a57022c4b75991dd4308943477e03819f5bfe2c0d23ea1030730256252d7d329"
    );
    assert!(TASK269A_SOURCE_TEXT.ends_with('\n'));
    assert!(!TASK269A_SOURCE_TEXT.ends_with("\n\n"));
    let (ast, module, _shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK269A_SOURCE_TEXT, 269_000);
    assert_eq!(diagnostics, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (51, Some(50))
    );

    let output =
        source_proof_local_declaration_output(&ast, module.clone(), &symbols, TASK269A_SOURCE_TEXT)
            .expect("Task269A exact selector")
            .expect("Task269A exact route");
    let handoff = output
        .typed_ast
        .source_proof_local_declaration()
        .expect("Task269A typed owner");
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task269A lower statement");
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("Task269A lower witnesses");
    let primary = output
        .typed_ast
        .source_term()
        .expect("Task269A lower primary terms");
    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(
        handoff.base_binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.witness_fingerprint(), witnesses.debug_text());
    assert_eq!(handoff.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        handoff.final_binding_fingerprint(),
        handoff.binding_env().debug_text()
    );

    assert_eq!(handoff.declarations().len(), 1);
    let declaration = handoff
        .declarations()
        .get(mizar_checker::source_proof_local_declaration::SourceProofLocalDeclarationId::new(0))
        .expect("Task269A declaration row");
    assert_eq!(declaration.witness().index(), 0);
    assert_eq!(declaration.name().index(), 0);
    assert_eq!(
        declaration.rhs(),
        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
            mizar_checker::source_term::SourcePrimaryTermId::new(2)
        )
    );
    assert_eq!(declaration.binding().index(), 1);
    assert_eq!(declaration.binding_context().index(), 1);
    assert_eq!(declaration.source_ordinal(), 1);
    assert_eq!(declaration.visible_after_ordinal(), 1);
    assert_eq!(
        declaration.kind(),
        mizar_checker::source_proof_local_declaration::SourceProofLocalDeclarationKind::NamedWitness
    );
    assert_eq!(
        declaration.recovery(),
        mizar_checker::source_proof_local_declaration::SourceProofLocalDeclarationRecovery::Normal
    );

    let bindings = handoff.binding_env();
    assert_eq!(
        (
            bindings.contexts().len(),
            bindings.bindings().len(),
            bindings.diagnostics().len()
        ),
        (2, 2, 0)
    );
    assert_eq!(
        bindings
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(0)),
        statement
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(0))
    );
    let binding = bindings
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(1))
        .expect("Task269A checker binding");
    assert_eq!(binding.spelling, "y");
    assert_eq!(
        binding.kind,
        mizar_checker::binding_env::BindingKind::LocalAbbreviation
    );
    assert_eq!(binding.owner_context.index(), 1);
    assert_eq!(
        (
            binding.declaration_range.start,
            binding.declaration_range.end
        ),
        (81, 82)
    );
    assert_eq!(binding.visible_after_ordinal, 1);
    assert_eq!(
        binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Missing
    );
    assert_eq!(
        binding.status,
        mizar_checker::binding_env::BindingStatus::Active
    );
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(
        binding.recovery,
        mizar_checker::binding_env::BindingRecoveryState::Normal
    );
    assert!(matches!(
        &binding.identity,
        mizar_checker::binding_env::BinderIdentity::ResolverLocal {
            scope,
            ordinal: 1,
            declaration_range,
        } if scope.path() == [0]
            && declaration_range.start == 81
            && declaration_range.end == 82
    ));
    let context = bindings
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("Task269A proof context");
    assert_eq!(
        context.bindings,
        vec![mizar_checker::binding_env::BindingId::new(1)]
    );
    assert_eq!(
        context.visible_bindings,
        vec![
            mizar_checker::binding_env::BindingId::new(0),
            mizar_checker::binding_env::BindingId::new(1),
        ]
    );
    let forward = bindings
        .lookup(&mizar_checker::binding_env::BindingLookupSite::new(
            "y",
            mizar_checker::binding_env::BindingContextId::new(1),
            Some(mizar_resolve::names::LocalTermScope::new(vec![0])),
            1,
        ))
        .expect("Task269A definition-site lookup");
    assert!(matches!(
        forward,
        mizar_checker::binding_env::BindingLookupResult::ForwardReference {
            candidates,
            ..
        } if candidates == [mizar_checker::binding_env::BindingId::new(1)]
    ));
    assert_eq!(
        bindings
            .lookup(&mizar_checker::binding_env::BindingLookupSite::new(
                "y",
                mizar_checker::binding_env::BindingContextId::new(1),
                Some(mizar_resolve::names::LocalTermScope::new(vec![0])),
                2,
            ))
            .expect("Task269A later lookup"),
        mizar_checker::binding_env::BindingLookupResult::Local(
            mizar_checker::binding_env::BindingId::new(1)
        )
    );

    let expected_debug = format!(
        concat!(
            "source-proof-local-declaration-debug-v1\n",
            "module: {}::{}\n",
            "base-binding-fingerprint: {:?}\n",
            "statement-fingerprint: {:?}\n",
            "witness-fingerprint: {:?}\n",
            "primary-term-fingerprint: {:?}\n",
            "declaration#0 kind=named-witness witness=0 name=0 rhs=primary#2 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\n",
            "final-binding-fingerprint: {:?}\n",
        ),
        module.package().as_str(),
        module.path().as_str(),
        handoff.base_binding_fingerprint(),
        handoff.statement_fingerprint(),
        handoff.witness_fingerprint(),
        handoff.primary_term_fingerprint(),
        handoff.final_binding_fingerprint(),
    );
    assert_eq!(handoff.debug_text(), expected_debug);
    assert_eq!(
        output
            .typed_ast
            .debug_text()
            .matches("source-proof-local-declaration-debug-v1")
            .count(),
        1
    );
    assert_task269b_exact_frontend_binding_transaction_and_debug();
}

fn assert_task269b_exact_frontend_binding_transaction_and_debug() {
    assert_eq!(TASK269B_SOURCE_TEXT.len(), 113);
    assert_eq!(
        sha256_text(TASK269B_SOURCE_TEXT),
        "412a6a7f8fddebd67418f3482855ea89a1e7da922b42ebb93463971d8e49c186"
    );
    assert!(TASK269B_SOURCE_TEXT.ends_with('\n'));
    assert!(!TASK269B_SOURCE_TEXT.ends_with("\n\n"));
    let (ast, module, _shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK269B_SOURCE_TEXT, 269_001);
    assert_eq!(diagnostics, 0);
    assert_eq!((ast.nodes().len(), ast.root().map(|id| id.index())), (56, Some(55)));

    let output =
        source_proof_local_declaration_output(&ast, module.clone(), &symbols, TASK269B_SOURCE_TEXT)
            .expect("Task269B exact selector")
            .expect("Task269B exact route");
    let handoff = output
        .typed_ast
        .source_proof_local_declaration()
        .expect("Task269B typed owner");
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task269B lower statement");
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("Task269B lower witnesses");
    let primary = output
        .typed_ast
        .source_term()
        .expect("Task269B lower primary terms");
    assert_eq!(handoff.base_binding_fingerprint(), statement.binding_env().debug_text());
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.witness_fingerprint(), witnesses.debug_text());
    assert_eq!(handoff.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        handoff.final_binding_fingerprint(),
        handoff.binding_env().debug_text()
    );
    assert_eq!((witnesses.witnesses().len(), witnesses.names().len()), (2, 1));
    let named = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
        .expect("Task269B named witness");
    let unnamed = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(1))
        .expect("Task269B unnamed witness");
    assert_eq!(named.name().map(|id| id.index()), Some(0));
    assert_eq!(named.term(), mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(2)));
    assert_eq!((named.source_ordinal(), named.ordinal()), (1, 0));
    assert_eq!(unnamed.name(), None);
    assert_eq!(unnamed.term(), mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(3)));
    assert_eq!((unnamed.source_ordinal(), unnamed.ordinal()), (1, 1));

    assert_eq!(handoff.declarations().len(), 1);
    assert_eq!(
        (
            handoff.binding_env().contexts().len(),
            handoff.binding_env().bindings().len(),
            handoff.binding_env().diagnostics().len(),
        ),
        (2, 2, 0),
    );
    let proof_context = handoff
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("Task269B proof context");
    assert_eq!(
        proof_context
            .bindings
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(
        proof_context
            .visible_bindings
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(
        handoff
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(2))
            .is_none(),
        "Task269B unnamed witness must not allocate a checker binding"
    );
    let declaration = handoff
        .declarations()
        .get(mizar_checker::source_proof_local_declaration::SourceProofLocalDeclarationId::new(0))
        .expect("Task269B declaration");
    assert_eq!(declaration.witness().index(), 0);
    assert_eq!(declaration.name().index(), 0);
    assert_eq!(declaration.rhs(), mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(2)));
    assert_eq!(declaration.binding().index(), 1);
    let binding = handoff
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(1))
        .expect("Task269B checker binding");
    assert_eq!(binding.spelling, "y");
    assert_eq!(binding.kind, mizar_checker::binding_env::BindingKind::LocalAbbreviation);
    assert_eq!((binding.declaration_range.start, binding.declaration_range.end), (84, 85));
    assert_eq!(binding.visible_after_ordinal, 1);
    assert!(binding.captured.identities().is_empty());
    assert!(matches!(
        &binding.identity,
        mizar_checker::binding_env::BinderIdentity::ResolverLocal {
            scope,
            ordinal: 1,
            declaration_range,
        } if scope.path() == [0]
            && declaration_range.start == 84
            && declaration_range.end == 85
    ));
    assert!(matches!(
        handoff.binding_env().lookup(&mizar_checker::binding_env::BindingLookupSite::new(
            "y",
            mizar_checker::binding_env::BindingContextId::new(1),
            Some(mizar_resolve::names::LocalTermScope::new(vec![0])),
            1,
        )),
        Ok(mizar_checker::binding_env::BindingLookupResult::ForwardReference {
            candidates,
            ..
        }) if candidates == [mizar_checker::binding_env::BindingId::new(1)]
    ));
    assert_eq!(
        handoff.binding_env().lookup(&mizar_checker::binding_env::BindingLookupSite::new(
            "y",
            mizar_checker::binding_env::BindingContextId::new(1),
            Some(mizar_resolve::names::LocalTermScope::new(vec![0])),
            2,
        )),
        Ok(mizar_checker::binding_env::BindingLookupResult::Local(
            mizar_checker::binding_env::BindingId::new(1)
        ))
    );
    assert_eq!(
        handoff.debug_text(),
        format!(
            concat!(
                "source-proof-local-declaration-debug-v1\n",
                "module: {}::{}\n",
                "base-binding-fingerprint: {:?}\n",
                "statement-fingerprint: {:?}\n",
                "witness-fingerprint: {:?}\n",
                "primary-term-fingerprint: {:?}\n",
                "declaration#0 kind=named-witness witness=0 name=0 rhs=primary#2 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\n",
                "final-binding-fingerprint: {:?}\n",
            ),
            module.package().as_str(),
            module.path().as_str(),
            handoff.base_binding_fingerprint(),
            handoff.statement_fingerprint(),
            handoff.witness_fingerprint(),
            handoff.primary_term_fingerprint(),
            handoff.final_binding_fingerprint(),
        )
    );
}

#[test]
fn task269a_resolver_local_row_and_lower_mutations_fail_at_the_owner() {
    for (profile, source_text, source_ordinal) in [
        ("Task269A", TASK269A_SOURCE_TEXT, 269_100),
        ("Task269B", TASK269B_SOURCE_TEXT, 269_101),
    ] {
        let (ast, module, _shells, symbols) =
            task253_ast_from_source_text(source_text, source_ordinal);
        for mutation in [
            SourceProofLocalDeclarationRouteMutation::WrongLocalSpelling,
            SourceProofLocalDeclarationRouteMutation::WrongLocalScope,
            SourceProofLocalDeclarationRouteMutation::WrongLocalRange,
            SourceProofLocalDeclarationRouteMutation::WrongLocalVisibleAfter,
            SourceProofLocalDeclarationRouteMutation::WrongWitness,
            SourceProofLocalDeclarationRouteMutation::WrongName,
            SourceProofLocalDeclarationRouteMutation::WrongRhs,
            SourceProofLocalDeclarationRouteMutation::WrongBindingContext,
            SourceProofLocalDeclarationRouteMutation::WrongSourceOrdinal,
            SourceProofLocalDeclarationRouteMutation::WrongLowerStatement,
            SourceProofLocalDeclarationRouteMutation::WrongLowerWitness,
            SourceProofLocalDeclarationRouteMutation::WrongLowerPrimary,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaCardinality,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaRoot,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaKind,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaAnchor,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaChildren,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaResolvedNode,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaRecovery,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaTyping,
            SourceProofLocalDeclarationRouteMutation::WrongLowerArenaLinks,
        ] {
            assert!(
                source_proof_local_declaration_output_with_mutation(
                    &ast,
                    module.clone(),
                    &symbols,
                    source_text,
                    mutation,
                )
                .unwrap_or_else(|| panic!("{profile} exact mutation selector"))
                .is_err(),
                "{profile} mutation {mutation:?} unexpectedly succeeded"
            );
        }
    }
}

#[test]
fn task269a_near_misses_and_public_route_remain_isolated() {
    let near_misses = [
        TASK269A_SOURCE_TEXT.trim_end_matches('\n').to_owned(),
        format!("{TASK269A_SOURCE_TEXT}\n"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "take x;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "take y = x, x;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "take z = x;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "take y = {x};"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "let y be set;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "set y = x;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "given y being set;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "consider y being set;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "reconsider y = x as set;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "deffunc y() = x;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "defpred y[] means x = x;"),
        TASK269A_SOURCE_TEXT.replace("take y = x;", "take y = imported_x;"),
    ];
    for (ordinal, source) in near_misses.into_iter().enumerate() {
        let (ast, module, _shells, symbols) =
            task253_ast_from_source_text(&source, 269_200 + ordinal);
        assert!(
            source_proof_local_declaration_output(&ast, module, &symbols, &source).is_none(),
            "Task269A selected near miss {ordinal}"
        );
    }

    let b3m1_near_misses = [
        TASK269B_SOURCE_TEXT.trim_end_matches('\n').to_owned(),
        format!("{TASK269B_SOURCE_TEXT}\n"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "take y = x;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "take x, y = x;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "take y = x, z = x;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "take z = x, x;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "take y = {x}, x;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "let y be set;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "set y = x;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "given y being set;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "consider y being set;"),
        TASK269B_SOURCE_TEXT.replace("take y = x, x;", "reconsider y = x as set;"),
    ];
    for (ordinal, source) in b3m1_near_misses.into_iter().enumerate() {
        let (ast, module, _shells, symbols) =
            task253_ast_from_source_text(&source, 269_220 + ordinal);
        assert!(
            source_proof_local_declaration_output(&ast, module, &symbols, &source).is_none(),
            "Task269B selected near miss {ordinal}"
        );
    }

    let (ast, module, _shells, symbols) =
        task253_ast_from_source_text(TASK269A_SOURCE_TEXT, 269_250);
    let recovered_ast = task269a_ast_with_recovered_name_token(&ast, 81, 82);
    assert!(
        source_proof_local_declaration_output(
            &recovered_ast,
            module.clone(),
            &symbols,
            TASK269A_SOURCE_TEXT,
        )
        .is_none(),
        "Task269A selected a recovered name token"
    );
    let imported_y = task269a_symbols_with_visible_imported_y(&symbols, ast.source_id);
    assert!(
        source_proof_local_declaration_output(
            &ast,
            module,
            &imported_y,
            TASK269A_SOURCE_TEXT,
        )
        .expect("Task269A exact selector with imported y")
        .is_err(),
        "Task269A accepted a visible imported y"
    );

    let (b3m1_ast, b3m1_module, _shells, b3m1_symbols) =
        task253_ast_from_source_text(TASK269B_SOURCE_TEXT, 269_251);
    let b3m1_recovered = task269a_ast_with_recovered_name_token(&b3m1_ast, 84, 85);
    assert!(
        source_proof_local_declaration_output(
            &b3m1_recovered,
            b3m1_module.clone(),
            &b3m1_symbols,
            TASK269B_SOURCE_TEXT,
        )
        .is_none(),
        "Task269B selected a recovered name token"
    );
    let b3m1_imported_y =
        task269a_symbols_with_visible_imported_y(&b3m1_symbols, b3m1_ast.source_id);
    assert!(
        source_proof_local_declaration_output(
            &b3m1_ast,
            b3m1_module,
            &b3m1_imported_y,
            TASK269B_SOURCE_TEXT,
        )
        .expect("Task269B exact selector with imported y")
        .is_err(),
        "Task269B accepted a visible imported y"
    );

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("mizar-test below workspace");
    let broad = std::fs::read_to_string(
        workspace_root
            .join("tests/miz/fail/types/fail_type_elaboration_proof_local_declaration_gap_001.miz"),
    )
    .expect("Task269 broad gap fixture");
    let (broad_ast, broad_module, _broad_shells, broad_symbols) =
        task253_ast_from_source_text(&broad, 269_299);
    assert!(
        source_proof_local_declaration_output(&broad_ast, broad_module, &broad_symbols, &broad,)
            .is_none()
    );

    let (ast, module, _shells, symbols) =
        task253_ast_from_source_text(TASK269A_SOURCE_TEXT, 269_300);
    let legacy = task269a_lower_output(&ast, module.clone(), &symbols, TASK269A_SOURCE_TEXT)
        .expect("Task258B3N selector")
        .expect("Task258B3N route");
    assert!(legacy.typed_ast.source_proof_local_declaration().is_none());
    assert_eq!(
        task269a_legacy_detail_keys(&ast, module, &symbols, TASK269A_SOURCE_TEXT,),
        Some(Vec::new())
    );

    let (ast, module, _shells, symbols) =
        task253_ast_from_source_text(TASK269B_SOURCE_TEXT, 269_301);
    let legacy = task269a_lower_output(&ast, module.clone(), &symbols, TASK269B_SOURCE_TEXT)
        .expect("Task258B3M1 selector")
        .expect("Task258B3M1 route");
    assert!(legacy.typed_ast.source_proof_local_declaration().is_none());
    assert_eq!(
        task269a_legacy_detail_keys(&ast, module, &symbols, TASK269B_SOURCE_TEXT,),
        Some(Vec::new())
    );
}

fn task269a_ast_with_recovered_name_token(
    ast: &mizar_syntax::SurfaceAst,
    start: usize,
    end: usize,
) -> mizar_syntax::SurfaceAst {
    let mut builder = mizar_syntax::SurfaceAstBuilder::new(ast.source_id);
    let mut rebuilt = Vec::<mizar_syntax::SurfaceBuilderNodeId>::with_capacity(ast.nodes().len());
    let mut recovered = false;
    for node in ast.nodes() {
        let children = node
            .children
            .iter()
            .map(|child| rebuilt[child.index()])
            .collect::<Vec<_>>();
        let id = match &node.kind {
            mizar_syntax::SurfaceNodeKind::Token(token) => {
                if !recovered
                    && token.text.as_ref() == "y"
                    && node.range.start == start
                    && node.range.end == end
                {
                    recovered = true;
                    builder.add_recovered_token(token.kind, token.text.clone(), node.range)
                } else {
                    builder.add_token(token.kind, token.text.clone(), node.range)
                }
            }
            structural => builder.add_node(structural.clone(), node.range, children),
        };
        rebuilt.push(id);
    }
    assert!(recovered, "Task269A name token must be recoverable");
    builder.finish(
        ast.root().map(|root| rebuilt[root.index()]),
        ast.expression_root()
            .map(|expression_root| rebuilt[expression_root.index()]),
    )
}

fn task269a_symbols_with_visible_imported_y(
    symbols: &SymbolEnv,
    source_id: mizar_session::SourceId,
) -> SymbolEnv {
    let module = symbols.module_id().clone();
    let contribution = symbols
        .contributions()
        .iter()
        .next()
        .expect("Task269A local contribution")
        .id();
    let imported_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("dep"),
        mizar_session::ModulePath::new("imported.fixture"),
    );
    let symbol = mizar_resolve::resolved_ast::SymbolId::new(
        imported_module.clone(),
        mizar_resolve::resolved_ast::LocalSymbolId::new("Imported/y/0"),
        mizar_resolve::resolved_ast::FullyQualifiedName::new("imported.fixture::y/0"),
    );
    let mut symbol_index = symbols.symbols().clone();
    symbol_index.insert(
        mizar_resolve::env::SymbolEntry::new(
            symbol.clone(),
            mizar_resolve::env::SymbolKind::Functor,
            mizar_resolve::env::NamespacePath::new(module.path().as_str()),
            "y",
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                source_id,
                imported_module,
                mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                    source_id,
                    start: 81,
                    end: 82,
                }),
                vec![269, 1],
            ),
            contribution,
        )
        .with_visibility(mizar_resolve::env::Visibility::Public)
        .with_export_status(mizar_resolve::env::ExportStatus::Exported),
    );
    let mut contributions = symbols.contributions().clone();
    contributions.add_symbol(contribution, symbol);
    mizar_resolve::env::SymbolEnv::new(
        module,
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbol_index,
            labels: symbols.labels().clone(),
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions,
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

#[test]
fn task269a_typed_and_final_replay_preserve_empty_semantics() {
    let (ast, module, _shells, symbols) =
        task253_ast_from_source_text(TASK269A_SOURCE_TEXT, 269_400);
    let SourceProofLocalDeclarationRouteOutput {
        typed_ast,
        resolved,
    } = source_proof_local_declaration_output(&ast, module, &symbols, TASK269A_SOURCE_TEXT)
        .expect("Task269A selector")
        .expect("Task269A route");

    assert_eq!(
        typed_ast.source_proof_local_declaration(),
        resolved.source_proof_local_declaration()
    );
    assert_eq!(
        (
            typed_ast.nodes().len(),
            typed_ast.nodes().root().map(|id| id.index())
        ),
        (51, Some(50))
    );
    assert_eq!(
        (
            resolved.nodes().len(),
            resolved.nodes().root().map(|id| id.index())
        ),
        (51, Some(50))
    );
    assert!(typed_ast.contexts().is_empty());
    assert!(typed_ast.types().is_empty());
    assert!(typed_ast.facts().is_empty());
    assert!(typed_ast.coercions().is_empty());
    assert!(typed_ast.initial_obligations().is_empty());
    assert!(typed_ast.diagnostics().is_empty());
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
    assert!(typed_ast.source_statement_references().is_none());
    assert!(resolved.source_statement_references().is_none());
    assert!(typed_ast.source_application().is_none());
    assert!(typed_ast.source_structure().is_none());
    assert!(typed_ast.source_set_term().is_none());
    assert!(typed_ast.source_attribute_definition().is_none());
    assert!(typed_ast.source_predicate_definition().is_none());
    assert!(typed_ast.source_functor_definition().is_none());
    assert!(typed_ast.source_mode_definition().is_none());
    assert!(typed_ast.source_structure_definition().is_none());
    assert_eq!(
        typed_ast
            .debug_text()
            .matches("source-proof-local-declaration-debug-v1")
            .count(),
        1
    );
    assert_eq!(
        resolved
            .debug_text()
            .matches("source-proof-local-declaration-debug-v1")
            .count(),
        1
    );
    for forbidden in [
        "accepted",
        "discharged",
        "theorem-fact",
        "verification-condition",
    ] {
        assert!(!typed_ast.debug_text().contains(forbidden));
        assert!(!resolved.debug_text().contains(forbidden));
    }
    assert_task269b_typed_and_final_replay_preserve_empty_semantics();
}

fn assert_task269b_typed_and_final_replay_preserve_empty_semantics() {
    let (ast, module, _shells, symbols) =
        task253_ast_from_source_text(TASK269B_SOURCE_TEXT, 269_401);
    let SourceProofLocalDeclarationRouteOutput {
        typed_ast,
        resolved,
    } = source_proof_local_declaration_output(&ast, module, &symbols, TASK269B_SOURCE_TEXT)
        .expect("Task269B selector")
        .expect("Task269B route");

    assert_eq!(
        typed_ast.source_proof_local_declaration(),
        resolved.source_proof_local_declaration()
    );
    assert_eq!(
        (
            typed_ast.nodes().len(),
            typed_ast.nodes().root().map(|id| id.index()),
            resolved.nodes().len(),
            resolved.nodes().root().map(|id| id.index()),
        ),
        (56, Some(55), 56, Some(55))
    );
    let witnesses = typed_ast
        .source_statement_witnesses()
        .expect("Task269B lower witnesses");
    assert_eq!((witnesses.witnesses().len(), witnesses.names().len()), (2, 1));
    assert_eq!(
        typed_ast
            .source_proof_local_declaration()
            .expect("Task269B declaration owner")
            .declarations()
            .len(),
        1
    );
    assert!(typed_ast.contexts().is_empty());
    assert!(typed_ast.types().is_empty());
    assert!(typed_ast.facts().is_empty());
    assert!(typed_ast.coercions().is_empty());
    assert!(typed_ast.initial_obligations().is_empty());
    assert!(typed_ast.diagnostics().is_empty());
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
    assert!(typed_ast.source_statement_references().is_none());
    assert!(resolved.source_statement_references().is_none());
    assert!(typed_ast.source_application().is_none());
    assert!(typed_ast.source_structure().is_none());
    assert!(typed_ast.source_set_term().is_none());
    assert!(typed_ast.source_attribute_definition().is_none());
    assert!(typed_ast.source_predicate_definition().is_none());
    assert!(typed_ast.source_functor_definition().is_none());
    assert!(typed_ast.source_mode_definition().is_none());
    assert!(typed_ast.source_structure_definition().is_none());
    assert_eq!(
        typed_ast
            .debug_text()
            .matches("source-proof-local-declaration-debug-v1")
            .count(),
        1
    );
    assert_eq!(
        resolved
            .debug_text()
            .matches("source-proof-local-declaration-debug-v1")
            .count(),
        1
    );
    for forbidden in [
        "accepted",
        "discharged",
        "theorem-fact",
        "verification-condition",
    ] {
        assert!(!typed_ast.debug_text().contains(forbidden));
        assert!(!resolved.debug_text().contains(forbidden));
    }
}
