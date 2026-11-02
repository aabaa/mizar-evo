#[test]
fn task258b3m2b2b1a_real_frontend_freezes_application_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B2B1A_TEXT.len(), 143);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B1A_TEXT),
        "22ce235030bc56720bfe7f52830182144ca6e4eee4414b7f8c2823e3d0f82c1b"
    );
    let (ast, module, symbols) = task258b3m2b2b1a_fixture(258_940);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (63, Some(62))
    );
    let extracted: SourceStatementB3M2B2B1AExtraction =
        extract_application_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B1A_TEXT)
            .expect("Task258B3M2B2B1A exact extraction");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.take_site.node().index(),
            extracted.witness_site.node().index(),
            extracted.application_node,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (59, 48, 142, 49, 48, 46, 103, 141)
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [38, 40, 44, 45, 50, 52]
    );
    let resolver = source_statement_b3m2b2b1a_resolver_env_for_test(&module, &symbols, &extracted)
        .expect("Task258B3M2B2B1A resolver projection");
    assert_eq!(resolver.imports().len(), 1);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    )
    .expect("Task258B3M2B2B1A selector")
    .unwrap_or_else(|error| panic!("Task258B3M2B2B1A route failed: {error}"));
    let primary = output.typed_ast.source_term().expect("Task252 handoff");
    let application = output
        .typed_ast
        .source_application()
        .expect("Task253 application handoff");
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 atomic handoff");
    let statements = output
        .typed_ast
        .source_statement()
        .expect("Task258 statement handoff");
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("Task258 application witness handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (6, 4, 2)
    );
    assert_eq!(
        (
            application.applications().len(),
            application.wrappers().len(),
            application.candidates().len(),
            application.arguments().len(),
            application.type_requests().len(),
        ),
        (1, 0, 1, 2, 2)
    );
    let candidate = application
        .candidates()
        .get(mizar_checker::source_application::SourceFunctorCandidateId::new(0))
        .expect("application candidate 0");
    assert_eq!(candidate.application().index(), 0);
    assert_eq!(candidate.ordinal(), 0);
    assert_eq!(candidate.symbol().module().package(), module.package());
    assert_eq!(
        candidate.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert_eq!(
        candidate.symbol().local().as_str(),
        "summary:parser.type_fixtures#parse-only#++:12"
    );
    assert_eq!(
        candidate.symbol().fqn().as_str(),
        "parser.type_fixtures::++#12"
    );
    assert_eq!(candidate.contribution().index(), 2);
    assert_eq!(candidate.origin().source_id(), ast.source_id);
    assert_eq!(candidate.origin().module_id(), candidate.symbol().module());
    assert_eq!(
        candidate.origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 7,
            end: 27,
        })
    );
    assert_eq!(candidate.origin().structural_path(), [12]);
    assert!(candidate.origin().import_edge().is_none());
    assert!(!candidate.origin().is_recovered());
    assert_eq!(
        candidate.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        candidate.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert!(candidate.signature().is_none());
    assert_eq!(
        application
            .arguments()
            .iter()
            .map(|(_, row)| (row.application().index(), row.ordinal(), row.target()))
            .collect::<Vec<_>>(),
        vec![
            (
                0,
                0,
                mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
            (
                0,
                1,
                mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                ),
            ),
        ]
    );
    assert_eq!(
        application
            .type_requests()
            .iter()
            .map(|(_, row)| {
                (
                    row.application().index(),
                    row.request_ordinal(),
                    row.candidate().map(|candidate| candidate.index()),
                    row.kind(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                0,
                0,
                Some(0),
                mizar_checker::source_application::SourceFunctorTypeRequestKind::CandidateSignature,
            ),
            (
                0,
                1,
                None,
                mizar_checker::source_application::SourceFunctorTypeRequestKind::ApplicationResultType,
            ),
        ]
    );
    let app = application
        .applications()
        .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
        .expect("application 0");
    assert_eq!(app.site().node().index(), 46);
    assert_eq!(
        (app.source_range().start, app.source_range().end),
        (116, 122)
    );
    assert_eq!(app.context().index(), 1);
    assert_eq!(app.spelling(), "1 ++ 2");
    assert_eq!(
        app.form(),
        mizar_checker::source_application::SourceFunctorApplicationForm::Infix
    );
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
        (2, 0, 0, 0, 0, 0, 0, 4, 4)
    );
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, row)| (row.formula().index(), row.ordinal(), row.target()))
            .collect::<Vec<_>>(),
        vec![
            (
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(0),
                ),
            ),
            (
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(1),
                ),
            ),
            (
                1,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(4),
                ),
            ),
            (
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(5),
                ),
            ),
        ]
    );
    assert_eq!(
        atomic
            .requests()
            .iter()
            .map(|(_, row)| {
                (
                    row.formula().index(),
                    row.ordinal(),
                    row.kind(),
                    row.edge().map(|edge| edge.index()),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(0),
            ),
            (
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(1),
            ),
            (
                1,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(2),
            ),
            (
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(3),
            ),
        ]
    );
    assert_eq!(
        (
            statements.owners().len(),
            statements.statements().len(),
            statements.contexts().len(),
            statements.input_facts().len(),
            statements.candidate_facts().len(),
            witnesses.witnesses().len(),
            witnesses.names().len(),
        ),
        (1, 2, 2, 2, 2, 1, 0)
    );
    assert_eq!(
        (
            statements.binding_env().contexts().len(),
            statements.binding_env().bindings().len(),
            statements.binding_env().diagnostics().len(),
        ),
        (2, 1, 0)
    );
    let witness = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
        .expect("application witness 0");
    assert_eq!(
        witness.term(),
        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Application(
            mizar_checker::source_application::SourceFunctorApplicationId::new(0)
        )
    );
    assert_eq!(
        (
            witness.take_site().node().index(),
            witness.take_range().start,
            witness.take_range().end,
            witness.site().node().index(),
            witness.source_range().start,
            witness.source_range().end,
            witness.binding_context().index(),
            witness.source_ordinal(),
            witness.ordinal(),
        ),
        (49, 111, 123, 48, 116, 122, 1, 1, 0)
    );
    assert_eq!(witness.spelling(), "1 ++ 2");
    assert_eq!(
        witness.kind(),
        mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed
    );
    assert_eq!(
        witnesses.application_fingerprint(),
        Some(application.debug_text().as_str())
    );
    assert!(
        witnesses
            .debug_text()
            .contains("application-fingerprint: Some(")
    );
    assert!(witnesses.debug_text().contains("term=application#0"));
    assert_eq!(
        output.typed_ast.source_application(),
        output.resolved.source_application()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert_eq!(
        (output.left_lookup_ordinal, output.right_lookup_ordinal),
        (1, 1)
    );
    assert_eq!(output.reference_use_ordinals, [1; 4]);
    assert_eq!(
        source_statement_transport_detail_keys(
            &ast,
            module,
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        ),
        Some(Vec::new())
    );
    for (id, node) in output.typed_ast.nodes().iter() {
        assert_eq!(
            node.kind.as_str(),
            match id.index() {
                19 => "source.term.functor-head.single",
                38 | 40 | 50 | 52 => "source.term.variable-reference",
                44 | 45 => "source.term.numeral",
                42 | 54 => "source.formula.atomic.equality",
                46 => "source.term.functor-application.symbolic",
                48 => "source.statement-witness.item",
                49 => "source.statement-witness.take",
                57 => "source.statement.conclusion",
                59 => "source.statement.theorem",
                _ => "source.surface.unowned",
            },
            "node {} kind",
            id.index()
        );
    }
}

#[test]
fn task258b3m2b2b1a_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols) = task258b3m2b2b1a_fixture(258_941);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    )
    .expect("baseline selector")
    .expect("baseline");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let resolver_replay = source_statement_b3m2b2b1a_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        |symbols| symbols,
    )
    .expect("resolver replay selector")
    .expect("resolver replay");
    assert_eq!(resolver_replay.typed_ast.debug_text(), baseline_typed);
    let (stale_ast, stale_module, stale_symbols) = task258b3m2b2b1a_fixture(258_942);
    let stale_application = source_statement_output_with_source(
        &stale_ast,
        stale_module,
        &stale_symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    )
    .expect("stale selector")
    .expect("stale output")
    .typed_ast
    .source_application()
    .expect("stale application")
    .clone();
    for mutation in 0..9 {
        let stale_application = stale_application.clone();
        let error = source_statement_b3m2b2b1a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1A_TEXT,
            move |input: &mut SourceStatementB3M2B2B1ARouteInputs| match mutation {
                0 => input.application = None,
                1 => input.application = Some(stale_application),
                2 => {
                    input.witness.witnesses[0].term =
                        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                            mizar_checker::source_term::SourcePrimaryTermId::new(2),
                        )
                }
                3 => input.witness.witnesses[0].term =
                    mizar_checker::source_statement::SourceStatementWitnessTermTarget::Application(
                        mizar_checker::source_application::SourceFunctorApplicationId::new(1),
                    ),
                4 => {
                    input.witness.witnesses[0].site = mizar_checker::typed_ast::TypedSiteRef::Node(
                        mizar_checker::typed_ast::TypedNodeId::new(47),
                    )
                }
                5 => input.witness.witnesses[0].source_range.start += 1,
                6 => input.witness.witnesses[0].spelling.push('x'),
                7 => input.witness.witnesses[0].source_ordinal = 0,
                8 => {
                    input.witness.witnesses[0].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0)
                }
                _ => unreachable!(),
            },
        )
        .expect("mutation selector")
        .expect_err("mutation must fail closed");
        assert!(
            error.to_ascii_lowercase().contains(if mutation <= 1 {
                "dependency"
            } else {
                "witness"
            }),
            "mutation {mutation}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        )
        .expect("replay selector")
        .expect("replay");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline_typed,
            "mutation {mutation}"
        );
        assert_eq!(
            replay.resolved.debug_text(),
            baseline_resolved,
            "mutation {mutation}"
        );
    }
}

#[test]
fn task258b3m2b2b1a_selector_and_byte_subtree_near_misses_are_exact() {
    let (exact_ast, exact_module, exact_symbols) = task258b3m2b2b1a_fixture(258_943);
    assert_eq!(SOURCE_STATEMENT_B3M2B2B1A_TEXT.len(), 143);
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B1A_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B1A_TEXT.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' {
            b'?'
        } else {
            b'!'
        };
        let loaded_source = String::from_utf8(bytes).expect("ASCII source mutation");
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none(),
            "byte mutation {byte_index} selected"
        );
    }
    for loaded_source in [
        SOURCE_STATEMENT_B3M2B2B1A_TEXT
            .trim_end_matches('\n')
            .to_owned(),
        format!("{SOURCE_STATEMENT_B3M2B2B1A_TEXT}\n"),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take 1  ++ 2;", 1),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none()
        );
    }
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take (1 ++ 2);", 1),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take 1;", 1),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take 2 ++ 1;", 1),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take 1 ++ 2, x;", 1),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take 1 + 2;", 1),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take y = 1 ++ 2;", 1),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen(
            "import parser.type_fixtures;",
            "import parser.other_fixtures;",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen("take 1 ++ 2;", "take 1 ++ ;", 1),
        SOURCE_STATEMENT_B3M2B2B1A_TEXT.replacen(
            "FormulaStatementApplicationWitnessSmoke",
            "FormulaStatementApplicationWitnessNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(&source, 258_950 + ordinal);
        let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
        assert!(
            extract_application_witness_source_statement(&ast, &source).is_none(),
            "near miss {ordinal} extracted"
        );
        assert!(
            source_statement_output_with_source(&ast, module, &symbols, &source).is_none(),
            "near miss {ordinal} selected"
        );
    }
}

#[test]
fn task258b3m2b2b1a_family_and_active_route_isolation_is_atomic() {
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3_TEXT,
        SOURCE_STATEMENT_B3N_TEXT,
        SOURCE_STATEMENT_B3M1_TEXT,
        SOURCE_STATEMENT_B3M2A_TEXT,
        SOURCE_STATEMENT_B3M2B1_TEXT,
        SOURCE_STATEMENT_B3M2B2A_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(source, 258_960 + ordinal);
        let typed = if source == SOURCE_STATEMENT_B3_TEXT || source == SOURCE_STATEMENT_B3N_TEXT || source == SOURCE_STATEMENT_B3M1_TEXT || source == SOURCE_STATEMENT_B3M2B1_TEXT || source == SOURCE_STATEMENT_B3M2B2A_TEXT {
            task258_native_output(&ast, &module).0
        } else {
            source_statement_output_with_source(&ast, module, &symbols, source)
                .expect("legacy family selector").expect("legacy family output").typed_ast
        };
        assert!(typed.source_statement_witnesses().is_some());
        assert!(typed.source_application().is_none());
        assert!(
            typed
                .source_statement_witnesses()
                .expect("legacy witnesses")
                .application_fingerprint()
                .is_none()
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
    let plan = build_test_plan(&config).expect("Task258B3M2B2B1A isolation plan");
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
        if extract_application_witness_source_statement(&ast, &source).is_some()
            || source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
                .is_some_and(|result| {
                    result.is_ok_and(|output| {
                        output.typed_ast.source_application().is_some()
                            && output.typed_ast.source_statement_witnesses().is_some()
                    })
                })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M2B2B1A selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b2b1a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, symbols) = task258b3m2b2b1a_fixture(258_970);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    )
    .expect("selector")
    .expect("route");
    assert_eq!(
        output.typed_ast.clone().debug_text(),
        output.typed_ast.debug_text()
    );
    assert_eq!(
        output.resolved.clone().debug_text(),
        output.resolved.debug_text()
    );
    assert_eq!(
        output.typed_ast.source_application(),
        output.resolved.source_application()
    );
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert!(output.typed_ast.source_statement_references().is_none());
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
    assert!(output.resolved.diagnostics().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
    let baseline = output.typed_ast.debug_text();
    let error = source_statement_b3m2b2b1a_output_with_mutation(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        |input| input.witness.witnesses.clear(),
    )
    .expect("rollback selector")
    .expect_err("empty witness aggregate must roll back");
    assert!(error.to_ascii_lowercase().contains("aggregate"), "{error}");
    assert_eq!(output.typed_ast.debug_text(), baseline);
}
