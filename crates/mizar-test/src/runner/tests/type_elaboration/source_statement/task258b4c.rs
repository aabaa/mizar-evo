#[test]
fn task258b4c_real_route_publishes_exact_nested_quantifier_statement_root() {
    use mizar_checker::source_composite_formula::{
        SourceFormulaRootId, SourceFormulaRootOwnership,
    };

    assert_eq!(SOURCE_STATEMENT_B4C_TEXT.len(), 139);
    assert_eq!(&SOURCE_STATEMENT_B4C_TEXT.as_bytes()[137..], &[10, 10]);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B4C_TEXT),
        "36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1"
    );
    let (ast, module, symbols, output) = task258b4c_output(266_500);
    assert_eq!(ast.nodes().len(), 66);
    assert_eq!(ast.root().expect("surface root").index(), 65);
    assert!(task258b4c_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B4C_TEXT,
        SourceStatementB4CSurfaceMutation::None,
    ));
    for index in 0..66 {
        for mutation in [
            SourceStatementB4CSurfaceMutation::NodeKind(index),
            SourceStatementB4CSurfaceMutation::NodeRange(index),
            SourceStatementB4CSurfaceMutation::NodeRecovery(index),
            SourceStatementB4CSurfaceMutation::NodeChildren(index),
        ] {
            assert!(
                !task258b4c_surface_profile_with_mutation_for_test(
                    &ast,
                    SOURCE_STATEMENT_B4C_TEXT,
                    mutation,
                ),
                "surface mutation {mutation:?} must fail"
            );
        }
    }
    assert!(!task258b4c_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B4C_TEXT,
        SourceStatementB4CSurfaceMutation::RootIdentity,
    ));
    for index in 0..SOURCE_STATEMENT_B4C_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B4C_TEXT.as_bytes().to_vec();
        bytes[index] ^= 1;
        let changed = String::from_utf8(bytes).expect("ASCII source mutation");
        assert!(
            !task258b4c_surface_profile_with_mutation_for_test(
                &ast,
                &changed,
                SourceStatementB4CSurfaceMutation::None,
            ),
            "source byte {index}"
        );
        assert!(
            source_statement_b4c_lower_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                &changed,
                |_| panic!("near miss reached lower capture"),
            )
            .is_none()
        );
    }
    assert_eq!(
        (
            symbols.symbols().len(),
            symbols.labels().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
            symbols.imports().len(),
        ),
        (1, 0, 1, 1, 0)
    );
    let resolver = source_statement_b4c_resolver_env_for_test(
        &ast,
        &module,
        &symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("Task258B4C resolver provenance");
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
        .visible_candidates(&namespace, "FormulaNestedQuantifierPayloadBoundary");
    let [resolver_owner] = owners.as_slice() else {
        panic!("one Task258B4C theorem owner")
    };
    let labels = resolver
        .labels()
        .visible_candidates(&namespace, "FormulaNestedQuantifierPayloadBoundary");
    let [label] = labels.as_slice() else {
        panic!("one Task258B4C theorem label")
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
            "FormulaNestedQuantifierPayloadBoundary",
            0,
            ast.source_id,
            &module,
            &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 137)),
            [2, 1].as_slice(),
            None,
            false,
        )
    );
    assert_eq!(label.origin(), resolver_owner.origin());
    assert_eq!(label.contribution(), resolver_owner.contribution());
    let contribution = resolver
        .contributions()
        .get(resolver_owner.contribution())
        .expect("Task258B4C contribution");
    assert_eq!(
        contribution.anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 18))
    );
    assert_eq!(
        contribution.effects().symbols(),
        [resolver_owner.symbol().clone()]
    );
    assert_eq!(contribution.effects().definitions().len(), 1);
    assert_eq!(
        contribution.effects().labels(),
        [label.origin_path().clone()]
    );

    assert_eq!(
        (
            output.binding_env.contexts().len(),
            output.binding_env.bindings().len(),
            output.binding_env.diagnostics().len(),
        ),
        (4, 4, 0)
    );
    let checked_owner =
        mizar_checker::type_checker::CheckedStatementOwner::validate_exact_local_theorem(
            &resolver,
            resolver_owner.symbol().clone(),
            ast.source_id,
            &module,
        )
        .expect("genuine checked theorem owner");
    assert_eq!(checked_owner.origin().structural_path(), [2, 1]);
    assert_eq!(checked_owner.symbol(), resolver_owner.symbol());
    assert_eq!(checked_owner.source_range(), range(ast.source_id, 19, 137));
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
        (6, 6, 0)
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, reference)| (reference.binding().index(), reference.use_ordinal()))
            .collect::<Vec<_>>(),
        [(1, 2), (1, 2), (3, 4), (2, 4), (1, 4), (3, 4)]
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
        (3, 0, 0, 0, 0, 0, 0, 6, 6)
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
        (3, 0, 1, 3, 3, 2, 6, 3, 6)
    );
    assert_eq!(
        composite
            .roots()
            .get(SourceFormulaRootId::new(0))
            .expect("formula root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    assert_eq!(output.arena.len(), 66);
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
    const LOWER_OWNED: [usize; 24] = [
        9, 17, 22, 32, 33, 36, 37, 38, 39, 41, 43, 44, 45, 46, 47, 48, 50, 52, 53, 55, 57, 58, 59,
        60,
    ];
    let expected_unowned = (0..66)
        .filter(|index| !LOWER_OWNED.contains(index) && *index != 62)
        .collect::<Vec<_>>();
    assert_eq!(lower_owned, LOWER_OWNED);
    assert_eq!(theorem_owned, [62]);
    assert_eq!(unowned, expected_unowned);
    let lower = source_formula_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("independent lower selector")
    .expect("independent lower transport");
    assert_eq!(primary, lower.typed_ast.source_term().unwrap());
    assert_eq!(atomic, lower.typed_ast.source_atomic_formula().unwrap());
    assert_eq!(
        composite,
        lower.resolved.source_composite_formula().unwrap()
    );
    assert_eq!(
        composition,
        lower.resolved.source_formula_composition().unwrap()
    );
    assert!(lower.typed_ast.source_statement().is_none());
    assert!(symbols.labels().is_empty());
    assert_eq!(module, lower.typed_ast.module_id().clone());
}

#[test]
fn task258b4c_lower_mutations_fail_at_owning_stage_and_replay() {
    let (ast, module, symbols, baseline) = task258b4c_output(266_600);
    let baseline_debug = format!("{baseline:?}");
    let symbols_before = format!("{symbols:?}");
    let lower = source_formula_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("lower selector")
    .expect("independent lower baseline");
    let foreign_source = task258b3m2b2b3a_foreign_source_id(ast.source_id);
    for mutation in 0..8 {
        let run = || {
            source_formula_composition_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4C_TEXT,
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
                        7 => input.composition.bound_uses.clear(),
                        _ => unreachable!(),
                    }
                    assert_ne!(
                        format!("{input:?}"),
                        before,
                        "actual lower mutation {mutation}"
                    );
                },
            )
            .expect("lower selector")
            .expect_err("lower mutation")
        };
        let error = run();
        assert_eq!(
            error,
            [
                "source primary-term transaction is inconsistent",
                "source primary-term reference 0 is inconsistent",
                "source atomic-formula environment identity mismatch",
                "source atomic-formula environment identity mismatch",
                "source composite formula 0 is invalid",
                "source composite formula tree is invalid",
                "source formula composition dependency mismatch",
                "source formula composition aggregate is invalid",
            ][mutation],
            "lower owning stage {mutation}"
        );
        assert_eq!(error, run(), "lower mutation {mutation} replay");
    }

    let (_, _, replay_symbols, replay) = task258b4c_output(266_600);
    assert_eq!(format!("{replay:?}"), baseline_debug);
    assert_eq!(format!("{symbols:?}"), symbols_before);
    assert_eq!(format!("{replay_symbols:?}"), symbols_before);
    let lower_replay = source_formula_composition_output_with_source(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("lower replay selector")
    .expect("lower pristine replay");
    assert_eq!(lower_replay.typed_ast, lower.typed_ast);
    assert_eq!(lower_replay.resolved, lower.resolved);
}

#[test]
fn task258b4c_source_resolver_and_active_b3_near_misses_stay_isolated() {
    let active_source = &SOURCE_STATEMENT_B4C_TEXT[..138];
    assert_eq!(active_source.as_bytes().last(), Some(&b'\n'));
    assert_eq!(
        sha256_text(active_source),
        "cbfd7077713e8e9630900e349d5f579251c19fba55434acb62170ea1dd940237"
    );
    let (active_ast, active_module, _, active_symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(active_source, 266_700);
    assert_eq!(diagnostics, 0);
    let active_symbols =
        augment_type_elaboration_import_summaries(&active_ast, &active_module, active_symbols);

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

    for (ordinal, near_miss) in [
        SOURCE_STATEMENT_B4C_TEXT.trim_end_matches('\n').to_owned(),
        format!("{SOURCE_STATEMENT_B4C_TEXT}\n"),
        SOURCE_STATEMENT_B4C_TEXT.replacen("for r st", "for s st", 1),
        SOURCE_STATEMENT_B4C_TEXT.replacen("r = y", "y = r", 1),
        SOURCE_STATEMENT_B4C_TEXT.replacen(
            "FormulaNestedQuantifierPayloadBoundary",
            "FormulaNestedQuantifierPayloadNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, _, near_symbols, _) =
            task253_ast_from_source_text_with_diagnostic_count(&near_miss, 266_710 + ordinal);
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_symbols);

        let lower = source_formula_composition_output_with_source(
            &near_ast,
            near_module,
            &near_symbols,
            &near_miss,
        );
        if ordinal < 2 {
            let lower = lower
                .expect("layout still selects lower B3")
                .expect("genuine lower layout transport");
            assert!(lower.typed_ast.source_statement().is_none());
            assert!(lower.typed_ast.source_composite_formula().is_some());
            assert!(lower.typed_ast.source_formula_composition().is_some());
        } else {
            assert!(
                lower.is_none(),
                "near miss {ordinal} must not reach Task257B3"
            );
        }
    }

    let (b4a_ast, b4a_module, b4a_symbols, b4a) = task258b4a_output(266_720);
    assert!(
        source_statement_b4c_lower_with_mutation(
            &b4a_ast,
            b4a_module,
            &b4a_symbols,
            SOURCE_STATEMENT_B4A_TEXT,
            |_| {},
        )
        .is_none()
    );
    assert_eq!(
        b4a.primary
            .references()
            .iter()
            .map(|(_, row)| row.use_ordinal())
            .collect::<Vec<_>>(),
        [1, 1]
    );
    let (b4b_ast, b4b_module, b4b_symbols, b4b) = task258b4b_output(266_721);
    assert!(
        source_statement_b4c_lower_with_mutation(
            &b4b_ast,
            b4b_module,
            &b4b_symbols,
            SOURCE_STATEMENT_B4B_TEXT,
            |_| {},
        )
        .is_none()
    );
    assert_eq!(b4b.primary.terms().len(), 16);
    assert_eq!(b4b.primary.numeric_type_requests().len(), 16);
    assert!(b4b.primary.references().is_empty());

    let (ast, module, symbols, baseline) = task258b4c_output(266_730);
    let pre_enriched = source_statement_b4c_resolver_env_for_test(
        &ast,
        &module,
        &symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("one enriched resolver environment");
    let symbols_before = format!("{symbols:?}");
    let baseline_before = format!("{baseline:?}");
    let pre_enriched_before = format!("{pre_enriched:?}");
    let pre_enriched_error = source_statement_b4c_lower_with_mutation(
        &ast,
        module.clone(),
        &pre_enriched,
        SOURCE_STATEMENT_B4C_TEXT,
        |_| panic!("pre-enriched resolver reached draft capture"),
    )
    .expect("pre-enriched selector")
    .expect_err("pre-enriched input must fail raw authentication");
    assert_eq!(
        pre_enriched_error,
        "Task258B4C raw resolver profile mismatch"
    );
    for mutation in [
        Task258B4CRawResolverMutation::OwnerOriginSource,
        Task258B4CRawResolverMutation::OwnerOriginModule,
        Task258B4CRawResolverMutation::OwnerOriginRange,
        Task258B4CRawResolverMutation::OwnerOriginPath,
        Task258B4CRawResolverMutation::OwnerKind,
        Task258B4CRawResolverMutation::OwnerVisibility,
        Task258B4CRawResolverMutation::OwnerExport,
        Task258B4CRawResolverMutation::OwnerSpelling,
        Task258B4CRawResolverMutation::ContributionModule,
        Task258B4CRawResolverMutation::ContributionKind,
        Task258B4CRawResolverMutation::ContributionSource,
        Task258B4CRawResolverMutation::ContributionAnchor,
        Task258B4CRawResolverMutation::ContributionSymbolEffects,
        Task258B4CRawResolverMutation::ContributionDefinitionEffects,
    ] {
        let run = || {
            source_statement_b4c_lower_with_raw_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4C_TEXT,
                |symbols| {
                    let before = format!("{symbols:?}");
                    let changed = task258b4c_mutate_raw_resolver(symbols, mutation);
                    assert_ne!(
                        format!("{changed:?}"),
                        before,
                        "actual resolver delta {mutation:?}"
                    );
                    changed
                },
            )
            .expect("raw resolver selector")
            .expect_err("raw resolver mutation")
        };
        let error = run();
        use mizar_checker::type_checker::StatementOwnerError;
        let owners = symbols.symbols().visible_candidates(
            &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
            "FormulaNestedQuantifierPayloadBoundary",
        );
        let [owner] = owners.as_slice() else {
            panic!("one Task258B4C theorem owner")
        };
        let symbol = owner.symbol().clone();
        let expected = match mutation {
            Task258B4CRawResolverMutation::OwnerOriginSource
            | Task258B4CRawResolverMutation::OwnerVisibility
            | Task258B4CRawResolverMutation::OwnerExport => {
                StatementOwnerError::InvalidOwner { symbol }.to_string()
            }
            Task258B4CRawResolverMutation::OwnerOriginModule
            | Task258B4CRawResolverMutation::OwnerOriginRange
            | Task258B4CRawResolverMutation::OwnerOriginPath => {
                StatementOwnerError::InvalidDefinition { symbol }.to_string()
            }
            Task258B4CRawResolverMutation::ContributionModule
            | Task258B4CRawResolverMutation::ContributionKind
            | Task258B4CRawResolverMutation::ContributionSource => {
                StatementOwnerError::InvalidProvenance { symbol }.to_string()
            }
            Task258B4CRawResolverMutation::OwnerKind
            | Task258B4CRawResolverMutation::OwnerSpelling => {
                "Task258B4C raw resolver theorem owner mismatch".to_owned()
            }
            Task258B4CRawResolverMutation::ContributionAnchor
            | Task258B4CRawResolverMutation::ContributionSymbolEffects
            | Task258B4CRawResolverMutation::ContributionDefinitionEffects => {
                "Task258B4C raw resolver provenance mismatch".to_owned()
            }
        };
        assert_eq!(error, expected, "raw owning stage {mutation:?}");
        assert_eq!(error, run(), "{mutation:?} raw replay");
    }
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
            source_statement_b4c_lower_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4C_TEXT,
                |symbols| {
                    let before = format!("{symbols:?}");
                    let changed = task258b2_mutate_resolver(symbols, mutation);
                    assert_ne!(
                        format!("{changed:?}"),
                        before,
                        "actual resolver delta {mutation:?}"
                    );
                    changed
                },
            )
            .expect("resolver selector")
            .expect_err("resolver mutation")
        };
        let error = run();
        assert_eq!(
            error,
            match mutation {
                Task258B2ResolverMutation::Missing | Task258B2ResolverMutation::Duplicate =>
                    "Task258B4C resolver theorem label mismatch",
                _ => "Task258B4C resolver theorem provenance mismatch",
            },
            "resolver owning stage {mutation:?}"
        );
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
            source_statement_b4c_lower_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B4C_TEXT,
                |symbols| {
                    let before = format!("{symbols:?}");
                    let changed = task258b3m2b2b3a_mutate_label_resolver(symbols, mutation);
                    assert_ne!(
                        format!("{changed:?}"),
                        before,
                        "actual resolver delta {mutation:?}"
                    );
                    changed
                },
            )
            .expect("label resolver selector")
            .expect_err("label resolver mutation")
        };
        let error = run();
        assert_eq!(
            error,
            match mutation {
                Task258B3M2B2B3ALabelMutation::Namespace
                | Task258B3M2B2B3ALabelMutation::PrimarySpelling =>
                    "Task258B4C resolver theorem label mismatch",
                _ => "Task258B4C resolver theorem provenance mismatch",
            },
            "label owning stage {mutation:?}"
        );
        assert_eq!(error, run(), "{mutation:?} replay");
    }
    let (_, _, replay_symbols, replay) = task258b4c_output(266_730);
    assert_eq!(format!("{replay:?}"), baseline_before);
    assert_eq!(format!("{baseline:?}"), baseline_before);
    assert_eq!(format!("{symbols:?}"), symbols_before);
    assert_eq!(format!("{replay_symbols:?}"), symbols_before);
    assert_eq!(format!("{pre_enriched:?}"), pre_enriched_before);
}

#[test]
fn task258b4c_cross_family_b4a_b4b_orders_and_ownership_are_atomic() {
    use mizar_checker::source_composite_formula::{
        SourceFormulaRootId, SourceFormulaRootOwnership,
    };

    let (b4c_ast, b4c_module, b4c_symbols, b4c) = task258b4c_output(266_800);
    let (b4a_ast, b4a_module, b4a_symbols, b4a) = task258b4a_output(266_801);
    let (b4b_ast, b4b_module, b4b_symbols, b4b) = task258b4b_output(266_802);
    let b4c_before = format!("{b4c:?}");
    let b4a_before = format!("{b4a:?}");
    let b4b_before = format!("{b4b:?}");
    let b4c_lower = source_formula_composition_output_with_source(
        &b4c_ast,
        b4c_module.clone(),
        &b4c_symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("B4C lower selector")
    .expect("B4C lower route");
    let b4a_lower = source_formula_composition_output_with_source(
        &b4a_ast,
        b4a_module.clone(),
        &b4a_symbols,
        SOURCE_STATEMENT_B4A_TEXT,
    )
    .expect("B4A lower selector")
    .expect("B4A lower route");
    let b4b_lower = source_formula_composition_output_with_source(
        &b4b_ast,
        b4b_module.clone(),
        &b4b_symbols,
        SOURCE_STATEMENT_B4B_TEXT,
    )
    .expect("B4B lower selector")
    .expect("B4B lower route");
    let (atomic_ast, atomic_module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_STATEMENT_TEXT, 266_803);
    assert_eq!(diagnostics, 0);
    let (atomic_typed, atomic_resolved) = task258_native_output(&atomic_ast, &atomic_module);
    let atomic_statement = atomic_typed
        .source_statement()
        .expect("atomic statement")
        .clone();
    assert!(atomic_statement.composite_formula_fingerprint().is_none());
    assert_eq!(format!("{b4c:?}"), b4c_before);
    assert_eq!(format!("{b4a:?}"), b4a_before);
    assert_eq!(format!("{b4b:?}"), b4b_before);
    for composite in [&b4a.composite, &b4b.composite, &b4c.composite] {
        assert_eq!(
            composite
                .roots()
                .get(SourceFormulaRootId::new(0))
                .expect("root")
                .ownership(),
            SourceFormulaRootOwnership::UnassignedStatement
        );
    }
    let (_, _, _, replay) = task258b4c_output(266_800);
    assert_eq!(format!("{replay:?}"), b4c_before);
    for (ast, module, symbols, source, lower) in [
        (
            &b4a_ast,
            &b4a_module,
            &b4a_symbols,
            SOURCE_STATEMENT_B4A_TEXT,
            &b4a_lower,
        ),
        (
            &b4b_ast,
            &b4b_module,
            &b4b_symbols,
            SOURCE_STATEMENT_B4B_TEXT,
            &b4b_lower,
        ),
        (
            &b4c_ast,
            &b4c_module,
            &b4c_symbols,
            SOURCE_STATEMENT_B4C_TEXT,
            &b4c_lower,
        ),
    ] {
        let replay =
            source_formula_composition_output_with_source(ast, module.clone(), symbols, source)
                .expect("lower replay selector")
                .expect("pristine lower replay");
        assert_eq!(replay.typed_ast, lower.typed_ast);
        assert_eq!(replay.resolved, lower.resolved);
    }
    let (atomic_replay, atomic_final_replay) = task258_native_output(&atomic_ast, &atomic_module);
    assert_eq!(atomic_replay, atomic_typed);
    assert_eq!(atomic_final_replay, atomic_resolved);
    let (_, _, _, replay_b) = task258b4b_output(266_802);
    assert_eq!(format!("{replay_b:?}"), b4b_before);
}

#[test]
fn task258b4c_final_clone_debug_and_semantic_outputs_are_stable() {
    let (ast, module, symbols, inputs) = task258b4c_output(266_900);
    assert_eq!(format!("{:?}", inputs.clone()), format!("{inputs:?}"));
    let output = source_formula_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("independent lower selector")
    .expect("independent lower output");
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
    assert_eq!(output.typed_ast.source_term(), Some(&inputs.primary));
    assert_eq!(
        output.typed_ast.source_atomic_formula(),
        Some(&inputs.atomic)
    );
    assert_eq!(
        output.typed_ast.source_composite_formula(),
        Some(&inputs.composite)
    );
    assert_eq!(
        output.typed_ast.source_formula_composition(),
        Some(&inputs.composition)
    );
    assert_eq!(
        output.resolved.source_composite_formula(),
        Some(&inputs.composite)
    );
    assert_eq!(
        output.resolved.source_formula_composition(),
        Some(&inputs.composition)
    );
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
}
