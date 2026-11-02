#[test]
fn task258b3m2b2b3c_real_frontend_freezes_choice_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B2B3C_TEXT.len(), 110);
    assert!(SOURCE_STATEMENT_B3M2B2B3C_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B3C_TEXT),
        "aad2fcc20f6e9da957a71a7a239164f7d405a8be25a43bbe64ae1fcf4e027370"
    );
    let (ast, module, symbols) = task258b3m2b2b3c_fixture(260_300);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (52, Some(51))
    );
    let extracted: SourceStatementB3M2B2B3CExtraction =
        extract_choice_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT)
            .expect("exact choice-witness extraction");
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
            48,
            range(ast.source_id, 19, 109),
            range(ast.source_id, 27, 61),
            38,
            range(ast.source_id, 77, 90),
            37,
            range(ast.source_id, 82, 89),
            35,
            range(ast.source_id, 69, 108),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [27, 29, 39, 41]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [31, 43]
    );

    assert_eq!(extracted.statement_sites.map(|site| site.node().index()), [48, 46]);
    assert_eq!(extracted.statement_ranges, [range(ast.source_id, 19, 109), range(ast.source_id, 93, 104)]);
    assert_eq!(extracted.formula_ranges, [range(ast.source_id, 63, 68), range(ast.source_id, 98, 103)]);
    assert_eq!(extracted.term_ranges, [
        range(ast.source_id, 63, 64), range(ast.source_id, 67, 68),
        range(ast.source_id, 98, 99), range(ast.source_id, 102, 103),
    ]);
    for (parent, child) in [(48, 47), (47, 38), (38, 37), (37, 36), (36, 35), (35, 34), (34, 33)] {
        assert!(ast.nodes()[parent].children.iter().any(|id| id.index() == child));
    }

    let resolver =
        source_statement_b3m2b2b3c_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("resolver");
    assert_eq!(resolver.module_id(), &module);
    assert!(resolver.imports().is_empty());
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver
        .symbols()
        .visible_candidates(&namespace, "FormulaStatementChoiceWitnessSmoke");
    let [owner] = owners.as_slice() else {
        panic!("one exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 109))
    );
    assert_eq!(owner.origin().structural_path(), [2, 1]);
    assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(owner.origin().source_id(), ast.source_id);
    assert_eq!(owner.origin().module_id(), &module);
    assert!(owner.origin().import_edge().is_none());
    assert!(!owner.origin().is_recovered());
    let labels = resolver
        .labels()
        .visible_candidates(&namespace, "FormulaStatementChoiceWitnessSmoke");
    let [label] = labels.as_slice() else {
        panic!("one exact theorem label");
    };
    assert_eq!(label.origin(), owner.origin());
    assert_eq!(label.contribution(), owner.contribution());
    assert_eq!(
        label.recovery(),
        mizar_resolve::resolved_ast::RecoveryState::Normal
    );
    let contribution = resolver
        .contributions()
        .get(owner.contribution())
        .expect("contribution");
    assert_eq!(
        contribution.anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 18))
    );
    assert_eq!(
        contribution.effects().labels(),
        [label.origin_path().clone()]
    );

    let before = (ast.snapshot_text(), module.clone(), symbols.clone(), resolver.clone());
    assert!(source_statement_output_with_source(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
    ).is_none());
    assert!(source_statement_transport_detail_keys(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
    ).is_none());
    assert_eq!(
        source_statement_b3m2b2b3c_output_with_resolver_mutation(
            &ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
            |symbols| symbols,
        ).expect("authentic resolver extraction").expect_err("retired statement profile"),
        "source statement dependency mismatch",
    );
    assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone(), resolver.clone()), before);
    assert!(extract_choice_witness_source_statement(
        &ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
    ).is_some());

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
        let error = source_statement_b3m2b2b3c_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3C_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("resolver extraction")
        .expect_err("resolver label mutation must fail before retirement");
        let expected = if matches!(mutation,
            Task258B2ResolverMutation::Missing | Task258B2ResolverMutation::Duplicate,
        ) {
            "Task258B3M2B2B3C resolver theorem label mismatch"
        } else {
            "Task258B3M2B2B3C resolver theorem provenance mismatch"
        };
        assert_eq!(error, expected, "{mutation:?}");
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone(), resolver.clone()), before);
        assert_eq!(
            source_statement_b3m2b2b3c_output_with_resolver_mutation(
                &ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
                |symbols| symbols,
            ).expect("resolver replay extraction").expect_err("retired statement profile"),
            "source statement dependency mismatch",
            "{mutation:?} authentic replay",
        );
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone(), resolver.clone()), before);
        assert!(extract_choice_witness_source_statement(
            &ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
        ).is_some());
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
        let error = source_statement_b3m2b2b3c_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3C_TEXT,
            |symbols| task258b3m2b2b3a_mutate_label_resolver(symbols, mutation),
        )
        .expect("label extraction")
        .expect_err("label mutation must fail before retirement");
        let expected = if matches!(mutation,
            Task258B3M2B2B3ALabelMutation::Namespace | Task258B3M2B2B3ALabelMutation::PrimarySpelling,
        ) {
            "Task258B3M2B2B3C resolver theorem label mismatch"
        } else {
            "Task258B3M2B2B3C resolver theorem provenance mismatch"
        };
        assert_eq!(error, expected, "{mutation:?}");
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone(), resolver.clone()), before);
        assert_eq!(
            source_statement_b3m2b2b3c_output_with_resolver_mutation(
                &ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
                |symbols| symbols,
            ).expect("label replay extraction").expect_err("retired statement profile"),
            "source statement dependency mismatch",
            "{mutation:?} authentic replay",
        );
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone(), resolver.clone()), before);
        assert!(extract_choice_witness_source_statement(
            &ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
        ).is_some());
    }
}

#[test]
fn task258b3m2b2b3c_choice_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols) = task258b3m2b2b3c_fixture(260_320);
    assert!(
        extract_choice_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT).is_some()
    );

    let before = (ast.snapshot_text(), module.clone(), symbols.clone());

    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B3C_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B3C_TEXT.as_bytes().to_vec();
        bytes[byte_index] = b'?';
        let source = String::from_utf8(bytes).expect("ASCII source");
        let source_before = source.clone();
        assert!(
            extract_choice_witness_source_statement(&ast, &source).is_none(),
            "byte {byte_index}"
        );
        assert_eq!(source, source_before);
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
        assert!(extract_choice_witness_source_statement(
            &ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
        ).is_some());
    }
    for node in 0..52 {
        for mutation in [
            SourceStatementB3M2B2B3CSurfaceMutation::NodeKind(node),
            SourceStatementB3M2B2B3CSurfaceMutation::NodeRange(node),
            SourceStatementB3M2B2B3CSurfaceMutation::NodeRecovery(node),
            SourceStatementB3M2B2B3CSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                extract_choice_witness_source_statement_with_surface_mutation(
                    &ast,
                    SOURCE_STATEMENT_B3M2B2B3C_TEXT,
                    mutation,
                )
                .is_none(),
                "{mutation:?}"
            );
            assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
            assert!(extract_choice_witness_source_statement(
                &ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
            ).is_some());
        }
    }
    assert!(
        extract_choice_witness_source_statement_with_surface_mutation(
            &ast,
            SOURCE_STATEMENT_B3M2B2B3C_TEXT,
            SourceStatementB3M2B2B3CSurfaceMutation::RootIdentity,
        )
        .is_none()
    );
    assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
    assert!(extract_choice_witness_source_statement(
        &ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
    ).is_some());

    let changed_label = SOURCE_STATEMENT_B3M2B2B3C_TEXT.replacen(
        "FormulaStatementChoiceWitnessSmoke",
        "FormulaStatementChoiceWitnessNearMiss",
        1,
    );
    let mut near_misses = vec![changed_label];
    for replacement in [
        "set",
        "the object",
        "the non empty set",
        "the set attr",
        "the set(x)",
        "(the set)",
        "{}",
        "{1, 2}",
        "{x where x is set : x = x}",
        "the set qua set",
        "x",
        "1 ++ 2",
        "TypeCaseStruct(x: 1, y: 2)",
    ] {
        near_misses.push(SOURCE_STATEMENT_B3M2B2B3C_TEXT.replacen("the set", replacement, 1));
    }
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3C_TEXT.replacen(
        "take the set",
        "take y = the set",
        1,
    ));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3C_TEXT.replacen(
        "take the set",
        "take the set, the set",
        1,
    ));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3C_TEXT.replace('\n', "\r\n"));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3C_TEXT.trim_end().to_owned());
    near_misses.push(format!("{SOURCE_STATEMENT_B3M2B2B3C_TEXT}\n"));

    assert_eq!(near_misses.len(), 19);
    for (index, source) in near_misses.into_iter().enumerate() {
        let (near_ast, near_module, _, near_symbols, _) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 260_400 + index);
        let near_before = (
            near_ast.snapshot_text(), near_module.clone(), near_symbols.clone(), source.clone(),
        );
        assert!(
            extract_choice_witness_source_statement(&near_ast, &source).is_none(),
            "near miss {index}: {source:?}"
        );
        assert_eq!(
            (near_ast.snapshot_text(), near_module, near_symbols, source),
            near_before,
        );
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
        assert!(extract_choice_witness_source_statement(
            &ast, SOURCE_STATEMENT_B3M2B2B3C_TEXT,
        ).is_some());
    }
}
