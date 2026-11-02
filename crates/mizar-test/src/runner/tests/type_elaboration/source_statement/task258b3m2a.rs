#[test]
fn task258b3m2a_real_frontend_freezes_numeral_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2A_TEXT.len(), 107);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2A_TEXT),
        "7b424949e98761b0179758065db5d164ad7d0a640f082801986683a54c43a2d1"
    );
    assert!(SOURCE_STATEMENT_B3M2A_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2A_TEXT, 258_900);
    assert_eq!(
        (ast.nodes().len(), ast.root().expect("root").index()),
        (49, 48)
    );
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let extracted: SourceStatementB3M2AExtraction =
        extract_numeral_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2A_TEXT)
            .expect("Task258B3M2A exact parser shape");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.take_site.node().index(),
            extracted.take_range.start,
        ),
        (45, 19, 106, 27, 62, 35, 78)
    );
    assert_eq!(
        (
            extracted.take_range.end,
            extracted.witness_site.node().index(),
            extracted.witness_range.start,
            extracted.witness_range.end,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (87, 34, 83, 86, 70, 105)
    );
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [45, 43]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [30, 40]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [26, 28, 32, 36, 38]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(64, 65), (68, 69), (83, 86), (95, 96), (99, 100)]
    );

    let resolver = source_statement_b3m2a_resolver_env_for_test(&module, &symbols, &extracted)
        .expect("Task258B3M2A resolver provenance");
    assert_eq!(
        (resolver.labels().len(), resolver.contributions().len()),
        (1, 1)
    );
    assert!(resolver.imports().is_empty());
    let labels = resolver.labels().visible_candidates(
        &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
        "FormulaStatementNumeralWitnessSmoke",
    );
    let [label] = labels.as_slice() else {
        panic!("one Task258B3M2A theorem label")
    };
    assert_eq!(
        label.kind(),
        mizar_resolve::resolved_ast::LabelKind::Theorem
    );
    assert_eq!(label.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        label.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(label.contribution().index(), 0);
    assert_eq!(label.origin().structural_path(), [2, 1]);
    assert!(
        resolver
            .symbols()
            .visible_candidates(
                &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
                "y",
            )
            .is_empty()
    );

    let before = (
        ast.snapshot_text(),
        module.clone(),
        symbols.clone(),
        resolver.clone(),
    );
    assert!(
        source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
        )
        .is_none()
    );
    assert!(
        source_statement_transport_detail_keys(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
        )
        .is_none()
    );
    let mut lower = None;
    assert_eq!(
        source_statement_b3m2a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |input| lower = Some(input.clone()),
        )
        .expect("authentic lower extraction")
        .expect_err("retired raw profile"),
        "source statement dependency mismatch",
    );
    let lower = lower.expect("lower data before raw admission");
    let statement = &lower.statement;
    let primary = &lower.primary;
    let atomic = &lower.atomic;
    assert_eq!(
        (statement.source_id, &statement.module_id),
        (ast.source_id, &module)
    );
    assert_eq!(
        (primary.source_id(), primary.module_id()),
        (ast.source_id, &module)
    );
    assert_eq!(
        (atomic.source_id(), atomic.module_id()),
        (ast.source_id, &module)
    );
    assert_eq!(
        (lower.witness.source_id, &lower.witness.module_id),
        (ast.source_id, &module)
    );
    assert_eq!(
        (
            lower.binding_env.contexts().len(),
            lower.binding_env.bindings().len(),
            lower.binding_env.diagnostics().len(),
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len()
        ),
        (2, 1, 0, 5, 4, 1, 2, 4, 4)
    );
    assert_eq!(
        (
            statement.owners.len(),
            statement.statements.len(),
            statement.contexts.len(),
            statement.input_facts.len(),
            statement.candidate_facts.len(),
            lower.witness.witnesses.len(),
            lower.witness.names.len()
        ),
        (1, 2, 2, 2, 2, 1, 0)
    );
    let proof = lower
        .binding_env
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement {
            source_range: mizar_session::SourceRange {
                source_id: ast.source_id,
                start: 70,
                end: 105,
            },
        }
    );
    assert_eq!(proof.parent.map(|id| id.index()), Some(0));
    assert_eq!(
        proof
            .lexical_scope
            .as_ref()
            .map(|scope| scope.path().to_vec()),
        Some(vec![0])
    );
    assert!(proof.bindings.is_empty());
    assert_eq!(
        proof
            .visible_bindings
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [0]
    );

    for index in 0..5 {
        let term = primary
            .terms()
            .get(mizar_checker::source_term::SourcePrimaryTermId::new(index))
            .expect("term");
        assert_eq!(term.site().node().index(), [26, 28, 32, 36, 38][index]);
        assert_eq!(
            (term.source_range().start, term.source_range().end),
            [(64, 65), (68, 69), (83, 86), (95, 96), (99, 100)][index]
        );
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context().index(), [0, 0, 1, 1, 1][index]);
        assert_eq!(term.spelling(), if index == 2 { "101" } else { "x" });
        assert_eq!(
            term.kind(),
            if index == 2 {
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral
            } else {
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference
            }
        );
        assert_eq!(
            term.role(),
            mizar_checker::source_term::SourcePrimaryTermRole::Value
        );
        assert_eq!(
            term.recovery(),
            mizar_checker::source_term::SourcePrimaryTermRecovery::Normal
        );
        assert!(term.parent().is_none());
    }
    for (reference_index, term_index) in [0, 1, 3, 4].into_iter().enumerate() {
        let reference = primary
            .references()
            .get(mizar_checker::source_term::SourcePrimaryTermReferenceId::new(reference_index))
            .expect("reference");
        assert_eq!(reference.term().index(), term_index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(
            reference.role(),
            mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
        );
        assert_eq!(reference.use_ordinal(), 1);
        assert_eq!(
            reference.lexical_scope().map(|scope| scope.path().to_vec()),
            if term_index < 2 { None } else { Some(vec![0]) }
        );
    }
    let numeric = primary
        .numeric_type_requests()
        .get(mizar_checker::source_term::SourceNumericTypeRequestId::new(
            0,
        ))
        .expect("numeric request");
    assert_eq!(
        (
            numeric.term().index(),
            numeric.owner().node().index(),
            numeric.source_range().start,
            numeric.source_range().end,
            numeric.spelling(),
            numeric.request_ordinal(),
        ),
        (2, 32, 83, 86, "101", 0)
    );
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
        [0, 1, 3, 4]
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
            (
                2,
                1,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(2),
                None,
                None,
                None,
            ),
            (
                3,
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(3),
                None,
                None,
                None,
            ),
        ]
    );
    for index in 0..2 {
        let row = &statement.statements[index];
        assert_eq!(
            (
                row.site.node().index(),
                row.source_range.start,
                row.source_range.end,
                row.source_ordinal
            ),
            (
                [45, 43][index],
                [19, 90][index],
                [106, 101][index],
                [0, 2][index]
            )
        );
        assert_eq!(
            statement.input_facts[index]
                .uses
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![2, 3] }
        );
    }
    let witness = &lower.witness.witnesses[0];
    assert_eq!(
        (
            witness.owner.index(),
            witness.binding_context.index(),
            witness.term,
            witness.take_site.node().index(),
            witness.take_range,
            witness.site.node().index(),
            witness.source_range
        ),
        (
            0,
            1,
            mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(2)
            ),
            35,
            range(ast.source_id, 78, 87),
            34,
            range(ast.source_id, 83, 86)
        )
    );
    assert_eq!(
        (
            witness.source_ordinal,
            witness.ordinal,
            witness.spelling.as_str(),
            witness.kind,
            witness.name
        ),
        (
            1,
            0,
            "101",
            mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed,
            None
        )
    );
    assert_eq!((ast.snapshot_text(), module, symbols, resolver), before);
}

#[test]
fn task258b3m2a_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2A_TEXT, 258_901);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let replay = || {
        assert_eq!(
            source_statement_b3m2a_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2A_TEXT,
                |symbols| symbols,
            )
            .expect("authentic resolver extraction")
            .expect_err("retired raw profile"),
            "source statement dependency mismatch"
        );
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
        assert!(
            extract_numeral_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2A_TEXT).is_some()
        );
    };
    replay();
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
        let error = source_statement_b3m2a_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("resolver selector")
        .expect_err("resolver mutation");
        let expected = if matches!(
            mutation,
            Task258B2ResolverMutation::Missing | Task258B2ResolverMutation::Duplicate,
        ) {
            "Task258B3M2A resolver theorem label mismatch"
        } else {
            "Task258B3M2A resolver theorem provenance mismatch"
        };
        assert_eq!(error, expected, "{mutation:?}");
        replay();
    }
    for mutation in [
        Task258B3M2B2B3ALabelMutation::Namespace,
        Task258B3M2B2B3ALabelMutation::PrimarySpelling,
        Task258B3M2B2B3ALabelMutation::Contribution,
        Task258B3M2B2B3ALabelMutation::OriginSource,
        Task258B3M2B2B3ALabelMutation::OriginModule,
        Task258B3M2B2B3ALabelMutation::OriginAnchor,
        Task258B3M2B2B3ALabelMutation::OriginStructuralPath,
        Task258B3M2B2B3ALabelMutation::ContributionEffectsMissing,
        Task258B3M2B2B3ALabelMutation::ContributionEffectsExtra,
        Task258B3M2B2B3ALabelMutation::ContributionEffectsWrongPath,
    ] {
        let error = source_statement_b3m2a_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |symbols| task258b3m2b2b3a_mutate_label_resolver(symbols, mutation),
        )
        .expect("label extraction")
        .expect_err("label preflight");
        let expected = if matches!(
            mutation,
            Task258B3M2B2B3ALabelMutation::Namespace
                | Task258B3M2B2B3ALabelMutation::PrimarySpelling,
        ) {
            "Task258B3M2A resolver theorem label mismatch"
        } else {
            "Task258B3M2A resolver theorem provenance mismatch"
        };
        assert_eq!(error, expected, "{mutation:?}");
        replay();
    }
    let mut lower = None;
    assert_eq!(
        source_statement_b3m2a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |input: &mut SourceStatementB3M2ARouteInputs| lower = Some(input.clone()),
        )
        .expect("authentic lower extraction")
        .expect_err("retired raw profile"),
        "source statement dependency mismatch"
    );
    let lower = lower.expect("real lower data before raw admission");
    let primary_input = task258b3m2a_primary_input(&lower.primary);
    for mutation in 0..8 {
        let mut input = primary_input.clone();
        match mutation {
            0 => input.numeric_type_requests.clear(),
            1 => input
                .numeric_type_requests
                .push(input.numeric_type_requests[0].clone()),
            2 => {
                input.numeric_type_requests[0].term =
                    mizar_checker::source_term::SourcePrimaryTermId::new(0)
            }
            3 => input.numeric_type_requests[0].owner = input.terms[0].site.clone(),
            4 => input.numeric_type_requests[0].source_range.start += 1,
            5 => input.numeric_type_requests[0].source_range.end -= 1,
            6 => input.numeric_type_requests[0].spelling.push('z'),
            7 => input.numeric_type_requests[0].request_ordinal = 1,
            _ => unreachable!(),
        }
        let error = mizar_checker::source_term::SourcePrimaryTermProducer::build(
            input,
            &lower.binding_env,
            &lower.arena,
        )
        .expect_err("numeric request mutation");
        let expected_numeric_error = match &error {
            mizar_checker::source_term::SourcePrimaryTermError::InvalidNumericTypeRequest {
                ..
            } => true,
            mizar_checker::source_term::SourcePrimaryTermError::InvalidTerm { term } => {
                mutation == 0 && term.index() == 2
            }
            _ => false,
        };
        assert!(
            expected_numeric_error,
            "numeric request field {mutation}: {error}"
        );
        replay();
    }
}

#[test]
fn task258b3m2a_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2A_TEXT, 258_903);
    assert_eq!(
        source_statement_b3m2a_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |symbols| symbols,
        )
        .expect("resolver selector")
        .expect_err("retired raw profile"),
        "source statement dependency mismatch"
    );
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    for byte in 0..SOURCE_STATEMENT_B3M2A_TEXT.len() {
        let mut source = SOURCE_STATEMENT_B3M2A_TEXT.as_bytes().to_vec();
        source[byte] = b'?';
        let source = String::from_utf8(source).expect("ASCII source");
        assert!(
            extract_numeral_witness_source_statement(&ast, &source).is_none(),
            "byte {byte}"
        );
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
        assert!(
            extract_numeral_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2A_TEXT).is_some()
        );
    }
    for target in 0..49 {
        for mutation in 0..4 {
            let mut builder = mizar_syntax::SurfaceAstBuilder::new(ast.source_id);
            let mut rebuilt = Vec::<mizar_syntax::SurfaceBuilderNodeId>::new();
            for (index, node) in ast.nodes().iter().enumerate() {
                let mut kind = node.kind.clone();
                let mut node_range = node.range;
                let mut children = node
                    .children
                    .iter()
                    .map(|id| rebuilt[id.index()])
                    .collect::<Vec<_>>();
                if index == target {
                    match mutation {
                        0 => match &mut kind {
                            mizar_syntax::SurfaceNodeKind::Token(token) => {
                                token.kind = mizar_syntax::SurfaceTokenKind::LexemeRun
                            }
                            _ => kind = mizar_syntax::SurfaceNodeKind::AttributeChain,
                        },
                        1 => node_range.end += 1,
                        2 if matches!(kind, mizar_syntax::SurfaceNodeKind::Token(_)) => {
                            let mizar_syntax::SurfaceNodeKind::Token(token) = &mut kind else {
                                unreachable!()
                            };
                            token.text = "?".into();
                        }
                        2 if children.len() > 1 => children.swap(0, 1),
                        2 if !children.is_empty() => children.clear(),
                        2 if index > 0 => children.push(rebuilt[0]),
                        2 => kind = mizar_syntax::SurfaceNodeKind::AttributeChain,
                        3 => {}
                        _ => unreachable!(),
                    }
                }
                let id = if index == target && mutation == 3 {
                    match kind {
                        mizar_syntax::SurfaceNodeKind::Token(token) => {
                            builder.add_recovered_token(token.kind, token.text, node_range)
                        }
                        _ => builder.add_recovery(
                            mizar_syntax::SyntaxRecoveryKind::MissingTerm,
                            node_range,
                            children,
                        ),
                    }
                } else {
                    match kind {
                        mizar_syntax::SurfaceNodeKind::Token(token) => {
                            builder.add_token(token.kind, token.text, node_range)
                        }
                        structural => builder.add_node(structural, node_range, children),
                    }
                };
                rebuilt.push(id);
            }
            let changed = builder.finish(
                ast.root().map(|id| rebuilt[id.index()]),
                ast.expression_root().map(|id| rebuilt[id.index()]),
            );
            assert!(
                extract_numeral_witness_source_statement(&changed, SOURCE_STATEMENT_B3M2A_TEXT)
                    .is_none(),
                "node {target} mutation {mutation}"
            );
            assert!(
                source_statement_b3m2a_output_with_resolver_mutation(
                    &changed,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B3M2A_TEXT,
                    |symbols| symbols,
                )
                .is_none(),
                "node {target} mutation {mutation} extraction gate"
            );
            assert_eq!(
                (ast.snapshot_text(), module.clone(), symbols.clone()),
                before
            );
            assert!(
                extract_numeral_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2A_TEXT)
                    .is_some()
            );
        }
    }
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3M2A_TEXT
            .trim_end_matches('\n')
            .to_owned(),
        format!("{SOURCE_STATEMENT_B3M2A_TEXT}\n"),
        SOURCE_STATEMENT_B3M2A_TEXT.replacen("  take 101;", "   take 101;", 1),
        SOURCE_STATEMENT_B3M2A_TEXT.replacen(
            "FormulaStatementNumeralWitnessSmoke",
            "FormulaStatementNumeralWitnessNearMiss",
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
    }

    let reordered = SOURCE_STATEMENT_B3M2A_TEXT.replacen(
        "  take 101;\n  thus x = x;\n",
        "  thus x = x;\n  take 101;\n",
        1,
    );
    let near_misses = [
        (
            "single variable witness",
            SOURCE_STATEMENT_B3_TEXT.to_owned(),
            false,
        ),
        ("named witness", SOURCE_STATEMENT_B3N_TEXT.to_owned(), false),
        (
            "multiple witness",
            SOURCE_STATEMENT_B3M1_TEXT.to_owned(),
            false,
        ),
        (
            "another numeral",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take 102;", 1),
            false,
        ),
        (
            "minus sign",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take -101;", 1),
            false,
        ),
        (
            "plus sign",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take +101;", 1),
            false,
        ),
        (
            "extra token",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take 101 x;", 1),
            false,
        ),
        (
            "named numeral",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take y = 101;", 1),
            false,
        ),
        (
            "multiple numerals",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take 101, 102;", 1),
            false,
        ),
        (
            "identifier",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take x;", 1),
            false,
        ),
        (
            "it",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take it;", 1),
            false,
        ),
        (
            "parenthesized",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take (101);", 1),
            false,
        ),
        (
            "application",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take f(101);", 1),
            false,
        ),
        (
            "selector",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take the carrier of x;", 1),
            false,
        ),
        (
            "functional update",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take x(# field -> 101 #);", 1),
            false,
        ),
        (
            "set",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take {101};", 1),
            false,
        ),
        (
            "choice",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take the set;", 1),
            false,
        ),
        (
            "missing witness token",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take;", 1),
            false,
        ),
        (
            "missing take",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("  take 101;\n", "", 1),
            false,
        ),
        (
            "two takes",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("  take 101;\n", "  take 101;\n  take 102;\n", 1),
            false,
        ),
        (
            "extra conclusion",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen(
                "  thus x = x;\n",
                "  thus x = x;\n  thus x = x;\n",
                1,
            ),
            false,
        ),
        ("reordered statements", reordered, false),
        (
            "recovery",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen("take 101;", "take 101", 1),
            false,
        ),
        (
            "changed theorem shape",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen(
                ": x = x proof\n  take 101;\n  thus x = x;\nend;",
                ": x = x;",
                1,
            ),
            false,
        ),
        (
            "existential root",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen(
                ": x = x proof",
                ": ex z being set st z = z proof",
                1,
            ),
            false,
        ),
        (
            "composite root",
            SOURCE_STATEMENT_B3M2A_TEXT.replacen(": x = x proof", ": (x = x) & (x = x) proof", 1),
            false,
        ),
    ];
    for (ordinal, (label, source, supported_family)) in near_misses.into_iter().enumerate() {
        let (near_ast, near_module, _, near_symbols) =
            task253_ast_from_source_text(&source, 258_920 + ordinal);
        assert!(
            extract_numeral_witness_source_statement(&near_ast, &source).is_none(),
            "{label}: own-source extraction"
        );
        assert!(
            extract_numeral_witness_source_statement(&near_ast, SOURCE_STATEMENT_B3M2A_TEXT)
                .is_none(),
            "{label}: exact-byte subtree extraction"
        );
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_symbols,
                SOURCE_STATEMENT_B3M2A_TEXT,
            )
            .is_none(),
            "{label}: exact-byte subtree route"
        );
        if [
            "single variable witness",
            "identifier",
            "named witness",
            "multiple witness",
        ]
        .contains(&label)
        {
            let (typed, _) = task258_native_output(&near_ast, &near_module);
            assert_eq!(typed.source_statement().unwrap().statements().len(), 2);
            assert_eq!(
                typed
                    .source_statement_witnesses()
                    .unwrap()
                    .witnesses()
                    .len(),
                if label == "multiple witness" { 2 } else { 1 }
            );
            assert!(typed.source_proof_local_declaration().is_none());
        }
        let selected =
            source_statement_output_with_source(&near_ast, near_module, &near_symbols, &source);
        assert_eq!(
            selected.is_some(),
            supported_family,
            "{label}: own-source generic selector"
        );
    }
}

#[test]
fn task258b3m2a_family_and_active_route_isolation_is_atomic_in_both_orders() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2A_TEXT, 258_904);
    assert!(extract_numeral_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2A_TEXT).is_some());
    assert!(
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B3M2A_TEXT,)
            .is_none()
    );
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
    let plan = build_test_plan(&config).expect("Task258B3M2A isolation plan");
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
        let extracted = extract_numeral_witness_source_statement(&active_ast, &source).is_some();
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
                        handoff.witnesses().len() == 1
                            && handoff.names().is_empty()
                            && output
                                .typed_ast
                                .source_term()
                                .is_some_and(|primary| primary.numeric_type_requests().len() == 1)
                    })
            })
        });
        if extracted || selected_route {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M2A selected active cases: {selected:?}"
    );
}
