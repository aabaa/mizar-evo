#[test]
fn task258b5a_real_route_publishes_exact_ancestor_label_descendant_citation() {
    assert_eq!(SOURCE_STATEMENT_B5A_TEXT.len(), 185);
    assert!(SOURCE_STATEMENT_B5A_TEXT.ends_with('\n'));
    assert!(!SOURCE_STATEMENT_B5A_TEXT.ends_with("\n\n"));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B5A_TEXT),
        "ce9639d454169ffb49452bd4a4b6b15767ff590cef2b3ed0210946132c5d26c7"
    );
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_500);
    assert!(task258b5a_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B5A_TEXT,
        SourceStatementB5ASurfaceMutation::None,
    ));
    for index in 0..93 {
        for mutation in [
            SourceStatementB5ASurfaceMutation::NodeKind(index),
            SourceStatementB5ASurfaceMutation::NodeRange(index),
            SourceStatementB5ASurfaceMutation::NodeRecovery(index),
            SourceStatementB5ASurfaceMutation::NodeChildren(index),
        ] {
            assert!(
                !task258b5a_surface_profile_with_mutation_for_test(
                    &ast,
                    SOURCE_STATEMENT_B5A_TEXT,
                    mutation,
                ),
                "{mutation:?}"
            );
        }
    }
    assert!(!task258b5a_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B5A_TEXT,
        SourceStatementB5ASurfaceMutation::RootIdentity,
    ));
    let extracted: SourceStatementB5AExtraction =
        extract_ancestor_label_descendant_citation_source_statement(
            &ast,
            SOURCE_STATEMENT_B5A_TEXT,
        )
        .expect("exact B5A surface");
    assert_eq!(ast.nodes().len(), 93);
    assert_eq!(ast.root().expect("root").index(), 92);
    assert_eq!(extracted.theorem_site.node().index(), 89);
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [89, 67, 65, 87, 85]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .map(|range| (range.start, range.end)),
        [(19, 184), (95, 132), (114, 125), (135, 179), (156, 172)]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [49, 55, 62, 72, 79]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [45, 47, 51, 53, 58, 60, 68, 70, 75, 77]
    );
    assert_eq!(
        extracted.proof_ranges.map(|range| (range.start, range.end)),
        [(87, 183), (104, 131), (146, 178)]
    );
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
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let raw_owners = symbols.symbols().visible_candidates(
        &namespace,
        "FormulaStatementAncestorLabelDescendantCitationSmoke",
    );
    let [raw_owner] = raw_owners.as_slice() else {
        panic!("one raw B5A theorem owner")
    };
    assert_eq!(
        (
            raw_owner.kind(),
            raw_owner.visibility(),
            raw_owner.export_status(),
            raw_owner.namespace(),
            raw_owner.primary_spelling(),
            raw_owner.contribution().index(),
            raw_owner.origin().source_id(),
            raw_owner.origin().module_id(),
            raw_owner.origin().anchor(),
            raw_owner.origin().structural_path(),
            raw_owner.origin().import_edge(),
            raw_owner.origin().is_recovered(),
        ),
        (
            mizar_resolve::env::SymbolKind::Theorem,
            mizar_resolve::env::Visibility::Public,
            mizar_resolve::env::ExportStatus::Exported,
            &namespace,
            "FormulaStatementAncestorLabelDescendantCitationSmoke",
            0,
            ast.source_id,
            &module,
            &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 184)),
            [2, 1].as_slice(),
            None,
            false,
        )
    );
    let raw_contribution = symbols
        .contributions()
        .get(raw_owner.contribution())
        .expect("raw B5A contribution");
    assert_eq!(
        raw_contribution.anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 18))
    );
    assert_eq!(
        raw_contribution.effects().symbols(),
        [raw_owner.symbol().clone()]
    );
    assert_eq!(raw_contribution.effects().definitions().len(), 1);
    assert!(raw_contribution.effects().labels().is_empty());
    let enriched = source_statement_b5a_resolver_env_for_test(
        &ast,
        &module,
        &symbols,
        SOURCE_STATEMENT_B5A_TEXT,
    )
    .expect("enriched B5A resolver provenance");
    assert_eq!(
        (
            enriched.symbols().len(),
            enriched.labels().len(),
            enriched.definitions().len(),
            enriched.contributions().len(),
            enriched.imports().len(),
        ),
        (1, 1, 1, 1, 0)
    );
    let enriched_labels = enriched.labels().visible_candidates(
        &namespace,
        "FormulaStatementAncestorLabelDescendantCitationSmoke",
    );
    let [theorem_label] = enriched_labels.as_slice() else {
        panic!("one enriched B5A theorem label")
    };
    assert_eq!(
        (
            theorem_label.kind(),
            theorem_label.visibility(),
            theorem_label.export_status(),
            theorem_label.namespace(),
            theorem_label.primary_spelling(),
            theorem_label.contribution().index(),
            theorem_label.origin().source_id(),
            theorem_label.origin().module_id(),
            theorem_label.origin().anchor(),
            theorem_label.origin().structural_path(),
            theorem_label.origin().import_edge(),
            theorem_label.origin().is_recovered(),
        ),
        (
            mizar_resolve::resolved_ast::LabelKind::Theorem,
            mizar_resolve::env::Visibility::Public,
            mizar_resolve::env::ExportStatus::Exported,
            &namespace,
            "FormulaStatementAncestorLabelDescendantCitationSmoke",
            0,
            ast.source_id,
            &module,
            &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 184)),
            [2, 1].as_slice(),
            None,
            false,
        )
    );
    assert_eq!(theorem_label.origin(), raw_owner.origin());
    assert_eq!(theorem_label.contribution(), raw_owner.contribution());
    let enriched_contribution = enriched
        .contributions()
        .get(theorem_label.contribution())
        .expect("enriched B5A contribution");
    assert_eq!(
        enriched_contribution.effects().symbols(),
        [raw_owner.symbol().clone()]
    );
    assert_eq!(enriched_contribution.effects().definitions().len(), 1);
    assert_eq!(
        enriched_contribution.effects().labels(),
        [theorem_label.origin_path().clone()]
    );
    let (native_source, native_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
        .expect("original B5A source collection").expect("original B5A theorem module");
    let (native_scope, native_labels, native_resolution) = mizar_resolve::symbols::source_proof_inputs(&native_source, &native_symbols)
        .expect("whole original B5A resolver inputs");
    assert_eq!(native_labels.projections().len(), 2);
    let native_projection = native_labels.projections().iter().find(|projection| projection.kind() == mizar_resolve::resolved_ast::LabelKind::ProofStep).expect("actual B5A proof projection");
    let native_owner = native_labels.projections().iter().find(|projection| projection.kind() == mizar_resolve::resolved_ast::LabelKind::Theorem).expect("actual B5A theorem projection");
    assert!(matches!(native_owner.source(), mizar_resolve::labels::LabelProjectionSource::CurrentModule { visible_after_ordinal: 4, proof_scope: None }));
    let (typed, resolved) = task258_native_output(&ast, &module);
    let statement = typed.source_statement().expect("B5A statement");
    let references = typed
        .source_statement_references()
        .expect("B5A references");
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
        (1, 5, 5, 5, 5, 1, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("owner");
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
        ),
        (
            89,
            19,
            184,
            "FormulaStatementAncestorLabelDescendantCitationSmoke",
            0,
            mizar_checker::source_statement::SourceTheoremRole::Theorem,
            mizar_checker::source_statement::SourceTheoremStatus::Unmodified,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(owner.symbol(), raw_owner.symbol());
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.context().index(),
                row.formula(),
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
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
                ),
                89,
                19,
                184,
                0,
                "theorem FormulaStatementAncestorLabelDescendantCitationSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x proof thus x = x by A ; end ; end ;",
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
            ),
            (
                1,
                1,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1),
                ),
                67,
                95,
                132,
                1,
                "A : x = x proof thus x = x ; end ;",
                mizar_checker::source_statement::SourceStatementKind::ProofStepProposition,
            ),
            (
                2,
                2,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                ),
                65,
                114,
                125,
                2,
                "thus x = x ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
            ),
            (
                3,
                3,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(3),
                ),
                87,
                135,
                179,
                3,
                "thus x = x proof thus x = x by A ; end ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
            ),
            (
                4,
                4,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(4),
                ),
                85,
                156,
                172,
                4,
                "thus x = x by A ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
            ),
        ]
    );
    assert_eq!(
        statement
            .contexts()
            .iter()
            .map(|(_, row)| (
                row.binding_context().index(),
                row.source_range().start,
                row.source_range().end,
            ))
            .collect::<Vec<_>>(),
        [
            (0, 19, 184),
            (1, 95, 132),
            (2, 114, 125),
            (1, 135, 179),
            (3, 156, 172)
        ]
    );
    assert_eq!(statement.binding_env().contexts().len(), 4);
    let module_context = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("module context");
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
            [mizar_checker::binding_env::BindingId::new(0)].as_slice(),
            [mizar_checker::binding_env::BindingId::new(0)].as_slice(),
            mizar_checker::binding_env::BindingContextRecovery::Normal,
        )
    );
    for (id, parent, scope, expected_range) in [
        (1, 0, &[4][..], (87, 183)),
        (2, 1, &[4, 1][..], (104, 131)),
        (3, 1, &[4, 2][..], (146, 178)),
    ] {
        let context = statement
            .binding_env()
            .contexts()
            .get(mizar_checker::binding_env::BindingContextId::new(id))
            .expect("proof context");
        assert_eq!(context.parent.map(|value| value.index()), Some(parent));
        assert_eq!(context.lexical_scope.as_ref().expect("scope").path(), scope);
        assert_eq!(
            context.layer,
            mizar_checker::binding_env::BindingContextLayer::Proof
        );
        assert!(context.bindings.is_empty());
        assert_eq!(
            context.visible_bindings,
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        assert_eq!(
            context.recovery,
            mizar_checker::binding_env::BindingContextRecovery::Normal
        );
        assert!(matches!(
            context.owner,
            mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
                if (source_range.start, source_range.end) == expected_range
        ));
    }
    assert_eq!(statement.binding_env().bindings().len(), 1);
    assert!(statement.binding_env().diagnostics().is_empty());
    let binding = statement
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("reserved binding");
    assert_eq!(
        (
            binding.id.index(),
            binding.spelling.as_str(),
            binding.kind,
            binding.owner_context.index(),
            binding.declaration_range.start,
            binding.declaration_range.end,
            binding.visible_after_ordinal,
            binding.status,
            binding.recovery,
        ),
        (
            0,
            "x",
            mizar_checker::binding_env::BindingKind::ReservedVariable,
            0,
            8,
            9,
            0,
            mizar_checker::binding_env::BindingStatus::Active,
            mizar_checker::binding_env::BindingRecoveryState::Normal,
        )
    );
    assert!(matches!(
        binding.identity,
        mizar_checker::binding_env::BinderIdentity::ReservedVariable { .. }
    ));
    assert_eq!(binding.type_site, mizar_checker::binding_env::BindingTypeSite::Missing);
    assert_eq!((ast.nodes()[42].range.start, ast.nodes()[42].range.end), (14, 17));
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(
        statement.binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(_, row)| row
                .uses()
                .iter()
                .map(|value| value.index())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [vec![0, 1], vec![2, 3], vec![4, 5], vec![6, 7], vec![8, 9]]
    );
    assert_eq!(
        statement
            .candidate_facts()
            .iter()
            .map(|(_, row)| row.formula())
            .collect::<Vec<_>>(),
        (0..5)
            .map(|index| {
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
                )
            })
            .collect::<Vec<_>>()
    );
    for index in 0..5 {
        let statement_row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(
                index,
            ))
            .expect("statement row");
        let context_row = statement
            .contexts()
            .get(mizar_checker::source_statement::SourceStatementContextId::new(index))
            .expect("statement context row");
        let input_row = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact row");
        let candidate_row = statement
            .candidate_facts()
            .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(index))
            .expect("candidate fact row");
        let expected_binding_context = [0, 1, 2, 1, 3][index];
        assert_eq!(statement_row.owner().index(), 0);
        assert_eq!(statement_row.context().index(), index);
        assert_eq!(
            statement_row.formula(),
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(
            statement_row.recovery(),
            mizar_checker::source_statement::SourceStatementRecovery::Normal
        );
        assert_eq!(context_row.statement().index(), index);
        assert_eq!(
            context_row.binding_context().index(),
            expected_binding_context
        );
        assert_eq!(
            context_row.visible_bindings(),
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        assert_eq!(input_row.statement().index(), index);
        assert_eq!(input_row.context().index(), index);
        assert_eq!(input_row.ordinal(), 0);
        assert_eq!(
            input_row.kind(),
            mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard
        );
        assert_eq!(input_row.binding().index(), 0);
        assert_eq!(
            input_row
                .uses()
                .iter()
                .map(|reference| reference.index())
                .collect::<Vec<_>>(),
            [2 * index, 2 * index + 1]
        );
        assert_eq!(candidate_row.statement().index(), index);
        assert_eq!(candidate_row.context().index(), index);
        assert_eq!(candidate_row.ordinal(), 0);
        assert_eq!(
            candidate_row.kind(),
            mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate_row.formula(), statement_row.formula());
    }
    let primary = typed.source_term().expect("Task252");
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(_, term)| (
                term.site().node().index(),
                term.context().index(),
                term.source_range().start,
                term.source_range().end,
            ))
            .collect::<Vec<_>>(),
        [
            (45, 0, 81, 82),
            (47, 0, 85, 86),
            (51, 1, 98, 99),
            (53, 1, 102, 103),
            (58, 2, 119, 120),
            (60, 2, 123, 124),
            (68, 1, 140, 141),
            (70, 1, 144, 145),
            (75, 3, 161, 162),
            (77, 3, 165, 166),
        ]
    );
    for (index, expected_scope) in [
        Some(vec![]),
        Some(vec![]),
        Some(vec![4]),
        Some(vec![4]),
        Some(vec![4, 1]),
        Some(vec![4, 1]),
        Some(vec![4]),
        Some(vec![4]),
        Some(vec![4, 2]),
        Some(vec![4, 2]),
    ]
    .into_iter()
    .enumerate()
    {
        let term = primary
            .terms()
            .get(mizar_checker::source_term::SourcePrimaryTermId::new(index))
            .expect("B5A primary term row");
        let reference = primary
            .references()
            .get(mizar_checker::source_term::SourcePrimaryTermReferenceId::new(index))
            .expect("B5A primary reference row");
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(
            term.recovery(),
            mizar_checker::source_term::SourcePrimaryTermRecovery::Normal
        );
        assert_eq!(term.spelling(), "x");
        assert_eq!(
            term.kind(),
            mizar_checker::source_term::SourcePrimaryTermKind::VariableReference
        );
        assert_eq!(
            term.role(),
            mizar_checker::source_term::SourcePrimaryTermRole::Value
        );
        assert_eq!(term.parent(), None);
        assert_eq!(reference.term().index(), index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(
            reference.lexical_scope().map(|scope| scope.path().to_vec()),
            expected_scope
        );
        assert_eq!(reference.use_ordinal(), index);
        assert_eq!(
            reference.role(),
            mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
        );
    }
    assert!(primary.numeric_type_requests().is_empty());
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| (
                row.binding().index(),
                row.lexical_scope().map(|scope| scope.path().to_vec()),
                row.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, Some(vec![]), 0),
            (0, Some(vec![]), 1),
            (0, Some(vec![4]), 2),
            (0, Some(vec![4]), 3),
            (0, Some(vec![4, 1]), 4),
            (0, Some(vec![4, 1]), 5),
            (0, Some(vec![4]), 6),
            (0, Some(vec![4]), 7),
            (0, Some(vec![4, 2]), 8),
            (0, Some(vec![4, 2]), 9),
        ]
    );
    let atomic = typed.source_atomic_formula().expect("Task256");
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (5, 10, 10)
    );
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.context().index(),
                row.source_range().start,
                row.source_range().end,
            ))
            .collect::<Vec<_>>(),
        [
            (49, 0, 81, 86),
            (55, 1, 98, 103),
            (62, 2, 119, 124),
            (72, 1, 140, 145),
            (79, 3, 161, 166),
        ]
    );
    assert!(atomic.wrappers().is_empty());
    assert!(atomic.predicate_segments().is_empty());
    assert!(atomic.predicate_heads().is_empty());
    assert!(atomic.candidates().is_empty());
    assert!(atomic.type_sites().is_empty());
    assert!(atomic.attributes().is_empty());
    for index in 0..5 {
        let formula = atomic
            .formulas()
            .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index))
            .expect("B5A atomic formula row");
        assert_eq!(formula.source_ordinal(), index);
        assert_eq!(formula.context().index(), [0, 1, 2, 1, 3][index]);
        assert_eq!(
            formula.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
        );
        assert_eq!(
            formula.recovery(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
        );
        assert_eq!(formula.spelling(), "x = x");
    }
    for index in 0..10 {
        let edge = atomic
            .edges()
            .get(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index))
            .expect("B5A atomic edge row");
        let request = atomic
            .requests()
            .get(mizar_checker::source_atomic_formula::SourceAtomicRequestId::new(index))
            .expect("B5A atomic request row");
        assert_eq!(edge.formula().index(), index / 2);
        assert_eq!(edge.ordinal(), index % 2);
        assert_eq!(
            edge.role(),
            if index % 2 == 0 {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
        assert_eq!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(index),
            )
        );
        assert_eq!(request.formula().index(), index / 2);
        assert_eq!(request.ordinal(), index % 2);
        assert_eq!(
            request.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
        );
        assert_eq!(request.edge().map(|edge| edge.index()), Some(index));
        assert_eq!(request.candidate(), None);
        assert_eq!(request.type_site(), None);
        assert_eq!(request.attribute(), None);
    }
    assert_eq!(primary.references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), (0..10).collect::<Vec<_>>());
    assert_eq!(primary.references().iter().map(|(_, reference)| (reference.lexical_scope().unwrap(), reference.use_ordinal(), reference.binding().index())).collect::<Vec<_>>(), native_scope.references().iter().map(|reference| (reference.scope(), reference.ordinal(), reference.binding().index())).collect::<Vec<_>>());
    assert_eq!(typed.nodes().len(), 93);
    assert_eq!(typed.nodes().root().map(|id| id.index()), Some(92));
    assert_eq!(typed.resolved_root(), Some(native_source.arena().root()));
    for (id, node) in typed.nodes().iter() {
        let (resolved_id, original) = native_source.arena().iter().nth(id.index()).expect("original native node");
        let expected_kind = match original.kind() {
            mizar_syntax::ast::SurfaceNodeKind::TermReference => "source.term.variable-reference".to_owned(),
            mizar_syntax::ast::SurfaceNodeKind::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
            kind => format!("{kind:?}"),
        };
        assert_eq!(node.kind.as_str(), expected_kind);
        assert_eq!(node.resolved_node, Some(resolved_id));
        assert_eq!(node.recovery, mizar_checker::typed_ast::NodeRecoveryState::Normal);
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range)
        );
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            ast.nodes()[id.index()]
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
    }
    let label = references
        .labels()
        .get(mizar_checker::source_statement::SourceStatementLabelId::new(0))
        .expect("label");
    let citation = references
        .citations()
        .get(mizar_checker::source_statement::SourceStatementCitationId::new(0))
        .expect("citation");
    assert_eq!(
        (
            label.statement().index(),
            label.context().index(),
            label.candidate().index(),
            label.proof_scope().path(),
            label.source_range().start,
            label.source_range().end,
            label.source_ordinal(),
            label.visible_after_ordinal(),
            label.spelling(),
            label.kind(),
            label.recovery(),
        ),
        (
            1,
            1,
            1,
            &[0][..],
            95,
            96,
            0,
            2,
            "A",
            mizar_checker::source_statement::SourceStatementLabelKind::ProofStep,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(label.origin_path(), native_projection.origin_path());
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
            4,
            4,
            mizar_checker::source_statement::SourceStatementCitationTarget::Local(
                mizar_checker::source_statement::SourceStatementLabelId::new(0),
            ),
            0,
            &[0, 1][..],
            170,
            171,
            0,
            mizar_checker::source_statement::SourceStatementCitationKind::SimpleLocal,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    let projection = references.label_projection();
    assert_eq!(projection, native_projection);
    assert_eq!(references.reference_candidate(), &native_labels.references()[0]);
    assert_eq!(references.label_resolution(), &native_resolution);
    assert_eq!(resolved.source_statement(), Some(statement));
    assert_eq!(resolved.source_statement_references(), Some(references));
    assert_eq!(
        (
            projection.origin_path(),
            projection.module(),
            projection.namespace(),
            projection.primary_spelling(),
            projection.kind(),
            projection.visibility(),
            projection.export_status(),
            projection.declaration_range(),
            projection.contribution().index(),
        ),
        (
            label.origin_path(),
            &module,
            &namespace,
            "A",
            mizar_resolve::resolved_ast::LabelKind::ProofStep,
            mizar_resolve::env::Visibility::Private,
            mizar_resolve::env::ExportStatus::LocalOnly,
            range(ast.source_id, 95, 96),
            0,
        )
    );
    assert_eq!(
        (
            projection.origin().source_id(),
            projection.origin().module_id(),
            projection.origin().anchor(),
            projection.origin().structural_path(),
            projection.origin().import_edge(),
            projection.origin().is_recovered(),
        ),
        (
            ast.source_id,
            &module,
            &mizar_session::SourceAnchor::Range(range(ast.source_id, 95, 96)),
            [89, 67, 12].as_slice(),
            None,
            false,
        )
    );
    assert!(matches!(
        projection.source(),
        mizar_resolve::labels::LabelProjectionSource::CurrentModule {
            visible_after_ordinal: 2,
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(
        (
            references.reference_candidate().site().node().index(),
            references.reference_candidate().site().range(),
            references.reference_candidate().site().spelling(),
            references.reference_candidate().ordinal(),
            references.reference_candidate().expectation(),
            references.reference_candidate().origin().source_id(),
            references.reference_candidate().origin().module_id(),
            references.reference_candidate().origin().anchor(),
            references.reference_candidate().origin().structural_path(),
            references.reference_candidate().origin().import_edge(),
            references.reference_candidate().origin().is_recovered(),
        ),
        (
            82,
            range(ast.source_id, 170, 171),
            "A",
            4,
            mizar_resolve::resolved_ast::LabelExpectation::ProofOrTheorem,
            ast.source_id,
            &module,
            &mizar_session::SourceAnchor::Range(range(ast.source_id, 170, 171)),
            &[89, 85, 82][..],
            None,
            false,
        )
    );
    assert!(matches!(
        references.reference_candidate().scope(),
        mizar_resolve::labels::LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } if scope.path() == [0, 1]
    ));
    assert_eq!(references.resolver_ast().nodes().len(), 93);
    assert_eq!(references.resolver_ast().nodes().root().index(), 92);
    assert_eq!(
        references.label_resolution().index().len(),
        2,
        "whole theorem and proof-step projections"
    );
    assert_eq!(references.label_resolution().table().len(), 1);
    assert_eq!(
        references
            .label_resolution()
            .ids()
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [0]
    );
    assert!(references.label_resolution().diagnostics().is_empty());
    assert!(!references.label_resolution().has_unresolved());
    assert!(references.resolver_ast().name_refs().is_empty());
    assert_eq!(references.resolver_ast().label_refs().len(), 1);
    assert!(
        references
            .resolver_ast()
            .imports()
            .imports()
            .next()
            .is_none()
    );
    assert!(
        references
            .resolver_ast()
            .imports()
            .exports()
            .next()
            .is_none()
    );
    for (id, node) in references.resolver_ast().nodes().iter() {
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
        if id.index() == 82 {
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
}

#[test]
fn task258b5a_lower_resolver_scope_and_row_mutations_fail_at_owning_stage() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_510);
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).expect("genuine B5A native base");
    let (typed, resolved) = task258_native_output(&ast, &module);
    let before = (source.clone(), symbols.clone(), scope.clone(), base.clone(), typed.clone(), resolved.clone());
    let baseline_primary = typed.source_term().expect("B5A primary");
    let baseline_binding = typed
        .source_statement()
        .expect("B5A statement")
        .binding_env();
    let primary_input = task258b3m2a_primary_input(baseline_primary);
    let primary_input_before = primary_input.clone();
    let generic_primary = mizar_checker::source_term::SourcePrimaryTermProducer::build(primary_input.clone(), baseline_binding, typed.nodes()).expect("generic primary baseline from genuine native rows");
    assert_eq!(primary_input, primary_input_before);
    assert_eq!(generic_primary.references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), vec![1; 10]);
    assert_eq!(baseline_primary.references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), (0..10).collect::<Vec<_>>());
    for term_index in 0..10 {
        let mut primary_input = task258b3m2a_primary_input(baseline_primary);
        primary_input.terms[term_index].source_ordinal = 10;
        assert!(
            mizar_checker::source_term::SourcePrimaryTermProducer::build(
                primary_input,
                baseline_binding,
                typed.nodes(),
            )
            .is_err(),
            "Task252 term row {term_index} was accepted"
        );
    }
    for reference_index in 0..10 {
        let mut primary_input = task258b3m2a_primary_input(baseline_primary);
        primary_input.references[reference_index].term =
            mizar_checker::source_term::SourcePrimaryTermId::new((reference_index + 1) % 10);
        assert!(
            mizar_checker::source_term::SourcePrimaryTermProducer::build(
                primary_input,
                baseline_binding,
                typed.nodes(),
            )
            .is_err(),
            "Task252 reference row {reference_index} was accepted"
        );
    }

    let baseline_atomic = typed
        .source_atomic_formula()
        .expect("B5A atomic");
    let atomic_input = task258b3m2b1_atomic_input(baseline_atomic);
    let atomic_input_before = atomic_input.clone();
    let generic_atomic = mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(atomic_input.clone(), baseline_binding, &symbols, baseline_primary, None, None, None, typed.nodes()).expect("generic atomic baseline from genuine native rows");
    assert_eq!(atomic_input, atomic_input_before);
    assert_eq!(&generic_atomic, baseline_atomic);
    for formula_index in 0..5 {
        let mut atomic_input = task258b3m2b1_atomic_input(baseline_atomic);
        atomic_input.formulas[formula_index].source_ordinal = 5;
        assert!(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                atomic_input,
                baseline_binding,
                &symbols,
                baseline_primary,
                None,
                None,
                None,
                typed.nodes(),
            )
            .is_err(),
            "Task256 formula row {formula_index} was accepted"
        );
    }
    for edge_index in 0..10 {
        let mut atomic_input = task258b3m2b1_atomic_input(baseline_atomic);
        atomic_input.edges[edge_index].ordinal = 2;
        assert!(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                atomic_input,
                baseline_binding,
                &symbols,
                baseline_primary,
                None,
                None,
                None,
                typed.nodes(),
            )
            .is_err(),
            "Task256 edge row {edge_index} was accepted"
        );
    }
    for request_index in 0..10 {
        let mut atomic_input = task258b3m2b1_atomic_input(baseline_atomic);
        atomic_input.requests[request_index].ordinal = 2;
        assert!(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                atomic_input,
                baseline_binding,
                &symbols,
                baseline_primary,
                None,
                None,
                None,
                typed.nodes(),
            )
            .is_err(),
            "Task256 request row {request_index} was accepted"
        );
    }

    assert_eq!(mizar_checker::source_term::SourcePrimaryTermProducer::build(primary_input, baseline_binding, typed.nodes()).expect("generic primary replay"), generic_primary);
    assert_eq!(mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(atomic_input, baseline_binding, &symbols, baseline_primary, None, None, None, typed.nodes()).expect("generic atomic replay"), generic_atomic);
    assert_eq!((source.clone(), symbols.clone(), scope.clone(), base.clone(), typed.clone(), resolved.clone()), before);
    assert_eq!(task258_native_output(&ast, &module), (typed.clone(), resolved.clone()));
    let (foreign_ast, foreign_module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_511);
    let (foreign_source, foreign_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&foreign_ast, &foreign_module).unwrap().unwrap();
    let (foreign_scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&foreign_source, &foreign_symbols).unwrap();
    for (current, env, occurrences) in [
        (&foreign_source, &symbols, &scope),
        (&source, &foreign_symbols, &scope),
        (&source, &symbols, &foreign_scope),
        (&foreign_source, &foreign_symbols, &foreign_scope),
    ] {
        assert_eq!(base.clone().with_source_statement_from_original(current, env, occurrences), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
        assert_eq!((source.clone(), symbols.clone(), scope.clone(), base.clone(), typed.clone(), resolved.clone()), before);
        assert_eq!(task258_native_output(&ast, &module), (typed.clone(), resolved.clone()));
    }
    let (_, _, foreign_base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&foreign_source, &foreign_symbols, &foreign_scope).unwrap();
    let foreign_before = foreign_base.clone();
    assert_eq!(foreign_base.clone().with_source_statement_from_original(&source, &symbols, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(foreign_base, foreign_before);
    assert_eq!((source, symbols, scope, base, typed.clone(), resolved.clone()), before);
    assert_eq!(task258_native_output(&ast, &module), (typed, resolved));
}

#[test]
fn task258b5a_source_active_fixture_and_visibility_near_misses_stay_isolated() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_520);
    let (native_source, native_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&exact_ast, &exact_module).unwrap().unwrap();
    let (native_scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&native_source, &native_symbols).unwrap();
    let (_, _, native_base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&native_source, &native_symbols, &native_scope).unwrap();
    let native_baseline = task258_native_output(&exact_ast, &exact_module);
    assert!(source_statement_output_with_source(&exact_ast, exact_module.clone(), &exact_symbols, SOURCE_STATEMENT_B5A_TEXT).is_none(), "B5A production dispatch is retired");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    for (ordinal, (relative_path, expected_hash)) in [
        (
            "tests/miz/pass/parser/pass_parser_theorems_proofs_001.miz",
            "89ed43564651e30bfc07d864bce3d33af71e7a5b07f47634ee964b017787f593",
        ),
        (
            "tests/miz/fail/types/fail_type_elaboration_statement_proof_gap_001.miz",
            "49f8fce525fe393e5c3cb0f63e9af7bd0965f12bd74f6202a941f6fa9c2221cb",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let fixture_path = workspace_root.join(relative_path);
        let fixture = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", fixture_path.display()));
        assert_eq!(sha256_text(&fixture), expected_hash, "{relative_path}");
        let (fixture_ast, fixture_module, _, fixture_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&fixture, 258_521 + ordinal);
        assert_eq!(diagnostics, 0, "{relative_path} must parse unchanged");
        assert!(
            extract_ancestor_label_descendant_citation_source_statement(&fixture_ast, &fixture)
                .is_none(),
            "{relative_path} must not satisfy the private exact extractor"
        );
        assert!(
            source_statement_output_with_source(
                &fixture_ast,
                fixture_module,
                &fixture_symbols,
                &fixture,
            )
            .is_none(),
            "{relative_path} must remain isolated from B5A"
        );
    }
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task258B5A isolation plan");
    assert!(plan.cases.iter().any(|case| {
        case.id.0 == "pass_parser_theorems_proofs_001"
            && case
                .source_path
                .ends_with("pass_parser_theorems_proofs_001.miz")
    }));
    assert!(active_type_elaboration_cases(&plan).any(|case| {
        case.id.0 == "fail_type_elaboration_statement_proof_gap_001"
            && case
                .source_path
                .ends_with("fail_type_elaboration_statement_proof_gap_001.miz")
    }));
    let mut selected = Vec::new();
    let mut active_count = 0;
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
    assert_eq!(active_count, 272);
    assert!(
        selected.is_empty(),
        "private B5A route selected active type cases: {selected:?}"
    );
    let near_misses = [
        (
            "missing final LF",
            SOURCE_STATEMENT_B5A_TEXT
                .trim_end_matches('\n')
                .to_owned(),
        ),
        ("extra final LF", format!("{SOURCE_STATEMENT_B5A_TEXT}\n")),
        (
            "theorem label drift",
            SOURCE_STATEMENT_B5A_TEXT.replacen(
                "FormulaStatementAncestorLabelDescendantCitationSmoke",
                "FormulaStatementAncestorLabelDescendantCitationOther",
                1,
            ),
        ),
        (
            "label spelling drift",
            SOURCE_STATEMENT_B5A_TEXT.replacen("  A:", "  B:", 1),
        ),
        (
            "citation spelling drift",
            SOURCE_STATEMENT_B5A_TEXT.replacen("by A", "by B", 1),
        ),
        (
            "qualified citation",
            SOURCE_STATEMENT_B5A_TEXT.replacen("by A", "by Other.A", 1),
        ),
        (
            "citation in first descendant",
            SOURCE_STATEMENT_B5A_TEXT.replacen(
                "    thus x = x;\n",
                "    thus x = x by A;\n",
                1,
            ),
        ),
        (
            "citation returned to ancestor",
            SOURCE_STATEMENT_B5A_TEXT.replacen(
                "  thus x = x proof\n    thus x = x by A;\n  end;",
                "  thus x = x by A;",
                1,
            ),
        ),
        (
            "descendant proof omitted",
            SOURCE_STATEMENT_B5A_TEXT.replacen(
                "  thus x = x proof\n    thus x = x by A;\n  end;",
                "  thus x = x;",
                1,
            ),
        ),
        (
            "sibling blocks reordered",
            SOURCE_STATEMENT_B5A_TEXT.replacen(
                "  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x proof\n    thus x = x by A;\n  end;",
                "  thus x = x proof\n    thus x = x by A;\n  end;\n  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
    ];
    for (ordinal, (label, source)) in near_misses.into_iter().enumerate() {
        assert!(extract_ancestor_label_descendant_citation_source_statement(&exact_ast, &source).is_none(), "{label} exact AST");
        let positive = matches!(label, "missing final LF" | "extra final LF" | "theorem label drift" | "citation returned to ancestor");
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let package_root = std::env::temp_dir().join(format!("mizar-test-b5a-native-selector-{}-{ordinal}-{unique}", std::process::id()));
        let source_path = package_root.join("src").join("selector.miz");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(&source_path, &source).unwrap();
        let package = PackageId::new("mizar-test-b5a-native-selector");
        let module_path = ModulePath::new(format!("tests.b5a_native_selector_{ordinal}"));
        let normalized_path = mizar_session::normalize_path(&package_root, &source_path).unwrap();
        let frontend = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(mizar_session::DiskSourceLoader::new(&package_root)),
            ParseOnlyImportProvider, mizar_frontend::parsing::MizarParserSeam,
        );
        let output = frontend.run(mizar_frontend::source::SourceUnitRequest {
            snapshot: super::shared::snapshot_id(280_000 + ordinal),
            input: mizar_session::SourceInput {
                package_id: package.clone(), module_path: module_path.clone(), normalized_path,
                edition: Edition::new("2026"), origin: mizar_session::SourceOriginInput::Disk { path: source_path },
            },
        }, &mizar_session::InMemorySessionIdAllocator::new()).unwrap();
        std::fs::remove_dir_all(&package_root).unwrap();
        let module = ResolverModuleId::new(package, module_path);
        if let Some(ast) = output.ast {
            assert!(extract_ancestor_label_descendant_citation_source_statement(&ast, &source).is_none(), "{label} parsed AST");
            if positive {
                assert!(output.diagnostics.is_empty(), "{label} coherent native source");
                let (typed, resolved) = task258_native_output(&ast, &module);
                assert_eq!(typed.source_statement().unwrap().statements().len(), if label == "citation returned to ancestor" { 4 } else { 5 });
                let references = typed.source_statement_references().expect("native reference pair");
                assert_eq!(resolved.source_statement_references(), Some(references));
            } else {
                match mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module) {
                    Err(error) => assert_eq!(error, mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module).unwrap_err().to_string(), "{label} source lowering refusal"),
                    Ok(None) => {
                        let shells = mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module).collect();
                        let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(&ast, &shells, mizar_resolve::env::NamespacePath::new(module.path().as_str())).collect(&module);
                        assert!(!collected.diagnostics().is_empty(), "{label} declaration collection refusal");
                    }
                    Ok(Some((original, symbols))) => match mizar_resolve::symbols::source_proof_inputs(&original, &symbols) {
                        Err(error) => assert_eq!(mizar_resolve::symbols::source_proof_inputs(&original, &symbols), Err(error), "{label} resolver refusal replay"),
                        Ok((scope, labels, resolution)) => {
                            mizar_resolve::symbols::validate_source_proof_occurrences(&original, &symbols, &scope).unwrap();
                            if matches!(label, "label spelling drift" | "citation spelling drift") {
                                assert!(resolution.has_unresolved(), "{label} actual unknown-label table");
                                assert!(!resolution.table().is_empty());
                                assert!(resolution.diagnostics().is_empty(), "unknown label is not a conflict diagnostic");
                            }
                            let before = (original.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone());
                            assert!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&original, &symbols, &scope).is_err(), "{label} bounded native refusal");
                            assert_eq!((original, symbols, scope, labels, resolution), before);
                        }
                    },
                }
            }
        } else {
            assert!(!positive, "{label} positive requires an AST");
            assert!(!output.diagnostics.is_empty(), "{label} Frontend owns absent AST");
        }
        assert_eq!(task258_native_output(&exact_ast, &exact_module), native_baseline, "{label} pristine native retry");
    }
    let enriched = source_statement_b5a_resolver_env_for_test(
        &exact_ast,
        &exact_module,
        &exact_symbols,
        SOURCE_STATEMENT_B5A_TEXT,
    )
    .expect("B5A enriched resolver");
    let env_before = (exact_symbols.clone(), enriched.clone());
    assert!(validate_task258b5a_raw_resolver_env(&exact_ast, &exact_module, &exact_symbols).is_ok());
    assert!(validate_task258b5a_enriched_resolver_env(&exact_ast, &exact_module, &enriched).is_ok());
    assert!(validate_task258b5a_raw_resolver_env(&exact_ast, &exact_module, &enriched).is_err(), "pre-enriched input must fail raw authentication");
    assert!(mizar_resolve::symbols::validate_source_symbol_env(&native_source, &enriched).is_err());
    assert_eq!(native_base.clone().with_source_statement_from_original(&native_source, &enriched, &native_scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(task258_native_output(&exact_ast, &exact_module), native_baseline);
    for mutation in [
        Task258B5ARawResolverMutation::OwnerOriginSource,
        Task258B5ARawResolverMutation::OwnerOriginModule,
        Task258B5ARawResolverMutation::OwnerOriginRange,
        Task258B5ARawResolverMutation::OwnerOriginPath,
        Task258B5ARawResolverMutation::OwnerKind,
        Task258B5ARawResolverMutation::OwnerVisibility,
        Task258B5ARawResolverMutation::OwnerExport,
        Task258B5ARawResolverMutation::OwnerSpelling,
        Task258B5ARawResolverMutation::ContributionModule,
        Task258B5ARawResolverMutation::ContributionKind,
        Task258B5ARawResolverMutation::ContributionSource,
        Task258B5ARawResolverMutation::ContributionAnchor,
        Task258B5ARawResolverMutation::ContributionSymbolEffects,
        Task258B5ARawResolverMutation::ContributionDefinitionEffects,
    ] {
        let run = || {
            let mutated = task258b5a_mutate_raw_resolver(exact_symbols.clone(), mutation);
            validate_task258b5a_raw_resolver_env(&exact_ast, &exact_module, &mutated).expect_err("B5A raw resolver mutation")
        };
        assert_eq!(run(), run(), "{mutation:?} raw replay");
        let mutated = task258b5a_mutate_raw_resolver(native_symbols.clone(), mutation);
        assert_ne!(mutated, native_symbols);
        assert!(mizar_resolve::symbols::validate_source_symbol_env(&native_source, &mutated).is_err());
        assert_eq!(native_base.clone().with_source_statement_from_original(&native_source, &mutated, &native_scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
        assert_eq!((exact_symbols.clone(), enriched.clone()), env_before);
        assert_eq!(task258_native_output(&exact_ast, &exact_module), native_baseline);
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
            let mutated = task258b2_mutate_resolver(enriched.clone(), mutation);
            validate_task258b5a_enriched_resolver_env(&exact_ast, &exact_module, &mutated).expect_err("B5A resolver mutation")
        };
        assert_eq!(run(), run(), "{mutation:?} resolver replay");
        assert_eq!((exact_symbols.clone(), enriched.clone()), env_before);
        assert!(validate_task258b5a_raw_resolver_env(&exact_ast, &exact_module, &exact_symbols).is_ok());
        assert!(validate_task258b5a_enriched_resolver_env(&exact_ast, &exact_module, &enriched).is_ok());
        assert_eq!(task258_native_output(&exact_ast, &exact_module), native_baseline);
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
            let mutated = task258b3m2b2b3a_mutate_label_resolver(enriched.clone(), mutation);
            validate_task258b5a_enriched_resolver_env(&exact_ast, &exact_module, &mutated).expect_err("B5A enriched label mutation")
        };
        assert_eq!(run(), run(), "{mutation:?} label replay");
        assert_eq!((exact_symbols.clone(), enriched.clone()), env_before);
        assert!(validate_task258b5a_raw_resolver_env(&exact_ast, &exact_module, &exact_symbols).is_ok());
        assert!(validate_task258b5a_enriched_resolver_env(&exact_ast, &exact_module, &enriched).is_ok());
        assert_eq!(task258_native_output(&exact_ast, &exact_module), native_baseline);
    }
    let (b1_ast, b1_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_522);
    let (b1, _) = task258_native_output(&b1_ast, &b1_module);
    assert_eq!(b1.source_term().unwrap().references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), (0..8).collect::<Vec<_>>());
    assert!(
        source_statement_output_with_source(
            &b1_ast,
            exact_module,
            &exact_symbols,
            SOURCE_STATEMENT_B5A_TEXT,
        )
        .is_none()
    );
    assert_eq!(
        sha256_text(TASK258A_SOURCE),
        sha256_text(SOURCE_STATEMENT_TEXT)
    );
}

#[test]
fn task258b5a_b1_profile_installation_orders_and_ownership_are_atomic() {
    let (b1_ast, b1_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_530);
    let (b1, _) = task258_native_output(&b1_ast, &b1_module);
    let (b5a_ast, b5a_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_530);
    let (b5a, _) = task258_native_output(&b5a_ast, &b5a_module);
    assert_eq!(b1.source_id(), b5a.source_id());
    assert_eq!(b1.module_id(), b5a.module_id());
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
    assert_eq!(
        task258_reference_syntax_base(&b1)
            .with_source_statement_references(b1_statement.clone(), b1_references.clone()),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        task258_reference_syntax_base(&b5a)
            .with_source_statement_references(b5a_statement.clone(), b5a_references.clone()),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        task258_reference_syntax_base(&b1)
            .with_source_statement_references(b5a_statement.clone(), b5a_references.clone())
            .expect_err("B5A pair on B1 base"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task258_reference_syntax_base(&b5a)
            .with_source_statement_references(b1_statement.clone(), b1_references.clone())
            .expect_err("B1 pair on B5A base"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task258_reference_syntax_base(&b1)
            .with_source_statement_references(b1_statement.clone(), b5a_references.clone())
            .expect_err("B1 statement with B5A references"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task258_reference_syntax_base(&b5a)
            .with_source_statement_references(b5a_statement.clone(), b1_references.clone())
            .expect_err("B5A statement with B1 references"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task258_reference_syntax_base(&b5a)
            .with_source_statement(b5a_statement.clone())
            .expect_err("partial B5A base"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    let installed = b5a.clone();
    let installed_debug = installed.debug_text();
    assert_eq!(
        installed
            .clone()
            .with_source_statement_references(b5a_statement.clone(), b5a_references.clone())
            .expect_err("duplicate B5A pair"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(installed.debug_text(), installed_debug);
    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement_references(b1_statement.clone(), b1_references.clone())
            .expect_err("Task248/B1 ownership"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement_references(b5a_statement.clone(), b5a_references.clone())
            .expect_err("Task248/B5A ownership"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
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
}

#[test]
fn task258b5a_final_clone_debug_replay_and_empty_semantics_are_stable() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_540);
    let (first, first_resolved) = task258_native_output(&ast, &module);
    let (second, second_resolved) = task258_native_output(&ast, &module);
    assert_eq!(first, first.clone());
    assert_eq!(first_resolved, first_resolved.clone());
    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first_resolved.debug_text(), second_resolved.debug_text());
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap();
    assert_eq!(first.source_term().unwrap().references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), scope.references().iter().map(|reference| reference.ordinal()).collect::<Vec<_>>());
    let before = (source.clone(), symbols.clone(), scope.clone(), base.clone(), first.clone(), first_resolved.clone());
    assert!(first.source_context().is_none());
    assert!(first.source_type().is_none());
    assert!(first.source_attribute().is_none());
    assert!(first.source_evidence().is_none());
    assert!(first.source_application().is_none());
    assert!(first.source_structure().is_none());
    assert!(first.source_set_term().is_none());
    assert!(first.source_statement_witnesses().is_none());
    assert!(first.source_composite_formula().is_none());
    assert!(first.source_formula_composition().is_none());
    assert!(first.facts().is_empty());
    assert!(first.diagnostics().is_empty());
    assert!(first_resolved.expr_metadata().is_empty());
    assert!(first_resolved.collection_candidates().is_empty());
    assert!(first_resolved.expanded_candidates().is_empty());
    assert!(first_resolved.template_expansions().is_empty());
    assert!(first_resolved.viable_candidates().is_empty());
    assert!(first_resolved.viability_decisions().is_empty());
    assert!(first_resolved.specificity_graphs().is_empty());
    assert!(first_resolved.resolved_overloads().is_empty());
    assert!(first_resolved.inserted_coercions().is_empty());
    assert!(first_resolved.cluster_facts().is_empty());
    assert!(first_resolved.diagnostics().is_empty());
    assert!(first_resolved.checked_formulas().is_empty());
    assert!(first_resolved.statement_semantics().is_empty());
    assert!(first_resolved.checked_proofs().is_empty());
    assert!(first_resolved.checked_proof_nodes().is_empty());
    assert!(first_resolved.checked_terminal_goals().is_empty());
    let (b1_ast, b1_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_541);
    let (b1_first, _) = task258_native_output(&b1_ast, &b1_module);
    assert_eq!(b1_first.source_term().unwrap().references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), (0..8).collect::<Vec<_>>());
    let (b1_source, b1_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&b1_ast, &b1_module).unwrap().unwrap();
    let (b1_scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&b1_source, &b1_symbols).unwrap();
    let (_, _, b1_base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&b1_source, &b1_symbols, &b1_scope).unwrap();
    let b1_before = b1_base.clone();
    assert_eq!(b1_base.clone().with_source_statement_from_original(&source, &symbols, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(base.clone().with_source_statement_from_original(&b1_source, &b1_symbols, &b1_scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(b1_base, b1_before);
    assert_eq!((source, symbols, scope, base, first.clone(), first_resolved.clone()), before);
    assert_eq!(task258_native_output(&ast, &module), (first.clone(), first_resolved.clone()));
    assert_eq!(task258_native_output(&b1_ast, &b1_module).0, b1_first);
    assert!(
        !b1_first
            .debug_text()
            .contains("AncestorLabelDescendant")
    );
}
