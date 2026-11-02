#[test]
fn task258b3m1_real_frontend_freezes_mixed_multiple_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M1_TEXT.len(), 113);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M1_TEXT),
        "412a6a7f8fddebd67418f3482855ea89a1e7da922b42ebb93463971d8e49c186"
    );
    assert!(SOURCE_STATEMENT_B3M1_TEXT.ends_with('\n'));
    let (ast, module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_800);
    assert_eq!((ast.nodes().len(), ast.root().unwrap().index()), (56, 55));
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    assert!(symbols.labels().is_empty());
    assert_eq!((labels.projections().len(), resolution.index().len()), (1, 1));
    let label = &labels.projections()[0];
    assert_eq!(label.kind(), mizar_resolve::resolved_ast::LabelKind::Theorem);
    assert_eq!(label.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(label.export_status(), mizar_resolve::env::ExportStatus::Exported);
    assert_eq!(label.contribution().index(), 0);
    assert_eq!(label.origin().structural_path(), [2, 1]);
    assert!(symbols.imports().is_empty());
    assert!(symbols.symbols().visible_candidates(&mizar_resolve::env::NamespacePath::new(module.path().as_str()), "y").is_empty());
    let (typed_ast, resolved) = task258_native_output(&ast, &module);
    let statement = typed_ast.source_statement().expect("statement");
    let primary = typed_ast.source_term().expect("primary");
    let atomic = typed_ast.source_atomic_formula().expect("atomic");
    let handoff = typed_ast
        .source_statement_witnesses()
        .expect("witness handoff");
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
            primary.terms().len(),
            primary.references().len(),
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (2, 1, 0, 6, 6, 2, 4, 4)
    );
    let proof = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement {
            source_range: mizar_session::SourceRange {
                source_id: ast.source_id,
                start: 71,
                end: 111,
            },
        }
    );
    for (index, ((_, term), (_, reference))) in primary
        .terms()
        .iter()
        .zip(primary.references().iter())
        .enumerate()
    {
        assert_eq!(term.site().node().index(), [30, 32, 36, 39, 43, 45][index]);
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context().index(), usize::from(index >= 2));
        assert_eq!(term.spelling(), "x");
        assert_eq!(reference.term().index(), index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(reference.use_ordinal(), scope.references()[index].ordinal());
    }
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term) => {
                    term.index()
                }
                _ => usize::MAX,
            })
            .collect::<Vec<_>>(),
        [0, 1, 4, 5]
    );
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            handoff.witnesses().len(),
            handoff.names().len(),
        ),
        (1, 2, 2, 2, 2, 2, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("owner");
    assert_eq!(
        (
            owner.site().node().index(),
            owner.source_range().start,
            owner.source_range().end,
            owner.spelling(),
            owner.contribution().index(),
        ),
        (52, 19, 112, "FormulaStatementMultipleWitnessSmoke", 0)
    );
    for index in 0..2 {
        let row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(
                index,
            ))
            .expect("statement row");
        assert_eq!(
            (
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
            ),
            (
                [52, 50][index],
                [19, 96][index],
                [112, 107][index],
                [0, 2][index],
            )
        );
        let fact = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(
            fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![4, 5] }
        );
    }
    for index in 0..2 {
        let witness = handoff
            .witnesses()
            .get(mizar_checker::source_statement::SourceStatementWitnessId::new(index))
            .expect("witness");
        assert_eq!(
            (
                witness.owner().index(),
                witness.binding_context().index(),
                witness.term(),
                witness.take_site().node().index(),
                witness.take_range().start,
                witness.take_range().end,
                witness.site().node().index(),
                witness.source_range().start,
                witness.source_range().end,
            ),
            (
                0,
                1,
                mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2 + index)
                ),
                42,
                79,
                93,
                [38, 41][index],
                [84, 91][index],
                [89, 92][index],
            )
        );
        assert_eq!(
            (
                witness.source_ordinal(),
                witness.ordinal(),
                witness.spelling(),
                witness.kind(),
                witness.name().map(|id| id.index()),
            ),
            (
                1,
                index,
                ["y = x", "x"][index],
                [
                    mizar_checker::source_statement::SourceStatementWitnessKind::Named,
                    mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed,
                ][index],
                [Some(0), None][index],
            )
        );
    }
    let name = handoff
        .names()
        .get(mizar_checker::source_statement::SourceStatementWitnessNameId::new(0))
        .expect("name");
    assert_eq!(name.witness().index(), 0);
    assert_eq!(
        (
            name.site().node().index(),
            name.source_range().start,
            name.source_range().end,
            name.spelling(),
        ),
        (13, 84, 85, "y")
    );
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(primary.references().iter().map(|(_, row)| row.use_ordinal()).collect::<Vec<_>>(), [0, 1, 2, 3, 4, 5]);
    assert_eq!(primary.terms().iter().map(|(_, term)| (term.source_range().start, term.source_range().end)).collect::<Vec<_>>(), [(65,66), (69,70), (88,89), (91,92), (101,102), (105,106)]);
    assert_eq!(atomic.formulas().iter().map(|(_, formula)| formula.site().node().index()).collect::<Vec<_>>(), [34,47]);
    assert!(typed_ast.source_proof_local_declaration().is_none());
    assert!(resolved.source_proof_local_declaration().is_none());
    for index in 0..56 {
        let surface = ast.nodes().get(index).expect("surface node");
        let typed = typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("typed node");
        assert_eq!(
            typed.anchor,
            mizar_session::SourceAnchor::Range(surface.range)
        );
        assert_eq!(
            typed
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            typed.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
    }
    assert_eq!(
        typed_ast.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        typed_ast.source_statement_witnesses(),
        resolved.source_statement_witnesses()
    );
    assert!(typed_ast.source_statement_references().is_none());
}

#[test]
fn task258b3m1_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_801);
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    assert!(symbols.labels().is_empty(), "native declaration environment has no legacy label rows");
    assert_eq!((labels.projections().len(), resolution.index().len()), (1, 1));
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).expect("pristine native lower baseline");
    let baseline = task258_native_output(&ast, &module);
    let installed = base.clone().with_source_statement_from_original(&source, &symbols, &scope).expect("pristine original installation");
    assert_eq!(installed, baseline.0);
    assert_eq!(assemble_empty_resolved_typed_ast(&installed, Vec::new()).unwrap(), baseline.1);
    let owner = installed.source_statement().unwrap().checked_owner().symbol();
    let mut missing_owner = mizar_resolve::env::SymbolIndex::new();
    for entry in symbols.symbols().iter() {
        if entry.symbol() != owner { missing_owner.insert(entry.clone()); }
    }
    let before = (ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone(), base.clone());
    // The historical mixed-row and bundle corruption duties are preserved at the private producers.
    // These controls alter the two actual native environment representations.
    for control in ["missing owner symbol", "unexpected label enrichment"] {
        let mutated = SymbolEnv::new(module.clone(), mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: if control == "missing owner symbol" { missing_owner.clone() } else { symbols.symbols().clone() },
            labels: if control == "unexpected label enrichment" { resolution.index().clone() } else { symbols.labels().clone() },
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        });
        assert_ne!(mutated, symbols, "{control} changes an actual native field");
        let mutated_before = mutated.clone();
        assert_eq!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &mutated, &scope).unwrap_err(), "source symbol environment does not match its declaration collection", "{control} original authentication");
        assert_eq!(base.clone().with_source_statement_from_original(&source, &mutated, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement), "{control} original installation");
        assert_eq!(mutated, mutated_before, "{control} passed dependency purity");
        assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone(), base.clone()), before, "{control} pristine dependency purity");
        assert_eq!(task258_native_output(&ast, &module), baseline, "{control} pristine native retry");
    }
}

#[test]
fn task258b3m1_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_803);
    let baseline = task258_native_output(&ast, &module);
    assert_eq!((baseline.0.nodes().len(), baseline.0.source_term().unwrap().references().len()), (56, 6));
    assert!(source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M1_TEXT).is_none());
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3M1_TEXT.trim_end_matches('\n').to_owned(),
        format!("{SOURCE_STATEMENT_B3M1_TEXT}\n"),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("  take y = x, x;", "   take y = x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(
            "FormulaStatementMultipleWitnessSmoke",
            "FormulaStatementMultipleWitnessNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "byte near miss {ordinal}"
        );
        let (coherent_ast, coherent_module, _, _) = task253_ast_from_source_text(&source, 258_860 + ordinal);
        let (typed, resolved) = task258_native_output(&coherent_ast, &coherent_module);
        let witnesses = typed.source_statement_witnesses().unwrap();
        assert_eq!((witnesses.witnesses().len(), witnesses.names().len()), (2, 1));
        assert_eq!(resolved.source_statement_witnesses(), Some(witnesses));
        assert!(typed.source_proof_local_declaration().is_none());
        assert!(resolved.source_proof_local_declaration().is_none());
        assert_eq!(task258_native_output(&ast, &module), baseline);
    }
    let reordered = SOURCE_STATEMENT_B3M1_TEXT.replacen(
        "  take y = x, x;\n  thus x = x;\n",
        "  thus x = x;\n  take y = x, x;\n",
        1,
    );
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3_TEXT.to_owned(),
        SOURCE_STATEMENT_B3N_TEXT.to_owned(),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("reserve x for set;", "reserve z for set;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take x, y = x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x, x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x, y = x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take z = x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y <> x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = {x}, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("  take y = x, x;\n", "", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("  take y = x, x;\n", "  take y = x;\n  take x;\n", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("  thus x = x;\n", "  thus x = x;\n  thus x = x;\n", 1),
        reordered,
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x, x", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(": x = x proof", ": ex z being set st z = z proof", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(": x = x proof", ": (x = x) & (x = x) proof", 1),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, _, near_symbols) =
            task253_ast_from_source_text(&source, 258_810 + ordinal);
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
            )
            .is_none(),
            "subtree route near miss {ordinal}"
        );
        if [0, 1, 5, 8].contains(&ordinal) {
            let (typed, resolved) = task258_native_output(&near_ast, &near_module);
            let witnesses = typed.source_statement_witnesses().unwrap();
            assert_eq!(witnesses.witnesses().len(), if ordinal == 8 { 2 } else { 1 });
            assert_eq!(witnesses.names().len(), usize::from(ordinal != 0));
            assert_eq!(resolved.source_statement_witnesses(), Some(witnesses));
            assert!(typed.source_proof_local_declaration().is_none());
            assert!(resolved.source_proof_local_declaration().is_none());
        } else if ordinal == 11 {
            let (original, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(&near_ast, &near_module).unwrap().unwrap();
            let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&original, &env).unwrap();
            assert_eq!(scope.references().len(), 4);
            let before = (original.clone(), env.clone(), scope.clone(), labels.clone(), resolution.clone());
            assert_eq!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&original, &env, &scope).unwrap_err(), "theorems.equality.unsupported_source_types");
            assert_eq!((original, env, scope, labels, resolution), before);
        }
        assert_eq!(task258_native_output(&ast, &module), baseline);
    }
}

#[test]
fn task258b3m1_family_and_active_route_isolation_is_atomic_in_both_orders() {
    let (ast, module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_804);
    let (typed_ast, _) = task258_native_output(&ast, &module);
    let statement = typed_ast.source_statement().expect("B3M1 statement").clone();
    let witnesses = typed_ast.source_statement_witnesses().expect("B3M1 witnesses").clone();
    let baseline_debug = typed_ast.debug_text();
    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
        SOURCE_STATEMENT_B3_TEXT,
        SOURCE_STATEMENT_B3N_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (foreign_ast, foreign_module, _, _) = task253_ast_from_source_text(source, 258_830 + ordinal);
        let (foreign_typed, _) = task258_native_output(&foreign_ast, &foreign_module);
        let foreign_debug = foreign_typed.debug_text();
        assert_eq!(
            foreign_typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
            "foreign-first family {ordinal}"
        );
        assert_eq!(foreign_typed.debug_text(), foreign_debug);
        if let Some(references) = foreign_typed.source_statement_references() {
            assert_eq!(
                typed_ast.clone().with_source_statement_references(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    references.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M1-first reference family {ordinal}"
            );
        } else if let Some(foreign_witnesses) = foreign_typed.source_statement_witnesses() {
            assert_eq!(
                typed_ast.clone().with_source_statement_witnesses(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    foreign_witnesses.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M1-first witness family {ordinal}"
            );
        } else {
            assert_eq!(
                typed_ast.clone().with_source_statement(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M1-first base family {ordinal}"
            );
        }
        assert_eq!(typed_ast.debug_text(), baseline_debug);
        assert_eq!(task258_native_output(&ast, &module).0, typed_ast);
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
    let plan = build_test_plan(&config).expect("Task258B3M1 isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(active_ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &active_ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let active_symbols =
            augment_type_elaboration_import_summaries(&active_ast, &resolver.module, resolver.env);
        let selected_route = source_statement_output_with_source(
            &active_ast,
            resolver.module.clone(),
            &active_symbols,
            &source,
        )
        .is_some_and(|result| {
            result.is_ok_and(|output| {
                output
                    .typed_ast
                    .source_statement_witnesses()
                    .is_some_and(|handoff| {
                        handoff.witnesses().len() == 2 && handoff.names().len() == 1
                    })
            })
        });
        if selected_route {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M1 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_805);
    let (typed, resolved) = task258_native_output(&ast, &module);
    assert_eq!(
        typed.clone().debug_text(),
        typed.debug_text()
    );
    assert_eq!(
        resolved.clone().debug_text(),
        resolved.debug_text()
    );
    assert_eq!(
        typed.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        typed.source_statement_witnesses(),
        resolved.source_statement_witnesses()
    );
    assert!(typed.source_statement_references().is_none());
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
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    let baseline_debug = typed.debug_text();
    assert_eq!(typed.clone().with_source_statement_witnesses(
        typed.source_statement().unwrap().clone(), typed.source_statement_witnesses().unwrap().clone(),
    ), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert!(typed.source_proof_local_declaration().is_none());
    assert!(resolved.source_proof_local_declaration().is_none());
    assert!(typed.contexts().is_empty());
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert_eq!(typed.debug_text(), baseline_debug);
    assert_eq!(task258_native_output(&ast, &module), (typed, resolved));
}
