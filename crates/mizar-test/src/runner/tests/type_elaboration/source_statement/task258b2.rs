#[test]
fn task258b2_real_frontend_freezes_single_assumption_transport() {
    assert_eq!(SOURCE_STATEMENT_B2_TEXT.len(), 113);
    assert!(SOURCE_STATEMENT_B2_TEXT.ends_with('\n'));
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_500);
    let extracted: SourceStatementB2Extraction =
        extract_single_assumption_source_statement(&ast, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 exact parser shape");
    assert_eq!(ast.nodes().len(), 55);
    assert_eq!(ast.root().expect("root").index(), 54);
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [51, 41, 49]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 112), (80, 93), (96, 107)]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [32, 38, 46]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(66, 71), (87, 92), (101, 106)]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [28, 30, 34, 36, 42, 44]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (66, 67),
            (70, 71),
            (87, 88),
            (91, 92),
            (101, 102),
            (105, 106)
        ]
    );
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (51, 19, 112, 27, 64, 72, 111)
    );

    let (typed, resolved) = task258_native_output(&ast, &module);
    let statement = typed.source_statement().expect("Task258B2 statement");
    assert_eq!(statement.statements().len(), 3);
    assert_eq!(
        statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(1))
            .expect("assumption row")
            .kind(),
        mizar_checker::source_statement::SourceStatementKind::Assumption
    );
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
        ),
        (2, 1, 0, 1, 3, 3, 3, 3)
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.context().index(),
                row.kind(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                51,
                0,
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
            ),
            (
                41,
                1,
                mizar_checker::source_statement::SourceStatementKind::Assumption,
                "assume x = x ;",
            ),
            (
                49,
                2,
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                "thus x = x ;",
            ),
        ]
    );
    assert_eq!(
        typed
            .source_term()
            .expect("Task252 handoff")
            .references()
            .iter()
            .map(|(_, row)| row.use_ordinal())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5]
    );
    assert_eq!(
        typed
            .source_atomic_formula()
            .expect("Task256 handoff")
            .formulas()
            .iter()
            .map(|(_, row)| row.context().index())
            .collect::<Vec<_>>(),
        [0, 1, 1]
    );
    let binding = statement.binding_env().bindings().get(mizar_checker::binding_env::BindingId::new(0)).expect("module reservation");
    assert_eq!(binding.owner_context.index(), 0);
    assert_eq!(binding.kind, mizar_checker::binding_env::BindingKind::ReservedVariable);
    assert_eq!(binding.type_site, mizar_checker::binding_env::BindingTypeSite::Missing);
    assert_eq!(binding.status, mizar_checker::binding_env::BindingStatus::Active);
    assert!(matches!(binding.identity, mizar_checker::binding_env::BinderIdentity::ReservedVariable { .. }));
    let contexts = statement.binding_env().contexts().iter().map(|(_, context)| context).collect::<Vec<_>>();
    assert_eq!(contexts[0].lexical_scope, None);
    assert_eq!(contexts[1].lexical_scope.as_ref().expect("actual proof scope").path(), [4]);
    assert_eq!(typed.source_term().unwrap().references().iter().map(|(_, reference)| reference.lexical_scope().unwrap().path()).collect::<Vec<_>>(), [&[][..], &[][..], &[4][..], &[4][..], &[4][..], &[4][..]]);
    assert_eq!(contexts[0].parent, None);
    assert_eq!(contexts[1].parent, Some(contexts[0].id));
    assert!(contexts[1].bindings.is_empty());
    assert_eq!(contexts[1].visible_bindings, [binding.id]);
    assert_eq!(statement.contexts().iter().map(|(_, row)| row.binding_context().index()).collect::<Vec<_>>(), [0, 1, 1]);
    assert_eq!(statement.input_facts().iter().map(|(_, row)| row.uses().iter().map(|reference| reference.index()).collect::<Vec<_>>()).collect::<Vec<_>>(), [vec![0, 1], vec![2, 3], vec![4, 5]]);
    assert!(statement.input_facts().iter().all(|(_, row)| row.kind() == mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard));
    assert!(statement.candidate_facts().iter().all(|(_, row)| row.kind() == mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition));
    assert!(typed.source_statement_references().is_none());
    assert_eq!(
        typed.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(typed.nodes().len(), ast.nodes().len());
    assert_eq!(
        typed.nodes().root().map(|root| root.index()),
        ast.root().map(|root| root.index())
    );
    for (id, node) in typed.nodes().iter() {
        let surface = &ast.nodes()[id.index()];
        let expected_kind = match surface.kind {
            mizar_syntax::ast::SurfaceNodeKind::TermReference => "source.term.variable-reference".to_owned(),
            mizar_syntax::ast::SurfaceNodeKind::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
            _ => format!("{:?}", surface.kind),
        };
        assert_eq!(node.kind.as_str(), expected_kind, "node {} kind", id.index());
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(surface.range),
            "node {} range",
            id.index()
        );
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            "node {} children",
            id.index()
        );
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal,
            "node {} recovery",
            id.index()
        );
    }
}

#[test]
fn task258b2_lower_resolver_and_row_mutations_fail_closed_then_replay() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_510);
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
    // The eight historical label-row faults are preserved at the private B2 producer.
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
        assert!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &mutated, &scope).is_err(), "{control} original authentication");
        assert_eq!(base.clone().with_source_statement_from_original(&source, &mutated, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement), "{control} original installation");
        assert_eq!(mutated, mutated_before, "{control} passed dependency purity");
        assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone(), base.clone()), before, "{control} pristine dependency purity");
        assert_eq!(task258_native_output(&ast, &module), baseline, "{control} pristine native retry");
    }
}

#[test]
fn task258b2_selector_rejects_exact_assumption_near_misses() {
    let (exact_ast, exact_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_520);
    let baseline = task258_native_output(&exact_ast, &exact_module);
    let capture = |source: &str, ordinal: usize| {
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let package_root = std::env::temp_dir().join(format!("mizar-test-b2-native-selector-{}-{ordinal}-{unique}", std::process::id()));
        let source_path = package_root.join("src").join("selector.miz");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(&source_path, source).unwrap();
        let package = PackageId::new("mizar-test-b2-native-selector");
        let module_path = ModulePath::new(format!("tests.b2_native_selector_{ordinal}"));
        let normalized_path = mizar_session::normalize_path(&package_root, &source_path).unwrap();
        let frontend = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(mizar_session::DiskSourceLoader::new(&package_root)),
            ParseOnlyImportProvider, mizar_frontend::parsing::MizarParserSeam,
        );
        let output = frontend.run(mizar_frontend::source::SourceUnitRequest {
            snapshot: super::shared::snapshot_id(258_600 + ordinal),
            input: mizar_session::SourceInput {
                package_id: package.clone(), module_path: module_path.clone(), normalized_path,
                edition: Edition::new("2026"), origin: mizar_session::SourceOriginInput::Disk { path: source_path },
            },
        }, &mizar_session::InMemorySessionIdAllocator::new()).unwrap();
        std::fs::remove_dir_all(&package_root).unwrap();
        (output, ResolverModuleId::new(package, module_path))
    };
    for (label, loaded) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B2_TEXT.trim_end_matches('\n').to_owned(),
        ),
        ("extra final LF", format!("{SOURCE_STATEMENT_B2_TEXT}\n")),
        (
            "whitespace byte drift",
            SOURCE_STATEMENT_B2_TEXT.replacen("  assume", " assume", 1),
        ),
        (
            "comment byte drift",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "reserve x for set;",
                "reserve x for set; :: drift",
                1,
            ),
        ),
        (
            "theorem name drift",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "FormulaStatementSingleAssumptionSmoke",
                "FormulaStatementSingleAssumptionOther",
                1,
            ),
        ),
    ] {
        assert!(extract_single_assumption_source_statement(&exact_ast, &loaded).is_none(), "{label} exact extractor guard");
        let (frontend, module) = capture(&loaded, 0);
        assert!(frontend.diagnostics.is_empty(), "{label} coherent native capture");
        let ast = frontend.ast.expect("coherent source requires AST");
        let (typed, resolved) = task258_native_output(&ast, &module);
        assert_eq!(typed.source_statement().unwrap().statements().len(), 3, "{label}");
        assert!(typed.source_statement_references().is_none(), "{label}");
        assert!(resolved.source_statement_references().is_none(), "{label}");
        assert_eq!(typed.source_statement(), resolved.source_statement(), "{label}");
        assert_eq!(task258_native_output(&exact_ast, &exact_module), baseline, "{label} pristine native retry");
    }
    for (ordinal, (label, source)) in [
        (
            "labeled assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("  assume", "  A: assume", 1),
        ),
        (
            "collective assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x", "assume that x = x", 1),
        ),
        (
            "given statement",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x", "given y being set", 1),
        ),
        (
            "consider statement",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "assume x = x",
                "consider y being set such that x = x",
                1,
            ),
        ),
        (
            "witness statement",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x;", "take x;", 1),
        ),
        (
            "then assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x", "then assume x = x", 1),
        ),
        (
            "hence conclusion",
            SOURCE_STATEMENT_B2_TEXT.replacen("thus x = x", "hence x = x", 1),
        ),
        (
            "nested proof",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "  assume x = x;",
                "  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "reordered statements",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "  assume x = x;\n  thus x = x;",
                "  thus x = x;\n  assume x = x;",
                1,
            ),
        ),
        (
            "composite assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x;", "assume x = x & x = x;", 1),
        ),
        (
            "extra statement",
            SOURCE_STATEMENT_B2_TEXT.replacen("  thus x = x;", "  assume x = x;\n  thus x = x;", 1),
        ),
        (
            "recovered assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x;", "assume x = x", 1),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(extract_single_assumption_source_statement(&exact_ast, &source).is_none(), "{label} exact AST");
        let (frontend, module) = capture(&source, ordinal + 1);
        if label == "witness statement" {
            assert!(frontend.diagnostics.is_empty(), "{label} coherent witness capture");
            let (typed, _) = task258_native_output(frontend.ast.as_ref().expect("native witness AST"), &module);
            assert_eq!(typed.source_statement().unwrap().statements().len(), 2);
            assert_eq!(typed.source_statement_witnesses().unwrap().witnesses().len(), 1);
        }
        if let Some(ast) = frontend.ast {
            assert!(extract_single_assumption_source_statement(&ast, &source).is_none(), "{label} parsed AST");
            assert!(extract_single_assumption_source_statement(&ast, SOURCE_STATEMENT_B2_TEXT).is_none(), "{label} exact guard with wrong AST");
            match mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module) {
                Err(error) => assert_eq!(error, mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module).unwrap_err().to_string(), "{label} source lowering refusal"),
                Ok(None) => {
                    let shells = mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module).collect();
                    let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(&ast, &shells, mizar_resolve::env::NamespacePath::new(module.path().as_str())).collect(&module);
                    assert!(!collected.diagnostics().is_empty(), "{label} declaration collection refusal");
                }
                Ok(Some((original, symbols))) => match mizar_resolve::symbols::source_proof_inputs(&original, &symbols) {
                    Err(error) => assert_eq!(mizar_resolve::symbols::source_proof_inputs(&original, &symbols), Err(error), "{label} resolver refusal replay"),
                    Ok((scope, labels, resolution)) => {
                        mizar_resolve::symbols::validate_source_proof_occurrences(&original, &symbols, &scope).unwrap();
                        let before = (original.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone());
                        let base = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&original, &symbols, &scope);
                        if label == "witness statement" {
                            assert!(base.is_ok(), "{label} native witness base");
                        } else {
                            assert!(base.is_err(), "{label} bounded native refusal");
                        }
                        assert_eq!((original, symbols, scope, labels, resolution), before, "{label} dependency purity");
                    }
                },
            }
        } else {
            assert!(!frontend.diagnostics.is_empty(), "{label} absent AST requires Frontend diagnostics");
        }
        assert_eq!(task258_native_output(&exact_ast, &exact_module), baseline, "{label} pristine native retry");
    }
}

#[test]
fn task258b2_keeps_task258a_task258b1_and_active_routes_isolated() {
    for (ordinal, source) in [TASK258A_SOURCE, SOURCE_STATEMENT_B1_TEXT, SOURCE_STATEMENT_B5A_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, _, _) = task253_ast_from_source_text(source, 258_540 + ordinal);
        assert!(
            extract_single_assumption_source_statement(&ast, source).is_none(),
            "cross-profile selector"
        );
        let (output_typed, _) = task258_native_output(&ast, &module);
        assert!(
            output_typed
                .source_statement()
                .is_some_and(|statement| {
                    statement.statements().len() != 3
                        || statement
                            .statements()
                            .get(mizar_checker::source_statement::SourceStatementId::new(1))
                            .is_none_or(|row| {
                                row.kind()
                                != mizar_checker::source_statement::SourceStatementKind::Assumption
                            })
                })
        );
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
    let plan = build_test_plan(&config).expect("Task258B2 isolation plan");
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
        if extract_single_assumption_source_statement(&ast, &source).is_some()
            || source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
                .is_some_and(|result| {
                    result.is_ok_and(|output| {
                        output
                            .typed_ast
                            .source_statement()
                            .is_some_and(|statement| {
                                statement.statements().len() == 3
                                    && statement
                                        .statements()
                                        .get(
                                            mizar_checker::source_statement::SourceStatementId::new(
                                                1,
                                            ),
                                        )
                                        .is_some_and(|row| {
                                            row.kind()
                                                == mizar_checker::source_statement::SourceStatementKind::Assumption
                                        })
                            })
                    })
                })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B2 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b2_typed_final_ownership_clone_and_empty_semantics_are_atomic() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_550);
    let (first, first_resolved) = task258_native_output(&ast, &module);
    let (second, second_resolved) = task258_native_output(&ast, &module);
    assert_eq!(first, first.clone());
    assert_eq!(first_resolved, first_resolved.clone());
    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first_resolved.debug_text(), second_resolved.debug_text());
    let debug = first.debug_text();
    let primary = debug.find("source-primary-term-debug-v1").expect("primary");
    let atomic = debug
        .find("source-atomic-formula-debug-v1")
        .expect("atomic");
    let statement = debug.find("source-statement-debug-v1").expect("statement");
    let nodes = debug.find("nodes:").expect("nodes");
    assert!(primary < atomic && atomic < statement && statement < nodes);
    assert!(first.source_statement_references().is_none());
    assert!(first_resolved.source_statement_references().is_none());
    assert!(first.types().is_empty());
    assert!(first.facts().is_empty());
    assert!(first.coercions().is_empty());
    assert!(first.initial_obligations().is_empty());
    assert!(first.diagnostics().is_empty());
    assert!(first_resolved.expr_metadata().is_empty());
    assert!(first_resolved.cluster_facts().is_empty());
    assert!(first_resolved.diagnostics().is_empty());
    assert!(first_resolved.checked_formulas().is_empty());
    assert!(first_resolved.statement_semantics().is_empty());
    assert!(first_resolved.checked_proofs().is_empty());
    assert!(first_resolved.checked_proof_nodes().is_empty());
    assert!(first_resolved.checked_terminal_goals().is_empty());

    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    let statement = first
        .source_statement()
        .expect("Task258B2 statement")
        .clone();
    assert_eq!(
        task248.typed_ast.clone().with_source_statement(statement),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
}
