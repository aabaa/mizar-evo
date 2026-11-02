use super::*;

#[test]
fn task258b3m2b2b1a_exact_application_witness_api_debug_and_legacy_compatibility_are_stable() {
    let fixture = B3M2B2B1AFixture::new(196);
    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b1a_profile());
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
    assert_eq!(fixture.atomic.application_fingerprint(), None);
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("application witness");
    assert_eq!(
        witnesses.application_fingerprint(),
        Some(fixture.application.debug_text().as_str())
    );
    assert_eq!(
        witnesses
            .witnesses()
            .get(SourceStatementWitnessId::new(0))
            .expect("witness")
            .term(),
        SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(0))
    );
    let debug = witnesses.debug_text();
    assert!(debug.starts_with("source-statement-witness-debug-v1\n"));
    assert!(debug.contains("\napplication-fingerprint: Some(\""));
    assert!(debug.contains(" term=application#0 "));
    assert_eq!(
        SourceStatementWitnessProducer::build(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );

    let legacy = B3M2B2AFixture::new(197);
    let legacy_statement = legacy.statement();
    let legacy_witnesses = legacy
        .witnesses(&legacy_statement, legacy.witness_input())
        .expect("legacy witness");
    assert_eq!(legacy_witnesses.application_fingerprint(), None);
    assert!(
        !legacy_witnesses
            .debug_text()
            .lines()
            .any(|line| line.starts_with("application-fingerprint:"))
    );
    assert_eq!(
        legacy_witnesses
            .witnesses()
            .get(SourceStatementWitnessId::new(0))
            .expect("legacy witness row")
            .term(),
        SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
    );
    assert_eq!(
        SourceStatementWitnessProducer::build_with_application(
            legacy.witness_input(),
            &legacy_statement,
            &legacy.primary,
            &fixture.application,
            &legacy.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(
        legacy_witnesses.validate_installation_with_application(
            legacy.source,
            &legacy.module,
            &legacy_statement,
            &legacy.primary,
            Some(&fixture.application),
            &legacy.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
}

#[test]
fn task258b3m2b2b1a_dependencies_application_witness_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B1AFixture::new(198);
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
    let typed = fixture
        .empty_typed()
        .with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement.clone(),
            baseline.clone(),
        )
        .expect("baseline installation");
    let assert_clean_replay = || {
        let replay_statement = fixture.statement();
        assert_eq!(replay_statement, statement);
        let replay = fixture
            .witnesses(&replay_statement, fixture.witness_input())
            .expect("pristine witness replay");
        assert_eq!(replay, baseline);
        assert_eq!(
            fixture
                .empty_typed()
                .with_source_application_statement_witnesses(
                    fixture.application.clone(),
                    replay_statement,
                    replay,
                )
                .expect("whole installation replay"),
            typed,
        );
    };

    for mutate in [
        |row: &mut SourceStatementWitnessInput| {
            row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        },
        |row: &mut SourceStatementWitnessInput| {
            row.term =
                SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId::new(1))
        },
        |row: &mut SourceStatementWitnessInput| row.binding_context = BindingContextId::new(0),
        |row: &mut SourceStatementWitnessInput| row.site = node(47),
        |row: &mut SourceStatementWitnessInput| row.source_range.start += 1,
        |row: &mut SourceStatementWitnessInput| row.spelling.push('x'),
        |row: &mut SourceStatementWitnessInput| row.source_ordinal = 0,
        |row: &mut SourceStatementWitnessInput| row.take_site = node(48),
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
    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_198);
    dependency_before_aggregate.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_before_aggregate),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut aggregate_before_witness = fixture.witness_input();
    aggregate_before_witness.witnesses[0].site = node(47);
    aggregate_before_witness
        .witnesses
        .push(aggregate_before_witness.witnesses[0].clone());
    assert_eq!(
        fixture.witnesses(&statement, aggregate_before_witness),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    let mut empty_name_before_witness = fixture.witness_input();
    empty_name_before_witness.witnesses[0].site = node(47);
    empty_name_before_witness
        .names
        .push(SourceStatementWitnessNameInput {
            witness: SourceStatementWitnessId::new(0),
            site: node(48),
            source_range: range(fixture.source, 116, 122),
            spelling: "x".to_owned(),
            recovery: SourceStatementRecovery::Normal,
        });
    assert_eq!(
        fixture.witnesses(&statement, empty_name_before_witness),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    let mut stale_statement_fingerprint = baseline.clone();
    stale_statement_fingerprint.statement_fingerprint.push('x');
    stale_statement_fingerprint.witnesses.rows.clear();
    assert_eq!(
        stale_statement_fingerprint.validate_installation_with_application(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.application),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut stale_primary_fingerprint = baseline.clone();
    stale_primary_fingerprint.primary_term_fingerprint.push('x');
    stale_primary_fingerprint.witnesses.rows.clear();
    assert_eq!(
        stale_primary_fingerprint.validate_installation_with_application(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.application),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
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
    let foreign = B3M2B2B1AFixture::new(199);
    assert_eq!(
        SourceStatementWitnessProducer::build_with_application(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &foreign.application,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    for (label, local, fqn, path, export, pad_contribution) in [
        (
            "symbol identity",
            "summary:parser.type_fixtures#parse-only#++:13",
            "parser.type_fixtures::++#13",
            vec![12],
            ExportStatus::Exported,
            false,
        ),
        (
            "structural path",
            "summary:parser.type_fixtures#parse-only#++:12",
            "parser.type_fixtures::++#12",
            vec![13],
            ExportStatus::Exported,
            false,
        ),
        (
            "export status",
            "summary:parser.type_fixtures#parse-only#++:12",
            "parser.type_fixtures::++#12",
            vec![12],
            ExportStatus::ReExported,
            false,
        ),
        (
            "contribution",
            "summary:parser.type_fixtures#parse-only#++:12",
            "parser.type_fixtures::++#12",
            vec![12],
            ExportStatus::Exported,
            true,
        ),
    ] {
        let (_, _, symbol, contribution, symbols) = b3m2b2b1a_symbol_env_with_functor_identity(
            fixture.source,
            &fixture.module,
            local,
            fqn,
            path,
            export,
            pad_contribution,
        );
        let application = SourceFunctorApplicationProducer::build(
            B3M2B2B1AFixture::application_input(
                fixture.source,
                &fixture.module,
                symbol,
                contribution,
            ),
            &symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.arena,
        )
        .unwrap_or_else(|error| panic!("{label} lower handoff failed: {error}"));
        assert_eq!(
            SourceStatementWitnessProducer::build_with_application(
                fixture.witness_input(),
                &statement,
                &fixture.primary,
                &application,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "{label}"
        );
    }
    let replay = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("failure replay");
    assert_eq!(replay.debug_text(), baseline.debug_text());
    let mut stale_primary_input = B3M2B2B1AFixture::primary_input(fixture.source, &fixture.module);
    stale_primary_input.terms[2].spelling = "3".to_owned();
    stale_primary_input.numeric_type_requests[0].spelling = "3".to_owned();
    let stale_primary =
        SourcePrimaryTermProducer::build(stale_primary_input, &fixture.bindings, &fixture.arena)
            .expect("same-source stale primary");
    let mut stale_application_input = B3M2B2B1AFixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    stale_application_input.applications[0].spelling = "3 ++ 2".to_owned();
    let stale_application = SourceFunctorApplicationProducer::build(
        stale_application_input,
        &fixture.symbols,
        &fixture.bindings,
        &stale_primary,
        &fixture.arena,
    )
    .expect("same-source stale-primary application");
    assert_eq!(
        SourceStatementWitnessProducer::build_with_application(
            fixture.witness_input(),
            &statement,
            &fixture.primary,
            &stale_application,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let reject_lower = |input: SourceFunctorApplicationHandoffInput, label: &str| {
        assert!(
            SourceFunctorApplicationProducer::build(
                input,
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.arena,
            )
            .is_err(),
            "{label}"
        );
    };
    let mut wrong_range = B3M2B2B1AFixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    wrong_range.applications[0].source_range.start += 1;
    reject_lower(wrong_range, "application range");
    let mut wrong_head = B3M2B2B1AFixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    match &mut wrong_head.applications[0].head {
        SourceFunctorHeadSite::Single { spelling, .. } => spelling.push('+'),
        SourceFunctorHeadSite::Paired { .. } => unreachable!("single infix head"),
    }
    reject_lower(wrong_head, "application head");
    let mut wrong_form = B3M2B2B1AFixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    wrong_form.applications[0].form = SourceFunctorApplicationForm::Bare;
    reject_lower(wrong_form, "application form");
    let mut wrong_argument = B3M2B2B1AFixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    wrong_argument.arguments[0].target =
        SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0));
    reject_lower(wrong_argument, "argument target");
    let mut wrong_request = B3M2B2B1AFixture::application_input(
        fixture.source,
        &fixture.module,
        fixture.functor_symbol.clone(),
        fixture.functor_contribution,
    );
    wrong_request.type_requests[0].kind = SourceFunctorTypeRequestKind::ApplicationResultType;
    reject_lower(wrong_request, "numeric request");
    for index in 0..63 {
        let kind_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.kind = "source.task258b3m2b2b1a.mutated".into();
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
        Some(TypedNodeId::new(61)),
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
}

#[test]
fn task258b3m2b2b1a_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B1AFixture::new(200);
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
            .with_source_application(fixture.application.clone())
            .expect("Task253 first")
            .with_source_application_statement_witnesses(
                fixture.application.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
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
        typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let legacy = B3M2B2AFixture::new(200);
    let legacy_statement = legacy.statement();
    let legacy_witnesses = legacy
        .witnesses(&legacy_statement, legacy.witness_input())
        .expect("legacy witnesses");
    assert_eq!(
        legacy
            .empty_typed()
            .with_source_application_statement_witnesses(
                fixture.application.clone(),
                legacy_statement,
                legacy_witnesses,
            ),
        Err(TypedAstError::InvalidSourceStatement)
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
    assert_eq!(typed.debug_text(), typed.clone().debug_text());
    assert_eq!(
        base.with_source_application_statement_witnesses(
            fixture.application.clone(),
            statement,
            witnesses,
        )
        .expect("atomic replay")
        .debug_text(),
        typed.debug_text()
    );
}

#[test]
fn task258b3m2b2b1a_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B1AFixture::new(201);
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
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
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

    let mut stale = witnesses.clone();
    stale.application_fingerprint = Some("stale".to_owned());
    let mut stale_typed = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
    stale_typed.inject_source_statement_witness_bundle_for_test(statement.clone(), stale);
    assert_eq!(
        assemble_empty_resolved(&stale_typed),
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
    let mut standalone_statement = fixture.empty_typed();
    standalone_statement.inject_source_statement_for_test(statement.clone());
    assert_eq!(
        assemble_empty_resolved(&standalone_statement),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut statement_witnesses_without_application = fixture.empty_typed();
    statement_witnesses_without_application
        .inject_source_statement_witness_bundle_for_test(statement.clone(), witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&statement_witnesses_without_application),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut application_statement_without_witness = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
    application_statement_without_witness.inject_source_statement_for_test(statement.clone());
    assert_eq!(
        assemble_empty_resolved(&application_statement_without_witness),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let legacy = B3M2B2AFixture::new(201);
    let legacy_statement = legacy.statement();
    let legacy_witnesses = legacy
        .witnesses(&legacy_statement, legacy.witness_input())
        .expect("legacy witnesses");
    let mut cross_profile = fixture
        .empty_typed()
        .with_source_application(fixture.application.clone())
        .expect("application");
    cross_profile
        .inject_source_statement_witness_bundle_for_test(legacy_statement, legacy_witnesses);
    assert_eq!(
        assemble_empty_resolved(&cross_profile),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("replay")
            .debug_text(),
        resolved.debug_text()
    );
}
