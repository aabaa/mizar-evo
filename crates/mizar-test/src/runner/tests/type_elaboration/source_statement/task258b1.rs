#[test]
fn task258b1_real_frontend_freezes_nested_statement_and_resolver_bundle() {
    assert_eq!(SOURCE_STATEMENT_B1_TEXT.len(), 139);
    assert!(SOURCE_STATEMENT_B1_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) = task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_400);
    let extracted: SourceStatementB1Extraction =
        extract_nested_source_statement(&ast, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 exact parser shape");
    assert_eq!(ast.nodes().len(), 77);
    assert_eq!(ast.root().expect("root").index(), 76);
    assert_eq!(extracted.theorem_site.node().index(), 73);
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [73, 60, 58, 71]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 138), (77, 114), (96, 107), (117, 133)]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(63, 68), (80, 85), (101, 106), (122, 127)]
    );
    assert_eq!(
        (extracted.theorem_range.start, extracted.theorem_range.end),
        (19, 138)
    );
    assert_eq!(
        (extracted.label_range.start, extracted.label_range.end),
        (27, 61)
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [42, 48, 55, 65]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [38, 40, 44, 46, 51, 53, 61, 63]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (63, 64),
            (67, 68),
            (80, 81),
            (84, 85),
            (101, 102),
            (105, 106),
            (122, 123),
            (126, 127),
        ]
    );
    assert_eq!(
        extracted.proof_ranges.map(|range| (range.start, range.end)),
        [(69, 137), (86, 113)]
    );
    let mut owners = symbols
        .symbols()
        .iter()
        .filter(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem);
    let owner = owners.next().expect("B1 theorem symbol");
    assert!(owners.next().is_none(), "B1 has one theorem owner");
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let lowered = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("B1 source arena");
    let labels = mizar_resolve::labels::ProofLabelSourceCollector::new(
        &ast,
        &module,
        namespace.clone(),
        owner.contribution(),
        &lowered,
    )
    .expect("B1 label collector")
    .collect()
    .expect("B1 source labels");
    let [projection] = labels.projections() else {
        panic!("B1 has one source-collected proof label");
    };
    let [reference] = labels.references() else {
        panic!("B1 has one source-collected citation");
    };
    assert_eq!(projection.primary_spelling(), "A");
    assert_eq!(
        projection.kind(),
        mizar_resolve::resolved_ast::LabelKind::ProofStep
    );
    assert_eq!(projection.module(), &module);
    assert_eq!(projection.namespace(), &namespace);
    assert_eq!(projection.contribution(), owner.contribution());
    assert_eq!(
        (
            projection.declaration_range().start,
            projection.declaration_range().end
        ),
        (77, 78)
    );
    assert_eq!(projection.origin().source_id(), ast.source_id);
    assert_eq!(projection.origin().module_id(), &module);
    assert_eq!(
        projection.origin().anchor(),
        &mizar_session::SourceAnchor::Range(projection.declaration_range())
    );
    assert_eq!(projection.origin().structural_path(), [73, 60, 12]);
    assert!(
        projection
            .origin_path()
            .as_str()
            .starts_with("proof-step-v1|")
    );
    assert!(matches!(
        projection.source(),
        mizar_resolve::labels::LabelProjectionSource::CurrentModule {
            visible_after_ordinal: 2,
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(reference.site().spelling(), "A");
    assert_eq!(reference.site().node().index(), 68);
    assert_eq!(
        (reference.site().range().start, reference.site().range().end),
        (131, 132)
    );
    assert_eq!(reference.origin().structural_path(), [73, 71, 68]);
    assert_eq!(reference.origin().source_id(), ast.source_id);
    assert_eq!(reference.origin().module_id(), &module);
    assert_eq!(
        reference.origin().anchor(),
        &mizar_session::SourceAnchor::Range(reference.site().range())
    );
    assert_eq!(reference.ordinal(), 3);
    assert!(matches!(
        reference.scope(),
        mizar_resolve::labels::LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    let resolution = mizar_resolve::labels::LabelResolver::new(labels.projections()).resolve(
        &module,
        &namespace,
        labels.references(),
    );
    assert!(resolution.diagnostics().is_empty());
    assert_eq!(resolution.ids().len(), 1);
    let entry = resolution.table().get(resolution.ids()[0]).unwrap();
    let mizar_resolve::resolved_ast::LabelResolution::Resolved(target) = entry.resolution() else {
        panic!("B1 post-proof citation resolves");
    };
    assert_eq!(target.origin(), projection.origin_path());
    let (native_source, native_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
        .expect("original B1 source collection").expect("original B1 theorem module");
    let (native_scope, native_labels, native_resolution) = mizar_resolve::symbols::source_proof_inputs(&native_source, &native_symbols)
        .expect("complete original B1 resolver inputs");
    assert_eq!(native_labels.projections().len(), 2);
    let owner_projection = native_labels.projections().iter().find(|projection| projection.kind() == mizar_resolve::resolved_ast::LabelKind::Theorem).expect("original theorem projection");
    assert!(matches!(owner_projection.source(), mizar_resolve::labels::LabelProjectionSource::CurrentModule { visible_after_ordinal: 3, proof_scope: None }));
    let native_projection = native_labels.projections().iter().find(|projection| projection.kind() == mizar_resolve::resolved_ast::LabelKind::ProofStep).expect("original proof projection");
    assert_eq!(native_projection, projection);
    assert_eq!(native_resolution.index().len(), 2);
    let (typed, resolved) = task258_native_output(&ast, &module);
    let statement = typed
        .source_statement()
        .expect("Task258B1 statement");
    let references = typed
        .source_statement_references()
        .expect("Task258B1 references");
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
        (1, 4, 4, 4, 4, 1, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("theorem owner");
    assert_eq!(owner.site().node().index(), 73);
    assert_eq!(
        (owner.source_range().start, owner.source_range().end),
        (19, 138)
    );
    assert_eq!(owner.spelling(), "FormulaStatementNestedContextSmoke");
    assert_eq!(
        (owner.role(), owner.status(), owner.recovery(),),
        (
            mizar_checker::source_statement::SourceTheoremRole::Theorem,
            mizar_checker::source_statement::SourceTheoremStatus::Unmodified,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.owner().index(),
                row.context().index(),
                row.formula(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
                row.recovery(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
                ),
                73,
                19,
                138,
                0,
                "theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;",
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                1,
                0,
                1,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1),
                ),
                60,
                77,
                114,
                1,
                "A : x = x proof thus x = x ; end ;",
                mizar_checker::source_statement::SourceStatementKind::ProofStepProposition,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                2,
                0,
                2,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                ),
                58,
                96,
                107,
                2,
                "thus x = x ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                3,
                0,
                3,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(3),
                ),
                71,
                117,
                133,
                3,
                "thus x = x by A ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
        ]
    );
    assert_eq!(
        statement
            .contexts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.binding_context().index(),
                row.source_range().start,
                row.source_range().end,
                row.visible_bindings()
                    .iter()
                    .map(|binding| binding.index())
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, 19, 138, vec![0]),
            (1, 1, 1, 77, 114, vec![0]),
            (2, 2, 2, 96, 107, vec![0]),
            (3, 3, 1, 117, 133, vec![0]),
        ]
    );
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.context().index(),
                row.ordinal(),
                row.kind(),
                row.binding().index(),
                row.uses()
                    .iter()
                    .map(|reference| reference.index())
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![0, 1],
            ),
            (
                1,
                1,
                1,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![2, 3],
            ),
            (
                2,
                2,
                2,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![4, 5],
            ),
            (
                3,
                3,
                3,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![6, 7],
            ),
        ]
    );
    assert_eq!(
        statement
            .candidate_facts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.context().index(),
                row.ordinal(),
                row.kind(),
                row.formula(),
            ))
            .collect::<Vec<_>>(),
        (0..4)
            .map(|index| (
                index,
                index,
                index,
                0,
                mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
                ),
            ))
            .collect::<Vec<_>>()
    );
    let binding_contexts = statement.binding_env().contexts();
    assert_eq!(binding_contexts.len(), 3);
    let module_context = binding_contexts
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("module context");
    assert_eq!(
        module_context.owner,
        mizar_checker::binding_env::BindingContextOwner::Module
    );
    assert_eq!(module_context.parent, None);
    assert_eq!(
        module_context.layer,
        mizar_checker::binding_env::BindingContextLayer::Module
    );
    assert_eq!(
        module_context.bindings,
        [mizar_checker::binding_env::BindingId::new(0)]
    );
    assert_eq!(
        module_context.visible_bindings,
        [mizar_checker::binding_env::BindingId::new(0)]
    );
    assert_eq!(
        module_context.recovery,
        mizar_checker::binding_env::BindingContextRecovery::Normal
    );
    for (id, parent, scope, range) in [(1, 0, &[4][..], (69, 137)), (2, 1, &[4, 1][..], (86, 113))]
    {
        let context = binding_contexts
            .get(mizar_checker::binding_env::BindingContextId::new(id))
            .expect("proof context");
        assert_eq!(context.parent.map(|parent| parent.index()), Some(parent));
        assert_eq!(
            context.layer,
            mizar_checker::binding_env::BindingContextLayer::Proof
        );
        assert_eq!(
            context.lexical_scope.as_ref().expect("proof scope").path(),
            scope
        );
        assert_eq!(
            context.owner,
            mizar_checker::binding_env::BindingContextOwner::SourceStatement {
                source_range: mizar_session::SourceRange {
                    source_id: ast.source_id,
                    start: range.0,
                    end: range.1,
                },
            }
        );
        assert_eq!(
            context.visible_bindings,
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        assert!(context.bindings.is_empty());
        assert_eq!(
            context.recovery,
            mizar_checker::binding_env::BindingContextRecovery::Normal
        );
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
    assert_eq!((ast.nodes()[35].range.start, ast.nodes()[35].range.end), (14, 17));
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(
        statement.binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    let primary = typed.source_term().expect("Task252 terms");
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, term)| (
                id.index(),
                term.site().node().index(),
                term.context().index(),
                term.source_range().start,
                term.source_range().end,
                term.source_ordinal(),
                term.recovery(),
                term.spelling(),
                term.kind(),
                term.role(),
                term.parent().map(|parent| parent.index()),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                38,
                0,
                63,
                64,
                0,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
            (
                1,
                40,
                0,
                67,
                68,
                1,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
            (
                2,
                44,
                1,
                80,
                81,
                2,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
            (
                3,
                46,
                1,
                84,
                85,
                3,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
            (
                4,
                51,
                2,
                101,
                102,
                4,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
            (
                5,
                53,
                2,
                105,
                106,
                5,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
            (
                6,
                61,
                1,
                122,
                123,
                6,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
            (
                7,
                63,
                1,
                126,
                127,
                7,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None
            ),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(id, reference)| (
                id.index(),
                reference.term().index(),
                reference.binding().index(),
                reference.lexical_scope().map(|scope| scope.path().to_vec()),
                reference.use_ordinal(),
                reference.role(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                Some(vec![]),
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
            (
                1,
                1,
                0,
                Some(vec![]),
                1,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
            (
                2,
                2,
                0,
                Some(vec![4]),
                2,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
            (
                3,
                3,
                0,
                Some(vec![4]),
                3,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
            (
                4,
                4,
                0,
                Some(vec![4, 1]),
                4,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
            (
                5,
                5,
                0,
                Some(vec![4, 1]),
                5,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
            (
                6,
                6,
                0,
                Some(vec![4]),
                6,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
            (
                7,
                7,
                0,
                Some(vec![4]),
                7,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
            ),
        ]
    );
    let atomic = typed
        .source_atomic_formula()
        .expect("Task256 atomics");
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len()
        ),
        (4, 8, 8)
    );
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(id, formula)| (
                id.index(),
                formula.site().node().index(),
                formula.context().index(),
                formula.source_range().start,
                formula.source_range().end,
                formula.source_ordinal(),
                formula.recovery(),
                formula.spelling(),
                formula.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                42,
                0,
                63,
                68,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
                "x = x",
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
            ),
            (
                1,
                48,
                1,
                80,
                85,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
                "x = x",
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
            ),
            (
                2,
                55,
                2,
                101,
                106,
                2,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
                "x = x",
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
            ),
            (
                3,
                65,
                1,
                122,
                127,
                3,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
                "x = x",
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
            ),
        ]
    );
    for (id, edge) in atomic.edges().iter() {
        assert_eq!(edge.formula().index(), id.index() / 2);
        assert_eq!(edge.ordinal(), id.index() % 2);
        assert_eq!(
            edge.role(),
            if id.index() % 2 == 0 {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
        assert!(matches!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term)
                if term.index() == id.index()
        ));
    }
    for (id, request) in atomic.requests().iter() {
        assert_eq!(request.formula().index(), id.index() / 2);
        assert_eq!(request.ordinal(), id.index() % 2);
        assert_eq!(
            request.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
        );
        assert_eq!(request.edge().map(|edge| edge.index()), Some(id.index()));
        assert_eq!(request.candidate(), None);
        assert_eq!(request.type_site(), None);
        assert_eq!(request.attribute(), None);
    }
    assert_eq!(primary.references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), (0..8).collect::<Vec<_>>());
    assert_eq!(primary.references().iter().map(|(_, reference)| (reference.lexical_scope().unwrap(), reference.use_ordinal(), reference.binding().index())).collect::<Vec<_>>(), native_scope.references().iter().map(|reference| (reference.scope(), reference.ordinal(), reference.binding().index())).collect::<Vec<_>>());
    assert_eq!(typed.resolved_root(), Some(native_source.arena().root()));
    assert_eq!(typed.nodes().len(), 77);
    assert_eq!(
        typed.nodes().root().map(|root| root.index()),
        Some(76)
    );
    for (id, node) in typed.nodes().iter() {
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
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
    }
    assert_eq!(references.resolver_ast().nodes().len(), 77);
    assert_eq!(references.resolver_ast().nodes().root().index(), 76);
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
        assert_eq!(node.origin().source_id(), ast.source_id);
        assert_eq!(node.origin().module_id(), typed.module_id());
        assert_eq!(node.origin().import_edge(), None);
        assert_eq!(node.origin().structural_path(), [id.index() as u32]);
        assert_eq!(
            node.origin().anchor(),
            &mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range)
        );
        assert!(!node.origin().is_recovered());
        assert_eq!(
            node.recovery(),
            mizar_resolve::resolved_ast::RecoveryState::Normal
        );
        if id.index() == 68 {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::Resolved
            );
            assert!(matches!(
                node.reference_key(),
                Some(mizar_resolve::resolved_ast::NodeReferenceKey::Label(id))
                    if id.index() == 0
            ));
        } else {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::NotApplicable
            );
            assert_eq!(node.reference_key(), None);
        }
    }
    let projection = references.label_projection();
    assert_eq!(projection.origin().structural_path(), [73, 60, 12]);
    assert_eq!(projection, native_projection);
    assert_eq!(references.label_resolution(), &native_resolution);
    assert_eq!(references.reference_candidate(), &native_labels.references()[0]);
    assert_eq!(
        (
            projection.declaration_range().start,
            projection.declaration_range().end
        ),
        (77, 78)
    );
    assert_eq!(projection.primary_spelling(), "A");
    assert_eq!(projection.module(), typed.module_id());
    assert_eq!(
        projection.namespace().as_str(),
        typed.module_id().path().as_str()
    );
    assert_eq!(
        projection.kind(),
        mizar_resolve::resolved_ast::LabelKind::ProofStep
    );
    assert_eq!(
        projection.visibility(),
        mizar_resolve::env::Visibility::Private
    );
    assert_eq!(
        projection.export_status(),
        mizar_resolve::env::ExportStatus::LocalOnly
    );
    assert_eq!(projection.contribution(), owner.contribution());
    assert_eq!(projection.origin().source_id(), ast.source_id);
    assert_eq!(
        projection.origin().module_id(),
        typed.module_id()
    );
    assert_eq!(projection.origin().import_edge(), None);
    assert_eq!(
        projection.origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 77,
            end: 78,
        })
    );
    assert!(!projection.origin().is_recovered());
    assert!(matches!(
        projection.source(),
        mizar_resolve::labels::LabelProjectionSource::CurrentModule {
            visible_after_ordinal: 2,
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(references.reference_candidate().site().node().index(), 68);
    assert_eq!(
        (
            references.reference_candidate().site().range().start,
            references.reference_candidate().site().range().end,
            references.reference_candidate().site().spelling(),
            references.reference_candidate().ordinal(),
            references.reference_candidate().expectation(),
        ),
        (
            131,
            132,
            "A",
            3,
            mizar_resolve::resolved_ast::LabelExpectation::ProofOrTheorem,
        )
    );
    assert_eq!(
        references.reference_candidate().origin().source_id(),
        ast.source_id
    );
    assert_eq!(
        references.reference_candidate().origin().module_id(),
        typed.module_id()
    );
    assert_eq!(
        references.reference_candidate().origin().import_edge(),
        None
    );
    assert_eq!(
        references.reference_candidate().origin().structural_path(),
        [73, 71, 68]
    );
    assert_eq!(
        references.reference_candidate().origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 131,
            end: 132,
        })
    );
    assert!(!references.reference_candidate().origin().is_recovered());
    assert!(matches!(
        references.reference_candidate().scope(),
        mizar_resolve::labels::LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(references.label_resolution().ids().len(), 1);
    assert!(references.label_resolution().diagnostics().is_empty());
    assert_eq!(
        references.resolver_ast().label_refs(),
        references.label_resolution().table()
    );
    let label_ref = references
        .label_resolution()
        .table()
        .get(references.label_resolution().ids()[0])
        .expect("resolved local label");
    assert_eq!(
        (
            label_ref.site().node().index(),
            label_ref.site().range().start,
            label_ref.site().range().end,
            label_ref.site().spelling(),
            label_ref.origin().source_id(),
            label_ref.origin().structural_path(),
            label_ref.recovery(),
        ),
        (
            68,
            131,
            132,
            "A",
            ast.source_id,
            &[73, 71, 68][..],
            mizar_resolve::resolved_ast::RecoveryState::Normal,
        )
    );
    assert_eq!(label_ref.origin().module_id(), typed.module_id());
    assert_eq!(label_ref.origin().import_edge(), None);
    assert!(!label_ref.origin().is_recovered());
    assert!(matches!(
        label_ref.resolution(),
        mizar_resolve::resolved_ast::LabelResolution::Resolved(row)
            if row.origin() == projection.origin_path()
                && row.kind() == mizar_resolve::resolved_ast::LabelKind::ProofStep
                && (row.range().start, row.range().end) == (131, 132)
    ));
    let label = references
        .labels()
        .get(mizar_checker::source_statement::SourceStatementLabelId::new(0))
        .expect("statement label");
    assert_eq!(
        (
            label.statement().index(),
            label.context().index(),
            label.candidate().index(),
            label.origin_path(),
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
            projection.origin_path(),
            &[0][..],
            77,
            78,
            0,
            2,
            "A",
            mizar_checker::source_statement::SourceStatementLabelKind::ProofStep,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    let citation = references
        .citations()
        .get(mizar_checker::source_statement::SourceStatementCitationId::new(0))
        .expect("statement citation");
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
            3,
            3,
            mizar_checker::source_statement::SourceStatementCitationTarget::Local(
                mizar_checker::source_statement::SourceStatementLabelId::new(0),
            ),
            0,
            &[0][..],
            131,
            132,
            0,
            mizar_checker::source_statement::SourceStatementCitationKind::SimpleLocal,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        typed.source_statement_references(),
        resolved.source_statement_references()
    );
    assert!(
        references
            .debug_text()
            .contains("resolver-ast root=76 nodes=77 name_refs=0 label_refs=1")
    );
}

#[test]
fn task258b1_lower_resolver_and_row_mutations_fail_closed_then_replay() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_410);
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
        .expect("original B1 collection").expect("B1 theorem module");
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols)
        .expect("original B1 occurrences");
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
        .expect("real B1 lower base");
    let baseline = task258_native_output(&ast, &module);
    let before = (ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), base.clone());
    let (foreign_ast, foreign_module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_411);
    let (foreign_source, foreign_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&foreign_ast, &foreign_module)
        .expect("foreign source collection").expect("foreign theorem module");
    let (foreign_scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&foreign_source, &foreign_symbols)
        .expect("foreign genuine occurrences");
    assert_ne!(source, foreign_source);
    assert_ne!(symbols, foreign_symbols);
    assert_ne!(scope, foreign_scope);
    for (current, env, occurrences) in [
        (&foreign_source, &symbols, &scope),
        (&source, &foreign_symbols, &scope),
        (&source, &symbols, &foreign_scope),
        (&foreign_source, &foreign_symbols, &foreign_scope),
    ] {
        assert_eq!(base.clone().with_source_statement_from_original(current, env, occurrences),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
        assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), base.clone()), before);
        assert_eq!(task258_native_output(&ast, &module), baseline);
    }
    let (_, _, foreign_base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&foreign_source, &foreign_symbols, &foreign_scope)
        .expect("foreign whole base");
    let foreign_before = foreign_base.clone();
    assert_eq!(foreign_base.clone().with_source_statement_from_original(&source, &symbols, &scope),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(foreign_base, foreign_before);
    assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), base.clone()), before);
    assert_eq!(task258_native_output(&ast, &module), baseline);
}

#[test]
fn task258b1_selector_rejects_exact_nested_statement_near_misses() {
    let (exact_ast, exact_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_420);
    let baseline = task258_native_output(&exact_ast, &exact_module);
    let loaded_cases = [
        (
            "missing final LF",
            SOURCE_STATEMENT_B1_TEXT.trim_end_matches('\n').to_owned(),
        ),
        ("extra final LF", format!("{SOURCE_STATEMENT_B1_TEXT}\n")),
        (
            "whitespace byte drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("  A:", " A:", 1),
        ),
        (
            "comment byte drift",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "reserve x for set;",
                "reserve x for set; :: drift",
                1,
            ),
        ),
        (
            "theorem name drift",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "FormulaStatementNestedContextSmoke",
                "FormulaStatementNestedContextOther",
                1,
            ),
        ),
        (
            "theorem role change",
            SOURCE_STATEMENT_B1_TEXT.replacen("theorem ", "scheme ", 1),
        ),
        (
            "theorem status change",
            SOURCE_STATEMENT_B1_TEXT.replacen("theorem ", "canceled theorem ", 1),
        ),
        (
            "reserve drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set", "reserve y for set", 1),
        ),
        (
            "omitted reserve item",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set;\n", "", 1),
        ),
        (
            "reordered top-level items",
            format!(
                "{}{}",
                &SOURCE_STATEMENT_B1_TEXT[19..],
                &SOURCE_STATEMENT_B1_TEXT[..19]
            ),
        ),
        (
            "omitted nested statement",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;\n", "", 1),
        ),
        (
            "reordered proof statements",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;",
                "  thus x = x by A;\n  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "omitted outer proof block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                " proof\n  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;\nend;",
                ";",
                1,
            ),
        ),
        (
            "reordered proof blocks",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;",
                "  thus x = x by A;\n  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "then statement",
            SOURCE_STATEMENT_B1_TEXT.replacen("  thus x = x by A;", "  then thus x = x by A;", 1),
        ),
        (
            "given statement",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  given y being set such that x = x;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "consider statement",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  consider y being set such that x = x;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "now block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  now\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "hereby block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  hereby\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "case block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  per cases;\n  suppose x = x;\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "suppose block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  suppose x = x;\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "iterative equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x = x;", 1),
        ),
        (
            "imported label citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by Other.A", 1),
        ),
        (
            "local label shadowing",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "    thus x = x;",
                "    A: x = x;\n    thus x = x;",
                1,
            ),
        ),
        (
            "missing nested proof",
            SOURCE_STATEMENT_B1_TEXT.replacen("  A: x = x proof", "  A: x = x", 1),
        ),
        (
            "recovered missing semicolon",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;", "    thus x = x", 1),
        ),
    ];
    let parsed_cases = [
        (
            "forward citation",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  x = x by A;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "theorem citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by FormulaStatementNestedContextSmoke", 1),
        ),
        (
            "hence",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x by A", "hence x = x by A", 1),
        ),
        (
            "parenthesized equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus (x = x);", 1),
        ),
        (
            "second label",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "B: thus x = x;", 1),
        ),
        (
            "extra theorem",
            format!("{SOURCE_STATEMENT_B1_TEXT}theorem Extra: x = x;\n"),
        ),
        (
            "assumption",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  A: x = x proof\n    assume x = x;",
                1,
            ),
        ),
        (
            "witness",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;", "    take x;\n    thus x = x;", 1),
        ),
        (
            "non-equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x <> x;", 1),
        ),
        (
            "composite formula",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x & x = x;", 1),
        ),
        (
            "qualified citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by Other.A", 1),
        ),
        (
            "citation moved inward",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;", "    thus x = x by A;", 1),
        ),
        (
            "explicit justification",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x by A;", 1),
        ),
    ];
    for (ordinal, (label, source, mixed_text)) in loaded_cases.into_iter()
        .map(|(label, source)| (label, source, true))
        .chain(parsed_cases.into_iter().map(|(label, source)| (label, source, false)))
        .enumerate()
    {
        if mixed_text {
            assert!(extract_nested_source_statement(&exact_ast, &source).is_none(), "{label} historical mixed source guard");
        }
        let lexical_positive = matches!(label, "missing final LF" | "extra final LF" | "whitespace byte drift" | "comment byte drift" | "theorem name drift");
        let proofless_positive = label == "omitted outer proof block";
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .expect("native selector clock").as_nanos();
        let package_root = std::env::temp_dir().join(format!("mizar-test-b1-native-selector-{}-{ordinal}-{unique}", std::process::id()));
        let source_path = package_root.join("src").join("selector.miz");
        std::fs::create_dir_all(source_path.parent().expect("source parent")).expect("create native selector package");
        std::fs::write(&source_path, &source).expect("write native selector source");
        let package = PackageId::new("mizar-test-b1-native-selector");
        let module_path = ModulePath::new(format!("tests.b1_native_selector_{ordinal}"));
        let normalized_path = mizar_session::normalize_path(&package_root, &source_path).expect("normalize native selector path");
        let frontend = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(mizar_session::DiskSourceLoader::new(&package_root)),
            ParseOnlyImportProvider,
            mizar_frontend::parsing::MizarParserSeam,
        );
        let output = frontend.run(mizar_frontend::source::SourceUnitRequest {
            snapshot: super::shared::snapshot_id(270_000 + ordinal),
            input: mizar_session::SourceInput {
                package_id: package.clone(), module_path: module_path.clone(), normalized_path,
                edition: Edition::new("2026"),
                origin: mizar_session::SourceOriginInput::Disk { path: source_path },
            },
        }, &mizar_session::InMemorySessionIdAllocator::new()).expect("native selector frontend");
        std::fs::remove_dir_all(&package_root).expect("clean native selector package");
        let diagnostics = output.diagnostics.len();
        let module = ResolverModuleId::new(package, module_path);
        if let Some(ast) = output.ast {
        if !mixed_text {
            assert!(extract_nested_source_statement(&ast, &source).is_none(), "{label}");
            assert!(extract_nested_source_statement(&ast, SOURCE_STATEMENT_B1_TEXT).is_none(), "{label} exact guard with wrong AST");
        }
        if lexical_positive {
            assert_eq!(diagnostics, 0, "{label} coherent native capture");
            let (typed, resolved) = task258_native_output(&ast, &module);
            assert_eq!(typed.source_statement().unwrap().statements().len(), 4);
            let references = typed.source_statement_references().expect("native B1 reference pair");
            assert_eq!(resolved.source_statement_references(), Some(references));
        } else if proofless_positive {
            assert_eq!(diagnostics, 0, "{label} coherent proofless capture");
            let (typed, resolved) = task258_native_output(&ast, &module);
            assert_eq!(typed.source_statement().unwrap().statements().len(), 1);
            assert!(typed.source_statement_references().is_none());
            assert!(resolved.source_statement_references().is_none());
        } else {
            match mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module) {
                Err(error) => {
                    let lower_error = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
                        .expect_err("original source lowering owns this refusal");
                    assert_eq!(error, lower_error.to_string(), "{label} original-source refusal");
                }
                Ok(None) => {
                    let shells = mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module).collect();
                    let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(&ast, &shells, mizar_resolve::env::NamespacePath::new(module.path().as_str())).collect(&module);
                    assert!(!collected.diagnostics().is_empty(), "{label} declaration collection owns this refusal");
                }
                Ok(Some((original, symbols))) => {
                    match mizar_resolve::symbols::source_proof_inputs(&original, &symbols) {
                        Err(error) => {
                            assert_eq!(mizar_resolve::symbols::source_proof_inputs(&original, &symbols), Err(error), "{label} resolver-owned refusal replay");
                        }
                        Ok((scope, labels, resolution)) => {
                            mizar_resolve::symbols::validate_source_proof_occurrences(&original, &symbols, &scope)
                                .unwrap_or_else(|error| panic!("{label} genuine occurrence validation: {error}"));
                            let before = (original.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone());
                            assert!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&original, &symbols, &scope).is_err(), "{label} native bounded source refusal");
                            assert_eq!((original, symbols, scope, labels, resolution), before, "{label} native refusal purity");
                        }
                    }
                }
            }
        }
        } else {
            assert!(!lexical_positive && !proofless_positive, "{label} positive capture requires an AST");
            assert!(!output.diagnostics.is_empty(), "{label} missing AST requires Frontend diagnostics");
        }
        assert_eq!(task258_native_output(&exact_ast, &exact_module), baseline, "{label} pristine native retry");
    }
}

#[test]
fn task258b1_keeps_task258a_and_active_corpus_routes_isolated() {
    let (a_ast, a_module, _, _) = task253_ast_from_source_text(TASK258A_SOURCE, 258_430);
    let (task_a, _) = task258_native_output(&a_ast, &a_module);
    assert!(task_a.source_statement_references().is_none());
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&a_ast, &a_module)
        .unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    let references = task_a.source_term().unwrap().references().iter()
        .map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>();
    assert_eq!(references, scope.references().iter().map(|reference| reference.ordinal()).collect::<Vec<_>>());
    assert_eq!(references, [0, 1]);
    let (b1_ast, b1_module, _, b1_symbols) = task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_439);
    let (b1, _) = task258_native_output(&b1_ast, &b1_module);
    assert!(b1.source_statement_references().is_some());
    assert!(source_statement_output_with_source(&b1_ast, b1_module, &b1_symbols, SOURCE_STATEMENT_B1_TEXT).is_none());

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
    let plan = build_test_plan(&config).expect("Task258B1 isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
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
        if source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
            .is_some_and(|result| {
                result.is_ok_and(|output| output.typed_ast.source_statement_references().is_some())
            })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B1 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b1_typed_final_ownership_clone_debug_and_empty_semantics_are_atomic() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_440);
    let (first, first_resolved) = task258_native_output(&ast, &module);
    let (second, second_resolved) = task258_native_output(&ast, &module);
    assert_eq!(first, first.clone());
    assert_eq!(first_resolved, first_resolved.clone());
    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first_resolved.debug_text(), second_resolved.debug_text());
    let debug = first.debug_text();
    let base = debug.find("source-statement-debug-v1").expect("base debug");
    let references = debug
        .find("source-statement-reference-debug-v1")
        .expect("reference debug");
    let nodes = debug.find("nodes:").expect("node debug");
    assert!(base < references && references < nodes);
    assert!(first.facts().is_empty());
    assert!(first.diagnostics().is_empty());
    assert!(first_resolved.checked_formulas().is_empty());
    assert!(first_resolved.statement_semantics().is_empty());
    assert!(first_resolved.checked_proofs().is_empty());
    assert!(first_resolved.checked_proof_nodes().is_empty());
    assert!(first_resolved.checked_terminal_goals().is_empty());
    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    let statement = first
        .source_statement()
        .expect("statement")
        .clone();
    let statement_references = first
        .source_statement_references()
        .expect("statement references")
        .clone();
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement(statement.clone())
            .expect_err("Task258B1 base must not use legacy installer"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement_references(statement, statement_references)
            .expect_err("Task248 and Task258B1 owners must not coexist"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
}
