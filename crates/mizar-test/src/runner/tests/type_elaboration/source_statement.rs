use super::{
    SOURCE_STATEMENT_B1_TEXT, SOURCE_STATEMENT_B2_TEXT, SOURCE_STATEMENT_TEXT,
    SourceStatementB1Extraction, SourceStatementB2Extraction, SourceStatementExtraction,
    SourceStatementRouteInputs, SourceStatementRouteOutput, extract_nested_source_statement,
    extract_single_assumption_source_statement,
    extract_source_reserved_variable_theorem_statement,
    source_statement_b1_output_with_mutation, source_statement_b2_output_with_mutation,
    source_statement_b2_output_with_resolver_mutation, source_statement_output_with_source,
    source_statement_output_with_source_and_mutation, source_statement_output_with_resolver_mutation,
    source_statement_resolver_env_for_test,
};

const TASK258A_SOURCE: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementReservedVariableEqualitySmoke: x = x;\n",
);
const TASK258A_LABEL: &str = "FormulaStatementReservedVariableEqualitySmoke";
const TASK258A_STATEMENT_SPELLING: &str =
    "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;";
const TASK258A_LOWER_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "theorem FormulaPredicateChainPayloadBoundary: ",
    "1 divides 2 does not divides 3;\n",
);

#[test]
fn task258b2_real_frontend_freezes_single_assumption_transport() {
    assert_eq!(SOURCE_STATEMENT_B2_TEXT.len(), 113);
    assert!(SOURCE_STATEMENT_B2_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_500);
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
        [(66, 67), (70, 71), (87, 88), (91, 92), (101, 102), (105, 106)]
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

    let output =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 selector")
            .unwrap_or_else(|error| panic!("Task258B2 route failed: {error}"));
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258B2 statement");
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
        output
            .typed_ast
            .source_term()
            .expect("Task252 handoff")
            .references()
            .iter()
            .map(|(_, row)| row.use_ordinal())
            .collect::<Vec<_>>(),
        [1; 6]
    );
    assert_eq!(
        output
            .typed_ast
            .source_atomic_formula()
            .expect("Task256 handoff")
            .formulas()
            .iter()
            .map(|(_, row)| row.context().index())
            .collect::<Vec<_>>(),
        [0, 1, 1]
    );
    assert_eq!(output.reference_use_ordinals, [1; 6]);
    assert!(output.typed_ast.source_statement_references().is_none());
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(output.typed_ast.nodes().len(), ast.nodes().len());
    assert_eq!(
        output.typed_ast.nodes().root().map(|root| root.index()),
        ast.root().map(|root| root.index())
    );
    for (id, node) in output.typed_ast.nodes().iter() {
        let surface = &ast.nodes()[id.index()];
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
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_510);
    let baseline =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 selector")
            .expect("Task258B2 baseline");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let (b1_ast, b1_module, _, b1_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_511);
    let b1 = source_statement_output_with_source(
        &b1_ast,
        b1_module,
        &b1_symbols,
        SOURCE_STATEMENT_B1_TEXT,
    )
    .expect("Task258B1 selector")
    .expect("Task258B1 output");
    let wrong_binding = b1
        .typed_ast
        .source_statement()
        .expect("Task258B1 statement")
        .binding_env()
        .clone();
    let wrong_primary = b1
        .typed_ast
        .source_term()
        .expect("Task258B1 Task252")
        .clone();
    let wrong_atomic = b1
        .typed_ast
        .source_atomic_formula()
        .expect("Task258B1 Task256")
        .clone();
    for (label, expected, mutate) in [
        ("statement aggregate", "aggregate", 0usize),
        ("assumption kind", "statement", 1usize),
        ("assumption context", "context", 2usize),
        ("assumption input", "input fact", 3usize),
        ("assumption candidate", "candidate", 4usize),
        ("cross-formula edge", "statement", 5usize),
        ("theorem owner range", "owner", 6usize),
        ("recovered assumption", "statement", 7usize),
        ("cross-profile binding", "dependency", 8usize),
        ("cross-profile primary", "dependency", 9usize),
        ("cross-profile atomic", "dependency", 10usize),
    ] {
        let error = source_statement_b2_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
            |inputs| match mutate {
                0 => inputs.statement.candidate_facts.pop().map(drop).unwrap(),
                1 => {
                    inputs.statement.statements[1].kind =
                        mizar_checker::source_statement::SourceStatementKind::Conclusion;
                }
                2 => {
                    inputs.statement.contexts[1].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0);
                }
                3 => inputs.statement.input_facts[1].uses.swap(0, 1),
                4 => {
                    inputs.statement.candidate_facts[1].formula =
                        mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                        );
                }
                5 => {
                    inputs.statement.statements[1].formula =
                        mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                        );
                }
                6 => {
                    inputs.statement.owners[0].source_range.end -= 1;
                }
                7 => {
                    inputs.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        inputs.arena.root(),
                        inputs
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == 41 {
                                    row.recovery =
                                        mizar_checker::typed_ast::NodeRecoveryState::Recovered;
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("recovered arena remains structurally valid");
                }
                8 => inputs.binding_env = wrong_binding.clone(),
                9 => inputs.primary = wrong_primary.clone(),
                10 => inputs.atomic = wrong_atomic.clone(),
                _ => unreachable!(),
            },
        )
        .unwrap_or_else(|| panic!("{label} selector"))
        .expect_err(label);
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "{label}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
        )
        .expect("replay selector")
        .expect("replay output");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed, "{label}");
        assert_eq!(replay.resolved.debug_text(), baseline_resolved, "{label}");
    }

    for mutation in [
        Task258B2ResolverMutation::Imported,
        Task258B2ResolverMutation::Missing,
        Task258B2ResolverMutation::Duplicate,
        Task258B2ResolverMutation::WrongPath,
        Task258B2ResolverMutation::WrongKind,
        Task258B2ResolverMutation::Private,
        Task258B2ResolverMutation::LocalOnly,
        Task258B2ResolverMutation::Recovered,
    ] {
        let resolver_error = source_statement_b2_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("Task258B2 resolver selector")
        .expect_err("resolver provenance mutation must fail");
        assert!(
            resolver_error.to_ascii_lowercase().contains("provenance")
                || resolver_error.to_ascii_lowercase().contains("owner")
                || resolver_error.to_ascii_lowercase().contains("label"),
            "{mutation:?}: {resolver_error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
        )
        .expect("resolver replay selector")
        .expect("resolver replay output");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline_typed,
            "{mutation:?}"
        );
        assert_eq!(
            replay.resolved.debug_text(),
            baseline_resolved,
            "{mutation:?}"
        );
    }
}

#[test]
fn task258b2_selector_rejects_exact_assumption_near_misses() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_520);
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
            SOURCE_STATEMENT_B2_TEXT.replacen("reserve x for set;", "reserve x for set; :: drift", 1),
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
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded,
            )
            .is_none(),
            "{label}"
        );
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
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "  thus x = x;",
                "  assume x = x;\n  thus x = x;",
                1,
            ),
        ),
        (
            "recovered assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x;", "assume x = x", 1),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) =
            task253_ast_from_source_text(&source, 258_521 + ordinal);
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "{label}"
        );
        assert!(
            source_statement_output_with_source(
                &ast,
                module,
                &symbols,
                SOURCE_STATEMENT_B2_TEXT,
            )
            .is_none(),
            "{label} exact guard with wrong AST"
        );
    }
}

#[test]
fn task258b2_keeps_task258a_task258b1_and_active_routes_isolated() {
    for (ordinal, source) in [TASK258A_SOURCE, SOURCE_STATEMENT_B1_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(source, 258_540 + ordinal);
        assert!(
            extract_single_assumption_source_statement(&ast, source).is_none(),
            "cross-profile selector"
        );
        let output = source_statement_output_with_source(&ast, module, &symbols, source)
            .expect("prior profile selector")
            .expect("prior profile output");
        assert!(
            output
                .typed_ast
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
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_550);
    let first =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 selector")
            .expect("Task258B2 output");
    let second =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 replay selector")
            .expect("Task258B2 replay output");
    assert_eq!(first.typed_ast, first.typed_ast.clone());
    assert_eq!(first.resolved, first.resolved.clone());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let debug = first.typed_ast.debug_text();
    let primary = debug.find("source-primary-term-debug-v1").expect("primary");
    let atomic = debug.find("source-atomic-formula-debug-v1").expect("atomic");
    let statement = debug.find("source-statement-debug-v1").expect("statement");
    let nodes = debug.find("nodes:").expect("nodes");
    assert!(primary < atomic && atomic < statement && statement < nodes);
    assert!(first.typed_ast.source_statement_references().is_none());
    assert!(first.resolved.source_statement_references().is_none());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());

    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    let statement = first
        .typed_ast
        .source_statement()
        .expect("Task258B2 statement")
        .clone();
    assert_eq!(
        task248.typed_ast.clone().with_source_statement(statement),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
}

#[test]
fn task258b1_real_frontend_freezes_nested_statement_and_resolver_bundle() {
    assert_eq!(SOURCE_STATEMENT_B1_TEXT.len(), 139);
    assert!(SOURCE_STATEMENT_B1_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_400);
    let extracted: SourceStatementB1Extraction =
        extract_nested_source_statement(&ast, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 exact parser shape");
    assert_eq!(ast.nodes().len(), 77);
    assert_eq!(ast.root().expect("root").index(), 76);
    assert_eq!(extracted.theorem_site.node().index(), 73);
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [73, 60, 58, 71]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 138), (77, 114), (96, 107), (117, 133)]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(63, 68), (80, 85), (101, 106), (122, 127)]
    );
    assert_eq!((extracted.theorem_range.start, extracted.theorem_range.end), (19, 138));
    assert_eq!((extracted.label_range.start, extracted.label_range.end), (27, 61));
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [42, 48, 55, 65]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [38, 40, 44, 46, 51, 53, 61, 63]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (63, 64),
            (67, 68),
            (80, 81),
            (84, 85),
            (101, 102),
            (105, 106),
            (122, 123),
            (126, 127),
        ]
    );
    assert_eq!(
        extracted
            .proof_ranges
            .map(|range| (range.start, range.end)),
        [(69, 137), (86, 113)]
    );
    let output =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 selector")
            .unwrap_or_else(|error| panic!("Task258B1 route failed: {error}"));
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258B1 statement");
    let references = output
        .typed_ast
        .source_statement_references()
        .expect("Task258B1 references");
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            references.labels().len(),
            references.citations().len(),
        ),
        (1, 4, 4, 4, 4, 1, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(0))
        .expect("theorem owner");
    assert_eq!(owner.site().node().index(), 73);
    assert_eq!((owner.source_range().start, owner.source_range().end), (19, 138));
    assert_eq!(owner.spelling(), "FormulaStatementNestedContextSmoke");
    assert_eq!(
        (
            owner.role(),
            owner.status(),
            owner.recovery(),
        ),
        (
            mizar_checker::source_statement::SourceTheoremRole::Theorem,
            mizar_checker::source_statement::SourceTheoremStatus::Unmodified,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.owner().index(),
                row.context().index(),
                row.formula(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
                row.recovery(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
                ),
                73,
                19,
                138,
                0,
                "theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;",
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                1,
                0,
                1,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1),
                ),
                60,
                77,
                114,
                1,
                "A : x = x proof thus x = x ; end ;",
                mizar_checker::source_statement::SourceStatementKind::ProofStepProposition,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                2,
                0,
                2,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                ),
                58,
                96,
                107,
                2,
                "thus x = x ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                3,
                0,
                3,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(3),
                ),
                71,
                117,
                133,
                3,
                "thus x = x by A ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
        ]
    );
    assert_eq!(
        statement
            .contexts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.binding_context().index(),
                row.source_range().start,
                row.source_range().end,
                row.visible_bindings()
                    .iter()
                    .map(|binding| binding.index())
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, 19, 138, vec![0]),
            (1, 1, 1, 77, 114, vec![0]),
            (2, 2, 2, 96, 107, vec![0]),
            (3, 3, 1, 117, 133, vec![0]),
        ]
    );
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.context().index(),
                row.ordinal(),
                row.kind(),
                row.binding().index(),
                row.uses()
                    .iter()
                    .map(|reference| reference.index())
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![0, 1],
            ),
            (
                1,
                1,
                1,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![2, 3],
            ),
            (
                2,
                2,
                2,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![4, 5],
            ),
            (
                3,
                3,
                3,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![6, 7],
            ),
        ]
    );
    assert_eq!(
        statement
            .candidate_facts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.context().index(),
                row.ordinal(),
                row.kind(),
                row.formula(),
            ))
            .collect::<Vec<_>>(),
        (0..4)
            .map(|index| (
                index,
                index,
                index,
                0,
                mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
                ),
            ))
            .collect::<Vec<_>>()
    );
    let binding_contexts = statement.binding_env().contexts();
    assert_eq!(binding_contexts.len(), 3);
    let module_context = binding_contexts
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("module context");
    assert_eq!(module_context.owner, mizar_checker::binding_env::BindingContextOwner::Module);
    assert_eq!(module_context.parent, None);
    assert_eq!(
        module_context.layer,
        mizar_checker::binding_env::BindingContextLayer::Module
    );
    assert_eq!(module_context.bindings, [mizar_checker::binding_env::BindingId::new(0)]);
    assert_eq!(
        module_context.visible_bindings,
        [mizar_checker::binding_env::BindingId::new(0)]
    );
    assert_eq!(
        module_context.recovery,
        mizar_checker::binding_env::BindingContextRecovery::Normal
    );
    for (id, parent, scope, range) in [
        (1, 0, &[0][..], (69, 137)),
        (2, 1, &[0, 0][..], (86, 113)),
    ] {
        let context = binding_contexts
            .get(mizar_checker::binding_env::BindingContextId::new(id))
            .expect("proof context");
        assert_eq!(
            context.parent.map(|parent| parent.index()),
            Some(parent)
        );
        assert_eq!(
            context.layer,
            mizar_checker::binding_env::BindingContextLayer::Proof
        );
        assert_eq!(
            context
                .lexical_scope
                .as_ref()
                .expect("proof scope")
                .path(),
            scope
        );
        assert_eq!(
            context.owner,
            mizar_checker::binding_env::BindingContextOwner::SourceStatement {
                source_range: mizar_session::SourceRange {
                    source_id: ast.source_id,
                    start: range.0,
                    end: range.1,
                },
            }
        );
        assert_eq!(context.visible_bindings, [mizar_checker::binding_env::BindingId::new(0)]);
        assert!(context.bindings.is_empty());
        assert_eq!(
            context.recovery,
            mizar_checker::binding_env::BindingContextRecovery::Normal
        );
    }
    assert_eq!(statement.binding_env().bindings().len(), 1);
    assert!(statement.binding_env().diagnostics().is_empty());
    let binding = statement
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("reserved binding");
    assert_eq!(
        (
            binding.id.index(),
            binding.spelling.as_str(),
            binding.kind,
            binding.owner_context.index(),
            binding.declaration_range.start,
            binding.declaration_range.end,
            binding.visible_after_ordinal,
            binding.status,
            binding.recovery,
        ),
        (
            0,
            "x",
            mizar_checker::binding_env::BindingKind::ReservedVariable,
            0,
            8,
            9,
            0,
            mizar_checker::binding_env::BindingStatus::Reserved,
            mizar_checker::binding_env::BindingRecoveryState::Normal,
        )
    );
    assert!(matches!(
        binding.identity,
        mizar_checker::binding_env::BinderIdentity::ReservedVariable { .. }
    ));
    assert!(matches!(
        binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(range)
            if (range.start, range.end) == (14, 17)
    ));
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(
        statement.binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    let primary = output.typed_ast.source_term().expect("Task252 terms");
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, term)| (
                id.index(),
                term.site().node().index(),
                term.context().index(),
                term.source_range().start,
                term.source_range().end,
                term.source_ordinal(),
                term.recovery(),
                term.spelling(),
                term.kind(),
                term.role(),
                term.parent().map(|parent| parent.index()),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 38, 0, 63, 64, 0, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (1, 40, 0, 67, 68, 1, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (2, 44, 1, 80, 81, 2, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (3, 46, 1, 84, 85, 3, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (4, 51, 2, 101, 102, 4, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (5, 53, 2, 105, 106, 5, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (6, 61, 1, 122, 123, 6, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (7, 63, 1, 126, 127, 7, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
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
                reference
                    .lexical_scope()
                    .map(|scope| scope.path().to_vec()),
                reference.use_ordinal(),
                reference.role(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, None, 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (1, 1, 0, None, 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (2, 2, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (3, 3, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (4, 4, 0, Some(vec![0, 0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (5, 5, 0, Some(vec![0, 0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (6, 6, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (7, 7, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
        ]
    );
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 atomics");
    assert_eq!((atomic.formulas().len(), atomic.edges().len(), atomic.requests().len()), (4, 8, 8));
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(id, formula)| (
                id.index(),
                formula.site().node().index(),
                formula.context().index(),
                formula.source_range().start,
                formula.source_range().end,
                formula.source_ordinal(),
                formula.recovery(),
                formula.spelling(),
                formula.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 42, 0, 63, 68, 0, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
            (1, 48, 1, 80, 85, 1, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
            (2, 55, 2, 101, 106, 2, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
            (3, 65, 1, 122, 127, 3, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
        ]
    );
    for (id, edge) in atomic.edges().iter() {
        assert_eq!(edge.formula().index(), id.index() / 2);
        assert_eq!(edge.ordinal(), id.index() % 2);
        assert_eq!(
            edge.role(),
            if id.index() % 2 == 0 {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
        assert!(matches!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term)
                if term.index() == id.index()
        ));
    }
    for (id, request) in atomic.requests().iter() {
        assert_eq!(request.formula().index(), id.index() / 2);
        assert_eq!(request.ordinal(), id.index() % 2);
        assert_eq!(
            request.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
        );
        assert_eq!(request.edge().map(|edge| edge.index()), Some(id.index()));
        assert_eq!(request.candidate(), None);
        assert_eq!(request.type_site(), None);
        assert_eq!(request.attribute(), None);
    }
    assert_eq!(output.reference_use_ordinals, [1; 8]);
    assert_eq!(output.typed_ast.nodes().len(), 77);
    assert_eq!(
        output.typed_ast.nodes().root().map(|root| root.index()),
        Some(76)
    );
    for (id, node) in output.typed_ast.nodes().iter() {
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range)
        );
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            ast.nodes()[id.index()]
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
    }
    assert_eq!(references.resolver_ast().nodes().len(), 77);
    assert_eq!(references.resolver_ast().nodes().root().index(), 76);
    for (id, node) in references.resolver_ast().nodes().iter() {
        assert_eq!(node.kind(), &ast.nodes()[id.index()].kind);
        assert_eq!(
            node.children()
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            ast.nodes()[id.index()]
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(node.origin().source_id(), ast.source_id);
        assert_eq!(node.origin().module_id(), output.typed_ast.module_id());
        assert_eq!(node.origin().import_edge(), None);
        assert_eq!(node.origin().structural_path(), [id.index() as u32]);
        assert_eq!(
            node.origin().anchor(),
            &mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range)
        );
        assert!(!node.origin().is_recovered());
        assert_eq!(
            node.recovery(),
            mizar_resolve::resolved_ast::RecoveryState::Normal
        );
        if id.index() == 68 {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::Resolved
            );
            assert!(matches!(
                node.reference_key(),
                Some(mizar_resolve::resolved_ast::NodeReferenceKey::Label(id))
                    if id.index() == 0
            ));
        } else {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::NotApplicable
            );
            assert_eq!(node.reference_key(), None);
        }
    }
    let projection = references.label_projection();
    assert_eq!(projection.origin().structural_path(), [12]);
    assert_eq!((projection.declaration_range().start, projection.declaration_range().end), (77, 78));
    assert_eq!(projection.primary_spelling(), "A");
    assert_eq!(projection.module(), output.typed_ast.module_id());
    assert_eq!(projection.namespace().as_str(), output.typed_ast.module_id().path().as_str());
    assert_eq!(
        projection.kind(),
        mizar_resolve::resolved_ast::LabelKind::ProofStep
    );
    assert_eq!(
        projection.visibility(),
        mizar_resolve::env::Visibility::Private
    );
    assert_eq!(
        projection.export_status(),
        mizar_resolve::env::ExportStatus::LocalOnly
    );
    assert_eq!(projection.contribution(), owner.contribution());
    assert_eq!(projection.origin().source_id(), ast.source_id);
    assert_eq!(projection.origin().module_id(), output.typed_ast.module_id());
    assert_eq!(projection.origin().import_edge(), None);
    assert_eq!(
        projection.origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 77,
            end: 78,
        })
    );
    assert!(!projection.origin().is_recovered());
    assert!(matches!(
        projection.source(),
        mizar_resolve::labels::LabelProjectionSource::CurrentModule {
            visible_after_ordinal: 1,
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(references.reference_candidate().site().node().index(), 68);
    assert_eq!(
        (
            references.reference_candidate().site().range().start,
            references.reference_candidate().site().range().end,
            references.reference_candidate().site().spelling(),
            references.reference_candidate().ordinal(),
            references.reference_candidate().expectation(),
        ),
        (
            131,
            132,
            "A",
            3,
            mizar_resolve::resolved_ast::LabelExpectation::ProofOrTheorem,
        )
    );
    assert_eq!(references.reference_candidate().origin().source_id(), ast.source_id);
    assert_eq!(
        references.reference_candidate().origin().module_id(),
        output.typed_ast.module_id()
    );
    assert_eq!(references.reference_candidate().origin().import_edge(), None);
    assert_eq!(
        references.reference_candidate().origin().structural_path(),
        [68]
    );
    assert_eq!(
        references.reference_candidate().origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 131,
            end: 132,
        })
    );
    assert!(!references.reference_candidate().origin().is_recovered());
    assert!(matches!(
        references.reference_candidate().scope(),
        mizar_resolve::labels::LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(references.label_resolution().ids().len(), 1);
    assert!(references.label_resolution().diagnostics().is_empty());
    assert_eq!(
        references.resolver_ast().label_refs(),
        references.label_resolution().table()
    );
    let label_ref = references
        .label_resolution()
        .table()
        .get(references.label_resolution().ids()[0])
        .expect("resolved local label");
    assert_eq!(
        (
            label_ref.site().node().index(),
            label_ref.site().range().start,
            label_ref.site().range().end,
            label_ref.site().spelling(),
            label_ref.origin().source_id(),
            label_ref.origin().structural_path(),
            label_ref.recovery(),
        ),
        (
            68,
            131,
            132,
            "A",
            ast.source_id,
            &[68][..],
            mizar_resolve::resolved_ast::RecoveryState::Normal,
        )
    );
    assert_eq!(label_ref.origin().module_id(), output.typed_ast.module_id());
    assert_eq!(label_ref.origin().import_edge(), None);
    assert!(!label_ref.origin().is_recovered());
    assert!(matches!(
        label_ref.resolution(),
        mizar_resolve::resolved_ast::LabelResolution::Resolved(row)
            if row.origin() == projection.origin_path()
                && row.kind() == mizar_resolve::resolved_ast::LabelKind::ProofStep
                && (row.range().start, row.range().end) == (131, 132)
    ));
    let label = references
        .labels()
        .get(mizar_checker::source_statement::SourceStatementLabelId::new(0))
        .expect("statement label");
    assert_eq!(
        (
            label.statement().index(),
            label.context().index(),
            label.candidate().index(),
            label.origin_path(),
            label.proof_scope().path(),
            label.source_range().start,
            label.source_range().end,
            label.source_ordinal(),
            label.visible_after_ordinal(),
            label.spelling(),
            label.kind(),
            label.recovery(),
        ),
        (
            1,
            1,
            1,
            projection.origin_path(),
            &[0][..],
            77,
            78,
            0,
            1,
            "A",
            mizar_checker::source_statement::SourceStatementLabelKind::ProofStep,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    let citation = references
        .citations()
        .get(mizar_checker::source_statement::SourceStatementCitationId::new(0))
        .expect("statement citation");
    assert_eq!(
        (
            citation.statement().index(),
            citation.context().index(),
            citation.label().index(),
            citation.label_ref().index(),
            citation.proof_scope().path(),
            citation.source_range().start,
            citation.source_range().end,
            citation.ordinal(),
            citation.kind(),
            citation.recovery(),
        ),
        (
            3,
            3,
            0,
            0,
            &[0][..],
            131,
            132,
            0,
            mizar_checker::source_statement::SourceStatementCitationKind::SimpleLocal,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        output.typed_ast.source_statement_references(),
        output.resolved.source_statement_references()
    );
    assert!(references
        .debug_text()
        .contains("resolver-ast root=76 nodes=77 name_refs=0 label_refs=1"));
}

#[test]
fn task258b1_lower_resolver_and_row_mutations_fail_closed_then_replay() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_410);
    let baseline =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 selector")
            .expect("Task258B1 baseline");
    let baseline_typed = baseline.typed_ast.debug_text();
    for (label, expected, mutate) in [
        ("statement aggregate", "aggregate", 0usize),
        ("statement row", "statement", 1usize),
        ("stale projection", "dependency", 2usize),
        ("citation row", "citation", 3usize),
        ("owner row", "owner", 4usize),
        ("context row", "context", 5usize),
        ("input row", "input fact", 6usize),
        ("candidate row", "candidate", 7usize),
        ("label row", "label", 8usize),
        ("stale reference expectation", "dependency", 9usize),
        ("typed recovery", "statement", 10usize),
    ] {
        let error = source_statement_b1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B1_TEXT,
            |inputs| match mutate {
                0 => inputs.statement.contexts.clear(),
                1 => inputs.statement.statements[2].source_ordinal = 9,
                2 => {
                    inputs.projection = inputs
                        .projection
                        .clone()
                        .with_visibility(mizar_resolve::env::Visibility::Public);
                }
                3 => inputs.reference_input.citations[0].ordinal = 1,
                4 => inputs.statement.owners[0].spelling.push_str("Drift"),
                5 => {
                    inputs.statement.contexts[2].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(1);
                }
                6 => inputs.statement.input_facts[2].uses.swap(0, 1),
                7 => inputs.statement.candidate_facts[2].ordinal = 1,
                8 => inputs.reference_input.labels[0].source_ordinal = 1,
                9 => {
                    inputs.reference = inputs
                        .reference
                        .clone()
                        .with_expectation(
                            mizar_resolve::resolved_ast::LabelExpectation::Theorem,
                        );
                }
                10 => {
                    inputs.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        inputs.arena.root(),
                        inputs
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == 50 {
                                    row.recovery =
                                        mizar_checker::typed_ast::NodeRecoveryState::Degraded;
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("recovered arena remains structurally valid");
                }
                _ => unreachable!(),
            },
        )
        .unwrap_or_else(|| panic!("{label} selector"))
        .expect_err(label);
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "{label}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B1_TEXT,
        )
        .expect("replay selector")
        .expect("replay output");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed, "{label}");
    }
}

#[test]
fn task258b1_selector_rejects_exact_nested_statement_near_misses() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_420);
    for (label, loaded) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B1_TEXT.trim_end_matches('\n').to_owned(),
        ),
        (
            "extra final LF",
            format!("{SOURCE_STATEMENT_B1_TEXT}\n"),
        ),
        (
            "whitespace byte drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("  A:", " A:", 1),
        ),
        (
            "comment byte drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set;", "reserve x for set; :: drift", 1),
        ),
        (
            "theorem name drift",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "FormulaStatementNestedContextSmoke",
                "FormulaStatementNestedContextOther",
                1,
            ),
        ),
        (
            "theorem role change",
            SOURCE_STATEMENT_B1_TEXT.replacen("theorem ", "scheme ", 1),
        ),
        (
            "theorem status change",
            SOURCE_STATEMENT_B1_TEXT.replacen("theorem ", "canceled theorem ", 1),
        ),
        (
            "reserve drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set", "reserve y for set", 1),
        ),
        (
            "omitted reserve item",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set;\n", "", 1),
        ),
        (
            "reordered top-level items",
            format!(
                "{}{}",
                &SOURCE_STATEMENT_B1_TEXT[19..],
                &SOURCE_STATEMENT_B1_TEXT[..19]
            ),
        ),
        (
            "omitted nested statement",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;\n", "", 1),
        ),
        (
            "reordered proof statements",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;",
                "  thus x = x by A;\n  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "omitted outer proof block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                " proof\n  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;\nend;",
                ";",
                1,
            ),
        ),
        (
            "reordered proof blocks",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;",
                "  thus x = x by A;\n  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "then statement",
            SOURCE_STATEMENT_B1_TEXT.replacen("  thus x = x by A;", "  then thus x = x by A;", 1),
        ),
        (
            "given statement",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  given y being set such that x = x;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "consider statement",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  consider y being set such that x = x;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "now block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  now\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "hereby block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  hereby\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "case block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  per cases;\n  suppose x = x;\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "suppose block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  suppose x = x;\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "iterative equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x = x;", 1),
        ),
        (
            "imported label citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by Other.A", 1),
        ),
        (
            "local label shadowing",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "    thus x = x;",
                "    A: x = x;\n    thus x = x;",
                1,
            ),
        ),
        (
            "missing nested proof",
            SOURCE_STATEMENT_B1_TEXT.replacen("  A: x = x proof", "  A: x = x", 1),
        ),
        (
            "recovered missing semicolon",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;", "    thus x = x", 1),
        ),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded,
            )
            .is_none(),
            "{label}"
        );
    }
    for (ordinal, (label, source)) in [
        (
            "forward citation",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  x = x by A;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "theorem citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by FormulaStatementNestedContextSmoke", 1),
        ),
        (
            "hence",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x by A", "hence x = x by A", 1),
        ),
        (
            "parenthesized equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus (x = x);", 1),
        ),
        (
            "second label",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "B: thus x = x;", 1),
        ),
        (
            "extra theorem",
            format!("{SOURCE_STATEMENT_B1_TEXT}theorem Extra: x = x;\n"),
        ),
        (
            "assumption",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  A: x = x proof\n    assume x = x;",
                1,
            ),
        ),
        (
            "witness",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "    thus x = x;",
                "    take x;\n    thus x = x;",
                1,
            ),
        ),
        (
            "non-equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x <> x;", 1),
        ),
        (
            "composite formula",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x & x = x;", 1),
        ),
        (
            "qualified citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by Other.A", 1),
        ),
        (
            "citation moved inward",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "    thus x = x;",
                "    thus x = x by A;",
                1,
            ),
        ),
        (
            "explicit justification",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x by A;", 1),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) =
            task253_ast_from_source_text(&source, 258_421 + ordinal);
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "{label}"
        );
        assert!(
            source_statement_output_with_source(
                &ast,
                module,
                &symbols,
                SOURCE_STATEMENT_B1_TEXT,
            )
            .is_none(),
            "{label} exact guard with wrong AST"
        );
    }
}

#[test]
fn task258b1_keeps_task258a_and_active_corpus_routes_isolated() {
    let (a_ast, a_module, _, a_symbols) =
        task253_ast_from_source_text(TASK258A_SOURCE, 258_430);
    let task_a =
        source_statement_output_with_source(&a_ast, a_module, &a_symbols, TASK258A_SOURCE)
            .expect("Task258A selector")
            .expect("Task258A output");
    assert!(task_a.typed_ast.source_statement_references().is_none());
    assert_eq!(task_a.reference_use_ordinals, [1, 2]);

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
    let plan = build_test_plan(&config).expect("Task258B1 isolation plan");
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
        if source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
            .is_some_and(|result| {
                result.is_ok_and(|output| {
                    output
                        .typed_ast
                        .source_statement_references()
                        .is_some()
                })
            })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(selected.is_empty(), "Task258B1 selected active cases: {selected:?}");
}

#[test]
fn task258b1_typed_final_ownership_clone_debug_and_empty_semantics_are_atomic() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_440);
    let first =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 selector")
            .expect("Task258B1 output");
    let second =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 replay selector")
            .expect("Task258B1 replay output");
    assert_eq!(first.typed_ast, first.typed_ast.clone());
    assert_eq!(first.resolved, first.resolved.clone());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let debug = first.typed_ast.debug_text();
    let base = debug.find("source-statement-debug-v1").expect("base debug");
    let references = debug
        .find("source-statement-reference-debug-v1")
        .expect("reference debug");
    let nodes = debug.find("nodes:").expect("node debug");
    assert!(base < references && references < nodes);
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    let statement = first
        .typed_ast
        .source_statement()
        .expect("statement")
        .clone();
    let statement_references = first
        .typed_ast
        .source_statement_references()
        .expect("statement references")
        .clone();
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement(statement.clone())
            .expect_err("Task258B1 base must not use legacy installer"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement_references(statement, statement_references)
            .expect_err("Task248 and Task258B1 owners must not coexist"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
}

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
    let symbols = source_statement_resolver_env_for_test(&module, &symbols, &extracted)
        .expect("Task258A exact label projection must resolve");
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
            extracted.payload.left_lookup_ordinal,
            extracted.payload.right_lookup_ordinal,
        ),
        ("x", "x", 1, 2)
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

    let first: SourceStatementRouteOutput =
        source_statement_output_with_source(&ast, module.clone(), &symbols, TASK258A_SOURCE)
            .expect("Task258A exact selector")
            .unwrap_or_else(|error| panic!("Task258A exact route failed: {error}"));
    let second = source_statement_output_with_source(&ast, module, &symbols, TASK258A_SOURCE)
        .expect("Task258A repeated selector")
        .unwrap_or_else(|error| panic!("Task258A repeated route failed: {error}"));
    assert_eq!(
        (first.left_lookup_ordinal, first.right_lookup_ordinal),
        (1, 2)
    );

    let primary = first.typed_ast.source_term().expect("Task252 handoff");
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
        [(0, 0, 0, 1), (1, 1, 0, 1)]
    );

    let atomic = first
        .typed_ast
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
        .typed_ast
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
    let labels = symbols.labels().by_contribution(owner.contribution());
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].primary_spelling(), TASK258A_LABEL);
    assert_eq!(
        labels[0].kind(),
        mizar_resolve::resolved_ast::LabelKind::Theorem
    );
    assert_eq!(
        labels[0].visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        labels[0].export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );

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
        mizar_checker::binding_env::BindingStatus::Reserved
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
    assert!(matches!(
        binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(range)
            if (range.start, range.end) == (14, 17)
    ));
    assert_eq!(
        statement.binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    assert_eq!(statement.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(statement.atomic_formula_fingerprint(), atomic.debug_text());
    assert_eq!(
        first.typed_ast.source_statement(),
        first.resolved.source_statement()
    );
    assert_eq!(first.typed_ast.source_term(), first.resolved.source_term());
    assert_eq!(
        first.typed_ast.source_atomic_formula(),
        first.resolved.source_atomic_formula()
    );
    assert!(first.typed_ast.source_context().is_none());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let typed_debug = first.typed_ast.debug_text();
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
    let typed_clone = first.typed_ast.clone();
    assert_eq!(typed_clone, first.typed_ast);
    assert_eq!(typed_clone.source_statement(), Some(statement));
    let resolved_clone = first.resolved.clone();
    assert_eq!(resolved_clone, first.resolved);
    assert_eq!(resolved_clone.source_statement(), Some(statement));
}

#[test]
fn task258a_dependency_and_row_corruption_fail_atomically_then_replay() {
    let (ast, module, _, symbols) = task253_ast_from_source_text(TASK258A_SOURCE, 258_100);
    let baseline =
        source_statement_output_with_source(&ast, module.clone(), &symbols, TASK258A_SOURCE)
            .expect("Task258A baseline selector")
            .expect("Task258A baseline output");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let imported_label_error = source_statement_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        TASK258A_SOURCE,
        statement_env_with_imported_label,
    )
    .expect("Task258A imported-label selector")
    .expect_err("imported theorem label provenance must fail");
    assert!(
        imported_label_error
            .to_ascii_lowercase()
            .contains("owner"),
        "unexpected imported-label error: {imported_label_error}"
    );

    let (lower_ast, lower_module, lower_symbols) =
        task252_real_ast("pass_type_elaboration_formula_predicate_chain_segment_payload_001");
    let lower = source_atomic_formula_output_with_source(
        &lower_ast,
        lower_module,
        &lower_symbols,
        TASK258A_LOWER_SOURCE,
    )
    .expect("Task257C1 lower selector")
    .expect("Task257C1 lower profile");
    let wrong_primary = lower
        .typed_ast
        .source_term()
        .expect("Task257C1 primary handoff")
        .clone();
    let wrong_atomic = lower
        .typed_ast
        .source_atomic_formula()
        .expect("Task257C1 atomic handoff")
        .clone();
    let replay = Task258AReplayFixture {
        ast: &ast,
        module,
        symbols: &symbols,
        baseline_typed: &baseline_typed,
        baseline_resolved: &baseline_resolved,
    };

    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "wrong primary profile",
        "dependency",
        move |input| input.primary = wrong_primary,
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "wrong atomic profile",
        "dependency",
        move |input| input.atomic = wrong_atomic,
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "missing owner aggregate",
        "aggregate",
        |input| input.statement.owners.clear(),
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "invalid statement ordinal",
        "statement",
        |input| input.statement.statements[0].source_ordinal = 1,
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "swapped input reference ids",
        "input fact",
        |input| input.statement.input_facts[0].uses.swap(0, 1),
    );
}

#[derive(Debug, Clone, Copy)]
enum Task258B2ResolverMutation {
    Imported,
    Missing,
    Duplicate,
    WrongPath,
    WrongKind,
    Private,
    LocalOnly,
    Recovered,
}

fn task258b2_mutate_resolver(
    symbols: SymbolEnv,
    mutation: Task258B2ResolverMutation,
) -> SymbolEnv {
    if matches!(mutation, Task258B2ResolverMutation::Imported) {
        return statement_env_with_imported_label(symbols);
    }
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258B2 exact theorem label")
        .clone();
    let mut labels = mizar_resolve::env::LabelIndex::new();
    if !matches!(mutation, Task258B2ResolverMutation::Missing) {
        let origin_path = if matches!(mutation, Task258B2ResolverMutation::WrongPath) {
            mizar_resolve::resolved_ast::LabelOriginPath::new(
                "task258b2::wrong::theorem::origin",
            )
        } else {
            label.origin_path().clone()
        };
        let kind = if matches!(mutation, Task258B2ResolverMutation::WrongKind) {
            mizar_resolve::resolved_ast::LabelKind::ProofStep
        } else {
            label.kind()
        };
        let origin = if matches!(mutation, Task258B2ResolverMutation::Recovered) {
            label.origin().clone().recovered()
        } else {
            label.origin().clone()
        };
        let visibility = if matches!(mutation, Task258B2ResolverMutation::Private) {
            mizar_resolve::env::Visibility::Private
        } else {
            label.visibility()
        };
        let export_status = if matches!(mutation, Task258B2ResolverMutation::LocalOnly) {
            mizar_resolve::env::ExportStatus::LocalOnly
        } else {
            label.export_status()
        };
        labels.insert(
            mizar_resolve::env::LabelEntry::new(
                origin_path,
                kind,
                label.namespace().clone(),
                label.primary_spelling(),
                origin,
                label.contribution(),
            )
            .with_visibility(visibility)
            .with_export_status(export_status),
        );
        if matches!(mutation, Task258B2ResolverMutation::Duplicate) {
            labels.insert(
                mizar_resolve::env::LabelEntry::new(
                    mizar_resolve::resolved_ast::LabelOriginPath::new(
                        "task258b2::duplicate::theorem::origin",
                    ),
                    label.kind(),
                    label.namespace().clone(),
                    label.primary_spelling(),
                    label.origin().clone(),
                    label.contribution(),
                )
                .with_visibility(label.visibility())
                .with_export_status(label.export_status()),
            );
        }
    }
    SymbolEnv::new(
        symbols.module_id().clone(),
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels,
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

fn statement_env_with_imported_label(symbols: SymbolEnv) -> SymbolEnv {
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258A exact label")
        .clone();
    let mut nodes = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let owner = nodes
        .push(mizar_resolve::resolved_ast::ResolvedNode::new(
            mizar_syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            label.origin().clone(),
        ))
        .expect("Task258A imported-label owner");
    let source_range = mizar_session::SourceRange {
        source_id: label.origin().source_id(),
        start: 0,
        end: 1,
    };
    let mut imports = mizar_resolve::resolved_ast::ResolvedImports::new();
    let import = imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
        owner,
        source_range,
        "import statement.fixture;",
        None,
        mizar_resolve::resolved_ast::ImportResolution::Resolved(
            symbols.module_id().clone(),
        ),
        label.origin().clone(),
    ));
    let mut labels = mizar_resolve::env::LabelIndex::new();
    labels.insert(
        mizar_resolve::env::LabelEntry::new(
            label.origin_path().clone(),
            label.kind(),
            label.namespace().clone(),
            label.primary_spelling(),
            label.origin().clone().with_import_edge(import),
            label.contribution(),
        )
        .with_visibility(label.visibility())
        .with_export_status(label.export_status()),
    );
    SymbolEnv::new(
        symbols.module_id().clone(),
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels,
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
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none(),
            "{label}"
        );
    }

    for (ordinal, (label, source)) in [
        (
            "named theorem near miss",
            TASK258A_SOURCE.replacen(
                TASK258A_LABEL,
                "FormulaStatementReservedVariableEqualityNearMiss",
                1,
            ),
        ),
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
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "{label} must fail its real loaded-source selector"
        );
        assert!(
            source_statement_output_with_source(&ast, module, &symbols, TASK258A_SOURCE).is_none(),
            "{label} must also fail when the exact loaded-source guard is supplied"
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
    let (ast, module, _, symbols) = task253_ast_from_source_text(TASK258A_SOURCE, 258_300);
    let output = source_statement_output_with_source(&ast, module, &symbols, TASK258A_SOURCE)
        .expect("Task258A ownership selector")
        .expect("Task258A ownership output");
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258A typed owner")
        .clone();
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert_eq!(
        output.typed_ast.source_atomic_formula(),
        output.resolved.source_atomic_formula()
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

    let replay = source_statement_output_with_source(
        &ast,
        output.typed_ast.module_id().clone(),
        &symbols,
        TASK258A_SOURCE,
    )
    .expect("Task258A replay selector")
    .expect("Task258A replay output");
    assert_eq!(replay.typed_ast.debug_text(), output.typed_ast.debug_text());
    assert_eq!(replay.resolved.debug_text(), output.resolved.debug_text());
}

struct Task258AReplayFixture<'a> {
    ast: &'a SurfaceAst,
    module: ResolverModuleId,
    symbols: &'a SymbolEnv,
    baseline_typed: &'a str,
    baseline_resolved: &'a str,
}

fn assert_task258a_mutation_rejects_then_replays(
    fixture: &Task258AReplayFixture<'_>,
    label: &str,
    expected_error_fragment: &str,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) {
    let error = source_statement_output_with_source_and_mutation(
        fixture.ast,
        fixture.module.clone(),
        fixture.symbols,
        TASK258A_SOURCE,
        mutate,
    )
    .unwrap_or_else(|| panic!("{label} must preserve the exact selector"))
    .expect_err(label);
    assert!(
        error.to_ascii_lowercase().contains(expected_error_fragment),
        "{label}: expected {expected_error_fragment:?}, got {error:?}"
    );
    let replay = source_statement_output_with_source(
        fixture.ast,
        fixture.module.clone(),
        fixture.symbols,
        TASK258A_SOURCE,
    )
    .unwrap_or_else(|| panic!("{label} replay must preserve the selector"))
    .unwrap_or_else(|error| panic!("{label} replay failed: {error}"));
    assert_eq!(
        replay.typed_ast.debug_text(),
        fixture.baseline_typed,
        "{label}"
    );
    assert_eq!(
        replay.resolved.debug_text(),
        fixture.baseline_resolved,
        "{label}"
    );
}
