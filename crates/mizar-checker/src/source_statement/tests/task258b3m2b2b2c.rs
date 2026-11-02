use super::*;

#[test]
fn task258b3m2b2b2c_exact_structure_update_witness_api_debug_and_legacy_compatibility_are_stable() {
    let fixture = B3M2B2B2CFixture::new(222);
    assert_eq!(TASK258B3M2B2B2C_SOURCE.len(), 181);
    assert_eq!(
        TASK258B3M2B2B2C_SOURCE,
        "import parser.type_fixtures;\nreserve x for set;\ntheorem FormulaStatementStructureUpdateWitnessSmoke: x = x proof\n  take TypeCaseStruct(x: 1, y: 2) with (x := 3);\n  thus x = x;\nend;\n"
    );
    assert_eq!(fixture.arena.len(), 86);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(85)));
    for (index, (start, end)) in TASK258B3M2B2B2C_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b2c_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b2c_node_children(index)
        );
    }
    assert_eq!(
        (
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len(),
        ),
        (7, 4, 3)
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
        (2, 0, 1, 3, 1, 4, 9)
    );
    let update = fixture
        .structure
        .terms()
        .get(SourceStructureTermId::new(0))
        .expect("update");
    assert_eq!(update.site().node(), TypedNodeId::new(69));
    assert_eq!(update.kind(), SourceStructureTermKind::FunctionalUpdate);
    assert_eq!(update.source_range(), range(fixture.source, 120, 160));
    let field_update = fixture
        .structure
        .field_updates()
        .get(SourceFieldUpdateId::new(0))
        .expect("field update");
    assert_eq!(field_update.term(), SourceStructureTermId::new(0));
    assert_eq!(field_update.site().node(), TypedNodeId::new(68));
    assert_eq!(field_update.first_member(), SourceStructureMemberId::new(0));
    assert_eq!(field_update.final_member(), SourceStructureMemberId::new(0));

    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b2c_profile());
    assert!(!statement.is_task_258b3m2b2b2a_profile());
    assert!(!statement.is_task_258b3m2b2b2b_profile());
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("update witness");
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

    let constructor = B3M2B2B2AFixture::new(223);
    let constructor_statement = constructor.statement();
    let constructor_witnesses = constructor
        .witnesses(&constructor_statement, constructor.witness_input())
        .expect("legacy constructor");
    assert!(constructor_statement.is_task_258b3m2b2b2a_profile());
    assert_eq!(constructor_witnesses.application_fingerprint(), None);
    assert!(constructor_witnesses.structure_fingerprint().is_some());
    let selector = B3M2B2B2BFixture::new(224);
    let selector_statement = selector.statement();
    let selector_witnesses = selector
        .witnesses(&selector_statement, selector.witness_input())
        .expect("legacy selector");
    assert!(selector_statement.is_task_258b3m2b2b2b_profile());
    assert_eq!(selector_witnesses.application_fingerprint(), None);
    assert!(selector_witnesses.structure_fingerprint().is_some());
    assert_ne!(constructor_witnesses.debug_text(), debug);
    assert_ne!(selector_witnesses.debug_text(), debug);
    assert_eq!(witnesses.clone().debug_text(), debug);
}

#[test]
fn task258b3m2b2b2c_dependencies_structure_update_witness_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B2CFixture::new(225);
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

    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_225);
    dependency_before_aggregate.witnesses.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_before_aggregate),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut invalid_witness = fixture.witness_input();
    invalid_witness.witnesses[0].source_range.end -= 1;
    assert!(fixture.witnesses(&statement, invalid_witness).is_err());

    let (_, _, structure_symbol, structure_contribution, _) =
        structure_witness_symbol_env_with_structure_identity(
            fixture.source,
            &fixture.module,
            "FormulaStatementStructureUpdateWitnessSmoke",
            180,
            "statement.fixture.theorem.b3m2b2b2c",
            "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5",
            "parser.type_fixtures::TypeCaseStruct#5",
            vec![5],
            ExportStatus::Exported,
            false,
        );
    let mut invalid_update = B3M2B2B2CFixture::structure_input(
        fixture.source,
        &fixture.module,
        structure_symbol,
        structure_contribution,
    );
    invalid_update.field_updates[0].source_range.end -= 1;
    assert!(
        SourceStructureProducer::build(
            invalid_update,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            None,
            &fixture.arena,
        )
        .is_err()
    );
    let mut stale = baseline.clone();
    stale.structure_fingerprint = Some("stale".to_owned());
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
        Err(SourceStatementWitnessError::DependencyMismatch)
    );

    for (id, row) in fixture.arena.iter() {
        assert!(row.resolved_node.is_none(), "node {id:?} resolved_node");
        assert_eq!(row.typing, TypingState::Unknown, "node {id:?} typing");
        assert!(row.links.context.is_none(), "node {id:?} links.context");
        assert!(
            row.links.type_entry.is_none(),
            "node {id:?} links.type_entry"
        );
        assert!(row.links.facts.is_empty(), "node {id:?} links.facts");
        assert!(
            row.links.coercions.is_empty(),
            "node {id:?} links.coercions"
        );
        assert!(
            row.links.initial_obligations.is_empty(),
            "node {id:?} links.initial_obligations"
        );
        assert!(
            row.links.diagnostics.is_empty(),
            "node {id:?} links.diagnostics"
        );
    }
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
    for index in 0..86 {
        let kind_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.kind = "source.task258b3m2b2b2c.mutated".into();
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
        let recovery_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.recovery = NodeRecoveryState::Degraded;
            }
        });
        let typing_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.typing = TypingState::Successful;
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
            ("recovery", recovery_mutated),
            ("typing", typing_mutated),
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
}

#[test]
fn task258b3m2b2b2c_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B2CFixture::new(226);
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
        .expect("atomic install");
    assert_eq!(typed.source_structure(), Some(&fixture.structure));
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
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
        structure_only.with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );

    macro_rules! reject_plain_witness_family_for_b2c {
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
                concat!("B2C base then ", $label)
            );
            assert_eq!(
                typed
                    .clone()
                    .with_source_statement_witnesses(family_statement, family_witnesses),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B2C then ", $label)
            );
            assert_eq!(
                family_base.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!($label, " base then B2C")
            );
            assert_eq!(
                family_installed.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed ", $label, " then B2C")
            );
        }};
    }
    reject_plain_witness_family_for_b2c!("B3", B3Fixture::new(226));
    reject_plain_witness_family_for_b2c!("B3N", B3NFixture::new(226));
    reject_plain_witness_family_for_b2c!("B3M1", B3M1Fixture::new(226));
    reject_plain_witness_family_for_b2c!("B3M2A", B3M2AFixture::new(226));
    reject_plain_witness_family_for_b2c!("B3M2B1", B3M2B1Fixture::new(226));
    reject_plain_witness_family_for_b2c!("B3M2B2A", B3M2B2AFixture::new(226));

    macro_rules! reject_application_witness_family_for_b2c {
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
                concat!("B2C base then ", $label)
            );
            assert_eq!(
                typed.clone().with_source_application_statement_witnesses(
                    family.application.clone(),
                    family_statement,
                    family_witnesses,
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B2C then ", $label)
            );
            assert_eq!(
                family_base.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!($label, " base then B2C")
            );
            assert_eq!(
                family_installed.with_source_structure_statement_witnesses(
                    fixture.structure.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed ", $label, " then B2C")
            );
        }};
    }
    reject_application_witness_family_for_b2c!("B1A", B3M2B2B1AFixture::new(226));
    reject_application_witness_family_for_b2c!("B1B1", B3M2B2B1B1Fixture::new(226));

    let constructor = B3M2B2B2AFixture::new(227);
    let constructor_statement = constructor.statement();
    let constructor_witnesses = constructor
        .witnesses(&constructor_statement, constructor.witness_input())
        .expect("constructor witnesses");
    let selector = B3M2B2B2BFixture::new(228);
    let selector_statement = selector.statement();
    let selector_witnesses = selector
        .witnesses(&selector_statement, selector.witness_input())
        .expect("selector witnesses");
    for (label, structure, foreign_statement, foreign_witnesses) in [
        (
            "constructor",
            constructor.structure,
            constructor_statement,
            constructor_witnesses,
        ),
        (
            "selector",
            selector.structure,
            selector_statement,
            selector_witnesses,
        ),
    ] {
        assert_eq!(
            base.clone().with_source_structure_statement_witnesses(
                structure.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "{label} structure with update statement/witness"
        );
        assert_eq!(
            base.clone().with_source_structure_statement_witnesses(
                fixture.structure.clone(),
                foreign_statement,
                foreign_witnesses,
            ),
            Err(TypedAstError::InvalidSourceStatement),
            "update structure with {label} statement/witness"
        );
    }
    let references = B1Fixture::new(226);
    let reference_rows = references
        .references(references.reference_input())
        .expect("reference rows");
    let reference_installed = references
        .empty_typed()
        .with_source_statement_references(references.statement.clone(), reference_rows.clone())
        .expect("reference installed");
    assert_eq!(
        base.clone()
            .with_source_statement_references(references.statement.clone(), reference_rows,),
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
    let base_only = B2Fixture::new(226);
    let base_statement = base_only.build(base_only.input()).expect("B2 statement");
    let base_installed = base_only
        .empty_typed()
        .with_source_statement(base_statement.clone())
        .expect("B2 installed");
    assert_eq!(
        base.clone().with_source_statement(base_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base_installed.with_source_structure_statement_witnesses(
            fixture.structure.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(base.debug_text(), base_debug);
}

#[test]
fn task258b3m2b2b2c_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B2CFixture::new(229);
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

    for (label, mutate) in [
        (
            "statement fingerprint",
            (|row: &mut SourceStatementWitnessHandoff| row.statement_fingerprint.push('x'))
                as fn(&mut SourceStatementWitnessHandoff),
        ),
        ("primary fingerprint", |row| {
            row.primary_term_fingerprint.push('x')
        }),
        ("structure fingerprint", |row| {
            row.structure_fingerprint = Some("stale".to_owned())
        }),
        ("application hybrid", |row| {
            row.application_fingerprint = Some("hybrid".to_owned())
        }),
    ] {
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
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b2c-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 120, 160),
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
                expr: ExprId::new("task258b3m2b2b2c-semantic-coexistence"),
                typed_site: node(69),
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
