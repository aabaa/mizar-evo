use super::*;

#[test]
fn task258b3m2b2a_exact_nested_parenthesized_witness_api_debug_and_compatibility_are_stable() {
    let fixture = B3M2B2AFixture::new(82);
    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2a_profile());
    assert!(!statement.is_task_258b3_profile());
    assert!(!statement.is_task_258b3n_profile());
    assert!(!statement.is_task_258b3m1_profile());
    assert!(!statement.is_task_258b3m2a_profile());
    assert!(!statement.is_task_258b3m2b1_profile());
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
        (2, 1, 0, 7, 5, 0, 2, 0, 4, 4)
    );
    assert_eq!(
        (
            fixture.atomic.predicate_segments().len(),
            fixture.atomic.predicate_heads().len(),
            fixture.atomic.candidates().len(),
            fixture.atomic.type_sites().len(),
            fixture.atomic.attributes().len(),
        ),
        (0, 0, 0, 0, 0)
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
                source_range: range(fixture.source, 82, 119),
            },
            Some(BindingContextId::new(0)),
            BindingContextLayer::Proof,
            Some(vec![0]),
            &[][..],
            &[BindingId::new(0)][..],
            BindingContextRecovery::Normal,
        )
    );
    let term_sites = [30, 32, 40, 38, 36, 44, 46];
    let term_ranges = [
        (76, 77),
        (80, 81),
        (95, 100),
        (96, 99),
        (97, 98),
        (109, 110),
        (113, 114),
    ];
    for index in 0..7 {
        let term = fixture
            .primary
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .expect("term");
        assert_eq!(term.site().node().index(), term_sites[index]);
        assert_eq!(
            term.source_range(),
            range(fixture.source, term_ranges[index].0, term_ranges[index].1)
        );
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(
            term.context(),
            BindingContextId::new(usize::from(index >= 2))
        );
        assert_eq!(
            term.spelling(),
            match index {
                2 => "( ( x ) )",
                3 => "( x )",
                _ => "x",
            }
        );
        assert_eq!(
            term.kind(),
            if matches!(index, 2 | 3) {
                SourcePrimaryTermKind::Parenthesized
            } else {
                SourcePrimaryTermKind::VariableReference
            }
        );
        assert_eq!(term.role(), SourcePrimaryTermRole::Value);
        assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
        assert_eq!(
            term.parent().map(SourcePrimaryTermId::index),
            match index {
                3 => Some(2),
                4 => Some(3),
                _ => None,
            }
        );
    }
    for (reference_index, term_index) in [0, 1, 4, 5, 6].into_iter().enumerate() {
        let reference = fixture
            .primary
            .references()
            .get(SourcePrimaryTermReferenceId::new(reference_index))
            .expect("dense reference");
        assert_eq!(reference.term(), SourcePrimaryTermId::new(term_index));
        assert_eq!(reference.binding(), BindingId::new(0));
        assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
        assert_eq!(reference.use_ordinal(), 1);
        assert_eq!(
            reference.lexical_scope().map(|scope| scope.path()),
            if term_index < 2 { None } else { Some(&[0][..]) }
        );
    }
    assert!(
        fixture
            .primary
            .references()
            .iter()
            .all(|(_, reference)| !matches!(reference.term().index(), 2 | 3))
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
        [0, 1, 5, 6]
    );
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(_, fact)| fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [vec![0, 1], vec![3, 4]]
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.source_range(),
                row.source_ordinal(),
                row.context().index(),
            ))
            .collect::<Vec<_>>(),
        [
            (53, range(fixture.source, 19, 120), 0, 0),
            (51, range(fixture.source, 104, 115), 2, 1),
        ]
    );
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(owner.symbol(), &fixture.symbol);
    assert_eq!(owner.contribution(), fixture.contribution);
    assert_eq!(owner.site().node().index(), 53);
    assert_eq!(owner.source_range(), range(fixture.source, 19, 120));
    assert_eq!(
        owner.spelling(),
        "FormulaStatementNestedParenthesizedWitnessSmoke"
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);

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
    assert_eq!(witness.owner(), SourceTheoremOwnerId::new(0));
    assert_eq!(witness.binding_context(), BindingContextId::new(1));
    assert_eq!(
        witness.term(),
        SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
    );
    assert_eq!(witness.take_site().node().index(), 43);
    assert_eq!(witness.take_range(), range(fixture.source, 90, 101));
    assert_eq!(witness.site().node().index(), 42);
    assert_eq!(witness.source_range(), range(fixture.source, 95, 100));
    assert_eq!(witness.source_ordinal(), 1);
    assert_eq!(witness.ordinal(), 0);
    assert_eq!(witness.spelling(), "( ( x ) )");
    assert_eq!(witness.kind(), SourceStatementWitnessKind::Unnamed);
    assert_eq!(witness.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(witness.name(), None);
    let mut source_partition = statement
        .statements()
        .iter()
        .map(|(_, row)| row.source_ordinal())
        .chain(
            witnesses
                .witnesses()
                .iter()
                .map(|(_, row)| row.source_ordinal()),
        )
        .collect::<Vec<_>>();
    source_partition.sort_unstable();
    assert_eq!(source_partition, [0, 1, 2]);
    assert_eq!(
        witnesses.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=90..101 take_site=43 range=95..100 site=42 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"( ( x ) )\"\n",
            statement.debug_text(),
            fixture.primary.debug_text(),
        )
    );
    for (index, (start, end)) in TASK258B3M2B2A_NODE_RANGES.iter().copied().enumerate() {
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
                task258b3m2b2a_node_kind(index),
                NodeRecoveryState::Normal,
                task258b3m2b2a_node_children(index).to_vec(),
            ),
            "node {index}"
        );
    }
    let prior = B3M2B1Fixture::new(83);
    let prior_statement = prior.statement();
    assert!(
        prior
            .witnesses(&prior_statement, prior.witness_input())
            .expect("B3M2B1 compatibility")
            .debug_text()
            .starts_with("source-statement-witness-debug-v1\n")
    );
}

#[test]
fn task258b3m2b2a_dependencies_parent_chain_witness_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2AFixture::new(84);
    let statement = fixture.statement();
    let statement_input = fixture.statement_input();
    let witness_input = fixture.witness_input();
    let baseline = fixture
        .witnesses(&statement, witness_input.clone())
        .expect("baseline");
    let baseline_debug = baseline.debug_text();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let pristine_statement = statement.clone();
    let pristine_statement_input = statement_input.clone();
    let pristine_witness_input = witness_input.clone();
    let pristine_baseline = baseline.clone();
    let pristine_typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), baseline.clone())
        .expect("pristine paired typed target");
    let pristine_typed_debug = pristine_typed.debug_text();
    let pristine_final = assemble_empty_resolved(&pristine_typed).expect("pristine final");
    let pristine_final_debug = pristine_final.debug_text();
    let assert_pristine = || {
        assert_eq!(fixture.symbols, pristine_symbols);
        assert_eq!(fixture.bindings, pristine_bindings);
        assert_eq!(fixture.primary, pristine_primary);
        assert_eq!(fixture.atomic, pristine_atomic);
        assert_eq!(fixture.arena, pristine_arena);
        assert_eq!(statement, pristine_statement);
        assert_eq!(fixture.statement_input(), pristine_statement_input);
        assert_eq!(statement_input, pristine_statement_input);
        assert_eq!(witness_input, pristine_witness_input);
        assert_eq!(baseline, pristine_baseline);
        let replay_statement = SourceStatementProducer::build(
            statement_input.clone(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        )
        .expect("pristine statement replay");
        assert_eq!(replay_statement, statement);
        let replay_witnesses = SourceStatementWitnessProducer::build(
            witness_input.clone(),
            &replay_statement,
            &fixture.primary,
            &fixture.arena,
        )
        .expect("pristine witness replay");
        assert_eq!(replay_witnesses, baseline);
        let replay_typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(replay_statement, replay_witnesses)
            .expect("pristine paired typed replay");
        assert_eq!(replay_typed.debug_text(), pristine_typed_debug);
        let replay_final = assemble_empty_resolved(&replay_typed).expect("pristine final replay");
        assert_eq!(replay_final.debug_text(), pristine_final_debug);
    };

    for mutation in 0..2 {
        let mut input = witness_input.clone();
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
        assert_pristine();
    }
    let mut extra_name = witness_input.clone();
    extra_name.names.push(SourceStatementWitnessNameInput {
        witness: SourceStatementWitnessId::new(0),
        site: node(42),
        source_range: range(fixture.source, 95, 100),
        spelling: "( ( x ) )".to_owned(),
        recovery: SourceStatementRecovery::Normal,
    });
    assert_eq!(
        fixture.witnesses(&statement, extra_name),
        Err(SourceStatementWitnessError::InvalidAggregate),
        "extra name for unnamed witness"
    );
    assert_pristine();
    for mutation in 0..15 {
        let mut input = witness_input.clone();
        let row = &mut input.witnesses[0];
        match mutation {
            0 => row.owner = SourceTheoremOwnerId::new(1),
            1 => row.binding_context = BindingContextId::new(0),
            2 => row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3)),
            3 => row.take_site = node(42),
            4 => row.take_range.start += 1,
            5 => row.take_range.end -= 1,
            6 => row.site = node(41),
            7 => row.source_range.start += 1,
            8 => row.source_range.end -= 1,
            9 => row.source_ordinal = 2,
            10 => row.ordinal = 1,
            11 => row.spelling = "( x )".to_owned(),
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
        assert_pristine();
    }
    let mut dependency_first = witness_input.clone();
    dependency_first.source_id = source_id(85);
    dependency_first.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_first),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_pristine();
    let mut aggregate_before_witness = witness_input.clone();
    aggregate_before_witness.witnesses[0].ordinal = 1;
    aggregate_before_witness
        .witnesses
        .push(aggregate_before_witness.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, aggregate_before_witness),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_pristine();

    let foreign_m2a = B3M2AFixture::new(84);
    assert_eq!(foreign_m2a.source, fixture.source);
    assert_eq!(foreign_m2a.module, fixture.module);
    let mut bad_target_aggregate = statement_input.clone();
    bad_target_aggregate.owners.clear();
    assert_eq!(
        SourceStatementProducer::build(
            bad_target_aggregate,
            &fixture.symbols,
            &foreign_m2a.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "foreign M2A binding precedes target aggregate"
    );
    assert_pristine();

    let primary_mutations = [
        "outer-extra-reference",
        "inner-extra-reference",
        "leaf-reference-removal",
        "leaf-reference-remap-outer",
        "leaf-reference-remap-inner",
        "leaf-reference-duplicate",
        "outer-inner-detach",
        "outer-inner-remap",
        "inner-leaf-detach",
        "inner-leaf-remap",
    ];
    for (mutation, label) in primary_mutations.into_iter().enumerate() {
        let mut input = B3M2B2AFixture::primary_input(fixture.source, &fixture.module);
        match mutation {
            0 | 1 => input.references.push(SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(2 + mutation),
                binding: BindingId::new(0),
                role: SourcePrimaryTermReferenceRole::Variable,
            }),
            2 => {
                input.references.remove(2);
            }
            3 => input.references[2].term = SourcePrimaryTermId::new(2),
            4 => input.references[2].term = SourcePrimaryTermId::new(3),
            5 => input.references.push(input.references[2].clone()),
            6 => input.terms[3].parent = None,
            7 => input.terms[3].parent = Some(SourcePrimaryTermId::new(1)),
            8 => input.terms[4].parent = None,
            9 => input.terms[4].parent = Some(SourcePrimaryTermId::new(2)),
            _ => unreachable!(),
        }
        let rejected =
            match SourcePrimaryTermProducer::build(input, &fixture.bindings, &fixture.arena) {
                Err(_) => true,
                Ok(primary) => {
                    let atomic = B3M2B2AFixture::build_atomic(
                        fixture.source,
                        &fixture.module,
                        &fixture.bindings,
                        &fixture.symbols,
                        &primary,
                        &fixture.arena,
                        [0, 5],
                    )
                    .expect("altered primary remains valid for Task256");
                    SourceStatementProducer::build(
                        fixture.statement_input(),
                        &fixture.symbols,
                        &fixture.bindings,
                        &primary,
                        &atomic,
                        &fixture.arena,
                    ) == Err(SourceStatementError::DependencyMismatch)
                }
            };
        assert!(rejected, "{label}");
        assert_pristine();
    }
    for target in [2, 3, 4] {
        let mut targets = [0, 1, 5, 6];
        targets[2] = target;
        let rejected = match B3M2B2AFixture::build_atomic_targets(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            &fixture.arena,
            targets,
        ) {
            Err(_) => true,
            Ok(atomic) => {
                SourceStatementProducer::build(
                    fixture.statement_input(),
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &atomic,
                    &fixture.arena,
                ) == Err(SourceStatementError::DependencyMismatch)
            }
        };
        assert!(
            rejected,
            "Task256 edge/request contamination by term {target}"
        );
        assert_pristine();
    }

    for mutation in 0..5 {
        let mut input = fixture.statement_input();
        match mutation {
            0 => input.statements[0].source_ordinal = 1,
            1 => input.statements[1].source_ordinal = 1,
            2 => input.statements[1].source_range.start += 1,
            3 => input.input_facts[1].uses.swap(0, 1),
            4 => {
                input.candidate_facts[1].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
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
            "base mutation {mutation}"
        );
        assert_pristine();
    }
    let mut stale_origin = statement.clone();
    stale_origin.checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
        fixture.symbol.clone(),
        range(fixture.source, 19, 120),
        SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 19, 120)),
            vec![1],
        ),
    );
    assert_eq!(
        stale_origin.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0),
        })
    );
    assert_pristine();
    for fingerprint in 0..3 {
        let mut stale_base = statement.clone();
        match fingerprint {
            0 => stale_base.binding_fingerprint.push('x'),
            1 => stale_base.primary_term_fingerprint.push('x'),
            2 => stale_base.atomic_formula_fingerprint.push('x'),
            _ => unreachable!(),
        }
        assert_eq!(
            stale_base.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch),
            "base fingerprint {fingerprint}"
        );
        assert_pristine();
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
    assert_pristine();
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
    assert_pristine();

    let actual_labels = pristine_symbols
        .labels()
        .by_contribution(fixture.contribution);
    assert_eq!(actual_labels.len(), 1);
    assert!(!pristine_symbols.labels().is_empty());
    let actual_label = actual_labels[0].clone();
    let pristine_contribution = pristine_symbols
        .contributions()
        .get(fixture.contribution)
        .expect("fixture contribution")
        .clone();
    let make_symbols_with_labels = |labels: LabelIndex| {
        SymbolEnv::new(
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
        )
    };
    let mut import_owner_builder = ResolvedArenaBuilder::new();
    let import_owner = import_owner_builder
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            actual_label.origin().clone(),
        ))
        .expect("real imported-label owner node");
    let mut temporary_imports = ResolvedImports::new();
    let imported_edge = temporary_imports.push_import(ResolvedImport::new(
        import_owner,
        range(fixture.source, 0, 1),
        "import statement.fixture;",
        None,
        ImportResolution::Resolved(fixture.module.clone()),
        actual_label.origin().clone(),
    ));
    assert_eq!(temporary_imports.imports().count(), 1);
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
                LabelOriginPath::new("statement.fixture.wrong.theorem")
            } else {
                actual_label.origin_path().clone()
            };
            let kind = if case == 4 {
                LabelKind::ProofStep
            } else {
                actual_label.kind()
            };
            let origin = match case {
                0 => actual_label
                    .origin()
                    .clone()
                    .with_import_edge(imported_edge),
                7 => actual_label.origin().clone().recovered(),
                _ => actual_label.origin().clone(),
            };
            let visibility = if case == 5 {
                Visibility::Private
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
                        LabelOriginPath::new("statement.fixture.duplicate.theorem"),
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
        let attempted_symbols = make_symbols_with_labels(labels);
        let attempted_symbols_before = attempted_symbols.clone();
        assert_ne!(
            attempted_symbols, pristine_symbols,
            "{name} environment differs"
        );
        assert_eq!(attempted_symbols.imports(), pristine_symbols.imports());
        assert_eq!(attempted_symbols.exports(), pristine_symbols.exports());
        assert_eq!(attempted_symbols.symbols(), pristine_symbols.symbols());
        assert_eq!(
            attempted_symbols.definitions(),
            pristine_symbols.definitions()
        );
        assert_eq!(attempted_symbols.overloads(), pristine_symbols.overloads());
        assert_eq!(
            attempted_symbols.registrations(),
            pristine_symbols.registrations()
        );
        assert_eq!(
            attempted_symbols.lexical_summaries(),
            pristine_symbols.lexical_summaries()
        );
        assert_eq!(
            attempted_symbols.namespace_graph(),
            pristine_symbols.namespace_graph()
        );
        assert_eq!(
            attempted_symbols.declaration_dependencies(),
            pristine_symbols.declaration_dependencies()
        );
        assert_eq!(
            attempted_symbols.module_summaries(),
            pristine_symbols.module_summaries()
        );
        assert_eq!(
            attempted_symbols.contributions(),
            pristine_symbols.contributions(),
            "{name} contribution effects stay unchanged"
        );
        let attempted_contribution = attempted_symbols
            .contributions()
            .get(fixture.contribution)
            .expect("preserved fixture contribution");
        assert_eq!(
            attempted_contribution.effects(),
            pristine_contribution.effects(),
            "{name} contribution effects remain pristine"
        );
        assert!(!attempted_contribution.effects().symbols().is_empty());
        assert!(!attempted_contribution.effects().definitions().is_empty());
        assert!(!attempted_contribution.effects().labels().is_empty());
        assert!(
            CheckedStatementOwner::validate_exact_local_theorem(
                &attempted_symbols,
                fixture.symbol.clone(),
                fixture.source,
                &fixture.module,
            )
            .is_ok(),
            "{name} owner-only check remains valid"
        );
        assert_eq!(
            SourceStatementProducer::build(
                statement_input.clone(),
                &attempted_symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            }),
            "{name} exact resolver-label join owner"
        );
        assert_eq!(
            attempted_symbols, attempted_symbols_before,
            "{name} env purity"
        );
        assert_pristine();
    }
    for index in 0..TASK258B3M2B2A_NODE_RANGES.len() {
        for mutation in 0..5 {
            let arena = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    match mutation {
                        0 => {
                            let SourceAnchor::Range(mut source_range) = row.anchor.clone() else {
                                unreachable!("Task258B3M2B2A range")
                            };
                            source_range.end += 1;
                            row.anchor = SourceAnchor::Range(source_range);
                        }
                        1 => row.recovery = NodeRecoveryState::Recovered,
                        2 => row.recovery = NodeRecoveryState::Degraded,
                        3 => row.kind = "source.task258b3m2b2a.mutated".into(),
                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                        4 if row.children.len() == 1 => row.children.clear(),
                        4 => row.children.push(TypedNodeId::new(usize::from(index == 0))),
                        _ => unreachable!(),
                    }
                }
            });
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
            assert_eq!(
                arena, arena_before,
                "installation arena purity {index}/{mutation}"
            );
            assert_pristine();
            assert_eq!(
                SourceStatementWitnessProducer::build(
                    witness_input.clone(),
                    &statement,
                    &fixture.primary,
                    &arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "constructor node {index} mutation {mutation}"
            );
            assert_eq!(
                arena, arena_before,
                "constructor arena purity {index}/{mutation}"
            );
            assert_pristine();
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
fn task258b3m2b2a_paired_ownership_hybrids_and_all_family_orders_are_atomic() {
    let source_ordinal = 86;
    let fixture = B3M2B2AFixture::new(source_ordinal);
    let statement = fixture.statement();
    let statement_input = fixture.statement_input();
    let witness_input = fixture.witness_input();
    let witnesses = fixture
        .witnesses(&statement, witness_input.clone())
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
    let final_debug = assemble_empty_resolved(&typed)
        .expect("target final before foreign attempts")
        .debug_text();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let pristine_statement_input = statement_input.clone();
    let pristine_witness_input = witness_input.clone();
    let assert_target_pristine = || {
        assert_eq!(fixture.symbols, pristine_symbols);
        assert_eq!(fixture.bindings, pristine_bindings);
        assert_eq!(fixture.primary, pristine_primary);
        assert_eq!(fixture.atomic, pristine_atomic);
        assert_eq!(fixture.arena, pristine_arena);
        assert_eq!(statement_input, pristine_statement_input);
        assert_eq!(witness_input, pristine_witness_input);
        assert_eq!(statement, fixture.statement());
        assert_eq!(
            witnesses,
            fixture
                .witnesses(&statement, witness_input.clone())
                .expect("target witness replay")
        );
        assert_eq!(base.debug_text(), base_debug);
        assert_eq!(typed.debug_text(), typed_debug);
        let replay_statement = SourceStatementProducer::build(
            statement_input.clone(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        )
        .expect("target statement replay");
        let replay_witnesses = SourceStatementWitnessProducer::build(
            witness_input.clone(),
            &replay_statement,
            &fixture.primary,
            &fixture.arena,
        )
        .expect("target witness constructor replay");
        let replay_typed = fixture
            .empty_typed()
            .with_source_statement_witnesses(replay_statement, replay_witnesses)
            .expect("target paired replay");
        assert_eq!(replay_typed.debug_text(), typed_debug);
        assert_eq!(
            assemble_empty_resolved(&replay_typed)
                .expect("target final replay")
                .debug_text(),
            final_debug
        );
    };
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert!(typed.source_statement_references().is_none());
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let task258a = Fixture::new(87);
    let task258a_statement = task258a
        .build(task258a.input())
        .expect("Task258A statement");
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
    .expect("Task258A typed base")
    .with_source_term(task258a.primary.clone())
    .expect("Task258A Task252")
    .with_source_atomic_formula(task258a.atomic.clone())
    .expect("Task258A Task256")
    .with_source_statement(task258a_statement.clone())
    .expect("Task258A installed");
    let task258b1 = B1Fixture::new(88);
    let task258b1_references = task258b1
        .references(task258b1.reference_input())
        .expect("Task258B1 references");
    let task258b1_typed = task258b1
        .empty_typed()
        .with_source_statement_references(task258b1.statement.clone(), task258b1_references.clone())
        .expect("Task258B1 installed");
    let task258b2 = B2Fixture::new(89);
    let task258b2_statement = task258b2
        .build(task258b2.input())
        .expect("Task258B2 statement");
    let task258b2_typed = task258b2
        .empty_typed()
        .with_source_statement(task258b2_statement.clone())
        .expect("Task258B2 installed");

    let b3 = B3Fixture::new(90);
    let b3_statement = b3.statement();
    let b3_witnesses = b3.witnesses(&b3_statement, b3.witness_input()).expect("B3");
    let b3n = B3NFixture::new(91);
    let b3n_statement = b3n.statement();
    let b3n_witnesses = b3n
        .witnesses(&b3n_statement, b3n.witness_input())
        .expect("B3N");
    let b3m1 = B3M1Fixture::new(92);
    let b3m1_statement = b3m1.statement();
    let b3m1_witnesses = b3m1
        .witnesses(&b3m1_statement, b3m1.witness_input())
        .expect("B3M1");
    let b3m2a = B3M2AFixture::new(93);
    let b3m2a_statement = b3m2a.statement();
    let b3m2a_witnesses = b3m2a
        .witnesses(&b3m2a_statement, b3m2a.witness_input())
        .expect("B3M2A");
    let b3m2b1 = B3M2B1Fixture::new(94);
    let b3m2b1_statement = b3m2b1.statement();
    let b3m2b1_witnesses = b3m2b1
        .witnesses(&b3m2b1_statement, b3m2b1.witness_input())
        .expect("B3M2B1");
    for (label, occupied) in [
        ("Task258A", task258a_typed),
        ("Task258B1", task258b1_typed),
        ("Task258B2", task258b2_typed),
        (
            "Task258B3",
            b3.empty_typed()
                .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
                .expect("B3 typed"),
        ),
        (
            "Task258B3N",
            b3n.empty_typed()
                .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone())
                .expect("B3N typed"),
        ),
        (
            "Task258B3M1",
            b3m1.empty_typed()
                .with_source_statement_witnesses(b3m1_statement.clone(), b3m1_witnesses.clone())
                .expect("B3M1 typed"),
        ),
        (
            "Task258B3M2A",
            b3m2a
                .empty_typed()
                .with_source_statement_witnesses(b3m2a_statement.clone(), b3m2a_witnesses.clone())
                .expect("B3M2A typed"),
        ),
        (
            "Task258B3M2B1",
            b3m2b1
                .empty_typed()
                .with_source_statement_witnesses(b3m2b1_statement.clone(), b3m2b1_witnesses.clone())
                .expect("B3M2B1 typed"),
        ),
    ] {
        let occupied_debug = occupied.debug_text();
        assert_eq!(
            occupied
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "{label} then B3M2B2A"
        );
        assert_eq!(
            occupied.debug_text(),
            occupied_debug,
            "{label} typed purity"
        );
        assert_target_pristine();
    }
    assert_eq!(
        typed.clone().with_source_statement(task258a_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_target_pristine();
    assert_eq!(
        typed
            .clone()
            .with_source_statement_references(task258b1.statement.clone(), task258b1_references,),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_target_pristine();
    assert_eq!(
        typed.clone().with_source_statement(task258b2_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_target_pristine();
    for (label, prior_statement, prior_witnesses) in [
        ("B3", b3_statement, b3_witnesses),
        ("B3N", b3n_statement, b3n_witnesses),
        ("B3M1", b3m1_statement, b3m1_witnesses),
        ("B3M2A", b3m2a_statement, b3m2a_witnesses),
        ("B3M2B1", b3m2b1_statement, b3m2b1_witnesses),
    ] {
        assert_eq!(
            typed
                .clone()
                .with_source_statement_witnesses(prior_statement, prior_witnesses),
            Err(TypedAstError::InvalidSourceStatement),
            "B3M2B2A then {label}"
        );
        assert_eq!(
            typed.debug_text(),
            typed_debug,
            "{label} target typed purity"
        );
        assert_target_pristine();
    }

    let same_source_a = Fixture::new(source_ordinal);
    let same_source_b1 = B1Fixture::new(source_ordinal);
    let same_source_b2 = B2Fixture::new(source_ordinal);
    let same_source_b3 = B3Fixture::new(source_ordinal);
    let same_source_b3n = B3NFixture::new(source_ordinal);
    let same_source_m1 = B3M1Fixture::new(source_ordinal);
    let same_source_m2a = B3M2AFixture::new(source_ordinal);
    let same_source_m2b1 = B3M2B1Fixture::new(source_ordinal);
    for (
        label,
        donor_source,
        donor_module,
        donor_bindings,
        donor_primary,
        donor_atomic,
        donor_arena,
        complete_error,
    ) in [
        (
            "A",
            same_source_a.source,
            &same_source_a.module,
            &same_source_a.bindings,
            &same_source_a.primary,
            &same_source_a.atomic,
            &same_source_a.arena,
            SourceStatementError::InvalidAggregate,
        ),
        (
            "B1",
            same_source_b1.source,
            &same_source_b1.module,
            &same_source_b1.bindings,
            &same_source_b1.primary,
            &same_source_b1.atomic,
            &same_source_b1.arena,
            SourceStatementError::InvalidAggregate,
        ),
        (
            "B2",
            same_source_b2.source,
            &same_source_b2.module,
            &same_source_b2.bindings,
            &same_source_b2.primary,
            &same_source_b2.atomic,
            &same_source_b2.arena,
            SourceStatementError::InvalidAggregate,
        ),
        (
            "B3",
            same_source_b3.source,
            &same_source_b3.module,
            &same_source_b3.bindings,
            &same_source_b3.primary,
            &same_source_b3.atomic,
            &same_source_b3.arena,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
        ),
        (
            "B3N",
            same_source_b3n.source,
            &same_source_b3n.module,
            &same_source_b3n.bindings,
            &same_source_b3n.primary,
            &same_source_b3n.atomic,
            &same_source_b3n.arena,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
        ),
        (
            "M1",
            same_source_m1.source,
            &same_source_m1.module,
            &same_source_m1.bindings,
            &same_source_m1.primary,
            &same_source_m1.atomic,
            &same_source_m1.arena,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
        ),
        (
            "M2A",
            same_source_m2a.source,
            &same_source_m2a.module,
            &same_source_m2a.bindings,
            &same_source_m2a.primary,
            &same_source_m2a.atomic,
            &same_source_m2a.arena,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
        ),
        (
            "M2B1",
            same_source_m2b1.source,
            &same_source_m2b1.module,
            &same_source_m2b1.bindings,
            &same_source_m2b1.primary,
            &same_source_m2b1.atomic,
            &same_source_m2b1.arena,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
        ),
    ] {
        assert_eq!(donor_source, fixture.source, "{label} same source");
        assert_eq!(donor_module, &fixture.module, "{label} same module");
        let donor_bindings_before = (*donor_bindings).clone();
        let donor_primary_before = (*donor_primary).clone();
        let donor_atomic_before = (*donor_atomic).clone();
        let donor_arena_before = (*donor_arena).clone();
        assert_eq!(
            SourceStatementProducer::build(
                statement_input.clone(),
                &fixture.symbols,
                &fixture.bindings,
                donor_primary,
                donor_atomic,
                donor_arena,
            ),
            Err(SourceStatementError::DependencyMismatch),
            "hybrid target bindings with {label} lower tuple"
        );
        assert_eq!(*donor_bindings, donor_bindings_before, "{label} bindings");
        assert_eq!(*donor_primary, donor_primary_before, "{label} primary");
        assert_eq!(*donor_atomic, donor_atomic_before, "{label} atomic");
        assert_eq!(*donor_arena, donor_arena_before, "{label} arena");
        assert_target_pristine();
        assert_eq!(
            SourceStatementProducer::build(
                statement_input.clone(),
                &fixture.symbols,
                donor_bindings,
                donor_primary,
                donor_atomic,
                donor_arena,
            ),
            Err(complete_error),
            "complete {label} lower tuple"
        );
        assert_eq!(*donor_bindings, donor_bindings_before, "{label} bindings");
        assert_eq!(*donor_primary, donor_primary_before, "{label} primary");
        assert_eq!(*donor_atomic, donor_atomic_before, "{label} atomic");
        assert_eq!(*donor_arena, donor_arena_before, "{label} arena");
        assert_target_pristine();
    }

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
            "{family} then B3M2B2A"
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

    let applications = SourceFunctorApplicationProducer::build(
        SourceFunctorApplicationHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            applications: Vec::new(),
            wrappers: Vec::new(),
            candidates: Vec::new(),
            arguments: Vec::new(),
            type_requests: Vec::new(),
        },
        &fixture.symbols,
        &fixture.bindings,
        &fixture.primary,
        &fixture.arena,
    )
    .expect("Task253 empty disjoint handoff");
    let structures = SourceStructureProducer::build(
        SourceStructureHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            terms: Vec::new(),
            wrappers: Vec::new(),
            roots: Vec::new(),
            members: Vec::new(),
            field_updates: Vec::new(),
            edges: Vec::new(),
            requests: Vec::new(),
        },
        &fixture.symbols,
        &fixture.bindings,
        &fixture.primary,
        None,
        &fixture.arena,
    )
    .expect("Task254 empty disjoint handoff");
    let set_terms = SourceSetTermProducer::build(
        SourceSetTermHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            terms: Vec::new(),
            wrappers: Vec::new(),
            generators: Vec::new(),
            type_sites: Vec::new(),
            conditions: Vec::new(),
            edges: Vec::new(),
            requests: Vec::new(),
        },
        &fixture.bindings,
        &fixture.primary,
        None,
        None,
        &fixture.arena,
    )
    .expect("Task255 empty disjoint handoff");
    let alternate_atomic = SourceAtomicFormulaProducer::build(
        SourceAtomicFormulaHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            formulas: Vec::new(),
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges: Vec::new(),
            requests: Vec::new(),
        },
        &fixture.bindings,
        &fixture.symbols,
        &fixture.primary,
        None,
        None,
        None,
        &fixture.arena,
    )
    .expect("Task256 distinct empty handoff");
    for (family, occupied) in [
        (
            "Task253",
            fixture
                .empty_typed()
                .with_source_application(applications.clone())
                .expect("Task253 first"),
        ),
        (
            "Task254",
            fixture
                .empty_typed()
                .with_source_structure(structures.clone())
                .expect("Task254 first"),
        ),
        (
            "Task255",
            fixture
                .empty_typed()
                .with_source_set_term(set_terms.clone())
                .expect("Task255 first"),
        ),
        (
            "Task256",
            fixture
                .primary_typed()
                .with_source_atomic_formula(alternate_atomic.clone())
                .expect("Task256 first"),
        ),
    ] {
        assert_eq!(
            occupied.with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "{family} then B3M2B2A"
        );
    }
    assert_eq!(
        typed.clone().with_source_application(applications),
        Err(TypedAstError::InvalidSourceApplication)
    );
    assert_eq!(
        typed.clone().with_source_structure(structures),
        Err(TypedAstError::InvalidSourceStructure)
    );
    assert_eq!(
        typed.clone().with_source_set_term(set_terms),
        Err(TypedAstError::InvalidSourceSetTerm)
    );
    assert_eq!(
        typed.clone().with_source_atomic_formula(alternate_atomic),
        Err(TypedAstError::InvalidSourceAtomicFormula)
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
fn task258b3m2b2a_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2AFixture::new(95);
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
    let b1 = B1Fixture::new(95);
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
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2a-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 95, 100),
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
                typed_node: TypedNodeId::new(40),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new(
                        "source.statement-witness-nested-parenthesized.semantic-coexistence",
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
                expr: ExprId::new("task258b3m2b2a-semantic-coexistence"),
                typed_site: node(40),
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
