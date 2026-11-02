use super::*;

#[test]
fn task258b3m2b2b2a_exact_structure_witness_api_debug_and_legacy_compatibility_are_stable() {
    let fixture = B3M2B2B2AFixture::new(207);
    assert_eq!(TASK258B3M2B2B2A_SOURCE.len(), 172);
    assert_eq!(
        TASK258B3M2B2B2A_SOURCE,
        "import parser.type_fixtures;\nreserve x for set;\ntheorem FormulaStatementStructureConstructorWitnessSmoke: x = x proof\n  take TypeCaseStruct(x: 1, y: 2);\n  thus x = x;\nend;\n"
    );
    assert_eq!(fixture.arena.len(), 76);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(75)));
    for (index, (start, end)) in TASK258B3M2B2B2A_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b2a_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b2a_node_children(index)
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
        (1, 0, 1, 2, 0, 2, 6)
    );
    let root = fixture
        .structure
        .roots()
        .get(SourceStructureRootId::new(0))
        .expect("imported root");
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
    assert_eq!(fixture.atomic.formulas().len(), 2);
    assert!(fixture.atomic.wrappers().is_empty());
    assert!(fixture.atomic.predicate_segments().is_empty());
    assert!(fixture.atomic.predicate_heads().is_empty());
    assert!(fixture.atomic.candidates().is_empty());
    assert!(fixture.atomic.type_sites().is_empty());
    assert!(fixture.atomic.attributes().is_empty());
    assert_eq!(fixture.atomic.edges().len(), 4);
    assert_eq!(fixture.atomic.requests().len(), 4);
    assert_eq!(fixture.atomic.application_fingerprint(), None);
    assert_eq!(fixture.atomic.structure_fingerprint(), None);

    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b2a_profile());
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
        .expect("structure witness");
    assert_eq!(
        (witnesses.witnesses().len(), witnesses.names().len()),
        (1, 0)
    );
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(
        witnesses.structure_fingerprint(),
        Some(fixture.structure.debug_text().as_str())
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
    assert_eq!(
        SourceStatementWitnessProducer::build(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );

    let legacy = B3M2B2AFixture::new(208);
    let legacy_statement = legacy.statement();
    let legacy_witnesses = legacy
        .witnesses(&legacy_statement, legacy.witness_input())
        .expect("legacy witness");
    assert_eq!(legacy_witnesses.application_fingerprint(), None);
    assert_eq!(legacy_witnesses.structure_fingerprint(), None);
    assert!(
        !legacy_witnesses
            .debug_text()
            .lines()
            .any(|line| line.starts_with("structure-fingerprint:"))
    );
    assert!(legacy_witnesses.debug_text().contains(" term=primary#2 "));

    let application = B3M2B2B1AFixture::new(209);
    let application_statement = application.statement();
    let application_witnesses = application
        .witnesses(&application_statement, application.witness_input())
        .expect("application witness");
    assert!(application_witnesses.application_fingerprint().is_some());
    assert_eq!(application_witnesses.structure_fingerprint(), None);
    assert!(
        !application_witnesses
            .debug_text()
            .lines()
            .any(|line| line.starts_with("structure-fingerprint:"))
    );
    assert!(
        application_witnesses
            .debug_text()
            .contains(" term=application#0 ")
    );
    assert_eq!(
        SourceStatementWitnessProducer::build_with_structure(
            legacy.witness_input(),
            &legacy_statement,
            &legacy.primary,
            &fixture.structure,
            &legacy.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(witnesses.clone().debug_text(), debug);
}

#[test]
fn task258b3m2b2b2a_dependencies_structure_witness_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B2AFixture::new(210);
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
    let statement_debug = statement.debug_text();
    let witness_debug = baseline.debug_text();
    let assert_clean_replay = || {
        let replay_statement = fixture.statement();
        assert_eq!(replay_statement.debug_text(), statement_debug);
        assert_eq!(
            fixture
                .witnesses(&replay_statement, fixture.witness_input())
                .expect("clean replay")
                .debug_text(),
            witness_debug
        );
    };

    let foreign_binding = b3m2b2b1b1_binding_env(fixture.source, &fixture.module);
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
        "Task48 dependency precedes aggregate validation"
    );

    let mut substituted_primary_input =
        B3M2B2B2AFixture::primary_input(fixture.source, &fixture.module);
    substituted_primary_input.terms[2].spelling = "3".to_owned();
    substituted_primary_input.numeric_type_requests[0].spelling = "3".to_owned();
    let substituted_primary = SourcePrimaryTermProducer::build(
        substituted_primary_input,
        &fixture.bindings,
        &fixture.arena,
    )
    .expect("coherent Task252 term/request substitution");
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
        "Task252 fingerprint precedes aggregate validation"
    );
    let mut invalid_reference = B3M2B2B2AFixture::primary_input(fixture.source, &fixture.module);
    invalid_reference.references[2].role = SourcePrimaryTermReferenceRole::LocalConstant;
    assert!(
        SourcePrimaryTermProducer::build(invalid_reference, &fixture.bindings, &fixture.arena,)
            .is_err(),
        "Task252 rejects reference corruption"
    );
    let mut invalid_numeric_request =
        B3M2B2B2AFixture::primary_input(fixture.source, &fixture.module);
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
            formulas: [(49, 106, 111, 0), (67, 160, 165, 1)]
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
        "excludes the constructor witness subtree",
    );

    let mut aggregate_before_owner = fixture.statement_input();
    aggregate_before_owner.owners[0].spelling.push('x');
    aggregate_before_owner.statements.clear();
    assert_eq!(
        SourceStatementProducer::build(
            aggregate_before_owner,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidAggregate)
    );
    let mut owner_before_statement = fixture.statement_input();
    owner_before_statement.owners[0].spelling.push('x');
    owner_before_statement.statements[0].spelling.push('x');
    assert!(matches!(
        SourceStatementProducer::build(
            owner_before_statement,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner { .. })
    ));
    type BaseMutation = (&'static str, fn(&mut SourceStatementHandoffInput));
    let base_mutations: [BaseMutation; 5] = [
        ("owner", |input| input.owners[0].source_range.end -= 1),
        ("statement", |input| {
            input.statements[1].source_range.start += 1
        }),
        ("context", |input| {
            input.contexts[1].binding_context = BindingContextId::new(0)
        }),
        ("input", |input| input.input_facts[1].uses.swap(0, 1)),
        ("candidate", |input| {
            input.candidate_facts[1].formula =
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
        }),
    ];
    for (label, mutate) in base_mutations {
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
            "Task258 base {label} corruption"
        );
    }
    assert_clean_replay();

    let witness_mutations: [fn(&mut SourceStatementWitnessInput); 6] = [
        |row: &mut SourceStatementWitnessInput| {
            row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        },
        |row: &mut SourceStatementWitnessInput| {
            row.term = SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(1))
        },
        |row: &mut SourceStatementWitnessInput| {
            row.term =
                SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(0))
        },
        |row: &mut SourceStatementWitnessInput| row.binding_context = BindingContextId::new(0),
        |row: &mut SourceStatementWitnessInput| row.site = node(60),
        |row: &mut SourceStatementWitnessInput| row.take_site = node(61),
    ];
    for mutate in witness_mutations {
        let mut input = fixture.witness_input();
        mutate(&mut input.witnesses[0]);
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            })
        );
    }
    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_210);
    dependency_before_aggregate.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_before_aggregate),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut aggregate_before_row = fixture.witness_input();
    aggregate_before_row.witnesses[0].site = node(60);
    aggregate_before_row
        .witnesses
        .push(aggregate_before_row.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, aggregate_before_row),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );

    let fingerprint_mutations: [StatementWitnessMutation; 4] = [
        (
            "statement fingerprint",
            |row: &mut SourceStatementWitnessHandoff| row.statement_fingerprint.push('x'),
        ),
        (
            "primary fingerprint",
            |row: &mut SourceStatementWitnessHandoff| row.primary_term_fingerprint.push('x'),
        ),
        (
            "structure fingerprint",
            |row: &mut SourceStatementWitnessHandoff| {
                row.structure_fingerprint = Some("stale".to_owned())
            },
        ),
        (
            "application/structure hybrid",
            |row: &mut SourceStatementWitnessHandoff| {
                row.application_fingerprint = Some("hybrid".to_owned())
            },
        ),
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

    let foreign = B3M2B2B2AFixture::new(211);
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
        let (_, _, symbol, contribution, symbols) = b3m2b2b2a_symbol_env_with_structure_identity(
            fixture.source,
            &fixture.module,
            local,
            fqn,
            path,
            export,
            pad_contribution,
        );
        let structure = SourceStructureProducer::build(
            B3M2B2B2AFixture::structure_input(
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
        (
            "term spelling",
            |input: &mut SourceStructureHandoffInput| {
                input.terms[0].spelling = "TypeCaseStruct ( x : 1 , y : 3 )".to_owned()
            },
        ),
        (
            "member spelling",
            |input: &mut SourceStructureHandoffInput| input.members[1].spelling = "z".to_owned(),
        ),
    ];
    for (label, mutate) in structure_mutations {
        let mut input = B3M2B2B2AFixture::structure_input(
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
    let mut invalid_edge = B3M2B2B2AFixture::structure_input(
        fixture.source,
        &fixture.module,
        fixture.structure_symbol.clone(),
        fixture.structure_contribution,
    );
    invalid_edge.edges[1].target = SourceStructureTarget::Primary(SourcePrimaryTermId::new(2));
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
    for index in 0..76 {
        let kind_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.kind = "source.task258b3m2b2b2a.mutated".into();
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
            assert!(
                baseline
                    .validate_installation_with_structure(
                        fixture.source,
                        &fixture.module,
                        &statement,
                        &fixture.primary,
                        Some(&fixture.structure),
                        &mutated,
                    )
                    .is_err(),
                "node {index} {field}"
            );
        }
    }
    let wrong_root = TypedArena::try_new(
        Some(TypedNodeId::new(74)),
        fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
    )
    .expect("wrong-root arena remains structurally valid");
    assert!(
        baseline
            .validate_installation_with_structure(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                Some(&fixture.structure),
                &wrong_root,
            )
            .is_err()
    );
    assert_eq!(
        fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("clean replay")
            .debug_text(),
        baseline.debug_text()
    );
}

#[test]
fn task258b3m2b2b2a_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B2AFixture::new(212);
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
        .expect("combined install");
    let typed_debug = typed.debug_text();
    assert_eq!(typed.source_structure(), Some(&fixture.structure));
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert!(typed.source_application().is_none());
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_structure(fixture.structure.clone())
            .expect("structure first")
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_structure(fixture.structure.clone())
            .expect("structure first")
            .with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_structure(fixture.structure.clone()),
        Err(TypedAstError::InvalidSourceStructure)
    );
    assert_eq!(
        typed.clone().with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let mut hybrid = witnesses.clone();
    hybrid.application_fingerprint = Some("hybrid".to_owned());
    assert_eq!(
        base.clone().with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            hybrid,
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let application = B3M2B2B1AFixture::new(212);
    let application_statement = application.statement();
    let application_witnesses = application
        .witnesses(&application_statement, application.witness_input())
        .expect("application witnesses");
    assert_eq!(
        base.clone().with_source_application_statement_witnesses(
            application.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        application
            .empty_typed()
            .with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                application_statement,
                application_witnesses,
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let mut legacy_families = Vec::new();
    let b3 = B3Fixture::new(212);
    let b3_statement = b3.statement();
    let b3_witnesses = b3
        .witnesses(&b3_statement, b3.witness_input())
        .expect("B3 witnesses");
    let b3_installed = b3
        .empty_typed()
        .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
        .expect("B3 installed");
    legacy_families.push((
        "B3",
        b3.empty_typed(),
        b3_statement,
        b3_witnesses,
        b3_installed,
    ));

    let b3n = B3NFixture::new(212);
    let b3n_statement = b3n.statement();
    let b3n_witnesses = b3n
        .witnesses(&b3n_statement, b3n.witness_input())
        .expect("B3N witnesses");
    let b3n_installed = b3n
        .empty_typed()
        .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone())
        .expect("B3N installed");
    legacy_families.push((
        "B3N",
        b3n.empty_typed(),
        b3n_statement,
        b3n_witnesses,
        b3n_installed,
    ));

    let b3m1 = B3M1Fixture::new(212);
    let b3m1_statement = b3m1.statement();
    let b3m1_witnesses = b3m1
        .witnesses(&b3m1_statement, b3m1.witness_input())
        .expect("B3M1 witnesses");
    let b3m1_installed = b3m1
        .empty_typed()
        .with_source_statement_witnesses(b3m1_statement.clone(), b3m1_witnesses.clone())
        .expect("B3M1 installed");
    legacy_families.push((
        "B3M1",
        b3m1.empty_typed(),
        b3m1_statement,
        b3m1_witnesses,
        b3m1_installed,
    ));

    let b3m2a = B3M2AFixture::new(212);
    let b3m2a_statement = b3m2a.statement();
    let b3m2a_witnesses = b3m2a
        .witnesses(&b3m2a_statement, b3m2a.witness_input())
        .expect("B3M2A witnesses");
    let b3m2a_installed = b3m2a
        .empty_typed()
        .with_source_statement_witnesses(b3m2a_statement.clone(), b3m2a_witnesses.clone())
        .expect("B3M2A installed");
    legacy_families.push((
        "B3M2A",
        b3m2a.empty_typed(),
        b3m2a_statement,
        b3m2a_witnesses,
        b3m2a_installed,
    ));

    let b3m2b1 = B3M2B1Fixture::new(212);
    let b3m2b1_statement = b3m2b1.statement();
    let b3m2b1_witnesses = b3m2b1
        .witnesses(&b3m2b1_statement, b3m2b1.witness_input())
        .expect("B3M2B1 witnesses");
    let b3m2b1_installed = b3m2b1
        .empty_typed()
        .with_source_statement_witnesses(b3m2b1_statement.clone(), b3m2b1_witnesses.clone())
        .expect("B3M2B1 installed");
    legacy_families.push((
        "B3M2B1",
        b3m2b1.empty_typed(),
        b3m2b1_statement,
        b3m2b1_witnesses,
        b3m2b1_installed,
    ));

    let b3m2b2a = B3M2B2AFixture::new(212);
    let b3m2b2a_statement = b3m2b2a.statement();
    let b3m2b2a_witnesses = b3m2b2a
        .witnesses(&b3m2b2a_statement, b3m2b2a.witness_input())
        .expect("B3M2B2A witnesses");
    let b3m2b2a_installed = b3m2b2a
        .empty_typed()
        .with_source_statement_witnesses(b3m2b2a_statement.clone(), b3m2b2a_witnesses.clone())
        .expect("B3M2B2A installed");
    legacy_families.push((
        "B3M2B2A",
        b3m2b2a.empty_typed(),
        b3m2b2a_statement,
        b3m2b2a_witnesses,
        b3m2b2a_installed,
    ));

    let b1a = B3M2B2B1AFixture::new(212);
    let b1a_statement = b1a.statement();
    let b1a_witnesses = b1a
        .witnesses(&b1a_statement, b1a.witness_input())
        .expect("B1A witnesses");
    let b1a_installed = b1a
        .empty_typed()
        .with_source_application_statement_witnesses(
            b1a.application.clone(),
            b1a_statement.clone(),
            b1a_witnesses.clone(),
        )
        .expect("B1A installed");
    assert_eq!(
        base.clone().with_source_application_statement_witnesses(
            b1a.application.clone(),
            b1a_statement.clone(),
            b1a_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "B2A base then correct B1A triple"
    );
    assert_eq!(
        typed.clone().with_source_application_statement_witnesses(
            b1a.application.clone(),
            b1a_statement.clone(),
            b1a_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B2A then correct B1A triple"
    );
    legacy_families.push((
        "B1A",
        b1a.empty_typed(),
        b1a_statement,
        b1a_witnesses,
        b1a_installed,
    ));

    let b1b1 = B3M2B2B1B1Fixture::new(212);
    let b1b1_statement = b1b1.statement();
    let b1b1_witnesses = b1b1
        .witnesses(&b1b1_statement, b1b1.witness_input())
        .expect("B1B1 witnesses");
    let b1b1_installed = b1b1
        .empty_typed()
        .with_source_application_statement_witnesses(
            b1b1.application.clone(),
            b1b1_statement.clone(),
            b1b1_witnesses.clone(),
        )
        .expect("B1B1 installed");
    assert_eq!(
        base.clone().with_source_application_statement_witnesses(
            b1b1.application.clone(),
            b1b1_statement.clone(),
            b1b1_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "B2A base then correct B1B1 triple"
    );
    assert_eq!(
        typed.clone().with_source_application_statement_witnesses(
            b1b1.application.clone(),
            b1b1_statement.clone(),
            b1b1_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B2A then correct B1B1 triple"
    );
    legacy_families.push((
        "B1B1",
        b1b1.empty_typed(),
        b1b1_statement,
        b1b1_witnesses,
        b1b1_installed,
    ));

    for (label, legacy_base, legacy_statement, legacy_witnesses, legacy_installed) in
        legacy_families
    {
        assert_eq!(
            base.clone().with_source_statement_witnesses(
                legacy_statement.clone(),
                legacy_witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "B2A base then {label}"
        );
        assert_eq!(
            typed
                .clone()
                .with_source_statement_witnesses(legacy_statement, legacy_witnesses,),
            Err(TypedAstError::InvalidSourceStatement),
            "installed B2A then {label}"
        );
        assert_eq!(
            legacy_base.with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "{label} base then B2A"
        );
        assert_eq!(
            legacy_installed.with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "installed {label} then B2A"
        );
    }

    let b1 = B1Fixture::new(212);
    let b1_references = b1.references(b1.reference_input()).expect("B1 references");
    let b1_installed = b1
        .empty_typed()
        .with_source_statement_references(b1.statement.clone(), b1_references.clone())
        .expect("B1 installed");
    assert_eq!(
        base.clone()
            .with_source_statement_references(b1.statement.clone(), b1_references.clone()),
        Err(TypedAstError::InvalidSourceStatement),
        "B2A base then B1"
    );
    assert_eq!(
        typed
            .clone()
            .with_source_statement_references(b1.statement.clone(), b1_references),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B2A then B1"
    );
    assert_eq!(
        b1.empty_typed().with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "B1 base then B2A"
    );
    assert_eq!(
        b1_installed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B1 then B2A"
    );

    let b2 = B2Fixture::new(212);
    let b2_statement = b2.build(b2.input()).expect("B2 statement");
    let b2_installed = b2
        .empty_typed()
        .with_source_statement(b2_statement.clone())
        .expect("B2 installed");
    assert_eq!(
        base.clone().with_source_statement(b2_statement.clone()),
        Err(TypedAstError::InvalidSourceStatement),
        "B2A base then B2"
    );
    assert_eq!(
        typed.clone().with_source_statement(b2_statement),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B2A then B2"
    );
    assert_eq!(
        b2.empty_typed().with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "B2 base then B2A"
    );
    assert_eq!(
        b2_installed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B2 then B2A"
    );

    let task258a = Fixture::new(212);
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
        Err(TypedAstError::InvalidSourceStatement),
        "B2A base then Task258A"
    );
    assert_eq!(
        typed.clone().with_source_statement(task258a_statement),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B2A then Task258A"
    );
    assert_eq!(
        task258a_base.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "Task258A base then B2A"
    );
    assert_eq!(
        task258a_installed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed Task258A then B2A"
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

    let task257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
    let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
    let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
    let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
    for (family, occupied) in [
        ("Task257A", task257a),
        ("Task257B", task257b),
        ("Task257C2", task257c2),
        ("Task257C3", task257c3),
    ] {
        let debug = occupied.debug_text();
        assert_eq!(
            occupied.with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "{family}"
        );
        assert!(!debug.is_empty(), "{family} baseline");
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
fn task258b3m2b2b2a_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B2AFixture::new(213);
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
        (
            "statement fingerprint",
            |row: &mut SourceStatementWitnessHandoff| row.statement_fingerprint.push('x'),
        ),
        (
            "primary fingerprint",
            |row: &mut SourceStatementWitnessHandoff| row.primary_term_fingerprint.push('x'),
        ),
        (
            "structure fingerprint",
            |row: &mut SourceStatementWitnessHandoff| {
                row.structure_fingerprint = Some("stale".to_owned())
            },
        ),
        (
            "application hybrid",
            |row: &mut SourceStatementWitnessHandoff| {
                row.application_fingerprint = Some("hybrid".to_owned())
            },
        ),
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
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b2a-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 125, 151),
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
                expr: ExprId::new("task258b3m2b2b2a-semantic-coexistence"),
                typed_site: node(59),
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
