use super::*;

#[test]
fn task269a_exact_binding_transition_links_debug_and_lookup_are_stable() {
    let fixture = B3NFixture::new(60);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269A witnesses");
    let handoff = fixture
        .proof_local(&statement, &witnesses, fixture.proof_local_input())
        .expect("Task269A handoff");

    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!(
        handoff.base_binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.witness_fingerprint(), witnesses.debug_text());
    assert_eq!(
        handoff.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(
        handoff.final_binding_fingerprint(),
        handoff.binding_env().debug_text()
    );
    assert_eq!(
        (
            handoff.binding_env().contexts().len(),
            handoff.binding_env().bindings().len(),
            handoff.binding_env().diagnostics().len(),
            handoff.declarations().len(),
            handoff.declarations().is_empty(),
        ),
        (2, 2, 0, 1, false)
    );

    let declaration_id = SourceProofLocalDeclarationId::new(0);
    assert_eq!(declaration_id.index(), 0);
    let declaration = handoff
        .declarations()
        .get(declaration_id)
        .expect("declaration");
    assert_eq!(
        (
            declaration.witness(),
            declaration.name(),
            declaration.rhs(),
            declaration.binding(),
            declaration.binding_context(),
            declaration.source_ordinal(),
            declaration.visible_after_ordinal(),
            declaration.kind(),
            declaration.recovery(),
        ),
        (
            SourceStatementWitnessId::new(0),
            SourceStatementWitnessNameId::new(0),
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2)),
            BindingId::new(1),
            BindingContextId::new(1),
            1,
            1,
            SourceProofLocalDeclarationKind::NamedWitness,
            SourceProofLocalDeclarationRecovery::Normal,
        )
    );
    assert_eq!(
        handoff
            .declarations()
            .iter()
            .map(|(id, row)| (id.index(), row.binding().index()))
            .collect::<Vec<_>>(),
        [(0, 1)]
    );

    assert_eq!(
        handoff
            .binding_env()
            .contexts()
            .get(BindingContextId::new(0)),
        statement
            .binding_env()
            .contexts()
            .get(BindingContextId::new(0))
    );
    let base_proof = statement
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .expect("base proof context");
    let proof = handoff
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .expect("final proof context");
    assert_eq!(
        (
            &proof.owner,
            proof.parent,
            proof.layer,
            proof.lexical_scope.as_ref(),
            proof.bindings.as_slice(),
            proof.visible_bindings.as_slice(),
            proof.recovery,
        ),
        (
            &base_proof.owner,
            base_proof.parent,
            base_proof.layer,
            base_proof.lexical_scope.as_ref(),
            &[BindingId::new(1)][..],
            &[BindingId::new(0), BindingId::new(1)][..],
            base_proof.recovery,
        )
    );
    assert_eq!(
        handoff.binding_env().bindings().get(BindingId::new(0)),
        statement.binding_env().bindings().get(BindingId::new(0))
    );
    let binding = handoff
        .binding_env()
        .bindings()
        .get(BindingId::new(1))
        .expect("named witness binding");
    assert_eq!(binding.id, BindingId::new(1));
    assert_eq!(binding.spelling, "y");
    assert_eq!(binding.kind, BindingKind::LocalAbbreviation);
    assert_eq!(binding.owner_context, BindingContextId::new(1));
    assert_eq!(binding.declaration_range, range(fixture.source, 81, 82));
    assert_eq!(binding.visible_after_ordinal, 1);
    assert_eq!(binding.type_site, BindingTypeSite::Missing);
    assert_eq!(binding.status, BindingStatus::Active);
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(binding.recovery, BindingRecoveryState::Normal);
    assert_eq!(
        binding.identity,
        BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![0]),
            ordinal: 1,
            declaration_range: range(fixture.source, 81, 82),
        }
    );

    let definition_site = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        1,
    );
    assert!(matches!(
        handoff
            .binding_env()
            .lookup(&definition_site)
            .expect("definition-site lookup"),
        BindingLookupResult::ForwardReference { candidates, .. }
            if candidates == [BindingId::new(1)]
    ));
    let later_site = BindingLookupSite::new(
        "y",
        BindingContextId::new(1),
        Some(LocalTermScope::new(vec![0])),
        2,
    );
    assert_eq!(
        handoff
            .binding_env()
            .lookup(&later_site)
            .expect("later lookup"),
        BindingLookupResult::Local(BindingId::new(1))
    );
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-proof-local-declaration-debug-v1\nmodule: pkg::statement.fixture\nbase-binding-fingerprint: {:?}\nstatement-fingerprint: {:?}\nwitness-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\ndeclaration#0 kind=named-witness witness=0 name=0 rhs=primary#2 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal\nfinal-binding-fingerprint: {:?}\n",
            statement.binding_env().debug_text(),
            statement.debug_text(),
            witnesses.debug_text(),
            fixture.primary.debug_text(),
            handoff.binding_env().debug_text(),
        )
    );
    assert_task269b_exact_mixed_witness_binding();
}

#[test]
fn task269a_transaction_dependency_row_arena_and_fingerprint_fail_closed() {
    let fixture = B3NFixture::new(61);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269A witnesses");
    let baseline = fixture
        .proof_local(&statement, &witnesses, fixture.proof_local_input())
        .expect("Task269A baseline");
    let pristine_input = fixture.proof_local_input();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let pristine_statement = statement.clone();
    let pristine_witnesses = witnesses.clone();
    let statement_debug = statement.debug_text();
    let witness_debug = witnesses.debug_text();
    let baseline_base = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("Task258B3N base replay");
    let baseline_base_debug = baseline_base.debug_text();
    let baseline_typed = baseline_base
        .clone()
        .with_source_proof_local_declaration(baseline.clone())
        .expect("Task269A typed replay");
    let baseline_typed_debug = baseline_typed.debug_text();
    let baseline_final_debug = assemble_empty_resolved(&baseline_typed)
        .expect("Task269A final replay")
        .debug_text();
    let assert_replay = || {
        assert_eq!(fixture.proof_local_input(), pristine_input);
        assert_eq!(fixture.symbols, pristine_symbols);
        assert_eq!(fixture.bindings, pristine_bindings);
        assert_eq!(fixture.primary, pristine_primary);
        assert_eq!(fixture.atomic, pristine_atomic);
        assert_eq!(fixture.arena, pristine_arena);
        assert_eq!(statement, pristine_statement);
        assert_eq!(witnesses, pristine_witnesses);
        assert_eq!(statement.debug_text(), statement_debug);
        assert_eq!(witnesses.debug_text(), witness_debug);
        let replay_statement = fixture.statement();
        let replay_witnesses = fixture
            .witnesses(&replay_statement, fixture.witness_input())
            .expect("Task269A canonical witness replay");
        let replay_handoff = fixture
            .proof_local(&replay_statement, &replay_witnesses, pristine_input.clone())
            .expect("Task269A canonical handoff replay");
        assert_eq!(replay_handoff.debug_text(), baseline.debug_text());
        let replay_base = fixture
            .empty_typed()
            .with_source_statement_witnesses(replay_statement, replay_witnesses)
            .expect("Task258B3N canonical typed replay");
        assert_eq!(replay_base.debug_text(), baseline_base_debug);
        let replay_typed = replay_base
            .with_source_proof_local_declaration(replay_handoff)
            .expect("Task269A complete typed replay");
        assert_eq!(replay_typed.debug_text(), baseline_typed_debug);
        assert_eq!(
            assemble_empty_resolved(&replay_typed)
                .expect("Task269A canonical final replay")
                .debug_text(),
            baseline_final_debug
        );
    };

    let mut wrong_transaction = fixture.proof_local_input();
    wrong_transaction.source_id = source_id(161);
    assert_eq!(
        fixture.proof_local(&statement, &witnesses, wrong_transaction),
        Err(SourceProofLocalDeclarationError::InvalidTransaction)
    );
    assert_replay();
    let mut stale_witnesses = witnesses.clone();
    stale_witnesses.statement_fingerprint.push('x');
    let stale_witnesses_before = stale_witnesses.clone();
    let statement_before_stale = statement.clone();
    let primary_before_stale = fixture.primary.clone();
    let arena_before_stale = fixture.arena.clone();
    assert_eq!(
        fixture.proof_local(&statement, &stale_witnesses, fixture.proof_local_input(),),
        Err(SourceProofLocalDeclarationError::DependencyMismatch)
    );
    assert_eq!(stale_witnesses, stale_witnesses_before);
    assert_eq!(statement, statement_before_stale);
    assert_eq!(fixture.primary, primary_before_stale);
    assert_eq!(fixture.arena, arena_before_stale);
    assert_replay();
    let mut empty = fixture.proof_local_input();
    empty.declarations.clear();
    assert_eq!(
        fixture.proof_local(&statement, &witnesses, empty),
        Err(SourceProofLocalDeclarationError::InvalidAggregate)
    );
    assert_replay();

    for label in ["witness", "name", "rhs", "context", "source ordinal"] {
        let mut input = fixture.proof_local_input();
        match label {
            "witness" => input.declarations[0].witness = SourceStatementWitnessId::new(1),
            "name" => input.declarations[0].name = SourceStatementWitnessNameId::new(1),
            "rhs" => {
                input.declarations[0].rhs =
                    SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3))
            }
            "context" => input.declarations[0].binding_context = BindingContextId::new(0),
            "source ordinal" => input.declarations[0].source_ordinal = 2,
            _ => unreachable!(),
        }
        let statement_before = statement.clone();
        let witnesses_before = witnesses.clone();
        let primary_before = fixture.primary.clone();
        let arena_before = fixture.arena.clone();
        assert_eq!(
            fixture.proof_local(&statement, &witnesses, input),
            Err(SourceProofLocalDeclarationError::InvalidDeclaration {
                declaration: SourceProofLocalDeclarationId::new(0),
            }),
            "input {label}"
        );
        assert_eq!(statement, statement_before);
        assert_eq!(witnesses, witnesses_before);
        assert_eq!(fixture.primary, primary_before);
        assert_eq!(fixture.arena, arena_before);
        assert_replay();
    }

    for (label, local) in [
        (
            "spelling",
            LocalTermBinding::new(
                "z",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 81, 82),
                1,
            ),
        ),
        (
            "scope",
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![1]),
                range(fixture.source, 81, 82),
                1,
            ),
        ),
        (
            "range",
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 80, 82),
                1,
            ),
        ),
        (
            "range-source",
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(source_id(1_061), 81, 82),
                1,
            ),
        ),
        (
            "visible-after",
            LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                range(fixture.source, 81, 82),
                2,
            ),
        ),
    ] {
        let mut invalid = fixture.proof_local_input();
        invalid.declarations[0].local = local;
        assert_eq!(
            fixture.proof_local(&statement, &witnesses, invalid),
            Err(SourceProofLocalDeclarationError::InvalidDeclaration {
                declaration: SourceProofLocalDeclarationId::new(0),
            }),
            "{label}"
        );
        assert_replay();
    }

    for index in 0..51 {
        let arena = b3n_typed_arena_with_corrupt_kind(fixture.source, index);
        let arena_before = arena.clone();
        let primary_before = fixture.primary.clone();
        assert_eq!(
            SourceProofLocalDeclarationProducer::build(
                fixture.proof_local_input(),
                &statement,
                &witnesses,
                &fixture.primary,
                &arena,
            ),
            Err(SourceProofLocalDeclarationError::InvalidArena),
            "node {index}"
        );
        assert_eq!(arena, arena_before);
        assert_eq!(fixture.primary, primary_before);
        assert_replay();
    }
    for corruption in [
        Task269AArenaCorruption::Cardinality,
        Task269AArenaCorruption::Root,
        Task269AArenaCorruption::Anchor,
        Task269AArenaCorruption::Children,
        Task269AArenaCorruption::ResolvedNode,
        Task269AArenaCorruption::Recovery,
        Task269AArenaCorruption::Typing,
        Task269AArenaCorruption::Links,
    ] {
        let arena = b3n_typed_arena_with_corruption(fixture.source, corruption);
        let arena_before = arena.clone();
        let primary_before = fixture.primary.clone();
        assert_eq!(
            SourceProofLocalDeclarationProducer::build(
                fixture.proof_local_input(),
                &statement,
                &witnesses,
                &fixture.primary,
                &arena,
            ),
            Err(SourceProofLocalDeclarationError::InvalidArena),
            "representative {corruption:?}"
        );
        assert_eq!(arena, arena_before);
        assert_eq!(fixture.primary, primary_before);
        assert_replay();
    }

    let mut invalid_row = baseline.clone();
    invalid_row.set_binding_for_test(SourceProofLocalDeclarationId::new(0), BindingId::new(0));
    assert_eq!(
        invalid_row.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidDeclaration {
            declaration: SourceProofLocalDeclarationId::new(0),
        })
    );
    assert_replay();
    for corruption in [
        SourceProofLocalBindingCorruptionForTest::Spelling,
        SourceProofLocalBindingCorruptionForTest::Scope,
        SourceProofLocalBindingCorruptionForTest::Range,
        SourceProofLocalBindingCorruptionForTest::Ordinal,
    ] {
        let mut invalid_local = baseline.clone();
        invalid_local.corrupt_local_binding_for_test(corruption);
        assert_eq!(
            invalid_local.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &witnesses,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceProofLocalDeclarationError::InvalidDeclaration {
                declaration: SourceProofLocalDeclarationId::new(0),
            }),
            "installed local {corruption:?}"
        );
        assert_replay();
    }
    let mut invalid_output_aggregate = baseline.clone();
    invalid_output_aggregate.truncate_declarations_for_test(0);
    assert_eq!(
        invalid_output_aggregate.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidAggregate)
    );
    assert_replay();
    let mut duplicate_output_aggregate = baseline.clone();
    duplicate_output_aggregate.duplicate_declaration_for_test();
    assert_eq!(
        duplicate_output_aggregate.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidAggregate)
    );
    assert_replay();

    let corrupt_fingerprint = |value: &str| {
        let mut corrupted = value.to_owned();
        corrupted.replace_range(0..1, "x");
        corrupted
    };
    for (label, invalid_fingerprint) in [
        {
            let mut handoff = baseline.clone();
            handoff.set_base_binding_fingerprint_for_test(corrupt_fingerprint(
                handoff.base_binding_fingerprint(),
            ));
            ("base", handoff)
        },
        {
            let mut handoff = baseline.clone();
            handoff.set_statement_fingerprint_for_test(corrupt_fingerprint(
                handoff.statement_fingerprint(),
            ));
            ("statement", handoff)
        },
        {
            let mut handoff = baseline.clone();
            handoff.set_witness_fingerprint_for_test(corrupt_fingerprint(
                handoff.witness_fingerprint(),
            ));
            ("witness", handoff)
        },
        {
            let mut handoff = baseline.clone();
            handoff.set_primary_term_fingerprint_for_test(corrupt_fingerprint(
                handoff.primary_term_fingerprint(),
            ));
            ("primary", handoff)
        },
    ] {
        assert_eq!(
            invalid_fingerprint.validate_installation(
                fixture.source,
                &fixture.module,
                &statement,
                &witnesses,
                &fixture.primary,
                &fixture.arena,
            ),
            Err(SourceProofLocalDeclarationError::DependencyMismatch),
            "fingerprint {label}"
        );
        assert_replay();
    }
    let mut invalid_fingerprint = baseline.clone();
    invalid_fingerprint.set_final_binding_fingerprint_for_test(corrupt_fingerprint(
        invalid_fingerprint.final_binding_fingerprint(),
    ));
    assert_eq!(
        invalid_fingerprint.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidBindingEnvironment)
    );
    assert_replay();

    let wrong_module = ModuleId::new(
        PackageId::new("pkg"),
        ModulePath::new("statement.task269a.wrong"),
    );
    assert_eq!(
        baseline.validate_installation(
            fixture.source,
            &wrong_module,
            &statement,
            &witnesses,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidTransaction)
    );
    assert_replay();
    let corrupt_arena =
        b3n_typed_arena_with_corruption(fixture.source, Task269AArenaCorruption::Recovery);
    let corrupt_arena_before = corrupt_arena.clone();
    let primary_before_precedence = fixture.primary.clone();
    let mut transaction_first = baseline.clone();
    transaction_first.duplicate_declaration_for_test();
    transaction_first.set_statement_fingerprint_for_test("stale");
    assert_eq!(
        transaction_first.validate_installation(
            source_id(1_062),
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &corrupt_arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidTransaction)
    );
    assert_eq!(corrupt_arena, corrupt_arena_before);
    assert_eq!(fixture.primary, primary_before_precedence);
    assert_replay();
    let mut dependency_first = baseline.clone();
    dependency_first.duplicate_declaration_for_test();
    dependency_first.set_statement_fingerprint_for_test("stale");
    assert_eq!(
        dependency_first.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &corrupt_arena,
        ),
        Err(SourceProofLocalDeclarationError::DependencyMismatch)
    );
    assert_eq!(corrupt_arena, corrupt_arena_before);
    assert_eq!(fixture.primary, primary_before_precedence);
    assert_replay();
    let mut aggregate_first = baseline.clone();
    aggregate_first.duplicate_declaration_for_test();
    aggregate_first.set_binding_for_test(SourceProofLocalDeclarationId::new(0), BindingId::new(0));
    aggregate_first.set_final_binding_fingerprint_for_test("stale");
    assert_eq!(
        aggregate_first.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &corrupt_arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidAggregate)
    );
    assert_eq!(corrupt_arena, corrupt_arena_before);
    assert_eq!(fixture.primary, primary_before_precedence);
    assert_replay();
    let mut row_first = baseline.clone();
    row_first.set_binding_for_test(SourceProofLocalDeclarationId::new(0), BindingId::new(0));
    row_first.set_final_binding_fingerprint_for_test("stale");
    assert_eq!(
        row_first.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &corrupt_arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidDeclaration {
            declaration: SourceProofLocalDeclarationId::new(0),
        })
    );
    assert_eq!(corrupt_arena, corrupt_arena_before);
    assert_eq!(fixture.primary, primary_before_precedence);
    assert_replay();
    let mut arena_first = baseline.clone();
    arena_first.set_final_binding_fingerprint_for_test("stale");
    assert_eq!(
        arena_first.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &corrupt_arena,
        ),
        Err(SourceProofLocalDeclarationError::InvalidArena)
    );
    assert_eq!(corrupt_arena, corrupt_arena_before);
    assert_eq!(fixture.primary, primary_before_precedence);
    assert_replay();
    assert_eq!(
        arena_first.validate_complete_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &corrupt_arena,
            false,
        ),
        Err(SourceProofLocalDeclarationError::InvalidArena)
    );
    assert_eq!(corrupt_arena, corrupt_arena_before);
    assert_eq!(fixture.primary, primary_before_precedence);
    assert_replay();
    assert_eq!(
        baseline.validate_complete_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &witnesses,
            &fixture.primary,
            &fixture.arena,
            false,
        ),
        Err(SourceProofLocalDeclarationError::InvalidInstallation)
    );
    assert_replay();
    assert_eq!(statement.debug_text(), statement_debug);
    assert_eq!(witnesses.debug_text(), witness_debug);
    assert_eq!(
        SourceProofLocalDeclarationError::InvalidTransaction.to_string(),
        "source proof-local declaration transaction is invalid"
    );
    assert_eq!(
        SourceProofLocalDeclarationError::DependencyMismatch.to_string(),
        "source proof-local declaration dependency mismatch"
    );
    assert_eq!(
        SourceProofLocalDeclarationError::InvalidAggregate.to_string(),
        "source proof-local declaration aggregate is invalid"
    );
    assert_eq!(
        SourceProofLocalDeclarationError::InvalidDeclaration {
            declaration: SourceProofLocalDeclarationId::new(0),
        }
        .to_string(),
        "source proof-local declaration 0 is invalid"
    );
    assert_eq!(
        SourceProofLocalDeclarationError::InvalidArena.to_string(),
        "source proof-local declaration arena is invalid"
    );
    assert_eq!(
        SourceProofLocalDeclarationError::InvalidBindingEnvironment.to_string(),
        "source proof-local declaration binding environment is invalid"
    );
    assert_eq!(
        SourceProofLocalDeclarationError::InvalidInstallation.to_string(),
        "source proof-local declaration installation is invalid"
    );
    assert_task269b_mixed_witness_corruption_and_cross_profile_fail_closed(
        &fixture, &statement, &witnesses,
    );
    assert_replay();
}

#[test]
fn task269a_typed_ownership_is_one_shot_atomic_and_legacy_compatible() {
    let fixture = B3NFixture::new(62);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269A witnesses");
    let handoff = fixture
        .proof_local(&statement, &witnesses, fixture.proof_local_input())
        .expect("Task269A handoff");
    let base = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement, witnesses)
        .expect("Task258B3N base");
    let legacy_debug = base.debug_text();
    let legacy_nodes = base.nodes().clone();
    assert!(base.source_proof_local_declaration().is_none());
    assert!(!legacy_debug.contains("source-proof-local-declaration-debug-v1"));

    assert_eq!(
        fixture
            .empty_typed()
            .with_source_proof_local_declaration(handoff.clone()),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );
    let typed = base
        .clone()
        .with_source_proof_local_declaration(handoff.clone())
        .expect("Task269A typed install");
    assert_eq!(typed.source_proof_local_declaration(), Some(&handoff));
    assert_eq!(typed.nodes(), &legacy_nodes);
    assert!(
        typed
            .debug_text()
            .contains("source-proof-local-declaration-debug-v1")
    );
    assert_eq!(base.debug_text(), legacy_debug);
    let typed_debug = typed.debug_text();
    assert_eq!(
        typed
            .clone()
            .with_source_proof_local_declaration(handoff.clone()),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );
    assert_eq!(typed.debug_text(), typed_debug);

    for table in [
        StatementTransportTableForTest::Context,
        StatementTransportTableForTest::Type,
        StatementTransportTableForTest::Fact,
        StatementTransportTableForTest::Coercion,
        StatementTransportTableForTest::InitialObligation,
        StatementTransportTableForTest::Diagnostic,
    ] {
        let mut occupied = base.clone();
        occupied.occupy_statement_transport_table_for_test(table);
        assert_eq!(
            occupied.with_source_proof_local_declaration(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalDeclaration),
            "occupied table {table:?}"
        );
    }

    let mut stale = handoff.clone();
    let mut same_length = stale.statement_fingerprint().to_owned();
    same_length.replace_range(0..1, "x");
    stale.set_statement_fingerprint_for_test(same_length);
    assert_eq!(
        base.clone().with_source_proof_local_declaration(stale),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );
    assert_eq!(base.debug_text(), legacy_debug);

    let mut orphan = fixture.empty_typed();
    orphan.inject_source_proof_local_declaration_for_test(handoff);
    assert_eq!(
        assemble_empty_resolved(&orphan),
        Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
    );
    assert_task269b_mixed_witness_typed_ownership();
}

#[test]
fn task269a_final_clone_replay_sibling_isolation_and_deferrals_are_stable() {
    let fixture = B3NFixture::new(63);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("Task269A witnesses");
    let handoff = fixture
        .proof_local(&statement, &witnesses, fixture.proof_local_input())
        .expect("Task269A handoff");
    let base = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("Task258B3N base");
    let typed = base
        .clone()
        .with_source_proof_local_declaration(handoff.clone())
        .expect("Task269A typed");
    let resolved = assemble_empty_resolved(&typed).expect("Task269A resolved");
    assert_eq!(resolved.source_proof_local_declaration(), Some(&handoff));
    assert_eq!(
        resolved.source_proof_local_declaration(),
        typed.source_proof_local_declaration()
    );
    assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("Task269A replay")
            .debug_text(),
        resolved.debug_text()
    );
    assert!(typed.contexts().is_empty());
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.diagnostics().is_empty());
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

    let mut stale = handoff.clone();
    let mut same_length = stale.statement_fingerprint().to_owned();
    same_length.replace_range(0..1, "x");
    stale.set_statement_fingerprint_for_test(same_length);
    let mut stale_final = base.clone();
    stale_final.inject_source_proof_local_declaration_for_test(stale);
    assert_eq!(
        assemble_empty_resolved(&stale_final),
        Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
    );

    let mut statement_only = fixture.empty_typed();
    statement_only.inject_source_statement_for_test(statement.clone());
    statement_only.inject_source_proof_local_declaration_for_test(handoff.clone());
    assert_eq!(
        assemble_empty_resolved(&statement_only),
        Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
    );
    let mut witness_only = fixture.empty_typed();
    witness_only.inject_source_statement_witnesses_for_test(witnesses.clone());
    witness_only.inject_source_proof_local_declaration_for_test(handoff.clone());
    assert_eq!(
        assemble_empty_resolved(&witness_only),
        Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
    );
    let mut missing_term = base.clone();
    missing_term.inject_source_proof_local_declaration_for_test(handoff.clone());
    missing_term.remove_source_term_for_test();
    assert_eq!(
        assemble_empty_resolved(&missing_term),
        Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
    );
    let mut missing_atomic = base.clone();
    missing_atomic.inject_source_proof_local_declaration_for_test(handoff.clone());
    missing_atomic.remove_source_atomic_formula_for_test();
    assert_eq!(
        assemble_empty_resolved(&missing_atomic),
        Err(ResolvedTypedAstError::InvalidSourceProofLocalDeclaration)
    );

    let reference_fixture = B1Fixture::new(63);
    let references = reference_fixture
        .references(reference_fixture.reference_input())
        .expect("Task258B1 sibling references");
    let mut reference_sibling = base.clone();
    reference_sibling.inject_source_statement_references_for_test(references);
    assert_eq!(
        reference_sibling.with_source_proof_local_declaration(handoff.clone()),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );

    let sibling = B3Fixture::new(63);
    let sibling_statement = sibling.statement();
    let sibling_witnesses = sibling
        .witnesses(&sibling_statement, sibling.witness_input())
        .expect("Task258B3 sibling witnesses");
    let sibling_base = sibling
        .empty_typed()
        .with_source_statement_witnesses(sibling_statement, sibling_witnesses)
        .expect("Task258B3 sibling");
    assert_eq!(
        sibling_base.with_source_proof_local_declaration(handoff),
        Err(TypedAstError::InvalidSourceProofLocalDeclaration)
    );
    assert_task269b_mixed_witness_final_replay_and_deferrals();
}
