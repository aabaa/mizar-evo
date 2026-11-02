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

    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2A_TEXT,
    )
    .expect("Task258B3M2A selector")
    .expect("Task258B3M2A route");
    let statement = output.typed_ast.source_statement().expect("statement");
    let primary = output.typed_ast.source_term().expect("primary terms");
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("atomic formulas");
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("witness handoff");
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (2, 1, 0, 5, 4, 1, 2, 4, 4)
    );
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            witnesses.witnesses().len(),
            witnesses.names().len(),
        ),
        (1, 2, 2, 2, 2, 1, 0)
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
                [45, 43][index],
                [19, 90][index],
                [106, 101][index],
                [0, 2][index],
            )
        );
        let fact = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(
            fact.uses()
                .iter()
                .map(|reference| reference.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![2, 3] }
        );
    }
    let witness = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
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
                mizar_checker::source_term::SourcePrimaryTermId::new(2),
            ),
            35,
            78,
            87,
            34,
            83,
            86
        )
    );
    assert_eq!(
        (
            witness.source_ordinal(),
            witness.ordinal(),
            witness.spelling(),
            witness.kind(),
            witness.name(),
        ),
        (
            1,
            0,
            "101",
            mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed,
            None,
        )
    );
    for (index, expected) in [
        (30, "source.formula.atomic.equality"),
        (32, "source.term.numeral"),
        (33, "source.surface.unowned"),
        (34, "source.statement-witness.item"),
        (35, "source.statement-witness.take"),
        (40, "source.formula.atomic.equality"),
        (43, "source.statement.conclusion"),
        (45, "source.statement.theorem"),
    ] {
        assert_eq!(
            output
                .typed_ast
                .nodes()
                .node(mizar_checker::typed_ast::TypedNodeId::new(index))
                .expect("typed node")
                .kind
                .as_str(),
            expected
        );
    }
    assert_eq!(output.reference_use_ordinals, [1; 4]);
    assert_eq!(
        (output.left_lookup_ordinal, output.right_lookup_ordinal),
        (1, 1)
    );
    assert_eq!(
        source_statement_transport_detail_keys(&ast, module, &symbols, SOURCE_STATEMENT_B3M2A_TEXT,),
        Some(Vec::new())
    );
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert_eq!(
        output.typed_ast.source_atomic_formula(),
        output.resolved.source_atomic_formula()
    );
    assert!(output.typed_ast.source_statement_references().is_none());
}

#[test]
fn task258b3m2a_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2A_TEXT, 258_901);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2A_TEXT,
    )
    .expect("baseline selector")
    .expect("baseline route");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let replay = || {
        let output = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
        )
        .expect("replay selector")
        .expect("replay route");
        assert_eq!(output.typed_ast.debug_text(), baseline_typed);
        assert_eq!(output.resolved.debug_text(), baseline_resolved);
    };

    let (foreign_ast, foreign_module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_902);
    let (foreign, _) = task258_native_output(&foreign_ast, &foreign_module);
    assert!(foreign.source_statement_witnesses().is_some(), "genuine native B3 foreign oracle");
    let wrong_binding = foreign
        .source_statement()
        .expect("foreign statement")
        .binding_env()
        .clone();
    let wrong_primary = foreign
        .source_term()
        .expect("foreign primary")
        .clone();
    let wrong_atomic = foreign
        .source_atomic_formula()
        .expect("foreign atomic")
        .clone();

    for (mutation, expected) in [(0, "dependency"), (1, "aggregate"), (2, "witness 0")] {
        let error = source_statement_b3m2a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |input: &mut SourceStatementB3M2ARouteInputs| match mutation {
                0 => {
                    input.binding_env = wrong_binding.clone();
                    input.witness.witnesses.clear();
                }
                1 => {
                    input.witness.witnesses[0].ordinal = 1;
                    input
                        .witness
                        .witnesses
                        .push(input.witness.witnesses[0].clone());
                }
                2 => input.witness.witnesses[0].ordinal = 1,
                _ => unreachable!(),
            },
        )
        .expect("precedence selector")
        .expect_err("mixed fault");
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "precedence {mutation}: {error}"
        );
        replay();
    }

    for (mutation, expected) in [
        (0, "aggregate"),
        (1, "aggregate"),
        (2, "aggregate"),
        (3, "dependency"),
        (4, "dependency"),
        (5, "aggregate"),
    ] {
        let error = source_statement_b3m2a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |input| match mutation {
                0 => {
                    input.witness.witnesses.clear();
                }
                1 => input
                    .witness
                    .witnesses
                    .push(input.witness.witnesses[0].clone()),
                2 => input.witness.names.push(
                    mizar_checker::source_statement::SourceStatementWitnessNameInput {
                        witness: mizar_checker::source_statement::SourceStatementWitnessId::new(0),
                        site: input.witness.witnesses[0].site.clone(),
                        source_range: input.witness.witnesses[0].source_range,
                        spelling: "y".to_owned(),
                        recovery: mizar_checker::source_statement::SourceStatementRecovery::Normal,
                    },
                ),
                3 => input.primary = wrong_primary.clone(),
                4 => input.atomic = wrong_atomic.clone(),
                5 => {
                    input.statement.candidate_facts.pop();
                }
                _ => unreachable!(),
            },
        )
        .expect("aggregate/dependency selector")
        .expect_err("aggregate/dependency mutation");
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "mutation {mutation}: {error}"
        );
        replay();
    }

    for mutation in 0..16 {
        let error = source_statement_b3m2a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |input| {
                let row = &mut input.witness.witnesses[0];
                match mutation {
                    0 => row.owner = mizar_checker::source_statement::SourceTheoremOwnerId::new(1),
                    1 => row.binding_context = mizar_checker::binding_env::BindingContextId::new(0),
                    2 => row.term =
                        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                            mizar_checker::source_term::SourcePrimaryTermId::new(0),
                        ),
                    3 => row.take_site = row.site.clone(),
                    4 => row.take_range.start += 1,
                    5 => row.take_range.end -= 1,
                    6 => row.site = row.take_site.clone(),
                    7 => row.source_range.start += 1,
                    8 => row.source_range.end -= 1,
                    9 => row.source_ordinal = 2,
                    10 => row.ordinal = 1,
                    11 => row.spelling.push('z'),
                    12 => {
                        row.recovery =
                            mizar_checker::source_statement::SourceStatementRecovery::Degraded
                    }
                    13 => {
                        row.kind =
                            mizar_checker::source_statement::SourceStatementWitnessKind::Named
                    }
                    14 => {
                        row.name = Some(
                            mizar_checker::source_statement::SourceStatementWitnessNameId::new(0),
                        )
                    }
                    15 => {
                        input.witness.module_id =
                            ResolverModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong"))
                    }
                    _ => unreachable!(),
                }
            },
        )
        .expect("witness selector")
        .expect_err("witness mutation");
        let expected = if mutation == 15 {
            "dependency"
        } else {
            "witness 0"
        };
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "witness field {mutation}: {error}"
        );
        replay();
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
        let error = source_statement_b3m2a_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("resolver selector")
        .expect_err("resolver mutation");
        assert!(
            error.to_ascii_lowercase().contains("resolver")
                || error.to_ascii_lowercase().contains("owner")
                || error.to_ascii_lowercase().contains("label"),
            "{mutation:?}: {error}"
        );
        replay();
    }

    let primary_input = task258b3m2a_primary_input(
        baseline
            .typed_ast
            .source_term()
            .expect("baseline primary terms"),
    );
    let binding_env = baseline
        .typed_ast
        .source_statement()
        .expect("baseline statement")
        .binding_env();
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
            binding_env,
            baseline.typed_ast.nodes(),
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

    for index in 0..49 {
        for mutation in 0..5 {
            let error = source_statement_b3m2a_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2A_TEXT,
                |input| {
                    input.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        input.arena.root(),
                        input
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == index {
                                    match mutation {
                                        0 => {
                                            let mizar_session::SourceAnchor::Range(mut range) =
                                                row.anchor.clone()
                                            else {
                                                unreachable!("range anchor")
                                            };
                                            range.end += 1;
                                            row.anchor = mizar_session::SourceAnchor::Range(range);
                                        }
                                        1 => row.recovery =
                                            mizar_checker::typed_ast::NodeRecoveryState::Recovered,
                                        2 => row.recovery =
                                            mizar_checker::typed_ast::NodeRecoveryState::Degraded,
                                        3 => row.kind = "source.task258b3m2a.mutated".into(),
                                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                                        4 if row.children.len() == 1 => row.children.clear(),
                                        4 => row.children.push(
                                            mizar_checker::typed_ast::TypedNodeId::new(
                                                usize::from(index == 0),
                                            ),
                                        ),
                                        _ => unreachable!(),
                                    }
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("structurally valid arena mutation");
                },
            )
            .expect("arena selector")
            .expect_err("arena mutation");
            assert!(!error.is_empty(), "node {index} mutation {mutation}");
        }
        replay();
    }
}

#[test]
fn task258b3m2a_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2A_TEXT, 258_903);
    let replay = source_statement_b3m2a_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2A_TEXT,
        |symbols| symbols,
    )
    .expect("resolver selector")
    .expect("resolver replay");
    assert_eq!(
        (
            replay.typed_ast.nodes().len(),
            replay.reference_use_ordinals.len(),
        ),
        (49, 4)
    );
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
        if ["single variable witness", "identifier", "named witness", "multiple witness"].contains(&label) {
            let (typed, _) = task258_native_output(&near_ast, &near_module);
            assert_eq!(typed.source_statement().unwrap().statements().len(), 2);
            assert_eq!(typed.source_statement_witnesses().unwrap().witnesses().len(), if label == "multiple witness" { 2 } else { 1 });
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
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2A_TEXT,
    )
    .expect("B3M2A selector")
    .expect("B3M2A route");
    let statement = output
        .typed_ast
        .source_statement()
        .expect("B3M2A statement")
        .clone();
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("B3M2A witnesses")
        .clone();
    let baseline_debug = output.typed_ast.debug_text();

    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
        SOURCE_STATEMENT_B3_TEXT,
        SOURCE_STATEMENT_B3N_TEXT,
        SOURCE_STATEMENT_B3M1_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (foreign_ast, foreign_module, _, foreign_symbols) =
            task253_ast_from_source_text(source, 258_970 + ordinal);
        let (foreign_typed, _) = if source == TASK258A_SOURCE || source == SOURCE_STATEMENT_B1_TEXT || source == SOURCE_STATEMENT_B2_TEXT || source == SOURCE_STATEMENT_B3_TEXT || source == SOURCE_STATEMENT_B3N_TEXT || source == SOURCE_STATEMENT_B3M1_TEXT {
            task258_native_output(&foreign_ast, &foreign_module)
        } else {
            let output = source_statement_output_with_source(
            &foreign_ast,
            foreign_module,
            &foreign_symbols,
            source,
        )
        .expect("foreign selector")
        .expect("foreign route");
            (output.typed_ast, output.resolved)
        };
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
                output.typed_ast.clone().with_source_statement_references(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    references.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M2A-first reference family {ordinal}"
            );
        } else if let Some(foreign_witnesses) = foreign_typed.source_statement_witnesses() {
            assert_eq!(
                output.typed_ast.clone().with_source_statement_witnesses(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    foreign_witnesses.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M2A-first witness family {ordinal}"
            );
        } else {
            assert_eq!(
                output.typed_ast.clone().with_source_statement(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M2A-first base family {ordinal}"
            );
        }
        let foreign_binding = foreign_typed
            .source_statement()
            .expect("foreign statement")
            .binding_env()
            .clone();
        let foreign_primary = foreign_typed
            .source_term()
            .expect("foreign primary")
            .clone();
        let foreign_atomic = foreign_typed
            .source_atomic_formula()
            .expect("foreign atomic")
            .clone();
        let error = source_statement_b3m2a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
            move |input| {
                input.binding_env = foreign_binding;
                input.primary = foreign_primary;
                input.atomic = foreign_atomic;
            },
        )
        .expect("cross-family selector")
        .expect_err("cross-family lower tuple");
        assert!(
            error.to_ascii_lowercase().contains("dependency"),
            "foreign family {ordinal}: {error}"
        );
        assert_eq!(output.typed_ast.debug_text(), baseline_debug);
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

#[test]
fn task258b3m2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2A_TEXT, 258_905);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2A_TEXT,
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
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert_eq!(
        output.typed_ast.source_atomic_formula(),
        output.resolved.source_atomic_formula()
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
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());

    let statement = output.typed_ast.source_statement().expect("statement");
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(_, fact)| {
                fact.uses()
                    .iter()
                    .map(|reference| reference.index())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        [vec![0, 1], vec![2, 3]]
    );
    let primary = output.typed_ast.source_term().expect("primary");
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, reference)| reference.term().index())
            .collect::<Vec<_>>(),
        [0, 1, 3, 4]
    );
    assert_eq!(
        source_statement_transport_detail_keys(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2A_TEXT,
        ),
        Some(Vec::new())
    );

    let baseline_typed = output.typed_ast.debug_text();
    let baseline_resolved = output.resolved.debug_text();
    let witness_error = source_statement_b3m2a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2A_TEXT,
        |input| input.witness.witnesses[0].spelling.push('z'),
    )
    .expect("rollback selector")
    .expect_err("invalid witness must roll back");
    assert!(
        witness_error.to_ascii_lowercase().contains("witness 0"),
        "{witness_error}"
    );

    let mut numeric_input = task258b3m2a_primary_input(primary);
    numeric_input.numeric_type_requests[0].spelling.push('z');
    assert!(matches!(
        mizar_checker::source_term::SourcePrimaryTermProducer::build(
            numeric_input,
            statement.binding_env(),
            output.typed_ast.nodes(),
        ),
        Err(
            mizar_checker::source_term::SourcePrimaryTermError::InvalidNumericTypeRequest {
                request
            }
        ) if request.index() == 0
    ));

    let replay =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B3M2A_TEXT)
            .expect("replay selector")
            .expect("replay");
    assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
    assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    assert_eq!(
        replay.typed_ast.source_statement_witnesses(),
        output.typed_ast.source_statement_witnesses()
    );
    assert_eq!(
        replay.typed_ast.source_term(),
        output.typed_ast.source_term()
    );
}
