use super::type_elaboration::{
    SOURCE_STATEMENT_B3M1_TEXT as TASK269B_SOURCE_TEXT,
    SOURCE_STATEMENT_B3N_TEXT as TASK269A_SOURCE_TEXT, SourceProofLocalDeclarationRouteMutation,
    SourceProofLocalDeclarationRouteOutput, source_proof_local_declaration_output,
    source_proof_local_declaration_output_with_mutation,
    source_statement_output_with_source as task269a_lower_output,
    source_statement_transport_detail_keys as task269a_legacy_detail_keys,
};
use super::{
    SOURCE_PROOF_LOCAL_LET_TEXT, SourceProofLocalLetBindingRouteMutation,
    SourceProofLocalLetBindingRouteOutput, SourceProofLocalLetLowerMutation,
    SourceProofLocalLetLowerOutput, SourceProofLocalLetResolverProfileMutation,
    SourceProofLocalLetShellMutation, SourceProofLocalLetSurfaceMutation,
    SourceProofLocalLetTypeRouteMutation, SourceProofLocalLetTypeRouteOutput,
    source_proof_local_let_binding_output, source_proof_local_let_binding_output_with_mutation,
    source_proof_local_let_lower_output, source_proof_local_let_lower_output_with_mutation,
    source_proof_local_let_lower_output_with_resolver_mutation,
    source_proof_local_let_lower_output_with_resolver_profile_mutation,
    source_proof_local_let_lower_output_with_shell_mutation,
    source_proof_local_let_lower_output_with_surface_mutation,
    source_proof_local_let_type_output, source_proof_local_let_type_output_with_mutation,
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

#[test]
fn task269cp_exact_source_surface_resolver_lower_output_and_debug_are_stable() {
    assert_eq!(SOURCE_PROOF_LOCAL_LET_TEXT.len(), 100);
    assert_eq!(
        sha256_text(SOURCE_PROOF_LOCAL_LET_TEXT),
        "7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a"
    );
    assert!(SOURCE_PROOF_LOCAL_LET_TEXT.ends_with('\n'));
    assert!(!SOURCE_PROOF_LOCAL_LET_TEXT.ends_with("\n\n"));
    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_PROOF_LOCAL_LET_TEXT, 269_500);
    assert_eq!(diagnostics, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (51, Some(50))
    );
    assert!(ast.expression_root().is_none());
    assert_eq!(
        ast.token_nodes()
            .iter()
            .map(|token| token.index())
            .collect::<Vec<_>>(),
        (0..24).collect::<Vec<_>>()
    );
    assert_eq!(
        sha256_text(&ast.snapshot_text()),
        "1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68"
    );

    let output = source_proof_local_let_lower_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
    )
    .expect("Task269CP exact selector")
    .expect("Task269CP exact lower output");
    assert_task269cp_exact_lower_output(&ast, &module, &output);

    assert_eq!((shells.declarations().len(), shells.exports().len()), (2, 0));
    let [reserve_shell, theorem_shell] = shells.declarations() else {
        panic!("Task269CP exact shells");
    };
    assert_eq!(
        (
            reserve_shell.id().index(),
            reserve_shell.ordinal(),
            reserve_shell.node_id().index(),
            reserve_shell.range().start,
            reserve_shell.range().end,
            reserve_shell.recovered(),
        ),
        (0, 0, 27, 0, 18, false)
    );
    assert_eq!(
        (
            theorem_shell.id().index(),
            theorem_shell.ordinal(),
            theorem_shell.node_id().index(),
            theorem_shell.range().start,
            theorem_shell.range().end,
            theorem_shell.recovered(),
        ),
        (1, 1, 47, 19, 99, false)
    );
    for (shell, kind, syntax) in [
        (
            reserve_shell,
            mizar_resolve::declarations::DeclarationShellKind::Reserve,
            mizar_syntax::SyntaxKind::ReserveItem,
        ),
        (
            theorem_shell,
            mizar_resolve::declarations::DeclarationShellKind::Theorem,
            mizar_syntax::SyntaxKind::TheoremItem,
        ),
    ] {
        assert_eq!(shell.kind(), kind);
        assert_eq!(shell.module(), &module);
        assert_eq!(shell.syntax_kind(), syntax);
        assert!(shell.parent().is_none());
        assert_eq!(
            shell.visibility().state(),
            mizar_resolve::declarations::DeclarationShellVisibilityState::Unspecified
        );
        assert!(shell.visibility().marker_range().is_none());
        assert!(shell.visibility().spelling().is_none());
    }
    assert_eq!(
        (
            symbols.symbols().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
            symbols.imports().len(),
            symbols.exports().len(),
            symbols.labels().len(),
            symbols.overloads().len(),
            symbols.registrations().len(),
        ),
        (1, 1, 1, 0, 0, 0, 0, 0)
    );
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = symbols
        .symbols()
        .visible_candidates(&namespace, "FormulaStatementLetSmoke");
    let [owner] = owners.as_slice() else {
        panic!("Task269CP exact theorem owner");
    };
    assert_eq!(owner.kind(), mizar_resolve::env::SymbolKind::Theorem);
    assert_eq!(owner.symbol().module(), &module);
    assert_eq!(owner.namespace(), &namespace);
    assert_eq!(owner.primary_spelling(), "FormulaStatementLetSmoke");
    assert!(owner.notation_spelling().is_none());
    assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(owner.contribution().index(), 0);
    assert!(matches!(
        owner.signature(),
        Some(mizar_resolve::env::SignatureShell::Opaque { schema, payload })
            if schema == "parser-signature-v1"
                && payload
                    == "node=TheoremItem;symbol=theorem;definition=theorem;\
primary_tokens=theorem FormulaStatementLetSmoke : x = x proof let y be set ; thus x = x ; end ;\
;notation=_;arity=_;roles=FormulaExpression,ProofBlock"
    ));
    assert!(owner.relations().is_empty());
    assert_eq!(owner.origin().source_id(), ast.source_id);
    assert_eq!(owner.origin().module_id(), &module);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 19,
            end: 99,
        })
    );
    assert_eq!(owner.origin().structural_path(), [2, 1]);
    assert!(owner.origin().import_edge().is_none());
    assert!(!owner.origin().is_recovered());
    let definition = symbols
        .definitions()
        .by_symbol(owner.symbol())
        .expect("Task269CP exact theorem definition");
    assert_eq!(definition.id().index(), 0);
    assert_eq!(definition.symbol(), owner.symbol());
    assert_eq!(definition.kind(), mizar_resolve::env::DefinitionKind::Theorem);
    assert_eq!(definition.visibility(), mizar_resolve::env::Visibility::Public);
    assert!(definition.parameters().is_empty());
    assert!(definition.binders().is_empty());
    assert!(definition.arity().is_none());
    assert!(definition.notation_shape().is_none());
    assert!(definition.doc_attachment().is_none());
    assert_eq!(definition.origin(), owner.origin());
    assert_eq!(definition.contribution(), owner.contribution());
    assert!(definition.conflict().is_none());
    assert!(definition.dependencies().is_empty());
    assert_eq!(definition.signature(), owner.signature());
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .expect("Task269CP exact contribution");
    assert_eq!(contribution.id().index(), 0);
    assert_eq!(contribution.module(), &module);
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::LocalSource { source_id }
            if *source_id == ast.source_id
    ));
    assert_eq!(
        contribution.anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 0,
            end: 18,
        })
    );
    assert_eq!(contribution.effects().symbols(), [owner.symbol().clone()]);
    assert_eq!(contribution.effects().definitions(), [definition.id()]);
    assert!(contribution.effects().labels().is_empty());
    assert!(contribution.effects().overload_groups().is_empty());
    assert!(contribution.effects().registrations().is_empty());
    assert!(contribution.effects().lexical_summaries().is_empty());
    assert!(contribution.effects().namespace_edges().is_empty());
    assert!(contribution.effects().declaration_dependencies().is_empty());
    assert!(contribution.effects().imports().is_empty());
    assert!(contribution.effects().exports().is_empty());
    assert!(contribution.effects().diagnostics().is_empty());
}

fn assert_task269cp_exact_lower_output(
    ast: &mizar_syntax::SurfaceAst,
    module: &mizar_resolve::resolved_ast::ModuleId,
    output: &SourceProofLocalLetLowerOutput,
) {
    assert_eq!(output.source_id(), ast.source_id);
    assert_eq!(output.module_id(), module);
    assert_eq!(
        output.source_fingerprint(),
        "7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a"
    );
    assert_eq!(
        output.surface_fingerprint(),
        "1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68"
    );
    assert_eq!(output.theorem_symbol().module(), module);
    assert_eq!(output.theorem_definition().index(), 0);
    assert_eq!(output.contribution().index(), 0);
    assert_eq!(
        task269cp_range_tuple(output.theorem_range()),
        (ast.source_id, 19, 99)
    );
    assert_eq!(
        task269cp_range_tuple(output.proof_range()),
        (ast.source_id, 59, 98)
    );
    assert_eq!(
        task269cp_range_tuple(output.let_range()),
        (ast.source_id, 67, 80)
    );
    assert_eq!(
        task269cp_range_tuple(output.segment_range()),
        (ast.source_id, 71, 79)
    );
    assert_eq!(
        task269cp_range_tuple(output.name_range()),
        (ast.source_id, 71, 72)
    );
    assert_eq!(
        task269cp_range_tuple(output.type_range()),
        (ast.source_id, 76, 79)
    );
    assert_eq!(
        task269cp_range_tuple(output.type_head_range()),
        (ast.source_id, 76, 79)
    );
    assert_eq!(output.source_ordinal(), 1);
    assert_eq!(output.local().spelling(), "y");
    assert_eq!(output.local().scope().path(), [0]);
    assert_eq!(output.local().declaration_range(), output.name_range());
    assert_eq!(output.local().visible_after_ordinal(), 1);
    let expected_debug = format!(
        concat!(
            "source-proof-local-let-lower-debug-v1\n",
            "module: {}::{}\n",
            "source-fingerprint: \"7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a\"\n",
            "surface-fingerprint: \"1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68\"\n",
            "theorem symbol={:?} definition=0 contribution=0 range=19..99 proof=59..98\n",
            "let range=67..80 segment=71..79 source_ordinal=1\n",
            "name range=71..72 spelling=\"y\" scope=[0] visible_after=1\n",
            "type range=76..79 head=76..79 spelling=\"set\" form=bare\n",
        ),
        module.package().as_str(),
        module.path().as_str(),
        output.theorem_symbol().fqn().as_str(),
    );
    assert_eq!(output.debug_text(), expected_debug);
}

fn task269cp_range_tuple(
    source_range: mizar_session::SourceRange,
) -> (mizar_session::SourceId, usize, usize) {
    (source_range.source_id, source_range.start, source_range.end)
}

fn task269cp_ast_with_expression_root(
    ast: &mizar_syntax::SurfaceAst,
) -> mizar_syntax::SurfaceAst {
    let mut builder = mizar_syntax::SurfaceAstBuilder::new(ast.source_id);
    let mut rebuilt = Vec::with_capacity(ast.nodes().len());
    for node in ast.nodes() {
        let children = node
            .children
            .iter()
            .map(|child| rebuilt[child.index()])
            .collect::<Vec<_>>();
        let id = match &node.kind {
            mizar_syntax::SurfaceNodeKind::Token(token) => {
                builder.add_token(token.kind, token.text.clone(), node.range)
            }
            structural => builder.add_node(structural.clone(), node.range, children),
        };
        rebuilt.push(id);
    }
    builder.finish(
        ast.root().map(|root| rebuilt[root.index()]),
        Some(rebuilt[33]),
    )
}

#[test]
fn task269cp_surface_resolver_and_local_corruption_matrix_fails_closed() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_510);
    for index in 0..51 {
        for mutation in [
            SourceProofLocalLetSurfaceMutation::NodeKind(index),
            SourceProofLocalLetSurfaceMutation::NodeSourceId(index),
            SourceProofLocalLetSurfaceMutation::NodeRange(index),
            SourceProofLocalLetSurfaceMutation::NodeRecovery(index),
            SourceProofLocalLetSurfaceMutation::NodeChildren(index),
        ] {
            assert!(
                source_proof_local_let_lower_output_with_surface_mutation(
                    &ast,
                    module.clone(),
                    &shells,
                    &symbols,
                    SOURCE_PROOF_LOCAL_LET_TEXT,
                    mutation,
                )
                .is_none(),
                "Task269CP Surface mutation {mutation:?} selected"
            );
        }
    }
    for index in 0..24 {
        assert!(
            source_proof_local_let_lower_output_with_surface_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_PROOF_LOCAL_LET_TEXT,
                SourceProofLocalLetSurfaceMutation::TokenNode(index),
            )
            .is_none(),
            "Task269CP token side-table mutation {index} selected"
        );
    }
    for mutation in [
        SourceProofLocalLetSurfaceMutation::ExpressionRoot,
        SourceProofLocalLetSurfaceMutation::TokenNodeCount,
    ] {
        assert!(
            source_proof_local_let_lower_output_with_surface_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_PROOF_LOCAL_LET_TEXT,
                mutation,
            )
            .is_none(),
            "Task269CP Surface side-table mutation {mutation:?} selected"
        );
    }
    let expression_root_ast = task269cp_ast_with_expression_root(&ast);
    assert!(
        source_proof_local_let_lower_output(
            &expression_root_ast,
            module.clone(),
            &shells,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .is_none()
    );
    assert!(
        source_proof_local_let_lower_output_with_surface_mutation(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
            SourceProofLocalLetSurfaceMutation::MissingRootIdentity,
        )
        .is_none()
    );
    assert!(
        source_proof_local_let_lower_output_with_surface_mutation(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
            SourceProofLocalLetSurfaceMutation::WrongRootIdentity,
        )
        .is_none()
    );

    for mutation in [
        SourceProofLocalLetLowerMutation::SourceId,
        SourceProofLocalLetLowerMutation::Module,
        SourceProofLocalLetLowerMutation::SourceFingerprint,
        SourceProofLocalLetLowerMutation::SurfaceFingerprint,
        SourceProofLocalLetLowerMutation::TheoremSymbol,
        SourceProofLocalLetLowerMutation::TheoremDefinition,
        SourceProofLocalLetLowerMutation::Contribution,
        SourceProofLocalLetLowerMutation::TheoremRange,
        SourceProofLocalLetLowerMutation::ProofRange,
        SourceProofLocalLetLowerMutation::LetRange,
        SourceProofLocalLetLowerMutation::SegmentRange,
        SourceProofLocalLetLowerMutation::NameRange,
        SourceProofLocalLetLowerMutation::TypeRange,
        SourceProofLocalLetLowerMutation::TypeHeadRange,
        SourceProofLocalLetLowerMutation::SourceOrdinal,
        SourceProofLocalLetLowerMutation::LocalSpelling,
        SourceProofLocalLetLowerMutation::LocalScope,
        SourceProofLocalLetLowerMutation::LocalRange,
        SourceProofLocalLetLowerMutation::LocalVisibleAfter,
    ] {
        assert!(
            source_proof_local_let_lower_output_with_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_PROOF_LOCAL_LET_TEXT,
                mutation,
            )
            .expect("Task269CP exact selector under lower mutation")
            .is_err(),
            "Task269CP lower mutation {mutation:?} succeeded"
        );
    }

    for shell in 0..2 {
        for mutation in [
            SourceProofLocalLetShellMutation::Id(shell),
            SourceProofLocalLetShellMutation::Ordinal(shell),
            SourceProofLocalLetShellMutation::Kind(shell),
            SourceProofLocalLetShellMutation::Module(shell),
            SourceProofLocalLetShellMutation::Node(shell),
            SourceProofLocalLetShellMutation::Syntax(shell),
            SourceProofLocalLetShellMutation::Range(shell),
            SourceProofLocalLetShellMutation::Parent(shell),
            SourceProofLocalLetShellMutation::VisibilityState(shell),
            SourceProofLocalLetShellMutation::VisibilityMarker(shell),
            SourceProofLocalLetShellMutation::VisibilitySpelling(shell),
            SourceProofLocalLetShellMutation::Recovery(shell),
        ] {
            assert!(
                source_proof_local_let_lower_output_with_shell_mutation(
                    &ast,
                    module.clone(),
                    &shells,
                    &symbols,
                    SOURCE_PROOF_LOCAL_LET_TEXT,
                    mutation,
                )
                .expect("Task269CP exact selector under shell mutation")
                .is_err(),
                "Task269CP shell mutation {mutation:?} succeeded"
            );
        }
    }
    for mutation in [
        SourceProofLocalLetResolverProfileMutation::ResolverModule,
        SourceProofLocalLetResolverProfileMutation::ImportIndex,
        SourceProofLocalLetResolverProfileMutation::ExportIndex,
        SourceProofLocalLetResolverProfileMutation::LabelIndex,
        SourceProofLocalLetResolverProfileMutation::OverloadIndex,
        SourceProofLocalLetResolverProfileMutation::RegistrationIndex,
        SourceProofLocalLetResolverProfileMutation::LexicalSummaryIndex,
        SourceProofLocalLetResolverProfileMutation::NamespaceGraph,
        SourceProofLocalLetResolverProfileMutation::DeclarationDependencyIndex,
        SourceProofLocalLetResolverProfileMutation::ModuleSummaryIndex,
        SourceProofLocalLetResolverProfileMutation::SymbolModule,
        SourceProofLocalLetResolverProfileMutation::SymbolNotation,
        SourceProofLocalLetResolverProfileMutation::SymbolContribution,
        SourceProofLocalLetResolverProfileMutation::SymbolRelations,
        SourceProofLocalLetResolverProfileMutation::SymbolOriginSource,
        SourceProofLocalLetResolverProfileMutation::SymbolOriginImport,
        SourceProofLocalLetResolverProfileMutation::DefinitionId,
        SourceProofLocalLetResolverProfileMutation::DefinitionParameters,
        SourceProofLocalLetResolverProfileMutation::DefinitionBinders,
        SourceProofLocalLetResolverProfileMutation::DefinitionNotation,
        SourceProofLocalLetResolverProfileMutation::DefinitionDoc,
        SourceProofLocalLetResolverProfileMutation::DefinitionContribution,
        SourceProofLocalLetResolverProfileMutation::DefinitionConflict,
        SourceProofLocalLetResolverProfileMutation::DefinitionDependencies,
        SourceProofLocalLetResolverProfileMutation::ContributionLabelEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionOverloadEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionRegistrationEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionLexicalEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionNamespaceEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionDeclarationDependencyEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionImportEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionExportEffect,
        SourceProofLocalLetResolverProfileMutation::ContributionDiagnosticEffect,
    ] {
        assert!(
            source_proof_local_let_lower_output_with_resolver_profile_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_PROOF_LOCAL_LET_TEXT,
                mutation,
            )
            .expect("Task269CP exact selector under resolver profile mutation")
            .is_err(),
            "Task269CP resolver profile mutation {mutation:?} succeeded"
        );
    }

    let (_, _, wrong_shells, _) = task253_ast_from_source_text(TASK269A_SOURCE_TEXT, 269_511);
    assert!(
        source_proof_local_let_lower_output(
            &ast,
            module.clone(),
            &wrong_shells,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .expect("Task269CP selector with wrong shells")
        .is_err()
    );
    for (drop_symbols, drop_definitions, drop_contributions) in
        [(true, false, false), (false, true, false), (false, false, true)]
    {
        let corrupted = task269cp_symbols_with_missing_index(
            &symbols,
            drop_symbols,
            drop_definitions,
            drop_contributions,
        );
        assert!(
            source_proof_local_let_lower_output(
                &ast,
                module.clone(),
                &shells,
                &corrupted,
                SOURCE_PROOF_LOCAL_LET_TEXT,
            )
            .expect("Task269CP selector with resolver index corruption")
            .is_err()
        );
    }
    let neutral_reconstruction = source_proof_local_let_lower_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
        |symbols| task269cp_mutate_resolver(symbols, Task269cpResolverMutation::None),
    )
    .expect("Task269CP selector under neutral resolver reconstruction")
    .expect("Task269CP neutral resolver reconstruction");
    assert_task269cp_exact_lower_output(&ast, &module, &neutral_reconstruction);
    for mutation in Task269cpResolverMutation::ALL {
        assert!(
            source_proof_local_let_lower_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_PROOF_LOCAL_LET_TEXT,
                |symbols| task269cp_mutate_resolver(symbols, mutation),
            )
            .expect("Task269CP exact selector under resolver field mutation")
            .is_err(),
            "Task269CP resolver mutation {mutation:?} succeeded"
        );
    }
    let visible_y = task269cp_symbols_with_visible_y(&symbols, ast.source_id);
    assert_eq!(
        source_proof_local_let_lower_output(
            &ast,
            module.clone(),
            &shells,
            &visible_y,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .expect("Task269CP selector with visible y"),
        Err("Task269CP local y already resolves as a module symbol".to_owned())
    );
    let wrong_module = mizar_resolve::resolved_ast::ModuleId::new(
        module.package().clone(),
        mizar_session::ModulePath::new("tests.task269cp_wrong_module"),
    );
    assert!(
        source_proof_local_let_lower_output(
            &ast,
            wrong_module,
            &shells,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .expect("Task269CP selector with wrong module")
        .is_err()
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task269cpResolverMutation {
    None,
    SymbolKind,
    SymbolNamespace,
    SymbolSpelling,
    SymbolVisibility,
    SymbolExport,
    SymbolOriginModule,
    SymbolOriginAnchor,
    SymbolOriginPath,
    SymbolOriginRecovery,
    SymbolSignature,
    SymbolCorruptPresentSignature,
    DefinitionSymbol,
    DefinitionKind,
    DefinitionVisibility,
    DefinitionArity,
    DefinitionOrigin,
    DefinitionSignature,
    ContributionModule,
    ContributionKind,
    ContributionAnchor,
    ContributionSymbolEffect,
    ContributionDefinitionEffect,
}

impl Task269cpResolverMutation {
    const ALL: [Self; 22] = [
        Self::SymbolKind,
        Self::SymbolNamespace,
        Self::SymbolSpelling,
        Self::SymbolVisibility,
        Self::SymbolExport,
        Self::SymbolOriginModule,
        Self::SymbolOriginAnchor,
        Self::SymbolOriginPath,
        Self::SymbolOriginRecovery,
        Self::SymbolSignature,
        Self::SymbolCorruptPresentSignature,
        Self::DefinitionSymbol,
        Self::DefinitionKind,
        Self::DefinitionVisibility,
        Self::DefinitionArity,
        Self::DefinitionOrigin,
        Self::DefinitionSignature,
        Self::ContributionModule,
        Self::ContributionKind,
        Self::ContributionAnchor,
        Self::ContributionSymbolEffect,
        Self::ContributionDefinitionEffect,
    ];
}

fn task269cp_mutate_resolver(
    symbols: mizar_resolve::env::SymbolEnv,
    mutation: Task269cpResolverMutation,
) -> mizar_resolve::env::SymbolEnv {
    let module = symbols.module_id().clone();
    let owner = symbols
        .symbols()
        .iter()
        .next()
        .expect("Task269CP owner before mutation");
    let definition = symbols
        .definitions()
        .by_symbol(owner.symbol())
        .expect("Task269CP definition before mutation");
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .expect("Task269CP contribution before mutation");
    let wrong_module = mizar_resolve::resolved_ast::ModuleId::new(
        module.package().clone(),
        mizar_session::ModulePath::new(format!("{}.field-mutation", module.path().as_str())),
    );
    let theorem_range = mizar_session::SourceRange {
        source_id: owner.origin().source_id(),
        start: 19,
        end: 99,
    };
    let symbol_origin = match mutation {
        Task269cpResolverMutation::SymbolOriginModule => {
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                owner.origin().source_id(),
                wrong_module.clone(),
                owner.origin().anchor().clone(),
                owner.origin().structural_path().to_vec(),
            )
        }
        Task269cpResolverMutation::SymbolOriginAnchor => {
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                owner.origin().source_id(),
                module.clone(),
                mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                    end: 98,
                    ..theorem_range
                }),
                owner.origin().structural_path().to_vec(),
            )
        }
        Task269cpResolverMutation::SymbolOriginPath => {
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                owner.origin().source_id(),
                module.clone(),
                owner.origin().anchor().clone(),
                vec![2, 2],
            )
        }
        Task269cpResolverMutation::SymbolOriginRecovery => {
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                owner.origin().source_id(),
                module.clone(),
                owner.origin().anchor().clone(),
                owner.origin().structural_path().to_vec(),
            )
            .recovered()
        }
        _ => owner.origin().clone(),
    };
    let contribution_module = if mutation == Task269cpResolverMutation::ContributionModule {
        wrong_module.clone()
    } else {
        contribution.module().clone()
    };
    let contribution_kind = if mutation == Task269cpResolverMutation::ContributionKind {
        mizar_resolve::env::ContributionKind::ImportedSource {
            source_id: owner.origin().source_id(),
        }
    } else {
        contribution.kind().clone()
    };
    let contribution_anchor = if mutation == Task269cpResolverMutation::ContributionAnchor {
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: owner.origin().source_id(),
            start: 0,
            end: 17,
        })
    } else {
        contribution.anchor().clone()
    };
    let mut contributions = mizar_resolve::env::SourceContributionIndex::new();
    let contribution_id = contributions.insert(
        contribution_module,
        contribution_kind,
        contribution_anchor,
    );

    let symbol_kind = if mutation == Task269cpResolverMutation::SymbolKind {
        mizar_resolve::env::SymbolKind::Functor
    } else {
        owner.kind()
    };
    let symbol_namespace = if mutation == Task269cpResolverMutation::SymbolNamespace {
        mizar_resolve::env::NamespacePath::new(format!("{}.wrong", module.path().as_str()))
    } else {
        owner.namespace().clone()
    };
    let symbol_spelling = if mutation == Task269cpResolverMutation::SymbolSpelling {
        "FormulaStatementLetSmokeWrong"
    } else {
        owner.primary_spelling()
    };
    let symbol_visibility = if mutation == Task269cpResolverMutation::SymbolVisibility {
        mizar_resolve::env::Visibility::Private
    } else {
        owner.visibility()
    };
    let symbol_export = if mutation == Task269cpResolverMutation::SymbolExport {
        mizar_resolve::env::ExportStatus::LocalOnly
    } else {
        owner.export_status()
    };
    let mut symbol_entry = mizar_resolve::env::SymbolEntry::new(
        owner.symbol().clone(),
        symbol_kind,
        symbol_namespace,
        symbol_spelling,
        symbol_origin,
        contribution_id,
    )
    .with_visibility(symbol_visibility)
    .with_export_status(symbol_export)
    .with_relations(owner.relations().to_vec());
    if let Some(spelling) = owner.notation_spelling() {
        symbol_entry = symbol_entry.with_notation_spelling(spelling);
    }
    if mutation == Task269cpResolverMutation::SymbolCorruptPresentSignature {
        symbol_entry =
            symbol_entry.with_signature(mizar_resolve::env::SignatureShell::Malformed {
                class: "Task269CP-corrupt-present".to_owned(),
            });
    } else if mutation != Task269cpResolverMutation::SymbolSignature
        && let Some(signature) = owner.signature()
    {
        symbol_entry = symbol_entry.with_signature(signature.clone());
    }
    let mut symbol_index = mizar_resolve::env::SymbolIndex::new();
    symbol_index.insert(symbol_entry);

    let definition_symbol = if mutation == Task269cpResolverMutation::DefinitionSymbol {
        mizar_resolve::resolved_ast::SymbolId::new(
            module.clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new("Task269CP/wrong-definition/0"),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                "{}::{}::Task269CP/wrong-definition/0",
                module.package().as_str(),
                module.path().as_str(),
            )),
        )
    } else {
        definition.symbol().clone()
    };
    let definition_kind = if mutation == Task269cpResolverMutation::DefinitionKind {
        mizar_resolve::env::DefinitionKind::Functor
    } else {
        definition.kind()
    };
    let definition_origin = if mutation == Task269cpResolverMutation::DefinitionOrigin {
        mizar_resolve::resolved_ast::SemanticOrigin::new(
            definition.origin().source_id(),
            module.clone(),
            mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                end: 98,
                ..theorem_range
            }),
            definition.origin().structural_path().to_vec(),
        )
    } else {
        definition.origin().clone()
    };
    let definition_visibility = if mutation == Task269cpResolverMutation::DefinitionVisibility {
        mizar_resolve::env::Visibility::Private
    } else {
        definition.visibility()
    };
    let mut definition_shell = mizar_resolve::env::DefinitionShell::new(
        definition_symbol,
        definition_kind,
        definition_origin,
        contribution_id,
    )
    .with_visibility(definition_visibility);
    if mutation == Task269cpResolverMutation::DefinitionArity {
        definition_shell = definition_shell.with_arity(1);
    }
    if mutation == Task269cpResolverMutation::SymbolCorruptPresentSignature {
        definition_shell =
            definition_shell.with_signature(mizar_resolve::env::SignatureShell::Malformed {
                class: "Task269CP-corrupt-present".to_owned(),
            });
    } else if mutation != Task269cpResolverMutation::DefinitionSignature
        && let Some(signature) = definition.signature()
    {
        definition_shell = definition_shell.with_signature(signature.clone());
    }
    let mut definitions = mizar_resolve::env::DefinitionIndex::new();
    let definition_id = definitions.insert(definition_shell);
    if mutation != Task269cpResolverMutation::ContributionSymbolEffect {
        contributions.add_symbol(contribution_id, owner.symbol().clone());
    }
    if mutation != Task269cpResolverMutation::ContributionDefinitionEffect {
        contributions.add_definition(contribution_id, definition_id);
    }

    mizar_resolve::env::SymbolEnv::new(
        module,
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbol_index,
            labels: symbols.labels().clone(),
            definitions,
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

fn task269cp_symbols_with_missing_index(
    symbols: &mizar_resolve::env::SymbolEnv,
    drop_symbols: bool,
    drop_definitions: bool,
    drop_contributions: bool,
) -> mizar_resolve::env::SymbolEnv {
    mizar_resolve::env::SymbolEnv::new(
        symbols.module_id().clone(),
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: if drop_symbols {
                mizar_resolve::env::SymbolIndex::new()
            } else {
                symbols.symbols().clone()
            },
            labels: symbols.labels().clone(),
            definitions: if drop_definitions {
                mizar_resolve::env::DefinitionIndex::new()
            } else {
                symbols.definitions().clone()
            },
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: if drop_contributions {
                mizar_resolve::env::SourceContributionIndex::new()
            } else {
                symbols.contributions().clone()
            },
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

fn task269cp_symbols_with_visible_y(
    symbols: &mizar_resolve::env::SymbolEnv,
    source_id: mizar_session::SourceId,
) -> mizar_resolve::env::SymbolEnv {
    let module = symbols.module_id().clone();
    let contribution = symbols
        .contributions()
        .iter()
        .next()
        .expect("Task269CP contribution")
        .id();
    let mut symbol_index = symbols.symbols().clone();
    let symbol = mizar_resolve::resolved_ast::SymbolId::new(
        module.clone(),
        mizar_resolve::resolved_ast::LocalSymbolId::new("Task269CP/y/0"),
        mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
            "{}::{}::Task269CP/y/0",
            module.package().as_str(),
            module.path().as_str(),
        )),
    );
    symbol_index.insert(
        mizar_resolve::env::SymbolEntry::new(
            symbol,
            mizar_resolve::env::SymbolKind::Functor,
            mizar_resolve::env::NamespacePath::new(module.path().as_str()),
            "y",
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                source_id,
                module.clone(),
                mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                    source_id,
                    start: 71,
                    end: 72,
                }),
                vec![269, 3],
            ),
            contribution,
        )
        .with_visibility(mizar_resolve::env::Visibility::Public)
        .with_export_status(mizar_resolve::env::ExportStatus::Exported),
    );
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
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

#[test]
fn task269cp_near_misses_and_neighbor_families_remain_isolated() {
    let near_misses = [
        SOURCE_PROOF_LOCAL_LET_TEXT.trim_end_matches('\n').to_owned(),
        format!("{SOURCE_PROOF_LOCAL_LET_TEXT}\n"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let z be set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y, z be set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y be set, z be set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y be empty set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y be set such that x = x;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y be set by A;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "given y being set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "consider y being set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "take y = x;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "set y = x;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "reconsider y = x as set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "deffunc y() = x;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "defpred y[] means x = x;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("thus x = x;", "thus y = y;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace(
            "let y be set;\n  thus x = x;",
            "thus x = x proof\n    let y be set;\n    thus x = x;\n  end;",
        ),
    ];
    for (ordinal, source) in near_misses.into_iter().enumerate() {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(&source, 269_520 + ordinal);
        assert!(
            source_proof_local_let_lower_output(&ast, module, &shells, &symbols, &source).is_none(),
            "Task269CP selected near miss {ordinal}"
        );
    }

    for (ordinal, source) in [TASK269A_SOURCE_TEXT, TASK269B_SOURCE_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, 269_550 + ordinal);
        assert!(
            source_proof_local_let_lower_output(&ast, module, &shells, &symbols, source).is_none(),
            "Task269CP selected Task269A/B family {ordinal}"
        );
    }

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("mizar-test below workspace");
    for (ordinal, path) in [
        "tests/miz/fail/types/fail_type_elaboration_proof_local_declaration_gap_001.miz",
        "tests/miz/pass/parser/pass_parser_simple_statements_001.miz",
    ]
    .into_iter()
    .enumerate()
    {
        let source = std::fs::read_to_string(workspace_root.join(path))
            .unwrap_or_else(|error| panic!("read Task269CP boundary {path}: {error}"));
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(&source, 269_560 + ordinal);
        assert!(
            source_proof_local_let_lower_output(&ast, module, &shells, &symbols, &source).is_none(),
            "Task269CP selected broad boundary {path}"
        );
    }
}

#[test]
fn task269cp_private_lower_route_has_zero_checker_and_active_semantic_effect() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_570);
    let output = source_proof_local_let_lower_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
    )
    .expect("Task269CP exact selector")
    .expect("Task269CP private lower output");
    assert_task269cp_exact_lower_output(&ast, &module, &output);
    for forbidden in [
        "binding-env",
        "typed-ast",
        "resolved-typed-ast",
        "initial-obligation",
        "fact",
        "goal",
        "accepted",
        "discharged",
        "verification-condition",
    ] {
        assert!(!output.debug_text().contains(forbidden));
    }
    assert!(
        source_proof_local_declaration_output(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .is_none()
    );
    assert!(
        task269a_lower_output(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .is_none()
    );
    assert_eq!(
        task269a_legacy_detail_keys(
            &ast,
            module,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        ),
        None
    );
}

#[test]
fn task269c_exact_binding_transaction_debug_and_lookup_are_stable() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_600);
    let SourceProofLocalLetBindingRouteOutput {
        typed_ast,
        resolved,
    } = source_proof_local_let_binding_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
    )
    .expect("Task269C exact private selector")
    .expect("Task269C exact binding transaction");
    let handoff = typed_ast
        .source_proof_local_let_binding()
        .expect("Task269C typed owner");
    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(handoff.theorem_symbol().module(), &module);
    assert_eq!((handoff.theorem_definition().index(), handoff.contribution().index()), (0, 0));
    assert_eq!((handoff.base_binding_env().contexts().len(), handoff.base_binding_env().bindings().len(), handoff.base_binding_env().diagnostics().len()), (1, 1, 0));
    assert_eq!((handoff.binding_env().contexts().len(), handoff.binding_env().bindings().len(), handoff.binding_env().diagnostics().len()), (2, 2, 0));
    assert_eq!(handoff.base_binding_fingerprint(), handoff.base_binding_env().debug_text());
    assert_eq!(handoff.final_binding_fingerprint(), handoff.binding_env().debug_text());
    let row = handoff
        .bindings()
        .get(mizar_checker::source_proof_local_declaration::SourceProofLocalLetBindingId::new(0))
        .expect("Task269C one dense row");
    assert_eq!((row.binding().index(), row.binding_context().index(), row.source_ordinal(), row.visible_after_ordinal()), (1, 1, 1, 1));
    assert_eq!(row.recovery(), mizar_checker::source_proof_local_declaration::SourceProofLocalLetBindingRecovery::Normal);
    let local = handoff
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(1))
        .expect("Task269C let binding");
    assert_eq!(local.kind, mizar_checker::binding_env::BindingKind::LetBinding);
    assert_eq!(local.type_site, mizar_checker::binding_env::BindingTypeSite::Missing);
    assert!(handoff.debug_text().starts_with("source-proof-local-let-binding-debug-v1\n"));
    assert!(handoff.debug_text().ends_with(&format!("final-binding-fingerprint: {:?}\n", handoff.final_binding_fingerprint())));
    assert_eq!(resolved.source_proof_local_let_binding(), Some(handoff));
    assert!(typed_ast.debug_text().contains(&handoff.debug_text()));
    assert!(resolved.debug_text().contains(&handoff.debug_text()));
}

#[test]
fn task269c_lower_base_and_checker_corruption_fail_closed() {
    let cases = [
        (
            SourceProofLocalLetBindingRouteMutation::WrongLowerFingerprint,
            "source proof-local let-binding dependency mismatch",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::EmptyBase,
            "source proof-local let-binding base binding environment is invalid",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongTheoremRange,
            "source proof-local let-binding dependency mismatch",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongProofRange,
            "source proof-local let-binding dependency mismatch",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongLetRange,
            "source proof-local let-binding dependency mismatch",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongSegmentRange,
            "source proof-local let-binding dependency mismatch",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongNameRange,
            "source proof-local let-binding dependency mismatch",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongLocalSpelling,
            "source proof-local let-binding 0 is invalid",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongLocalScope,
            "source proof-local let-binding 0 is invalid",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongLocalRange,
            "source proof-local let-binding 0 is invalid",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongLocalVisibleAfter,
            "source proof-local let-binding 0 is invalid",
        ),
        (
            SourceProofLocalLetBindingRouteMutation::WrongSourceOrdinal,
            "source proof-local let-binding 0 is invalid",
        ),
    ];
    for (ordinal, (mutation, expected)) in cases.into_iter().enumerate() {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_610 + ordinal);
        assert_eq!(
            source_proof_local_let_binding_output_with_mutation(
                &ast,
                module,
                &shells,
                &symbols,
                SOURCE_PROOF_LOCAL_LET_TEXT,
                mutation,
            )
            .expect("Task269C exact selector under corruption"),
            Err(expected.to_owned()),
            "Task269C mutation {mutation:?}",
        );
    }
}

#[test]
fn task269c_typed_and_resolved_owners_are_one_shot_and_semantically_empty() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_630);
    let output = source_proof_local_let_binding_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
    )
    .expect("Task269C exact selector")
    .expect("Task269C owners");
    let handoff = output
        .typed_ast
        .source_proof_local_let_binding()
        .expect("Task269C typed owner")
        .clone();
    assert_eq!(
        output
            .typed_ast
            .clone()
            .with_source_proof_local_let_binding(handoff),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceProofLocalLetBinding)
    );
    assert!(output.typed_ast.nodes().is_empty());
    assert!(output.typed_ast.contexts().is_empty());
    assert!(output.typed_ast.types().is_empty());
    assert!(output.typed_ast.facts().is_empty());
    assert!(output.typed_ast.coercions().is_empty());
    assert!(output.typed_ast.initial_obligations().is_empty());
    assert!(output.typed_ast.diagnostics().is_empty());
    assert!(output.resolved.nodes().is_empty());
    assert!(output.resolved.expr_metadata().is_empty());
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
    assert!(output.resolved.diagnostics().is_empty());
    assert!(!output.resolved.debug_text().contains("initial-obligations:"));
}

#[test]
fn task269c_near_miss_neighbor_and_active_routes_remain_isolated() {
    let near_misses = [
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let z be set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y, z be set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y be set such that x = x;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y be set by A;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("thus x = x;", "thus y = y;"),
    ];
    for (ordinal, source) in near_misses.into_iter().enumerate() {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(&source, 269_640 + ordinal);
        assert!(
            source_proof_local_let_binding_output(
                &ast,
                module,
                &shells,
                &symbols,
                &source,
            )
            .is_none(),
            "Task269C selected near miss {ordinal}",
        );
    }
    for (ordinal, source) in [TASK269A_SOURCE_TEXT, TASK269B_SOURCE_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, 269_650 + ordinal);
        assert!(
            source_proof_local_let_binding_output(
                &ast,
                module,
                &shells,
                &symbols,
                source,
            )
            .is_none(),
            "Task269C selected Task269A/B family {ordinal}",
        );
    }
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_660);
    assert!(
        source_proof_local_declaration_output(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .is_none()
    );
    assert_eq!(
        task269a_legacy_detail_keys(
            &ast,
            module,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        ),
        None
    );
    assert_eq!(shells.declarations().len(), 2);
}

#[test]
fn task269ct_exact_type_composition_fingerprints_and_replay_are_stable() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_700);
    let SourceProofLocalLetTypeRouteOutput {
        typed_ast,
        resolved,
    } = source_proof_local_let_type_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
    )
    .expect("Task269CT exact private selector")
    .expect("Task269CT exact type composition");
    let handoff = typed_ast
        .source_proof_local_let_type()
        .expect("Task269CT typed owner");
    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(
        handoff.dependency_fingerprint(),
        handoff.dependency().debug_text()
    );
    assert_eq!(
        handoff.binding_fingerprint(),
        handoff.binding_env().debug_text()
    );
    assert_eq!(
        handoff.source_type_fingerprint(),
        handoff.source_type().debug_text()
    );
    assert_eq!(
        (
            handoff.binding_env().contexts().len(),
            handoff.binding_env().bindings().len(),
            handoff.binding_env().diagnostics().len(),
        ),
        (2, 2, 0)
    );
    assert_eq!(
        handoff.binding_env().contexts(),
        handoff.dependency().binding_env().contexts()
    );
    assert_eq!(
        handoff.binding_env().diagnostics(),
        handoff.dependency().binding_env().diagnostics()
    );
    assert_eq!(
        handoff
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(0)),
        handoff
            .dependency()
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(0))
    );
    assert_eq!(
        handoff
            .dependency()
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(1))
            .expect("Task269CT dependency binding")
            .type_site,
        mizar_checker::binding_env::BindingTypeSite::Missing
    );
    assert_eq!(
        handoff
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(1))
            .expect("Task269CT typed binding")
            .type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(SourceRange {
            source_id: ast.source_id,
            start: 76,
            end: 79,
        })
    );
    assert_eq!(
        (
            handoff.source_type().applications().len(),
            handoff.source_type().expressions().len(),
            handoff.source_type().arguments().len(),
            handoff.source_type().definition_returns().len(),
            handoff.source_type().mode_rhs().len(),
            handoff.source_type().structure_members().len(),
        ),
        (2, 2, 0, 0, 0, 0)
    );
    for (index, (start, end)) in [(14, 17), (76, 79)].into_iter().enumerate() {
        let application = handoff
            .source_type()
            .applications()
            .get(mizar_checker::source_type::SourceTypeApplicationId::new(index))
            .expect("Task269CT source type application");
        assert_eq!(
            application.id(),
            mizar_checker::source_type::SourceTypeApplicationId::new(index)
        );
        assert_eq!(
            application.binding(),
            mizar_checker::binding_env::BindingId::new(index)
        );
        assert_eq!(application.source_ordinal(), index);
        assert_eq!(
            application.root(),
            mizar_checker::source_type::SourceTypeExpressionId::new(index)
        );

        let expression = handoff
            .source_type()
            .expressions()
            .get(mizar_checker::source_type::SourceTypeExpressionId::new(index))
            .expect("Task269CT source type expression");
        assert_eq!(
            expression.id(),
            mizar_checker::source_type::SourceTypeExpressionId::new(index)
        );
        assert_eq!(expression.source_id(), ast.source_id);
        assert_eq!(expression.module_id(), &module);
        assert!(matches!(
            expression.site(),
            mizar_checker::typed_ast::TypedSiteRef::Role { node, role }
                if *node == mizar_checker::typed_ast::TypedNodeId::new(index)
                    && role.as_str() == "source.type.expression"
        ));
        assert_eq!(
            expression.source_range(),
            SourceRange {
                source_id: ast.source_id,
                start,
                end,
            }
        );
        assert_eq!(expression.spelling(), "set");
        assert!(matches!(
            expression.head_site(),
            mizar_checker::typed_ast::TypedSiteRef::Role { node, role }
                if *node == mizar_checker::typed_ast::TypedNodeId::new(index)
                    && role.as_str() == "source.type.head"
        ));
        assert_eq!(
            expression.head_range(),
            SourceRange {
                source_id: ast.source_id,
                start,
                end,
            }
        );
        assert_eq!(expression.head_spelling(), "set");
        assert_eq!(
            expression.form(),
            mizar_checker::source_type::SourceTypeApplicationForm::Bare
        );
        assert_eq!(
            expression.head(),
            &mizar_checker::source_type::SourceTypeHead::BuiltinSet
        );
        assert_eq!(
            expression.recovery(),
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
    }
    assert_eq!(
        typed_ast.nodes().root(),
        Some(mizar_checker::typed_ast::TypedNodeId::new(2))
    );
    for (index, (kind, start, end, children)) in [
        ("source.proof-local.let.reserve-type", 14, 17, Vec::new()),
        ("source.proof-local.let.type", 76, 79, Vec::new()),
        (
            "source.proof-local.let.type-root",
            0,
            99,
            vec![
                mizar_checker::typed_ast::TypedNodeId::new(0),
                mizar_checker::typed_ast::TypedNodeId::new(1),
            ],
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let node = typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("Task269CT typed node");
        assert_eq!(node.kind.as_str(), kind);
        assert!(node.resolved_node.is_none());
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(SourceRange {
                source_id: ast.source_id,
                start,
                end,
            })
        );
        assert_eq!(node.children, children);
        assert_eq!(node.typing, mizar_checker::typed_ast::TypingState::Unknown);
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
        assert_eq!(
            node.links,
            mizar_checker::typed_ast::TypedNodeLinks::default()
        );
    }
    assert!(
        handoff
            .debug_text()
            .starts_with("source-proof-local-let-type-debug-v1\n")
    );
    assert_eq!(resolved.source_proof_local_let_type(), Some(handoff));
    assert!(typed_ast.source_proof_local_let_binding().is_none());
    assert!(typed_ast.source_type().is_none());
    assert!(resolved.source_proof_local_let_binding().is_none());
    assert!(resolved.source_type().is_none());
    assert_eq!(resolved.nodes().len(), 3);
    for (index, node) in resolved.nodes().iter() {
        assert!(matches!(
            &node.kind,
            mizar_checker::resolved_typed_ast::ResolvedTypedNodeKind::SourcePreserved { role }
                if role.as_str() == "source.proof-local.let.type"
        ));
        assert_eq!(node.id, index);
        assert_eq!(node.typed_node.index(), index.index());
        assert!(node.final_type.is_none());
        assert!(node.metadata.is_none());
        assert!(node.diagnostics.is_empty());
        assert_eq!(
            node.recovery,
            mizar_checker::resolved_typed_ast::ResolvedNodeRecovery::Normal
        );
    }
}

#[test]
fn task269ct_dependency_input_and_arena_corruption_fail_closed() {
    let cases = [
        (
            SourceProofLocalLetTypeRouteMutation::WrongDependencyModule,
            "source proof-local let type dependency is invalid",
        ),
        (
            SourceProofLocalLetTypeRouteMutation::WrongTypeRange,
            "source proof-local let source type is invalid",
        ),
        (
            SourceProofLocalLetTypeRouteMutation::WrongArenaRoot,
            "source proof-local let source type is invalid",
        ),
        (
            SourceProofLocalLetTypeRouteMutation::WrongArenaKind,
            "source proof-local let source type is invalid",
        ),
    ];
    for (ordinal, (mutation, expected)) in cases.into_iter().enumerate() {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_710 + ordinal);
        assert_eq!(
            source_proof_local_let_type_output_with_mutation(
                &ast,
                module,
                &shells,
                &symbols,
                SOURCE_PROOF_LOCAL_LET_TEXT,
                mutation,
            )
            .expect("Task269CT exact selector under corruption"),
            Err(expected.to_owned()),
            "Task269CT mutation {mutation:?}",
        );
    }
}

#[test]
fn task269ct_typed_and_resolved_owners_are_one_shot_and_semantically_empty() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_730);
    let output = source_proof_local_let_type_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
    )
    .expect("Task269CT exact selector")
    .expect("Task269CT owners");
    let handoff = output
        .typed_ast
        .source_proof_local_let_type()
        .expect("Task269CT typed owner")
        .clone();
    assert_eq!(
        output
            .typed_ast
            .clone()
            .with_source_proof_local_let_type(handoff),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceProofLocalLetType)
    );
    assert_eq!(output.typed_ast.nodes().len(), 3);
    assert!(output.typed_ast.contexts().is_empty());
    assert!(output.typed_ast.types().is_empty());
    assert!(output.typed_ast.facts().is_empty());
    assert!(output.typed_ast.coercions().is_empty());
    assert!(output.typed_ast.initial_obligations().is_empty());
    assert!(output.typed_ast.diagnostics().is_empty());
    assert!(output.resolved.source_context().is_none());
    assert!(output.resolved.source_type().is_none());
    assert!(output.resolved.source_attribute().is_none());
    assert!(output.resolved.source_evidence().is_none());
    assert!(output.resolved.source_term().is_none());
    assert!(output.resolved.source_application().is_none());
    assert!(output.resolved.source_structure().is_none());
    assert!(output.resolved.source_set_term().is_none());
    assert!(output.resolved.source_atomic_formula().is_none());
    assert!(output.resolved.source_attribute_definition().is_none());
    assert!(output.resolved.source_functor_definition().is_none());
    assert!(output.resolved.source_property_implementation().is_none());
    assert!(output.resolved.source_mode_definition().is_none());
    assert!(output.resolved.source_structure_definition().is_none());
    assert!(output.resolved.source_predicate_definition().is_none());
    assert!(output.resolved.source_composite_formula().is_none());
    assert!(output.resolved.source_formula_composition().is_none());
    assert!(
        output
            .resolved
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        output
            .resolved
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(output.resolved.source_statement().is_none());
    assert!(output.resolved.source_statement_references().is_none());
    assert!(output.resolved.source_statement_witnesses().is_none());
    assert!(output.resolved.source_proof_local_declaration().is_none());
    assert!(output.resolved.source_proof_local_let_binding().is_none());
    assert!(output.resolved.expr_metadata().is_empty());
    assert!(output.resolved.collection_candidates().is_empty());
    assert!(output.resolved.expanded_candidates().is_empty());
    assert!(output.resolved.template_expansions().is_empty());
    assert!(output.resolved.viable_candidates().is_empty());
    assert!(output.resolved.viability_decisions().is_empty());
    assert!(output.resolved.specificity_graphs().is_empty());
    assert!(output.resolved.resolved_overloads().is_empty());
    assert!(output.resolved.inserted_coercions().is_empty());
    assert!(output.resolved.cluster_facts().is_empty());
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
    assert!(output.resolved.diagnostics().is_empty());
    for forbidden in [
        "initial-obligation#",
        "fact#",
        "terminal-goal#",
        "accepted",
        "discharged",
        "verification-condition",
    ] {
        assert!(!output.resolved.debug_text().contains(forbidden));
    }
}

#[test]
fn task269ct_near_miss_task269c_and_active_routes_remain_isolated() {
    let near_misses = [
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let z be set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y, z be set;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace(
            "let y be set;",
            "let y be set such that x = x;",
        ),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("let y be set;", "let y be set by A;"),
        SOURCE_PROOF_LOCAL_LET_TEXT.replace("thus x = x;", "thus y = y;"),
    ];
    for (ordinal, source) in near_misses.into_iter().enumerate() {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(&source, 269_740 + ordinal);
        assert!(
            source_proof_local_let_type_output(&ast, module, &shells, &symbols, &source).is_none(),
            "Task269CT selected near miss {ordinal}",
        );
    }
    for (ordinal, source) in [TASK269A_SOURCE_TEXT, TASK269B_SOURCE_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, 269_750 + ordinal);
        assert!(
            source_proof_local_let_type_output(&ast, module, &shells, &symbols, source).is_none(),
            "Task269CT selected Task269A/B family {ordinal}",
        );
    }
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PROOF_LOCAL_LET_TEXT, 269_760);
    let task269c = source_proof_local_let_binding_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PROOF_LOCAL_LET_TEXT,
    )
    .expect("Task269C remains selected")
    .expect("Task269C remains valid");
    assert_eq!(
        task269c
            .typed_ast
            .source_proof_local_let_binding()
            .expect("Task269C direct owner")
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(1))
            .expect("Task269C local binding")
            .type_site,
        mizar_checker::binding_env::BindingTypeSite::Missing
    );
    assert!(
        source_proof_local_declaration_output(
            &ast,
            module,
            &symbols,
            SOURCE_PROOF_LOCAL_LET_TEXT,
        )
        .is_none()
    );
}
