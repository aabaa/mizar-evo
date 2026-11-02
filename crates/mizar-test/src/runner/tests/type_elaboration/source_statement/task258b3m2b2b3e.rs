#[test]
fn task258b3m2b2b3e_real_frontend_freezes_comprehension_witness_contract() {
    use mizar_checker::{
        source_set_term::{
            SourceSetEdgeRole, SourceSetGeneratorId, SourceSetRequestKind, SourceSetTarget,
            SourceSetTermId, SourceSetTermKind, SourceSetTypeHead, SourceSetTypeOwner,
            SourceSetTypeSiteId,
        },
        source_statement::{
            SourceStatementWitnessId, SourceStatementWitnessKind, SourceStatementWitnessTermTarget,
        },
        source_term::SourcePrimaryTermKind,
    };

    assert_eq!(SOURCE_STATEMENT_B3M2B2B3E_TEXT.len(), 139);
    assert!(SOURCE_STATEMENT_B3M2B2B3E_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B3E_TEXT),
        "b3b12979a119c859c3e563eb1aa47fa4601045a686c5a09460ee72873bf7a29d"
    );
    let (ast, module, symbols, output) = task258b3m2b2b3e_output(262_000);
    assert_eq!(
        (
            ast.token_nodes().len(),
            ast.nodes().len(),
            ast.root().map(|id| id.index())
        ),
        (28, 60, Some(59))
    );
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let extracted: SourceStatementB3M2B2B3EExtraction =
        extract_comprehension_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3E_TEXT)
            .expect("exact comprehension-witness extraction");
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
            56,
            range(ast.source_id, 19, 138),
            range(ast.source_id, 27, 68),
            46,
            range(ast.source_id, 84, 119),
            45,
            range(ast.source_id, 89, 118),
            43,
            range(ast.source_id, 76, 137),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [32, 34, 38, 47, 49]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [36, 51]
    );

    let resolver =
        source_statement_b3m2b2b3e_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("resolver");
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver
        .symbols()
        .visible_candidates(&namespace, "FormulaStatementComprehensionWitnessSmoke");
    let [owner] = owners.as_slice() else {
        panic!("one exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 138))
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
        .visible_candidates(&namespace, "FormulaStatementComprehensionWitnessSmoke");
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
            if source_range == range(ast.source_id, 76, 137)
    ));
    assert_eq!(
        (
            proof.parent,
            proof.layer,
            proof.lexical_scope.as_ref().map(|scope| scope.path()),
            proof.visible_bindings.as_slice(),
        ),
        (
            Some(mizar_checker::binding_env::BindingContextId::new(0)),
            mizar_checker::binding_env::BindingContextLayer::Proof,
            Some([0].as_slice()),
            [mizar_checker::binding_env::BindingId::new(0)].as_slice(),
        )
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
            (32, 0, SourcePrimaryTermKind::VariableReference),
            (34, 0, SourcePrimaryTermKind::VariableReference),
            (38, 1, SourcePrimaryTermKind::Numeral),
            (47, 1, SourcePrimaryTermKind::VariableReference),
            (49, 1, SourcePrimaryTermKind::VariableReference),
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
        .expect("numeric")
        .1;
    assert_eq!(
        (
            numeric.term().index(),
            numeric.owner().node().index(),
            numeric.source_range(),
            numeric.spelling(),
        ),
        (2, 38, range(ast.source_id, 90, 91), "3")
    );

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
        (1, 0, 1, 1, 0, 1, 2)
    );
    let set_row = set
        .terms()
        .get(SourceSetTermId::new(0))
        .expect("comprehension");
    assert_eq!(
        (
            set_row.site().node().index(),
            set_row.source_range(),
            set_row.source_ordinal(),
            set_row.context().index(),
            set_row.spelling(),
            set_row.kind(),
        ),
        (
            43,
            range(ast.source_id, 89, 118),
            0,
            1,
            "{ 3 where candidate255 is set }",
            SourceSetTermKind::Comprehension,
        )
    );
    let generator = set
        .generators()
        .get(SourceSetGeneratorId::new(0))
        .expect("generator");
    assert_eq!(
        (
            generator.term().index(),
            generator.ordinal(),
            generator.site().node().index(),
            generator.source_range(),
            generator.spelling(),
            generator.context().index(),
            generator.type_site().index(),
        ),
        (
            0,
            0,
            16,
            range(ast.source_id, 98, 110),
            "candidate255",
            1,
            0,
        )
    );
    let type_site = set
        .type_sites()
        .get(SourceSetTypeSiteId::new(0))
        .expect("type");
    assert_eq!(
        type_site.owner(),
        SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0))
    );
    assert_eq!(
        (
            type_site.site().node().index(),
            type_site.head_site().node().index(),
            type_site.source_range(),
            type_site.head_range(),
            type_site.spelling(),
            type_site.head(),
        ),
        (
            41,
            40,
            range(ast.source_id, 114, 117),
            range(ast.source_id, 114, 117),
            "set",
            SourceSetTypeHead::BuiltinSet,
        )
    );
    assert_eq!(
        set.edges()
            .iter()
            .map(|(_, row)| (row.term().index(), row.ordinal(), row.role(), row.target()))
            .collect::<Vec<_>>(),
        [(
            0,
            0,
            SourceSetEdgeRole::ComprehensionMapper,
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
                row.generator(),
                row.type_site(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                SourceSetRequestKind::GeneratorSethood,
                Some(SourceSetGeneratorId::new(0)),
                Some(SourceSetTypeSiteId::new(0)),
            ),
            (0, 1, SourceSetRequestKind::ResultType, None, None),
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
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("B3E witnesses");
    assert_eq!(
        (witnesses.witnesses().len(), witnesses.names().len()),
        (1, 0)
    );
    let witness = witnesses
        .witnesses()
        .get(SourceStatementWitnessId::new(0))
        .expect("witness");
    assert_eq!(
        witness.term(),
        SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
    );
    assert_eq!(
        (
            witness.take_site().node().index(),
            witness.site().node().index(),
            witness.binding_context().index(),
            witness.source_ordinal(),
            witness.ordinal(),
            witness.spelling(),
            witness.kind(),
            witness.name(),
        ),
        (
            46,
            45,
            1,
            1,
            0,
            "{ 3 where candidate255 is set }",
            SourceStatementWitnessKind::Unnamed,
            None,
        )
    );
    assert_eq!(
        output
            .typed_ast
            .nodes()
            .iter()
            .filter(|(_, node)| node.kind.as_str() != "source.surface.unowned")
            .map(|(id, node)| (id.index(), node.kind.as_str()))
            .collect::<Vec<_>>(),
        [
            (16, "source.term.set.comprehension-generator"),
            (32, "source.term.variable-reference"),
            (34, "source.term.variable-reference"),
            (36, "source.formula.atomic.equality"),
            (38, "source.term.numeral"),
            (40, "source.term.set.target-type-head"),
            (41, "source.term.set.target-type"),
            (43, "source.term.set.comprehension"),
            (45, "source.statement-witness.item"),
            (46, "source.statement-witness.take"),
            (47, "source.term.variable-reference"),
            (49, "source.term.variable-reference"),
            (51, "source.formula.atomic.equality"),
            (54, "source.statement.conclusion"),
            (56, "source.statement.theorem"),
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
fn task258b3m2b2b3e_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols, baseline) = task258b3m2b2b3e_output(262_010);
    let (_, _, _, stale) = task258b3m2b2b3e_output(262_011);
    for mutation in 0..5 {
        let run = || {
            source_statement_b3m2b2b3e_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                |input: &mut SourceStatementB3M2B2B3ERouteInputs| match mutation {
                    0 => {
                        input.arena =
                            mizar_checker::typed_ast::TypedArena::try_new(None, Vec::new())
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
            .expect_err("lower mutation must fail")
        };
        let first = run();
        let repeated = run();
        assert_eq!(first, repeated, "coarse mutation {mutation} replay");
        assert!(
            first.to_ascii_lowercase().contains("dependency")
                || first.to_ascii_lowercase().contains("invalid"),
            "mutation {mutation}: {first}"
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
        let run = || {
            source_statement_b3m2b2b3e_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                |symbols| task258b2_mutate_resolver(symbols, mutation),
            )
            .expect("resolver selector")
            .expect_err("resolver mutation must fail")
        };
        let error = run();
        assert_eq!(error, run(), "{mutation:?} repeated failure");
        assert!(
            error.starts_with("Task258B3M2B2B3E resolver"),
            "{mutation:?}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3E_TEXT,
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
        let run = || {
            source_statement_b3m2b2b3e_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                |symbols| task258b3m2b2b3a_mutate_label_resolver(symbols, mutation),
            )
            .expect("label selector")
            .expect_err("label mutation must fail")
        };
        let error = run();
        assert_eq!(error, run(), "{mutation:?} repeated failure");
        assert!(
            error.to_ascii_lowercase().contains("resolver")
                || error.to_ascii_lowercase().contains("owner"),
            "{mutation:?}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3E_TEXT,
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
            TASK258B3M2B2B3E_TASK48_FIELD_COUNT,
            TASK258B3M2B2B3E_TASK252_FIELD_COUNT,
            TASK258B3M2B2B3E_TASK255_FIELD_COUNT,
            TASK258B3M2B2B3E_TASK256_FIELD_COUNT,
            TASK258B3M2B2B3E_TASK258_FIELD_COUNT,
            TASK258B3M2B2B3E_WITNESS_FIELD_COUNT,
        ),
        (32, 70, 53, 72, 62, 21)
    );
    for (stage, fields, error_prefix) in [
        (
            SourceStatementB3M2B2B3ELowerStage::Task48,
            TASK258B3M2B2B3E_TASK48_FIELD_COUNT,
            "Task48:",
        ),
        (
            SourceStatementB3M2B2B3ELowerStage::Task252,
            TASK258B3M2B2B3E_TASK252_FIELD_COUNT,
            "Task252:",
        ),
        (
            SourceStatementB3M2B2B3ELowerStage::Task255,
            TASK258B3M2B2B3E_TASK255_FIELD_COUNT,
            "Task255:",
        ),
    ] {
        for field in 0..fields {
            let run = || {
                source_statement_b3m2b2b3e_output_with_lower_stage_mutation(
                    &ast,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                    stage,
                    field,
                )
                .expect("lower-stage selector")
                .expect_err("lower-stage mutation must fail")
            };
            let error = run();
            assert_eq!(error, run(), "lower stage {stage:?} field {field} replay");
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
                SOURCE_STATEMENT_B3M2B2B3E_TEXT,
            )
            .expect("clean replay selector")
            .expect("clean replay");
            assert_eq!(
                replay.typed_ast.debug_text(),
                baseline.typed_ast.debug_text()
            );
            assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
        }
    }
    for (stage, fields, error_prefix) in [
        (
            SourceStatementB3M2B2B3EStage::Task256,
            TASK258B3M2B2B3E_TASK256_FIELD_COUNT,
            "Task256:",
        ),
        (
            SourceStatementB3M2B2B3EStage::Task258,
            TASK258B3M2B2B3E_TASK258_FIELD_COUNT,
            "Task258:",
        ),
        (
            SourceStatementB3M2B2B3EStage::Witness,
            TASK258B3M2B2B3E_WITNESS_FIELD_COUNT,
            "B3E:",
        ),
    ] {
        for field in 0..fields {
            let run = || {
                source_statement_b3m2b2b3e_output_with_stage_mutation(
                    &ast,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                    stage,
                    field,
                )
                .expect("stage selector")
                .expect_err("stage mutation must fail")
            };
            let error = run();
            assert_eq!(error, run(), "stage {stage:?} field {field} replay");
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
                SOURCE_STATEMENT_B3M2B2B3E_TEXT,
            )
            .expect("clean replay selector")
            .expect("clean replay");
            assert_eq!(
                replay.typed_ast.debug_text(),
                baseline.typed_ast.debug_text()
            );
            assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
        }
    }
}

#[test]
fn task258b3m2b2b3e_comprehension_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols, baseline) = task258b3m2b2b3e_output(262_020);
    assert!(
        extract_comprehension_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3E_TEXT)
            .is_some()
    );
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B3E_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B3E_TEXT.as_bytes().to_vec();
        bytes[byte_index] = b'?';
        let source = String::from_utf8(bytes).expect("ASCII source");
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "byte {byte_index}"
        );
    }
    for node in 0..60 {
        for mutation in [
            SourceStatementB3M2B2B3ESurfaceMutation::NodeKind(node),
            SourceStatementB3M2B2B3ESurfaceMutation::NodeRange(node),
            SourceStatementB3M2B2B3ESurfaceMutation::NodeRecovery(node),
            SourceStatementB3M2B2B3ESurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                extract_comprehension_witness_source_statement_with_surface_mutation(
                    &ast,
                    SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                    mutation,
                )
                .is_none(),
                "{mutation:?}"
            );
        }
    }
    assert!(
        extract_comprehension_witness_source_statement_with_surface_mutation(
            &ast,
            SOURCE_STATEMENT_B3M2B2B3E_TEXT,
            SourceStatementB3M2B2B3ESurfaceMutation::RootIdentity,
        )
        .is_none()
    );

    let mut near_misses = vec![SOURCE_STATEMENT_B3M2B2B3E_TEXT.replacen(
        "FormulaStatementComprehensionWitnessSmoke",
        "FormulaStatementComprehensionWitnessNearMiss",
        1,
    )];
    for replacement in [
        "{4 where candidate255 is set}",
        "{x where candidate255 is set}",
        "{3}",
        "{}",
        "{3 where candidate255 is set, second255 is set}",
        "{3 where candidate255 is object}",
        "{3 where candidate255 is non empty set}",
        "{3 where candidate255 is set attr}",
        "{3 where candidate255 is set : x = x}",
        "{{1} where candidate255 is set}",
        "{candidate255 where candidate255 is set}",
        "{3 where candidate255 being set}",
        "{3 where candidate255 is set qua set}",
        "{3 where candidate255 is set} qua set",
        "({3 where candidate255 is set})",
        "{3 where candidate255 is set, candidate255 is set}",
        "4 qua set",
        "the set",
        "{1, 2}",
        "TypeCaseStruct(x: 1, y: 2)",
    ] {
        near_misses.push(SOURCE_STATEMENT_B3M2B2B3E_TEXT.replacen(
            "{3 where candidate255 is set}",
            replacement,
            1,
        ));
    }
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3E_TEXT.replacen(
        "take {3 where candidate255 is set}",
        "take y = {3 where candidate255 is set}",
        1,
    ));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3E_TEXT.replacen(
        "take {3 where candidate255 is set}",
        "take {3 where candidate255 is set}, {3 where candidate255 is set}",
        1,
    ));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3E_TEXT.replace('\n', "\r\n"));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3E_TEXT.trim_end().to_owned());
    near_misses.push(format!("{SOURCE_STATEMENT_B3M2B2B3E_TEXT}\n"));
    for (index, source) in near_misses.into_iter().enumerate() {
        let (near_ast, near_module, _, near_symbols, _) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 262_100 + index);
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

    // These lower handoffs come from their own routes; the choice donor uses original-source
    // recollection and public native lower before entering the B3E post-auth boundary.
    let zero_generator = task258b3m2b2b3a_output(262_020)
        .3
        .typed_ast
        .source_set_term()
        .expect("zero-generator lower handoff")
        .clone();
    let wrong_generator_and_type = {
        use mizar_checker::source_set_term::{
            SourceSetRequestKind, SourceSetTermId, SourceSetTermKind, SourceSetTermRecovery,
            SourceSetTypeHead, SourceSetTypeOwner, SourceSetTypeRole, SourceSetTypeSiteId,
        };
        let (donor_ast, donor_module, donor_symbols) = task258b3m2b2b3c_fixture(262_020);
        let (source, native_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(
            &donor_ast, &donor_module,
        ).expect("choice donor recollection").expect("choice donor module");
        assert_eq!((source.source_id(), source.module()), (donor_ast.source_id, &donor_module));
        let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &native_symbols)
            .expect("choice donor occurrences");
        let before = (
            donor_ast.snapshot_text(), donor_module.clone(), donor_symbols.clone(),
            source.clone(), native_symbols.clone(), scope.clone(), labels.clone(), resolution.clone(),
        );
        let (_, _, lower) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(
            &source, &native_symbols, &scope,
        ).expect("choice donor native lower");
        assert_eq!((lower.source_id(), lower.module_id()), (source.source_id(), source.module()));
        let set = lower.source_set_term().expect("actual native choice donor");
        assert_eq!((set.source_id(), set.module_id()), (source.source_id(), source.module()));
        assert_eq!(set.primary_term_fingerprint(), lower.source_term().expect("native donor primary").debug_text());
        assert!(set.application_fingerprint().is_none() && set.structure_fingerprint().is_none());
        assert_eq!((set.terms().len(), set.type_sites().len(), set.requests().len()), (1, 1, 2));
        assert!(set.wrappers().is_empty() && set.generators().is_empty() && set.conditions().is_empty() && set.edges().is_empty());
        let term = set.terms().get(SourceSetTermId::new(0)).expect("native donor choice");
        assert_eq!((term.source_range(), term.source_ordinal(), term.context().index(), term.kind(), term.recovery(), term.spelling()),
            (range(donor_ast.source_id, 82, 89), 0, 1, SourceSetTermKind::Choice, SourceSetTermRecovery::Normal, "the set"));
        let target = set.type_sites().get(SourceSetTypeSiteId::new(0)).expect("native donor target");
        assert_eq!(target.owner(), SourceSetTypeOwner::Term {
            term: SourceSetTermId::new(0), role: SourceSetTypeRole::ChoiceTarget,
        });
        for (site, surface) in [(term.site(), 35), (target.site(), 34), (target.head_site(), 33)] {
            let actual = donor_ast.node_views().find(|view| view.id().index() == surface)
                .expect("actual donor source node");
            assert_eq!(lower.nodes().node(site.node()).expect("donor typed node").resolved_node,
                Some(source.resolved_node_for(actual.id()).expect("donor source projection")));
        }
        assert_eq!((target.source_range(), target.head_range(), target.context().index(), target.head(), target.recovery(), target.spelling(), target.head_spelling()),
            (range(donor_ast.source_id, 86, 89), range(donor_ast.source_id, 86, 89), 1, SourceSetTypeHead::BuiltinSet, SourceSetTermRecovery::Normal, "set", "set"));
        assert_eq!(set.requests().iter().map(|(_, request)| (
            request.term().index(), request.ordinal(), request.kind(), request.generator(), request.type_site(),
        )).collect::<Vec<_>>(), [
            (0, 0, SourceSetRequestKind::ChoiceNonempty, None, Some(SourceSetTypeSiteId::new(0))),
            (0, 1, SourceSetRequestKind::ResultType, None, None),
        ]);
        assert_eq!((
            donor_ast.snapshot_text(), donor_module, donor_symbols,
            source, native_symbols, scope, labels, resolution,
        ), before);
        set.clone()
    };

    let (
        condition_ast,
        condition_module,
        condition_shells,
        condition_symbols,
        condition_diagnostics,
    ) = task253_ast_from_source_text_with_diagnostic_count(TASK255C1_SET_SOURCE, 262_020);
    assert_eq!(condition_diagnostics, 0);
    let condition_symbols = augment_type_elaboration_import_summaries(
        &condition_ast,
        &condition_module,
        condition_symbols,
    );
    let conditioned_comprehension = source_set_term_output_with_source(
        &condition_ast,
        condition_module,
        &condition_shells,
        &condition_symbols,
        TASK255C1_SET_SOURCE,
    )
    .expect("conditioned same-provenance selector")
    .expect("conditioned same-provenance lower route")
    .typed_ast
    .source_set_term()
    .expect("conditioned same-provenance handoff")
    .clone();
    assert_eq!(conditioned_comprehension.conditions().len(), 1);

    let multiple_source = r#"
definition
  func Multi255B3E: multi255b3e() -> set
    equals {4 where alpha255 is set, beta255 is object};
end;
"#;
    let (multiple_ast, multiple_module, _) = task255_ast_from_source_text(multiple_source, 262_020);
    let multiple_roots = task255_outer_set_roots(&multiple_ast);
    assert_eq!(multiple_roots.len(), 1);
    let multiple_generators = synthetic_source_set_term_output(
        &multiple_ast,
        multiple_module.clone(),
        task254_module_binding_env(&multiple_ast, multiple_module),
        &multiple_roots,
        None::<SyntheticSourceSetTermDependencies>,
        &std::collections::BTreeSet::new(),
    )
    .expect("multiple-generator lower route")
    .typed_ast
    .source_set_term()
    .expect("multiple-generator lower handoff")
    .clone();
    assert_eq!(multiple_generators.generators().len(), 2);

    let nested_source = r#"
definition
  func Nested255B3E: nested255b3e() -> set
    equals {{5 where inner255b3e is set} where outer255b3e is object};
end;
"#;
    let (nested_ast, nested_module, _) = task255_ast_from_source_text(nested_source, 262_020);
    let nested_roots = task255_outer_set_roots(&nested_ast);
    assert_eq!(nested_roots.len(), 1);
    let nested_comprehension = synthetic_source_set_term_output(
        &nested_ast,
        nested_module.clone(),
        task254_module_binding_env(&nested_ast, nested_module),
        &nested_roots,
        None::<SyntheticSourceSetTermDependencies>,
        &std::collections::BTreeSet::new(),
    )
    .expect("nested-comprehension lower route")
    .typed_ast
    .source_set_term()
    .expect("nested-comprehension set")
    .clone();

    let generator_reference_source = r#"
definition
  func GeneratorReference255B3E: generator_reference255b3e() -> set
    equals {referenced_generator255b3e where referenced_generator255b3e is set};
end;
"#;
    let (generator_reference_ast, generator_reference_module, _) =
        task255_ast_from_source_text(generator_reference_source, 262_020);
    let generator_reference_roots = task255_outer_set_roots(&generator_reference_ast);
    assert_eq!(generator_reference_roots.len(), 1);
    let generator_reference_exclusion = synthetic_source_set_term_output(
        &generator_reference_ast,
        generator_reference_module.clone(),
        task254_module_binding_env(&generator_reference_ast, generator_reference_module),
        &generator_reference_roots,
        None::<SyntheticSourceSetTermDependencies>,
        &std::collections::BTreeSet::new(),
    )
    .expect("generator-reference Task255 exclusion route")
    .typed_ast
    .source_set_term()
    .expect("generator-reference exclusion handoff")
    .clone();

    assert_eq!(nested_comprehension.terms().len(), 2);
    assert_eq!(
        (
            generator_reference_exclusion.terms().len(),
            generator_reference_exclusion.generators().len(),
            generator_reference_exclusion.edges().len(),
        ),
        (0, 0, 0),
        "Task255 owns generator-reference exclusion before B3E"
    );
    let baseline_set = baseline
        .typed_ast
        .source_set_term()
        .expect("baseline comprehension handoff");
    for (label, replacement) in [
        ("zero generators", zero_generator),
        ("wrong generator/type-site rows", wrong_generator_and_type),
        ("present/nonzero condition", conditioned_comprehension),
        (
            "multiple/extra generator and type-site rows",
            multiple_generators,
        ),
        ("nested comprehension", nested_comprehension),
        (
            "generator-referencing mapper Task255 exclusion",
            generator_reference_exclusion,
        ),
    ] {
        assert_eq!(
            replacement.source_id(),
            baseline_set.source_id(),
            "{label} source provenance"
        );
        assert_eq!(
            replacement.module_id(),
            baseline_set.module_id(),
            "{label} module provenance"
        );
        let run = || {
            source_statement_b3m2b2b3e_output_with_post_auth_family_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                |input| input.set_term = Some(replacement.clone()),
            )
            .expect("post-auth selector")
            .expect_err("post-auth shape must fail")
        };
        let error = run();
        assert_eq!(error, run(), "{label} repeated failure");
        if label == "wrong generator/type-site rows" {
            assert_eq!(error, "source statement witness dependency mismatch", "{label}");
        } else {
            assert!(
                error.to_ascii_lowercase().contains("dependency")
                    || error.to_ascii_lowercase().contains("invalid"),
                "{label}: {error}"
            );
        }
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3E_TEXT,
        )
        .expect("post-auth clean replay selector")
        .expect("post-auth clean replay");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline.typed_ast.debug_text(),
            "{label} typed replay"
        );
        assert_eq!(
            replay.resolved.debug_text(),
            baseline.resolved.debug_text(),
            "{label} resolved replay"
        );
    }

    for node in 38..=46 {
        let run = || {
            source_statement_b3m2b2b3e_output_with_post_auth_family_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3E_TEXT,
                |input| {
                    let root = input.arena.root();
                    let mut rows = input
                        .arena
                        .iter()
                        .map(|(_, row)| row.clone())
                        .collect::<Vec<_>>();
                    rows[node].kind = if matches!(node, 38 | 40 | 41 | 43 | 45 | 46) {
                        "source.surface.unowned".into()
                    } else {
                        "source.statement-witness.item".into()
                    };
                    input.arena =
                        mizar_checker::typed_ast::TypedArena::try_new(root, rows).expect("arena");
                },
            )
            .expect("ownership selector")
            .expect_err("partial/extra subtree ownership must fail")
        };
        let error = run();
        assert_eq!(error, run(), "ownership node {node} repeated failure");
        assert!(
            error.to_ascii_lowercase().contains("dependency")
                || error.to_ascii_lowercase().contains("invalid"),
            "ownership node {node}: {error}"
        );
    }
}

#[test]
fn task258b3m2b2b3e_family_and_active_route_isolation_is_atomic() {
    let (ast, module, symbols, b3e) = task258b3m2b2b3e_output(262_300);
    let baselines = [
        task258b3m2b2b3_family_debug(0, 262_301),
        task258b3m2b2b3_family_debug(1, 262_302),
        task258b3m2b2b3_family_debug(2, 262_303),
        task258b3m2b2b3_family_debug(3, 262_304),
        task258b3m2b2b3_family_debug(4, 262_305),
    ];
    for left in 0..5 {
        for right in (left + 1)..5 {
            assert_ne!(baselines[left], baselines[right], "{left}/{right}");
        }
    }
    let set = b3e.typed_ast.source_set_term().expect("B3E set");
    assert_eq!(
        (
            set.generators().len(),
            set.conditions().len(),
            set.edges().len()
        ),
        (1, 0, 1)
    );
    let application = {
        let (case_ast, case_module, _, case_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(
                SOURCE_STATEMENT_B3M2B2B1A_TEXT,
                262_306,
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
                262_307,
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
        let result = source_statement_b3m2b2b3e_output_with_post_auth_family_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3E_TEXT,
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
    for a in 0..5 {
        for b in 0..5 {
            for c in 0..5 {
                for d in 0..5 {
                    for e in 0..5 {
                        let order = [a, b, c, d, e];
                        if order
                            .iter()
                            .copied()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                            != 5
                        {
                            continue;
                        }
                        for (position, family) in order.into_iter().enumerate() {
                            assert_eq!(
                                task258b3m2b2b3_family_debug(
                                    family,
                                    262_400 + order_index * 10 + position,
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
    }
    assert_eq!(order_index, 120);

    for (source, expects_application, expects_structure, expects_set) in [
        (SOURCE_STATEMENT_B3M2B2B1A_TEXT, true, false, false),
        (SOURCE_STATEMENT_B3M2B2B2A_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B2B_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B2C_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B3A_TEXT, false, false, true),
        (SOURCE_STATEMENT_B3M2B2B3C_TEXT, false, false, true),
        (SOURCE_STATEMENT_B3M2B2B3D_TEXT, false, false, true),
        (SOURCE_STATEMENT_B3M2B2B3E_TEXT, false, false, true),
    ] {
        let (case_ast, case_module, _, case_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(source, 263_800 + source.len());
        assert_eq!(diagnostics, 0);
        let case_symbols =
            augment_type_elaboration_import_summaries(&case_ast, &case_module, case_symbols);
        if source == SOURCE_STATEMENT_B3M2B2B3C_TEXT || source == SOURCE_STATEMENT_B3M2B2B3D_TEXT {
            let before = (case_ast.snapshot_text(), case_module.clone(), case_symbols.clone());
            assert!(source_statement_output_with_source(
                &case_ast, case_module.clone(), &case_symbols, source,
            ).is_none());
            assert!(source_statement_transport_detail_keys(
                &case_ast, case_module.clone(), &case_symbols, source,
            ).is_none());
            if source == SOURCE_STATEMENT_B3M2B2B3D_TEXT {
                assert!(extract_qua_witness_source_statement(&case_ast, source).is_some());
                assert_eq!(
                    source_statement_b3m2b2b3d_output_with_mutation(
                        &case_ast,
                        case_module.clone(),
                        &case_symbols,
                        source,
                        |_| {},
                    )
                    .expect("authentic retired Qua extraction")
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
            SOURCE_STATEMENT_B3M2B2B3E_TEXT,
        )
        .expect("exact private detail route")
        .is_empty()
    );
}

#[test]
fn task258b3m2b2b3e_typed_final_clone_debug_rollback_and_comprehension_semantics_are_stable() {
    use mizar_checker::source_set_term::{
        SourceSetEdgeRole, SourceSetGeneratorId, SourceSetRequestKind, SourceSetTarget,
    };

    let (ast, module, symbols, output) = task258b3m2b2b3e_output(264_000);
    let replay = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3E_TEXT,
    )
    .expect("selector")
    .expect("replay");
    assert_eq!(output.typed_ast.debug_text(), replay.typed_ast.debug_text());
    assert_eq!(output.resolved.debug_text(), replay.resolved.debug_text());
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
    assert!(debug.contains("spelling=\"{ 3 where candidate255 is set }\""));

    let set_term = output.typed_ast.source_set_term().expect("set term");
    assert!(set_term.wrappers().is_empty());
    assert!(set_term.conditions().is_empty());
    let generator = set_term
        .generators()
        .get(SourceSetGeneratorId::new(0))
        .expect("generator");
    assert_eq!(generator.spelling(), "candidate255");
    assert_eq!(
        output
            .typed_ast
            .source_statement()
            .expect("statement")
            .binding_env()
            .bindings()
            .len(),
        1,
        "the comprehension generator must not fabricate a BindingId"
    );
    assert!(
        output
            .typed_ast
            .source_term()
            .expect("primary")
            .references()
            .iter()
            .all(|(_, reference)| reference.term().index() != 2),
        "mapper numeral has no binding/reference capture"
    );
    let edge = set_term.edges().iter().next().expect("mapper edge").1;
    assert_eq!(edge.role(), SourceSetEdgeRole::ComprehensionMapper);
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
            SourceSetRequestKind::GeneratorSethood,
            SourceSetRequestKind::ResultType
        ]
    );

    let baseline = output.typed_ast.debug_text();
    let run_rollback = || {
        source_statement_b3m2b2b3e_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3E_TEXT,
            |input| input.witness.witnesses.clear(),
        )
        .expect("rollback selector")
        .expect_err("empty witness aggregate must roll back")
    };
    let error = run_rollback();
    assert_eq!(error, run_rollback());
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
