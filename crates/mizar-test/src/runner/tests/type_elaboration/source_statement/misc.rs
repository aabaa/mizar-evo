#[test]
fn task258b3m2b2b3b_real_frontend_freezes_empty_set_enumeration_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B2B3B_TEXT.len(), 118);
    assert!(SOURCE_STATEMENT_B3M2B2B3B_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B3B_TEXT),
        "4899f5660afab3b5caf7c51bf760643ed5c7bd80445d44de86b5cfe0abee9dcb"
    );
    let (ast, module, symbols) = task258b3m2b2b3b_fixture(259_500);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (50, Some(49))
    );
    let extracted: SourceStatementB3M2B2B3BExtraction =
        extract_empty_set_enumeration_witness_source_statement(
            &ast,
            SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        )
        .expect("exact empty-enumeration extraction");
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
            46,
            range(ast.source_id, 19, 117),
            range(ast.source_id, 27, 74),
            36,
            range(ast.source_id, 90, 98),
            35,
            range(ast.source_id, 95, 97),
            33,
            range(ast.source_id, 82, 116),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [27, 29, 37, 39]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [31, 41]
    );
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [46, 44]
    );
    assert_eq!(
        extracted.statement_ranges,
        [range(ast.source_id, 19, 117), range(ast.source_id, 101, 112)]
    );
    assert_eq!(
        extracted.formula_ranges,
        [range(ast.source_id, 76, 81), range(ast.source_id, 106, 111)]
    );
    assert_eq!(
        extracted.term_ranges,
        [
            range(ast.source_id, 76, 77),
            range(ast.source_id, 80, 81),
            range(ast.source_id, 106, 107),
            range(ast.source_id, 110, 111),
        ]
    );

    let resolver =
        source_statement_b3m2b2b3b_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("resolver");
    assert_eq!(resolver.module_id(), &module);
    assert!(resolver.imports().is_empty());
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver.symbols().visible_candidates(
        &namespace,
        "FormulaStatementEmptySetEnumerationWitnessSmoke",
    );
    let [owner] = owners.as_slice() else {
        panic!("one exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 117))
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
    let labels = resolver.labels().visible_candidates(
        &namespace,
        "FormulaStatementEmptySetEnumerationWitnessSmoke",
    );
    let [label] = labels.as_slice() else {
        panic!("one exact theorem label");
    };
    assert_eq!(label.origin(), owner.origin());
    assert_eq!(label.contribution(), owner.contribution());
    assert_eq!(label.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        label.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
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

    assert!(
        source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        )
        .is_none()
    );
    assert!(
        source_statement_transport_detail_keys(
            &ast,
            module,
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        )
        .is_none()
    );
}

#[test]
fn task258b3m2b2b3b_empty_enumeration_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols) = task258b3m2b2b3b_fixture(259_520);
    assert!(
        extract_empty_set_enumeration_witness_source_statement(
            &ast,
            SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        )
        .is_some()
    );

    let before = (ast.snapshot_text(), module.clone(), symbols.clone());

    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B3B_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B3B_TEXT.as_bytes().to_vec();
        bytes[byte_index] = b'?';
        let source = String::from_utf8(bytes).expect("ASCII source");
        let source_before = source.clone();
        assert!(
            extract_empty_set_enumeration_witness_source_statement(&ast, &source).is_none(),
            "byte {byte_index}"
        );
        assert_eq!(source, source_before);
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
        assert!(extract_empty_set_enumeration_witness_source_statement(
            &ast, SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        ).is_some());
    }

    for node in 0..50 {
        for mutation in [
            SourceStatementB3M2B2B3BSurfaceMutation::NodeKind(node),
            SourceStatementB3M2B2B3BSurfaceMutation::NodeRange(node),
            SourceStatementB3M2B2B3BSurfaceMutation::NodeRecovery(node),
            SourceStatementB3M2B2B3BSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                extract_empty_set_enumeration_witness_source_statement_with_surface_mutation(
                    &ast,
                    SOURCE_STATEMENT_B3M2B2B3B_TEXT,
                    mutation,
                )
                .is_none(),
                "{mutation:?}"
            );
            assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
            assert!(extract_empty_set_enumeration_witness_source_statement(
                &ast, SOURCE_STATEMENT_B3M2B2B3B_TEXT,
            ).is_some());
        }
    }
    assert!(
        extract_empty_set_enumeration_witness_source_statement_with_surface_mutation(
            &ast,
            SOURCE_STATEMENT_B3M2B2B3B_TEXT,
            SourceStatementB3M2B2B3BSurfaceMutation::RootIdentity,
        )
        .is_none()
    );
    assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
    assert!(extract_empty_set_enumeration_witness_source_statement(
        &ast, SOURCE_STATEMENT_B3M2B2B3B_TEXT,
    ).is_some());
    let extra_newline = format!("{SOURCE_STATEMENT_B3M2B2B3B_TEXT}\n");
    let extra_newline_before = extra_newline.clone();
    assert!(
        extract_empty_set_enumeration_witness_source_statement(&ast, &extra_newline).is_none()
    );
    assert_eq!(extra_newline, extra_newline_before);
    assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
    assert!(extract_empty_set_enumeration_witness_source_statement(
        &ast, SOURCE_STATEMENT_B3M2B2B3B_TEXT,
    ).is_some());

    let changed_label = SOURCE_STATEMENT_B3M2B2B3B_TEXT.replacen(
        "FormulaStatementEmptySetEnumerationWitnessSmoke",
        "FormulaStatementEmptySetEnumerationWitnessNearMiss",
        1,
    );
    let mut near_misses = vec![changed_label];
    for replacement in [
        "{1}",
        "{1, 2}",
        "{1, 2, 3}",
        "({})",
        "{{}}",
        "{x where x is set : x = x}",
        "the set",
        "{} qua set",
        "choose set",
    ] {
        near_misses.push(SOURCE_STATEMENT_B3M2B2B3B_TEXT.replacen("{}", replacement, 1));
    }
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3B_TEXT.replacen("take {}", "take y = {}", 1));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3B_TEXT.replacen("take {}", "take {}, {}", 1));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3B_TEXT.replace("\n", "\r\n"));
    near_misses.push(SOURCE_STATEMENT_B3M2B2B3B_TEXT.trim_end().to_owned());

    for (index, source) in near_misses.into_iter().enumerate() {
        let (near_ast, near_module, _, near_symbols, _) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 260_200 + index);
        let near_before = (
            near_ast.snapshot_text(), near_module.clone(), near_symbols.clone(), source.clone(),
        );
        assert!(
            extract_empty_set_enumeration_witness_source_statement(&near_ast, &source).is_none(),
            "near miss {index}: {source:?}"
        );
        assert_eq!(
            (near_ast.snapshot_text(), near_module, near_symbols, source),
            near_before,
        );
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
        assert!(extract_empty_set_enumeration_witness_source_statement(
            &ast, SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        ).is_some());
    }
}
