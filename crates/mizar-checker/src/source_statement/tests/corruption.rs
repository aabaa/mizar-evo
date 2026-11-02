use super::*;

#[test]
fn corruption_precedence_provenance_and_revalidation_fail_closed() {
    let fixture = Fixture::new(2);
    let valid = fixture.build(fixture.input()).expect("baseline");
    let valid_debug = valid.debug_text();
    let mut wrong_ordinal_primary = fixture.primary.clone();
    wrong_ordinal_primary
        .set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(0), 2);
    let mut matching_atomic = fixture.atomic.clone();
    matching_atomic.set_primary_term_fingerprint_for_test(wrong_ordinal_primary.debug_text());
    let mut wrong_second_ordinal_primary = fixture.primary.clone();
    wrong_second_ordinal_primary
        .set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(1), 2);
    let mut matching_second_atomic = fixture.atomic.clone();
    matching_second_atomic
        .set_primary_term_fingerprint_for_test(wrong_second_ordinal_primary.debug_text());

    let mut aggregate = fixture.input();
    aggregate.owners.push(aggregate.owners[0].clone());
    aggregate.statements[0].source_ordinal = 7;
    assert_eq!(
        fixture.build(aggregate),
        Err(SourceStatementError::InvalidAggregate)
    );

    let mut missing_owner = fixture.input();
    missing_owner.owners.clear();
    assert_eq!(
        fixture.build(missing_owner),
        Err(SourceStatementError::InvalidAggregate)
    );
    assert_eq!(fixture.build(fixture.input()), Ok(valid.clone()));

    let (_, _, imported_label_symbols) = symbol_env_with_mutation(
        fixture.source,
        &fixture.module,
        ResolverMutation::ImportedLabelOrigin,
    );
    assert_eq!(
        SourceStatementProducer::build(
            fixture.input(),
            &imported_label_symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        }),
    );
    assert_eq!(fixture.build(fixture.input()), Ok(valid.clone()));

    let mut owner = fixture.input();
    owner.owners[0].spelling.push('x');
    assert_eq!(
        fixture.build(owner),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        })
    );
    let mut statement = fixture.input();
    statement.statements[0].source_ordinal = 1;
    assert_eq!(
        fixture.build(statement),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0)
        })
    );
    let mut context = fixture.input();
    context.contexts[0].visible_bindings.clear();
    assert_eq!(
        fixture.build(context),
        Err(SourceStatementError::InvalidContext {
            context: SourceStatementContextId::new(0)
        })
    );
    let mut input_fact = fixture.input();
    input_fact.input_facts[0].uses.swap(0, 1);
    assert_eq!(
        fixture.build(input_fact),
        Err(SourceStatementError::InvalidInputFact {
            fact: SourceStatementInputFactId::new(0)
        })
    );
    let mut candidate = fixture.input();
    candidate.candidate_facts[0].ordinal = 1;
    assert_eq!(
        fixture.build(candidate),
        Err(SourceStatementError::InvalidCandidateFact {
            fact: SourceStatementCandidateFactId::new(0)
        })
    );

    let mut dependency_and_aggregate_input = fixture.input();
    dependency_and_aggregate_input.owners.clear();
    assert_eq!(
        SourceStatementProducer::build(
            dependency_and_aggregate_input,
            &fixture.symbols,
            &fixture.bindings,
            &wrong_ordinal_primary,
            &matching_atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let mut owner_and_later_rows = fixture.input();
    owner_and_later_rows.owners[0].source_range = range(fixture.source, 20, 80);
    owner_and_later_rows.statements[0].source_ordinal = 1;
    owner_and_later_rows.contexts[0].visible_bindings.clear();
    owner_and_later_rows.input_facts[0].ordinal = 1;
    owner_and_later_rows.candidate_facts[0].ordinal = 1;
    assert_eq!(
        fixture.build(owner_and_later_rows),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        })
    );
    let mut statement_and_later_rows = fixture.input();
    statement_and_later_rows.statements[0].source_ordinal = 1;
    statement_and_later_rows.contexts[0]
        .visible_bindings
        .clear();
    statement_and_later_rows.input_facts[0].ordinal = 1;
    statement_and_later_rows.candidate_facts[0].ordinal = 1;
    assert_eq!(
        fixture.build(statement_and_later_rows),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0)
        })
    );
    let mut context_and_later_rows = fixture.input();
    context_and_later_rows.contexts[0].visible_bindings.clear();
    context_and_later_rows.input_facts[0].ordinal = 1;
    context_and_later_rows.candidate_facts[0].ordinal = 1;
    assert_eq!(
        fixture.build(context_and_later_rows),
        Err(SourceStatementError::InvalidContext {
            context: SourceStatementContextId::new(0)
        })
    );
    let mut input_and_candidate_rows = fixture.input();
    input_and_candidate_rows.input_facts[0].ordinal = 1;
    input_and_candidate_rows.candidate_facts[0].ordinal = 1;
    assert_eq!(
        fixture.build(input_and_candidate_rows),
        Err(SourceStatementError::InvalidInputFact {
            fact: SourceStatementInputFactId::new(0)
        })
    );

    let mut dependency_and_aggregate = valid.clone();
    dependency_and_aggregate.binding_fingerprint.push('x');
    dependency_and_aggregate.owners.rows.clear();
    assert_eq!(
        dependency_and_aggregate.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let mut aggregate_and_row = valid.clone();
    aggregate_and_row.contexts.rows.clear();
    aggregate_and_row.statements.rows[0].source_ordinal = 1;
    assert_eq!(
        aggregate_and_row.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidAggregate)
    );
    let mut stale_primary_fingerprint = valid.clone();
    stale_primary_fingerprint
        .primary_term_fingerprint
        .push_str(":stale");
    assert_eq!(
        stale_primary_fingerprint.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let mut stale_atomic_fingerprint = valid.clone();
    stale_atomic_fingerprint
        .atomic_formula_fingerprint
        .push_str(":stale");
    assert_eq!(
        stale_atomic_fingerprint.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let mut coherent_wrong_ordinal = valid.clone();
    coherent_wrong_ordinal.primary_term_fingerprint = wrong_ordinal_primary.debug_text();
    coherent_wrong_ordinal.atomic_formula_fingerprint = matching_atomic.debug_text();
    assert_eq!(
        coherent_wrong_ordinal.validate_installation(
            fixture.source,
            &fixture.module,
            &wrong_ordinal_primary,
            &matching_atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let mut coherent_wrong_second_ordinal = valid.clone();
    coherent_wrong_second_ordinal.primary_term_fingerprint =
        wrong_second_ordinal_primary.debug_text();
    coherent_wrong_second_ordinal.atomic_formula_fingerprint = matching_second_atomic.debug_text();
    assert_eq!(
        coherent_wrong_second_ordinal.validate_installation(
            fixture.source,
            &fixture.module,
            &wrong_second_ordinal_primary,
            &matching_second_atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let substituted_binding = binding_env_with_visible_after(fixture.source, &fixture.module, 1);
    let mut coherent_wrong_binding = valid.clone();
    coherent_wrong_binding.binding_env = substituted_binding.clone();
    coherent_wrong_binding.binding_fingerprint = substituted_binding.debug_text();
    assert_eq!(
        coherent_wrong_binding.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );

    let mut stale_checked = valid.clone();
    stale_checked.checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
        fixture.symbol.clone(),
        range(fixture.source, 20, 80),
        SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 20, 80)),
            vec![1],
        ),
    );
    assert_eq!(
        stale_checked.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        })
    );
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
    let mut stale_contribution = valid.clone();
    stale_contribution.owners.rows[0].contribution = foreign_contribution;
    assert_eq!(
        stale_contribution.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        })
    );

    for mutation in [
        ResolverMutation::MissingLabelEffect,
        ResolverMutation::RecoveredLabel,
        ResolverMutation::WrongContribution,
        ResolverMutation::DuplicateLabel,
        ResolverMutation::StaleLabel,
        ResolverMutation::WrongLabelKind,
        ResolverMutation::PrivateLabel,
        ResolverMutation::LocalOnlyLabel,
        ResolverMutation::WrongLabelSource,
        ResolverMutation::WrongLabelModule,
        ResolverMutation::WrongLabelNamespace,
        ResolverMutation::WrongResolverNamespace,
        ResolverMutation::WrongContributionAnchor,
    ] {
        let (symbol, contribution, symbols) =
            symbol_env_with_mutation(fixture.source, &fixture.module, mutation);
        let mut input = fixture.input();
        input.owners[0].symbol = symbol;
        input.owners[0].contribution = contribution;
        assert_eq!(
            SourceStatementProducer::build(
                input,
                &symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            })
        );
    }

    let replay = fixture.build(fixture.input()).expect("valid replay");
    assert_eq!(replay.debug_text(), valid_debug);
    assert_eq!(replay, valid);
}
