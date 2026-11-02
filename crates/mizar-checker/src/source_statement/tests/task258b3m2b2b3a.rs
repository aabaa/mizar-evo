use super::*;

#[test]
fn task258b3m2b2b3a_exact_set_enumeration_witness_api_debug_and_legacy_compatibility_are_stable() {
    const SOURCE: &str = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementSetEnumerationWitnessSmoke: x = x proof\n",
        "  take {1, 2};\n",
        "  thus x = x;\n",
        "end;\n",
    );
    let fixture = B3M2B2B3AFixture::new(230);
    assert_eq!(SOURCE.len(), 117);
    assert_eq!(
        SOURCE,
        "reserve x for set;\ntheorem FormulaStatementSetEnumerationWitnessSmoke: x = x proof\n  take {1, 2};\n  thus x = x;\nend;\n"
    );
    assert_eq!(fixture.arena.len(), 57);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(56)));
    for (index, (start, end)) in TASK258B3M2B2B3A_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b3a_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b3a_node_children(index)
        );
        assert!(row.resolved_node.is_none(), "node {index} resolved_node");
        assert_eq!(row.typing, TypingState::Unknown, "node {index} typing");
        assert!(row.links.context.is_none(), "node {index} context");
        assert!(row.links.type_entry.is_none(), "node {index} type");
        assert!(row.links.facts.is_empty(), "node {index} facts");
        assert!(row.links.coercions.is_empty(), "node {index} coercions");
        assert!(
            row.links.initial_obligations.is_empty(),
            "node {index} obligations"
        );
        assert!(row.links.diagnostics.is_empty(), "node {index} diagnostics");
    }
    assert_eq!(
        (
            fixture.bindings.contexts().len(),
            fixture.bindings.bindings().len(),
            fixture.bindings.diagnostics().len(),
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len(),
        ),
        (2, 1, 0, 6, 4, 2)
    );
    assert_eq!(
        (
            fixture.set_terms.terms().len(),
            fixture.set_terms.wrappers().len(),
            fixture.set_terms.generators().len(),
            fixture.set_terms.type_sites().len(),
            fixture.set_terms.conditions().len(),
            fixture.set_terms.edges().len(),
            fixture.set_terms.requests().len(),
        ),
        (1, 0, 0, 0, 0, 2, 1)
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
    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b3a_profile());
    let witnesses = fixture.witnesses(&statement).expect("set witness");
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(witnesses.structure_fingerprint(), None);
    assert_eq!(
        witnesses.set_term_fingerprint(),
        Some(fixture.set_terms.debug_text().as_str())
    );
    assert_eq!(
        witnesses
            .witnesses()
            .get(SourceStatementWitnessId::new(0))
            .expect("witness")
            .term(),
        SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
    );
    let debug = witnesses.debug_text();
    assert_eq!(
        debug,
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nset-term-fingerprint: Some({:?})\nwitness#0 owner=0 binding_context=1 term=set-term#0 take_range=85..97 take_site=43 range=90..96 site=42 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"{{ 1 , 2 }}\"\n",
            statement.debug_text(),
            fixture.primary.debug_text(),
            fixture.set_terms.debug_text(),
        )
    );
    let primary_owned = fixture
        .primary
        .terms()
        .iter()
        .map(|(_, row)| row.site().node().index())
        .collect::<Vec<_>>();
    let set_owned = fixture
        .set_terms
        .terms()
        .iter()
        .map(|(_, row)| row.site().node().index())
        .collect::<Vec<_>>();
    let atomic_owned = fixture
        .atomic
        .formulas()
        .iter()
        .map(|(_, row)| row.site().node().index())
        .collect::<Vec<_>>();
    let statement_owned = statement
        .statements()
        .iter()
        .map(|(_, row)| row.site().node().index())
        .collect::<Vec<_>>();
    let witness_owned = witnesses
        .witnesses()
        .iter()
        .flat_map(|(_, row)| [row.site().node().index(), row.take_site().node().index()])
        .collect::<Vec<_>>();
    assert_eq!(primary_owned, [30, 32, 36, 38, 44, 46]);
    assert_eq!(set_owned, [40]);
    assert_eq!(atomic_owned, [34, 48]);
    assert_eq!(statement_owned, [53, 51]);
    assert_eq!(witness_owned, [42, 43]);
    let owned = primary_owned
        .into_iter()
        .chain(set_owned)
        .chain(atomic_owned)
        .chain(statement_owned)
        .chain(witness_owned)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        (0..57)
            .filter(|node| !owned.contains(node))
            .collect::<Vec<_>>(),
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 31, 33, 35, 37, 39, 41, 45, 47, 49, 50, 52, 54, 55, 56,
        ]
    );
    let legacy = B3Fixture::new(230);
    let legacy_statement = legacy.statement();
    let legacy_witnesses = legacy
        .witnesses(&legacy_statement, legacy.witness_input())
        .expect("legacy");
    assert_eq!(legacy_witnesses.application_fingerprint(), None);
    assert_eq!(legacy_witnesses.structure_fingerprint(), None);
    assert_eq!(legacy_witnesses.set_term_fingerprint(), None);
    assert_eq!(
        legacy_witnesses.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\"\n",
            legacy_statement.debug_text(),
            legacy.primary.debug_text(),
        )
    );
}

#[test]
fn task258b3m2b2b3a_dependencies_set_enumeration_witness_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B3AFixture::new(231);
    let statement = fixture.statement();
    let witnesses = fixture.witnesses(&statement).expect("witnesses");
    witnesses
        .validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        )
        .expect("revalidation");
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            source_id(999_231),
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let foreign_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"));
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            fixture.source,
            &foreign_module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut stale_statement = witnesses.clone();
    stale_statement.statement_fingerprint.push('!');
    assert_eq!(
        stale_statement.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let foreign = B3M2B2B3AFixture::new(232);
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&foreign.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut stale_primary = witnesses.clone();
    stale_primary.primary_term_fingerprint.push('!');
    assert_eq!(
        stale_primary.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut stale_set = witnesses.clone();
    stale_set.set_term_fingerprint = Some("stale".to_owned());
    assert_eq!(
        stale_set.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            None,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
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
    for index in 0..fixture.arena.len() {
        let kind_mutated = mutate_arena(&fixture.arena, |id, row| {
            if id.index() == index {
                row.kind = "source.task258b3m2b2b3a.mutated".into();
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
            assert_eq!(
                witnesses.validate_installation_with_set_term(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    Some(&fixture.set_terms),
                    &mutated,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {field}"
            );
        }
    }
    for root in [None, Some(TypedNodeId::new(55))] {
        let wrong_root = TypedArena::try_new(
            root,
            fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
        )
        .expect("wrong-root arena remains structurally valid");
        assert_eq!(
            witnesses.validate_installation_with_set_term(
                fixture.source,
                &fixture.module,
                &statement,
                &fixture.primary,
                Some(&fixture.set_terms),
                &wrong_root,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "root {root:?}"
        );
    }

    let missing_proof_context = binding_env(fixture.source, &fixture.module);
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &missing_proof_context,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "Task48 must own its missing proof context"
    );
    for (label, bindings) in [
        (
            "Task48 module context",
            b3m2b2b3a_binding_env_with_mutation(
                fixture.source,
                &fixture.module,
                BindingContextRecovery::Degraded,
                115,
                0,
            ),
        ),
        (
            "Task48 proof context",
            b3m2b2b3a_binding_env_with_mutation(
                fixture.source,
                &fixture.module,
                BindingContextRecovery::Normal,
                114,
                0,
            ),
        ),
        (
            "Task48 binding",
            b3m2b2b3a_binding_env_with_mutation(
                fixture.source,
                &fixture.module,
                BindingContextRecovery::Normal,
                115,
                1,
            ),
        ),
    ] {
        assert_eq!(
            SourceStatementProducer::build(
                fixture.statement_input(),
                &fixture.symbols,
                &bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(SourceStatementError::DependencyMismatch),
            "{label}"
        );
    }

    for term in 0..6 {
        let mut input = B3M2B2B3AFixture::primary_input(fixture.source, &fixture.module);
        input.terms[term].spelling.push('!');
        if let Ok(primary) =
            SourcePrimaryTermProducer::build(input, &fixture.bindings, &fixture.arena)
        {
            assert_eq!(
                witnesses.validate_installation_with_set_term(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &primary,
                    Some(&fixture.set_terms),
                    &fixture.arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "Task252 term {term}"
            );
        }
    }
    for reference in 0..4 {
        let mut primary = fixture.primary.clone();
        primary
            .set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(reference), 999);
        assert_eq!(
            witnesses.validate_installation_with_set_term(
                fixture.source,
                &fixture.module,
                &statement,
                &primary,
                Some(&fixture.set_terms),
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "Task252 reference {reference}"
        );
    }
    for request in 0..2 {
        let mut input = B3M2B2B3AFixture::primary_input(fixture.source, &fixture.module);
        input.numeric_type_requests[request].spelling.push('!');
        if let Ok(primary) =
            SourcePrimaryTermProducer::build(input, &fixture.bindings, &fixture.arena)
        {
            assert_eq!(
                witnesses.validate_installation_with_set_term(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &primary,
                    Some(&fixture.set_terms),
                    &fixture.arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "Task252 numeric request {request}"
            );
        }
    }

    for (label, mutate) in [
        (
            "Task255 term",
            (|input: &mut SourceSetTermHandoffInput| input.terms[0].spelling.push('!'))
                as fn(&mut SourceSetTermHandoffInput),
        ),
        ("Task255 edge 0", |input| input.edges[0].ordinal = 1),
        ("Task255 edge 1", |input| input.edges[1].ordinal = 0),
        ("Task255 request", |input| input.requests[0].ordinal = 1),
    ] {
        let mut input = B3M2B2B3AFixture::set_input(fixture.source, &fixture.module);
        mutate(&mut input);
        if let Ok(set_terms) = SourceSetTermProducer::build(
            input,
            &fixture.bindings,
            &fixture.primary,
            None,
            None,
            &fixture.arena,
        ) {
            assert_eq!(
                witnesses.validate_installation_with_set_term(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    Some(&set_terms),
                    &fixture.arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "{label}"
            );
        }
    }

    for formula in 0..2 {
        let mut input = B3M2B2B3AFixture::atomic_input(fixture.source, &fixture.module);
        input.formulas[formula].spelling.push('!');
        if let Ok(atomic) = SourceAtomicFormulaProducer::build(
            input,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        ) {
            assert_eq!(
                SourceStatementProducer::build(
                    fixture.statement_input(),
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &atomic,
                    &fixture.arena,
                ),
                Err(SourceStatementError::DependencyMismatch),
                "Task256 formula {formula}"
            );
        }
    }
    for edge in 0..4 {
        let mut input = B3M2B2B3AFixture::atomic_input(fixture.source, &fixture.module);
        input.edges[edge].ordinal = 999;
        if let Ok(atomic) = SourceAtomicFormulaProducer::build(
            input,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        ) {
            assert_eq!(
                SourceStatementProducer::build(
                    fixture.statement_input(),
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &atomic,
                    &fixture.arena,
                ),
                Err(SourceStatementError::DependencyMismatch),
                "Task256 edge {edge}"
            );
        }
    }
    for request in 0..4 {
        let mut input = B3M2B2B3AFixture::atomic_input(fixture.source, &fixture.module);
        input.requests[request].ordinal = 999;
        if let Ok(atomic) = SourceAtomicFormulaProducer::build(
            input,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        ) {
            assert_eq!(
                SourceStatementProducer::build(
                    fixture.statement_input(),
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &atomic,
                    &fixture.arena,
                ),
                Err(SourceStatementError::DependencyMismatch),
                "Task256 request {request}"
            );
        }
    }

    for (label, mutate) in [
        (
            "Task258 owner",
            (|input: &mut SourceStatementHandoffInput| input.owners[0].source_range.end -= 1)
                as fn(&mut SourceStatementHandoffInput),
        ),
        ("Task258 statement 0", |input| {
            input.statements[0].source_ordinal = 1
        }),
        ("Task258 statement 1", |input| {
            input.statements[1].source_ordinal = 1
        }),
        ("Task258 context 0", |input| {
            input.contexts[0].source_range.end -= 1
        }),
        ("Task258 context 1", |input| {
            input.contexts[1].source_range.end -= 1
        }),
        ("Task258 input fact 0", |input| {
            input.input_facts[0].uses.reverse()
        }),
        ("Task258 input fact 1", |input| {
            input.input_facts[1].uses.reverse()
        }),
        ("Task258 candidate 0", |input| {
            input.candidate_facts[0].ordinal = 1
        }),
        ("Task258 candidate 1", |input| {
            input.candidate_facts[1].ordinal = 1
        }),
    ] {
        let mut input = fixture.statement_input();
        mutate(&mut input);
        if let Ok(mutated_statement) = SourceStatementProducer::build(
            input,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ) {
            assert_eq!(
                witnesses.validate_installation_with_set_term(
                    fixture.source,
                    &fixture.module,
                    &mutated_statement,
                    &fixture.primary,
                    Some(&fixture.set_terms),
                    &fixture.arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "{label}"
            );
        }
    }

    for (label, mutate) in [
        (
            "source",
            (|input: &mut SourceStatementWitnessHandoffInput| input.source_id = source_id(999_232))
                as fn(&mut SourceStatementWitnessHandoffInput),
        ),
        ("module", |input| {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }),
        ("owner", |input| {
            input.witnesses[0].owner = SourceTheoremOwnerId::new(1)
        }),
        ("binding context", |input| {
            input.witnesses[0].binding_context = BindingContextId::new(0)
        }),
        ("term", |input| {
            input.witnesses[0].term =
                SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(0))
        }),
        ("take site", |input| input.witnesses[0].take_site = node(42)),
        ("take range", |input| input.witnesses[0].take_range.end -= 1),
        ("site", |input| input.witnesses[0].site = node(40)),
        ("source range", |input| {
            input.witnesses[0].source_range.end -= 1
        }),
        ("source ordinal", |input| {
            input.witnesses[0].source_ordinal = 0
        }),
        ("ordinal", |input| input.witnesses[0].ordinal = 1),
        ("spelling", |input| input.witnesses[0].spelling.push('!')),
        ("kind", |input| {
            input.witnesses[0].kind = SourceStatementWitnessKind::Named
        }),
        ("recovery", |input| {
            input.witnesses[0].recovery = SourceStatementRecovery::Degraded
        }),
        ("name", |input| {
            input.witnesses[0].name = Some(SourceStatementWitnessNameId::new(0))
        }),
    ] {
        let mut input = fixture.witness_input();
        mutate(&mut input);
        assert!(
            SourceStatementWitnessProducer::build_with_set_term(
                input,
                &statement,
                &fixture.primary,
                &fixture.set_terms,
                &fixture.arena,
            )
            .is_err(),
            "witness {label}"
        );
    }
    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_233);
    dependency_before_aggregate.witnesses.clear();
    assert_eq!(
        SourceStatementWitnessProducer::build_with_set_term(
            dependency_before_aggregate,
            &statement,
            &fixture.primary,
            &fixture.set_terms,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    witnesses
        .validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        )
        .expect("clean replay after all dependency mutations");
}

#[test]
fn task258b3m2b2b3a_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B3AFixture::new(233);
    let statement = fixture.statement();
    let witnesses = fixture.witnesses(&statement).expect("witnesses");
    let base = fixture.empty_typed();
    let base_debug = base.debug_text();
    let typed = base
        .clone()
        .with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("combined install");
    assert_eq!(typed.source_set_term(), Some(&fixture.set_terms));
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_set_term(fixture.set_terms.clone())
            .expect("lower set first")
            .with_source_set_term_statement_witnesses(
                fixture.set_terms.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let application_family = B3M2B2B1AFixture::new(233);
    let application_statement = application_family.statement();
    let application_witnesses = application_family
        .witnesses(&application_statement, application_family.witness_input())
        .expect("application witnesses");
    let structure_family = B3M2B2B2CFixture::new(233);
    let structure_statement = structure_family.statement();
    let structure_witnesses = structure_family
        .witnesses(&structure_statement, structure_family.witness_input())
        .expect("structure witnesses");
    for mask in 0_u8..8 {
        let result = witnesses.validate_installation_inner(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            (mask & 1 != 0).then_some(&application_family.application),
            (mask & 2 != 0).then_some(&structure_family.structure),
            (mask & 4 != 0).then_some(&fixture.set_terms),
            &fixture.arena,
        );
        assert_eq!(
            result.is_ok(),
            mask == 4,
            "application/structure/set tuple {mask:03b}"
        );
    }
    let mut application_hybrid = witnesses.clone();
    application_hybrid.application_fingerprint = Some("hybrid".to_owned());
    assert_eq!(
        base.clone().with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            application_hybrid,
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let mut structure_hybrid = witnesses.clone();
    structure_hybrid.structure_fingerprint = Some("hybrid".to_owned());
    assert_eq!(
        base.clone().with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            structure_hybrid,
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    macro_rules! reject_plain_witness_family_for_b3a {
        ($label:literal, $family:expr) => {{
            run_in_isolated_test_stack_frame(|| {
                let family = $family;
                let family_statement = family.statement();
                let family_witnesses = family
                    .witnesses(&family_statement, family.witness_input())
                    .expect(concat!($label, " witnesses"));
                let family_base = family.empty_typed();
                let family_installed = family_base
                    .clone()
                    .with_source_statement_witnesses(
                        family_statement.clone(),
                        family_witnesses.clone(),
                    )
                    .expect(concat!($label, " installed"));
                assert_eq!(
                    base.clone().with_source_statement_witnesses(
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("B3A base then ", $label)
                );
                assert_eq!(
                    typed.clone().with_source_statement_witnesses(
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed B3A then ", $label)
                );
                assert_eq!(
                    family_base
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!($label, " base then B3A")
                );
                assert_eq!(
                    family_installed
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed ", $label, " then B3A")
                );
                assert_eq!(
                    family_base.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!($label, " base rejects set injection")
                );
                assert_eq!(
                    family_installed.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!("installed ", $label, " rejects set injection")
                );
            });
        }};
    }
    reject_plain_witness_family_for_b3a!("B3", B3Fixture::new(233));
    reject_plain_witness_family_for_b3a!("B3N", B3NFixture::new(233));
    reject_plain_witness_family_for_b3a!("B3M1", B3M1Fixture::new(233));
    reject_plain_witness_family_for_b3a!("B3M2A", B3M2AFixture::new(233));
    reject_plain_witness_family_for_b3a!("B3M2B1", B3M2B1Fixture::new(233));
    reject_plain_witness_family_for_b3a!("B3M2B2A", B3M2B2AFixture::new(233));

    macro_rules! reject_application_witness_family_for_b3a {
        ($label:literal, $family:expr) => {{
            run_in_isolated_test_stack_frame(|| {
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
                    concat!("B3A base then ", $label)
                );
                assert_eq!(
                    typed.clone().with_source_application_statement_witnesses(
                        family.application.clone(),
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed B3A then ", $label)
                );
                assert_eq!(
                    family_base
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!($label, " base then B3A")
                );
                assert_eq!(
                    family_installed
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed ", $label, " then B3A")
                );
                assert_eq!(
                    family_base.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!($label, " base rejects set injection")
                );
                assert_eq!(
                    family_installed.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!("installed ", $label, " rejects set injection")
                );
            });
        }};
    }
    reject_application_witness_family_for_b3a!("B1A", B3M2B2B1AFixture::new(233));
    reject_application_witness_family_for_b3a!("B1B1", B3M2B2B1B1Fixture::new(233));

    macro_rules! reject_structure_witness_family_for_b3a {
        ($label:literal, $family:expr) => {{
            run_in_isolated_test_stack_frame(|| {
                let family = $family;
                let family_statement = family.statement();
                let family_witnesses = family
                    .witnesses(&family_statement, family.witness_input())
                    .expect(concat!($label, " witnesses"));
                let family_base = family.empty_typed();
                let family_installed = family_base
                    .clone()
                    .with_source_structure_statement_witnesses(
                        family.structure.clone(),
                        family_statement.clone(),
                        family_witnesses.clone(),
                    )
                    .expect(concat!($label, " installed"));
                assert_eq!(
                    base.clone().with_source_structure_statement_witnesses(
                        family.structure.clone(),
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("B3A base then ", $label)
                );
                assert_eq!(
                    typed.clone().with_source_structure_statement_witnesses(
                        family.structure.clone(),
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed B3A then ", $label)
                );
                assert_eq!(
                    family_base
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!($label, " base then B3A")
                );
                assert_eq!(
                    family_installed
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed ", $label, " then B3A")
                );
                assert_eq!(
                    family_base.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!($label, " base rejects set injection")
                );
                assert_eq!(
                    family_installed.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!("installed ", $label, " rejects set injection")
                );
            });
        }};
    }
    reject_structure_witness_family_for_b3a!("B2A", B3M2B2B2AFixture::new(233));
    reject_structure_witness_family_for_b3a!("B2B", B3M2B2B2BFixture::new(233));
    reject_structure_witness_family_for_b3a!("B2C", B3M2B2B2CFixture::new(233));

    let references = B1Fixture::new(233);
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
        reference_installed.with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let base_only = B2Fixture::new(233);
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
        base_installed.with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.clone().with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        application_family
            .empty_typed()
            .with_source_structure_statement_witnesses(
                structure_family.structure.clone(),
                structure_statement,
                structure_witnesses,
            ),
        Err(TypedAstError::InvalidSourceStatement),
        "application + structure tuple"
    );
    assert_eq!(
        typed.with_source_application_statement_witnesses(
            application_family.application,
            application_statement,
            application_witnesses,
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "set + application tuple"
    );
    assert_eq!(base.debug_text(), base_debug);
}

#[test]
fn task258b3m2b2b3a_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B3AFixture::new(234);
    let statement = fixture.statement();
    let witnesses = fixture.witnesses(&statement).expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("typed");
    let typed_debug = typed.debug_text();
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
    let resolved_debug = resolved.debug_text();
    assert_eq!(resolved.source_set_term(), Some(&fixture.set_terms));
    assert_eq!(resolved.source_statement(), Some(&statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
    assert!(resolved.source_application().is_none());
    assert!(resolved.source_structure().is_none());
    assert!(resolved.source_statement_references().is_none());
    assert!(resolved.source_context().is_none());
    assert!(resolved.source_type().is_none());
    assert!(resolved.source_attribute().is_none());
    assert!(resolved.source_evidence().is_none());
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
        ("missing set fingerprint", |row| {
            row.set_term_fingerprint = None
        }),
        ("stale set fingerprint", |row| {
            row.set_term_fingerprint = Some("stale".to_owned())
        }),
        ("application hybrid", |row| {
            row.application_fingerprint = Some("hybrid".to_owned())
        }),
        ("structure hybrid", |row| {
            row.structure_fingerprint = Some("hybrid".to_owned())
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

    let foreign = B3M2B2B3AFixture::new(235);
    let mut invalid_lower_pair = typed.clone();
    invalid_lower_pair.replace_source_set_term_and_atomic_formula_for_test(
        foreign.set_terms.clone(),
        foreign.atomic.clone(),
    );
    assert_eq!(
        assemble_empty_resolved(&invalid_lower_pair),
        Err(ResolvedTypedAstError::InvalidSourceSetTerm),
        "set-term validation must precede atomic-formula validation"
    );
    let mut invalid_atomic = typed.clone();
    invalid_atomic.replace_source_set_term_and_atomic_formula_for_test(
        fixture.set_terms.clone(),
        foreign.atomic,
    );
    assert_eq!(
        assemble_empty_resolved(&invalid_atomic),
        Err(ResolvedTypedAstError::InvalidSourceAtomicFormula)
    );

    let mut witness_without_statement = fixture
        .empty_typed()
        .with_source_set_term(fixture.set_terms.clone())
        .expect("set term");
    witness_without_statement.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&witness_without_statement),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut statement_without_witness = fixture
        .empty_typed()
        .with_source_set_term(fixture.set_terms.clone())
        .expect("set term");
    statement_without_witness.inject_source_statement_for_test(statement.clone());
    assert_eq!(
        assemble_empty_resolved(&statement_without_witness),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut bundle_without_set = fixture.empty_typed();
    bundle_without_set
        .inject_source_statement_witness_bundle_for_test(statement.clone(), witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&bundle_without_set),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut orphan_witness = fixture.empty_typed();
    orphan_witness.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&orphan_witness),
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
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b3a-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 90, 96),
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
                expr: ExprId::new("task258b3m2b2b3a-semantic-coexistence"),
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
