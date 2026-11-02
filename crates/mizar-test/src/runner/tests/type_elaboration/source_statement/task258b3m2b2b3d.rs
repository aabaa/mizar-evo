#[test]
fn task258b3m2b2b3d_actual_active_plan_has_no_qua_witness_route() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task258B3M2B2B3D active isolation plan");
    let (mut scanned, mut with_ast, mut resolved) = (0, 0, 0);
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        scanned += 1;
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            // Frontend refusal cases may have no AST to pass to the resolver.
            continue;
        };
        with_ast += 1;
        let extracted = extract_qua_witness_source_statement(&ast, &source).is_some();
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        let selected_route = if resolver.detail_keys.is_empty() {
            resolved += 1;
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
                .is_some_and(|result| {
                    result.is_ok_and(|output| {
                        output.typed_ast.source_statement_witnesses().is_some_and(|witnesses| {
                            witnesses.witnesses().len() == 1 && witnesses.names().is_empty()
                        }) && output.typed_ast.source_term().is_some_and(|primary| {
                            primary.numeric_type_requests().len() == 1
                        }) && output.typed_ast.source_set_term().is_some_and(|set| {
                            set.terms().len() == 1
                                && set.terms().iter().all(|(_, term)| {
                                    term.kind() == mizar_checker::source_set_term::SourceSetTermKind::Qua
                                })
                        })
                    })
                })
        } else {
            false
        };
        if extracted || selected_route {
            selected.push(case.id.0.clone());
        }
    }
    eprintln!(
        "Qua active isolation: scanned={scanned} with_ast={with_ast} resolved={resolved} selected={}",
        selected.len()
    );
    assert!(
        scanned > 0 && with_ast > 0 && resolved > 0,
        "empty active scan"
    );
    assert!(
        selected.is_empty(),
        "Qua selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b2b3d_real_frontend_freezes_qua_witness_contract() {
    use mizar_checker::{
        source_set_term::{
            SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermId,
            SourceSetTermKind, SourceSetTypeHead, SourceSetTypeOwner, SourceSetTypeRole,
            SourceSetTypeSiteId,
        },
        source_statement::{SourceStatementWitnessKind, SourceStatementWitnessTermTarget},
        source_term::SourcePrimaryTermKind,
    };

    assert_eq!(SOURCE_STATEMENT_B3M2B2B3D_TEXT.len(), 109);
    assert!(SOURCE_STATEMENT_B3M2B2B3D_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B3D_TEXT),
        "e8fc77b3ed3bbf4313c59f6be389fe369f7d6d7e67a987e5771de9f94f37b34f"
    );
    let (ast, module, symbols, lower) = task258b3m2b2b3d_lower_inputs(261_000);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (54, Some(53))
    );
    let extracted: SourceStatementB3M2B2B3DExtraction =
        extract_qua_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3D_TEXT)
            .expect("exact qua-witness extraction");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range,
            extracted.label_range,
            extracted.take_site.node().index(),
            extracted.take_range,
            extracted.witness_site.node().index(),
            extracted.witness_range,
            extracted.set_term_node,
            extracted.proof_range,
        ),
        (
            50,
            range(ast.source_id, 19, 108),
            range(ast.source_id, 27, 58),
            40,
            range(ast.source_id, 74, 89),
            39,
            range(ast.source_id, 79, 88),
            37,
            range(ast.source_id, 66, 107),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [28, 30, 34, 41, 43]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [32, 45]
    );

    let resolver =
        source_statement_b3m2b2b3d_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("resolver");
    assert_eq!(resolver.module_id(), &module);
    assert!(resolver.imports().is_empty());
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver
        .symbols()
        .visible_candidates(&namespace, "FormulaStatementQuaWitnessSmoke");
    let [owner] = owners.as_slice() else {
        panic!("one exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 108))
    );
    assert_eq!(owner.origin().structural_path(), [2, 1]);
    assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert!(owner.origin().import_edge().is_none());
    assert!(!owner.origin().is_recovered());
    let labels = resolver
        .labels()
        .visible_candidates(&namespace, "FormulaStatementQuaWitnessSmoke");
    let [label] = labels.as_slice() else {
        panic!("one exact theorem label");
    };
    assert_eq!(label.origin(), owner.origin());
    assert_eq!(label.contribution(), owner.contribution());
    assert_eq!(
        resolver
            .contributions()
            .get(owner.contribution())
            .expect("contribution")
            .anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 18))
    );

    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    assert!(
        source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        )
        .is_none()
    );
    assert!(
        source_statement_transport_detail_keys(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        )
        .is_none()
    );
    assert_eq!(
        (ast.snapshot_text(), module.clone(), symbols.clone()),
        before
    );
    assert_eq!(
        (
            lower.binding_env.contexts().len(),
            lower.binding_env.bindings().len(),
            lower.binding_env.diagnostics().len(),
            lower.statement.owners.len(),
            lower.statement.statements.len(),
            lower.statement.contexts.len(),
            lower.statement.input_facts.len(),
            lower.statement.candidate_facts.len()
        ),
        (2, 1, 0, 1, 2, 2, 2, 2)
    );
    let proof = lower
        .binding_env
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert!(matches!(proof.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
            if source_range == range(ast.source_id, 66, 107)));
    assert_eq!(
        proof.parent,
        Some(mizar_checker::binding_env::BindingContextId::new(0))
    );

    let primary = &lower.primary;
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (5, 4, 1)
    );
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(_, row)| (row.site().node().index(), row.context().index(), row.kind()))
            .collect::<Vec<_>>(),
        [
            (28, 0, SourcePrimaryTermKind::VariableReference),
            (30, 0, SourcePrimaryTermKind::VariableReference),
            (34, 1, SourcePrimaryTermKind::Numeral),
            (41, 1, SourcePrimaryTermKind::VariableReference),
            (43, 1, SourcePrimaryTermKind::VariableReference),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| (row.term().index(), row.binding().index(), row.use_ordinal()))
            .collect::<Vec<_>>(),
        [(0, 0, 1), (1, 0, 1), (3, 0, 1), (4, 0, 1)]
    );
    let numeric = primary
        .numeric_type_requests()
        .iter()
        .next()
        .expect("numeric request")
        .1;
    assert_eq!(numeric.term().index(), 2);
    assert_eq!(numeric.owner().node().index(), 34);
    assert_eq!(numeric.source_range(), range(ast.source_id, 79, 80));
    assert_eq!(numeric.spelling(), "4");

    let set = lower.set_term.as_ref().expect("Task255");
    assert_eq!(
        (
            set.terms().len(),
            set.wrappers().len(),
            set.generators().len(),
            set.type_sites().len(),
            set.conditions().len(),
            set.edges().len(),
            set.requests().len(),
        ),
        (1, 0, 0, 1, 0, 1, 2)
    );
    let set_row = set.terms().get(SourceSetTermId::new(0)).expect("qua");
    assert_eq!(set_row.site().node().index(), 37);
    assert_eq!(set_row.source_range(), range(ast.source_id, 79, 88));
    assert_eq!(set_row.context().index(), 1);
    assert_eq!(set_row.spelling(), "4 qua set");
    assert_eq!(set_row.kind(), SourceSetTermKind::Qua);
    let type_site = set
        .type_sites()
        .get(SourceSetTypeSiteId::new(0))
        .expect("qua target");
    assert_eq!(
        type_site.owner(),
        SourceSetTypeOwner::Term {
            term: SourceSetTermId::new(0),
            role: SourceSetTypeRole::QuaTarget,
        }
    );
    assert_eq!(type_site.site().node().index(), 36);
    assert_eq!(type_site.head_site().node().index(), 35);
    assert_eq!(type_site.head(), SourceSetTypeHead::BuiltinSet);
    assert_eq!(
        set.edges()
            .iter()
            .map(|(_, row)| (row.term().index(), row.ordinal(), row.role(), row.target()))
            .collect::<Vec<_>>(),
        [(
            0,
            0,
            SourceSetEdgeRole::QuaBase,
            SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(2)),
        )]
    );
    assert_eq!(
        set.requests()
            .iter()
            .map(|(_, row)| (
                row.term().index(),
                row.ordinal(),
                row.kind(),
                row.type_site()
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                SourceSetRequestKind::QuaWidening,
                Some(SourceSetTypeSiteId::new(0)),
            ),
            (0, 1, SourceSetRequestKind::ResultType, None),
        ]
    );

    let atomic = &lower.atomic;
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len()
        ),
        (2, 4, 4)
    );
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(id) => {
                    id.index()
                }
                _ => usize::MAX,
            })
            .collect::<Vec<_>>(),
        [0, 1, 3, 4]
    );
    assert_eq!(
        lower
            .statement
            .input_facts
            .iter()
            .map(|row| row.uses.iter().map(|id| id.index()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [vec![0, 1], vec![2, 3]]
    );

    assert_eq!(
        (lower.witness.witnesses.len(), lower.witness.names.len()),
        (1, 0)
    );
    let witness = &lower.witness.witnesses[0];
    assert_eq!(
        witness.term,
        SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
    );
    assert_eq!(
        (
            witness.take_site.node().index(),
            witness.site.node().index(),
            witness.binding_context.index(),
            witness.source_ordinal,
            witness.ordinal
        ),
        (40, 39, 1, 1, 0)
    );
    assert_eq!(witness.spelling, "4 qua set");
    assert_eq!(witness.kind, SourceStatementWitnessKind::Unnamed);
    assert!(witness.name.is_none());
    assert_eq!(
        lower
            .arena
            .iter()
            .filter(|(_, node)| node.kind.as_str() != "source.surface.unowned")
            .map(|(id, node)| (id.index(), node.kind.as_str()))
            .collect::<Vec<_>>(),
        [
            (28, "source.term.variable-reference"),
            (30, "source.term.variable-reference"),
            (32, "source.formula.atomic.equality"),
            (34, "source.term.numeral"),
            (35, "source.term.set.target-type-head"),
            (36, "source.term.set.target-type"),
            (37, "source.term.set.qua"),
            (39, "source.statement-witness.item"),
            (40, "source.statement-witness.take"),
            (41, "source.term.variable-reference"),
            (43, "source.term.variable-reference"),
            (45, "source.formula.atomic.equality"),
            (48, "source.statement.conclusion"),
            (50, "source.statement.theorem"),
        ]
    );
}

#[test]
fn task258b3m2b2b3d_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols, baseline) = task258b3m2b2b3d_lower_inputs(261_010);
    let (_, _, _, stale) = task258b3m2b2b3d_lower_inputs(261_011);
    let baseline_debug = format!("{baseline:?}");
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let replay = || {
        let (_, _, _, pristine) = task258b3m2b2b3d_lower_inputs(261_010);
        assert_eq!(format!("{pristine:?}"), baseline_debug);
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
    };
    for mutation in 0..5 {
        let error = source_statement_b3m2b2b3d_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            |input| match mutation {
                0 => {
                    input.arena = mizar_checker::typed_ast::TypedArena::try_new(None, Vec::new())
                        .expect("empty arena")
                }
                1 => input.binding_env = stale.binding_env.clone(),
                2 => input.primary = stale.primary.clone(),
                3 => input.atomic = stale.atomic.clone(),
                4 => input.set_term = None,
                _ => unreachable!(),
            },
        )
        .expect("authentic selector")
        .expect_err("lower authentication");
        assert_eq!(
            error, "Task258B3M2B2B3D lower dependency mismatch",
            "mutation {mutation}"
        );
        replay();
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
        let error = source_statement_b3m2b2b3d_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("resolver selector")
        .expect_err("resolver mutation must fail");
        let expected = match mutation {
            Task258B2ResolverMutation::Missing | Task258B2ResolverMutation::Duplicate => {
                "Task258B3M2B2B3D resolver theorem label mismatch"
            }
            _ => "Task258B3M2B2B3D resolver theorem provenance mismatch",
        };
        assert_eq!(error, expected, "{mutation:?}");
        replay();
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
        let error = source_statement_b3m2b2b3d_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            |symbols| task258b3m2b2b3a_mutate_label_resolver(symbols, mutation),
        )
        .expect("selector")
        .expect_err("label mutation must fail");
        let expected = match mutation {
            Task258B3M2B2B3ALabelMutation::Namespace
            | Task258B3M2B2B3ALabelMutation::PrimarySpelling => {
                "Task258B3M2B2B3D resolver theorem label mismatch"
            }
            _ => "Task258B3M2B2B3D resolver theorem provenance mismatch",
        };
        assert_eq!(error, expected, "{mutation:?}");
        replay();
    }
    assert_eq!(
        (
            TASK258B3M2B2B3D_TASK48_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK252_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK255_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK256_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK258_FIELD_COUNT,
            TASK258B3M2B2B3D_WITNESS_FIELD_COUNT
        ),
        (32, 70, 44, 72, 62, 21)
    );
    for (stage, fields, prefix) in [
        (SourceStatementB3M2B2B3DLowerStage::Task48, 32, "Task48:"),
        (SourceStatementB3M2B2B3DLowerStage::Task252, 70, "Task252:"),
        (SourceStatementB3M2B2B3DLowerStage::Task255, 44, "Task255:"),
    ] {
        for field in 0..fields {
            match source_statement_b3m2b2b3d_output_with_lower_stage_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3D_TEXT,
                stage,
                field,
            )
            .expect("authentic lower extraction")
            {
                Err(error) => assert!(error.starts_with(prefix), "{stage:?} {field}: {error}"),
                Ok(changed) => match stage {
                    SourceStatementB3M2B2B3DLowerStage::Task48 => {
                        assert_ne!(changed.binding_env, baseline.binding_env, "field {field}")
                    }
                    SourceStatementB3M2B2B3DLowerStage::Task252 => {
                        assert_ne!(changed.primary, baseline.primary, "field {field}")
                    }
                    SourceStatementB3M2B2B3DLowerStage::Task255 => {
                        assert_ne!(changed.set_term, baseline.set_term, "field {field}")
                    }
                },
            }
            replay();
        }
    }
    for field in 0..TASK258B3M2B2B3D_TASK256_FIELD_COUNT {
        match source_statement_b3m2b2b3d_output_with_stage_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            SourceStatementB3M2B2B3DStage::Task256,
            field,
        )
        .expect("authentic atomic extraction")
        {
            Err(error) => assert!(error.starts_with("Task256:"), "{field}: {error}"),
            Ok(changed) => assert_ne!(changed.atomic, baseline.atomic, "field {field}"),
        }
        replay();
    }
    // Task258/witness field causality remains in the Checker-private Qua matrices.
    assert_eq!((ast.snapshot_text(), module, symbols), before);
}

#[test]
fn task258b3m2b2b3d_qua_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols) = task258b3m2b2b3d_fixture(261_020);
    assert!(extract_qua_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3D_TEXT).is_some());
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());

    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B3D_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B3D_TEXT.as_bytes().to_vec();
        bytes[byte_index] = b'?';
        let source = String::from_utf8(bytes).expect("ASCII source");
        assert!(
            extract_qua_witness_source_statement(&ast, &source).is_none(),
            "byte {byte_index}"
        );
        assert!(
            source_statement_b3m2b2b3d_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                &source,
                |_| panic!("inauthentic source admitted"),
            )
            .is_none(),
            "byte {byte_index}"
        );
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
        assert!(
            extract_qua_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3D_TEXT).is_some()
        );
    }
    for target in 0..55 {
        for mutation in 0..if target == 54 { 1 } else { 4 } {
            let changed = task258_mutated_surface(&ast, target, mutation);
            assert!(
                extract_qua_witness_source_statement(&changed, SOURCE_STATEMENT_B3M2B2B3D_TEXT)
                    .is_none(),
                "node {target} mutation {mutation}"
            );
            assert!(
                source_statement_b3m2b2b3d_output_with_resolver_mutation(
                    &changed,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B3M2B2B3D_TEXT,
                    |symbols| symbols,
                )
                .is_none(),
                "node {target} mutation {mutation} extraction gate"
            );
            assert_eq!(
                (ast.snapshot_text(), module.clone(), symbols.clone()),
                before
            );
            assert!(
                extract_qua_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3D_TEXT)
                    .is_some()
            );
        }
    }
    assert!(
        extract_qua_witness_source_statement_with_surface_mutation(
            &ast,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            SourceStatementB3M2B2B3DSurfaceMutation::RootIdentity,
        )
        .is_none()
    );

    let changed_label = SOURCE_STATEMENT_B3M2B2B3D_TEXT.replacen(
        "FormulaStatementQuaWitnessSmoke",
        "FormulaStatementQuaWitnessNearMiss",
        1,
    );
    let mut near_misses = vec![changed_label];
    for replacement in [
        "5 qua set",
        "x qua set",
        "4 qua object",
        "4 qua non empty set",
        "4 qua set attr",
        "4 qua set(x)",
        "(4 qua set)",
        "(4) qua set",
        "4 qua set qua set",
        "4",
        "the set",
        "{}",
        "{4}",
        "{x where x is set : x = x}",
        "TypeCaseStruct(x: 1, y: 2)",
    ] {
        near_misses.push(SOURCE_STATEMENT_B3M2B2B3D_TEXT.replacen("4 qua set", replacement, 1));
    }
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3D_TEXT.replacen(
        "take 4 qua set",
        "take y = 4 qua set",
        1,
    ));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3D_TEXT.replacen(
        "take 4 qua set",
        "take 4 qua set, 4 qua set",
        1,
    ));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3D_TEXT.replace('\n', "\r\n"));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3D_TEXT.trim_end().to_owned());
    near_misses.push(format!("{SOURCE_STATEMENT_B3M2B2B3D_TEXT}\n"));

    for (index, source) in near_misses.into_iter().enumerate() {
        let (near_ast, near_module, _, near_symbols, _) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 261_100 + index);
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_symbols,
                &source,
            )
            .is_none(),
            "near miss {index}: {source:?}"
        );
        assert!(
            source_statement_transport_detail_keys(&near_ast, near_module, &near_symbols, &source)
                .is_none(),
            "near-miss detail route {index}"
        );
    }
}

#[test]
fn task258b3m2b2b3d_family_and_active_route_isolation_is_atomic() {
    let (ast, module, symbols, lower) = task258b3m2b2b3d_lower_inputs(261_200);
    let baselines = [
        task258b3m2b2b3_family_debug(0, 261_201),
        task258b3m2b2b3_family_debug(1, 261_202),
        task258b3m2b2b3_family_debug(2, 261_203),
        task258b3m2b2b3_family_debug(3, 261_204),
    ];
    for left in 0..4 {
        for right in (left + 1)..4 {
            assert_ne!(baselines[left], baselines[right], "{left}/{right}");
        }
    }
    let set = lower.set_term.as_ref().expect("B3D set");
    assert_eq!(set.edges().len(), 1);

    let application = {
        let (case_ast, case_module, _, case_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(
                SOURCE_STATEMENT_B3M2B2B1A_TEXT,
                261_205,
            );
        assert_eq!(diagnostics, 0);
        let case_symbols =
            augment_type_elaboration_import_summaries(&case_ast, &case_module, case_symbols);
        task258b3m2b2b1a_lower_inputs(&case_ast, &case_module, &case_symbols)
            .application
            .expect("genuine bare application")
    };
    let structure = {
        let (case_ast, case_module, _, case_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(
                SOURCE_STATEMENT_B3M2B2B2A_TEXT,
                261_206,
            );
        assert_eq!(diagnostics, 0);
        let case_symbols =
            augment_type_elaboration_import_summaries(&case_ast, &case_module, case_symbols);
        task258b3m2b2b2a_lower_inputs(&case_ast, &case_module, &case_symbols)
        .structure.expect("genuine constructor structure")
    };
    let set = set.clone();
    for tuple in 0u8..8 {
        let mut captured = None;
        let error = source_statement_b3m2b2b3d_output_with_post_auth_family_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            |input| {
                input.application = (tuple & 0b001 != 0).then(|| application.clone());
                input.structure = (tuple & 0b010 != 0).then(|| structure.clone());
                input.set_term = (tuple & 0b100 != 0).then(|| set.clone());
                captured = Some((
                    input.application.is_some(),
                    input.structure.is_some(),
                    input.set_term.is_some(),
                ));
            },
        )
        .expect("authentic selector")
        .expect_err("retired raw profile");
        assert_eq!(
            captured,
            Some((tuple & 1 != 0, tuple & 2 != 0, tuple & 4 != 0))
        );
        assert_eq!(error, "source statement dependency mismatch");
    }

    let mut order_index = 0;
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let order = [a, b, c, d];
                    if [a, b, c, d]
                        .iter()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        != 4
                    {
                        continue;
                    }
                    for (position, family) in order.into_iter().enumerate() {
                        assert_eq!(
                            task258b3m2b2b3_family_debug(
                                family,
                                261_300 + order_index * 10 + position,
                            ),
                            baselines[family],
                            "order {order_index} position {position} family {family}"
                        );
                    }
                    order_index += 1;
                }
            }
        }
    }
    assert_eq!(order_index, 24);

    for (source, expects_application, expects_structure, expects_set) in [
        (SOURCE_STATEMENT_B3M2B2B1A_TEXT, true, false, false),
        (SOURCE_STATEMENT_B3M2B2B2A_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B2B_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B2C_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B3A_TEXT, false, false, true),
        (SOURCE_STATEMENT_B3M2B2B3C_TEXT, false, false, true),
        (SOURCE_STATEMENT_B3M2B2B3D_TEXT, false, false, true),
    ] {
        let (case_ast, case_module, _, case_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(source, 261_700 + source.len());
        assert_eq!(diagnostics, 0);
        let case_symbols =
            augment_type_elaboration_import_summaries(&case_ast, &case_module, case_symbols);
        if source == SOURCE_STATEMENT_B3M2B2B2A_TEXT {
            let lower = task258b3m2b2b2a_lower_inputs(&case_ast, &case_module, &case_symbols);
            assert!(lower.application.is_none() && lower.set_term.is_none());
            let structure = lower.structure.expect("genuine constructor structure");
            assert_eq!((structure.terms().len(), structure.field_updates().len()), (1, 0));
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B2B_TEXT {
            let lower = task258b3m2b2b2b_lower_inputs(&case_ast, &case_module, &case_symbols);
            assert!(lower.application.is_none() && lower.set_term.is_none());
            let structure = lower.structure.expect("authenticated selector lower structure");
            assert_eq!((structure.terms().len(), structure.field_updates().len()), (2, 0));
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B2C_TEXT {
            let lower = task258b3m2b2b2c_lower_inputs(&case_ast, &case_module, &case_symbols);
            assert_eq!(
                format!("{lower:?}"),
                format!(
                    "{:?}",
                    task258b3m2b2b2c_lower_inputs(&case_ast, &case_module, &case_symbols)
                )
            );
            assert!(lower.application.is_none() && lower.set_term.is_none());
            let structure = lower
                .structure
                .expect("authenticated update lower structure");
            assert_eq!(
                (structure.terms().len(), structure.field_updates().len()),
                (2, 1)
            );
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B1A_TEXT {
            task258b3m2b2b1a_lower_inputs(&case_ast, &case_module, &case_symbols);
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B3A_TEXT
            || source == SOURCE_STATEMENT_B3M2B2B3C_TEXT
            || source == SOURCE_STATEMENT_B3M2B2B3D_TEXT
        {
            let before = (case_ast.snapshot_text(), case_module.clone(), case_symbols.clone());
            assert!(source_statement_output_with_source(
                &case_ast, case_module.clone(), &case_symbols, source,
            ).is_none());
            assert!(source_statement_transport_detail_keys(
                &case_ast, case_module.clone(), &case_symbols, source,
            ).is_none());
            if source == SOURCE_STATEMENT_B3M2B2B3A_TEXT {
                assert!(
                    extract_set_enumeration_witness_source_statement(&case_ast, source).is_some()
                );
                assert_eq!(
                    source_statement_b3m2b2b3a_output_with_mutation(
                        &case_ast,
                        case_module.clone(),
                        &case_symbols,
                        source,
                        |_| {},
                    )
                    .expect("authentic numeric enumeration extraction")
                    .expect_err("retired raw profile"),
                    "source statement dependency mismatch"
                );
            }
            assert_eq!((case_ast.snapshot_text(), case_module, case_symbols), before);
            continue;
        }
        let output =
            source_statement_output_with_source(&case_ast, case_module, &case_symbols, source)
                .expect("dormant selector")
                .expect("dormant route");
        assert_eq!(
            output.typed_ast.source_application().is_some(),
            expects_application
        );
        assert_eq!(
            output.typed_ast.source_structure().is_some(),
            expects_structure
        );
        assert_eq!(output.typed_ast.source_set_term().is_some(), expects_set);
    }

    assert!(
        source_statement_transport_detail_keys(
            &ast,
            module,
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        )
        .is_none()
    );
}

#[test]
fn task258b3m2b2b3d_raw_refusal_preserves_lower_clone_debug_and_qua_requests() {
    use mizar_checker::source_set_term::{
        SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget,
    };
    let (ast, module, symbols, lower) = task258b3m2b2b3d_lower_inputs(261_800);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let debug = format!("{lower:?}");
    assert_eq!(format!("{:?}", lower.clone()), debug);
    let set = lower.set_term.as_ref().expect("Qua lower set");
    let edge = set.edges().iter().next().expect("Qua edge").1;
    assert_eq!(edge.role(), SourceSetEdgeRole::QuaBase);
    assert_eq!(
        edge.target(),
        SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(2))
    );
    assert_eq!(
        set.requests()
            .iter()
            .map(|(_, request)| request.kind())
            .collect::<Vec<_>>(),
        [
            SourceSetRequestKind::QuaWidening,
            SourceSetRequestKind::ResultType
        ]
    );
    let error = source_statement_b3m2b2b3d_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        |inputs| inputs.witness.witnesses.clear(),
    )
    .expect("authentic extraction")
    .expect_err("raw admission precedes witness validation");
    assert_eq!(error, "source statement dependency mismatch");
    let (_, _, _, replay) = task258b3m2b2b3d_lower_inputs(261_800);
    assert_eq!(format!("{replay:?}"), debug);
    assert_eq!((ast.snapshot_text(), module, symbols), before);
}
