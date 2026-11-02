#[test]
fn task258b5b_opt_in_imported_public_theorem_label_is_exact() {
    assert_eq!(SOURCE_STATEMENT_B5B_LOWER_TEXT.len(), 146);
    assert!(SOURCE_STATEMENT_B5B_LOWER_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B5B_LOWER_TEXT),
        "671e940c9dc749757dc8fddcc30a1a230aecb650058e64d6f1e73c1c66e93e9e"
    );
    let (ast, module, _, raw) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5B_LOWER_TEXT, 258_550);
    assert_eq!(task258b5b_lower_resolver_profile(&raw), [1, 0, 1, 1, 0]);
    let normal = augment_type_elaboration_import_summaries(&ast, &module, raw.clone());
    assert_eq!(task258b5b_lower_resolver_profile(&normal), [8, 0, 1, 3, 1]);
    let opt_in = super::import_fixtures::
        augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
            &ast, &module, raw,
        );
    assert!(task258b5b_imported_label_contract_is_exact(
        &ast, &module, &opt_in
    ));
    assert_eq!(normal.imports(), opt_in.imports());
    assert_eq!(normal.exports(), opt_in.exports());
    assert_eq!(normal.symbols(), opt_in.symbols());
    assert_eq!(normal.definitions(), opt_in.definitions());
    assert_eq!(normal.overloads(), opt_in.overloads());
    assert_eq!(normal.registrations(), opt_in.registrations());
    assert_eq!(normal.lexical_summaries(), opt_in.lexical_summaries());
    assert_eq!(normal.namespace_graph(), opt_in.namespace_graph());
    assert_eq!(
        normal.declaration_dependencies(),
        opt_in.declaration_dependencies()
    );
    assert_eq!(normal.module_summaries(), opt_in.module_summaries());
    for (normal_record, opt_in_record) in normal
        .contributions()
        .iter()
        .zip(opt_in.contributions().iter())
    {
        assert_eq!(normal_record.id(), opt_in_record.id());
        assert_eq!(normal_record.module(), opt_in_record.module());
        assert_eq!(normal_record.kind(), opt_in_record.kind());
        assert_eq!(normal_record.anchor(), opt_in_record.anchor());
        assert_eq!(
            normal_record.effects().symbols(),
            opt_in_record.effects().symbols()
        );
        assert_eq!(
            normal_record.effects().definitions(),
            opt_in_record.effects().definitions()
        );
        assert_eq!(
            normal_record.effects().overload_groups(),
            opt_in_record.effects().overload_groups()
        );
        assert_eq!(
            normal_record.effects().registrations(),
            opt_in_record.effects().registrations()
        );
        assert_eq!(
            normal_record.effects().lexical_summaries(),
            opt_in_record.effects().lexical_summaries()
        );
        assert_eq!(
            normal_record.effects().namespace_edges(),
            opt_in_record.effects().namespace_edges()
        );
        assert_eq!(
            normal_record.effects().declaration_dependencies(),
            opt_in_record.effects().declaration_dependencies()
        );
        assert_eq!(
            normal_record.effects().imports(),
            opt_in_record.effects().imports()
        );
        assert_eq!(
            normal_record.effects().exports(),
            opt_in_record.effects().exports()
        );
        assert_eq!(
            normal_record.effects().diagnostics(),
            opt_in_record.effects().diagnostics()
        );
        if opt_in_record.id().index() != 2 {
            assert_eq!(
                normal_record.effects().labels(),
                opt_in_record.effects().labels()
            );
        } else {
            assert!(normal_record.effects().labels().is_empty());
            assert_eq!(
                opt_in_record.effects().labels()[0].as_str(),
                "summary:parser.type_fixtures::Ref:label:Ref"
            );
        }
    }
}

#[test]
fn task258b5b_opt_in_imported_label_corruption_and_default_routes_stay_isolated() {
    let (ast, module, _, raw) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5B_LOWER_TEXT, 258_551);
    let normal = augment_type_elaboration_import_summaries(&ast, &module, raw.clone());
    assert!(normal.labels().is_empty());
    assert_eq!(
        normal,
        augment_type_elaboration_import_summaries(&ast, &module, raw.clone())
    );
    let opt_in = super::import_fixtures::
        augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
            &ast, &module, raw,
        );
    assert!(task258b5b_imported_label_contract_is_exact(
        &ast, &module, &opt_in
    ));
    let foreign_source = task258b3m2b2b3a_foreign_source_id(ast.source_id);
    for mutation in [
        Task258B5BImportedLabelMutation::Missing,
        Task258B5BImportedLabelMutation::Duplicate,
        Task258B5BImportedLabelMutation::Private,
        Task258B5BImportedLabelMutation::LocalOnly,
        Task258B5BImportedLabelMutation::ReExported,
        Task258B5BImportedLabelMutation::WrongKind,
        Task258B5BImportedLabelMutation::WrongSpelling,
        Task258B5BImportedLabelMutation::WrongModule,
        Task258B5BImportedLabelMutation::WrongNamespace,
        Task258B5BImportedLabelMutation::WrongContribution,
        Task258B5BImportedLabelMutation::Recovered,
        Task258B5BImportedLabelMutation::WrongOriginPath,
        Task258B5BImportedLabelMutation::WrongSource,
        Task258B5BImportedLabelMutation::WrongRange,
        Task258B5BImportedLabelMutation::WrongStructuralPath,
        Task258B5BImportedLabelMutation::MissingEffect,
        Task258B5BImportedLabelMutation::ExtraEffect,
        Task258B5BImportedLabelMutation::WrongEffect,
    ] {
        let corrupted =
            task258b5b_mutate_imported_label(&opt_in, &module, foreign_source, mutation);
        assert!(
            !task258b5b_imported_label_contract_is_exact(&ast, &module, &corrupted),
            "corruption remained exact: {mutation:?}"
        );
    }
    let (b5a_ast, b5a_module, _, b5a_raw) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_553);
    let b5a_default =
        augment_type_elaboration_import_summaries(&b5a_ast, &b5a_module, b5a_raw.clone());
    assert_eq!(
        super::import_fixtures::
            augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
                &b5a_ast,
                &b5a_module,
                b5a_raw,
            ),
        b5a_default
    );
}

#[test]
fn task258b5b_real_route_publishes_exact_imported_public_theorem_citation() {
    assert_eq!(SOURCE_STATEMENT_B5B_TEXT, SOURCE_STATEMENT_B5B_LOWER_TEXT);
    assert_eq!(SOURCE_STATEMENT_B5B_TEXT.len(), 146);
    assert!(SOURCE_STATEMENT_B5B_TEXT.ends_with('\n'));
    assert!(!SOURCE_STATEMENT_B5B_TEXT.ends_with("\n\n"));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B5B_TEXT),
        "671e940c9dc749757dc8fddcc30a1a230aecb650058e64d6f1e73c1c66e93e9e"
    );
    let (ast, module, raw, enriched) = task258b5b_exact_env(258_560);
    assert!(task258b5b_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B5B_TEXT,
        SourceStatementB5BSurfaceMutation::None,
    ));
    for index in 0..57 {
        for mutation in [
            SourceStatementB5BSurfaceMutation::NodeKind(index),
            SourceStatementB5BSurfaceMutation::NodeRange(index),
            SourceStatementB5BSurfaceMutation::NodeRecovery(index),
            SourceStatementB5BSurfaceMutation::NodeChildren(index),
        ] {
            assert!(
                !task258b5b_surface_profile_with_mutation_for_test(
                    &ast,
                    SOURCE_STATEMENT_B5B_TEXT,
                    mutation,
                ),
                "{mutation:?}"
            );
        }
    }
    assert!(!task258b5b_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B5B_TEXT,
        SourceStatementB5BSurfaceMutation::RootIdentity,
    ));
    let extracted: SourceStatementB5BExtraction =
        extract_imported_public_theorem_citation_source_statement(&ast, SOURCE_STATEMENT_B5B_TEXT)
            .expect("exact B5B surface");
    assert_eq!(
        (ast.nodes().len(), ast.root().expect("root").index()),
        (57, 56)
    );
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (53, 48, 145, 56, 106, 114, 144)
    );
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [53, 51]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .map(|source_range| (source_range.start, source_range.end)),
        [(48, 145), (122, 140)]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [40, 46]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [35, 37, 41, 43]
    );
    assert_eq!(task258b5b_lower_resolver_profile(&raw), [1, 0, 1, 1, 0]);
    let normal = augment_type_elaboration_import_summaries(&ast, &module, raw.clone());
    assert_eq!(task258b5b_lower_resolver_profile(&normal), [8, 0, 1, 3, 1]);
    assert!(normal.module_summaries().iter().next().is_none());
    assert!(task258b5b_imported_label_contract_is_exact(
        &ast, &module, &enriched
    ));
    assert_eq!(
        source_statement_b5b_resolver_env_for_test(&ast, &module, &raw, SOURCE_STATEMENT_B5B_TEXT,)
            .expect("B5B lower-stage opt-in"),
        enriched
    );
    assert_eq!(
        source_statement_b5b_output_with_raw_resolver_mutation(
            &ast,
            module.clone(),
            &raw,
            SOURCE_STATEMENT_B5B_TEXT,
            |symbols| symbols,
        )
        .expect("raw B5B selector")
        .expect("raw B5B route")
        .reference_use_ordinals,
        [1; 4]
    );
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &enriched,
        SOURCE_STATEMENT_B5B_TEXT,
    )
    .expect("B5B selector")
    .unwrap_or_else(|error| panic!("B5B route failed: {error}"));
    assert_eq!(
        (
            output.left_lookup_ordinal,
            output.right_lookup_ordinal,
            output.reference_use_ordinals.as_slice(),
        ),
        (1, 1, &[1, 1, 1, 1][..])
    );
    let statement = output.typed_ast.source_statement().expect("B5B statement");
    let references = output
        .typed_ast
        .source_statement_references()
        .expect("B5B references");
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
            output
                .typed_ast
                .source_term()
                .expect("Task252")
                .terms()
                .len(),
            output
                .typed_ast
                .source_term()
                .expect("Task252")
                .references()
                .len(),
            output
                .typed_ast
                .source_term()
                .expect("Task252")
                .numeric_type_requests()
                .len(),
        ),
        (2, 1, 0, 4, 4, 0)
    );
    let module_context = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("module context");
    let proof_context = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        (
            &module_context.owner,
            module_context.parent,
            module_context.layer,
            module_context.lexical_scope.as_ref(),
            module_context.bindings.as_slice(),
            module_context.visible_bindings.as_slice(),
            module_context.recovery,
        ),
        (
            &mizar_checker::binding_env::BindingContextOwner::Module,
            None,
            mizar_checker::binding_env::BindingContextLayer::Module,
            None,
            &[mizar_checker::binding_env::BindingId::new(0)][..],
            &[mizar_checker::binding_env::BindingId::new(0)][..],
            mizar_checker::binding_env::BindingContextRecovery::Normal,
        )
    );
    assert!(matches!(
        proof_context.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
            if (source_range.start, source_range.end) == (114, 144)
    ));
    assert_eq!(
        (
            proof_context.parent.map(|id| id.index()),
            proof_context.layer,
            proof_context
                .lexical_scope
                .as_ref()
                .map(|scope| scope.path()),
            proof_context.bindings.as_slice(),
            proof_context.visible_bindings.as_slice(),
            proof_context.recovery,
        ),
        (
            Some(0),
            mizar_checker::binding_env::BindingContextLayer::Proof,
            Some(&[0][..]),
            &[][..],
            &[mizar_checker::binding_env::BindingId::new(0)][..],
            mizar_checker::binding_env::BindingContextRecovery::Normal,
        )
    );
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            references.labels().len(),
            references.citations().len(),
        ),
        (1, 2, 2, 2, 2, 0, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("B5B owner");
    assert_eq!(
        (
            owner.site().node().index(),
            owner.source_range().start,
            owner.source_range().end,
            owner.spelling(),
            owner.contribution().index(),
            owner.role(),
            owner.status(),
            owner.recovery(),
            statement.checked_owner().origin().structural_path(),
        ),
        (
            53,
            48,
            145,
            "FormulaStatementImportedPublicTheoremCitationSmoke",
            0,
            mizar_checker::source_statement::SourceTheoremRole::Theorem,
            mizar_checker::source_statement::SourceTheoremStatus::Unmodified,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
            &[2, 1][..],
        )
    );
    let primary = output.typed_ast.source_term().expect("Task252");
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.context().index(),
                row.source_ordinal(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 35, 108, 109, 0, 0, "x"),
            (1, 37, 112, 113, 0, 1, "x"),
            (2, 41, 127, 128, 1, 2, "x"),
            (3, 43, 131, 132, 1, 3, "x"),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.term().index(),
                row.binding().index(),
                row.lexical_scope().map(|scope| scope.path().to_vec()),
                row.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, None, 1),
            (1, 1, 0, None, 1),
            (2, 2, 0, Some(vec![0]), 1),
            (3, 3, 0, Some(vec![0]), 1),
        ]
    );
    let atomic = output.typed_ast.source_atomic_formula().expect("Task256");
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
        (2, 0, 0, 0, 0, 0, 0, 4, 4)
    );
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.context().index(),
                row.source_ordinal(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 40, 108, 113, 0, 0, "x = x"),
            (1, 46, 127, 132, 1, 1, "x = x"),
        ]
    );
    for index in 0..4 {
        let edge = atomic
            .edges()
            .get(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index))
            .expect("edge");
        let request = atomic
            .requests()
            .get(mizar_checker::source_atomic_formula::SourceAtomicRequestId::new(index))
            .expect("request");
        assert_eq!(
            (edge.formula().index(), edge.ordinal()),
            (index / 2, index % 2)
        );
        assert_eq!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(index),
            )
        );
        assert_eq!(
            (request.formula().index(), request.ordinal(), request.edge()),
            (
                index / 2,
                index % 2,
                Some(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index)),
            )
        );
    }
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.context().index(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                53,
                48,
                145,
                0,
                "theorem FormulaStatementImportedPublicTheoremCitationSmoke : x = x proof thus x = x by Ref ; end ;",
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
            ),
            (
                1,
                1,
                51,
                122,
                140,
                1,
                "thus x = x by Ref ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
            ),
        ]
    );
    for index in 0..2 {
        let row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(
                index,
            ))
            .expect("statement row");
        let context = statement
            .contexts()
            .get(mizar_checker::source_statement::SourceStatementContextId::new(index))
            .expect("context row");
        let input = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input row");
        let candidate = statement
            .candidate_facts()
            .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(index))
            .expect("candidate row");
        assert_eq!(row.formula(), candidate.formula());
        assert_eq!(
            (context.statement().index(), input.statement().index()),
            (index, index)
        );
        assert_eq!(
            (input.context().index(), candidate.context().index()),
            (index, index)
        );
        assert_eq!((input.ordinal(), candidate.ordinal()), (0, 0));
        assert_eq!(
            input.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
            [2 * index, 2 * index + 1]
        );
    }
    let owned = output
        .typed_ast
        .nodes()
        .iter()
        .filter(|(_, row)| row.kind.as_str() != "source.surface.unowned")
        .map(|(id, _)| id.index())
        .collect::<Vec<_>>();
    assert_eq!(owned, [35, 37, 40, 41, 43, 46, 51, 53]);
    assert_eq!(
        output
            .typed_ast
            .nodes()
            .iter()
            .filter(|(_, row)| row.kind.as_str() == "source.surface.unowned")
            .count(),
        49
    );
    let citation = references
        .citations()
        .get(mizar_checker::source_statement::SourceStatementCitationId::new(0))
        .expect("imported citation");
    assert_eq!(
        (
            citation.statement().index(),
            citation.context().index(),
            citation.target(),
            citation.label_ref().index(),
            citation.proof_scope().path(),
            citation.source_range().start,
            citation.source_range().end,
            citation.ordinal(),
            citation.kind(),
            citation.recovery(),
        ),
        (
            1,
            1,
            mizar_checker::source_statement::SourceStatementCitationTarget::Imported,
            0,
            &[0][..],
            136,
            139,
            0,
            mizar_checker::source_statement::SourceStatementCitationKind::SimpleImported,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    let projection = references.label_projection();
    assert!(matches!(
        projection.source(),
        mizar_resolve::labels::LabelProjectionSource::Imported
    ));
    assert_eq!(
        (
            projection.origin_path().as_str(),
            projection.module().path().as_str(),
            projection.namespace().as_str(),
            projection.primary_spelling(),
            projection.kind(),
            projection.visibility(),
            projection.export_status(),
            projection.declaration_range().start,
            projection.declaration_range().end,
            projection.contribution().index(),
        ),
        (
            "summary:parser.type_fixtures::Ref:label:Ref",
            "parser.type_fixtures",
            module.path().as_str(),
            "Ref",
            mizar_resolve::resolved_ast::LabelKind::Theorem,
            mizar_resolve::env::Visibility::Public,
            mizar_resolve::env::ExportStatus::Exported,
            7,
            27,
            2,
        )
    );
    assert_eq!(
        (
            projection.origin().source_id(),
            projection.origin().module_id().path().as_str(),
            projection.origin().structural_path(),
            projection.origin().import_edge(),
            projection.origin().is_recovered(),
        ),
        (
            ast.source_id,
            "parser.type_fixtures",
            &[1, 0][..],
            None,
            false,
        )
    );
    let reference = references.reference_candidate();
    assert_eq!(
        (
            reference.site().node().index(),
            reference.site().range().start,
            reference.site().range().end,
            reference.site().spelling(),
            reference.ordinal(),
            reference.expectation(),
            reference.origin().source_id(),
            reference.origin().module_id(),
            reference.origin().structural_path(),
            reference.origin().import_edge(),
            reference.origin().is_recovered(),
        ),
        (
            48,
            136,
            139,
            "Ref",
            1,
            mizar_resolve::resolved_ast::LabelExpectation::ProofOrTheorem,
            ast.source_id,
            &module,
            &[48][..],
            None,
            false,
        )
    );
    assert!(matches!(
        reference.scope(),
        mizar_resolve::labels::LabelReferenceScope::Unqualified {
            proof_scope: Some(scope)
        } if scope.path() == [0]
    ));
    let resolver_ast = references.resolver_ast();
    assert_eq!(
        (
            resolver_ast.nodes().root().index(),
            resolver_ast.nodes().len(),
            resolver_ast.name_refs().len(),
            resolver_ast.label_refs().len(),
            resolver_ast.imports().imports().count(),
            resolver_ast.imports().exports().count(),
        ),
        (56, 57, 0, 1, 1, 0)
    );
    assert_eq!(
        (
            references.label_resolution().index().len(),
            references.label_resolution().table().len(),
            references
                .label_resolution()
                .ids()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            references.label_resolution().diagnostics().len(),
            references.label_resolution().has_unresolved(),
        ),
        (1, 1, vec![0], 0, false)
    );
    let (_, import) = resolver_ast
        .imports()
        .imports()
        .next()
        .expect("one resolved import");
    assert_eq!(
        (
            import.owner().index(),
            import.range().start,
            import.range().end,
            import.source_spelling(),
            import.alias(),
            import.origin().source_id(),
            import.origin().module_id(),
            import.origin().structural_path(),
            import.origin().import_edge(),
            import.recovery(),
        ),
        (
            29,
            7,
            27,
            "import parser.type_fixtures;",
            None,
            ast.source_id,
            &module,
            &[0][..],
            None,
            mizar_resolve::resolved_ast::RecoveryState::Normal,
        )
    );
    assert!(matches!(
        import.resolution(),
        mizar_resolve::resolved_ast::ImportResolution::Resolved(target)
            if target.package() == module.package()
                && target.path().as_str() == "parser.type_fixtures"
    ));
    for (id, node) in resolver_ast.nodes().iter() {
        assert_eq!(node.kind(), &ast.nodes()[id.index()].kind);
        assert_eq!(
            node.children()
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            ast.nodes()[id.index()]
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            (
                node.origin().source_id(),
                node.origin().module_id(),
                node.origin().anchor(),
                node.origin().structural_path(),
                node.origin().import_edge(),
                node.origin().is_recovered(),
                node.recovery(),
            ),
            (
                ast.source_id,
                &module,
                &mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range),
                &[id.index() as u32][..],
                None,
                false,
                mizar_resolve::resolved_ast::RecoveryState::Normal,
            )
        );
        if id.index() == 48 {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::Resolved
            );
            assert!(matches!(
                node.reference_key(),
                Some(mizar_resolve::resolved_ast::NodeReferenceKey::Label(key))
                    if key.index() == 0
            ));
        } else {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::NotApplicable
            );
            assert_eq!(node.reference_key(), None);
        }
    }
    let expected_reference_debug =
        task258b5b_expected_reference_debug(&module, references.statement_fingerprint());
    assert_eq!(references.debug_text(), expected_reference_debug);
    assert_eq!(
        output
            .typed_ast
            .debug_text()
            .matches("source-statement-reference-debug-v1")
            .count(),
        1
    );
    assert!(!references.debug_text().contains("\nlabel#0 "));
}

#[test]
fn task258b5b_lower_import_resolver_and_row_mutations_fail_at_owning_stage() {
    let (ast, module, raw, enriched) = task258b5b_exact_env(258_570);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &enriched,
        SOURCE_STATEMENT_B5B_TEXT,
    )
    .expect("B5B selector")
    .expect("B5B baseline");
    let baseline_debug = baseline.typed_ast.debug_text();
    let baseline_resolved_debug = baseline.resolved.debug_text();
    let baseline_references = baseline
        .typed_ast
        .source_statement_references()
        .expect("B5B references");
    let import_id = baseline_references
        .resolver_ast()
        .imports()
        .imports()
        .next()
        .expect("B5B import")
        .0;
    let wrong_node = baseline_references
        .resolver_ast()
        .nodes()
        .iter()
        .find_map(|(id, _)| (id.index() == 47).then_some(id))
        .expect("B5B adjacent node");
    let wrong_contribution = enriched
        .contributions()
        .iter()
        .find_map(|record| (record.id().index() == 1).then_some(record.id()))
        .expect("B5B foreign contribution");

    for context_index in 0..2 {
        let mutated = task258b5a_binding_env_with_context_recovery(
            baseline
                .typed_ast
                .source_statement()
                .expect("B5B statement")
                .binding_env(),
            context_index,
        );
        assert!(
            source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| inputs.binding_env = mutated,
            )
            .expect("Task48 context selector")
            .is_err(),
            "Task48 context {context_index}"
        );
    }
    let mutated_binding = task258b5a_binding_env_with_binding_recovery(
        baseline
            .typed_ast
            .source_statement()
            .expect("B5B statement")
            .binding_env(),
    );
    assert!(
        source_statement_b5b_output_with_mutation(
            &ast,
            module.clone(),
            &enriched,
            SOURCE_STATEMENT_B5B_TEXT,
            |inputs| inputs.binding_env = mutated_binding,
        )
        .expect("Task48 binding selector")
        .is_err()
    );

    let baseline_primary = baseline.typed_ast.source_term().expect("Task252");
    let baseline_binding = baseline
        .typed_ast
        .source_statement()
        .expect("B5B statement")
        .binding_env();
    for index in 0..4 {
        let mut input = task258b3m2a_primary_input(baseline_primary);
        input.terms[index].source_ordinal = 4;
        assert!(
            mizar_checker::source_term::SourcePrimaryTermProducer::build(
                input,
                baseline_binding,
                baseline.typed_ast.nodes(),
            )
            .is_err(),
            "Task252 term {index}"
        );
        let mut input = task258b3m2a_primary_input(baseline_primary);
        input.references[index].term =
            mizar_checker::source_term::SourcePrimaryTermId::new((index + 1) % 4);
        assert!(
            mizar_checker::source_term::SourcePrimaryTermProducer::build(
                input,
                baseline_binding,
                baseline.typed_ast.nodes(),
            )
            .is_err(),
            "Task252 reference {index}"
        );
    }
    let baseline_atomic = baseline.typed_ast.source_atomic_formula().expect("Task256");
    for index in 0..2 {
        let mut input = task258b3m2b1_atomic_input(baseline_atomic);
        input.formulas[index].source_ordinal = 2;
        assert!(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input,
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                baseline.typed_ast.nodes(),
            )
            .is_err(),
            "Task256 formula {index}"
        );
    }
    for index in 0..4 {
        let mut input = task258b3m2b1_atomic_input(baseline_atomic);
        input.edges[index].ordinal = 2;
        assert!(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input,
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                baseline.typed_ast.nodes(),
            )
            .is_err(),
            "Task256 edge {index}"
        );
        let mut input = task258b3m2b1_atomic_input(baseline_atomic);
        input.requests[index].ordinal = 2;
        assert!(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input,
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                baseline.typed_ast.nodes(),
            )
            .is_err(),
            "Task256 request {index}"
        );
    }

    for row in 0..2 {
        for family in 0..4 {
            let result = source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| match family {
                    0 => inputs.statement.statements[row].source_ordinal = 2,
                    1 => {
                        inputs.statement.contexts[row].binding_context =
                            mizar_checker::binding_env::BindingContextId::new((row + 1) % 2)
                    }
                    2 => inputs.statement.input_facts[row].uses.swap(0, 1),
                    3 => inputs.statement.candidate_facts[row].ordinal = 1,
                    _ => unreachable!(),
                },
            )
            .expect("Task258 row selector");
            assert!(result.is_err(), "Task258 family {family} row {row}");
        }
    }
    for mutation in 0..8 {
        let result = source_statement_b5b_output_with_mutation(
            &ast,
            module.clone(),
            &enriched,
            SOURCE_STATEMENT_B5B_TEXT,
            |inputs| match mutation {
                0 => inputs.reference_input.labels.push(
                    mizar_checker::source_statement::SourceStatementLabelInput {
                        statement: mizar_checker::source_statement::SourceStatementId::new(0),
                        context: mizar_checker::source_statement::SourceStatementContextId::new(0),
                        candidate:
                            mizar_checker::source_statement::SourceStatementCandidateFactId::new(0),
                        origin_path: mizar_resolve::resolved_ast::LabelOriginPath::new(
                            "local::wrong",
                        ),
                        proof_scope: mizar_resolve::labels::LabelScopePath::new(vec![0]),
                        source_range: range(ast.source_id, 56, 106),
                        source_ordinal: 0,
                        visible_after_ordinal: 1,
                        spelling: "Wrong".to_owned(),
                        kind: mizar_checker::source_statement::SourceStatementLabelKind::ProofStep,
                        recovery: mizar_checker::source_statement::SourceStatementRecovery::Normal,
                    },
                ),
                1 => {
                    inputs.reference_input.citations[0].statement =
                        mizar_checker::source_statement::SourceStatementId::new(0)
                }
                2 => {
                    inputs.reference_input.citations[0].context =
                        mizar_checker::source_statement::SourceStatementContextId::new(0)
                }
                3 => {
                    inputs.reference_input.citations[0].target =
                        mizar_checker::source_statement::SourceStatementCitationTarget::Local(
                            mizar_checker::source_statement::SourceStatementLabelId::new(0),
                        )
                }
                4 => {
                    inputs.reference_input.citations[0].proof_scope =
                        mizar_resolve::labels::LabelScopePath::new(vec![0, 1])
                }
                5 => inputs.reference_input.citations[0].source_range.start -= 1,
                6 => inputs.reference_input.citations[0].ordinal = 1,
                7 => {
                    inputs.reference_input.citations[0].recovery =
                        mizar_checker::source_statement::SourceStatementRecovery::Degraded
                }
                _ => unreachable!(),
            },
        )
        .expect("citation selector");
        assert!(result.is_err(), "citation mutation {mutation}");
    }

    for mutation in TASK258B5B_IMPORT_MUTATIONS.iter().copied() {
        let mutated = task258b5b_resolver_ast_with_import_mutation(
            baseline
                .typed_ast
                .source_statement_references()
                .expect("B5B references")
                .resolver_ast(),
            &module,
            mutation,
        );
        let rejected = match mutated {
            Err(_) => true,
            Ok(mutated) => source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| inputs.resolver_ast = mutated,
            )
            .expect("import mutation selector")
            .is_err(),
        };
        assert!(rejected, "import mutation {mutation:?}");
    }
    for mutation in TASK258B5B_RESOLVER_NODE_MUTATIONS.iter().copied() {
        let mutated = task258b5b_resolver_ast_with_node_mutation(
            baseline
                .typed_ast
                .source_statement_references()
                .expect("B5B references")
                .resolver_ast(),
            mutation,
        );
        let rejected = match mutated {
            Err(_) => true,
            Ok(mutated) => source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| inputs.resolver_ast = mutated,
            )
            .expect("resolver-node mutation selector")
            .is_err(),
        };
        assert!(rejected, "resolver-node mutation {mutation:?}");
    }
    for mutation in TASK258B5B_PROJECTION_MUTATIONS.iter().copied() {
        assert!(
            source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| {
                    inputs.projection = task258b5b_projection_with_mutation(
                        &inputs.projection,
                        &module,
                        wrong_contribution,
                        import_id,
                        mutation,
                    )
                },
            )
            .expect("projection selector")
            .is_err(),
            "projection mutation {mutation:?}"
        );
    }
    for mutation in TASK258B5B_REFERENCE_MUTATIONS.iter().copied() {
        assert!(
            source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| {
                    inputs.reference = task258b5b_reference_with_mutation(
                        &inputs.reference,
                        &module,
                        wrong_node,
                        import_id,
                        mutation,
                    )
                },
            )
            .expect("reference selector")
            .is_err(),
            "reference mutation {mutation:?}"
        );
    }
    for mutation in 0..2 {
        assert!(
            source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| {
                    if mutation == 0 {
                        inputs.reference_input.citations[0].ordinal = 1;
                    } else {
                        inputs.reference = task258b5b_reference_with_mutation(
                            &inputs.reference,
                            &module,
                            wrong_node,
                            import_id,
                            Task258B5BReferenceMutation::SourceOrdinal,
                        );
                    }
                },
            )
            .expect("independent ordinal selector")
            .is_err(),
            "independent ordinal mutation {mutation}"
        );
    }
    assert!(
        source_statement_b5b_output_with_raw_resolver_mutation(
            &ast,
            module.clone(),
            &raw,
            SOURCE_STATEMENT_B5B_TEXT,
            |mutated| task258b5a_mutate_raw_resolver(
                mutated,
                Task258B5ARawResolverMutation::OwnerOriginRange,
            ),
        )
        .expect("raw resolver selector")
        .is_err()
    );
    let replay =
        source_statement_output_with_source(&ast, module, &enriched, SOURCE_STATEMENT_B5B_TEXT)
            .expect("replay selector")
            .expect("replay");
    assert_eq!(replay.typed_ast.debug_text(), baseline_debug);
    assert_eq!(replay.resolved.debug_text(), baseline_resolved_debug);
}

#[test]
fn task258b5b_source_fixture_and_import_visibility_near_misses_stay_isolated() {
    let (exact_ast, exact_module, raw, enriched) = task258b5b_exact_env(258_580);
    let normal = augment_type_elaboration_import_summaries(&exact_ast, &exact_module, raw.clone());
    assert!(
        source_statement_output_with_source(
            &exact_ast,
            exact_module.clone(),
            &raw,
            SOURCE_STATEMENT_B5B_TEXT,
        )
        .expect("raw B5B selector")
        .is_err()
    );
    assert!(
        source_statement_output_with_source(
            &exact_ast,
            exact_module.clone(),
            &normal,
            SOURCE_STATEMENT_B5B_TEXT,
        )
        .expect("normal B5B selector")
        .is_err()
    );
    assert!(
        source_statement_output_with_source(
            &exact_ast,
            exact_module.clone(),
            &enriched,
            SOURCE_STATEMENT_B5B_TEXT,
        )
        .expect("enriched B5B selector")
        .is_ok()
    );

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    for (ordinal, (relative_path, expected_hash)) in [
        (
            "tests/miz/pass/parser/pass_parser_justifications_001.miz",
            "1f6d8a1a461aa682fffa987adb726bec6b66941bac407728d625f1b9e387d80d",
        ),
        (
            "tests/miz/pass/parser/pass_parser_redefinition_notation_001.miz",
            "ce77250e61a9c483d8d055a7ea195399dd4f602c8dfeb39089acc1fed7d3012e",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let fixture_path = workspace_root.join(relative_path);
        let fixture = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", fixture_path.display()));
        assert_eq!(sha256_text(&fixture), expected_hash, "{relative_path}");
        let (fixture_ast, fixture_module, _, fixture_raw, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&fixture, 258_581 + ordinal);
        assert_eq!(diagnostics, 0, "{relative_path}");
        assert!(
            extract_imported_public_theorem_citation_source_statement(&fixture_ast, &fixture)
                .is_none(),
            "{relative_path}"
        );
        let fixture_symbols =
            augment_type_elaboration_import_summaries(&fixture_ast, &fixture_module, fixture_raw);
        assert!(
            source_statement_output_with_source(
                &fixture_ast,
                fixture_module,
                &fixture_symbols,
                &fixture,
            )
            .is_none(),
            "{relative_path}"
        );
    }
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task258B5B isolation plan");
    assert!(plan.cases.iter().any(|case| {
        case.id.0 == "pass_parser_justifications_001"
            && case
                .source_path
                .ends_with("pass_parser_justifications_001.miz")
    }));
    assert!(plan.cases.iter().any(|case| {
        case.id.0 == "pass_parser_redefinition_notation_001"
            && case
                .source_path
                .ends_with("pass_parser_redefinition_notation_001.miz")
    }));
    let mut active_count = 0;
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        active_count += 1;
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_statement_output_with_source(&ast, resolver.module, &symbols, &source).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(active_count, 266);
    assert!(
        selected.is_empty(),
        "private B5B route selected active type cases: {selected:?}"
    );

    for (label, source) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B5B_TEXT.trim_end_matches('\n').to_owned(),
        ),
        ("extra final LF", format!("{SOURCE_STATEMENT_B5B_TEXT}\n")),
        (
            "import path",
            SOURCE_STATEMENT_B5B_TEXT.replacen("parser.type_fixtures", "parser.other_fixtures", 1),
        ),
        (
            "theorem label",
            SOURCE_STATEMENT_B5B_TEXT.replacen(
                "FormulaStatementImportedPublicTheoremCitationSmoke",
                "FormulaStatementImportedPublicTheoremCitationOther",
                1,
            ),
        ),
        (
            "citation spelling",
            SOURCE_STATEMENT_B5B_TEXT.replacen("by Ref", "by Other", 1),
        ),
        (
            "qualified citation",
            SOURCE_STATEMENT_B5B_TEXT.replacen("by Ref", "by parser.type_fixtures.Ref", 1),
        ),
        (
            "citation removed",
            SOURCE_STATEMENT_B5B_TEXT.replacen(" by Ref", "", 1),
        ),
        (
            "proof removed",
            SOURCE_STATEMENT_B5B_TEXT.replacen(" proof\n  thus x = x by Ref;\nend;", ";", 1),
        ),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &enriched,
                &source,
            )
            .is_none(),
            "{label} loaded-source isolation"
        );
        let (ast, module, _, raw) = task253_ast_from_source_text(&source, 258_583);
        let symbols = super::import_fixtures::
            augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
                &ast, &module, raw,
            );
        assert!(
            source_statement_output_with_source(&ast, module, &symbols, &source).is_none(),
            "{label} parsed-source isolation"
        );
    }
    let foreign_source = task258b3m2b2b3a_foreign_source_id(exact_ast.source_id);
    for mutation in [
        Task258B5BImportedLabelMutation::Missing,
        Task258B5BImportedLabelMutation::Duplicate,
        Task258B5BImportedLabelMutation::Private,
        Task258B5BImportedLabelMutation::LocalOnly,
        Task258B5BImportedLabelMutation::ReExported,
        Task258B5BImportedLabelMutation::WrongKind,
        Task258B5BImportedLabelMutation::WrongSpelling,
        Task258B5BImportedLabelMutation::WrongModule,
        Task258B5BImportedLabelMutation::WrongNamespace,
        Task258B5BImportedLabelMutation::WrongContribution,
        Task258B5BImportedLabelMutation::Recovered,
        Task258B5BImportedLabelMutation::WrongOriginPath,
        Task258B5BImportedLabelMutation::WrongSource,
        Task258B5BImportedLabelMutation::WrongRange,
        Task258B5BImportedLabelMutation::WrongStructuralPath,
        Task258B5BImportedLabelMutation::MissingEffect,
        Task258B5BImportedLabelMutation::ExtraEffect,
        Task258B5BImportedLabelMutation::WrongEffect,
    ] {
        let run = || {
            source_statement_b5b_output_with_resolver_mutation(
                &exact_ast,
                exact_module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |symbols| {
                    task258b5b_mutate_imported_label(
                        &symbols,
                        &exact_module,
                        foreign_source,
                        mutation,
                    )
                },
            )
            .expect("B5B resolver selector")
            .expect_err("B5B resolver mutation")
        };
        assert_eq!(run(), run(), "{mutation:?}");
    }
    let (b1_ast, b1_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_584);
    let (b1, _) = task258_native_output(&b1_ast, &b1_module);
    assert_eq!(b1.source_term().unwrap().references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), (0..8).collect::<Vec<_>>());
    let (b5a_ast, b5a_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_585);
    let (b5a, _) = task258_native_output(&b5a_ast, &b5a_module);
    assert_eq!(b5a.source_term().unwrap().references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), (0..10).collect::<Vec<_>>());
    assert!(
        source_statement_output_with_source(
            &b1_ast,
            exact_module.clone(),
            &enriched,
            SOURCE_STATEMENT_B5B_TEXT,
        )
        .is_none()
    );
}

#[test]
fn task258b5b_b5a_profile_installation_orders_and_ownership_are_atomic() {
    let (b1_ast, b1_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_590);
    let (b1, _) = task258_native_output(&b1_ast, &b1_module);
    let (b5a_ast, b5a_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_590);
    let (b5a, _) = task258_native_output(&b5a_ast, &b5a_module);
    let (b5b_ast, b5b_module, _, b5b_symbols) = task258b5b_exact_env(258_590);
    let b5b = source_statement_output_with_source(
        &b5b_ast,
        b5b_module,
        &b5b_symbols,
        SOURCE_STATEMENT_B5B_TEXT,
    )
    .expect("B5B selector")
    .expect("B5B output");
    assert_eq!(b1.source_id(), b5b.typed_ast.source_id());
    assert_eq!(b1.module_id(), b5b.typed_ast.module_id());
    assert_eq!(b5a.source_id(), b5b.typed_ast.source_id());
    assert_eq!(b5a.module_id(), b5b.typed_ast.module_id());
    let b1_statement = b1
        .source_statement()
        .expect("B1 statement")
        .clone();
    let b1_references = b1
        .source_statement_references()
        .expect("B1 references")
        .clone();
    let b5a_statement = b5a
        .source_statement()
        .expect("B5A statement")
        .clone();
    let b5a_references = b5a
        .source_statement_references()
        .expect("B5A references")
        .clone();
    let b5b_statement = b5b
        .typed_ast
        .source_statement()
        .expect("B5B statement")
        .clone();
    let b5b_references = b5b
        .typed_ast
        .source_statement_references()
        .expect("B5B references")
        .clone();
    assert_eq!(task258_reference_syntax_base(&b1)
        .with_source_statement_references(b1_statement.clone(), b1_references.clone()),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(task258_reference_syntax_base(&b1).with_source_statement(b1_statement.clone()),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(task258_reference_syntax_base(&b5a).with_source_statement_references(b5a_statement.clone(), b5a_references.clone()), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(task258_reference_syntax_base(&b5a).with_source_statement(b5a_statement.clone()), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    {
        let (base, statement, references) =
            (&b5b.typed_ast, b5b_statement.clone(), b5b_references.clone());
        assert!(
            task258_reference_syntax_base(base)
                .with_source_statement_references(statement.clone(), references.clone())
                .is_ok()
        );
        assert_eq!(
            task258_reference_syntax_base(base)
                .with_source_statement(statement)
                .expect_err("partial reference-family installation"),
            mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
        );
    }
    for (label, base, statement, references) in [
        (
            "B5B pair on B1 base",
            &b1,
            b5b_statement.clone(),
            b5b_references.clone(),
        ),
        (
            "B1 pair on B5B base",
            &b5b.typed_ast,
            b1_statement.clone(),
            b1_references.clone(),
        ),
        (
            "B1 statement with B5B references",
            &b1,
            b1_statement.clone(),
            b5b_references.clone(),
        ),
        (
            "B5B statement with B1 references",
            &b5b.typed_ast,
            b5b_statement.clone(),
            b1_references.clone(),
        ),
        (
            "B5B pair on B5A base",
            &b5a,
            b5b_statement.clone(),
            b5b_references.clone(),
        ),
        (
            "B5A pair on B5B base",
            &b5b.typed_ast,
            b5a_statement.clone(),
            b5a_references.clone(),
        ),
        (
            "B5A statement with B5B references",
            &b5a,
            b5a_statement.clone(),
            b5b_references.clone(),
        ),
        (
            "B5B statement with B5A references",
            &b5b.typed_ast,
            b5b_statement.clone(),
            b5a_references.clone(),
        ),
    ] {
        assert_eq!(
            task258_reference_syntax_base(base)
                .with_source_statement_references(statement, references)
                .unwrap_err(),
            mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement,
            "{label}"
        );
    }
    let installed = task258_reference_syntax_base(&b5b.typed_ast)
        .with_source_statement_references(b5b_statement.clone(), b5b_references.clone())
        .expect("matching B5B pair");
    let installed_debug = installed.debug_text();
    assert_eq!(
        installed
            .clone()
            .with_source_statement_references(b5b_statement.clone(), b5b_references.clone())
            .expect_err("duplicate B5B pair"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(installed.debug_text(), installed_debug);
    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    for (label, statement, references) in [
        ("Task248/B1", b1_statement.clone(), b1_references.clone()),
        ("Task248/B5A", b5a_statement.clone(), b5a_references.clone()),
        ("Task248/B5B", b5b_statement.clone(), b5b_references.clone()),
    ] {
        assert_eq!(
            task248
                .typed_ast
                .clone()
                .with_source_statement_references(statement, references)
                .unwrap_err(),
            mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement,
            "{label}"
        );
    }
    assert_eq!(task248.typed_ast.debug_text(), before);
    assert_eq!(b1.source_statement(), Some(&b1_statement));
    assert_eq!(
        b1.source_statement_references(),
        Some(&b1_references)
    );
    assert_eq!(b5a.source_statement(), Some(&b5a_statement));
    assert_eq!(
        b5a.source_statement_references(),
        Some(&b5a_references)
    );
    assert_eq!(b5b.typed_ast.source_statement(), Some(&b5b_statement));
    assert_eq!(
        b5b.typed_ast.source_statement_references(),
        Some(&b5b_references)
    );
}

#[test]
fn task258b5b_final_clone_debug_replay_and_empty_semantics_are_stable() {
    let (ast, module, _, enriched) = task258b5b_exact_env(258_600);
    let first = source_statement_output_with_source(
        &ast,
        module.clone(),
        &enriched,
        SOURCE_STATEMENT_B5B_TEXT,
    )
    .expect("B5B selector")
    .expect("B5B output");
    let second = source_statement_output_with_source(
        &ast,
        module.clone(),
        &enriched,
        SOURCE_STATEMENT_B5B_TEXT,
    )
    .expect("B5B replay selector")
    .expect("B5B replay");
    assert_eq!(first.typed_ast, first.typed_ast.clone());
    assert_eq!(first.resolved, first.resolved.clone());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let final_references = first
        .typed_ast
        .source_statement_references()
        .expect("B5B references");
    let replay_references = second
        .typed_ast
        .source_statement_references()
        .expect("B5B replay references");
    let final_reference_debug = final_references.debug_text();
    assert_eq!(
        final_reference_debug,
        task258b5b_expected_reference_debug(&module, final_references.statement_fingerprint())
    );
    assert_eq!(final_reference_debug, replay_references.debug_text());
    assert!(final_reference_debug.contains("label_node=absent"));
    assert!(final_reference_debug.contains("resolver-projection source=imported"));
    assert!(final_reference_debug.contains("target=imported"));
    assert!(!final_reference_debug.contains("\nlabel#0 "));
    assert_eq!(first.reference_use_ordinals, [1; 4]);
    assert_eq!(
        (first.left_lookup_ordinal, first.right_lookup_ordinal),
        (1, 1)
    );
    assert!(
        source_statement_transport_detail_keys(
            &ast,
            module.clone(),
            &enriched,
            SOURCE_STATEMENT_B5B_TEXT,
        )
        .expect("B5B transport")
        .is_empty()
    );
    assert_eq!(
        source_statement_transport_detail_keys_with_output_mutation_for_test(
            &ast,
            module.clone(),
            &enriched,
            SOURCE_STATEMENT_B5B_TEXT,
            |output| {
                output.reference_use_ordinals.pop();
            },
        )
        .expect("mutated B5B telemetry"),
        ["type_elaboration.checker.typed_ast_invalid"]
    );
    let final_import = final_references
        .resolver_ast()
        .imports()
        .imports()
        .next()
        .expect("B5B import")
        .0;
    let wrong_node = final_references
        .resolver_ast()
        .nodes()
        .iter()
        .find_map(|(id, _)| (id.index() == 47).then_some(id))
        .expect("B5B adjacent resolver node");
    let wrong_contribution = enriched
        .contributions()
        .iter()
        .find_map(|record| (record.id().index() == 1).then_some(record.id()))
        .expect("B5B non-label contribution");
    for mutation in TASK258B5B_IMPORT_MUTATIONS.iter().copied() {
        let mutated = task258b5b_resolver_ast_with_import_mutation(
            final_references.resolver_ast(),
            &module,
            mutation,
        );
        let rejected = match mutated {
            Err(_) => true,
            Ok(mutated) => source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| inputs.resolver_ast = mutated,
            )
            .expect("final import mutation selector")
            .is_err(),
        };
        assert!(rejected, "final import mutation {mutation:?}");
    }
    for mutation in TASK258B5B_RESOLVER_NODE_MUTATIONS.iter().copied() {
        let mutated =
            task258b5b_resolver_ast_with_node_mutation(final_references.resolver_ast(), mutation);
        let rejected = match mutated {
            Err(_) => true,
            Ok(mutated) => source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| inputs.resolver_ast = mutated,
            )
            .expect("final resolver-node mutation selector")
            .is_err(),
        };
        assert!(rejected, "final resolver-node mutation {mutation:?}");
    }
    for mutation in TASK258B5B_PROJECTION_MUTATIONS.iter().copied() {
        assert!(
            source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| {
                    inputs.projection = task258b5b_projection_with_mutation(
                        &inputs.projection,
                        &module,
                        wrong_contribution,
                        final_import,
                        mutation,
                    )
                },
            )
            .expect("final projection mutation selector")
            .is_err(),
            "final projection mutation {mutation:?}"
        );
    }
    for mutation in TASK258B5B_REFERENCE_MUTATIONS.iter().copied() {
        assert!(
            source_statement_b5b_output_with_mutation(
                &ast,
                module.clone(),
                &enriched,
                SOURCE_STATEMENT_B5B_TEXT,
                |inputs| {
                    inputs.reference = task258b5b_reference_with_mutation(
                        &inputs.reference,
                        &module,
                        wrong_node,
                        final_import,
                        mutation,
                    )
                },
            )
            .expect("final reference mutation selector")
            .is_err(),
            "final reference mutation {mutation:?}"
        );
    }
    let post_mutation_replay = source_statement_output_with_source(
        &ast,
        module.clone(),
        &enriched,
        SOURCE_STATEMENT_B5B_TEXT,
    )
    .expect("post-mutation replay selector")
    .expect("post-mutation replay");
    assert_eq!(
        post_mutation_replay.typed_ast.debug_text(),
        first.typed_ast.debug_text()
    );
    assert_eq!(
        post_mutation_replay.resolved.debug_text(),
        first.resolved.debug_text()
    );
    assert_eq!(
        post_mutation_replay
            .typed_ast
            .source_statement_references()
            .expect("post-mutation B5B references")
            .debug_text(),
        final_reference_debug
    );
    assert!(first.typed_ast.source_context().is_none());
    assert!(first.typed_ast.source_type().is_none());
    assert!(first.typed_ast.source_attribute().is_none());
    assert!(first.typed_ast.source_evidence().is_none());
    assert!(first.typed_ast.source_application().is_none());
    assert!(first.typed_ast.source_structure().is_none());
    assert!(first.typed_ast.source_set_term().is_none());
    assert!(first.typed_ast.source_statement_witnesses().is_none());
    assert!(first.typed_ast.source_composite_formula().is_none());
    assert!(first.typed_ast.source_formula_composition().is_none());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());

    let (b5a_ast, b5a_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_601);
    let (b5a, b5a_resolved) = task258_native_output(&b5a_ast, &b5a_module);
    let hybrid = source_statement_transport_detail_keys_with_output_mutation_for_test(
        &ast,
        module.clone(),
        &enriched,
        SOURCE_STATEMENT_B5B_TEXT,
        |output| {
            output.typed_ast = b5a.clone();
            output.resolved = b5a_resolved.clone();
            output.left_lookup_ordinal = 1;
            output.right_lookup_ordinal = 1;
            output.reference_use_ordinals = vec![1; 4];
        },
    )
    .expect("B5B/B5A final transport");
    assert!(
        !hybrid.is_empty(),
        "B5A typed/resolved pair plus B5B telemetry passed under B5B source"
    );
    assert_eq!(
        hybrid,
        source_statement_transport_detail_keys_with_output_mutation_for_test(
            &ast,
            module,
            &enriched,
            SOURCE_STATEMENT_B5B_TEXT,
            |output| {
                output.typed_ast = b5a.clone();
                output.resolved = b5a_resolved.clone();
                output.left_lookup_ordinal = 1;
                output.right_lookup_ordinal = 1;
                output.reference_use_ordinals = vec![1; 4];
            },
        )
        .expect("B5B/B5A final transport replay")
    );
}
