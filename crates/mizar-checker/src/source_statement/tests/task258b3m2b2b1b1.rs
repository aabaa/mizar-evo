use super::*;

#[test]
fn task258b3m2b2b1b1_exact_wrapped_application_witness_api_debug_and_b1a_compatibility_are_stable()
{
    let fixture = B3M2B2B1B1Fixture::new(202);
    const EXPECTED_RANGES: [(usize, usize); 67] = [
        (0, 6),
        (7, 13),
        (13, 14),
        (14, 27),
        (27, 28),
        (29, 36),
        (37, 38),
        (39, 42),
        (43, 46),
        (46, 47),
        (48, 55),
        (56, 108),
        (108, 109),
        (110, 111),
        (112, 113),
        (114, 115),
        (116, 121),
        (124, 128),
        (129, 130),
        (130, 131),
        (132, 134),
        (135, 136),
        (136, 137),
        (137, 138),
        (141, 145),
        (146, 147),
        (148, 149),
        (150, 151),
        (151, 152),
        (153, 156),
        (156, 157),
        (7, 13),
        (14, 27),
        (7, 27),
        (7, 27),
        (0, 28),
        (43, 46),
        (43, 46),
        (37, 46),
        (29, 47),
        (110, 111),
        (110, 111),
        (114, 115),
        (114, 115),
        (110, 115),
        (110, 115),
        (130, 131),
        (135, 136),
        (130, 136),
        (130, 136),
        (129, 137),
        (129, 137),
        (129, 137),
        (124, 138),
        (146, 147),
        (146, 147),
        (150, 151),
        (150, 151),
        (146, 151),
        (146, 151),
        (146, 151),
        (141, 152),
        (116, 156),
        (48, 157),
        (0, 157),
        (0, 157),
        (0, 157),
    ];
    const EXPECTED_CHILDREN: [&[usize]; 67] = [
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[1],
        &[3],
        &[31, 2, 32],
        &[33],
        &[0, 34, 4],
        &[8],
        &[36],
        &[6, 7, 37],
        &[5, 38, 9],
        &[13],
        &[40],
        &[15],
        &[42],
        &[41, 14, 43],
        &[44],
        &[19],
        &[21],
        &[46, 20, 47],
        &[48],
        &[18, 49, 22],
        &[50],
        &[51],
        &[17, 52, 23],
        &[25],
        &[54],
        &[27],
        &[56],
        &[55, 26, 57],
        &[58],
        &[59],
        &[24, 60, 28],
        &[16, 53, 61, 29],
        &[10, 11, 12, 45, 62, 30],
        &[35, 39, 63],
        &[64],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 65,
        ],
    ];
    assert_eq!(fixture.arena.len(), 67);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(66)));
    for (index, (_, row)) in fixture.arena.iter().enumerate() {
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(
                fixture.source,
                EXPECTED_RANGES[index].0,
                EXPECTED_RANGES[index].1,
            )),
            "independent node {index} range snapshot"
        );
        let expected_kind = match index {
            40 | 42 | 54 | 56 => "source.term.variable-reference",
            46 | 47 => "source.term.numeral",
            20 => "source.term.functor-head.single",
            48 => "source.term.functor-application.symbolic",
            50 => "source.term.functor-application.parenthesized",
            44 | 58 => "source.formula.atomic.equality",
            52 => "source.statement-witness.item",
            53 => "source.statement-witness.take",
            61 => "source.statement.conclusion",
            63 => "source.statement.theorem",
            _ => "source.surface.unowned",
        };
        assert_eq!(
            row.kind.as_str(),
            expected_kind,
            "independent node {index} kind snapshot"
        );
        assert_eq!(
            row.recovery,
            NodeRecoveryState::Normal,
            "independent node {index} recovery snapshot"
        );
        assert!(
            row.children
                .iter()
                .map(|child| child.index())
                .eq(EXPECTED_CHILDREN[index].iter().copied()),
            "independent node {index} children snapshot"
        );
    }
    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b1b1_profile());
    assert!(!statement.is_task_258b3m2b2b1a_profile());
    assert_eq!(fixture.functor_contribution.index(), 2);
    assert_eq!(
        fixture
            .application
            .candidates()
            .get(SourceFunctorCandidateId::new(0))
            .expect("candidate")
            .symbol(),
        &fixture.functor_symbol
    );
    let wrapper = fixture
        .application
        .wrappers()
        .get(crate::source_application::SourceFunctorWrapperId::new(0))
        .expect("wrapped application");
    assert_eq!(wrapper.application(), SourceFunctorApplicationId::new(0));
    assert_eq!(wrapper.site().node(), TypedNodeId::new(50));
    assert_eq!(wrapper.source_range(), range(fixture.source, 129, 137));
    assert_eq!(wrapper.spelling(), "( 1 ++ 2 )");
    assert_eq!(fixture.atomic.application_fingerprint(), None);
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("wrapped application witness");
    assert_eq!(
        witnesses.application_fingerprint(),
        Some(fixture.application.debug_text().as_str())
    );
    let witness = witnesses
        .witnesses()
        .get(SourceStatementWitnessId::new(0))
        .expect("witness");
    assert_eq!(
        witness.term(),
        SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(0))
    );
    assert_eq!(witness.take_site().node(), TypedNodeId::new(53));
    assert_eq!(witness.site().node(), TypedNodeId::new(52));
    assert_eq!(witness.spelling(), "( 1 ++ 2 )");
    assert!(witness.name().is_none());
    let debug = witnesses.debug_text();
    assert!(debug.starts_with("source-statement-witness-debug-v1\n"));
    assert!(debug.contains("\napplication-fingerprint: Some(\""));
    assert!(debug.contains(" term=application#0 "));
    assert!(debug.contains(" spelling=\"( 1 ++ 2 )\""));

    let typed = fixture
        .empty_typed()
        .with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("B1B1 typed");
    assert_eq!(typed.source_application(), Some(&fixture.application));
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));

    let b1a = B3M2B2B1AFixture::new(203);
    let b1a_statement = b1a.statement();
    assert!(b1a_statement.is_task_258b3m2b2b1a_profile());
    assert!(!b1a_statement.is_task_258b3m2b2b1b1_profile());
    assert!(b1a.application.wrappers().is_empty());
    let b1a_witnesses = b1a
        .witnesses(&b1a_statement, b1a.witness_input())
        .expect("B1A witness");
    let b1a_application_debug = b1a.application.debug_text();
    let b1a_statement_debug = b1a_statement.debug_text();
    let b1a_witness_debug = b1a_witnesses.debug_text();
    let b1a_typed = b1a
        .empty_typed()
        .with_source_application_statement_witnesses(
            b1a.application.clone(),
            b1a_statement.clone(),
            b1a_witnesses.clone(),
        )
        .expect("B1A compatibility");
    assert_eq!(b1a.application.debug_text(), b1a_application_debug);
    assert_eq!(b1a_statement.debug_text(), b1a_statement_debug);
    assert_eq!(b1a_witnesses.debug_text(), b1a_witness_debug);
    assert_eq!(b1a_typed.source_application(), Some(&b1a.application));
}

#[test]
fn task258b3m2b2b1b1_dependencies_wrapped_application_witness_precedence_and_all_nodes_fail_closed()
{
    let fixture = B3M2B2B1B1Fixture::new(204);
    let statement = fixture.statement();
    let baseline = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("baseline");
    baseline
        .validate_installation_with_application(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.application),
            &fixture.arena,
        )
        .expect("revalidation");
    let statement_debug = statement.debug_text();
    let witness_debug = baseline.debug_text();
    let typed_debug = fixture
        .empty_typed()
        .with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            baseline.clone(),
        )
        .expect("baseline installation")
        .debug_text();
    let assert_clean_replay = || {
        let replay_statement = fixture.statement();
        assert_eq!(replay_statement.debug_text(), statement_debug);
        let replay_witnesses = fixture
            .witnesses(&replay_statement, fixture.witness_input())
            .expect("clean replay");
        assert_eq!(replay_witnesses.debug_text(), witness_debug);
        assert_eq!(
            fixture
                .empty_typed()
                .with_source_application_statement_witnesses(
                    fixture.application.clone(),
                    replay_statement,
                    replay_witnesses,
                )
                .expect("clean installation replay")
                .debug_text(),
            typed_debug,
        );
    };

    let foreign = B3M2B2B1B1Fixture::new(999_204);
    let wrong_arena = B3M2B2B1AFixture::new(204);
    for (label, bindings, primary, atomic, arena) in [
        (
            "foreign binding",
            &foreign.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        (
            "foreign arena",
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &wrong_arena.arena,
        ),
        (
            "foreign primary",
            &fixture.bindings,
            &foreign.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        (
            "foreign atomic",
            &fixture.bindings,
            &fixture.primary,
            &foreign.atomic,
            &fixture.arena,
        ),
    ] {
        assert_eq!(
            SourceStatementProducer::build(
                fixture.statement_input(),
                &fixture.symbols,
                bindings,
                primary,
                atomic,
                arena,
            ),
            Err(SourceStatementError::DependencyMismatch),
            "{label}"
        );
        assert_clean_replay();
    }
    assert_eq!(
        SourceStatementWitnessProducer::build(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch),
        "missing application"
    );
    assert_clean_replay();
    assert_eq!(
        SourceStatementWitnessProducer::build_with_application(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &foreign.application,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch),
        "foreign application"
    );
    assert_clean_replay();

    let b1a_binding = b3m2b2b1a_binding_env(fixture.source, &fixture.module);
    let mut binding_and_aggregate = fixture.statement_input();
    binding_and_aggregate.owners.clear();
    assert_eq!(
        SourceStatementProducer::build(
            binding_and_aggregate,
            &fixture.symbols,
            &b1a_binding,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "Task48 dependency precedes aggregate validation"
    );
    assert_clean_replay();

    let mut substituted_primary_input =
        B3M2B2B1B1Fixture::primary_input(fixture.source, &fixture.module);
    substituted_primary_input.terms[2].spelling = "3".to_owned();
    substituted_primary_input.numeric_type_requests[0].spelling = "3".to_owned();
    let substituted_primary = SourcePrimaryTermProducer::build(
        substituted_primary_input,
        &fixture.bindings,
        &fixture.arena,
    )
    .expect("coherent Task252 term/request substitution");
    assert_ne!(substituted_primary, fixture.primary);
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
        "Task252 dependency/fingerprint precedes aggregate validation"
    );
    let mut bad_reference = B3M2B2B1B1Fixture::primary_input(fixture.source, &fixture.module);
    bad_reference.references[2].role = SourcePrimaryTermReferenceRole::LocalConstant;
    assert!(
        SourcePrimaryTermProducer::build(bad_reference, &fixture.bindings, &fixture.arena,)
            .is_err(),
        "Task252 rejects reference-role corruption"
    );
    let mut bad_numeric_request = B3M2B2B1B1Fixture::primary_input(fixture.source, &fixture.module);
    bad_numeric_request.numeric_type_requests[0].request_ordinal = 1;
    assert!(
        SourcePrimaryTermProducer::build(bad_numeric_request, &fixture.bindings, &fixture.arena,)
            .is_err(),
        "Task252 rejects numeric-request order corruption"
    );
    assert_clean_replay();

    let reject_application = |input: SourceFunctorApplicationHandoffInput, label: &str| {
        assert!(
            SourceFunctorApplicationProducer::build(
                input,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.arena,
            )
            .is_err(),
            "Task253 {label}"
        );
    };
    let mut bad_candidate = B3M2B2B1B1Fixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    bad_candidate.candidates[0].ordinal = 1;
    reject_application(bad_candidate, "candidate corruption");
    let mut bad_argument = B3M2B2B1B1Fixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    bad_argument.arguments[0].target =
        SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0));
    reject_application(bad_argument, "argument corruption");
    let mut bad_application_request = B3M2B2B1B1Fixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    bad_application_request.type_requests[0].kind =
        SourceFunctorTypeRequestKind::ApplicationResultType;
    reject_application(bad_application_request, "request corruption");
    let mut stale_application_fingerprint = baseline.clone();
    stale_application_fingerprint.application_fingerprint = Some("stale".to_owned());
    stale_application_fingerprint.witnesses.rows.clear();
    assert_eq!(
        stale_application_fingerprint.validate_installation_with_application(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.application),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_clean_replay();

    // These are the five actual imported Resolver identity substitutions.
    for mutation in [
        "local/fqn",
        "structural path",
        "signature",
        "export",
        "contribution",
    ] {
        let selected = fixture
            .symbols
            .symbols()
            .get(&fixture.functor_symbol)
            .expect("imported ++");
        let mut indexes = SymbolEnvIndexes {
            imports: fixture.symbols.imports().clone(),
            symbols: Default::default(),
            labels: fixture.symbols.labels().clone(),
            definitions: fixture.symbols.definitions().clone(),
            contributions: fixture.symbols.contributions().clone(),
            ..Default::default()
        };
        let contribution = if mutation == "contribution" {
            indexes.contributions.insert(
                selected.symbol().module().clone(),
                ContributionKind::ImportedSource {
                    source_id: fixture.source,
                },
                selected.origin().anchor().clone(),
            )
        } else {
            selected.contribution()
        };
        let symbol = if mutation == "local/fqn" {
            SymbolId::new(
                selected.symbol().module().clone(),
                LocalSymbolId::new("summary:parser.type_fixtures#parse-only#++:substitute"),
                FullyQualifiedName::new("parser.type_fixtures::++#substitute"),
            )
        } else {
            selected.symbol().clone()
        };
        let origin = SemanticOrigin::new(
            selected.origin().source_id(),
            selected.origin().module_id().clone(),
            selected.origin().anchor().clone(),
            if mutation == "structural path" {
                vec![13]
            } else {
                vec![12]
            },
        );
        let mut replacement = SymbolEntry::new(
            symbol.clone(),
            selected.kind(),
            selected.namespace().clone(),
            selected.primary_spelling(),
            origin,
            contribution,
        )
        .with_visibility(selected.visibility())
        .with_export_status(if mutation == "export" {
            ExportStatus::ReExported
        } else {
            selected.export_status()
        });
        if mutation == "signature" {
            replacement = replacement.with_signature(mizar_resolve::env::SignatureShell::Pending);
        }
        assert_ne!(&replacement, selected, "Resolver {mutation}");
        for entry in fixture.symbols.symbols().iter() {
            indexes
                .symbols
                .insert(if entry.symbol() == selected.symbol() {
                    replacement.clone()
                } else {
                    entry.clone()
                });
        }
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        let symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        let application = SourceFunctorApplicationProducer::build(
            B3M2B2B1B1Fixture::application_input(
                fixture.source,
                &fixture.module,
                symbol,
                contribution,
            ),
            &symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.arena,
        );
        if mutation == "contribution" {
            assert_eq!(
                application,
                Err(
                    crate::source_application::SourceFunctorApplicationError::InvalidCandidate {
                        candidate: SourceFunctorCandidateId::new(0),
                    }
                ),
                "Resolver {mutation} loses its authentic import edge"
            );
        } else {
            let application =
                application.expect("coherent Resolver substitution reaches lower handoff");
            assert_ne!(application, fixture.application, "Resolver {mutation}");
            assert_eq!(
                SourceStatementWitnessProducer::build_with_application(
                    fixture.witness_input(),
                    &statement,
                    &fixture.primary,
                    &application,
                    &fixture.arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "Resolver {mutation}"
            );
        }
        assert_clean_replay();
    }

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
            formulas: [(44, 110, 115, 0), (58, 146, 151, 1)]
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
                Some(&fixture.application),
                None,
                None,
                &fixture.arena,
            )
            .is_err(),
            "Task256 {label}"
        );
    };
    let mut bad_formula = atomic_input();
    bad_formula.formulas[0].source_range.end -= 1;
    reject_atomic(bad_formula, "formula corruption");
    let mut bad_atomic_edge = atomic_input();
    bad_atomic_edge.edges[0].role = SourceAtomicEdgeRole::BuiltinRightOperand;
    reject_atomic(bad_atomic_edge, "edge corruption");
    let mut bad_atomic_request = atomic_input();
    bad_atomic_request.requests[0].edge = None;
    reject_atomic(bad_atomic_request, "request corruption");
    let mut excluded_application_edge = atomic_input();
    excluded_application_edge.edges[2].target =
        SourceAtomicTermTarget::Application(SourceFunctorApplicationId::new(0));
    assert!(
        SourceAtomicFormulaProducer::build(
            excluded_application_edge,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            Some(&fixture.application),
            None,
            None,
            &fixture.arena,
        )
        .is_err(),
        "Task256 excludes the witness application subtree"
    );
    assert_clean_replay();

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
    let mut theorem_resolver_owner = fixture.statement_input();
    theorem_resolver_owner.owners[0].symbol = fixture.functor_symbol.clone();
    assert!(matches!(
        SourceStatementProducer::build(
            theorem_resolver_owner,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner { .. })
    ));
    assert_clean_replay();
    type BaseMutation = (&'static str, fn(&mut SourceStatementHandoffInput));
    let base_mutations: [BaseMutation; 10] = [
        ("owner", |input: &mut SourceStatementHandoffInput| {
            input.owners[0].source_range.end -= 1
        }),
        ("statement", |input: &mut SourceStatementHandoffInput| {
            input.statements[1].source_range.start += 1
        }),
        ("context", |input: &mut SourceStatementHandoffInput| {
            input.contexts[1].binding_context = BindingContextId::new(0)
        }),
        ("input", |input: &mut SourceStatementHandoffInput| {
            input.input_facts[1].uses.swap(0, 1)
        }),
        ("candidate", |input: &mut SourceStatementHandoffInput| {
            input.candidate_facts[1].formula =
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
        }),
        ("owner spelling", |input| input.owners[0].spelling.push('x')),
        ("owners", |input| input.owners.clear()),
        ("statement ordinal", |input| {
            input.statements[0].source_ordinal = 1
        }),
        ("context range", |input| {
            input.contexts[1].source_range.start += 1
        }),
        ("candidate ordinal", |input| {
            input.candidate_facts[1].ordinal = 1
        }),
    ];
    for (label, mutate) in base_mutations {
        let mut input = fixture.statement_input();
        mutate(&mut input);
        assert_ne!(input, fixture.statement_input(), "{label}");
        let expected = match label {
            "owners" => SourceStatementError::InvalidAggregate,
            "statement ordinal" => SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0),
            },
            "owner" | "owner spelling" => SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
            "statement" => SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(1),
            },
            "context" | "context range" => SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(1),
            },
            "input" => SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new(1),
            },
            "candidate" | "candidate ordinal" => SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new(1),
            },
            _ => unreachable!(),
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
            Err(expected),
            "Task258 base {label} corruption"
        );
        assert_clean_replay();
    }

    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_204);
    dependency_before_aggregate.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_before_aggregate),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut aggregate_before_witness = fixture.witness_input();
    aggregate_before_witness.witnesses[0].site = node(51);
    aggregate_before_witness
        .witnesses
        .push(aggregate_before_witness.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, aggregate_before_witness),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    for mutate in [
        |row: &mut SourceStatementWitnessInput| {
            row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        },
        |row: &mut SourceStatementWitnessInput| {
            row.term =
                SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(1))
        },
        |row: &mut SourceStatementWitnessInput| row.source_range.start += 1,
        |row: &mut SourceStatementWitnessInput| row.source_ordinal = 0,
        |row: &mut SourceStatementWitnessInput| row.binding_context = BindingContextId::new(0),
        |row: &mut SourceStatementWitnessInput| row.site = node(51),
        |row: &mut SourceStatementWitnessInput| row.take_site = node(52),
        |row: &mut SourceStatementWitnessInput| row.spelling.push('x'),
    ] {
        let mut input = fixture.witness_input();
        mutate(&mut input.witnesses[0]);
        assert_ne!(input, fixture.witness_input());
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            })
        );
        assert_clean_replay();
    }
    let mut empty_witnesses = fixture.witness_input();
    empty_witnesses.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, empty_witnesses),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    assert_clean_replay();
    let mut nonempty_names = fixture.witness_input();
    nonempty_names.names.push(SourceStatementWitnessNameInput {
        witness: SourceStatementWitnessId::new(0),
        site: node(52),
        source_range: range(fixture.source, 129, 137),
        spelling: "x".to_owned(),
        recovery: SourceStatementRecovery::Normal,
    });
    assert_eq!(
        fixture.witnesses(&statement, nonempty_names),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );

    let mut substituted_wrapper = B3M2B2B1B1Fixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    substituted_wrapper.wrappers[0].spelling.push('x');
    assert!(
        SourceFunctorApplicationProducer::build(
            substituted_wrapper,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.arena,
        )
        .is_err(),
        "Task253 rejects wrapper substitution before Task258 publication"
    );
    let unwrapped = B3M2B2B1AFixture::new(204);
    assert_eq!(
        SourceStatementWitnessProducer::build_with_application(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &unwrapped.application,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );

    for index in 0..67 {
        let kind_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.kind = "source.task258b3m2b2b1b1.mutated".into();
            }
        });
        let anchor_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
            }
        });
        let recovery_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.recovery = NodeRecoveryState::Degraded;
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
        for (field, mutated) in [
            ("kind", kind_mutated),
            ("anchor", anchor_mutated),
            ("recovery", recovery_mutated),
            ("children", children_mutated),
        ] {
            assert_ne!(mutated, fixture.arena, "node {index} {field} mutation");
            assert_eq!(
                SourceStatementWitnessProducer::build_with_application(
                    fixture.witness_input(),
                    &statement,
                    &fixture.primary,
                    &fixture.application,
                    &mutated,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {field} constructor",
            );
            assert!(
                baseline
                    .validate_installation_with_application(
                        fixture.source,
                        &fixture.module,
                        &statement,
                        &fixture.primary,
                        Some(&fixture.application),
                        &mutated,
                    )
                    .is_err(),
                "node {index} {field}"
            );
            assert_clean_replay();
        }
    }
    let wrong_root = TypedArena::try_new(
        Some(TypedNodeId::new(65)),
        fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
    )
    .expect("wrong-root arena remains structurally valid");
    assert_ne!(wrong_root, fixture.arena);
    assert_eq!(
        SourceStatementWitnessProducer::build_with_application(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &fixture.application,
            &wrong_root,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert!(
        baseline
            .validate_installation_with_application(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                Some(&fixture.application),
                &wrong_root,
            )
            .is_err()
    );
    assert_clean_replay();
    assert_eq!(
        fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("clean replay")
            .debug_text(),
        baseline.debug_text()
    );
}

#[test]
fn task258b3m2b2b1b1_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B1B1Fixture::new(205);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let base = fixture.empty_typed();
    let typed = base
        .clone()
        .with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("combined install");
    let typed_debug = typed.debug_text();
    assert_eq!(typed.source_application(), Some(&fixture.application));
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert_eq!(
        base.clone()
            .with_source_application(fixture.application.clone())
            .expect("application first")
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed
            .clone()
            .with_source_application(fixture.application.clone()),
        Err(TypedAstError::InvalidSourceApplication)
    );
    assert_eq!(
        typed.clone().with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b1a = B3M2B2B1AFixture::new(205);
    let b1a_statement = b1a.statement();
    let b1a_witnesses = b1a
        .witnesses(&b1a_statement, b1a.witness_input())
        .expect("B1A witnesses");
    assert_eq!(
        base.clone().with_source_application_statement_witnesses(
            b1a.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone().with_source_application_statement_witnesses(
            fixture.application.clone(),
            b1a_statement.clone(),
            b1a_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let b1a_typed = b1a
        .empty_typed()
        .with_source_application_statement_witnesses(
            b1a.application.clone(),
            b1a_statement.clone(),
            b1a_witnesses.clone(),
        )
        .expect("independent B1A");
    let b1a_debug = b1a_typed.debug_text();
    assert_eq!(
        b1a.empty_typed()
            .with_source_application_statement_witnesses(
                b1a.application.clone(),
                b1a.statement(),
                b1a.witnesses(&b1a.statement(), b1a.witness_input())
                    .expect("B1A replay witnesses"),
            )
            .expect("B1A replay")
            .debug_text(),
        b1a_debug
    );
    assert_eq!(
        b1a_typed.with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B1A then B1B1"
    );

    let b3 = B3Fixture::new(205);
    let b3_statement = b3.statement();
    let b3_witnesses = b3
        .witnesses(&b3_statement, b3.witness_input())
        .expect("B3 witnesses");
    let b3_installed = b3
        .empty_typed()
        .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
        .expect("B3 installed");
    let b3n = B3NFixture::new(205);
    let b3n_statement = b3n.statement();
    let b3n_witnesses = b3n
        .witnesses(&b3n_statement, b3n.witness_input())
        .expect("B3N witnesses");
    let b3n_installed = b3n
        .empty_typed()
        .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone())
        .expect("B3N installed");
    let b3m1 = B3M1Fixture::new(205);
    let b3m1_statement = b3m1.statement();
    let b3m1_witnesses = b3m1
        .witnesses(&b3m1_statement, b3m1.witness_input())
        .expect("B3M1 witnesses");
    let b3m1_installed = b3m1
        .empty_typed()
        .with_source_statement_witnesses(b3m1_statement.clone(), b3m1_witnesses.clone())
        .expect("B3M1 installed");
    let b3m2a = B3M2AFixture::new(205);
    let b3m2a_statement = b3m2a.statement();
    let b3m2a_witnesses = b3m2a
        .witnesses(&b3m2a_statement, b3m2a.witness_input())
        .expect("B3M2A witnesses");
    let b3m2a_installed = b3m2a
        .empty_typed()
        .with_source_statement_witnesses(b3m2a_statement.clone(), b3m2a_witnesses.clone())
        .expect("B3M2A installed");
    let b3m2b1 = B3M2B1Fixture::new(205);
    let b3m2b1_statement = b3m2b1.statement();
    let b3m2b1_witnesses = b3m2b1
        .witnesses(&b3m2b1_statement, b3m2b1.witness_input())
        .expect("B3M2B1 witnesses");
    let b3m2b1_installed = b3m2b1
        .empty_typed()
        .with_source_statement_witnesses(b3m2b1_statement.clone(), b3m2b1_witnesses.clone())
        .expect("B3M2B1 installed");
    let b3m2b2a = B3M2B2AFixture::new(205);
    let b3m2b2a_statement = b3m2b2a.statement();
    let b3m2b2a_witnesses = b3m2b2a
        .witnesses(&b3m2b2a_statement, b3m2b2a.witness_input())
        .expect("B3M2B2A witnesses");
    let b3m2b2a_installed = b3m2b2a
        .empty_typed()
        .with_source_statement_witnesses(b3m2b2a_statement.clone(), b3m2b2a_witnesses.clone())
        .expect("B3M2B2A installed");
    for (label, legacy_base, legacy_statement, legacy_witnesses, legacy_installed) in [
        (
            "B3",
            b3.empty_typed(),
            b3_statement,
            b3_witnesses,
            b3_installed,
        ),
        (
            "B3N",
            b3n.empty_typed(),
            b3n_statement,
            b3n_witnesses,
            b3n_installed,
        ),
        (
            "B3M1",
            b3m1.empty_typed(),
            b3m1_statement,
            b3m1_witnesses,
            b3m1_installed,
        ),
        (
            "B3M2A",
            b3m2a.empty_typed(),
            b3m2a_statement,
            b3m2a_witnesses,
            b3m2a_installed,
        ),
        (
            "B3M2B1",
            b3m2b1.empty_typed(),
            b3m2b1_statement,
            b3m2b1_witnesses,
            b3m2b1_installed,
        ),
        (
            "B3M2B2A",
            b3m2b2a.empty_typed(),
            b3m2b2a_statement,
            b3m2b2a_witnesses,
            b3m2b2a_installed,
        ),
    ] {
        assert_eq!(
            base.clone()
                .with_source_statement_witnesses(legacy_statement, legacy_witnesses),
            Err(TypedAstError::InvalidSourceStatement),
            "B1B1 base then legacy {label}"
        );
        assert_eq!(
            legacy_base.with_source_application_statement_witnesses(
                fixture.application.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "legacy {label} base then B1B1"
        );
        assert_eq!(
            legacy_installed.with_source_application_statement_witnesses(
                fixture.application.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "installed legacy {label} then B1B1"
        );
    }

    let b1 = B1Fixture::new(205);
    let b1_references = b1.references(b1.reference_input()).expect("B1 references");
    let b1_installed = b1
        .empty_typed()
        .with_source_statement_references(b1.statement.clone(), b1_references.clone())
        .expect("B1 installed");
    assert_eq!(
        base.clone()
            .with_source_statement_references(b1.statement.clone(), b1_references),
        Err(TypedAstError::InvalidSourceStatement),
        "B1B1 base then B1"
    );
    assert_eq!(
        b1_installed.with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B1 then B1B1"
    );
    let b2 = B2Fixture::new(205);
    let b2_statement = b2.build(b2.input()).expect("B2 statement");
    let b2_installed = b2
        .empty_typed()
        .with_source_statement(b2_statement.clone())
        .expect("B2 installed");
    assert_eq!(
        base.clone().with_source_statement(b2_statement),
        Err(TypedAstError::InvalidSourceStatement),
        "B1B1 base then B2"
    );
    assert_eq!(
        b2_installed.with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B2 then B1B1"
    );
    let task258a = Fixture::new(205);
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
        base.clone().with_source_statement(task258a_statement),
        Err(TypedAstError::InvalidSourceStatement),
        "B1B1 base then Task258A"
    );
    assert_eq!(
        task258a_base.with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "Task258A base then B1B1"
    );
    assert_eq!(
        task258a_installed.with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed Task258A then B1B1"
    );

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
    .expect("Task254 empty handoff");
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
            .with_source_structure(structures.clone())
            .expect("Task254 first")
            .with_source_application_statement_witnesses(
                fixture.application.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_set_term(set_terms.clone())
            .expect("Task255 first")
            .with_source_application_statement_witnesses(
                fixture.application.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.clone().with_source_structure(structures),
        Err(TypedAstError::InvalidSourceStructure)
    );
    assert_eq!(
        typed.clone().with_source_set_term(set_terms),
        Err(TypedAstError::InvalidSourceSetTerm)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        base.with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement,
            witnesses,
        )
        .expect("atomic replay")
        .debug_text(),
        typed_debug
    );
}

#[test]
fn task258b3m2b2b1b1_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B1B1Fixture::new(206);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("typed");
    let typed_debug = typed.debug_text();
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
    let resolved_debug = resolved.debug_text();
    assert_eq!(resolved.source_application(), Some(&fixture.application));
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
    assert_eq!(typed.clone().debug_text(), typed_debug);
    assert_eq!(resolved.clone().debug_text(), resolved_debug);

    let mut stale_application = witnesses.clone();
    stale_application.application_fingerprint = Some("stale".to_owned());
    let mut stale_application_typed = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
    stale_application_typed
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_application);
    assert_eq!(
        assemble_empty_resolved(&stale_application_typed),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut stale_statement = witnesses.clone();
    stale_statement.statement_fingerprint.push('x');
    let mut stale_statement_typed = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
    stale_statement_typed
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_statement);
    assert_eq!(
        assemble_empty_resolved(&stale_statement_typed),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut stale_primary = witnesses.clone();
    stale_primary.primary_term_fingerprint.push('x');
    let mut stale_primary_typed = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
    stale_primary_typed
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_primary);
    assert_eq!(
        assemble_empty_resolved(&stale_primary_typed),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut orphan = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
    orphan.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&orphan),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut without_application = fixture.empty_typed();
    without_application
        .inject_source_statement_witness_bundle_for_test(statement.clone(), witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&without_application),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut without_witness = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
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
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("final replay")
            .debug_text(),
        resolved_debug
    );
}
