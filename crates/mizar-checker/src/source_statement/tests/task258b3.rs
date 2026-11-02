use super::*;

#[test]
fn task258b3_exact_witness_api_debug_and_lower_profile_are_stable() {
    let fixture = B3Fixture::new(40);
    let statement = fixture.statement();
    assert!(statement.is_task_258b3_profile());
    assert!(!statement.is_task_258a_profile());
    assert!(!statement.is_task_258b1_profile());
    assert!(!statement.is_task_258b2_profile());
    assert_eq!(statement.binding_env(), &fixture.bindings);
    assert_eq!(statement.binding_env().contexts().len(), 2);
    assert_eq!(statement.binding_env().bindings().len(), 1);
    assert!(statement.binding_env().diagnostics().is_empty());
    let proof_context = statement
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof_context.owner,
        BindingContextOwner::SourceStatement {
            source_range: range(fixture.source, 69, 102),
        }
    );
    assert_eq!(proof_context.parent, Some(BindingContextId::new(0)));
    assert_eq!(proof_context.layer, BindingContextLayer::Proof);
    assert_eq!(
        proof_context.lexical_scope.as_ref().expect("scope").path(),
        [0]
    );
    assert!(proof_context.bindings.is_empty());
    assert_eq!(proof_context.visible_bindings, [BindingId::new(0)]);
    assert_eq!(proof_context.recovery, BindingContextRecovery::Normal);
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| row.source_ordinal())
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(fixture.primary.terms().len(), 5);
    assert_eq!(fixture.primary.references().len(), 5);
    assert!(fixture.primary.numeric_type_requests().is_empty());
    assert_eq!(
        fixture
            .primary
            .terms()
            .iter()
            .map(|(_, row)| row.context().index())
            .collect::<Vec<_>>(),
        [0, 0, 1, 1, 1]
    );
    for (index, ((_, term), (_, reference))) in fixture
        .primary
        .terms()
        .iter()
        .zip(fixture.primary.references().iter())
        .enumerate()
    {
        let sites = [26, 28, 32, 36, 38];
        let ranges = [(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)];
        let contexts = [0, 0, 1, 1, 1];
        assert_eq!(term.site().node().index(), sites[index]);
        assert_eq!(
            term.source_range(),
            range(fixture.source, ranges[index].0, ranges[index].1)
        );
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context(), BindingContextId::new(contexts[index]));
        assert_eq!(term.spelling(), "x");
        assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
        assert_eq!(term.role(), SourcePrimaryTermRole::Value);
        assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
        assert!(term.parent().is_none());
        assert_eq!(reference.term(), SourcePrimaryTermId::new(index));
        assert_eq!(reference.binding(), BindingId::new(0));
        assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
        if index < 2 {
            assert!(reference.lexical_scope().is_none());
        } else {
            assert_eq!(
                reference.lexical_scope().expect("reference scope").path(),
                [0]
            );
        }
        assert_eq!(reference.use_ordinal(), 1);
    }
    assert_eq!(fixture.atomic.formulas().len(), 2);
    assert!(fixture.atomic.wrappers().is_empty());
    assert!(fixture.atomic.predicate_segments().is_empty());
    assert!(fixture.atomic.predicate_heads().is_empty());
    assert!(fixture.atomic.candidates().is_empty());
    assert!(fixture.atomic.type_sites().is_empty());
    assert!(fixture.atomic.attributes().is_empty());
    assert_eq!(fixture.atomic.edges().len(), 4);
    assert_eq!(fixture.atomic.requests().len(), 4);
    for index in 0..2 {
        let formula = fixture
            .atomic
            .formulas()
            .get(SourceAtomicFormulaId::new(index))
            .expect("atomic formula");
        assert_eq!(formula.site().node().index(), [30, 40][index]);
        assert_eq!(
            formula.source_range(),
            range(
                fixture.source,
                [(63, 68), (92, 97)][index].0,
                [(63, 68), (92, 97)][index].1
            )
        );
        assert_eq!(formula.source_ordinal(), index);
        assert_eq!(formula.context(), BindingContextId::new(index));
        assert_eq!(formula.spelling(), "x = x");
        assert_eq!(formula.kind(), SourceAtomicFormulaKind::Equality);
        assert_eq!(formula.recovery(), SourceAtomicFormulaRecovery::Normal);
    }
    assert_eq!(
        fixture
            .atomic
            .edges()
            .iter()
            .map(|(_, row)| row.target())
            .collect::<Vec<_>>(),
        [
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(3)),
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(4)),
        ]
    );
    for edge_index in 0..4 {
        let edge = fixture
            .atomic
            .edges()
            .get(SourceAtomicEdgeId::new(edge_index))
            .expect("atomic edge");
        let formula = if edge_index < 2 { 0 } else { 1 };
        let ordinal = edge_index % 2;
        assert_eq!(edge.formula(), SourceAtomicFormulaId::new(formula));
        assert_eq!(edge.ordinal(), ordinal);
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
            .get(crate::source_atomic_formula::SourceAtomicRequestId::new(
                edge_index,
            ))
            .expect("atomic request");
        assert_eq!(request.formula(), SourceAtomicFormulaId::new(formula));
        assert_eq!(request.ordinal(), ordinal);
        assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
        assert_eq!(request.edge(), Some(SourceAtomicEdgeId::new(edge_index)));
        assert!(request.candidate().is_none());
        assert!(request.type_site().is_none());
        assert!(request.attribute().is_none());
    }
    assert_eq!(statement.owners().len(), 1);
    assert_eq!(statement.contexts().len(), 2);
    assert_eq!(statement.input_facts().len(), 2);
    assert_eq!(statement.candidate_facts().len(), 2);
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(owner.symbol(), &fixture.symbol);
    assert_eq!(owner.contribution(), fixture.contribution);
    assert_eq!(owner.site().node().index(), 45);
    assert_eq!(owner.source_range(), range(fixture.source, 19, 103));
    assert_eq!(owner.spelling(), "FormulaStatementSingleWitnessSmoke");
    assert_eq!(owner.role(), SourceTheoremRole::Theorem);
    assert_eq!(owner.status(), SourceTheoremStatus::Unmodified);
    assert_eq!(owner.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    assert_eq!(
        statement.binding_fingerprint(),
        fixture.bindings.debug_text()
    );
    assert_eq!(
        statement.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(
        statement.atomic_formula_fingerprint(),
        fixture.atomic.debug_text()
    );
    for index in 0..2 {
        let row = statement
            .statements()
            .get(SourceStatementId::new(index))
            .expect("statement");
        assert_eq!(row.owner(), SourceTheoremOwnerId::new(0));
        assert_eq!(row.context(), SourceStatementContextId::new(index));
        assert_eq!(
            row.formula(),
            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index))
        );
        assert_eq!(row.site().node().index(), [45, 43][index]);
        assert_eq!(
            row.source_range(),
            range(
                fixture.source,
                [(19, 103), (87, 98)][index].0,
                [(19, 103), (87, 98)][index].1
            )
        );
        assert_eq!(row.source_ordinal(), [0, 2][index]);
        assert_eq!(
            row.kind(),
            [
                SourceStatementKind::TheoremProposition,
                SourceStatementKind::Conclusion,
            ][index]
        );
        assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
        let context = statement
            .contexts()
            .get(SourceStatementContextId::new(index))
            .expect("statement context");
        assert_eq!(context.statement(), SourceStatementId::new(index));
        assert_eq!(context.binding_context(), BindingContextId::new(index));
        assert_eq!(context.source_range(), row.source_range());
        assert_eq!(context.visible_bindings(), [BindingId::new(0)]);
        let fact = statement
            .input_facts()
            .get(SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(fact.statement(), SourceStatementId::new(index));
        assert_eq!(fact.context(), SourceStatementContextId::new(index));
        assert_eq!(fact.ordinal(), 0);
        assert_eq!(fact.kind(), SourceStatementInputFactKind::ReservedTypeGuard);
        assert_eq!(fact.binding(), BindingId::new(0));
        let first_term = if index == 0 { 0 } else { 3 };
        assert_eq!(
            fact.uses(),
            [
                SourcePrimaryTermReferenceId::new(first_term),
                SourcePrimaryTermReferenceId::new(first_term + 1),
            ]
        );
        let candidate = statement
            .candidate_facts()
            .get(SourceStatementCandidateFactId::new(index))
            .expect("candidate");
        assert_eq!(candidate.statement(), SourceStatementId::new(index));
        assert_eq!(candidate.context(), SourceStatementContextId::new(index));
        assert_eq!(candidate.ordinal(), 0);
        assert_eq!(
            candidate.kind(),
            SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate.formula(), row.formula());
    }
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task258B3 witnesses");
    assert_eq!(witnesses.source_id(), fixture.source);
    assert_eq!(witnesses.module_id(), &fixture.module);
    assert_eq!(witnesses.statement_fingerprint(), statement.debug_text());
    assert_eq!(
        witnesses.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(witnesses.witnesses().len(), 1);
    assert!(!witnesses.witnesses().is_empty());
    let row = witnesses
        .witnesses()
        .get(SourceStatementWitnessId::new(0))
        .expect("witness row");
    assert_eq!(row.owner(), SourceTheoremOwnerId::new(0));
    assert_eq!(row.binding_context(), BindingContextId::new(1));
    assert_eq!(
        row.term(),
        SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
    );
    assert_eq!(
        (row.take_site().node().index(), row.site().node().index()),
        (35, 34)
    );
    assert_eq!((row.take_range().start, row.take_range().end), (77, 84));
    assert_eq!((row.source_range().start, row.source_range().end), (82, 83));
    assert_eq!((row.source_ordinal(), row.ordinal()), (1, 0));
    assert_eq!(row.spelling(), "x");
    assert_eq!(row.kind(), SourceStatementWitnessKind::Unnamed);
    assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
    let debug = witnesses.debug_text();
    assert!(debug.starts_with("source-statement-witness-debug-v1\n"));
    assert!(debug.contains(
        "witness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\""
    ));
    assert_eq!(
        fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("replay")
            .debug_text(),
        debug
    );
}

#[test]
fn task258b3_dependency_aggregate_rows_provenance_and_replay_fail_closed() {
    let fixture = B3Fixture::new(41);
    let pristine_input = fixture.statement_input();
    let pristine_witness_input = fixture.witness_input();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let statement = fixture.statement();
    let valid = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("baseline");
    let statement_baseline = statement.debug_text();
    let baseline = valid.debug_text();
    let baseline_typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), valid.clone())
        .expect("canonical paired install replay");
    let typed_baseline = baseline_typed.debug_text();
    let final_baseline = assemble_empty_resolved(&baseline_typed)
        .expect("canonical final replay")
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
        assert_eq!(replay_statement.debug_text(), statement_baseline);
        let replay_witness = SourceStatementWitnessProducer::build(
            pristine_witness_input.clone(),
            &replay_statement,
            &pristine_primary,
            &pristine_arena,
        )
        .expect("canonical witness replay");
        assert_eq!(replay_witness.debug_text(), baseline);
        let replay_typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(replay_statement, replay_witness)
            .expect("canonical paired install replay");
        assert_eq!(replay_typed, baseline_typed);
        assert_eq!(replay_typed.debug_text(), typed_baseline);
        assert_eq!(
            assemble_empty_resolved(&replay_typed)
                .expect("canonical final replay")
                .debug_text(),
            final_baseline
        );
    };
    let mut empty = fixture.witness_input();
    empty.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, empty),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_replay();
    let mut duplicate = fixture.witness_input();
    duplicate.witnesses.push(duplicate.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, duplicate),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_replay();
    let mut wrong_source = fixture.witness_input();
    wrong_source.source_id = source_id(999);
    assert_eq!(
        fixture.witnesses(&statement, wrong_source),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    let mut wrong_module = fixture.witness_input();
    wrong_module.module_id =
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"));
    assert_eq!(
        fixture.witnesses(&statement, wrong_module),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    for (label, mutate) in [
        (
            "owner",
            (|row: &mut SourceStatementWitnessInput| row.owner = SourceTheoremOwnerId::new(1))
                as fn(&mut SourceStatementWitnessInput),
        ),
        ("context", |row| {
            row.binding_context = BindingContextId::new(0)
        }),
        ("term 0", |row| {
            row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(0))
        }),
        ("term 1", |row| {
            row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(1))
        }),
        ("term 3", |row| {
            row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3))
        }),
        ("term 4", |row| {
            row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(4))
        }),
        ("take site", |row| row.take_site = node(34)),
        ("take range start", |row| row.take_range.start = 78),
        ("take range end", |row| row.take_range.end = 83),
        ("site", |row| row.site = node(35)),
        ("range start", |row| row.source_range.start = 81),
        ("range end", |row| row.source_range.end = 84),
        ("source ordinal", |row| row.source_ordinal = 2),
        ("ordinal", |row| row.ordinal = 1),
        ("spelling", |row| row.spelling = "y".to_owned()),
        ("recovery", |row| {
            row.recovery = SourceStatementRecovery::Degraded
        }),
    ] {
        let mut input = fixture.witness_input();
        mutate(&mut input.witnesses[0]);
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0)
            }),
            "{label}"
        );
        assert_replay();
    }
    let assert_base_rejects = |label: &str, mutate: &dyn Fn(&mut SourceStatementHandoffInput)| {
        let mut input = fixture.statement_input();
        mutate(&mut input);
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
            "{label}"
        );
        assert_replay();
    };
    let mut foreign_contributions = SourceContributionIndex::new();
    foreign_contributions.insert(
        fixture.module.clone(),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 0, 1)),
    );
    let foreign_contribution = foreign_contributions.insert(
        fixture.module.clone(),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 1, 2)),
    );
    assert_eq!(foreign_contribution.index(), 1);
    assert_base_rejects("base source", &|input| input.source_id = source_id(999));
    assert_base_rejects("base module", &|input| {
        input.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
    });
    assert_base_rejects("base owner contribution", &|input| {
        input.owners[0].contribution = foreign_contribution
    });
    assert_base_rejects("extra base owner", &|input| {
        input.owners.push(input.owners[0].clone())
    });
    for (label, mutate) in [
        (
            "missing base statement",
            (|input: &mut SourceStatementHandoffInput| {
                input.statements.pop();
            }) as fn(&mut SourceStatementHandoffInput),
        ),
        ("extra base statement", |input| {
            input.statements.push(input.statements[1].clone())
        }),
        ("missing base context", |input| {
            input.contexts.pop();
        }),
        ("extra base context", |input| {
            input.contexts.push(input.contexts[1].clone())
        }),
        ("missing base input fact", |input| {
            input.input_facts.pop();
        }),
        ("extra base input fact", |input| {
            input.input_facts.push(input.input_facts[1].clone())
        }),
        ("missing base candidate", |input| {
            input.candidate_facts.pop();
        }),
        ("extra base candidate", |input| {
            input.candidate_facts.push(input.candidate_facts[1].clone())
        }),
    ] {
        assert_base_rejects(label, &mutate);
    }
    for (label, mutate) in [
        (
            "base owner symbol",
            (|input: &mut SourceStatementHandoffInput| {
                input.owners[0].symbol = SymbolId::new(
                    input.module_id.clone(),
                    LocalSymbolId::new("ForeignTask258B3Owner"),
                    FullyQualifiedName::new(
                        "pkg::statement.fixture::theorem::ForeignTask258B3Owner",
                    ),
                )
            }) as fn(&mut SourceStatementHandoffInput),
        ),
        ("base owner site", |input| input.owners[0].site = node(43)),
        ("base owner range start", |input| {
            input.owners[0].source_range.start += 1
        }),
        ("base owner range end", |input| {
            input.owners[0].source_range.end -= 1
        }),
        ("base owner spelling", |input| {
            input.owners[0].spelling.push('x')
        }),
        ("base owner recovery", |input| {
            input.owners[0].recovery = SourceStatementRecovery::Degraded
        }),
    ] {
        assert_base_rejects(label, &mutate);
    }
    for index in 0..2 {
        let other = 1 - index;
        assert_base_rejects(&format!("base statement {index} owner"), &|input| {
            input.statements[index].owner = SourceTheoremOwnerId::new(1)
        });
        assert_base_rejects(&format!("base statement {index} context"), &|input| {
            input.statements[index].context = SourceStatementContextId::new(other)
        });
        assert_base_rejects(&format!("base statement {index} formula"), &|input| {
            input.statements[index].formula =
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
        });
        assert_base_rejects(&format!("base statement {index} site"), &|input| {
            input.statements[index].site = node([43, 45][index])
        });
        assert_base_rejects(&format!("base statement {index} range start"), &|input| {
            input.statements[index].source_range.start += 1
        });
        assert_base_rejects(&format!("base statement {index} range end"), &|input| {
            input.statements[index].source_range.end -= 1
        });
        assert_base_rejects(&format!("base statement {index} spelling"), &|input| {
            input.statements[index].spelling.push('x')
        });
        assert_base_rejects(&format!("base statement {index} kind"), &|input| {
            input.statements[index].kind = [
                SourceStatementKind::Conclusion,
                SourceStatementKind::TheoremProposition,
            ][index]
        });
        assert_base_rejects(&format!("base statement {index} recovery"), &|input| {
            input.statements[index].recovery = SourceStatementRecovery::Degraded
        });
        assert_base_rejects(&format!("base context {index} statement"), &|input| {
            input.contexts[index].statement = SourceStatementId::new(other)
        });
        assert_base_rejects(&format!("base context {index} binding context"), &|input| {
            input.contexts[index].binding_context = BindingContextId::new(other)
        });
        assert_base_rejects(&format!("base context {index} range start"), &|input| {
            input.contexts[index].source_range.start += 1
        });
        assert_base_rejects(&format!("base context {index} range end"), &|input| {
            input.contexts[index].source_range.end -= 1
        });
        assert_base_rejects(&format!("base context {index} visibility"), &|input| {
            input.contexts[index].visible_bindings.clear()
        });
        assert_base_rejects(&format!("base input fact {index} statement"), &|input| {
            input.input_facts[index].statement = SourceStatementId::new(other)
        });
        assert_base_rejects(&format!("base input fact {index} context"), &|input| {
            input.input_facts[index].context = SourceStatementContextId::new(other)
        });
        assert_base_rejects(&format!("base input fact {index} ordinal"), &|input| {
            input.input_facts[index].ordinal = 1
        });
        assert_base_rejects(&format!("base input fact {index} binding"), &|input| {
            input.input_facts[index].binding = BindingId::new(1)
        });
        assert_base_rejects(&format!("base input fact {index} uses"), &|input| {
            input.input_facts[index].uses.swap(0, 1)
        });
        assert_base_rejects(&format!("base candidate {index} statement"), &|input| {
            input.candidate_facts[index].statement = SourceStatementId::new(other)
        });
        assert_base_rejects(&format!("base candidate {index} context"), &|input| {
            input.candidate_facts[index].context = SourceStatementContextId::new(other)
        });
        assert_base_rejects(&format!("base candidate {index} ordinal"), &|input| {
            input.candidate_facts[index].ordinal = 1
        });
        assert_base_rejects(&format!("base candidate {index} formula"), &|input| {
            input.candidate_facts[index].formula =
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
        });
    }
    let mut reordered_statements = fixture.statement_input();
    reordered_statements.statements.swap(0, 1);
    assert_eq!(
        SourceStatementProducer::build(
            reordered_statements,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0),
        })
    );
    assert_replay();
    for (index, replacement) in [(0, 1), (1, 1)] {
        let mut input = fixture.statement_input();
        input.statements[index].source_ordinal = replacement;
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
            "base source ordinal {index}"
        );
        assert_replay();
    }
    let mut missing_owner = fixture.statement_input();
    missing_owner.owners.clear();
    assert_eq!(
        SourceStatementProducer::build(
            missing_owner,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidAggregate)
    );
    assert_replay();
    let mut crossed_context = fixture.statement_input();
    crossed_context.contexts[1].binding_context = BindingContextId::new(0);
    assert_eq!(
        SourceStatementProducer::build(
            crossed_context,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidContext {
            context: SourceStatementContextId::new(1),
        })
    );
    assert_replay();
    let mut crossed_input = fixture.statement_input();
    crossed_input.input_facts[1].uses.swap(0, 1);
    assert_eq!(
        SourceStatementProducer::build(
            crossed_input,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidInputFact {
            fact: SourceStatementInputFactId::new(1),
        })
    );
    assert_replay();
    let mut crossed_candidate = fixture.statement_input();
    crossed_candidate.candidate_facts[1].formula =
        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0));
    assert_eq!(
        SourceStatementProducer::build(
            crossed_candidate,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidCandidateFact {
            fact: SourceStatementCandidateFactId::new(1),
        })
    );
    assert_replay();
    let mut stale_statement = valid.clone();
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
    let mut stale_primary = valid.clone();
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
    let b1 = B1Fixture::new(41);
    assert_eq!(b1.source, fixture.source);
    assert_eq!(b1.module, fixture.module);
    let b1_bindings_before = b1.bindings.clone();
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &b1.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "same-source B1 binding substitution"
    );
    assert_eq!(b1.bindings, b1_bindings_before);
    assert_replay();
    let b1_primary_before = b1.primary.clone();
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &fixture.bindings,
            &b1.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "same-source B1 primary-term substitution"
    );
    assert_eq!(b1.primary, b1_primary_before);
    assert_replay();
    let b1_atomic_before = b1.atomic.clone();
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &b1.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "same-source B1 atomic-formula substitution"
    );
    assert_eq!(b1.atomic, b1_atomic_before);
    assert_replay();
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &b1.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    let mut foreign_contexts = binding_env(fixture.source, &fixture.module)
        .contexts()
        .clone();
    foreign_contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(fixture.source, 69, 102),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![9])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    let foreign_binding = BindingEnv::try_new(BindingEnvParts {
        source_id: fixture.source,
        module_id: fixture.module.clone(),
        contexts: foreign_contexts,
        bindings: fixture.bindings.bindings().clone(),
        diagnostics: fixture.bindings.diagnostics().clone(),
    })
    .expect("foreign proof scope binding env");
    let foreign_primary = SourcePrimaryTermProducer::build(
        b3_primary_input(fixture.source, &fixture.module),
        &foreign_binding,
        &fixture.arena,
    )
    .expect("foreign proof scope lower handoff");
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &foreign_primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    assert_eq!(
        valid.validate_installation(
            source_id(999),
            &fixture.module,
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign")),
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    let substituted_wrapper = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(33) {
            row.children = vec![TypedNodeId::new(26)];
        }
    });
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            &substituted_wrapper,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    let substituted_reference = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(32) {
            row.children = vec![TypedNodeId::new(8)];
        }
    });
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            &substituted_reference,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_replay();
    let actual_labels = pristine_symbols
        .labels()
        .by_contribution(fixture.contribution);
    assert_eq!(actual_labels.len(), 1);
    let actual_label = actual_labels[0].clone();
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
                LabelOriginPath::new("task258b3::wrong::theorem::origin")
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
                    let (import_owner, _) = b1
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
                        LabelOriginPath::new("task258b3::duplicate::theorem::origin"),
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
            "Task258B3 label mutation {name}"
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
    for index in 0..49 {
        for mutation in ["range", "recovery", "degraded", "kind", "children"] {
            let mutated_arena = match mutation {
                "range" => mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        let SourceAnchor::Range(mut source_range) = row.anchor.clone() else {
                            unreachable!("Task258B3 range anchor")
                        };
                        source_range.end += 1;
                        row.anchor = SourceAnchor::Range(source_range);
                    }
                }),
                "recovery" => mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.recovery = NodeRecoveryState::Recovered;
                    }
                }),
                "degraded" => mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.recovery = NodeRecoveryState::Degraded;
                    }
                }),
                "kind" => mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.kind = "source.task258b3.mutated".into();
                    }
                }),
                "children" => mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        if row.children.len() > 1 {
                            row.children.swap(0, 1);
                        } else if row.children.len() == 1 {
                            row.children.clear();
                        } else {
                            row.children.push(TypedNodeId::new(usize::from(index == 0)));
                        }
                    }
                }),
                _ => unreachable!("Task258B3 arena mutation"),
            };
            let statement_before_installation = statement.clone();
            let primary_before_installation = fixture.primary.clone();
            let arena_before_installation = mutated_arena.clone();
            assert_eq!(
                valid.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &mutated_arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {mutation} installation"
            );
            assert_eq!(statement, statement_before_installation);
            assert_eq!(fixture.primary, primary_before_installation);
            assert_eq!(mutated_arena, arena_before_installation);

            let statement_before_build = statement.clone();
            let primary_before_build = fixture.primary.clone();
            let arena_before_build = mutated_arena.clone();
            assert_eq!(
                SourceStatementWitnessProducer::build(
                    fixture.witness_input(),
                    &statement,
                    &fixture.primary,
                    &mutated_arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {mutation} producer"
            );
            assert_eq!(statement, statement_before_build);
            assert_eq!(fixture.primary, primary_before_build);
            assert_eq!(mutated_arena, arena_before_build);
            assert_replay();
        }
    }
}

#[test]
fn task258b3_paired_typed_ownership_and_cross_family_orders_are_atomic() {
    let fixture = B3Fixture::new(42);
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
    let installed = base
        .clone()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("paired install");
    assert_eq!(installed.source_statement(), Some(&statement));
    assert_eq!(installed.source_statement_witnesses(), Some(&witnesses));
    assert!(installed.source_statement_references().is_none());
    let installed_debug = installed.debug_text();
    assert_eq!(
        installed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(installed.debug_text(), installed_debug);

    let task258a = Fixture::new(42);
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
    .expect("Task258A empty typed AST")
    .with_source_term(task258a.primary.clone())
    .expect("Task258A Task252")
    .with_source_atomic_formula(task258a.atomic.clone())
    .expect("Task258A Task256")
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
        installed.clone().with_source_statement(task258a_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(installed.debug_text(), installed_debug);

    let b2 = B2Fixture::new(42);
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
        installed.clone().with_source_statement(b2_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(installed.debug_text(), installed_debug);
    let b1 = B1Fixture::new(42);
    let b1_references = b1.references(b1.reference_input()).expect("B1 references");
    let b1_installed = b1
        .empty_typed()
        .with_source_statement_references(b1.statement.clone(), b1_references.clone())
        .expect("B1 install");
    let b1_debug = b1_installed.debug_text();
    assert_eq!(
        b1_installed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b1_installed.debug_text(), b1_debug);
    assert_eq!(
        installed
            .clone()
            .with_source_statement_references(b1.statement.clone(), b1_references,),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(installed.debug_text(), installed_debug);
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
        installed.clone().with_source_context_for_test(
            task248
                .source_context()
                .expect("Task248 source context")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceContext)
    );
    assert_eq!(installed.debug_text(), installed_debug);

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
            "{family} first"
        );
        assert_eq!(occupied.debug_text(), debug, "{family} rollback");
    }
    assert_eq!(
        installed.clone().with_source_composite_formula(
            task257a
                .source_composite_formula()
                .expect("Task257A composite")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceCompositeFormula)
    );
    assert_eq!(
        installed.clone().with_source_formula_composition(
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
        installed.clone().with_source_condition_formula_composition(
            task257c2
                .source_condition_formula_composition()
                .expect("Task257C2 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceConditionFormulaComposition)
    );
    assert_eq!(
        installed.clone().with_source_predicate_chain_composition(
            task257c3
                .source_predicate_chain_composition()
                .expect("Task257C3 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourcePredicateChainComposition)
    );
    assert_eq!(installed.debug_text(), installed_debug);
    assert_eq!(
        base.with_source_statement_witnesses(statement, witnesses)
            .expect("Task258B3 replay")
            .debug_text(),
        installed_debug
    );
}

#[test]
fn task258b3_final_clone_orphan_stale_half_and_empty_semantics_are_stable() {
    let fixture = B3Fixture::new(43);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("typed pair");
    let resolved = assemble_empty_resolved(&typed).expect("final assembly");
    let typed_debug = typed.debug_text();
    let resolved_debug = resolved.debug_text();
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
    let mut stale = witnesses;
    stale.statement_fingerprint.push('x');
    let mut stale_pair = fixture.empty_typed();
    stale_pair.inject_source_statement_witness_bundle_for_test(statement, stale);
    assert_eq!(
        assemble_empty_resolved(&stale_pair),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let statement = typed.source_statement().expect("statement").clone();
    let witnesses = typed
        .source_statement_witnesses()
        .expect("witnesses")
        .clone();
    let mut stale_primary = witnesses.clone();
    stale_primary.primary_term_fingerprint.push('x');
    let mut stale_primary_pair = fixture.empty_typed();
    stale_primary_pair
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_primary);
    assert_eq!(
        assemble_empty_resolved(&stale_primary_pair),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let b2 = B2Fixture::new(43);
    let mut cross_profile = fixture.empty_typed();
    cross_profile.inject_source_statement_witness_bundle_for_test(
        b2.build(b2.input()).expect("B2 statement"),
        witnesses.clone(),
    );
    assert_eq!(
        assemble_empty_resolved(&cross_profile),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let b1 = B1Fixture::new(43);
    let mut references_and_witnesses = fixture.empty_typed();
    references_and_witnesses.inject_source_statement_bundle_for_test(
        statement.clone(),
        b1.references(b1.reference_input()).expect("B1 references"),
    );
    references_and_witnesses.inject_source_statement_witnesses_for_test(witnesses);
    assert_eq!(
        assemble_empty_resolved(&references_and_witnesses),
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
        let occupied_debug = occupied.debug_text();
        assert_eq!(
            assemble_empty_resolved(&occupied),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{table:?}"
        );
        assert_eq!(occupied.debug_text(), occupied_debug, "{table:?} rollback");
    }
    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("task258b3-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 82, 83),
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
                    role: SourceNodeRole::new("source.statement-witness.semantic-coexistence",),
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
                expr: ExprId::new("task258b3-semantic-coexistence"),
                typed_site: node(32),
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
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
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("replay")
            .debug_text(),
        resolved_debug
    );
}
