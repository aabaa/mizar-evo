use super::*;

#[test]
fn task258b3m2a_exact_numeral_witness_api_debug_and_compatibility_are_stable() {
    let fixture = B3M2AFixture::new(61);
    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2a_profile());
    assert!(!statement.is_task_258b3_profile());
    assert!(!statement.is_task_258b3n_profile());
    assert!(!statement.is_task_258b3m1_profile());
    assert_eq!(
        (
            fixture.bindings.contexts().len(),
            fixture.bindings.bindings().len(),
            fixture.bindings.diagnostics().len(),
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len(),
            fixture.atomic.formulas().len(),
            fixture.atomic.wrappers().len(),
            fixture.atomic.edges().len(),
            fixture.atomic.requests().len(),
        ),
        (2, 1, 0, 5, 4, 1, 2, 0, 4, 4)
    );
    let proof = fixture
        .bindings
        .contexts()
        .get(BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        (
            &proof.owner,
            proof.parent,
            proof.layer,
            proof
                .lexical_scope
                .as_ref()
                .map(|scope| scope.path().to_vec()),
            proof.bindings.as_slice(),
            proof.visible_bindings.as_slice(),
            proof.recovery,
        ),
        (
            &BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 70, 105),
            },
            Some(BindingContextId::new(0)),
            BindingContextLayer::Proof,
            Some(vec![0]),
            &[][..],
            &[BindingId::new(0)][..],
            BindingContextRecovery::Normal,
        )
    );
    for index in 0..5 {
        let term = fixture
            .primary
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .expect("term");
        assert_eq!(
            (
                term.site().node().index(),
                term.source_range(),
                term.source_ordinal(),
                term.context().index(),
                term.spelling(),
                term.kind(),
                term.role(),
                term.recovery(),
                term.parent(),
            ),
            (
                [26, 28, 32, 36, 38][index],
                range(
                    fixture.source,
                    [64, 68, 83, 95, 99][index],
                    [65, 69, 86, 96, 100][index],
                ),
                index,
                [0, 0, 1, 1, 1][index],
                if index == 2 { "101" } else { "x" },
                if index == 2 {
                    SourcePrimaryTermKind::Numeral
                } else {
                    SourcePrimaryTermKind::VariableReference
                },
                SourcePrimaryTermRole::Value,
                SourcePrimaryTermRecovery::Normal,
                None,
            )
        );
    }
    for (reference_index, term_index) in [0, 1, 3, 4].into_iter().enumerate() {
        let reference = fixture
            .primary
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
            .expect("dense reference");
        assert_eq!(
            (
                reference.term().index(),
                reference.binding().index(),
                reference.role(),
                reference.use_ordinal(),
                reference.lexical_scope().map(|scope| scope.path().to_vec()),
            ),
            (
                term_index,
                0,
                SourcePrimaryTermReferenceRole::Variable,
                1,
                if term_index < 2 { None } else { Some(vec![0]) },
            )
        );
    }
    let numeric = fixture
        .primary
        .numeric_type_requests()
        .get(crate::source_term::SourceNumericTypeRequestId::new(0))
        .expect("numeric request");
    assert_eq!(
        (
            numeric.term().index(),
            numeric.owner().node().index(),
            numeric.source_range(),
            numeric.spelling(),
            numeric.request_ordinal(),
        ),
        (2, 32, range(fixture.source, 83, 86), "101", 0)
    );
    assert_eq!(
        fixture
            .atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                SourceAtomicTermTarget::Primary(term) => term.index(),
                _ => usize::MAX,
            })
            .collect::<Vec<_>>(),
        [0, 1, 3, 4]
    );
    for index in 0..4 {
        let formula = usize::from(index >= 2);
        let ordinal = index % 2;
        let request = fixture
            .atomic
            .requests()
            .get(SourceAtomicRequestId::new(index))
            .expect("request");
        assert_eq!(
            (
                request.formula().index(),
                request.ordinal(),
                request.kind(),
                request.edge().map(|id| id.index()),
                request.candidate(),
                request.type_site(),
                request.attribute(),
            ),
            (
                formula,
                ordinal,
                SourceAtomicRequestKind::OperandExpectedType,
                Some(index),
                None,
                None,
                None,
            )
        );
    }
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            statement
                .input_facts()
                .get(SourceStatementInputFactId::new(0))
                .expect("theorem fact")
                .uses()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            statement
                .input_facts()
                .get(SourceStatementInputFactId::new(1))
                .expect("conclusion fact")
                .uses()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
        ),
        (1, 2, 2, 2, 2, vec![0, 1], vec![2, 3])
    );
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(
        (
            owner.symbol(),
            owner.contribution(),
            owner.site().node().index(),
            owner.source_range(),
            owner.spelling(),
            owner.role(),
            owner.status(),
            owner.recovery(),
            statement.checked_owner().origin().structural_path(),
        ),
        (
            &fixture.symbol,
            fixture.contribution,
            45,
            range(fixture.source, 19, 106),
            "FormulaStatementNumeralWitnessSmoke",
            SourceTheoremRole::Theorem,
            SourceTheoremStatus::Unmodified,
            SourceStatementRecovery::Normal,
            &[2, 1][..],
        )
    );
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witness handoff");
    assert_eq!(
        (witnesses.witnesses().len(), witnesses.names().len()),
        (1, 0)
    );
    let witness = witnesses
        .witnesses()
        .get(SourceStatementWitnessId::new(0))
        .expect("witness");
    assert_eq!(
        (
            witness.owner().index(),
            witness.binding_context().index(),
            witness.term(),
            witness.take_site().node().index(),
            witness.take_range(),
            witness.site().node().index(),
            witness.source_range(),
            witness.source_ordinal(),
            witness.ordinal(),
        ),
        (
            0,
            1,
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
            35,
            range(fixture.source, 78, 87),
            34,
            range(fixture.source, 83, 86),
            1,
            0,
        )
    );
    assert_eq!(
        (
            witness.spelling(),
            witness.kind(),
            witness.recovery(),
            witness.name(),
        ),
        (
            "101",
            SourceStatementWitnessKind::Unnamed,
            SourceStatementRecovery::Normal,
            None,
        )
    );
    assert_eq!(
        witnesses.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=78..87 take_site=35 range=83..86 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"101\"\n",
            statement.debug_text(),
            fixture.primary.debug_text(),
        )
    );
    for (index, (start, end)) in TASK258B3M2A_NODE_RANGES.iter().copied().enumerate() {
        let arena_node = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("arena node");
        assert_eq!(
            (
                &arena_node.anchor,
                arena_node.kind.as_str(),
                arena_node.recovery,
                arena_node
                    .children
                    .iter()
                    .map(|child| child.index())
                    .collect::<Vec<_>>(),
            ),
            (
                &SourceAnchor::Range(range(fixture.source, start, end)),
                task258b3m2a_node_kind(index),
                NodeRecoveryState::Normal,
                task258b3m2a_node_children(index).to_vec(),
            ),
            "node {index}"
        );
    }
    for prior_debug in [
        {
            let prior = B3Fixture::new(62);
            let prior_statement = prior.statement();
            prior
                .witnesses(&prior_statement, prior.witness_input())
                .expect("B3")
                .debug_text()
        },
        {
            let prior = B3NFixture::new(63);
            let prior_statement = prior.statement();
            prior
                .witnesses(&prior_statement, prior.witness_input())
                .expect("B3N")
                .debug_text()
        },
        {
            let prior = B3M1Fixture::new(64);
            let prior_statement = prior.statement();
            prior
                .witnesses(&prior_statement, prior.witness_input())
                .expect("B3M1")
                .debug_text()
        },
    ] {
        assert!(prior_debug.starts_with("source-statement-witness-debug-v1\n"));
    }
}

#[test]
fn task258b3m2a_dependencies_numeric_request_witness_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2AFixture::new(65);
    let statement = fixture.statement();
    let baseline = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("baseline");
    let baseline_debug = baseline.debug_text();
    let replay = || {
        assert_eq!(
            fixture
                .witnesses(&statement, fixture.witness_input())
                .expect("pristine replay")
                .debug_text(),
            baseline_debug
        );
    };

    for mutation in 0..2 {
        let mut input = fixture.witness_input();
        if mutation == 0 {
            input.witnesses.clear();
        } else {
            input.witnesses.push(input.witnesses[0].clone());
        }
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidAggregate),
            "aggregate {mutation}"
        );
        replay();
    }
    let mut extra_name = fixture.witness_input();
    extra_name.names.push(SourceStatementWitnessNameInput {
        witness: SourceStatementWitnessId::new(0),
        site: node(15),
        source_range: range(fixture.source, 97, 98),
        spelling: "x".to_owned(),
        recovery: SourceStatementRecovery::Normal,
    });
    assert_eq!(
        fixture.witnesses(&statement, extra_name),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    replay();
    for mutation in 0..15 {
        let mut input = fixture.witness_input();
        let row = &mut input.witnesses[0];
        match mutation {
            0 => row.owner = SourceTheoremOwnerId::new(1),
            1 => row.binding_context = BindingContextId::new(0),
            2 => row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3)),
            3 => row.take_site = node(34),
            4 => row.take_range.start += 1,
            5 => row.take_range.end -= 1,
            6 => row.site = node(33),
            7 => row.source_range.start += 1,
            8 => row.source_range.end -= 1,
            9 => row.source_ordinal = 2,
            10 => row.ordinal = 1,
            11 => row.spelling = "102".to_owned(),
            12 => row.kind = SourceStatementWitnessKind::Named,
            13 => row.recovery = SourceStatementRecovery::Degraded,
            14 => row.name = Some(SourceStatementWitnessNameId::new(0)),
            _ => unreachable!(),
        }
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            }),
            "witness field {mutation}"
        );
        replay();
    }
    for term in [0, 1, 3, 4, 5, 6] {
        let mut input = fixture.witness_input();
        input.witnesses[0].term =
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(term));
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            }),
            "witness term substitution {term}"
        );
        replay();
    }
    let mut foreign_context = fixture.witness_input();
    foreign_context.witnesses[0].binding_context = BindingContextId::new(2);
    assert_eq!(
        fixture.witnesses(&statement, foreign_context),
        Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0),
        })
    );
    replay();
    let mut aggregate_first = fixture.witness_input();
    aggregate_first.witnesses.clear();
    aggregate_first.names.push(SourceStatementWitnessNameInput {
        witness: SourceStatementWitnessId::new(0),
        site: node(13),
        source_range: range(fixture.source, 83, 86),
        spelling: "101".to_owned(),
        recovery: SourceStatementRecovery::Normal,
    });
    assert_eq!(
        fixture.witnesses(&statement, aggregate_first),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    replay();
    let mut aggregate_before_witness = fixture.witness_input();
    aggregate_before_witness.witnesses[0].ordinal = 1;
    aggregate_before_witness
        .witnesses
        .push(aggregate_before_witness.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, aggregate_before_witness),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    replay();
    let mut dependency_first = fixture.witness_input();
    dependency_first.source_id = source_id(66);
    dependency_first.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_first),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    replay();

    let mut stale_statement = baseline.clone();
    stale_statement.statement_fingerprint.push('x');
    assert_eq!(
        stale_statement.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    replay();
    let mut stale_primary = baseline.clone();
    stale_primary.primary_term_fingerprint.push('x');
    assert_eq!(
        stale_primary.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    replay();

    let mut numeric_input = B3M2AFixture::primary_input(fixture.source, &fixture.module);
    numeric_input.terms[2].spelling = "102".to_owned();
    numeric_input.numeric_type_requests[0].spelling = "102".to_owned();
    let altered_primary =
        SourcePrimaryTermProducer::build(numeric_input, &fixture.bindings, &fixture.arena)
            .expect("internally valid altered numeric request");
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &fixture.bindings,
            &altered_primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    replay();

    for index in 0..49 {
        for mutation in 0..5 {
            let arena = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    match mutation {
                        0 => {
                            let SourceAnchor::Range(mut source_range) = row.anchor.clone() else {
                                unreachable!("Task258B3M2A range")
                            };
                            source_range.end += 1;
                            row.anchor = SourceAnchor::Range(source_range);
                        }
                        1 => row.recovery = NodeRecoveryState::Recovered,
                        2 => row.recovery = NodeRecoveryState::Degraded,
                        3 => row.kind = "source.task258b3m2a.mutated".into(),
                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                        4 if row.children.len() == 1 => row.children.clear(),
                        4 => row.children.push(TypedNodeId::new(usize::from(index == 0))),
                        _ => unreachable!(),
                    }
                }
            });
            assert_eq!(
                baseline.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} mutation {mutation}"
            );
            replay();
        }
    }
    assert_eq!(
        fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("replay")
            .debug_text(),
        baseline_debug
    );
}

#[test]
fn task258b3m2a_statement_dependencies_and_candidate_aggregate_precede_witness_rows() {
    let fixture = B3M2AFixture::new(72);
    let foreign = B3Fixture::new(73);
    let statement = fixture.statement();
    let pristine = fixture.witness_input();
    let baseline = fixture
        .witnesses(&statement, pristine.clone())
        .expect("baseline");
    let replay = || {
        assert_eq!(fixture.statement(), statement);
        assert_eq!(fixture.witness_input(), pristine);
        assert_eq!(
            fixture.witnesses(&statement, fixture.witness_input()),
            Ok(baseline.clone())
        );
    };
    for mutation in 0..4 {
        let mut input = fixture.statement_input();
        let mut witness = fixture.witness_input();
        if mutation == 0 {
            witness.witnesses.clear();
        } else if mutation == 3 {
            input.candidate_facts.pop();
        }
        let built = SourceStatementProducer::build(
            input,
            &fixture.symbols,
            if mutation == 0 {
                &foreign.bindings
            } else {
                &fixture.bindings
            },
            if mutation == 1 {
                &foreign.primary
            } else {
                &fixture.primary
            },
            if mutation == 2 {
                &foreign.atomic
            } else {
                &fixture.atomic
            },
            &fixture.arena,
        );
        assert_eq!(
            built.map(|statement| fixture.witnesses(&statement, witness)),
            Err(if mutation == 3 {
                SourceStatementError::InvalidAggregate
            } else {
                SourceStatementError::DependencyMismatch
            }),
            "statement fault {mutation} before witness admission"
        );
        replay();
    }
    let mut wrong_module = fixture.witness_input();
    wrong_module.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong"));
    assert_eq!(
        fixture.witnesses(&statement, wrong_module),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    replay();
}

#[test]
fn task258b3m2a_paired_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2AFixture::new(67);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let base = fixture.empty_typed();
    let base_debug = base.debug_text();
    assert_eq!(
        base.clone().with_source_statement(statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(base.debug_text(), base_debug);
    let typed = base
        .clone()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("paired install");
    let typed_debug = typed.debug_text();
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert!(typed.source_statement_references().is_none());
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let task258a = Fixture::new(67);
    let task258a_statement = task258a.build(task258a.input()).expect("Task258A");
    let task258a_typed = TypedAst::try_new(TypedAstParts {
        source_id: task258a.source,
        module_id: task258a.module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: task258a.arena.clone(),
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("Task258A empty")
    .with_source_term(task258a.primary.clone())
    .expect("Task258A Task252")
    .with_source_atomic_formula(task258a.atomic.clone())
    .expect("Task258A Task256")
    .with_source_statement(task258a_statement.clone())
    .expect("Task258A install");
    assert_eq!(
        task258a_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.clone().with_source_statement(task258a_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b1 = B1Fixture::new(67);
    let b1_references = b1.references(b1.reference_input()).expect("B1 references");
    let b1_typed = b1
        .empty_typed()
        .with_source_statement_references(b1.statement.clone(), b1_references.clone())
        .expect("B1");
    assert_eq!(
        b1_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_statement_references(b1.statement.clone(), b1_references,),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b2 = B2Fixture::new(67);
    let b2_statement = b2.build(b2.input()).expect("B2");
    let b2_typed = b2
        .empty_typed()
        .with_source_statement(b2_statement.clone())
        .expect("B2 install");
    assert_eq!(
        b2_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.clone().with_source_statement(b2_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b3 = B3Fixture::new(68);
    let b3_statement = b3.statement();
    let b3_witnesses = b3.witnesses(&b3_statement, b3.witness_input()).expect("B3");
    let b3_typed = b3
        .empty_typed()
        .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
        .expect("B3 install");
    assert_eq!(
        b3_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(b3_statement, b3_witnesses),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b3n = B3NFixture::new(69);
    let b3n_statement = b3n.statement();
    let b3n_witnesses = b3n
        .witnesses(&b3n_statement, b3n.witness_input())
        .expect("B3N");
    let b3n_typed = b3n
        .empty_typed()
        .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone())
        .expect("B3N install");
    assert_eq!(
        b3n_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(b3n_statement, b3n_witnesses),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b3m1 = B3M1Fixture::new(70);
    let b3m1_statement = b3m1.statement();
    let b3m1_witnesses = b3m1
        .witnesses(&b3m1_statement, b3m1.witness_input())
        .expect("B3M1");
    let b3m1_typed = b3m1
        .empty_typed()
        .with_source_statement_witnesses(b3m1_statement.clone(), b3m1_witnesses.clone())
        .expect("B3M1 install");
    assert_eq!(
        b3m1_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(b3m1_statement, b3m1_witnesses),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
        fixture.source,
        fixture.module.clone(),
    );
    assert_eq!(
        task248
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.clone().with_source_context_for_test(
            task248.source_context().expect("Task248 context").clone(),
        ),
        Err(TypedAstError::InvalidSourceContext)
    );
    let task257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
    let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
    let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
    let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
    for (family, occupied) in [
        ("Task257A", &task257a),
        ("Task257B", &task257b),
        ("Task257C2", &task257c2),
        ("Task257C3", &task257c3),
    ] {
        assert_eq!(
            occupied
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "{family}"
        );
    }
    assert_eq!(
        typed.clone().with_source_composite_formula(
            task257a
                .source_composite_formula()
                .expect("Task257A composite")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceCompositeFormula)
    );
    assert_eq!(
        typed.clone().with_source_formula_composition(
            task257b
                .source_composite_formula()
                .expect("Task257B composite")
                .clone(),
            task257b
                .source_formula_composition()
                .expect("Task257B composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceFormulaComposition)
    );
    assert_eq!(
        typed.clone().with_source_condition_formula_composition(
            task257c2
                .source_condition_formula_composition()
                .expect("Task257C2 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceConditionFormulaComposition)
    );
    assert_eq!(
        typed.clone().with_source_predicate_chain_composition(
            task257c3
                .source_predicate_chain_composition()
                .expect("Task257C3 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourcePredicateChainComposition)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        base.with_source_statement_witnesses(statement, witnesses)
            .expect("replay")
            .debug_text(),
        typed_debug
    );
}

#[test]
fn task258b3m2a_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2AFixture::new(71);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("typed");
    let typed_debug = typed.debug_text();
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
    let resolved_debug = resolved.debug_text();
    assert_eq!(typed.clone().debug_text(), typed_debug);
    assert_eq!(resolved.clone().debug_text(), resolved_debug);
    assert_eq!(resolved.source_statement(), Some(&statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
    assert!(resolved.source_statement_references().is_none());
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

    let mut orphan = fixture.empty_typed();
    orphan.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&orphan),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut standalone = fixture.empty_typed();
    standalone.inject_source_statement_for_test(statement.clone());
    assert_eq!(
        assemble_empty_resolved(&standalone),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut stale_statement = witnesses.clone();
    stale_statement.statement_fingerprint.push('x');
    let mut stale_statement_pair = fixture.empty_typed();
    stale_statement_pair
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_statement);
    assert_eq!(
        assemble_empty_resolved(&stale_statement_pair),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut stale_primary = witnesses.clone();
    stale_primary.primary_term_fingerprint.push('x');
    let mut stale_primary_pair = fixture.empty_typed();
    stale_primary_pair
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_primary);
    assert_eq!(
        assemble_empty_resolved(&stale_primary_pair),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let b1 = B1Fixture::new(71);
    let mut reference_hybrid = fixture.empty_typed();
    reference_hybrid.inject_source_statement_bundle_for_test(
        statement.clone(),
        b1.references(b1.reference_input()).expect("B1 references"),
    );
    reference_hybrid.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&reference_hybrid),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    for table in [
        StatementTransportTableForTest::Context,
        StatementTransportTableForTest::Type,
        StatementTransportTableForTest::Fact,
        StatementTransportTableForTest::Coercion,
        StatementTransportTableForTest::InitialObligation,
        StatementTransportTableForTest::Diagnostic,
    ] {
        let mut occupied = typed.clone();
        occupied.occupy_statement_transport_table_for_test(table);
        assert_eq!(
            assemble_empty_resolved(&occupied),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{table:?}"
        );
    }
    let term_formula = TermFormulaChecker::default().infer(
        &fixture.symbols,
        &fixture.bindings,
        Vec::<crate::type_checker::TermInput>::new(),
        Vec::<crate::type_checker::FormulaInput>::new(),
    );
    assert_eq!(
        assemble_resolved_with_statement_semantics(
            &typed,
            statement.checked_owner(),
            &fixture.bindings,
            &term_formula,
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("task258b3m2a-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 83, 86),
    });
    assert_eq!(
        assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved(
            &typed,
            &ClusterFactTable::new(),
            vec![ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(32),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new(
                        "source.statement-witness-numeral.semantic-coexistence",
                    ),
                },
            }],
            None,
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved(
            &typed,
            &ClusterFactTable::new(),
            Vec::new(),
            Some(StatementProofInputs {
                owner: statement.checked_owner(),
                rows: Vec::new(),
            }),
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved_with_expressions(
            &typed,
            vec![ExpressionMetadataInput {
                expr: ExprId::new("task258b3m2a-semantic-coexistence"),
                typed_site: node(32),
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("final replay")
            .debug_text(),
        resolved_debug
    );
}
