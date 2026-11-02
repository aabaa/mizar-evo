use super::*;

#[test]
fn task258b3m2b2b2b_exact_structure_selector_witness_api_debug_and_legacy_compatibility_are_stable()
{
    let fixture = B3M2B2B2BFixture::new(214);
    assert_eq!(TASK258B3M2B2B2B_SOURCE.len(), 171);
    assert_eq!(
        TASK258B3M2B2B2B_SOURCE,
        "import parser.type_fixtures;\nreserve x for set;\ntheorem FormulaStatementStructureSelectorWitnessSmoke: x = x proof\n  take TypeCaseStruct(x: 1, y: 2).x;\n  thus x = x;\nend;\n"
    );
    assert_eq!(fixture.arena.len(), 79);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(78)));
    for (index, (start, end)) in TASK258B3M2B2B2B_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b2b_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b2b_node_children(index)
        );
    }

    assert_eq!(
        (
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len(),
        ),
        (6, 4, 2)
    );
    assert_eq!(
        (
            fixture.structure.terms().len(),
            fixture.structure.wrappers().len(),
            fixture.structure.roots().len(),
            fixture.structure.members().len(),
            fixture.structure.field_updates().len(),
            fixture.structure.edges().len(),
            fixture.structure.requests().len(),
        ),
        (2, 0, 1, 3, 0, 3, 9)
    );
    let selector = fixture
        .structure
        .terms()
        .get(SourceStructureTermId::new(0))
        .expect("selector");
    let constructor = fixture
        .structure
        .terms()
        .get(SourceStructureTermId::new(1))
        .expect("constructor");
    assert_eq!(selector.site().node(), TypedNodeId::new(62));
    assert_eq!(selector.kind(), SourceStructureTermKind::SelectorAccess);
    assert_eq!(constructor.site().node(), TypedNodeId::new(61));
    assert_eq!(constructor.kind(), SourceStructureTermKind::Constructor);
    let root = fixture
        .structure
        .roots()
        .get(SourceStructureRootId::new(0))
        .expect("imported root");
    assert_eq!(root.term(), SourceStructureTermId::new(1));
    assert_eq!(root.symbol(), &fixture.structure_symbol);
    assert_eq!(root.contribution(), fixture.structure_contribution);
    assert_eq!(root.contribution().index(), 2);
    assert_eq!(
        root.symbol().local().as_str(),
        "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
    );
    assert_eq!(
        root.symbol().fqn().as_str(),
        "parser.type_fixtures::TypeCaseStruct#5"
    );
    assert_eq!(root.origin().structural_path(), [5]);
    assert_eq!(
        root.origin().anchor(),
        &SourceAnchor::Range(range(fixture.source, 7, 27))
    );
    assert_eq!(root.visibility(), Visibility::Public);
    assert_eq!(root.export_status(), ExportStatus::Exported);
    assert!(root.signature().is_none());
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
    assert_eq!(fixture.atomic.application_fingerprint(), None);
    assert_eq!(fixture.atomic.structure_fingerprint(), None);

    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b2b_profile());
    assert!(!statement.is_task_258b3m2b2b2a_profile());
    let checked_owner = statement.checked_owner();
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .unwrap();
    assert_eq!(checked_owner.symbol(), &fixture.theorem_symbol);
    assert_eq!(owner.symbol(), checked_owner.symbol());
    assert_eq!(owner.contribution(), fixture.theorem_contribution);
    assert_eq!(checked_owner.source_range(), range(fixture.source, 48, 170));
    assert_eq!(checked_owner.source_range(), owner.source_range());
    assert_eq!(checked_owner.origin().structural_path(), [2, 1]);
    assert_eq!(checked_owner.origin().source_id(), fixture.source);
    assert_eq!(checked_owner.origin().module_id(), &fixture.module);
    assert_eq!(
        checked_owner.origin().anchor(),
        &SourceAnchor::Range(owner.source_range())
    );
    assert!(checked_owner.origin().import_edge().is_none());
    assert!(!checked_owner.origin().is_recovered());
    assert_eq!(checked_owner.visibility(), Visibility::Public);
    assert_eq!(checked_owner.export_status(), ExportStatus::Exported);
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
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("selector witness");
    assert_eq!(
        (witnesses.witnesses().len(), witnesses.names().len()),
        (1, 0)
    );
    assert_eq!(witnesses.application_fingerprint(), None);
    let structure_debug = fixture.structure.debug_text();
    assert_eq!(
        witnesses.structure_fingerprint(),
        Some(structure_debug.as_str())
    );
    assert_eq!(
        witnesses
            .witnesses()
            .get(SourceStatementWitnessId::new(0))
            .expect("witness")
            .term(),
        SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0))
    );
    let debug = witnesses.debug_text();
    let lines = debug.lines().collect::<Vec<_>>();
    let primary_line = lines
        .iter()
        .position(|line| line.starts_with("primary-term-fingerprint:"))
        .expect("primary fingerprint");
    let structure_line = lines
        .iter()
        .position(|line| line.starts_with("structure-fingerprint:"))
        .expect("structure fingerprint");
    let witness_line = lines
        .iter()
        .position(|line| line.starts_with("witness#0 "))
        .expect("witness row");
    assert!(primary_line < structure_line && structure_line < witness_line);
    assert!(!debug.contains("\napplication-fingerprint:"));
    assert!(debug.contains(" term=structure#0 "));

    let constructor_fixture = B3M2B2B2AFixture::new(215);
    let constructor_statement = constructor_fixture.statement();
    let constructor_witnesses = constructor_fixture
        .witnesses(&constructor_statement, constructor_fixture.witness_input())
        .expect("constructor witness");
    assert!(constructor_statement.is_task_258b3m2b2b2a_profile());
    assert!(!constructor_statement.is_task_258b3m2b2b2b_profile());
    assert_eq!(
        constructor_witnesses
            .witnesses()
            .get(SourceStatementWitnessId::new(0))
            .expect("constructor witness row")
            .term(),
        SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0))
    );
    assert_ne!(constructor_witnesses.debug_text(), debug);

    let legacy = B3M2B2AFixture::new(216);
    let legacy_statement = legacy.statement();
    let legacy_witnesses = legacy
        .witnesses(&legacy_statement, legacy.witness_input())
        .expect("legacy witness");
    assert_eq!(legacy_witnesses.application_fingerprint(), None);
    assert_eq!(legacy_witnesses.structure_fingerprint(), None);
    assert!(legacy_witnesses.debug_text().contains(" term=primary#2 "));
    let application = B3M2B2B1AFixture::new(217);
    let application_statement = application.statement();
    let application_witnesses = application
        .witnesses(&application_statement, application.witness_input())
        .expect("application witness");
    assert!(application_witnesses.application_fingerprint().is_some());
    assert_eq!(application_witnesses.structure_fingerprint(), None);
    assert!(
        application_witnesses
            .debug_text()
            .contains(" term=application#0 ")
    );
    assert_eq!(witnesses.clone().debug_text(), debug);
}

#[test]
fn task258b3m2b2b2b_dependencies_structure_selector_witness_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B2BFixture::new(218);
    let statement = fixture.statement();
    let baseline = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("baseline");
    baseline
        .validate_installation_with_structure(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.structure),
            &fixture.arena,
        )
        .expect("baseline revalidation");
    let typed = fixture
        .empty_typed()
        .with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            baseline.clone(),
        )
        .expect("baseline typed");
    let resolved = assemble_empty_resolved(&typed).expect("baseline resolved");
    let assert_clean_replay = || {
        let replay_statement = fixture.statement();
        assert_eq!(replay_statement, statement);
        let replay_witnesses = fixture
            .witnesses(&replay_statement, fixture.witness_input())
            .expect("producer replay");
        assert_eq!(replay_witnesses, baseline);
        let replay_typed = fixture
            .empty_typed()
            .with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                replay_statement,
                replay_witnesses,
            )
            .expect("whole installation replay");
        assert_eq!(replay_typed, typed);
        assert_eq!(assemble_empty_resolved(&replay_typed), Ok(resolved.clone()));
    };

    let foreign_binding = b3m2b2b2a_binding_env(fixture.source, &fixture.module);
    let mut binding_and_aggregate = fixture.statement_input();
    binding_and_aggregate.owners.clear();
    assert_eq!(
        SourceStatementProducer::build(
            binding_and_aggregate,
            &fixture.symbols,
            &foreign_binding,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "Task48 mismatch precedes aggregate validation"
    );

    let mut substituted_primary_input =
        B3M2B2B2BFixture::primary_input(fixture.source, &fixture.module);
    substituted_primary_input.terms[2].spelling = "3".to_owned();
    substituted_primary_input.numeric_type_requests[0].spelling = "3".to_owned();
    let substituted_primary = SourcePrimaryTermProducer::build(
        substituted_primary_input,
        &fixture.bindings,
        &fixture.arena,
    )
    .expect("coherent lower substitution");
    let mut primary_and_aggregate = fixture.statement_input();
    primary_and_aggregate.statements.clear();
    assert_eq!(
        SourceStatementProducer::build(
            primary_and_aggregate,
            &fixture.symbols,
            &fixture.bindings,
            &substituted_primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "Task252 mismatch precedes aggregate validation"
    );
    let foreign = B3M2B2B2BFixture::new(221);
    for (bindings, primary, atomic) in [
        (&foreign_binding, &fixture.primary, &fixture.atomic),
        (&fixture.bindings, &substituted_primary, &fixture.atomic),
        (&fixture.bindings, &fixture.primary, &foreign.atomic),
    ] {
        assert_ne!(
            (bindings, primary, atomic),
            (&fixture.bindings, &fixture.primary, &fixture.atomic)
        );
        assert_eq!(
            SourceStatementProducer::build(
                fixture.statement_input(),
                &fixture.symbols,
                bindings,
                primary,
                atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch)
        );
        assert_clean_replay();
    }
    let mut invalid_reference = B3M2B2B2BFixture::primary_input(fixture.source, &fixture.module);
    invalid_reference.references[2].role = SourcePrimaryTermReferenceRole::LocalConstant;
    assert!(
        SourcePrimaryTermProducer::build(invalid_reference, &fixture.bindings, &fixture.arena,)
            .is_err(),
        "Task252 rejects reference corruption"
    );
    let mut invalid_numeric_request =
        B3M2B2B2BFixture::primary_input(fixture.source, &fixture.module);
    invalid_numeric_request.numeric_type_requests[0].request_ordinal = 1;
    assert!(
        SourcePrimaryTermProducer::build(
            invalid_numeric_request,
            &fixture.bindings,
            &fixture.arena,
        )
        .is_err(),
        "Task252 rejects numeric-request corruption"
    );

    let atomic_input = || {
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            let first_term = if formula == 0 { 0 } else { 4 };
            for ordinal in 0..2 {
                let edge = SourceAtomicEdgeId::new(formula * 2 + ordinal);
                edges.push(SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal,
                    role: if ordinal == 0 {
                        SourceAtomicEdgeRole::BuiltinLeftOperand
                    } else {
                        SourceAtomicEdgeRole::BuiltinRightOperand
                    },
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                        first_term + ordinal,
                    )),
                });
                requests.push(SourceAtomicRequestInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal,
                    kind: SourceAtomicRequestKind::OperandExpectedType,
                    edge: Some(edge),
                    candidate: None,
                    type_site: None,
                    attribute: None,
                });
            }
        }
        SourceAtomicFormulaHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            formulas: [(51, 103, 108, 0), (70, 159, 164, 1)]
                .into_iter()
                .enumerate()
                .map(
                    |(source_ordinal, (site, start, end, context))| SourceAtomicFormulaInput {
                        site: node(site),
                        source_range: range(fixture.source, start, end),
                        source_ordinal,
                        context: BindingContextId::new(context),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "x = x".to_owned(),
                        kind: SourceAtomicFormulaKind::Equality,
                    },
                )
                .collect(),
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges,
            requests,
        }
    };
    let reject_atomic = |input: SourceAtomicFormulaHandoffInput, label: &str| {
        assert!(
            SourceAtomicFormulaProducer::build(
                input,
                &fixture.bindings,
                &fixture.symbols,
                &fixture.primary,
                None,
                Some(&fixture.structure),
                None,
                &fixture.arena,
            )
            .is_err(),
            "Task256 {label}"
        );
    };
    let mut invalid_formula = atomic_input();
    invalid_formula.formulas[0].source_range.end -= 1;
    reject_atomic(invalid_formula, "formula corruption");
    let mut invalid_atomic_edge = atomic_input();
    invalid_atomic_edge.edges[0].role = SourceAtomicEdgeRole::BuiltinRightOperand;
    reject_atomic(invalid_atomic_edge, "edge corruption");
    let mut invalid_atomic_request = atomic_input();
    invalid_atomic_request.requests[0].edge = None;
    reject_atomic(invalid_atomic_request, "request corruption");
    let mut excluded_structure_edge = atomic_input();
    excluded_structure_edge.edges[2].target =
        SourceAtomicTermTarget::Structure(SourceStructureTermId::new(0));
    reject_atomic(
        excluded_structure_edge,
        "excludes the selector witness subtree",
    );

    let mut missing_witness = fixture.witness_input();
    missing_witness.witnesses.clear();
    assert_ne!(missing_witness, fixture.witness_input());
    let publish_with_missing_witness = |input| {
        SourceStatementProducer::build(
            input,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        )
        .map(|statement| fixture.witnesses(&statement, missing_witness.clone()))
    };
    let mut aggregate_before_owner = fixture.statement_input();
    aggregate_before_owner.owners[0].spelling.push('x');
    aggregate_before_owner.statements.clear();
    assert_eq!(
        publish_with_missing_witness(aggregate_before_owner),
        Err(SourceStatementError::InvalidAggregate)
    );
    let mut owner_before_statement = fixture.statement_input();
    owner_before_statement.owners[0].spelling.push('x');
    owner_before_statement.statements[0].spelling.push('x');
    assert_eq!(
        publish_with_missing_witness(owner_before_statement),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        })
    );
    let mut owner_before_witness = fixture.statement_input();
    owner_before_witness.owners[0].spelling.push('!');
    assert_eq!(
        publish_with_missing_witness(owner_before_witness),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        })
    );
    assert_clean_replay();

    type BaseMutation = (&'static str, fn(&mut SourceStatementHandoffInput));
    let base_mutations: [BaseMutation; 27] = [
        ("owners clear", |input| input.owners.clear()),
        ("candidate ordinal", |input| {
            input.candidate_facts[1].ordinal = 1
        }),
        ("owner site", |input| input.owners[0].site = node(74)),
        ("owner range", |input| input.owners[0].source_range.end -= 1),
        ("owner spelling", |input| input.owners[0].spelling.push('x')),
        ("owner recovery", |input| {
            input.owners[0].recovery = SourceStatementRecovery::Degraded
        }),
        ("statement owner", |input| {
            input.statements[0].owner = SourceTheoremOwnerId::new(1)
        }),
        ("statement context", |input| {
            input.statements[0].context = SourceStatementContextId::new(1)
        }),
        ("statement formula", |input| {
            input.statements[0].formula =
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(1))
        }),
        ("statement site", |input| {
            input.statements[0].site = node(74)
        }),
        ("statement range", |input| {
            input.statements[1].source_range.start += 1
        }),
        ("statement ordinal", |input| {
            input.statements[0].source_ordinal = 1
        }),
        ("statement spelling", |input| {
            input.statements[0].spelling.push('x')
        }),
        ("statement kind", |input| {
            input.statements[1].kind = SourceStatementKind::Assumption
        }),
        ("statement recovery", |input| {
            input.statements[0].recovery = SourceStatementRecovery::Degraded
        }),
        ("context statement", |input| {
            input.contexts[0].statement = SourceStatementId::new(1)
        }),
        ("context binding", |input| {
            input.contexts[1].binding_context = BindingContextId::new(0)
        }),
        ("context range", |input| {
            input.contexts[1].source_range.start += 1
        }),
        ("context visible", |input| {
            input.contexts[1].visible_bindings.clear()
        }),
        ("input statement", |input| {
            input.input_facts[0].statement = SourceStatementId::new(1)
        }),
        ("input context", |input| {
            input.input_facts[0].context = SourceStatementContextId::new(1)
        }),
        ("input ordinal", |input| input.input_facts[0].ordinal = 1),
        ("input binding", |input| {
            input.input_facts[0].binding = BindingId::new(1)
        }),
        ("input uses", |input| input.input_facts[1].uses.swap(0, 1)),
        ("candidate statement", |input| {
            input.candidate_facts[0].statement = SourceStatementId::new(1)
        }),
        ("candidate context", |input| {
            input.candidate_facts[0].context = SourceStatementContextId::new(1)
        }),
        ("candidate formula", |input| {
            input.candidate_facts[1].formula =
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
        }),
    ];
    for (label, mutate) in base_mutations {
        let mut input = fixture.statement_input();
        mutate(&mut input);
        assert_ne!(input, fixture.statement_input(), "base {label} mutation");
        let result = SourceStatementProducer::build(
            input,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        );
        let expected = match label {
            "owners clear" => SourceStatementError::InvalidAggregate,
            "candidate ordinal" => SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new(1),
            },
            "owner spelling" => SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
            "statement ordinal" => SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0),
            },
            "context range" => SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(1),
            },
            "input uses" => SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new(1),
            },
            _ => {
                assert!(result.is_err(), "Task258 base {label}");
                assert_clean_replay();
                continue;
            }
        };
        assert_eq!(result, Err(expected), "Task258 base {label}");
        assert_clean_replay();
    }

    let witness_mutations: [fn(&mut SourceStatementWitnessInput); 17] = [
        |row| row.take_site = node(73),
        |row| row.site = node(73),
        |row| row.owner = SourceTheoremOwnerId::new(1),
        |row| row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
        |row| row.term = SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(1)),
        |row| {
            row.term =
                SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(0))
        },
        |row| row.binding_context = BindingContextId::new(0),
        |row| row.site = node(63),
        |row| row.take_site = node(64),
        |row| row.take_range.start += 1,
        |row| row.source_range.end -= 1,
        |row| row.source_ordinal = 0,
        |row| row.ordinal = 1,
        |row| row.spelling.push('x'),
        |row| row.kind = SourceStatementWitnessKind::Named,
        |row| row.recovery = SourceStatementRecovery::Degraded,
        |row| row.name = Some(SourceStatementWitnessNameId::new(0)),
    ];
    for (mutation, mutate) in witness_mutations.into_iter().enumerate() {
        let mut input = fixture.witness_input();
        mutate(&mut input.witnesses[0]);
        assert_ne!(
            input,
            fixture.witness_input(),
            "witness mutation {mutation}"
        );
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            }),
            "witness mutation {mutation} must fail"
        );
        assert_clean_replay();
    }
    assert_eq!(
        fixture.witnesses(&statement, missing_witness),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_clean_replay();
    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_218);
    dependency_before_aggregate.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_before_aggregate),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut aggregate_before_row = fixture.witness_input();
    aggregate_before_row.witnesses[0].site = node(63);
    aggregate_before_row
        .witnesses
        .push(aggregate_before_row.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, aggregate_before_row),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );

    let fingerprint_mutations: [StatementWitnessMutation; 4] = [
        ("statement fingerprint", |row| {
            row.statement_fingerprint.push('x')
        }),
        ("primary fingerprint", |row| {
            row.primary_term_fingerprint.push('x')
        }),
        ("structure fingerprint", |row| {
            row.structure_fingerprint = Some("stale".to_owned())
        }),
        ("application/structure hybrid", |row| {
            row.application_fingerprint = Some("hybrid".to_owned())
        }),
    ];
    for (label, mutate) in fingerprint_mutations {
        let mut stale = baseline.clone();
        mutate(&mut stale);
        stale.witnesses.rows.clear();
        assert_eq!(
            stale.validate_installation_with_structure(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                Some(&fixture.structure),
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "{label}"
        );
    }

    assert_eq!(
        SourceStatementWitnessProducer::build(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_clean_replay();
    assert_ne!(foreign.structure, fixture.structure);
    assert_eq!(
        SourceStatementWitnessProducer::build_with_structure(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &foreign.structure,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_clean_replay();

    for (label, local, fqn, path, export, pad_contribution) in [
        (
            "symbol identity",
            "summary:parser.type_fixtures#parse-only#TypeCaseStruct:6",
            "parser.type_fixtures::TypeCaseStruct#6",
            vec![5],
            ExportStatus::Exported,
            false,
        ),
        (
            "structural path",
            "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5",
            "parser.type_fixtures::TypeCaseStruct#5",
            vec![6],
            ExportStatus::Exported,
            false,
        ),
        (
            "export status",
            "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5",
            "parser.type_fixtures::TypeCaseStruct#5",
            vec![5],
            ExportStatus::ReExported,
            false,
        ),
        (
            "contribution",
            "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5",
            "parser.type_fixtures::TypeCaseStruct#5",
            vec![5],
            ExportStatus::Exported,
            true,
        ),
    ] {
        let (_, _, symbol, contribution, symbols) =
            structure_witness_symbol_env_with_structure_identity(
                fixture.source,
                &fixture.module,
                "FormulaStatementStructureSelectorWitnessSmoke",
                170,
                "statement.fixture.theorem.b3m2b2b2b",
                local,
                fqn,
                path,
                export,
                pad_contribution,
            );
        let structure = SourceStructureProducer::build(
            B3M2B2B2BFixture::structure_input(
                fixture.source,
                &fixture.module,
                symbol,
                contribution,
            ),
            &symbols,
            &fixture.bindings,
            &fixture.primary,
            None,
            &fixture.arena,
        )
        .unwrap_or_else(|error| panic!("{label} lower handoff failed: {error}"));
        assert_ne!(structure, fixture.structure, "{label} lower mutation");
        assert_eq!(
            SourceStatementWitnessProducer::build_with_structure(
                fixture.witness_input(),
                &statement,
                &fixture.primary,
                &structure,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "{label}"
        );
    }

    let structure_mutations: [StructureHandoffMutation; 2] = [
        ("term spelling", |input| input.terms[0].spelling.push('x')),
        ("member spelling", |input| {
            input.members[1].spelling = "z".to_owned()
        }),
    ];
    for (label, mutate) in structure_mutations {
        let mut input = B3M2B2B2BFixture::structure_input(
            fixture.source,
            &fixture.module,
            fixture.structure_symbol.clone(),
            fixture.structure_contribution,
        );
        mutate(&mut input);
        let structure = SourceStructureProducer::build(
            input,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            None,
            &fixture.arena,
        )
        .unwrap_or_else(|error| panic!("{label} lower handoff failed: {error}"));
        assert_ne!(structure, fixture.structure, "{label} lower mutation");
        assert_eq!(
            SourceStatementWitnessProducer::build_with_structure(
                fixture.witness_input(),
                &statement,
                &fixture.primary,
                &structure,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "{label}"
        );
    }
    let mut invalid_edge = B3M2B2B2BFixture::structure_input(
        fixture.source,
        &fixture.module,
        fixture.structure_symbol.clone(),
        fixture.structure_contribution,
    );
    invalid_edge.edges[1].target = SourceStructureTarget::Primary(SourcePrimaryTermId::new(3));
    assert!(
        SourceStructureProducer::build(
            invalid_edge,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            None,
            &fixture.arena,
        )
        .is_err(),
        "Task254 rejects an incoherent edge before Task258 witness publication"
    );

    let mut resolved_builder = ResolvedArenaBuilder::new();
    let foreign_resolved_node = resolved_builder
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::Root,
            Vec::new(),
            SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 0, 0)),
                vec![999],
            ),
        ))
        .expect("foreign resolved-node id");
    for index in 0..79 {
        let kind_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.kind = "source.task258b3m2b2b2b.mutated".into();
            }
        });
        let resolved_node_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.resolved_node = Some(foreign_resolved_node);
            }
        });
        let anchor_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
            }
        });
        let children_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                if row.children.is_empty() {
                    row.children.push(TypedNodeId::new(usize::from(index == 0)));
                } else {
                    row.children.pop();
                }
            }
        });
        let typing_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.typing = TypingState::Successful;
            }
        });
        let recovery_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.recovery = NodeRecoveryState::Degraded;
            }
        });
        let context_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.links.context = Some(LocalTypeContextId::new(0));
            }
        });
        let type_entry_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.links.type_entry = Some(TypeEntryId::new(0));
            }
        });
        let facts_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.links.facts.push(TypeFactId::new(0));
            }
        });
        let coercions_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.links.coercions.push(CoercionId::new(0));
            }
        });
        let obligations_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.links
                    .initial_obligations
                    .push(InitialObligationId::new(0));
            }
        });
        let diagnostics_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.links.diagnostics.push(TypeDiagnosticId::new(0));
            }
        });
        for (field, mutated) in [
            ("kind", kind_mutated),
            ("resolved_node", resolved_node_mutated),
            ("anchor", anchor_mutated),
            ("children", children_mutated),
            ("typing", typing_mutated),
            ("recovery", recovery_mutated),
            ("links.context", context_mutated),
            ("links.type_entry", type_entry_mutated),
            ("links.facts", facts_mutated),
            ("links.coercions", coercions_mutated),
            ("links.initial_obligations", obligations_mutated),
            ("links.diagnostics", diagnostics_mutated),
        ] {
            assert_ne!(mutated, fixture.arena, "node {index} {field} mutation");
            assert_eq!(
                SourceStatementWitnessProducer::build_with_structure(
                    fixture.witness_input(),
                    &statement,
                    &fixture.primary,
                    &fixture.structure,
                    &mutated,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {field} producer"
            );
            assert_eq!(
                baseline.validate_installation_with_structure(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    Some(&fixture.structure),
                    &mutated,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {field} installation"
            );
            assert_clean_replay();
        }
    }
    let wrong_root = TypedArena::try_new(
        Some(TypedNodeId::new(77)),
        fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
    )
    .expect("wrong-root arena remains structurally valid");
    assert_ne!(wrong_root, fixture.arena);
    assert_eq!(
        SourceStatementWitnessProducer::build_with_structure(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &fixture.structure,
            &wrong_root,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(
        baseline.validate_installation_with_structure(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.structure),
            &wrong_root,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_clean_replay();
}

#[test]
fn task258b3m2b2b2b_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B2BFixture::new(219);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let base = fixture.empty_typed();
    let base_debug = base.debug_text();
    let typed = base
        .clone()
        .with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("combined typed");
    let typed_debug = typed.debug_text();
    assert_eq!(typed.source_structure(), Some(&fixture.structure));
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert!(typed.source_application().is_none());
    assert!(typed.source_statement_references().is_none());
    assert_eq!(
        typed.clone().with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let structure_only = base
        .clone()
        .with_source_structure(fixture.structure.clone())
        .expect("structure-only lower handoff");
    assert_eq!(
        structure_only.with_source_statement_witnesses(statement.clone(), witnesses.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let constructor = B3M2B2B2AFixture::new(219);
    let constructor_statement = constructor.statement();
    let constructor_witnesses = constructor
        .witnesses(&constructor_statement, constructor.witness_input())
        .expect("constructor witnesses");
    let constructor_typed = constructor
        .empty_typed()
        .with_source_structure_statement_witnesses(
            constructor.structure.clone(),
            constructor_statement.clone(),
            constructor_witnesses.clone(),
        )
        .expect("constructor typed");
    for (label, structure, hybrid_statement, hybrid_witnesses) in [
        (
            "B2A structure with B2B upper",
            constructor.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        (
            "B2B structure with B2A upper",
            fixture.structure.clone(),
            constructor_statement.clone(),
            constructor_witnesses.clone(),
        ),
        (
            "B2B structure/statement with B2A witness",
            fixture.structure.clone(),
            statement.clone(),
            constructor_witnesses.clone(),
        ),
        (
            "B2A structure/statement with B2B witness",
            constructor.structure.clone(),
            constructor_statement.clone(),
            witnesses.clone(),
        ),
    ] {
        assert_eq!(
            base.clone().with_source_structure_statement_witnesses(
                structure,
                hybrid_statement,
                hybrid_witnesses,
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "{label}"
        );
    }
    assert_eq!(
        typed.clone().with_source_structure_statement_witnesses(
            constructor.structure.clone(),
            constructor_statement.clone(),
            constructor_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        constructor_typed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        constructor
            .empty_typed()
            .with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    macro_rules! reject_plain_witness_family {
        ($label:literal, $family:expr) => {{
            let family = $family;
            let family_statement = family.statement();
            let family_witnesses = family
                .witnesses(&family_statement, family.witness_input())
                .expect(concat!($label, " witnesses"));
            let family_base = family.empty_typed();
            let family_installed = family_base
                .clone()
                .with_source_statement_witnesses(family_statement.clone(), family_witnesses.clone())
                .expect(concat!($label, " installed"));
            assert_eq!(
                base.clone().with_source_statement_witnesses(
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("B2B base then ", $label)
            );
            assert_eq!(
                typed
                    .clone()
                    .with_source_statement_witnesses(family_statement, family_witnesses,),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B2B then ", $label)
            );
            assert_eq!(
                family_base.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!($label, " base then B2B")
            );
            assert_eq!(
                family_installed.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed ", $label, " then B2B")
            );
        }};
    }
    reject_plain_witness_family!("B3", B3Fixture::new(219));
    reject_plain_witness_family!("B3N", B3NFixture::new(219));
    reject_plain_witness_family!("B3M1", B3M1Fixture::new(219));
    reject_plain_witness_family!("B3M2A", B3M2AFixture::new(219));
    reject_plain_witness_family!("B3M2B1", B3M2B1Fixture::new(219));
    reject_plain_witness_family!("B3M2B2A", B3M2B2AFixture::new(219));

    macro_rules! reject_application_witness_family {
        ($label:literal, $family:expr) => {{
            let family = $family;
            let family_statement = family.statement();
            let family_witnesses = family
                .witnesses(&family_statement, family.witness_input())
                .expect(concat!($label, " witnesses"));
            let family_base = family.empty_typed();
            let family_installed = family_base
                .clone()
                .with_source_application_statement_witnesses(
                    family.application.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                )
                .expect(concat!($label, " installed"));
            assert_eq!(
                base.clone().with_source_application_statement_witnesses(
                    family.application.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("B2B base then ", $label)
            );
            assert_eq!(
                typed.clone().with_source_application_statement_witnesses(
                    family.application.clone(),
                    family_statement,
                    family_witnesses,
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B2B then ", $label)
            );
            assert_eq!(
                family_base.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!($label, " base then B2B")
            );
            assert_eq!(
                family_installed.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed ", $label, " then B2B")
            );
        }};
    }
    reject_application_witness_family!("B1A", B3M2B2B1AFixture::new(219));
    reject_application_witness_family!("B1B1", B3M2B2B1B1Fixture::new(219));

    let b1 = B1Fixture::new(219);
    let references = b1.references(b1.reference_input()).expect("references");
    let reference_installed = b1
        .empty_typed()
        .with_source_statement_references(b1.statement.clone(), references.clone())
        .expect("reference installed");
    assert_eq!(
        base.clone()
            .with_source_statement_references(b1.statement.clone(), references.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_statement_references(b1.statement.clone(), references),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b1.empty_typed().with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        reference_installed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b2 = B2Fixture::new(219);
    let b2_statement = b2.build(b2.input()).expect("B2 statement");
    let b2_base = b2.empty_typed();
    let b2_installed = b2_base
        .clone()
        .with_source_statement(b2_statement.clone())
        .expect("B2 installed");
    assert_eq!(
        base.clone().with_source_statement(b2_statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.clone().with_source_statement(b2_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b2_base.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b2_installed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let task258a = Fixture::new(219);
    let task258a_statement = task258a
        .build(task258a.input())
        .expect("Task258A statement");
    let task258a_base = TypedAst::try_new(TypedAstParts {
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
    .expect("Task258A base")
    .with_source_term(task258a.primary.clone())
    .expect("Task258A Task252")
    .with_source_atomic_formula(task258a.atomic.clone())
    .expect("Task258A Task256");
    let task258a_installed = task258a_base
        .clone()
        .with_source_statement(task258a_statement.clone())
        .expect("Task258A installed");
    assert_eq!(
        base.clone()
            .with_source_statement(task258a_statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_statement(task258a_statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        task258a_base.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        task258a_installed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

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
    .expect("Task255 empty handoff");
    assert_eq!(
        base.clone()
            .with_source_set_term(set_terms.clone())
            .expect("Task255 first")
            .with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.clone().with_source_set_term(set_terms),
        Err(TypedAstError::InvalidSourceSetTerm)
    );

    for (family, occupied) in [
        (
            "Task257A",
            crate::source_composite_formula::tests::task_257a_installed_typed_ast(),
        ),
        (
            "Task257B",
            crate::source_formula_composition::tests::task_257b_installed_typed_ast(),
        ),
        (
            "Task257C2",
            crate::source_formula_composition::tests::task_257c2_installed_typed_ast(),
        ),
        (
            "Task257C3",
            crate::source_formula_composition::tests::task_257c3_installed_typed_ast(),
        ),
    ] {
        let occupied_debug = occupied.debug_text();
        assert_eq!(
            occupied.with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "{family}"
        );
        assert!(!occupied_debug.is_empty());
    }
    assert_eq!(
        typed.clone().with_source_composite_formula(
            crate::source_composite_formula::tests::task_257a_installed_typed_ast()
                .source_composite_formula()
                .expect("Task257A composite")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceCompositeFormula)
    );
    assert_eq!(base.debug_text(), base_debug);
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        base.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement,
            witnesses,
        )
        .expect("atomic replay")
        .debug_text(),
        typed_debug
    );
}

#[test]
fn task258b3m2b2b2b_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B2BFixture::new(220);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("typed");
    let typed_debug = typed.debug_text();
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
    let resolved_debug = resolved.debug_text();
    assert_eq!(resolved.source_structure(), Some(&fixture.structure));
    assert_eq!(resolved.source_statement(), Some(&statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
    assert!(resolved.source_application().is_none());
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
    assert_eq!(typed.clone().debug_text(), typed_debug);
    assert_eq!(resolved.clone().debug_text(), resolved_debug);

    let fingerprint_mutations: [StatementWitnessMutation; 4] = [
        ("statement fingerprint", |row| {
            row.statement_fingerprint.push('x')
        }),
        ("primary fingerprint", |row| {
            row.primary_term_fingerprint.push('x')
        }),
        ("structure fingerprint", |row| {
            row.structure_fingerprint = Some("stale".to_owned())
        }),
        ("application hybrid", |row| {
            row.application_fingerprint = Some("hybrid".to_owned())
        }),
    ];
    for (label, mutate) in fingerprint_mutations {
        let mut stale = witnesses.clone();
        mutate(&mut stale);
        let mut stale_typed = typed.clone();
        stale_typed.inject_source_statement_witness_bundle_for_test(statement.clone(), stale);
        assert_eq!(
            assemble_empty_resolved(&stale_typed),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{label}"
        );
    }

    let mut orphan = fixture.empty_typed();
    orphan.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&orphan),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut without_structure = fixture.empty_typed();
    without_structure
        .inject_source_statement_witness_bundle_for_test(statement.clone(), witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&without_structure),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut without_witness = fixture
        .empty_typed()
        .with_source_structure(fixture.structure.clone())
        .expect("structure");
    without_witness.inject_source_statement_for_test(statement.clone());
    assert_eq!(
        assemble_empty_resolved(&without_witness),
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

    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b2b-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 122, 150),
    });
    assert_eq!(
        assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
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
                expr: ExprId::new("task258b3m2b2b2b-semantic-coexistence"),
                typed_site: node(62),
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
