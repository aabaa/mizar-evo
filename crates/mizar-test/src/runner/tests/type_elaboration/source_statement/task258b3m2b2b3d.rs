#[test]
fn task258b3m2b2b3d_real_frontend_freezes_qua_witness_contract() {
    use mizar_checker::{
        source_set_term::{
            SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermId,
            SourceSetTermKind, SourceSetTypeHead, SourceSetTypeOwner, SourceSetTypeRole,
            SourceSetTypeSiteId,
        },
        source_statement::{
            SourceStatementWitnessId, SourceStatementWitnessKind, SourceStatementWitnessTermTarget,
        },
        source_term::SourcePrimaryTermKind,
    };

    assert_eq!(SOURCE_STATEMENT_B3M2B2B3D_TEXT.len(), 109);
    assert!(SOURCE_STATEMENT_B3M2B2B3D_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B3D_TEXT),
        "e8fc77b3ed3bbf4313c59f6be389fe369f7d6d7e67a987e5771de9f94f37b34f"
    );
    let (ast, module, symbols, output) = task258b3m2b2b3d_output(261_000);
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

    let statements = output.typed_ast.source_statement().expect("Task258");
    assert_eq!(
        (
            statements.binding_env().contexts().len(),
            statements.binding_env().bindings().len(),
            statements.binding_env().diagnostics().len(),
            statements.owners().len(),
            statements.statements().len(),
            statements.contexts().len(),
            statements.input_facts().len(),
            statements.candidate_facts().len(),
        ),
        (2, 1, 0, 1, 2, 2, 2, 2)
    );
    let proof = statements
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert!(matches!(
        proof.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
            if source_range == range(ast.source_id, 66, 107)
    ));
    assert_eq!(
        proof.parent,
        Some(mizar_checker::binding_env::BindingContextId::new(0))
    );

    let primary = output.typed_ast.source_term().expect("Task252");
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

    let set = output.typed_ast.source_set_term().expect("Task255");
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

    let atomic = output.typed_ast.source_atomic_formula().expect("Task256");
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
        statements
            .input_facts()
            .iter()
            .map(|(_, row)| row.uses().iter().map(|id| id.index()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [vec![0, 1], vec![2, 3]]
    );

    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("witness");
    assert_eq!(
        (witnesses.witnesses().len(), witnesses.names().len()),
        (1, 0)
    );
    let witness = witnesses
        .witnesses()
        .get(SourceStatementWitnessId::new(0))
        .expect("witness row");
    assert_eq!(
        witness.term(),
        SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
    );
    assert_eq!(witness.take_site().node().index(), 40);
    assert_eq!(witness.site().node().index(), 39);
    assert_eq!(witness.binding_context().index(), 1);
    assert_eq!(witness.source_ordinal(), 1);
    assert_eq!(witness.ordinal(), 0);
    assert_eq!(witness.spelling(), "4 qua set");
    assert_eq!(witness.kind(), SourceStatementWitnessKind::Unnamed);
    assert!(witness.name().is_none());

    assert_eq!(
        output
            .typed_ast
            .nodes()
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
    assert_eq!(output.reference_use_ordinals, [1; 4]);
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert_eq!(
        output.typed_ast.source_set_term(),
        output.resolved.source_set_term()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
}

#[test]
fn task258b3m2b2b3d_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols, baseline) = task258b3m2b2b3d_output(261_010);
    let (_, _, _, stale) = task258b3m2b2b3d_output(261_011);

    for mutation in 0..5 {
        let error = source_statement_b3m2b2b3d_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            |input: &mut SourceStatementB3M2B2B3DRouteInputs| match mutation {
                0 => {
                    input.arena = mizar_checker::typed_ast::TypedArena::try_new(None, Vec::new())
                        .expect("empty arena")
                }
                1 => {
                    input.binding_env = stale
                        .typed_ast
                        .source_statement()
                        .expect("statement")
                        .binding_env()
                        .clone()
                }
                2 => input.primary = stale.typed_ast.source_term().expect("primary").clone(),
                3 => {
                    input.atomic = stale
                        .typed_ast
                        .source_atomic_formula()
                        .expect("atomic")
                        .clone()
                }
                4 => input.set_term = None,
                _ => unreachable!(),
            },
        )
        .expect("selector")
        .expect_err("lower mutation must fail");
        assert!(
            error.to_ascii_lowercase().contains("dependency")
                || error.to_ascii_lowercase().contains("invalid"),
            "mutation {mutation}: {error}"
        );
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
        assert!(
            error.starts_with("Task258B3M2B2B3D resolver"),
            "{mutation:?}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        )
        .expect("resolver replay selector")
        .expect("resolver replay");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline.typed_ast.debug_text()
        );
        assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
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
        assert!(
            error.to_ascii_lowercase().contains("resolver")
                || error.to_ascii_lowercase().contains("owner"),
            "{mutation:?}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        )
        .expect("label replay selector")
        .expect("label replay");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline.typed_ast.debug_text()
        );
        assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
    }

    assert_eq!(
        (
            TASK258B3M2B2B3D_TASK48_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK252_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK255_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK256_FIELD_COUNT,
            TASK258B3M2B2B3D_TASK258_FIELD_COUNT,
            TASK258B3M2B2B3D_WITNESS_FIELD_COUNT,
        ),
        (32, 70, 44, 72, 62, 21)
    );
    for (stage, fields, error_prefix) in [
        (
            SourceStatementB3M2B2B3DLowerStage::Task48,
            TASK258B3M2B2B3D_TASK48_FIELD_COUNT,
            "Task48:",
        ),
        (
            SourceStatementB3M2B2B3DLowerStage::Task252,
            TASK258B3M2B2B3D_TASK252_FIELD_COUNT,
            "Task252:",
        ),
        (
            SourceStatementB3M2B2B3DLowerStage::Task255,
            TASK258B3M2B2B3D_TASK255_FIELD_COUNT,
            "Task255:",
        ),
    ] {
        for field in 0..fields {
            let error = source_statement_b3m2b2b3d_output_with_lower_stage_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3D_TEXT,
                stage,
                field,
            )
            .expect("lower-stage selector")
            .expect_err("lower-stage mutation must fail");
            assert!(
                error.starts_with(error_prefix),
                "lower stage {stage:?} field {field}: {error}"
            );
            assert!(
                !error.contains("lower dependency mismatch"),
                "lower stage {stage:?} field {field}: {error}"
            );
            let replay = source_statement_output_with_source(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            )
            .expect("replay selector")
            .expect("replay");
            assert_eq!(
                replay.typed_ast.debug_text(),
                baseline.typed_ast.debug_text()
            );
            assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
        }
    }

    for (stage, fields, error_prefix) in [
        (
            SourceStatementB3M2B2B3DStage::Task256,
            TASK258B3M2B2B3D_TASK256_FIELD_COUNT,
            "Task256:",
        ),
        (
            SourceStatementB3M2B2B3DStage::Task258,
            TASK258B3M2B2B3D_TASK258_FIELD_COUNT,
            "Task258:",
        ),
        (
            SourceStatementB3M2B2B3DStage::Witness,
            TASK258B3M2B2B3D_WITNESS_FIELD_COUNT,
            "B3D:",
        ),
    ] {
        for field in 0..fields {
            let error = source_statement_b3m2b2b3d_output_with_stage_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3D_TEXT,
                stage,
                field,
            )
            .expect("selector")
            .expect_err("stage mutation must fail");
            assert!(
                error.starts_with(error_prefix),
                "stage {stage:?} field {field}: {error}"
            );
            assert!(
                !error.contains("lower dependency mismatch"),
                "stage {stage:?} field {field}: {error}"
            );
            let replay = source_statement_output_with_source(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            )
            .expect("replay selector")
            .expect("replay");
            assert_eq!(
                replay.typed_ast.debug_text(),
                baseline.typed_ast.debug_text()
            );
            assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
        }
    }
}

#[test]
fn task258b3m2b2b3d_qua_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols) = task258b3m2b2b3d_fixture(261_020);
    assert!(extract_qua_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3D_TEXT).is_some());

    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B3D_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B3D_TEXT.as_bytes().to_vec();
        bytes[byte_index] = b'?';
        let source = String::from_utf8(bytes).expect("ASCII source");
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "byte {byte_index}"
        );
    }
    for node in 0..54 {
        for mutation in [
            SourceStatementB3M2B2B3DSurfaceMutation::NodeKind(node),
            SourceStatementB3M2B2B3DSurfaceMutation::NodeRange(node),
            SourceStatementB3M2B2B3DSurfaceMutation::NodeRecovery(node),
            SourceStatementB3M2B2B3DSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                extract_qua_witness_source_statement_with_surface_mutation(
                    &ast,
                    SOURCE_STATEMENT_B3M2B2B3D_TEXT,
                    mutation,
                )
                .is_none(),
                "{mutation:?}"
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
    let (ast, module, symbols, b3d) = task258b3m2b2b3d_output(261_200);
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
    let set = b3d.typed_ast.source_set_term().expect("B3D set");
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
        source_statement_output_with_source(
            &case_ast,
            case_module,
            &case_symbols,
            SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        )
        .expect("application selector")
        .expect("application route")
        .typed_ast
        .source_application()
        .expect("application")
        .clone()
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
        source_statement_output_with_source(
            &case_ast,
            case_module,
            &case_symbols,
            SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        )
        .expect("structure selector")
        .expect("structure route")
        .typed_ast
        .source_structure()
        .expect("structure")
        .clone()
    };
    let set = set.clone();
    for tuple in 0u8..8 {
        let result = source_statement_b3m2b2b3d_output_with_post_auth_family_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3D_TEXT,
            |input| {
                input.application = (tuple & 0b001 != 0).then(|| application.clone());
                input.structure = (tuple & 0b010 != 0).then(|| structure.clone());
                input.set_term = (tuple & 0b100 != 0).then(|| set.clone());
            },
        )
        .expect("selector");
        if tuple == 0b100 {
            assert!(result.is_ok(), "set-only tuple: {result:?}");
        } else {
            let error = result.expect_err("all non-set-only tuples must fail");
            assert!(
                error.to_ascii_lowercase().contains("dependency")
                    || error.to_ascii_lowercase().contains("exclusive")
                    || error.to_ascii_lowercase().contains("invalid"),
                "tuple {tuple:03b}: {error}"
            );
        }
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
        if source == SOURCE_STATEMENT_B3M2B2B3C_TEXT {
            let before = (case_ast.snapshot_text(), case_module.clone(), case_symbols.clone());
            assert!(source_statement_output_with_source(
                &case_ast, case_module.clone(), &case_symbols, source,
            ).is_none());
            assert!(source_statement_transport_detail_keys(
                &case_ast, case_module.clone(), &case_symbols, source,
            ).is_none());
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
        .expect("exact private detail route")
        .is_empty()
    );
}

#[test]
fn task258b3m2b2b3d_typed_final_clone_debug_rollback_and_qua_semantics_are_stable() {
    use mizar_checker::source_set_term::{
        SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget,
    };

    let (ast, module, symbols, output) = task258b3m2b2b3d_output(261_800);
    let replay = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3D_TEXT,
    )
    .expect("selector")
    .expect("replay");
    assert_eq!(output.typed_ast.debug_text(), replay.typed_ast.debug_text());
    assert_eq!(output.resolved.debug_text(), replay.resolved.debug_text());
    assert_eq!(
        output.typed_ast.clone().debug_text(),
        output.typed_ast.debug_text()
    );
    assert_eq!(
        output.resolved.clone().debug_text(),
        output.resolved.debug_text()
    );

    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("witnesses");
    let debug = witnesses.debug_text();
    let lines = debug.lines().collect::<Vec<_>>();
    let statement = lines
        .iter()
        .position(|line| line.starts_with("statement-fingerprint:"))
        .expect("statement fingerprint");
    let primary = lines
        .iter()
        .position(|line| line.starts_with("primary-term-fingerprint:"))
        .expect("primary fingerprint");
    let set = lines
        .iter()
        .position(|line| line.starts_with("set-term-fingerprint:"))
        .expect("set fingerprint");
    let row = lines
        .iter()
        .position(|line| line.starts_with("witness#0 "))
        .expect("witness row");
    assert!(statement < primary && primary < set && set < row);
    assert!(!debug.contains("\napplication-fingerprint:"));
    assert!(!debug.contains("\nstructure-fingerprint:"));
    assert!(debug.contains(" term=set-term#0 "));

    let set_term = output.typed_ast.source_set_term().expect("set term");
    let edge = set_term.edges().iter().next().expect("qua edge").1;
    assert_eq!(edge.role(), SourceSetEdgeRole::QuaBase);
    assert_eq!(
        edge.target(),
        SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(2))
    );
    assert_eq!(
        set_term
            .requests()
            .iter()
            .map(|(_, request)| request.kind())
            .collect::<Vec<_>>(),
        [
            SourceSetRequestKind::QuaWidening,
            SourceSetRequestKind::ResultType
        ]
    );

    let baseline = output.typed_ast.debug_text();
    let error = source_statement_b3m2b2b3d_output_with_mutation(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        |input| input.witness.witnesses.clear(),
    )
    .expect("rollback selector")
    .expect_err("empty witness aggregate must roll back");
    assert!(error.to_ascii_lowercase().contains("aggregate"), "{error}");
    assert_eq!(output.typed_ast.debug_text(), baseline);

    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert_eq!(
        output.typed_ast.source_set_term(),
        output.resolved.source_set_term()
    );
    assert_eq!(
        output.typed_ast.source_atomic_formula(),
        output.resolved.source_atomic_formula()
    );
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert!(output.typed_ast.source_application().is_none());
    assert!(output.typed_ast.source_structure().is_none());
    assert!(output.typed_ast.source_statement_references().is_none());
    assert!(output.typed_ast.source_context().is_none());
    assert!(output.typed_ast.source_type().is_none());
    assert!(output.typed_ast.source_attribute().is_none());
    assert!(output.typed_ast.source_evidence().is_none());
    assert!(output.typed_ast.source_composite_formula().is_none());
    assert!(output.typed_ast.source_formula_composition().is_none());
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
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
}
