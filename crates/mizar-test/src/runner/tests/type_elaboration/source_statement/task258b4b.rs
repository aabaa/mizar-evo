#[test]
fn task258b4b_real_route_publishes_exact_connective_grouping_statement_root() {
    use mizar_checker::{
        source_composite_formula::{
            SourceCompositeFormulaId, SourceFormulaRootId, SourceFormulaRootOwnership,
        },
        source_statement::{
            SourceStatementCandidateFactKind,
            SourceStatementContextId, SourceStatementFormulaTarget, SourceStatementId,
            SourceStatementKind, SourceStatementRecovery, SourceTheoremOwnerId, SourceTheoremRole,
            SourceTheoremStatus,
        },
    };

    assert_eq!(SOURCE_STATEMENT_B4B_TEXT.len(), 167);
    assert_eq!(&SOURCE_STATEMENT_B4B_TEXT.as_bytes()[165..], &[10, 10]);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B4B_TEXT),
        "3145e60413841ae005977400f1acd21f0974c7bad635f37fe3df6eeae7700748"
    );
    let (ast, module, symbols, output) = task258b4b_output(266_000);
    assert_eq!(ast.nodes().len(), 124);
    assert_eq!(ast.root().expect("surface root").index(), 123);
    assert!(task258b4b_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B4B_TEXT,
        SourceStatementB4BSurfaceMutation::None,
    ));
    for index in 0..124 {
        for mutation in [
            SourceStatementB4BSurfaceMutation::NodeKind(index),
            SourceStatementB4BSurfaceMutation::NodeRange(index),
            SourceStatementB4BSurfaceMutation::NodeRecovery(index),
            SourceStatementB4BSurfaceMutation::NodeChildren(index),
        ] {
            assert!(
                !task258b4b_surface_profile_with_mutation_for_test(
                    &ast,
                    SOURCE_STATEMENT_B4B_TEXT,
                    mutation,
                ),
                "surface mutation {mutation:?} must fail"
            );
        }
    }
    assert!(!task258b4b_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B4B_TEXT,
        SourceStatementB4BSurfaceMutation::RootIdentity,
    ));

    let resolver = source_statement_b4b_resolver_env_for_test(
        &ast,
        &module,
        &symbols,
        SOURCE_STATEMENT_B4B_TEXT,
    )
    .expect("Task258B4B resolver provenance");
    assert_eq!(
        (
            resolver.symbols().len(),
            resolver.labels().len(),
            resolver.definitions().len(),
            resolver.contributions().len(),
            resolver.imports().len(),
        ),
        (1, 1, 1, 1, 0)
    );
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver
        .symbols()
        .visible_candidates(&namespace, "FormulaConnectiveGroupingPayloadBoundary");
    let [resolver_owner] = owners.as_slice() else {
        panic!("one Task258B4B theorem owner")
    };
    let labels = resolver
        .labels()
        .visible_candidates(&namespace, "FormulaConnectiveGroupingPayloadBoundary");
    let [label] = labels.as_slice() else {
        panic!("one Task258B4B theorem label")
    };
    assert_eq!(
        (
            label.kind(),
            label.visibility(),
            label.export_status(),
            label.namespace(),
            label.primary_spelling(),
            label.contribution().index(),
            label.origin().source_id(),
            label.origin().module_id(),
            label.origin().anchor(),
            label.origin().structural_path(),
            label.origin().import_edge(),
            label.origin().is_recovered(),
        ),
        (
            mizar_resolve::resolved_ast::LabelKind::Theorem,
            mizar_resolve::env::Visibility::Public,
            mizar_resolve::env::ExportStatus::Exported,
            &namespace,
            "FormulaConnectiveGroupingPayloadBoundary",
            0,
            ast.source_id,
            &module,
            &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 165)),
            [2, 0].as_slice(),
            None,
            false,
        )
    );
    assert_eq!(label.origin(), resolver_owner.origin());
    assert_eq!(label.contribution(), resolver_owner.contribution());
    let contribution = resolver
        .contributions()
        .get(resolver_owner.contribution())
        .expect("Task258B4B contribution");
    assert_eq!(
        contribution.effects().labels(),
        [label.origin_path().clone()]
    );

    let statement = &output.statement;
    assert_eq!(statement.source_id, ast.source_id);
    assert_eq!(statement.module_id, module);
    assert_eq!(
        (
            output.binding_env.contexts().len(),
            output.binding_env.bindings().len(),
            output.binding_env.diagnostics().len(),
            statement.owners.len(),
            statement.statements.len(),
            statement.contexts.len(),
            statement.input_facts.len(),
            statement.candidate_facts.len(),
        ),
        (2, 1, 4, 1, 1, 1, 0, 1)
    );
    let owner = &statement.owners[0];
    assert_eq!(owner.contribution.index(), 0);
    assert_eq!(owner.site.node().index(), 120);
    assert_eq!(owner.source_range, range(ast.source_id, 0, 165));
    assert_eq!(owner.spelling, "FormulaConnectiveGroupingPayloadBoundary");
    assert_eq!(owner.role, SourceTheoremRole::Theorem);
    assert_eq!(owner.status, SourceTheoremStatus::Unmodified);
    assert_eq!(owner.recovery, SourceStatementRecovery::Normal);
    let checked_owner = mizar_checker::type_checker::CheckedStatementOwner::validate_exact_local_theorem(
        &resolver, owner.symbol.clone(), ast.source_id, &module,
    ).expect("genuine checked theorem owner before raw admission");
    assert_eq!(checked_owner.origin().structural_path(), [2, 0]);
    assert_eq!(
        checked_owner.source_range(),
        owner.source_range
    );
    let row = &statement.statements[0];
    assert_eq!(row.owner, SourceTheoremOwnerId::new(0));
    assert_eq!(row.context, SourceStatementContextId::new(0));
    assert_eq!(
        row.formula,
        SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0))
    );
    assert_eq!(row.site.node().index(), 120);
    assert_eq!(row.source_range, range(ast.source_id, 0, 165));
    assert_eq!(row.source_ordinal, 0);
    assert_eq!(
        row.spelling,
        concat!(
            "theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ",
            "( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ",
            "( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;",
        )
    );
    assert_eq!(row.kind, SourceStatementKind::TheoremProposition);
    assert_eq!(row.recovery, SourceStatementRecovery::Normal);
    let context = &statement.contexts[0];
    assert_eq!(context.statement, SourceStatementId::new(0));
    assert_eq!(context.binding_context, BindingContextId::new(0));
    assert_eq!(context.source_range, range(ast.source_id, 0, 165));
    assert!(context.visible_bindings.is_empty());
    let candidate = &statement.candidate_facts[0];
    assert_eq!(candidate.statement, SourceStatementId::new(0));
    assert_eq!(candidate.context, SourceStatementContextId::new(0));
    assert_eq!(candidate.ordinal, 0);
    assert_eq!(
        candidate.kind,
        SourceStatementCandidateFactKind::UnverifiedProposition
    );
    assert_eq!(candidate.formula, row.formula);

    let primary = &output.primary;
    let atomic = &output.atomic;
    let composite = &output.composite;
    let composition = &output.composition;
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (16, 0, 16)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (8, 0, 0, 0, 0, 0, 0, 16, 16)
    );
    assert_eq!(
        (
            composite.formulas().len(),
            composite.wrappers().len(),
            composite.roots().len(),
            composite.binders().len(),
            composite.type_sites().len(),
            composite.edges().len(),
            composite.requests().len(),
            composition.atomic_edges().len(),
            composition.bound_uses().len(),
        ),
        (8, 6, 1, 1, 1, 7, 9, 8, 0)
    );
    assert_eq!(
        composite
            .roots()
            .get(SourceFormulaRootId::new(0))
            .expect("formula root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    assert_eq!(output.arena.len(), 124);
    assert!(output.arena.root().is_none());
    let mut lower_owned = Vec::new();
    let mut theorem_owned = Vec::new();
    let mut unowned = Vec::new();
    for (id, node) in output.arena.iter() {
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range)
        );
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
        match node.kind.as_str() {
            "source.formula.composition.unowned" => unowned.push(id.index()),
            "source.statement.theorem" => theorem_owned.push(id.index()),
            _ => lower_owned.push(id.index()),
        }
    }
    const LOWER_OWNED: [usize; 42] = [
        4, 56, 57, 58, 59, 61, 63, 64, 66, 68, 69, 71, 72, 74, 76, 77, 79, 81, 82, 84, 85, 87, 88,
        90, 92, 93, 95, 97, 98, 100, 101, 103, 105, 106, 108, 110, 111, 113, 114, 116, 117, 118,
    ];
    let expected_unowned = (0..124)
        .filter(|index| !LOWER_OWNED.contains(index) && *index != 120)
        .collect::<Vec<_>>();
    assert_eq!(lower_owned, LOWER_OWNED);
    assert_eq!(theorem_owned, [120]);
    assert_eq!(unowned, expected_unowned);
    let lower = source_formula_composition_output_with_source(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4B_TEXT,
    ).expect("independent lower selector").expect("independent lower transport");
    assert_eq!(primary, lower.typed_ast.source_term().unwrap());
    assert_eq!(atomic, lower.typed_ast.source_atomic_formula().unwrap());
    assert_eq!(composite, lower.resolved.source_composite_formula().unwrap());
    assert_eq!(composition, lower.resolved.source_formula_composition().unwrap());
    assert!(lower.typed_ast.source_statement().is_none());
    assert!(symbols.labels().is_empty());
}

#[test]
fn task258b4b_lower_and_upper_mutations_fail_at_owning_stage_and_replay() {
    use mizar_checker::{
        source_atomic_formula::SourceAtomicFormulaId,
        source_statement::{SourceStatementFormulaTarget, SourceStatementRecovery},
        typed_ast::{TypedArena, TypedNodeId},
    };

    let (ast, module, symbols, baseline) = task258b4b_output(266_100);
    let baseline_debug = format!("{baseline:?}");
    let lower = source_formula_composition_output_with_source(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4B_TEXT,
    ).expect("lower selector").expect("independent lower baseline");
    let baseline_typed = lower.typed_ast.debug_text();
    let baseline_resolved = lower.resolved.debug_text();
    let foreign_source = task258b3m2b2b3a_foreign_source_id(ast.source_id);
    let wrong_module = ResolverModuleId::new(
        PackageId::new("pkg"),
        ModulePath::new("statement.b4b.wrong"),
    );
    let mut foreign_contributions = symbols.contributions().clone();
    let foreign_contribution = foreign_contributions.insert(
        module.clone(),
        mizar_resolve::env::ContributionKind::LocalSource {
            source_id: foreign_source,
        },
        mizar_session::SourceAnchor::Range(range(foreign_source, 0, 1)),
    );

    for mutation in 0..8 {
        let run = || {
            source_formula_composition_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4B_TEXT,
                |input| {
                    let before = format!("{input:?}");
                    match mutation {
                    0 => input.primary.source_id = foreign_source,
                    1 => input.primary.terms.clear(),
                    2 => input.atomic.source_id = foreign_source,
                    3 => input.atomic.formulas.clear(),
                    4 => input.composite.source_id = foreign_source,
                    5 => input.composite.formulas.clear(),
                    6 => input.composition.source_id = foreign_source,
                    7 => input.composition.atomic_edges.clear(),
                    _ => unreachable!(),
                    }
                    assert_ne!(format!("{input:?}"), before, "lower mutation {mutation} actual delta");
                },
            )
            .expect("lower selector")
            .expect_err("lower mutation")
        };
        let error = run();
        assert_eq!(error, run(), "lower mutation {mutation} replay");
        let expected = match mutation {
            0 => "source primary-term transaction is inconsistent",
            1 => "source numeric-type request 0 is inconsistent",
            2 => "source atomic-formula environment identity mismatch",
            3 => "source atomic-formula environment identity mismatch",
            4 => "source composite formula 0 is invalid",
            5 => "source composite formula tree is invalid",
            6 => "source formula composition dependency mismatch",
            7 => "source formula composition aggregate is invalid",
            _ => unreachable!(),
        };
        assert_eq!(error, expected, "lower owning stage {mutation}");
        assert_eq!(format!("{baseline:?}"), baseline_debug, "lower purity {mutation}");
        let pristine = source_formula_composition_output_with_source(
            &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4B_TEXT,
        ).expect("pristine lower selector").expect("pristine lower replay");
        assert_eq!(pristine.typed_ast, lower.typed_ast);
        assert_eq!(pristine.resolved, lower.resolved);
    }

    // Private Checker tests retain owning guard errors; this route captures
    // actual upper drafts before the retired raw admission refuses them.
    for mutation in 0..37 {
        let run = || {
            let mut captured = None;
            let error = source_statement_b4b_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4B_TEXT,
                |input| {
                    let before = format!("{input:?}");
                    match mutation {
                    0 => input.statement.source_id = foreign_source,
                    1 => input.statement.module_id = wrong_module.clone(),
                    2 => input.statement.owners.clear(),
                    3 => input
                        .statement
                        .owners
                        .push(input.statement.owners[0].clone()),
                    4 => {
                        input.statement.owners[0].symbol =
                            mizar_resolve::resolved_ast::SymbolId::new(
                                input.statement.module_id.clone(),
                                mizar_resolve::resolved_ast::LocalSymbolId::new(
                                    "ForeignTask258B4BOwner",
                                ),
                                mizar_resolve::resolved_ast::FullyQualifiedName::new(
                                    "pkg::statement.b4b::ForeignTask258B4BOwner",
                                ),
                            )
                    }
                    5 => input.statement.owners[0].contribution = foreign_contribution,
                    6 => {
                        input.statement.owners[0].site =
                            mizar_checker::typed_ast::TypedSiteRef::Node(TypedNodeId::new(119))
                    }
                    7 => input.statement.owners[0].source_range.start += 1,
                    8 => input.statement.owners[0].source_range.end -= 1,
                    9 => input.statement.owners[0].spelling.push_str("NearMiss"),
                    10 => input.statement.owners[0].recovery = SourceStatementRecovery::Degraded,
                    11 => input.statement.statements.clear(),
                    12 => input
                        .statement
                        .statements
                        .push(input.statement.statements[0].clone()),
                    13 => {
                        input.statement.statements[0].owner =
                            mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                    }
                    14 => {
                        input.statement.statements[0].context =
                            mizar_checker::source_statement::SourceStatementContextId::new(1)
                    }
                    15 => {
                        input.statement.statements[0].formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
                    }
                    16 => {
                        input.statement.statements[0].site =
                            mizar_checker::typed_ast::TypedSiteRef::Node(TypedNodeId::new(119))
                    }
                    17 => input.statement.statements[0].source_range.start += 1,
                    18 => input.statement.statements[0].source_range.end -= 1,
                    19 => input.statement.statements[0].source_ordinal = 1,
                    20 => input.statement.statements[0].spelling.push_str(" NearMiss"),
                    21 => {
                        input.statement.statements[0].kind =
                            mizar_checker::source_statement::SourceStatementKind::Assumption
                    }
                    22 => {
                        input.statement.statements[0].recovery = SourceStatementRecovery::Degraded
                    }
                    23 => input.statement.contexts.clear(),
                    24 => input
                        .statement
                        .contexts
                        .push(input.statement.contexts[0].clone()),
                    25 => {
                        input.statement.contexts[0].statement =
                            mizar_checker::source_statement::SourceStatementId::new(1)
                    }
                    26 => input.statement.contexts[0].binding_context = BindingContextId::new(1),
                    27 => input.statement.contexts[0].source_range.start += 1,
                    28 => input.statement.contexts[0].source_range.end -= 1,
                    29 => input.statement.contexts[0]
                        .visible_bindings
                        .push(mizar_checker::binding_env::BindingId::new(0)),
                    30 => input.statement.input_facts.push(
                        mizar_checker::source_statement::SourceStatementInputFactInput {
                            statement:
                                mizar_checker::source_statement::SourceStatementId::new(0),
                            context:
                                mizar_checker::source_statement::SourceStatementContextId::new(0),
                            ordinal: 0,
                            kind: mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                            binding: mizar_checker::binding_env::BindingId::new(0),
                            uses: Vec::new(),
                        },
                    ),
                    31 => input.statement.candidate_facts.clear(),
                    32 => input
                        .statement
                        .candidate_facts
                        .push(input.statement.candidate_facts[0].clone()),
                    33 => {
                        input.statement.candidate_facts[0].statement =
                            mizar_checker::source_statement::SourceStatementId::new(1)
                    }
                    34 => {
                        input.statement.candidate_facts[0].context =
                            mizar_checker::source_statement::SourceStatementContextId::new(1)
                    }
                    35 => input.statement.candidate_facts[0].ordinal = 1,
                    36 => {
                        input.statement.candidate_facts[0].formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
                    }
                    _ => unreachable!(),
                    }
                    let changed = format!("{input:?}");
                    assert_ne!(changed, before, "upper draft mutation {mutation} actual delta");
                    captured = Some(changed);
                },
            )
            .expect("upper selector")
            .expect_err("retired raw upper");
            assert_eq!(error, "source statement dependency mismatch");
            captured.expect("actual upper draft mutation before refusal")
        };
        assert_eq!(run(), run(), "upper draft mutation {mutation} replay");
        assert_eq!(format!("{baseline:?}"), baseline_debug, "upper draft purity {mutation}");
    }

    const OWNED_NODES: [usize; 43] = [
        4, 56, 57, 58, 59, 61, 63, 64, 66, 68, 69, 71, 72, 74, 76, 77, 79, 81, 82, 84, 85, 87, 88,
        90, 92, 93, 95, 97, 98, 100, 101, 103, 105, 106, 108, 110, 111, 113, 114, 116, 117, 118,
        120,
    ];
    for node_index in OWNED_NODES {
        for mutate_kind in [false, true] {
            let run = || {
                let mut captured = None;
                let error = source_statement_b4b_output_with_mutation(
                    &ast,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B4B_TEXT,
                    |input| {
                        let before = format!("{:?}", input.arena);
                        let mut nodes = input
                            .arena
                            .iter()
                            .map(|(_, node)| node.clone())
                            .collect::<Vec<_>>();
                        if mutate_kind {
                            nodes[node_index].kind = "source.statement.corrupted".into();
                        } else {
                            nodes[node_index].anchor =
                                mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 0));
                        }
                        input.arena =
                            TypedArena::try_new(input.arena.root(), nodes).expect("mutated arena");
                        let changed = format!("{:?}", input.arena);
                        assert_ne!(changed, before, "owned arena {node_index} kind={mutate_kind} actual delta");
                        captured = Some(changed);
                    },
                )
                .expect("owned-node selector")
                .expect_err("retired raw upper");
                assert_eq!(error, "source statement dependency mismatch");
                captured.expect("actual owned arena draft mutation")
            };
            let error = run();
            assert_eq!(
                error,
                run(),
                "owned node {node_index} kind={mutate_kind} replay"
            );
        }
    }
    let rooted_run = || {
        let mut captured = None;
        let error = source_statement_b4b_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B4B_TEXT,
            |input| {
                assert!(input.arena.root().is_none());
                input.arena = TypedArena::try_new(
                    Some(TypedNodeId::new(123)),
                    input.arena.iter().map(|(_, node)| node.clone()).collect(),
                )
                .expect("coherent rooted arena");
                captured = Some(format!("{:?}", input.arena));
            },
        )
        .expect("rooted selector")
        .expect_err("retired raw upper");
        assert_eq!(error, "source statement dependency mismatch");
        captured.expect("actual rooted draft")
    };
    assert_eq!(rooted_run(), rooted_run(), "rooted replay");

    let replay =
        source_formula_composition_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B4B_TEXT)
            .expect("replay selector")
            .expect("replay");
    assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
    assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    assert_eq!(format!("{baseline:?}"), baseline_debug);
    let (_, _, _, pristine) = task258b4b_output(266_100);
    assert_eq!(format!("{pristine:?}"), baseline_debug);
}

#[test]
fn task258b4b_source_resolver_and_active_b2_near_misses_stay_isolated() {
    let active_source = &SOURCE_STATEMENT_B4B_TEXT[..166];
    assert_eq!(active_source.as_bytes().last(), Some(&b'\n'));
    assert_eq!(
        sha256_text(active_source),
        "af9e19390f9fd37e12184486ce50600603237157e50b29c32868a73b375c023c"
    );
    let (active_ast, active_module, _, active_symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(active_source, 266_200);
    assert_eq!(diagnostics, 0);
    let active_symbols =
        augment_type_elaboration_import_summaries(&active_ast, &active_module, active_symbols);
    assert!(
        source_statement_output_with_source(
            &active_ast,
            active_module.clone(),
            &active_symbols,
            active_source,
        )
        .is_none(),
        "the active one-LF source must remain lower-only"
    );
    let lower = source_formula_composition_output_with_source(
        &active_ast,
        active_module,
        &active_symbols,
        active_source,
    )
    .expect("active lower selector")
    .expect("active lower output");
    assert!(lower.typed_ast.source_statement().is_none());
    assert!(lower.typed_ast.source_composite_formula().is_some());
    assert!(lower.typed_ast.source_formula_composition().is_some());

    let (b4a_ast, b4a_module, b4a_symbols, b4a) = task258b4a_output(266_201);
    assert!(
        source_statement_b4b_output_with_mutation(
            &b4a_ast,
            b4a_module,
            &b4a_symbols,
            SOURCE_STATEMENT_B4A_TEXT,
            |_| {},
        )
        .is_none()
    );
    assert_eq!(
        b4a.primary.references().iter().map(|(_, row)| row.use_ordinal()).collect::<Vec<_>>(),
        [1, 1]
    );

    let (ast, module, symbols, baseline) = task258b4b_output(266_210);
    let pre_enriched = source_statement_b4b_resolver_env_for_test(
        &ast,
        &module,
        &symbols,
        SOURCE_STATEMENT_B4B_TEXT,
    )
    .expect("one enriched resolver environment");
    let symbols_before = format!("{symbols:?}");
    let baseline_before = format!("{baseline:?}");
    let pre_enriched_before = format!("{pre_enriched:?}");
    let pristine_lower = source_formula_composition_output_with_source(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4B_TEXT,
    ).expect("pristine lower selector").expect("pristine lower");
    let pre_enriched_error = source_statement_b4b_output_with_mutation(
        &ast,
        module.clone(),
        &pre_enriched,
        SOURCE_STATEMENT_B4B_TEXT,
        |_| panic!("pre-enriched input reached lower capture"),
    )
    .expect("pre-enriched selector")
    .expect_err("pre-enriched input must fail raw authentication");
    assert_eq!(
        pre_enriched_error,
        "Task258B4B raw resolver profile mismatch"
    );
    for mutation in [
        Task258B2ResolverMutation::Imported,
        Task258B2ResolverMutation::Missing,
        Task258B2ResolverMutation::Duplicate,
        Task258B2ResolverMutation::WrongPath,
        Task258B2ResolverMutation::WrongKind,
        Task258B2ResolverMutation::Private,
        Task258B2ResolverMutation::LocalOnly,
        Task258B2ResolverMutation::Recovered,
    ] {
        let run = || {
            source_statement_b4b_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4B_TEXT,
                |symbols| {
                    let before = format!("{symbols:?}");
                    let changed = task258b2_mutate_resolver(symbols, mutation);
                    assert_ne!(format!("{changed:?}"), before, "actual resolver delta {mutation:?}");
                    changed
                },
            )
            .expect("resolver selector")
            .expect_err("resolver mutation")
        };
        let error = run();
        assert_eq!(error, match mutation {
            Task258B2ResolverMutation::Missing | Task258B2ResolverMutation::Duplicate =>
                "Task258B4B resolver theorem label mismatch",
            _ => "Task258B4B resolver theorem provenance mismatch",
        }, "resolver owning stage {mutation:?}");
        assert_eq!(error, run(), "{mutation:?} replay");
    }
    for mutation in [
        Task258B3M2B2B3ALabelMutation::Namespace,
        Task258B3M2B2B3ALabelMutation::PrimarySpelling,
        Task258B3M2B2B3ALabelMutation::Contribution,
        Task258B3M2B2B3ALabelMutation::OriginSource,
        Task258B3M2B2B3ALabelMutation::OriginModule,
        Task258B3M2B2B3ALabelMutation::OriginAnchor,
        Task258B3M2B2B3ALabelMutation::OriginStructuralPath,
        Task258B3M2B2B3ALabelMutation::ContributionEffectsMissing,
        Task258B3M2B2B3ALabelMutation::ContributionEffectsExtra,
        Task258B3M2B2B3ALabelMutation::ContributionEffectsWrongPath,
    ] {
        let run = || {
            source_statement_b4b_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4B_TEXT,
                |symbols| {
                    let before = format!("{symbols:?}");
                    let changed = task258b3m2b2b3a_mutate_label_resolver(symbols, mutation);
                    assert_ne!(format!("{changed:?}"), before, "actual resolver delta {mutation:?}");
                    changed
                },
            )
            .expect("label resolver selector")
            .expect_err("label resolver mutation")
        };
        let error = run();
        assert_eq!(error, match mutation {
            Task258B3M2B2B3ALabelMutation::Namespace
            | Task258B3M2B2B3ALabelMutation::PrimarySpelling =>
                "Task258B4B resolver theorem label mismatch",
            _ => "Task258B4B resolver theorem provenance mismatch",
        }, "label owning stage {mutation:?}");
        assert_eq!(error, run(), "{mutation:?} replay");
    }
    let replay = source_formula_composition_output_with_source(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4B_TEXT,
    ).expect("replay selector").expect("pristine lower replay");
    assert_eq!(replay.typed_ast, pristine_lower.typed_ast);
    assert_eq!(replay.resolved, pristine_lower.resolved);
    let (_, _, replay_symbols, replay) = task258b4b_output(266_210);
    assert_eq!(format!("{replay:?}"), baseline_before);
    assert_eq!(format!("{baseline:?}"), baseline_before);
    assert_eq!(format!("{symbols:?}"), symbols_before);
    assert_eq!(format!("{replay_symbols:?}"), symbols_before);
    assert_eq!(format!("{pre_enriched:?}"), pre_enriched_before);
}

#[test]
fn task258b4b_cross_family_b4a_orders_and_ownership_are_atomic() {
    use mizar_checker::{
        source_composite_formula::{SourceFormulaRootId, SourceFormulaRootOwnership},
        typed_ast::TypedAstError,
    };

    let (b4b_ast, b4b_module, b4b_symbols, b4b) = task258b4b_output(266_300);
    let (b4a_ast, b4a_module, b4a_symbols, b4a) = task258b4a_output(266_301);
    let b4b_before = format!("{b4b:?}");
    let b4a_before = format!("{b4a:?}");
    let b4b_lower = source_formula_composition_output_with_source(
        &b4b_ast,
        b4b_module.clone(),
        &b4b_symbols,
        SOURCE_STATEMENT_B4B_TEXT,
    )
    .expect("B4B lower selector")
    .expect("B4B lower route");
    let b4a_lower = source_formula_composition_output_with_source(
        &b4a_ast,
        b4a_module.clone(),
        &b4a_symbols,
        SOURCE_STATEMENT_B4A_TEXT,
    )
    .expect("B4A lower selector")
    .expect("B4A lower route");
    let (atomic_ast, atomic_module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_STATEMENT_TEXT, 266_302);
    assert_eq!(diagnostics, 0);
    let (atomic_typed, atomic_resolved) = task258_native_output(&atomic_ast, &atomic_module);
    let atomic_before = atomic_typed.clone();
    let atomic_statement = atomic_typed
        .source_statement()
        .expect("atomic statement")
        .clone();
    assert!(atomic_statement.composite_formula_fingerprint().is_none());
    assert!(atomic_statement.formula_composition_fingerprint().is_none());
    assert_eq!(
        b4b_lower
            .typed_ast
            .clone()
            .with_source_statement(atomic_statement)
            .expect_err("B4B lower then atomic statement must fail"),
        TypedAstError::InvalidSourceStatement
    );
    let run_b = || {
        let mut changed = None;
        let error = source_statement_b4b_output_with_mutation(
            &b4b_ast, b4b_module.clone(), &b4b_symbols, SOURCE_STATEMENT_B4B_TEXT,
            |input| {
                let before = format!("{input:?}");
                input.composite = b4a.composite.clone();
                input.composition = b4a.composition.clone();
                let after = format!("{input:?}");
                assert_ne!(after, before, "actual A lower donor delta");
                changed = Some(after);
            },
        ).expect("B4B/A selector").expect_err("retired raw B4B");
        assert_eq!(error, "source statement dependency mismatch");
        changed.expect("genuine B4B draft with A lower donor")
    };
    assert_eq!(run_b(), run_b(), "B/A draft replay");
    let run_a = || {
        let mut changed = None;
        let error = source_statement_b4a_output_with_mutation(
            &b4a_ast, b4a_module.clone(), &b4a_symbols, SOURCE_STATEMENT_B4A_TEXT,
            |input| {
                let before = format!("{input:?}");
                input.composite = b4b.composite.clone();
                input.composition = b4b.composition.clone();
                let after = format!("{input:?}");
                assert_ne!(after, before, "actual B lower donor delta");
                changed = Some(after);
            },
        ).expect("B4A/B selector").expect_err("retired raw B4A");
        assert_eq!(error, "source statement dependency mismatch");
        changed.expect("genuine B4A draft with B lower donor")
    };
    assert_eq!(run_a(), run_a(), "A/B draft replay");
    assert_eq!(format!("{b4b:?}"), b4b_before);
    assert_eq!(format!("{b4a:?}"), b4a_before);
    assert_eq!(
        b4b.composite
            .roots()
            .get(SourceFormulaRootId::new(0))
            .expect("B4B root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    assert_eq!(
        b4a.composite
            .roots()
            .get(SourceFormulaRootId::new(0))
            .expect("B4A root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    for (ast, module, symbols, source, lower) in [
        (&b4a_ast, &b4a_module, &b4a_symbols, SOURCE_STATEMENT_B4A_TEXT, &b4a_lower),
        (&b4b_ast, &b4b_module, &b4b_symbols, SOURCE_STATEMENT_B4B_TEXT, &b4b_lower),
    ] {
        let replay = source_formula_composition_output_with_source(ast, module.clone(), symbols, source)
            .expect("lower replay selector").expect("pristine lower replay");
        assert_eq!(replay.typed_ast, lower.typed_ast);
        assert_eq!(replay.resolved, lower.resolved);
    }
    assert_eq!(atomic_typed, atomic_before);
    let (atomic_replay, atomic_final_replay) = task258_native_output(&atomic_ast, &atomic_module);
    assert_eq!(atomic_replay, atomic_typed);
    assert_eq!(atomic_final_replay, atomic_resolved);
    let (_, _, _, replay_b) = task258b4b_output(266_300);
    let (_, _, _, replay_a) = task258b4a_output(266_301);
    assert_eq!(format!("{replay_b:?}"), b4b_before);
    assert_eq!(format!("{replay_a:?}"), b4a_before);
}

#[test]
fn task258b4b_final_clone_debug_and_semantic_outputs_are_stable() {
    let (ast, module, symbols, captured) = task258b4b_output(266_400);
    assert_eq!(format!("{:?}", captured.clone()), format!("{captured:?}"));
    let output = source_formula_composition_output_with_source(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B4B_TEXT,
    ).expect("lower selector").expect("independent lower final");
    assert_eq!(output.typed_ast.clone(), output.typed_ast);
    assert_eq!(output.resolved.clone(), output.resolved);
    assert_eq!(
        output.typed_ast.clone().debug_text(),
        output.typed_ast.debug_text()
    );
    assert_eq!(
        output.resolved.clone().debug_text(),
        output.resolved.debug_text()
    );
    assert!(output.typed_ast.source_statement().is_none());
    assert!(output.resolved.source_statement().is_none());
    assert_eq!(output.typed_ast.source_term(), Some(&captured.primary));
    assert_eq!(output.typed_ast.source_atomic_formula(), Some(&captured.atomic));
    assert_eq!(output.typed_ast.source_composite_formula(), Some(&captured.composite));
    assert_eq!(output.typed_ast.source_formula_composition(), Some(&captured.composition));
    assert!(output.typed_ast.source_statement_references().is_none());
    assert!(output.typed_ast.source_statement_witnesses().is_none());
    assert!(output.typed_ast.source_context().is_none());
    assert!(output.typed_ast.source_type().is_none());
    assert!(output.typed_ast.source_attribute().is_none());
    assert!(output.typed_ast.source_evidence().is_none());
    assert!(output.typed_ast.source_application().is_none());
    assert!(output.typed_ast.source_structure().is_none());
    assert!(output.typed_ast.source_set_term().is_none());
    assert!(
        output
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        output
            .typed_ast
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(output.typed_ast.contexts().is_empty());
    assert!(output.typed_ast.types().is_empty());
    assert!(output.typed_ast.facts().is_empty());
    assert!(output.typed_ast.coercions().is_empty());
    assert!(output.typed_ast.initial_obligations().is_empty());
    assert!(output.typed_ast.diagnostics().is_empty());
    assert!(output.resolved.expr_metadata().is_empty());
    assert!(output.resolved.collection_candidates().is_empty());
    assert!(output.resolved.expanded_candidates().is_empty());
    assert!(output.resolved.template_expansions().is_empty());
    assert!(output.resolved.viable_candidates().is_empty());
    assert!(output.resolved.viability_decisions().is_empty());
    assert!(output.resolved.specificity_graphs().is_empty());
    assert!(output.resolved.resolved_overloads().is_empty());
    assert!(output.resolved.inserted_coercions().is_empty());
    assert!(output.resolved.cluster_facts().is_empty());
    assert!(output.resolved.diagnostics().is_empty());
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
    assert!(
        source_statement_transport_detail_keys(&ast, module, &symbols, SOURCE_STATEMENT_B4B_TEXT,)
            .is_none()
    );

    let (b4a_ast, b4a_module, b4a_symbols, _) = task258b4a_output(266_401);
    assert!(
        source_statement_transport_detail_keys(
            &b4a_ast,
            b4a_module,
            &b4a_symbols,
            SOURCE_STATEMENT_B4A_TEXT,
        )
        .is_none()
    );
}
