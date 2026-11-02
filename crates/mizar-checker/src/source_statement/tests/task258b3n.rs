use super::*;

#[test]
fn task258b3n_exact_name_api_debug_and_b3_byte_compatibility_are_stable() {
    let fixture = B3NFixture::new(44);
    let statement = fixture.statement();
    assert!(statement.is_task_258b3n_profile());
    assert!(!statement.is_task_258b3_profile());
    let handoff = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task258B3N witness handoff");
    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!((handoff.witnesses().len(), handoff.names().len()), (1, 1));
    let witness_id = SourceStatementWitnessId::new(0);
    let name_id = SourceStatementWitnessNameId::new(0);
    assert_eq!((witness_id.index(), name_id.index()), (0, 0));
    let witness = handoff.witnesses().get(witness_id).expect("witness");
    assert_eq!(witness.owner(), SourceTheoremOwnerId::new(0));
    assert_eq!(witness.binding_context(), BindingContextId::new(1));
    assert_eq!(
        witness.term(),
        SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
    );
    assert_eq!(
        (
            witness.take_site().node().index(),
            witness.site().node().index()
        ),
        (37, 36)
    );
    assert_eq!(witness.take_range(), range(fixture.source, 76, 87));
    assert_eq!(witness.source_range(), range(fixture.source, 81, 86));
    assert_eq!((witness.source_ordinal(), witness.ordinal()), (1, 0));
    assert_eq!(witness.spelling(), "y = x");
    assert_eq!(witness.kind(), SourceStatementWitnessKind::Named);
    assert_eq!(witness.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(witness.name(), Some(name_id));
    let name = handoff.names().get(name_id).expect("name");
    assert_eq!(name.witness(), witness_id);
    assert_eq!(name.site().node().index(), 13);
    assert_eq!(name.source_range(), range(fixture.source, 81, 82));
    assert_eq!(name.spelling(), "y");
    assert_eq!(name.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(
        handoff
            .names()
            .iter()
            .map(|(id, row)| (id.index(), row.spelling()))
            .collect::<Vec<_>>(),
        [(0, "y")]
    );
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=76..87 take_site=37 range=81..86 site=36 source_ordinal=1 ordinal=0 kind=named recovery=normal spelling=\"y = x\" name=0\nwitness-name#0 witness=0 range=81..82 site=13 recovery=normal spelling=\"y\"\n",
            statement.debug_text(),
            fixture.primary.debug_text(),
        )
    );
    assert_eq!(
        (
            fixture.bindings.contexts().len(),
            fixture.bindings.bindings().len(),
            fixture.bindings.diagnostics().len(),
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len(),
        ),
        (2, 1, 0, 5, 5, 0)
    );
    assert_eq!(
        (
            fixture.atomic.formulas().len(),
            fixture.atomic.wrappers().len(),
            fixture.atomic.predicate_segments().len(),
            fixture.atomic.predicate_heads().len(),
            fixture.atomic.candidates().len(),
            fixture.atomic.type_sites().len(),
            fixture.atomic.attributes().len(),
            fixture.atomic.edges().len(),
            fixture.atomic.requests().len(),
        ),
        (2, 0, 0, 0, 0, 0, 0, 4, 4)
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
            proof
                .bindings
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            proof
                .visible_bindings
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            proof.recovery,
        ),
        (
            &BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 68, 105),
            },
            Some(BindingContextId::new(0)),
            BindingContextLayer::Proof,
            Some(vec![0]),
            Vec::new(),
            vec![0],
            BindingContextRecovery::Normal,
        )
    );
    for (index, ((_, term), (_, reference))) in fixture
        .primary
        .terms()
        .iter()
        .zip(fixture.primary.references().iter())
        .enumerate()
    {
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
                [28, 30, 34, 38, 40][index],
                range(
                    fixture.source,
                    [62, 66, 85, 95, 99][index],
                    [63, 67, 86, 96, 100][index],
                ),
                index,
                [0, 0, 1, 1, 1][index],
                "x",
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
                SourcePrimaryTermRecovery::Normal,
                None,
            )
        );
        assert_eq!(
            (
                reference.term().index(),
                reference.binding().index(),
                reference.role(),
                reference.use_ordinal(),
                reference.lexical_scope().map(|scope| scope.path().to_vec()),
            ),
            (
                index,
                0,
                SourcePrimaryTermReferenceRole::Variable,
                1,
                if index < 2 { None } else { Some(vec![0]) },
            )
        );
    }
    for index in 0..2 {
        let formula = fixture
            .atomic
            .formulas()
            .get(SourceAtomicFormulaId::new(index))
            .expect("formula");
        assert_eq!(
            (
                formula.site().node().index(),
                formula.source_range(),
                formula.source_ordinal(),
                formula.context().index(),
                formula.spelling(),
                formula.kind(),
                formula.recovery(),
            ),
            (
                [32, 42][index],
                range(fixture.source, [62, 95][index], [67, 100][index]),
                index,
                index,
                "x = x",
                SourceAtomicFormulaKind::Equality,
                SourceAtomicFormulaRecovery::Normal,
            )
        );
    }
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
        let edge = fixture
            .atomic
            .edges()
            .get(SourceAtomicEdgeId::new(index))
            .expect("edge");
        assert_eq!((edge.formula().index(), edge.ordinal()), (formula, ordinal));
        assert_eq!(
            edge.role(),
            if ordinal == 0 {
                SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
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
        ),
        (1, 2, 2, 2, 2)
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
        ),
        (
            &fixture.symbol,
            fixture.contribution,
            47,
            range(fixture.source, 19, 106),
            "FormulaStatementNamedWitnessSmoke",
            SourceTheoremRole::Theorem,
            SourceTheoremStatus::Unmodified,
            SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(statement.checked_owner().symbol(), &fixture.symbol);
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    let labels = fixture
        .symbols
        .labels()
        .by_contribution(fixture.contribution);
    assert_eq!(labels.len(), 1);
    assert_eq!(
        (
            labels[0].primary_spelling(),
            labels[0].kind(),
            labels[0].visibility(),
            labels[0].export_status(),
            labels[0].contribution(),
            labels[0].origin().structural_path(),
        ),
        (
            "FormulaStatementNamedWitnessSmoke",
            mizar_resolve::resolved_ast::LabelKind::Theorem,
            Visibility::Public,
            ExportStatus::Exported,
            fixture.contribution,
            &[2, 1][..],
        )
    );
    assert!(fixture.symbols.imports().is_empty());
    for index in 0..2 {
        let row = statement
            .statements()
            .get(SourceStatementId::new(index))
            .expect("statement row");
        assert_eq!(
            (
                row.owner().index(),
                row.context().index(),
                row.formula(),
                row.site().node().index(),
                row.source_range(),
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
                row.recovery(),
            ),
            (
                0,
                index,
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index)),
                [47, 45][index],
                range(fixture.source, [19, 90][index], [106, 101][index]),
                [0, 2][index],
                [
                    "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
                    "thus x = x ;",
                ][index],
                [
                    SourceStatementKind::TheoremProposition,
                    SourceStatementKind::Conclusion,
                ][index],
                SourceStatementRecovery::Normal,
            )
        );
        let context = statement
            .contexts()
            .get(SourceStatementContextId::new(index))
            .expect("statement context");
        assert_eq!(
            (
                context.statement().index(),
                context.binding_context().index(),
                context.source_range(),
                context
                    .visible_bindings()
                    .iter()
                    .map(|id| id.index())
                    .collect::<Vec<_>>(),
            ),
            (index, index, row.source_range(), vec![0])
        );
        let fact = statement
            .input_facts()
            .get(SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(
            (
                fact.statement().index(),
                fact.context().index(),
                fact.ordinal(),
                fact.kind(),
                fact.binding().index(),
                fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
            ),
            (
                index,
                index,
                0,
                SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                if index == 0 { vec![0, 1] } else { vec![3, 4] },
            )
        );
        let candidate = statement
            .candidate_facts()
            .get(SourceStatementCandidateFactId::new(index))
            .expect("candidate fact");
        assert_eq!(
            (
                candidate.statement().index(),
                candidate.context().index(),
                candidate.ordinal(),
                candidate.kind(),
                candidate.formula(),
            ),
            (
                index,
                index,
                0,
                SourceStatementCandidateFactKind::UnverifiedProposition,
                row.formula(),
            )
        );
    }
    let typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), handoff.clone())
        .expect("paired typed publication");
    let resolved = assemble_empty_resolved(&typed).expect("paired final publication");
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&handoff));
    assert!(typed.source_statement_references().is_none());
    assert_eq!(resolved.source_statement(), Some(&statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(&handoff));
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

    let b3 = B3Fixture::new(45);
    let b3_statement = b3.statement();
    let b3_handoff = b3
        .witnesses(&b3_statement, b3.witness_input())
        .expect("B3 handoff");
    assert!(b3_handoff.names().is_empty());
    assert_eq!(
        b3_handoff.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\"\n",
            b3_statement.debug_text(),
            b3.primary.debug_text(),
        )
    );
}

#[test]
fn task258b3n_dependency_aggregate_witness_name_and_all_nodes_fail_closed() {
    let fixture = B3NFixture::new(46);
    let pristine_input = fixture.statement_input();
    let pristine_witness_input = fixture.witness_input();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let statement = fixture.statement();
    let baseline = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("baseline");
    let baseline_debug = baseline.debug_text();
    let baseline_typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), baseline.clone())
        .expect("canonical paired install");
    let baseline_typed_debug = baseline_typed.debug_text();
    let baseline_final_debug = assemble_empty_resolved(&baseline_typed)
        .expect("canonical final assembly")
        .debug_text();
    let assert_replay = || {
        assert_eq!(fixture.statement_input(), pristine_input);
        assert_eq!(fixture.witness_input(), pristine_witness_input);
        assert_eq!(fixture.symbols, pristine_symbols);
        assert_eq!(fixture.bindings, pristine_bindings);
        assert_eq!(fixture.primary, pristine_primary);
        assert_eq!(fixture.atomic, pristine_atomic);
        assert_eq!(fixture.arena, pristine_arena);
        let replay_statement = SourceStatementProducer::build(
            pristine_input.clone(),
            &pristine_symbols,
            &pristine_bindings,
            &pristine_primary,
            &pristine_atomic,
            &pristine_arena,
        )
        .expect("canonical statement replay");
        assert_eq!(replay_statement.debug_text(), statement.debug_text());
        let replay_witness = SourceStatementWitnessProducer::build(
            pristine_witness_input.clone(),
            &replay_statement,
            &pristine_primary,
            &pristine_arena,
        )
        .expect("canonical witness replay");
        assert_eq!(replay_witness.debug_text(), baseline_debug);
        let replay_typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(replay_statement, replay_witness)
            .expect("canonical paired replay");
        assert_eq!(replay_typed.debug_text(), baseline_typed_debug);
        assert_eq!(
            assemble_empty_resolved(&replay_typed)
                .expect("canonical final replay")
                .debug_text(),
            baseline_final_debug
        );
    };

    let mut aggregate = fixture.witness_input();
    aggregate.names.clear();
    assert_eq!(
        fixture.witnesses(&statement, aggregate),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_replay();
    let mut empty_witnesses = fixture.witness_input();
    empty_witnesses.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, empty_witnesses),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_replay();
    let mut duplicate_witness = fixture.witness_input();
    duplicate_witness
        .witnesses
        .push(duplicate_witness.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, duplicate_witness),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_replay();
    let mut duplicate_name = fixture.witness_input();
    duplicate_name.names.push(duplicate_name.names[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, duplicate_name),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_replay();
    let mut invalid_witness = fixture.witness_input();
    invalid_witness.witnesses[0].kind = SourceStatementWitnessKind::Unnamed;
    assert_eq!(
        fixture.witnesses(&statement, invalid_witness),
        Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0)
        })
    );
    assert_replay();
    let mut invalid_forward = fixture.witness_input();
    invalid_forward.witnesses[0].name = Some(SourceStatementWitnessNameId::new(1));
    assert_eq!(
        fixture.witnesses(&statement, invalid_forward),
        Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0)
        })
    );
    assert_replay();
    let mut invalid_reverse = fixture.witness_input();
    invalid_reverse.names[0].witness = SourceStatementWitnessId::new(1);
    assert_eq!(
        fixture.witnesses(&statement, invalid_reverse),
        Err(SourceStatementWitnessError::InvalidName {
            name: SourceStatementWitnessNameId::new(0)
        })
    );
    assert_replay();
    let mut invalid_name = fixture.witness_input();
    invalid_name.names[0].spelling = "z".to_owned();
    assert_eq!(
        fixture.witnesses(&statement, invalid_name),
        Err(SourceStatementWitnessError::InvalidName {
            name: SourceStatementWitnessNameId::new(0)
        })
    );
    assert_replay();
    let mut aggregate_precedence = fixture.witness_input();
    aggregate_precedence.names.clear();
    aggregate_precedence.witnesses[0].kind = SourceStatementWitnessKind::Unnamed;
    assert_eq!(
        fixture.witnesses(&statement, aggregate_precedence),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_replay();
    let mut witness_precedence = fixture.witness_input();
    witness_precedence.witnesses[0].kind = SourceStatementWitnessKind::Unnamed;
    witness_precedence.names[0].spelling = "z".to_owned();
    assert_eq!(
        fixture.witnesses(&statement, witness_precedence),
        Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0)
        })
    );
    assert_replay();
    let mut precedence = fixture.witness_input();
    precedence.source_id = B3Fixture::new(47).source;
    precedence.names.clear();
    assert_eq!(
        fixture.witnesses(&statement, precedence),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    let mut foreign_witness_module = fixture.witness_input();
    foreign_witness_module.module_id = ModuleId::new(
        PackageId::new("pkg"),
        ModulePath::new("statement.task258b3n.foreign"),
    );
    assert_eq!(
        fixture.witnesses(&statement, foreign_witness_module),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    for (label, foreign_source, foreign_module) in [
        ("source", Some(source_id(1_046)), None),
        (
            "module",
            None,
            Some(ModuleId::new(
                PackageId::new("pkg"),
                ModulePath::new("statement.task258b3n.foreign"),
            )),
        ),
    ] {
        let mut input = fixture.statement_input();
        if let Some(source) = foreign_source {
            input.source_id = source;
        }
        if let Some(module) = foreign_module {
            input.module_id = module;
        }
        let actual = SourceStatementProducer::build(
            input,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        );
        assert_eq!(
            actual,
            Err(SourceStatementError::DependencyMismatch),
            "{label}"
        );
        assert_replay();
    }
    for index in 0..2 {
        let mut input = fixture.statement_input();
        input.statements[index].kind = if index == 0 {
            SourceStatementKind::Conclusion
        } else {
            SourceStatementKind::TheoremProposition
        };
        assert_eq!(
            SourceStatementProducer::build(
                input,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(index),
            }),
            "statement kind {index}"
        );
        assert_replay();
    }
    for mutation in 0..16 {
        let mut input = fixture.witness_input();
        let row = &mut input.witnesses[0];
        match mutation {
            0 => row.owner = SourceTheoremOwnerId::new(1),
            1 => row.binding_context = BindingContextId::new(0),
            2 => row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(1)),
            3 => row.take_site = node(36),
            4 => row.take_range.start += 1,
            5 => row.take_range.end -= 1,
            6 => row.site = node(37),
            7 => row.source_range.start += 1,
            8 => row.source_range.end -= 1,
            9 => row.source_ordinal = 2,
            10 => row.ordinal = 1,
            11 => row.spelling.push('x'),
            12 => row.kind = SourceStatementWitnessKind::Unnamed,
            13 => row.recovery = SourceStatementRecovery::Degraded,
            14 => row.name = None,
            15 => row.name = Some(SourceStatementWitnessNameId::new(1)),
            _ => unreachable!(),
        }
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0)
            }),
            "witness field {mutation}"
        );
        assert_replay();
    }
    for mutation in 0..6 {
        let mut input = fixture.witness_input();
        let row = &mut input.names[0];
        match mutation {
            0 => row.witness = SourceStatementWitnessId::new(1),
            1 => row.site = node(14),
            2 => row.source_range.start += 1,
            3 => row.source_range.end -= 1,
            4 => row.spelling.push('x'),
            5 => row.recovery = SourceStatementRecovery::Degraded,
            _ => unreachable!(),
        }
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidName {
                name: SourceStatementWitnessNameId::new(0)
            }),
            "name field {mutation}"
        );
        assert_replay();
    }
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
    assert_replay();
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
    assert_replay();
    let b3 = B3Fixture::new(47);
    assert_eq!(
        baseline.validate_installation(
            fixture.source,
            &fixture.module,
            &b3.statement(),
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    assert_eq!(
        baseline.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &b3.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    let foreign_b3_lower = B3Fixture::new(147);
    let foreign_b3_bindings_before = foreign_b3_lower.bindings.clone();
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &foreign_b3_lower.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(foreign_b3_lower.bindings, foreign_b3_bindings_before);
    assert_replay();
    let foreign_b3_primary_before = foreign_b3_lower.primary.clone();
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &fixture.bindings,
            &foreign_b3_lower.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(foreign_b3_lower.primary, foreign_b3_primary_before);
    assert_replay();
    let foreign_b3_atomic_before = foreign_b3_lower.atomic.clone();
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &foreign_b3_lower.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(foreign_b3_lower.atomic, foreign_b3_atomic_before);
    assert_replay();
    let foreign_a = Fixture::new(148);
    let foreign_b1 = B1Fixture::new(149);
    let foreign_b2 = B2Fixture::new(150);
    let foreign_b3 = B3Fixture::new(151);
    for (label, symbols, bindings, primary, atomic, arena) in [
        (
            "A",
            &foreign_a.symbols,
            &foreign_a.bindings,
            &foreign_a.primary,
            &foreign_a.atomic,
            &foreign_a.arena,
        ),
        (
            "B1",
            &foreign_b1.symbols,
            &foreign_b1.bindings,
            &foreign_b1.primary,
            &foreign_b1.atomic,
            &foreign_b1.arena,
        ),
        (
            "B2",
            &foreign_b2.symbols,
            &foreign_b2.bindings,
            &foreign_b2.primary,
            &foreign_b2.atomic,
            &foreign_b2.arena,
        ),
        (
            "B3",
            &foreign_b3.symbols,
            &foreign_b3.bindings,
            &foreign_b3.primary,
            &foreign_b3.atomic,
            &foreign_b3.arena,
        ),
    ] {
        let symbols_before = (*symbols).clone();
        let bindings_before = (*bindings).clone();
        let primary_before = (*primary).clone();
        let atomic_before = (*atomic).clone();
        let arena_before = (*arena).clone();
        assert_eq!(
            SourceStatementProducer::build(
                fixture.statement_input(),
                symbols,
                bindings,
                primary,
                atomic,
                arena,
            ),
            Err(SourceStatementError::DependencyMismatch),
            "foreign {label} lower tuple"
        );
        assert_eq!(*symbols, symbols_before, "foreign {label} symbols");
        assert_eq!(*bindings, bindings_before, "foreign {label} bindings");
        assert_eq!(*primary, primary_before, "foreign {label} primary");
        assert_eq!(*atomic, atomic_before, "foreign {label} atomic");
        assert_eq!(*arena, arena_before, "foreign {label} arena");
        assert_replay();
    }
    for mutation in 0..6 {
        let mut input = fixture.statement_input();
        let row = &mut input.owners[0];
        match mutation {
            0 => row.symbol = b3.symbol.clone(),
            1 => row.site = node(45),
            2 => row.source_range.start += 1,
            3 => row.source_range.end -= 1,
            4 => row.spelling.push('x'),
            5 => row.recovery = SourceStatementRecovery::Degraded,
            _ => unreachable!(),
        }
        assert!(
            SourceStatementProducer::build(
                input,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            )
            .is_err(),
            "owner provenance field {mutation}"
        );
        assert_replay();
    }
    let actual_labels = pristine_symbols
        .labels()
        .by_contribution(fixture.contribution);
    assert_eq!(actual_labels.len(), 1);
    let actual_label = actual_labels[0].clone();
    let same_source_b1 = B1Fixture::new(46);
    assert_eq!(same_source_b1.source, fixture.source);
    assert_eq!(same_source_b1.module, fixture.module);
    for (case, name) in [
        "Imported",
        "Missing",
        "Duplicate",
        "WrongPath",
        "WrongKind",
        "Private",
        "LocalOnly",
        "Recovered",
    ]
    .into_iter()
    .enumerate()
    {
        let mut labels = LabelIndex::new();
        if case != 1 {
            let origin_path = if case == 3 {
                LabelOriginPath::new("task258b3n::wrong::theorem::origin")
            } else {
                actual_label.origin_path().clone()
            };
            let kind = if case == 4 {
                LabelKind::ProofStep
            } else {
                actual_label.kind()
            };
            let origin = match case {
                0 => {
                    let (import_owner, _) = same_source_b1
                        .resolver_ast
                        .nodes()
                        .iter()
                        .next()
                        .expect("same-source opaque resolved node");
                    assert_eq!(import_owner.index(), 0);
                    let mut temporary_imports = ResolvedImports::new();
                    let import = temporary_imports.push_import(ResolvedImport::new(
                        import_owner,
                        range(actual_label.origin().source_id(), 0, 1),
                        "import statement.fixture;",
                        None,
                        ImportResolution::Resolved(fixture.module.clone()),
                        actual_label.origin().clone(),
                    ));
                    assert_eq!(import.index(), 0);
                    assert_eq!(temporary_imports.imports().count(), 1);
                    actual_label.origin().clone().with_import_edge(import)
                }
                7 => actual_label.origin().clone().recovered(),
                _ => actual_label.origin().clone(),
            };
            let visibility = if case == 5 {
                mizar_resolve::env::Visibility::Private
            } else {
                actual_label.visibility()
            };
            let export_status = if case == 6 {
                ExportStatus::LocalOnly
            } else {
                actual_label.export_status()
            };
            labels.insert(
                LabelEntry::new(
                    origin_path,
                    kind,
                    actual_label.namespace().clone(),
                    actual_label.primary_spelling(),
                    origin,
                    actual_label.contribution(),
                )
                .with_visibility(visibility)
                .with_export_status(export_status),
            );
            if case == 2 {
                labels.insert(
                    LabelEntry::new(
                        LabelOriginPath::new("task258b3n::duplicate::theorem::origin"),
                        actual_label.kind(),
                        actual_label.namespace().clone(),
                        actual_label.primary_spelling(),
                        actual_label.origin().clone(),
                        actual_label.contribution(),
                    )
                    .with_visibility(actual_label.visibility())
                    .with_export_status(actual_label.export_status()),
                );
            }
        }
        let attempted_symbols = SymbolEnv::new(
            pristine_symbols.module_id().clone(),
            SymbolEnvIndexes {
                imports: pristine_symbols.imports().clone(),
                exports: pristine_symbols.exports().clone(),
                symbols: pristine_symbols.symbols().clone(),
                labels,
                definitions: pristine_symbols.definitions().clone(),
                overloads: pristine_symbols.overloads().clone(),
                registrations: pristine_symbols.registrations().clone(),
                lexical_summaries: pristine_symbols.lexical_summaries().clone(),
                namespace_graph: pristine_symbols.namespace_graph().clone(),
                declaration_dependencies: pristine_symbols.declaration_dependencies().clone(),
                contributions: pristine_symbols.contributions().clone(),
                module_summaries: pristine_symbols.module_summaries().clone(),
            },
        );
        assert_eq!(
            attempted_symbols.imports(),
            pristine_symbols.imports(),
            "{name} imports"
        );
        assert_eq!(
            attempted_symbols.contributions(),
            pristine_symbols.contributions(),
            "{name} contribution effects"
        );
        let attempted_symbols_before = attempted_symbols.clone();
        assert_eq!(
            SourceStatementProducer::build(
                pristine_input.clone(),
                &attempted_symbols,
                &pristine_bindings,
                &pristine_primary,
                &pristine_atomic,
                &pristine_arena,
            ),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            }),
            "Task258B3N label mutation {name}"
        );
        assert_eq!(
            attempted_symbols, attempted_symbols_before,
            "{name} env purity"
        );
        assert_eq!(
            attempted_symbols.imports(),
            pristine_symbols.imports(),
            "{name} imports"
        );
        assert_eq!(
            attempted_symbols.contributions(),
            pristine_symbols.contributions(),
            "{name} contribution effects"
        );
        assert_replay();
    }
    let mut contributions = fixture.symbols.contributions().clone();
    let foreign_contribution = contributions.insert(
        fixture.module.clone(),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 0, 18)),
    );
    let foreign_symbols = SymbolEnv::new(
        fixture.module.clone(),
        SymbolEnvIndexes {
            imports: fixture.symbols.imports().clone(),
            exports: fixture.symbols.exports().clone(),
            symbols: fixture.symbols.symbols().clone(),
            labels: fixture.symbols.labels().clone(),
            definitions: fixture.symbols.definitions().clone(),
            overloads: fixture.symbols.overloads().clone(),
            registrations: fixture.symbols.registrations().clone(),
            lexical_summaries: fixture.symbols.lexical_summaries().clone(),
            namespace_graph: fixture.symbols.namespace_graph().clone(),
            declaration_dependencies: fixture.symbols.declaration_dependencies().clone(),
            contributions,
            module_summaries: fixture.symbols.module_summaries().clone(),
        },
    );
    let mut foreign_contribution_input = fixture.statement_input();
    foreign_contribution_input.owners[0].contribution = foreign_contribution;
    assert!(
        SourceStatementProducer::build(
            foreign_contribution_input,
            &foreign_symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        )
        .is_err(),
        "owner contribution provenance"
    );
    assert_replay();
    for aggregate_mutation in 0..10 {
        let mut input = fixture.statement_input();
        match aggregate_mutation {
            0 => input.owners.clear(),
            1 => input.owners.push(input.owners[0].clone()),
            2 => {
                input.statements.pop();
            }
            3 => input.statements.push(input.statements[1].clone()),
            4 => {
                input.contexts.pop();
            }
            5 => input.contexts.push(input.contexts[1].clone()),
            6 => {
                input.input_facts.pop();
            }
            7 => input.input_facts.push(input.input_facts[1].clone()),
            8 => {
                input.candidate_facts.pop();
            }
            9 => input.candidate_facts.push(input.candidate_facts[1].clone()),
            _ => unreachable!(),
        }
        assert_eq!(
            SourceStatementProducer::build(
                input,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidAggregate),
            "base aggregate {aggregate_mutation}"
        );
        assert_replay();
    }
    for index in 0..2 {
        let other = 1 - index;
        for mutation in 0..9 {
            let mut input = fixture.statement_input();
            let row = &mut input.statements[index];
            match mutation {
                0 => row.owner = SourceTheoremOwnerId::new(1),
                1 => row.context = SourceStatementContextId::new(other),
                2 => {
                    row.formula =
                        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                }
                3 => row.site = node([45, 47][index]),
                4 => row.source_range.start += 1,
                5 => row.source_range.end -= 1,
                6 => row.source_ordinal = 1,
                7 => row.spelling.push('x'),
                8 => row.recovery = SourceStatementRecovery::Degraded,
                _ => unreachable!(),
            }
            assert!(
                SourceStatementProducer::build(
                    input,
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                )
                .is_err(),
                "base statement {index} field {mutation}"
            );
            assert_replay();
        }
        for mutation in 0..5 {
            let mut input = fixture.statement_input();
            let row = &mut input.contexts[index];
            match mutation {
                0 => row.statement = SourceStatementId::new(other),
                1 => row.binding_context = BindingContextId::new(other),
                2 => row.source_range.start += 1,
                3 => row.source_range.end -= 1,
                4 => row.visible_bindings.clear(),
                _ => unreachable!(),
            }
            assert!(
                SourceStatementProducer::build(
                    input,
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                )
                .is_err(),
                "base context {index} field {mutation}"
            );
            assert_replay();
        }
        for mutation in 0..5 {
            let mut input = fixture.statement_input();
            let row = &mut input.input_facts[index];
            match mutation {
                0 => row.statement = SourceStatementId::new(other),
                1 => row.context = SourceStatementContextId::new(other),
                2 => row.ordinal = 1,
                3 => row.binding = BindingId::new(1),
                4 => row.uses.swap(0, 1),
                _ => unreachable!(),
            }
            assert!(
                SourceStatementProducer::build(
                    input,
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                )
                .is_err(),
                "base input fact {index} field {mutation}"
            );
            assert_replay();
        }
        for mutation in 0..4 {
            let mut input = fixture.statement_input();
            let row = &mut input.candidate_facts[index];
            match mutation {
                0 => row.statement = SourceStatementId::new(other),
                1 => row.context = SourceStatementContextId::new(other),
                2 => row.ordinal = 1,
                3 => {
                    row.formula =
                        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                }
                _ => unreachable!(),
            }
            assert!(
                SourceStatementProducer::build(
                    input,
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                )
                .is_err(),
                "base candidate {index} field {mutation}"
            );
            assert_replay();
        }
    }
    for index in 0..51 {
        for mutation in 0..5 {
            let arena = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    match mutation {
                        0 => {
                            let SourceAnchor::Range(mut source_range) = row.anchor.clone() else {
                                unreachable!("Task258B3N range")
                            };
                            source_range.end += 1;
                            row.anchor = SourceAnchor::Range(source_range);
                        }
                        1 => row.recovery = NodeRecoveryState::Recovered,
                        2 => row.recovery = NodeRecoveryState::Degraded,
                        3 => row.kind = "source.task258b3n.mutated".into(),
                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                        4 if row.children.len() == 1 => row.children.clear(),
                        4 => row.children.push(TypedNodeId::new(usize::from(index == 0))),
                        _ => unreachable!(),
                    }
                }
            });
            let statement_before = statement.clone();
            let primary_before = fixture.primary.clone();
            let arena_before = arena.clone();
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
            assert_eq!(statement, statement_before);
            assert_eq!(fixture.primary, primary_before);
            assert_eq!(arena, arena_before);
            assert_eq!(
                SourceStatementWitnessProducer::build(
                    fixture.witness_input(),
                    &statement,
                    &fixture.primary,
                    &arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} mutation {mutation} producer"
            );
            assert_eq!(statement, statement_before);
            assert_eq!(fixture.primary, primary_before);
            assert_eq!(arena, arena_before);
            assert_replay();
        }
    }
    assert_eq!(
        fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("replay")
            .debug_text(),
        baseline_debug
    );
    assert_replay();
}

#[test]
fn task258b3n_paired_typed_ownership_hybrids_and_orders_are_atomic() {
    let fixture = B3NFixture::new(48);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let base = fixture.empty_typed();
    let base_debug = base.debug_text();
    let typed = base
        .clone()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("paired install");
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert!(typed.source_statement_references().is_none());
    assert_eq!(base.debug_text(), base_debug);
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone().with_source_statement(statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let task258a = Fixture::new(48);
    let task258a_statement = task258a
        .build(task258a.input())
        .expect("Task258A statement");
    let task258a_installed = TypedAst::try_new(TypedAstParts {
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
    .expect("Task258A primary")
    .with_source_atomic_formula(task258a.atomic.clone())
    .expect("Task258A atomic")
    .with_source_statement(task258a_statement.clone())
    .expect("Task258A install");
    let task258a_debug = task258a_installed.debug_text();
    assert_eq!(
        task258a_installed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task258a_installed.debug_text(), task258a_debug);
    assert_eq!(
        typed.clone().with_source_statement(task258a_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let b2 = B2Fixture::new(48);
    let b2_statement = b2.build(b2.input()).expect("B2 statement");
    let b2_installed = b2
        .empty_typed()
        .with_source_statement(b2_statement.clone())
        .expect("B2 install");
    let b2_debug = b2_installed.debug_text();
    assert_eq!(
        b2_installed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b2_installed.debug_text(), b2_debug);
    assert_eq!(
        typed.clone().with_source_statement(b2_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let b3 = B3Fixture::new(49);
    let b3_statement = b3.statement();
    let b3_witnesses = b3
        .witnesses(&b3_statement, b3.witness_input())
        .expect("B3 witnesses");
    let b3_installed = b3
        .empty_typed()
        .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
        .expect("B3 paired install");
    let b3_debug = b3_installed.debug_text();
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b3_installed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b3_installed.debug_text(), b3_debug);
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(b3_statement, b3_witnesses),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let b1 = B1Fixture::new(49);
    let b1_references = b1.references(b1.reference_input()).expect("B1 references");
    let b1_installed = b1
        .empty_typed()
        .with_source_statement_references(b1.statement.clone(), b1_references.clone())
        .expect("B1 paired install");
    let b1_debug = b1_installed.debug_text();
    assert_eq!(
        typed
            .clone()
            .with_source_statement_references(b1.statement.clone(), b1_references,),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b1_installed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b1_installed.debug_text(), b1_debug);
    let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
        fixture.source,
        fixture.module.clone(),
    );
    let task248_debug = task248.debug_text();
    assert_eq!(
        task248
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248.debug_text(), task248_debug);
    assert_eq!(
        typed.clone().with_source_context_for_test(
            task248
                .source_context()
                .expect("Task248 source context")
                .clone(),
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
        let debug = occupied.debug_text();
        assert_eq!(
            occupied
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "{family}"
        );
        assert_eq!(occupied.debug_text(), debug, "{family} rollback");
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
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert_eq!(
        base.with_source_statement_witnesses(statement, witnesses)
            .expect("replay")
            .debug_text(),
        typed.debug_text()
    );
}

#[test]
fn task258b3n_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3NFixture::new(50);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("typed");
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
    assert_eq!(resolved.source_statement(), Some(&statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
    assert!(resolved.source_statement_references().is_none());
    assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
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
    let mut stale = witnesses.clone();
    stale.statement_fingerprint.push('x');
    let mut stale_pair = fixture.empty_typed();
    stale_pair.inject_source_statement_witness_bundle_for_test(statement.clone(), stale);
    assert_eq!(
        assemble_empty_resolved(&stale_pair),
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
    let b1 = B1Fixture::new(50);
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
        fingerprint: ClusterFactFingerprint::new("task258b3n-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 81, 82),
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
                typed_node: TypedNodeId::new(13),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement-witness-name.semantic-coexistence",),
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
                expr: ExprId::new("task258b3n-semantic-coexistence"),
                typed_site: node(34),
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let empty_collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let empty_expansion = TemplateExpansionOutput::expand(&empty_collection);
    let empty_viability =
        CandidateViabilityOutput::filter(&empty_expansion, Vec::<CandidateViabilityInput>::new());
    let empty_specificity =
        SpecificityGraphOutput::build(&empty_viability, Vec::<SpecificityComparisonInput>::new());
    let empty_selection = OverloadSelectionOutput::resolve(
        &empty_specificity,
        Vec::<OverloadSiteResolutionInput>::new(),
    );
    let occupied_collection = OverloadCollectionOutput::collect(
        vec![OverloadSiteInput {
            key: OverloadSiteKey::new("task258b3n-overload"),
            owner: TypedSiteRef::Role {
                node: TypedNodeId::new(34),
                role: TypeRole::new("task258b3n-overload-owner"),
            },
            source_range: range(fixture.source, 85, 86),
            kind: OverloadSiteKind::FunctorApplication,
            name: OverloadNameKey::new("task258b3n-overload"),
            arguments: Vec::new(),
            expected: None,
            source_qua: Vec::<SourceQuaView>::new(),
            recovery: OverloadSiteRecovery::Normal,
        }],
        vec![OverloadCandidateInput {
            site: OverloadSiteKey::new("task258b3n-overload"),
            symbol: fixture.symbol.clone(),
            ordinary_root: fixture.symbol.clone(),
            declaration_kind: CandidateDeclarationKind::Functor,
            parameters: Vec::new(),
            result: None,
            origin: CandidateOrigin::Ordinary,
            template: None,
            coherence: None,
            provenance: CandidateProvenance {
                stable_key: CandidateProvenanceKey::new("task258b3n-overload"),
                source_range: Some(range(fixture.source, 85, 86)),
                scope: CandidateScope::Local,
                declaration_order: 0,
            },
        }],
    );
    let occupied_expansion = TemplateExpansionOutput::expand(&occupied_collection);
    let occupied_viability = CandidateViabilityOutput::filter(
        &occupied_expansion,
        vec![CandidateViabilityInput {
            candidate: OverloadCandidateId::new(0),
            arguments: Vec::new(),
        }],
    );
    let occupied_specificity = SpecificityGraphOutput::build(
        &occupied_viability,
        Vec::<SpecificityComparisonInput>::new(),
    );
    let occupied_selection = OverloadSelectionOutput::resolve(
        &occupied_specificity,
        vec![OverloadSiteResolutionInput {
            site: OverloadSiteId::new(0),
            refinements: vec![OverloadCandidateId::new(0)],
            refinement_join: RefinementJoinPayload {
                status: RefinementJoinStatus::Compatible,
                exposed_result: None,
            },
            inserted_views: Vec::new(),
        }],
    );
    assert_eq!(occupied_collection.candidates().len(), 1);
    assert_eq!(occupied_expansion.candidates().len(), 1);
    assert_eq!(occupied_viability.candidates().len(), 1);
    assert_eq!(occupied_specificity.candidates().len(), 1);
    assert_eq!(occupied_selection.results().len(), 1);
    let cluster_facts = ClusterFactTable::new();
    for (label, collection, expansion, viability, specificity, selection) in [
        (
            "collection",
            &occupied_collection,
            &empty_expansion,
            &empty_viability,
            &empty_specificity,
            &empty_selection,
        ),
        (
            "expansion",
            &empty_collection,
            &occupied_expansion,
            &empty_viability,
            &empty_specificity,
            &empty_selection,
        ),
        (
            "viability",
            &empty_collection,
            &empty_expansion,
            &occupied_viability,
            &empty_specificity,
            &empty_selection,
        ),
        (
            "specificity",
            &empty_collection,
            &empty_expansion,
            &empty_viability,
            &occupied_specificity,
            &empty_selection,
        ),
        (
            "selection",
            &empty_collection,
            &empty_expansion,
            &empty_viability,
            &empty_specificity,
            &occupied_selection,
        ),
    ] {
        assert_eq!(
            ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                overload_collection: collection,
                template_expansion: expansion,
                viability,
                specificity,
                overload_selection: selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            }),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{label}"
        );
    }
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("replay")
            .debug_text(),
        resolved.debug_text()
    );
}
