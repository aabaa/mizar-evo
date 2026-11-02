use super::*;

#[test]
fn task258b3m2b2b3d_exact_qua_witness_profile_and_legacy_compatibility_are_stable() {
    const SOURCE: &str = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementQuaWitnessSmoke: x = x proof\n",
        "  take 4 qua set;\n",
        "  thus x = x;\n",
        "end;\n",
    );
    let fixture = B3M2B2B3DFixture::new(241);
    assert_eq!(SOURCE.len(), 109);
    assert_eq!(
        SOURCE,
        "reserve x for set;\ntheorem FormulaStatementQuaWitnessSmoke: x = x proof\n  take 4 qua set;\n  thus x = x;\nend;\n"
    );
    assert_eq!(fixture.arena.len(), 54);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(53)));
    for (index, (start, end)) in TASK258B3M2B2B3D_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b3d_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b3d_node_children(index)
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
        (1, 0, 0, 1, 0, 1, 2)
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
    assert!(statement.is_task_258b3m2b2b3d_profile());
    assert!(!statement.is_task_258b3m2b2b3a_profile());
    assert!(!statement.is_task_258b3m2b2b3b_profile());
    assert!(!statement.is_task_258b3m2b2b3c_profile());
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    assert_eq!(
        statement.checked_owner().origin().anchor(),
        &SourceAnchor::Range(range(fixture.source, 19, 108))
    );
    let witnesses = fixture.witnesses(&statement).expect("qua witness");
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(witnesses.structure_fingerprint(), None);
    assert_eq!(
        witnesses.set_term_fingerprint(),
        Some(fixture.set_terms.debug_text().as_str())
    );
    assert_eq!(
        witnesses.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nset-term-fingerprint: Some({:?})\nwitness#0 owner=0 binding_context=1 term=set-term#0 take_range=74..89 take_site=40 range=79..88 site=39 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"4 qua set\"\n",
            statement.debug_text(),
            fixture.primary.debug_text(),
            fixture.set_terms.debug_text(),
        )
    );
    let owned =
        fixture
            .primary
            .terms()
            .iter()
            .map(|(_, row)| row.site().node().index())
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
        [28, 30, 32, 34, 35, 36, 37, 39, 40, 41, 43, 45, 48, 50]
            .into_iter()
            .collect()
    );
    assert_eq!(
        (0..54)
            .filter(|index| !owned.contains(index))
            .collect::<Vec<_>>(),
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 29, 31, 33, 38, 42, 44, 46, 47, 49, 51, 52, 53,
        ]
    );

    for family in ["B3A", "B3B", "B3C"] {
        match family {
            "B3A" => {
                let legacy = B3M2B2B3AFixture::new(241);
                legacy
                    .witnesses(&legacy.statement())
                    .expect("nonempty enumeration remains admitted");
            }
            "B3B" => {
                let legacy = B3M2B2B3BFixture::new(241);
                legacy
                    .witnesses(&legacy.statement())
                    .expect("empty enumeration remains admitted");
            }
            "B3C" => {
                let legacy = B3M2B2B3CFixture::new(241);
                legacy
                    .witnesses(&legacy.statement())
                    .expect("choice remains admitted");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn task258b3m2b2b3d_dependencies_qua_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B3DFixture::new(242);
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
            source_id(999_242),
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
                        row.kind = "source.task258b3m2b2b3d.mutated".into();
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
    for root in [None, Some(TypedNodeId::new(52))] {
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
    const TASK255_FIELDS: usize = 44;
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
        (32, 70, 44, 72, 62, 21)
    );

    let mut task48_own_stage = 0;
    let mut task48_generic_guard = 0;
    for field in 0..TASK48_FIELDS {
        match task258b3m2b2b3d_mutated_binding_env(&fixture.bindings, field) {
            Err(_) => task48_own_stage += 1,
            Ok(bindings) => {
                assert_ne!(
                    bindings.debug_text(),
                    fixture.bindings.debug_text(),
                    "Task48 handoff delta"
                );
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
                task48_generic_guard += 1;
            }
        }
        validate(&fixture.arena).expect("Task48 replay");
    }
    assert_eq!(task48_own_stage + task48_generic_guard, TASK48_FIELDS);

    let mut task252_own_stage = 0;
    let mut task252_generic_guard = 0;
    for field in 0..TASK252_FIELDS {
        let mut input = B3M2B2B3DFixture::primary_input(fixture.source, &fixture.module);
        task258b3m2b2b3d_mutate_primary_input(&mut input, field);
        match SourcePrimaryTermProducer::build(input, &fixture.bindings, &fixture.arena) {
            Err(_) => task252_own_stage += 1,
            Ok(primary) => {
                assert_ne!(
                    primary.debug_text(),
                    fixture.primary.debug_text(),
                    "Task252 handoff delta"
                );
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
                task252_generic_guard += 1;
            }
        }
        validate(&fixture.arena).expect("Task252 replay");
    }
    assert_eq!(task252_own_stage + task252_generic_guard, TASK252_FIELDS);

    type Task255Mutation = (&'static str, fn(&mut SourceSetTermHandoffInput));
    let task255_mutations: [Task255Mutation; 44] = [
        ("envelope source", |input| {
            input.source_id = source_id(999_255)
        }),
        ("envelope module", |input| {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }),
        ("term missing", |input| input.terms.clear()),
        ("term duplicate", |input| {
            input.terms.push(input.terms[0].clone())
        }),
        ("term site", |input| input.terms[0].site = node(36)),
        ("term range", |input| input.terms[0].source_range.end -= 1),
        ("term source ordinal", |input| {
            input.terms[0].source_ordinal = 1
        }),
        ("term context", |input| {
            input.terms[0].context = BindingContextId::new(0)
        }),
        ("term recovery", |input| {
            input.terms[0].recovery = SourceSetTermRecovery::Degraded
        }),
        ("term spelling", |input| input.terms[0].spelling.push('!')),
        ("term kind", |input| {
            input.terms[0].kind = SourceSetTermKind::Choice
        }),
        ("wrapper present", |input| {
            input.wrappers.push(SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(38),
                source_range: input.terms[0].source_range,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "4 qua set".to_owned(),
            })
        }),
        ("generator present", |input| {
            input.generators.push(SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(36),
                source_range: input.type_sites[0].source_range,
                spelling: "set".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            })
        }),
        ("type site missing", |input| input.type_sites.clear()),
        ("type site duplicate", |input| {
            input.type_sites.push(input.type_sites[0].clone())
        }),
        ("type owner", |input| {
            input.type_sites[0].owner = SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::ChoiceTarget,
            }
        }),
        ("type site", |input| input.type_sites[0].site = node(35)),
        ("type range", |input| {
            input.type_sites[0].source_range.end -= 1
        }),
        ("type spelling", |input| {
            input.type_sites[0].spelling.push('!')
        }),
        ("type head site", |input| {
            input.type_sites[0].head_site = node(36)
        }),
        ("type head range", |input| {
            input.type_sites[0].head_range.end -= 1
        }),
        ("type head spelling", |input| {
            input.type_sites[0].head_spelling.push('!')
        }),
        ("type context", |input| {
            input.type_sites[0].context = BindingContextId::new(0)
        }),
        ("type recovery", |input| {
            input.type_sites[0].recovery = SourceSetTermRecovery::Degraded
        }),
        ("type head", |input| {
            input.type_sites[0].head = SourceSetTypeHead::BuiltinObject
        }),
        ("condition present", |input| {
            input.conditions.push(SourceSetConditionInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                colon_site: node(14),
                colon_range: input.terms[0].source_range,
                colon_spelling: ":".to_owned(),
                condition_site: node(32),
                source_range: input.terms[0].source_range,
                spelling: "x = x".to_owned(),
                recovery: SourceSetTermRecovery::Normal,
            })
        }),
        ("edge missing", |input| input.edges.clear()),
        ("edge duplicate", |input| {
            input.edges.push(input.edges[0].clone())
        }),
        ("edge term", |input| {
            input.edges[0].term = SourceSetTermId::new(1)
        }),
        ("edge ordinal", |input| input.edges[0].ordinal = 1),
        ("edge role", |input| {
            input.edges[0].role = SourceSetEdgeRole::EnumerationElement
        }),
        ("edge target", |input| {
            input.edges[0].target = SourceSetTarget::Primary(SourcePrimaryTermId::new(0))
        }),
        ("request missing", |input| {
            input.requests.pop();
        }),
        ("request duplicate", |input| {
            input.requests.push(input.requests[1].clone())
        }),
        ("request0 term", |input| {
            input.requests[0].term = SourceSetTermId::new(1)
        }),
        ("request0 ordinal", |input| input.requests[0].ordinal = 1),
        ("request0 kind", |input| {
            input.requests[0].kind = SourceSetRequestKind::ResultType
        }),
        ("request0 generator", |input| {
            input.requests[0].generator = Some(crate::source_set_term::SourceSetGeneratorId::new(0))
        }),
        ("request0 type", |input| input.requests[0].type_site = None),
        ("request1 term", |input| {
            input.requests[1].term = SourceSetTermId::new(1)
        }),
        ("request1 ordinal", |input| input.requests[1].ordinal = 0),
        ("request1 kind", |input| {
            input.requests[1].kind = SourceSetRequestKind::QuaWidening
        }),
        ("request1 generator", |input| {
            input.requests[1].generator = Some(crate::source_set_term::SourceSetGeneratorId::new(0))
        }),
        ("request1 type", |input| {
            input.requests[1].type_site = Some(SourceSetTypeSiteId::new(0))
        }),
    ];
    assert_eq!(task255_mutations.len(), TASK255_FIELDS);
    let mut task255_own_stage = 0;
    let mut task255_generic_guard = 0;
    for (label, mutate) in task255_mutations {
        let mut input = B3M2B2B3DFixture::set_input(fixture.source, &fixture.module);
        mutate(&mut input);
        match SourceSetTermProducer::build(
            input,
            &fixture.bindings,
            &fixture.primary,
            None,
            None,
            &fixture.arena,
        ) {
            Err(_) => task255_own_stage += 1,
            Ok(set_terms) => {
                assert_ne!(
                    set_terms.debug_text(),
                    fixture.set_terms.debug_text(),
                    "Task255 handoff delta"
                );
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
                    "Task255 {label}"
                );
                task255_generic_guard += 1;
            }
        }
        validate(&fixture.arena).expect("Task255 replay");
    }
    assert_eq!(task255_own_stage + task255_generic_guard, TASK255_FIELDS);

    let mut task256_own_stage = 0;
    let mut task256_generic_guard = 0;
    for field in 0..TASK256_FIELDS {
        let mut input = B3M2B2B3DFixture::atomic_input(fixture.source, &fixture.module);
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
            Err(_) => task256_own_stage += 1,
            Ok(atomic) => {
                assert_ne!(
                    atomic.debug_text(),
                    fixture.atomic.debug_text(),
                    "Task256 handoff delta"
                );
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
                task256_generic_guard += 1;
            }
        }
        validate(&fixture.arena).expect("Task256 replay");
    }
    assert_eq!(task256_own_stage + task256_generic_guard, TASK256_FIELDS);

    let mut task258_own_stage = 0;
    let mut task258_generic_guard = 0;
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
            Err(_) => task258_own_stage += 1,
            Ok(mutated_statement) => {
                assert_ne!(
                    mutated_statement.debug_text(),
                    statement.debug_text(),
                    "Task258 handoff delta"
                );
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
                task258_generic_guard += 1;
            }
        }
        validate(&fixture.arena).expect("Task258 replay");
    }
    assert_eq!(task258_own_stage + task258_generic_guard, TASK258_FIELDS);

    let mut witness_own_stage = 0;
    let mut witness_generic_guard = 0;
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
            Err(_) => witness_own_stage += 1,
            Ok(mutated_witnesses) => {
                assert_ne!(
                    mutated_witnesses.debug_text(),
                    witnesses.debug_text(),
                    "witness handoff delta"
                );
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
                    "witness field {field}"
                );
                witness_generic_guard += 1;
            }
        }
        validate(&fixture.arena).expect("witness replay");
    }
    assert_eq!(witness_own_stage + witness_generic_guard, WITNESS_FIELDS);
    assert!(
        task48_own_stage
            + task252_own_stage
            + task255_own_stage
            + task256_own_stage
            + task258_own_stage
            + witness_own_stage
            > 0,
        "at least one malformed mutation must fail at its owning stage"
    );
    assert!(
        task48_generic_guard
            + task252_generic_guard
            + task255_generic_guard
            + task256_generic_guard
            + task258_generic_guard
            + witness_generic_guard
            > 0,
        "at least one well-formed mutation must reach the generic guard"
    );
    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_243);
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
fn task258b3m2b2b3d_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B3DFixture::new(243);
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
    let application_family = B3M2B2B1AFixture::new(243);
    let structure_family = B3M2B2B2CFixture::new(243);
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
                concat!("B3D base then ", $label)
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
                concat!($label, " base then B3D")
            );
            assert_eq!(
                typed.clone().with_source_set_term_statement_witnesses(
                    family.set_terms.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B3D then ", $label)
            );
            assert_eq!(
                family_typed.with_source_set_term_statement_witnesses(
                    fixture.set_terms.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed ", $label, " then B3D")
            );
        }};
    }
    reject_set_family_pair!("B3A", B3M2B2B3AFixture::new(243));
    reject_set_family_pair!("B3B", B3M2B2B3BFixture::new(243));
    reject_set_family_pair!("B3C", B3M2B2B3CFixture::new(243));
    assert_eq!(base.debug_text(), base_debug);
}

#[test]
fn task258b3m2b2b3d_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B3DFixture::new(244);
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
    let foreign = B3M2B2B3DFixture::new(245);
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
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b3d-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 79, 88),
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
                expr: ExprId::new("task258b3m2b2b3d-semantic-coexistence"),
                typed_site: node(37),
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
