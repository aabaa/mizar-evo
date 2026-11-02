use super::*;

#[test]
fn task258b3m2b2b3e_exact_comprehension_witness_profile_and_legacy_compatibility_are_stable() {
    const SOURCE: &str = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementComprehensionWitnessSmoke: x = x proof\n",
        "  take {3 where candidate255 is set};\n",
        "  thus x = x;\n",
        "end;\n",
    );
    let fixture = B3M2B2B3EFixture::new(246);
    assert_eq!(SOURCE.len(), 139);
    assert_eq!(SOURCE.as_bytes().last(), Some(&b'\n'));
    assert_eq!(fixture.arena.len(), 60);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(59)));
    for (index, (start, end)) in TASK258B3M2B2B3E_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b3e_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b3e_node_children(index)
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
        (2, 1, 0, 5, 4, 1)
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
        (1, 0, 1, 1, 0, 1, 2)
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
    assert_eq!(fixture.set_terms.conditions().len(), 0);
    let generator = fixture
        .set_terms
        .generators()
        .get(SourceSetGeneratorId::new(0))
        .expect("generator");
    assert_eq!(generator.site().node().index(), 16);
    assert_eq!(generator.spelling(), "candidate255");
    assert_eq!(fixture.bindings.bindings().len(), 1);
    assert_eq!(fixture.primary.references().len(), 4);

    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b3e_profile());
    assert!(!statement.is_task_258b3m2b2b3a_profile());
    assert!(!statement.is_task_258b3m2b2b3b_profile());
    assert!(!statement.is_task_258b3m2b2b3c_profile());
    assert!(!statement.is_task_258b3m2b2b3d_profile());
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    assert_eq!(
        statement.checked_owner().origin().anchor(),
        &SourceAnchor::Range(range(fixture.source, 19, 138))
    );
    let witnesses = fixture
        .witnesses(&statement)
        .expect("comprehension witness");
    assert_eq!(witnesses.witnesses().len(), 1);
    assert!(witnesses.names().is_empty());
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(witnesses.structure_fingerprint(), None);
    assert_eq!(
        witnesses.set_term_fingerprint(),
        Some(fixture.set_terms.debug_text().as_str())
    );

    let owned =
        fixture
            .primary
            .terms()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .chain(
                fixture
                    .set_terms
                    .generators()
                    .iter()
                    .map(|(_, row)| row.site().node().index()),
            )
            .chain(
                fixture.set_terms.type_sites().iter().flat_map(|(_, row)| {
                    [row.head_site().node().index(), row.site().node().index()]
                }),
            )
            .chain(
                fixture
                    .set_terms
                    .terms()
                    .iter()
                    .map(|(_, row)| row.site().node().index()),
            )
            .chain(
                fixture
                    .atomic
                    .formulas()
                    .iter()
                    .map(|(_, row)| row.site().node().index()),
            )
            .chain(
                statement
                    .statements()
                    .iter()
                    .map(|(_, row)| row.site().node().index()),
            )
            .chain(
                witnesses.witnesses().iter().flat_map(|(_, row)| {
                    [row.site().node().index(), row.take_site().node().index()]
                }),
            )
            .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        owned,
        [16, 32, 34, 36, 38, 40, 41, 43, 45, 46, 47, 49, 51, 54, 56]
            .into_iter()
            .collect()
    );
    assert!(!owned.contains(&42));
    assert_eq!(
        (0..60)
            .filter(|index| !owned.contains(index))
            .collect::<Vec<_>>(),
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 33, 35, 37, 39, 42, 44, 48, 50, 52, 53, 55, 57, 58, 59,
        ]
    );

    let legacy_a = B3M2B2B3AFixture::new(246);
    legacy_a
        .witnesses(&legacy_a.statement())
        .expect("nonempty enumeration remains admitted");
    let legacy_b = B3M2B2B3BFixture::new(246);
    legacy_b
        .witnesses(&legacy_b.statement())
        .expect("empty enumeration remains admitted");
    let legacy_c = B3M2B2B3CFixture::new(246);
    legacy_c
        .witnesses(&legacy_c.statement())
        .expect("choice remains admitted");
    let legacy_d = B3M2B2B3DFixture::new(246);
    legacy_d
        .witnesses(&legacy_d.statement())
        .expect("qua remains admitted");
}

#[test]
fn task258b3m2b2b3e_dependencies_generator_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B3EFixture::new(247);
    let statement = fixture.statement();
    let witnesses = fixture.witnesses(&statement).expect("witnesses");
    let validate = |arena: &TypedArena| {
        witnesses.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            arena,
        )
    };
    validate(&fixture.arena).expect("clean dependency tuple");
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            source_id(999_247),
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
    let mut stale = witnesses.clone();
    stale.set_term_fingerprint = Some("stale".to_owned());
    assert_eq!(
        stale.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );

    for index in 0..fixture.arena.len() {
        let mutations = [
            (
                "kind",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.kind = "source.task258b3m2b2b3e.mutated".into();
                    }
                }),
            ),
            (
                "anchor",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
                    }
                }),
            ),
            (
                "recovery",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.recovery = NodeRecoveryState::Degraded;
                    }
                }),
            ),
            (
                "children",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        if row.children.is_empty() {
                            row.children.push(TypedNodeId::new(usize::from(index == 0)));
                        } else {
                            row.children.pop();
                        }
                    }
                }),
            ),
        ];
        for (field, mutated) in mutations {
            assert_eq!(
                validate(&mutated),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {field}"
            );
        }
    }
    for root in [None, Some(TypedNodeId::new(58))] {
        let wrong_root = TypedArena::try_new(
            root,
            fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
        )
        .expect("wrong-root arena remains structurally valid");
        assert_eq!(
            validate(&wrong_root),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "root {root:?}"
        );
    }

    const TASK48_FIELDS: usize = 32;
    const TASK252_FIELDS: usize = 70;
    const TASK255_FIELDS: usize = 53;
    const TASK256_FIELDS: usize = 72;
    const TASK258_FIELDS: usize = 62;
    const WITNESS_FIELDS: usize = 21;
    assert_eq!(
        (
            TASK48_FIELDS,
            TASK252_FIELDS,
            TASK255_FIELDS,
            TASK256_FIELDS,
            TASK258_FIELDS,
            WITNESS_FIELDS,
        ),
        (32, 70, 53, 72, 62, 21)
    );

    let mut own_stage = [0_usize; 6];
    let mut generic_guard = [0_usize; 6];
    for field in 0..TASK48_FIELDS {
        match task258b3m2b2b3d_mutated_binding_env(&fixture.bindings, field) {
            Err(_) => own_stage[0] += 1,
            Ok(bindings) => {
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
                    "Task48 field {field}"
                );
                generic_guard[0] += 1;
            }
        }
        validate(&fixture.arena).expect("Task48 replay");
    }
    for field in 0..TASK252_FIELDS {
        let mut input = B3M2B2B3EFixture::primary_input(fixture.source, &fixture.module);
        task258b3m2b2b3d_mutate_primary_input(&mut input, field);
        match SourcePrimaryTermProducer::build(input, &fixture.bindings, &fixture.arena) {
            Err(_) => own_stage[1] += 1,
            Ok(primary) => {
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
                    "Task252 field {field}"
                );
                generic_guard[1] += 1;
            }
        }
        validate(&fixture.arena).expect("Task252 replay");
    }
    for field in 0..TASK255_FIELDS {
        let mut input = B3M2B2B3EFixture::set_input(fixture.source, &fixture.module);
        task258b3m2b2b3e_mutate_set_input(&mut input, field);
        match SourceSetTermProducer::build(
            input,
            &fixture.bindings,
            &fixture.primary,
            None,
            None,
            &fixture.arena,
        ) {
            Err(_) => own_stage[2] += 1,
            Ok(set_terms) => {
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
                    "Task255 field {field}"
                );
                generic_guard[2] += 1;
            }
        }
        validate(&fixture.arena).expect("Task255 replay");
    }
    for field in 0..TASK256_FIELDS {
        let mut input = B3M2B2B3EFixture::atomic_input(fixture.source, &fixture.module);
        task258b3m2b2b3d_mutate_atomic_input(
            &mut input,
            &fixture.symbol,
            fixture.contribution,
            field,
        );
        match SourceAtomicFormulaProducer::build(
            input,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        ) {
            Err(_) => own_stage[3] += 1,
            Ok(atomic) => {
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
                    "Task256 field {field}"
                );
                generic_guard[3] += 1;
            }
        }
        validate(&fixture.arena).expect("Task256 replay");
    }
    for field in 0..TASK258_FIELDS {
        let mut input = fixture.statement_input();
        task258b3m2b2b3d_mutate_statement_input(&mut input, field);
        match SourceStatementProducer::build(
            input,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ) {
            Err(_) => own_stage[4] += 1,
            Ok(mutated_statement) => {
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
                    "Task258 field {field}"
                );
                generic_guard[4] += 1;
            }
        }
        validate(&fixture.arena).expect("Task258 replay");
    }
    for field in 0..WITNESS_FIELDS {
        let mut input = fixture.witness_input();
        task258b3m2b2b3d_mutate_witness_input(&mut input, field);
        match SourceStatementWitnessProducer::build_with_set_term(
            input,
            &statement,
            &fixture.primary,
            &fixture.set_terms,
            &fixture.arena,
        ) {
            Err(_) => own_stage[5] += 1,
            Ok(mutated_witnesses) => {
                assert_eq!(
                    mutated_witnesses.validate_installation_with_set_term(
                        fixture.source,
                        &fixture.module,
                        &statement,
                        &fixture.primary,
                        Some(&fixture.set_terms),
                        &fixture.arena,
                    ),
                    Err(SourceStatementWitnessError::DependencyMismatch),
                    "B3E witness field {field}"
                );
                generic_guard[5] += 1;
            }
        }
        validate(&fixture.arena).expect("B3E replay");
    }
    assert_eq!(
        own_stage
            .iter()
            .zip(generic_guard.iter())
            .map(|(own, generic)| own + generic)
            .collect::<Vec<_>>(),
        [
            TASK48_FIELDS,
            TASK252_FIELDS,
            TASK255_FIELDS,
            TASK256_FIELDS,
            TASK258_FIELDS,
            WITNESS_FIELDS,
        ]
    );
    assert!(
        own_stage.iter().sum::<usize>() > 0,
        "at least one mutation must fail at its owning stage"
    );
    assert!(
        generic_guard.iter().sum::<usize>() > 0,
        "at least one well-formed mutation must reach the B3E guard"
    );

    for field in [11, 12, 13, 21, 23, 34, 35, 36, 39, 40, 41, 42] {
        let mut input = B3M2B2B3EFixture::set_input(fixture.source, &fixture.module);
        task258b3m2b2b3e_mutate_set_input(&mut input, field);
        if let Ok(near_miss) = SourceSetTermProducer::build(
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
                    Some(&near_miss),
                    &fixture.arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "post-auth near miss {field}"
            );
        }
        validate(&fixture.arena).expect("near-miss replay");
    }
    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_248);
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
}

#[test]
fn task258b3m2b2b3e_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B3EFixture::new(248);
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
    let application_family = B3M2B2B1AFixture::new(248);
    let structure_family = B3M2B2B2CFixture::new(248);
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

    macro_rules! reject_set_family_pair {
        ($label:literal, $family:expr) => {{
            let family = $family;
            let family_statement = family.statement();
            let family_witnesses = family
                .witnesses(&family_statement)
                .expect(concat!($label, " witnesses"));
            let family_base = family.empty_typed();
            let family_typed = family_base
                .clone()
                .with_source_set_term_statement_witnesses(
                    family.set_terms.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                )
                .expect(concat!($label, " installed"));
            assert_eq!(
                base.clone().with_source_set_term_statement_witnesses(
                    family.set_terms.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("B3E base then ", $label)
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
                concat!($label, " base then B3E")
            );
            assert_eq!(
                typed.clone().with_source_set_term_statement_witnesses(
                    family.set_terms.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B3E then ", $label)
            );
            assert_eq!(
                family_typed.with_source_set_term_statement_witnesses(
                    fixture.set_terms.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed ", $label, " then B3E")
            );
        }};
    }
    reject_set_family_pair!("B3A", B3M2B2B3AFixture::new(248));
    reject_set_family_pair!("B3B", B3M2B2B3BFixture::new(248));
    reject_set_family_pair!("B3C", B3M2B2B3CFixture::new(248));
    reject_set_family_pair!("B3D", B3M2B2B3DFixture::new(248));

    fn permutations(families: &mut [usize; 5], start: usize, output: &mut Vec<[usize; 5]>) {
        if start == families.len() {
            output.push(*families);
            return;
        }
        for index in start..families.len() {
            families.swap(start, index);
            permutations(families, start + 1, output);
            families.swap(start, index);
        }
    }
    fn install_family(family: usize, source_ordinal: usize) {
        match family {
            0 => {
                let fixture = B3M2B2B3AFixture::new(source_ordinal);
                let statement = fixture.statement();
                let witnesses = fixture.witnesses(&statement).expect("B3A witnesses");
                fixture
                    .empty_typed()
                    .with_source_set_term_statement_witnesses(
                        fixture.set_terms,
                        statement,
                        witnesses,
                    )
                    .expect("B3A install");
            }
            1 => {
                let fixture = B3M2B2B3BFixture::new(source_ordinal);
                let statement = fixture.statement();
                let witnesses = fixture.witnesses(&statement).expect("B3B witnesses");
                fixture
                    .empty_typed()
                    .with_source_set_term_statement_witnesses(
                        fixture.set_terms,
                        statement,
                        witnesses,
                    )
                    .expect("B3B install");
            }
            2 => {
                let fixture = B3M2B2B3CFixture::new(source_ordinal);
                let statement = fixture.statement();
                let witnesses = fixture.witnesses(&statement).expect("B3C witnesses");
                fixture
                    .empty_typed()
                    .with_source_set_term_statement_witnesses(
                        fixture.set_terms,
                        statement,
                        witnesses,
                    )
                    .expect("B3C install");
            }
            3 => {
                let fixture = B3M2B2B3DFixture::new(source_ordinal);
                let statement = fixture.statement();
                let witnesses = fixture.witnesses(&statement).expect("B3D witnesses");
                fixture
                    .empty_typed()
                    .with_source_set_term_statement_witnesses(
                        fixture.set_terms,
                        statement,
                        witnesses,
                    )
                    .expect("B3D install");
            }
            4 => {
                let fixture = B3M2B2B3EFixture::new(source_ordinal);
                let statement = fixture.statement();
                let witnesses = fixture.witnesses(&statement).expect("B3E witnesses");
                fixture
                    .empty_typed()
                    .with_source_set_term_statement_witnesses(
                        fixture.set_terms,
                        statement,
                        witnesses,
                    )
                    .expect("B3E install");
            }
            _ => unreachable!(),
        }
    }
    let mut orders = Vec::new();
    permutations(&mut [0, 1, 2, 3, 4], 0, &mut orders);
    assert_eq!(orders.len(), 120);
    assert_eq!(
        orders
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        120
    );
    for (order_index, order) in orders.into_iter().enumerate() {
        for (position, family) in order.into_iter().enumerate() {
            install_family(family, 10_000 + order_index * 5 + position);
        }
    }
    assert_eq!(base.debug_text(), base_debug);
}

#[test]
fn task258b3m2b2b3e_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B3EFixture::new(249);
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
    let foreign = B3M2B2B3EFixture::new(250);
    let mut invalid_lower_pair = typed.clone();
    invalid_lower_pair
        .replace_source_set_term_and_atomic_formula_for_test(foreign.set_terms, foreign.atomic);
    assert_eq!(
        assemble_empty_resolved(&invalid_lower_pair),
        Err(ResolvedTypedAstError::InvalidSourceSetTerm)
    );
    let mut bundle_without_set = fixture.empty_typed();
    bundle_without_set
        .inject_source_statement_witness_bundle_for_test(statement.clone(), witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&bundle_without_set),
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
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b3e-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 89, 118),
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
                expr: ExprId::new("task258b3m2b2b3e-semantic-coexistence"),
                typed_site: node(43),
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
