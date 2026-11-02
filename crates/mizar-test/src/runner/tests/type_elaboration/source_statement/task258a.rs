#[test]
fn task258a_real_frontend_publishes_exact_statement_provenance_and_empty_semantics() {
    assert_eq!(TASK258A_SOURCE.len(), 81);
    assert!(TASK258A_SOURCE.ends_with('\n'));
    assert_eq!(TASK258A_SOURCE, SOURCE_STATEMENT_TEXT);
    let (ast, module, _, symbols) = task253_ast_from_source_text(TASK258A_SOURCE, 258_000);
    let extracted: SourceStatementExtraction = extract_source_reserved_variable_theorem_statement(
        &ast,
        module.clone(),
        &symbols,
        TASK258A_SOURCE,
    )
    .expect("Task258A exact source must select");
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
        .unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    let (native_bindings, _, native_base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(
        &source, &symbols, &scope,
    ).unwrap();
    let source_before = (source.clone(), symbols.clone(), scope.clone(), native_base.clone());
    assert_eq!(
        (
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.payload.formula_range.start,
            extracted.payload.formula_range.end,
            extracted.payload.left_range.start,
            extracted.payload.left_range.end,
            extracted.payload.right_range.start,
            extracted.payload.right_range.end,
        ),
        (19, 80, 27, 72, 74, 79, 74, 75, 78, 79)
    );
    assert_eq!(extracted.label_spelling, TASK258A_LABEL);
    assert_eq!(extracted.statement_spelling, TASK258A_STATEMENT_SPELLING);
    assert_eq!(
        (
            extracted.payload.left_spelling.as_str(),
            extracted.payload.right_spelling.as_str(),
        ),
        ("x", "x")
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let theorem_candidates = symbols
        .symbols()
        .visible_candidates(&namespace, TASK258A_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [theorem_candidate] = theorem_candidates.as_slice() else {
        panic!(
            "Task258A requires one real resolver theorem candidate, got {}",
            theorem_candidates.len()
        );
    };
    let prechecked_owner =
        mizar_checker::type_checker::CheckedStatementOwner::validate_exact_local_theorem(
            &symbols,
            theorem_candidate.symbol().clone(),
            ast.source_id,
            &module,
        )
        .expect("Task258A real resolver theorem must validate");
    assert_eq!(prechecked_owner.source_range(), extracted.theorem_range);

    let (first, first_resolved) = task258_native_output(&ast, &module);
    let (second, second_resolved) = task258_native_output(&ast, &module);
    assert_eq!(first.nodes(), native_base.nodes());
    assert_eq!(first.source_term(), native_base.source_term());
    assert_eq!(first.source_atomic_formula(), native_base.source_atomic_formula());
    assert_eq!(first.source_statement().unwrap().binding_env(), &native_bindings);
    assert_eq!((source.clone(), symbols.clone(), scope.clone(), native_base.clone()), source_before);

    let primary = first.source_term().expect("Task252 handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (2, 2, 0)
    );
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, term)| (
                id.index(),
                term.source_range().start,
                term.source_range().end,
                term.source_ordinal(),
                term.context().index(),
                term.kind(),
                term.role(),
                term.recovery(),
                term.parent(),
                term.spelling().to_owned(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                74,
                75,
                0,
                0,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                None,
                "x".to_owned(),
            ),
            (
                1,
                78,
                79,
                1,
                0,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                None,
                "x".to_owned(),
            ),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(id, reference)| (
                id.index(),
                reference.term().index(),
                reference.binding().index(),
                reference.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [(0, 0, 0, 0), (1, 1, 0, 1)]
    );

    for ((_, reference), occurrence) in primary.references().iter().zip(scope.references()) {
        assert_eq!(reference.use_ordinal(), occurrence.ordinal());
    }

    let atomic = first
        .source_atomic_formula()
        .expect("Task256 handoff");
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (1, 0, 0, 0, 0, 0, 0, 2, 2)
    );
    let formula = atomic
        .formulas()
        .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0))
        .expect("Task258A equality");
    assert_eq!(
        (
            formula.source_range().start,
            formula.source_range().end,
            formula.source_ordinal(),
            formula.context().index(),
            formula.kind(),
            formula.recovery(),
            formula.spelling(),
        ),
        (
            74,
            79,
            0,
            0,
            mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
            mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
            "x = x",
        )
    );
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(id, edge)| (
                id.index(),
                edge.formula().index(),
                edge.ordinal(),
                edge.role(),
                edge.target(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(0),
                ),
            ),
            (
                1,
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(1),
                ),
            ),
        ]
    );
    assert_eq!(
        atomic
            .requests()
            .iter()
            .map(|(id, request)| (
                id.index(),
                request.formula().index(),
                request.ordinal(),
                request.kind(),
                request.edge().map(|id| id.index()),
                request.candidate(),
                request.type_site(),
                request.attribute(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(0),
                None,
                None,
                None,
            ),
            (
                1,
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(1),
                None,
                None,
                None,
            ),
        ]
    );

    let statement = first
        .source_statement()
        .expect("Task258A statement handoff");
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
        ),
        (1, 1, 1, 1, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("Task258A owner row");
    assert_eq!(owner.source_range(), extracted.theorem_range);
    assert_eq!(owner.site(), &extracted.theorem_site);
    assert_eq!(owner.spelling(), TASK258A_LABEL);
    assert_eq!(
        owner.role(),
        mizar_checker::source_statement::SourceTheoremRole::Theorem
    );
    assert_eq!(
        owner.status(),
        mizar_checker::source_statement::SourceTheoremStatus::Unmodified
    );
    assert_eq!(
        owner.recovery(),
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    assert_eq!(statement.checked_owner().symbol(), owner.symbol());
    assert_eq!(
        statement.checked_owner().source_range(),
        owner.source_range()
    );
    let statement_row = statement
        .statements()
        .get(mizar_checker::source_statement::SourceStatementId::new(0))
        .expect("Task258A statement row");
    assert_eq!(statement_row.owner().index(), 0);
    assert_eq!(statement_row.context().index(), 0);
    assert_eq!(statement_row.site(), &extracted.theorem_site);
    assert_eq!(statement_row.source_range(), extracted.theorem_range);
    assert_eq!(statement_row.source_ordinal(), 0);
    assert_eq!(statement_row.spelling(), TASK258A_STATEMENT_SPELLING);
    assert_eq!(
        statement_row.kind(),
        mizar_checker::source_statement::SourceStatementKind::TheoremProposition
    );
    assert_eq!(
        statement_row.recovery(),
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    assert_eq!(
        statement_row.formula(),
        mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
        )
    );
    let context = statement
        .contexts()
        .get(mizar_checker::source_statement::SourceStatementContextId::new(0))
        .expect("Task258A context row");
    assert_eq!(context.statement().index(), 0);
    assert_eq!(context.binding_context().index(), 0);
    assert_eq!(context.source_range(), extracted.theorem_range);
    assert_eq!(
        context
            .visible_bindings()
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>(),
        [0]
    );
    let input_fact = statement
        .input_facts()
        .get(mizar_checker::source_statement::SourceStatementInputFactId::new(0))
        .expect("Task258A input fact");
    assert_eq!(input_fact.statement().index(), 0);
    assert_eq!(input_fact.context().index(), 0);
    assert_eq!(input_fact.ordinal(), 0);
    assert_eq!(input_fact.binding().index(), 0);
    assert_eq!(
        input_fact.kind(),
        mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard
    );
    assert_eq!(
        input_fact
            .uses()
            .iter()
            .map(|reference| reference.index())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let candidate = statement
        .candidate_facts()
        .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(0))
        .expect("Task258A candidate fact");
    assert_eq!(candidate.statement().index(), 0);
    assert_eq!(candidate.context().index(), 0);
    assert_eq!(candidate.ordinal(), 0);
    assert_eq!(
        candidate.kind(),
        mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
    );
    assert_eq!(candidate.formula(), statement_row.formula());
    let resolver_owner = symbols
        .symbols()
        .get(owner.symbol())
        .expect("Task258A resolver symbol");
    assert_eq!(
        resolver_owner.kind(),
        mizar_resolve::env::SymbolKind::Theorem
    );
    assert_eq!(
        resolver_owner.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        resolver_owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(resolver_owner.contribution(), owner.contribution());
    let definition = symbols
        .definitions()
        .by_symbol(owner.symbol())
        .expect("Task258A theorem definition");
    assert_eq!(
        definition.kind(),
        mizar_resolve::env::DefinitionKind::Theorem
    );
    assert_eq!(definition.contribution(), owner.contribution());
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .expect("Task258A local contribution");
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::LocalSource { .. }
    ));
    assert!(contribution.effects().symbols().contains(owner.symbol()));
    assert!(
        contribution
            .effects()
            .definitions()
            .contains(&definition.id())
    );
    assert!(contribution.effects().imports().is_empty());
    let recollected = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
        .unwrap().unwrap().1;
    assert_eq!(symbols.labels(), recollected.labels());

    assert_eq!(statement.binding_env().bindings().len(), 1);
    assert_eq!(statement.binding_env().contexts().len(), 1);
    assert!(statement.binding_env().diagnostics().is_empty());
    let binding = statement
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("Task258A reserved binding");
    assert_eq!(binding.spelling, "x");
    assert_eq!(
        binding.kind,
        mizar_checker::binding_env::BindingKind::ReservedVariable
    );
    assert_eq!(
        binding.status,
        mizar_checker::binding_env::BindingStatus::Active
    );
    assert_eq!(binding.owner_context.index(), 0);
    assert_eq!(
        (
            binding.declaration_range.start,
            binding.declaration_range.end,
            binding.visible_after_ordinal,
        ),
        (8, 9, 0)
    );
    assert_eq!(binding.type_site, mizar_checker::binding_env::BindingTypeSite::Missing);
    let written_type = ast.node_views().find(|view| matches!(view.kind(), mizar_syntax::SurfaceNodeKind::Token(token) if token.text.as_ref() == "set")).unwrap();
    assert_eq!((written_type.range().start, written_type.range().end), (14, 17));

    assert_eq!(
        statement.binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    assert_eq!(statement.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(statement.atomic_formula_fingerprint(), atomic.debug_text());
    assert_eq!(
        first.source_statement(),
        first_resolved.source_statement()
    );
    assert_eq!(first.source_term(), first_resolved.source_term());
    assert_eq!(
        first.source_atomic_formula(),
        first_resolved.source_atomic_formula()
    );
    assert!(first.source_context().is_none());
    assert!(first.types().is_empty());
    assert!(first.facts().is_empty());
    assert!(first.diagnostics().is_empty());
    assert!(first_resolved.expr_metadata().is_empty());
    assert!(first_resolved.cluster_facts().is_empty());
    assert!(first_resolved.diagnostics().is_empty());
    assert!(first_resolved.checked_formulas().is_empty());
    assert!(first_resolved.statement_semantics().is_empty());
    assert!(first_resolved.checked_proofs().is_empty());
    assert!(first_resolved.checked_proof_nodes().is_empty());
    assert!(first_resolved.checked_terminal_goals().is_empty());
    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first_resolved.debug_text(), second_resolved.debug_text());
    let typed_debug = first.debug_text();
    let primary_at = typed_debug
        .find("source-primary-term-debug-v1")
        .expect("typed Task252 debug");
    let atomic_at = typed_debug
        .find("source-atomic-formula-debug-v1")
        .expect("typed Task256 debug");
    let statement_at = typed_debug
        .find("source-statement-debug-v1")
        .expect("typed Task258A debug");
    let nodes_at = typed_debug.find("nodes:").expect("typed node debug");
    assert!(primary_at < atomic_at && atomic_at < statement_at && statement_at < nodes_at);
    let typed_clone = first.clone();
    assert_eq!(typed_clone, first);
    assert_eq!(typed_clone.source_statement(), Some(statement));
    let resolved_clone = first_resolved.clone();
    assert_eq!(resolved_clone, first_resolved);
    assert_eq!(resolved_clone.source_statement(), Some(statement));
}

#[test]
fn task258a_dependency_and_row_corruption_fail_atomically_then_replay() {
    let (ast, module, _, raw_symbols) = task253_ast_from_source_text(TASK258A_SOURCE, 258_100);
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap();
    let baseline = task258_native_output(&ast, &module);
    let before = (ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), base.clone());
    let extracted = extract_source_reserved_variable_theorem_statement(&ast, module.clone(), &raw_symbols, TASK258A_SOURCE).unwrap();
    let enriched = source_statement_resolver_env_for_test(&module, &raw_symbols, &extracted).unwrap();
    let imported = statement_env_with_imported_label(enriched.clone());
    assert_ne!(enriched, symbols);
    assert_ne!(imported, symbols);
    let (foreign_ast, foreign_module, _, _) = task253_ast_from_source_text(TASK258A_SOURCE, 258_101);
    let (foreign_source, foreign_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&foreign_ast, &foreign_module).unwrap().unwrap();
    let (foreign_scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&foreign_source, &foreign_symbols).unwrap();
    for (current, env, occurrences) in [
        (&source, &enriched, &scope),
        (&source, &imported, &scope),
        (&foreign_source, &symbols, &scope),
        (&source, &foreign_symbols, &scope),
        (&source, &symbols, &foreign_scope),
    ] {
        assert_eq!(base.clone().with_source_statement_from_original(current, env, occurrences),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
        assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), base.clone()), before);
        assert_eq!(task258_native_output(&ast, &module), baseline);
    }
    let (lower_ast, lower_module, lower_symbols) = task252_real_ast("pass_type_elaboration_formula_predicate_chain_segment_payload_001");
    let lower = source_atomic_formula_output_with_source(&lower_ast, lower_module, &lower_symbols, TASK258A_LOWER_SOURCE).unwrap().unwrap();
    assert_eq!(
        lower.typed_ast.with_source_statement_from_original(&source, &symbols, &scope),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
    );
    assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), base.clone()), before);
    assert_eq!(task258_native_output(&ast, &module), baseline);
}

#[test]
fn task258a_selector_rejects_loaded_named_recovered_subtree_and_active_near_misses() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(TASK258A_SOURCE, 258_200);
    for (label, loaded_source) in [
        (
            "missing final LF",
            TASK258A_SOURCE.trim_end_matches('\n').to_owned(),
        ),
        (
            "loaded source has an extra final LF",
            format!("{TASK258A_SOURCE}\n"),
        ),
        (
            "loaded source has byte-different whitespace",
            TASK258A_SOURCE.replacen(": x = x;", ":  x = x;", 1),
        ),
    ] {
        assert!(
            extract_source_reserved_variable_theorem_statement(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none(),
            "{label}"
        );
    }

    for (ordinal, source) in [
        TASK258A_SOURCE.trim_end_matches('\n').to_owned(),
        format!("{TASK258A_SOURCE}\n"),
        TASK258A_SOURCE.replacen(": x = x;", ":  x = x;", 1),
        TASK258A_SOURCE.replace("reserve x", "reserve\tx").replace("theorem ", ":: layout comment\ntheorem "),
        TASK258A_SOURCE.replacen(TASK258A_LABEL, "FormulaStatementReservedVariableEqualityNearMiss", 1),
    ].into_iter().enumerate() {
        let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(&source, 258_205 + ordinal);
        assert_eq!(diagnostics, 0);
        if source.contains("FormulaStatementReservedVariableEqualityNearMiss") {
            assert!(extract_source_reserved_variable_theorem_statement(&ast, module.clone(), &symbols, TASK258A_SOURCE).is_none(),
                "renamed capture must refuse unrelated historical raw source");
        }
        let (typed, resolved) = task258_native_output(&ast, &module);
        assert_eq!(typed.source_statement(), resolved.source_statement());
    }
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&exact_ast, &exact_module).unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap();
    let baseline = task258_native_output(&exact_ast, &exact_module);

    for (ordinal, (label, source)) in [
        (
            "recovered equality",
            TASK258A_SOURCE.replacen("x = x;", "x = ;", 1),
        ),
        (
            "parenthesized formula subtree",
            TASK258A_SOURCE.replacen("x = x;", "(x = x);", 1),
        ),
        (
            "composite formula subtree",
            TASK258A_SOURCE.replacen("x = x;", "(x = x) & (x = x);", 1),
        ),
        (
            "multiple theorem items",
            format!("{TASK258A_SOURCE}theorem ExtraStatement: x = x;\n"),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(&source, 258_210 + ordinal);
        assert!(
            extract_source_reserved_variable_theorem_statement(&ast, module.clone(), &symbols, &source).is_none(),
            "{label} must fail its historical selector"
        );
        assert!(
            extract_source_reserved_variable_theorem_statement(&ast, module.clone(), &symbols, TASK258A_SOURCE).is_none(),
            "{label} must also fail the mixed AST/source identity guard"
        );
        let (current, current_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
        let current_scope = mizar_resolve::symbols::source_proof_inputs(&current, &current_symbols)
            .map(|(occurrences, _, _)| occurrences);
        let occurrences = current_scope.as_ref().unwrap_or(&scope);
        let lower = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&current, &current_symbols, occurrences);
        let candidate = lower.as_ref().map(|(_, _, typed)| typed).unwrap_or(&base);
        assert_eq!(candidate.clone().with_source_statement_from_original(&current, &current_symbols, occurrences),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement), "{label}");
        if current_scope.is_err() {
            assert!(mizar_resolve::symbols::validate_source_proof_occurrences(&current, &current_symbols, &scope).is_err());
            assert!(lower.is_err());
        }
        assert_eq!(task258_native_output(&exact_ast, &exact_module), baseline);
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task258A isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_statement_output_with_source(&ast, resolver.module, &symbols, &source).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "dormant Task258A route must not select an active type-elaboration case: {selected:?}"
    );
}

#[test]
fn task258a_typed_resolved_ownership_and_task248_first_exclusion_are_atomic() {
    let (ast, module, _, _) = task253_ast_from_source_text(TASK258A_SOURCE, 258_300);
    let (output, resolved) = task258_native_output(&ast, &module);
    let statement = output
        .source_statement()
        .expect("Task258A typed owner")
        .clone();
    assert_eq!(
        output.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        output.source_term(),
        resolved.source_term()
    );
    assert_eq!(
        output.source_atomic_formula(),
        resolved.source_atomic_formula()
    );

    let task248 = task248_real_output();
    assert!(task248.typed_ast.source_context().is_some());
    assert!(task248.typed_ast.source_statement().is_none());
    let before = task248.typed_ast.debug_text();
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement(statement)
            .expect_err("Task248-first/Task258A-second must fail"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
    assert!(task248.typed_ast.source_statement().is_none());
    assert_eq!(
        task248.typed_ast.source_context(),
        task248.resolved.source_context()
    );
    assert_eq!(task248.typed_ast.debug_text(), before);

    let (replay, replay_resolved) = task258_native_output(&ast, &module);
    assert_eq!(replay.debug_text(), output.debug_text());
    assert_eq!(replay_resolved.debug_text(), resolved.debug_text());
}
