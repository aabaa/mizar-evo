#[test]
fn task258b3n_real_frontend_freezes_named_witness_table_contract() {
    assert_eq!(SOURCE_STATEMENT_B3N_TEXT.len(), 107);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3N_TEXT),
        "a57022c4b75991dd4308943477e03819f5bfe2c0d23ea1030730256252d7d329"
    );
    assert!(SOURCE_STATEMENT_B3N_TEXT.ends_with('\n'));
    let (ast, module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_700);
    let extracted: SourceStatementB3NExtraction =
        extract_named_witness_source_statement(&ast, SOURCE_STATEMENT_B3N_TEXT)
            .expect("Task258B3N exact parser shape");
    assert_eq!(
        (ast.nodes().len(), ast.root().expect("root").index()),
        (51, 50)
    );
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [47, 45]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [32, 42]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 106), (90, 101)]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(62, 67), (95, 100)]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [28, 30, 34, 38, 40]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)]
    );
    assert_eq!(
        (
            extracted.take_site.node().index(),
            extracted.witness_site.node().index(),
            extracted.name_site.node().index(),
        ),
        (37, 36, 13)
    );
    let (source, resolver) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, owner_labels, label_resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &resolver).unwrap();
    assert!(resolver.labels().is_empty());
    assert!(resolver.imports().is_empty());
    assert_eq!(resolver.contributions().len(), 1);
    assert_eq!((owner_labels.projections().len(), label_resolution.index().len()), (1, 1));
    let labels = label_resolution.index().visible_candidates(
        &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
        "FormulaStatementNamedWitnessSmoke",
    );
    let [label] = labels.as_slice() else { panic!("one native theorem owner label") };
    assert_eq!(label.kind(), mizar_resolve::resolved_ast::LabelKind::Theorem);
    assert_eq!(label.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(label.export_status(), mizar_resolve::env::ExportStatus::Exported);
    assert_eq!(label.contribution().index(), 0);
    assert_eq!(label.origin().structural_path(), [2, 1]);
    let (typed, resolved) = task258_native_output(&ast, &module);
    let statement = typed.source_statement().expect("statement");
    let handoff = typed
        .source_statement_witnesses()
        .expect("witness handoff");
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
        ),
        (2, 1, 0)
    );
    let primary = typed.source_term().expect("primary");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (5, 5, 0)
    );
    let atomic = typed.source_atomic_formula().expect("atomic");
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
    let proof_context = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof_context.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement {
            source_range: mizar_session::SourceRange {
                source_id: ast.source_id,
                start: 68,
                end: 105,
            }
        }
    );
    assert_eq!(
        (
            proof_context.parent.map(|id| id.index()),
            proof_context.layer,
            proof_context
                .lexical_scope
                .as_ref()
                .map(|scope| scope.path().to_vec()),
            proof_context
                .bindings
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            proof_context
                .visible_bindings
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            proof_context.recovery,
        ),
        (
            Some(0),
            mizar_checker::binding_env::BindingContextLayer::Proof,
            Some(vec![4]),
            Vec::new(),
            vec![0],
            mizar_checker::binding_env::BindingContextRecovery::Normal,
        )
    );
    for (index, ((_, term), (_, reference))) in primary
        .terms()
        .iter()
        .zip(primary.references().iter())
        .enumerate()
    {
        assert_eq!(term.site().node().index(), [28, 30, 34, 38, 40][index]);
        assert_eq!(
            (term.source_range().start, term.source_range().end),
            [(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)][index]
        );
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context().index(), [0, 0, 1, 1, 1][index]);
        assert_eq!(term.spelling(), "x");
        assert_eq!(
            term.kind(),
            mizar_checker::source_term::SourcePrimaryTermKind::VariableReference
        );
        assert_eq!(
            term.role(),
            mizar_checker::source_term::SourcePrimaryTermRole::Value
        );
        assert_eq!(
            term.recovery(),
            mizar_checker::source_term::SourcePrimaryTermRecovery::Normal
        );
        assert!(term.parent().is_none());
        assert_eq!(reference.term().index(), index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(
            reference.role(),
            mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
        );
        assert_eq!(reference.use_ordinal(), index);
        assert_eq!(scope.references()[index].ordinal(), index);
        assert_eq!(scope.references()[index].range(), term.source_range());
        assert_eq!(
            reference.lexical_scope().map(|scope| scope.path().to_vec()),
            Some(if index < 2 { vec![] } else { vec![4] })
        );
    }
    for index in 0..2 {
        let formula = atomic
            .formulas()
            .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index))
            .expect("atomic formula");
        assert_eq!(formula.site().node().index(), [32, 42][index]);
        assert_eq!(
            (formula.source_range().start, formula.source_range().end),
            [(62, 67), (95, 100)][index]
        );
        assert_eq!(formula.source_ordinal(), index);
        assert_eq!(formula.context().index(), index);
        assert_eq!(formula.spelling(), "x = x");
        assert_eq!(
            formula.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
        );
        assert_eq!(
            formula.recovery(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
        );
    }
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term) => {
                    term.index()
                }
                _ => usize::MAX,
            })
            .collect::<Vec<_>>(),
        [0, 1, 3, 4]
    );
    for index in 0..4 {
        let edge = atomic
            .edges()
            .get(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index))
            .expect("atomic edge");
        let formula = usize::from(index >= 2);
        let ordinal = index % 2;
        assert_eq!((edge.formula().index(), edge.ordinal()), (formula, ordinal));
        assert_eq!(
            edge.role(),
            if ordinal == 0 {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
        let request = atomic
            .requests()
            .get(mizar_checker::source_atomic_formula::SourceAtomicRequestId::new(index))
            .expect("atomic request");
        assert_eq!(
            (request.formula().index(), request.ordinal()),
            (formula, ordinal)
        );
        assert_eq!(
            request.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
        );
        assert_eq!(request.edge().map(|id| id.index()), Some(index));
        assert!(request.candidate().is_none());
        assert!(request.type_site().is_none());
        assert!(request.attribute().is_none());
    }
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
            47,
            19,
            106,
            "FormulaStatementNamedWitnessSmoke",
            0,
            mizar_checker::source_statement::SourceTheoremRole::Theorem,
            mizar_checker::source_statement::SourceTheoremStatus::Unmodified,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    assert_eq!(statement.checked_owner().symbol(), owner.symbol());
    for index in 0..2 {
        let row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(
                index,
            ))
            .expect("statement row");
        assert_eq!(
            (
                row.owner().index(),
                row.context().index(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
                row.recovery(),
            ),
            (
                0,
                index,
                [47, 45][index],
                [19, 90][index],
                [106, 101][index],
                [0, 2][index],
                [
                    "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
                    "thus x = x ;",
                ][index],
                [
                    mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                    mizar_checker::source_statement::SourceStatementKind::Conclusion,
                ][index],
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            )
        );
        assert_eq!(
            row.formula(),
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index)
            )
        );
        let context = statement
            .contexts()
            .get(mizar_checker::source_statement::SourceStatementContextId::new(index))
            .expect("statement context");
        assert_eq!(
            (
                context.statement().index(),
                context.binding_context().index(),
                context.source_range(),
                context
                    .visible_bindings()
                    .iter()
                    .map(|id| id.index())
                    .collect::<Vec<_>>(),
            ),
            (index, index, row.source_range(), vec![0])
        );
        let fact = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(
            (
                fact.statement().index(),
                fact.context().index(),
                fact.ordinal(),
                fact.kind(),
                fact.binding().index(),
                fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
            ),
            (
                index,
                index,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                if index == 0 { vec![0, 1] } else { vec![3, 4] },
            )
        );
        let candidate = statement
            .candidate_facts()
            .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(index))
            .expect("candidate fact");
        assert_eq!(
            (
                candidate.statement().index(),
                candidate.context().index(),
                candidate.ordinal(),
                candidate.kind(),
                candidate.formula(),
            ),
            (
                index,
                index,
                0,
                mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition,
                row.formula(),
            )
        );
    }
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            handoff.witnesses().len(),
            handoff.names().len(),
        ),
        (1, 2, 2, 2, 2, 1, 1)
    );
    let witness = handoff
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
        .expect("witness");
    let name = handoff
        .names()
        .get(mizar_checker::source_statement::SourceStatementWitnessNameId::new(0))
        .expect("name");
    assert_eq!(
        witness.name(),
        Some(mizar_checker::source_statement::SourceStatementWitnessNameId::new(0))
    );
    assert_eq!(
        witness.kind(),
        mizar_checker::source_statement::SourceStatementWitnessKind::Named
    );
    assert_eq!(witness.spelling(), "y = x");
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        name.witness(),
        mizar_checker::source_statement::SourceStatementWitnessId::new(0)
    );
    assert_eq!(
        (
            name.site().node().index(),
            name.source_range().start,
            name.source_range().end
        ),
        (13, 81, 82)
    );
    assert_eq!(name.spelling(), "y");
    assert_eq!(
        [
            statement
                .statements()
                .get(mizar_checker::source_statement::SourceStatementId::new(0))
                .expect("theorem")
                .source_ordinal(),
            witness.source_ordinal(),
            statement
                .statements()
                .get(mizar_checker::source_statement::SourceStatementId::new(1))
                .expect("conclusion")
                .source_ordinal(),
        ],
        [0, 1, 2]
    );
    for index in 0..51 {
        let surface = ast.nodes().get(index).expect("surface node");
        let typed = typed
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("typed node");
        assert_eq!(
            typed.anchor,
            mizar_session::SourceAnchor::Range(surface.range),
            "node {index}"
        );
        assert_eq!(
            typed
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            "node {index} children"
        );
    }
    assert!(handoff.debug_text().contains(
        "witness#0 owner=0 binding_context=1 term=primary#2 take_range=76..87 take_site=37 range=81..86 site=36 source_ordinal=1 ordinal=0 kind=named recovery=normal spelling=\"y = x\" name=0\nwitness-name#0 witness=0 range=81..82 site=13 recovery=normal spelling=\"y\""
    ));
    assert_eq!(primary.references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), [0, 1, 2, 3, 4]);
    assert!(!primary.terms().iter().any(|(_, term)| term.site() == name.site()));
    let reserve = statement.binding_env().bindings().get(mizar_checker::binding_env::BindingId::new(0)).unwrap();
    assert_eq!(reserve.kind, mizar_checker::binding_env::BindingKind::ReservedVariable);
    assert_eq!(reserve.status, mizar_checker::binding_env::BindingStatus::Active);
    assert_eq!(reserve.visible_after_ordinal, 0);
    assert!(typed.source_proof_local_declaration().is_none());
    assert!(resolved.source_proof_local_declaration().is_none());
    assert_eq!(
        typed.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        typed.source_statement_witnesses(),
        resolved.source_statement_witnesses()
    );
    assert!(resolved.source_statement_references().is_none());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.collection_candidates().is_empty());
    assert!(resolved.expanded_candidates().is_empty());
    assert!(resolved.template_expansions().is_empty());
    assert!(resolved.viable_candidates().is_empty());
    assert!(resolved.viability_decisions().is_empty());
    assert!(resolved.specificity_graphs().is_empty());
    assert!(resolved.resolved_overloads().is_empty());
    assert!(resolved.inserted_coercions().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
}

#[test]
fn task258b3n_validation_precedence_rows_and_replay_fail_closed() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_701);
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    assert!(symbols.labels().is_empty(), "native declaration environment has no legacy label rows");
    assert_eq!((labels.projections().len(), resolution.index().len()), (1, 1));
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).expect("pristine native lower baseline");
    let baseline = task258_native_output(&ast, &module);
    let installed = base.clone().with_source_statement_from_original(&source, &symbols, &scope).expect("pristine original installation");
    assert_eq!(installed, baseline.0);
    assert_eq!(assemble_empty_resolved_typed_ast(&installed, Vec::new()).unwrap(), baseline.1);
    let owner = installed.source_statement().unwrap().checked_owner().symbol();
    let mut missing_owner = mizar_resolve::env::SymbolIndex::new();
    for entry in symbols.symbols().iter() {
        if entry.symbol() != owner { missing_owner.insert(entry.clone()); }
    }
    let before = (ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone(), base.clone());
    // The historical named-row and bundle corruption duties are preserved at the private producers.
    // These controls alter the two actual native environment representations.
    for control in ["missing owner symbol", "unexpected label enrichment"] {
        let mutated = SymbolEnv::new(module.clone(), mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: if control == "missing owner symbol" { missing_owner.clone() } else { symbols.symbols().clone() },
            labels: if control == "unexpected label enrichment" { resolution.index().clone() } else { symbols.labels().clone() },
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        });
        assert_ne!(mutated, symbols, "{control} changes an actual native field");
        let mutated_before = mutated.clone();
        assert_eq!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &mutated, &scope).unwrap_err(), "source symbol environment does not match its declaration collection", "{control} original authentication");
        assert_eq!(base.clone().with_source_statement_from_original(&source, &mutated, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement), "{control} original installation");
        assert_eq!(mutated, mutated_before, "{control} passed dependency purity");
        assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone(), base.clone()), before, "{control} pristine dependency purity");
        assert_eq!(task258_native_output(&ast, &module), baseline, "{control} pristine native retry");
    }
}

#[test]
fn task258b3n_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_704);
    let baseline = task258_native_output(&ast, &module);
    assert_eq!(baseline.0.nodes().len(), 51);
    assert!(source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B3N_TEXT).is_none());
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3N_TEXT.trim_end_matches('\n').to_owned(),
        format!("{SOURCE_STATEMENT_B3N_TEXT}\n"),
        SOURCE_STATEMENT_B3N_TEXT.replacen("  take y = x;", "   take y = x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen(
            "FormulaStatementNamedWitnessSmoke",
            "FormulaStatementNamedWitnessNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source,).is_none(),
            "byte near miss {ordinal}"
        );
        let (coherent_ast, coherent_module, _, _) = task253_ast_from_source_text(&source, 258_730 + ordinal);
        let (typed, resolved) = task258_native_output(&coherent_ast, &coherent_module);
        assert_eq!(typed.source_statement_witnesses().unwrap().names().len(), 1);
        assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
        assert_eq!(task258_native_output(&ast, &module), baseline);

    }
    let reordered = SOURCE_STATEMENT_B3N_TEXT.replacen(
        "  take y = x;\n  thus x = x;\n",
        "  thus x = x;\n  take y = x;\n",
        1,
    );
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3_TEXT.to_owned(),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take z = x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take = x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y = x, x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y = {x};", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("  take y = x;\n", "", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("  take y = x;\n", "  take y = x;\n  take z = x;\n", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("  thus x = x;\n", "  thus x = x;\n  thus x = x;\n", 1),
        reordered,
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y = x", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen(": x = x proof", ": ex z being set st z = z proof", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen(": x = x proof", ": (x = x) & (x = x) proof", 1),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, _, near_symbols) =
            task253_ast_from_source_text(&source, 258_710 + ordinal);
        assert!(
            extract_named_witness_source_statement(&near_ast, SOURCE_STATEMENT_B3N_TEXT).is_none(),
            "subtree extraction near miss {ordinal}"
        );
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_symbols,
                SOURCE_STATEMENT_B3N_TEXT,
            )
            .is_none(),
            "subtree route near miss {ordinal}"
        );
        if ordinal <= 1 {
            let (typed, resolved) = task258_native_output(&near_ast, &near_module);
            assert_eq!(typed.source_statement_witnesses().unwrap().names().len(), usize::from(ordinal == 1));
            assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
        }

    }    for (ordinal, source_text) in [
        SOURCE_STATEMENT_B3N_TEXT.replace("take y = x;", "take x = x;"),
        SOURCE_STATEMENT_B3N_TEXT.replace("thus x = x;", "thus y = y;"),
        SOURCE_STATEMENT_B3N_TEXT.replace("take y = x;", "take y = x, x;"),
        SOURCE_STATEMENT_B3N_TEXT.replace("take y = x;", "take y = {x};"),
        SOURCE_STATEMENT_B3N_TEXT.replace("take y = x;", "take y = x;\n  take z = x;"),
        SOURCE_STATEMENT_B3N_TEXT.replace("thus x = x;", "thus x = x;\n  thus x = x;"),
        SOURCE_STATEMENT_B3N_TEXT.replace("  take y = x;\n  thus x = x;", "  thus x = x;\n  take y = x;"),
        SOURCE_STATEMENT_B3N_TEXT.replace("take y = x;", "take y = x"),
    ].into_iter().enumerate() {
        let (near_ast, near_module, _, _, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(&source_text, 258_740 + ordinal);
        if diagnostics != 0 || near_ast.nodes().iter().any(|node| node.recovered) {
            assert!(diagnostics != 0 || near_ast.nodes().iter().any(|node| node.recovered), "native near miss {ordinal} Frontend refusal");
        } else {
            match mizar_resolve::symbols::source_theorem_module_original_inputs(&near_ast, &near_module) {
                Err(error) => assert_eq!(mizar_resolve::symbols::source_theorem_module_original_inputs(&near_ast, &near_module), Err(error), "native near miss {ordinal} source refusal replay"),
                Ok(None) => {
                    let shells = mizar_resolve::declarations::DeclarationShellCollector::new(&near_ast, &near_module).collect();
                    let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(&near_ast, &shells, mizar_resolve::env::NamespacePath::new(near_module.path().as_str())).collect(&near_module);
                    assert!(!collected.diagnostics().is_empty(), "native near miss {ordinal} declaration refusal");
                }
                Ok(Some((original, env))) => match mizar_resolve::symbols::source_proof_inputs(&original, &env) {
                    Err(error) => assert_eq!(mizar_resolve::symbols::source_proof_inputs(&original, &env), Err(error), "native near miss {ordinal} Resolver refusal replay"),
                    Ok((scope, labels, resolution)) => {
                        let before = (original.clone(), env.clone(), scope.clone(), labels.clone(), resolution.clone());
                        if ordinal == 2 {
                            let (typed, resolved) = task258_native_output(&near_ast, &near_module);
                            let mixed = typed.source_statement_witnesses().unwrap();
                            assert_eq!((mixed.witnesses().len(), mixed.names().len()), (2, 1));
                            assert_eq!(resolved.source_statement_witnesses(), Some(mixed));
                            assert!(typed.source_proof_local_declaration().is_none());
                            assert!(resolved.source_proof_local_declaration().is_none());
                        } else {
                            assert!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&original, &env, &scope).is_err(), "native near miss {ordinal} bounded intake refusal");
                        }
                        assert_eq!((original, env, scope, labels, resolution), before, "native near miss {ordinal} dependency purity");
                    }
                },
            }
        }
        assert_eq!(task258_native_output(&ast, &module), baseline, "native near miss {ordinal} pristine retry");
    }

}

#[test]
fn task258b3n_family_and_active_route_isolation_is_atomic_in_both_orders() {
    let (ast, module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_704);
    let (typed, _) = task258_native_output(&ast, &module);
    let (original, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&original, &env).unwrap();
    let b3n_statement = typed
        .source_statement()
        .expect("B3N statement")
        .clone();
    let b3n_witnesses = typed
        .source_statement_witnesses()
        .expect("B3N witnesses")
        .clone();
    let baseline_debug = typed.debug_text();

    let (b3_ast, b3_module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_720);
    let (b3, _) = task258_native_output(&b3_ast, &b3_module);
    let b3_witness = b3
        .source_statement_witnesses()
        .expect("B3 witness");
    assert!(b3_witness.names().is_empty());
    assert_eq!(
        b3_witness.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: {}::{}\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\"\n",
            b3_witness.module_id().package().as_str(),
            b3_witness.module_id().path().as_str(),
            b3
                .source_statement()
                .expect("B3 statement")
                .debug_text(),
            b3.source_term().expect("B3 primary").debug_text(),
        )
    );
    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
        SOURCE_STATEMENT_B3_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (foreign_ast, foreign_module, _, _) = task253_ast_from_source_text(source, 258_722 + ordinal);
        let (foreign_typed, _) = task258_native_output(&foreign_ast, &foreign_module);
        let foreign_debug = foreign_typed.debug_text();
        assert_eq!(
            foreign_typed
                .clone()
                .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone(),),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
            "foreign family {ordinal}"
        );
        assert_eq!(foreign_typed.debug_text(), foreign_debug);
        if let Some(references) = foreign_typed.source_statement_references() {
            assert_eq!(
                typed.clone().with_source_statement_references(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    references.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3N receiver family {ordinal}"
            );
        } else if let Some(foreign_witnesses) = foreign_typed.source_statement_witnesses() {
            assert_eq!(
                typed.clone().with_source_statement_witnesses(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    foreign_witnesses.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3N receiver family {ordinal}"
            );
        } else {
            assert_eq!(
                typed.clone().with_source_statement(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3N receiver family {ordinal}"
            );
        }
        let (foreign_original, foreign_env) = mizar_resolve::symbols::source_theorem_module_original_inputs(&foreign_ast, &foreign_module).unwrap().unwrap();
        let (foreign_scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&foreign_original, &foreign_env).unwrap();
        let (_, _, foreign_base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&foreign_original, &foreign_env, &foreign_scope).expect("genuine foreign lower tuple");
        let foreign_before = (foreign_original.clone(), foreign_env.clone(), foreign_scope.clone(), foreign_base.clone());
        assert_eq!(foreign_base.clone().with_source_statement_from_original(&original, &env, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement), "foreign lower tuple {ordinal} original-entry refusal");
        assert_eq!((foreign_original, foreign_env, foreign_scope, foreign_base), foreign_before, "foreign lower tuple {ordinal} purity");
        assert_eq!(typed.debug_text(), baseline_debug);
    }

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
    let plan = build_test_plan(&config).expect("Task258B3N isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(active_ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &active_ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let active_symbols =
            augment_type_elaboration_import_summaries(&active_ast, &resolver.module, resolver.env);
        let extracted = extract_named_witness_source_statement(&active_ast, &source).is_some();
        let selected_route = source_statement_output_with_source(
            &active_ast,
            resolver.module.clone(),
            &active_symbols,
            &source,
        )
        .is_some_and(|result| {
            result.is_ok_and(|output| {
                output
                    .typed_ast
                    .source_statement_witnesses()
                    .is_some_and(|witnesses| !witnesses.names().is_empty())
            })
        });
        if extracted || selected_route {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3N selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3n_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_721);
    let (typed, resolved) = task258_native_output(&ast, &module);
    assert_eq!(
        typed.clone().debug_text(),
        typed.debug_text()
    );
    assert_eq!(
        resolved.clone().debug_text(),
        resolved.debug_text()
    );
    assert_eq!(
        typed.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        typed.source_statement_witnesses(),
        resolved.source_statement_witnesses()
    );
    assert!(typed.source_statement_references().is_none());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.collection_candidates().is_empty());
    assert!(resolved.expanded_candidates().is_empty());
    assert!(resolved.template_expansions().is_empty());
    assert!(resolved.viable_candidates().is_empty());
    assert!(resolved.viability_decisions().is_empty());
    assert!(resolved.specificity_graphs().is_empty());
    assert!(resolved.resolved_overloads().is_empty());
    assert!(resolved.inserted_coercions().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    let baseline_debug = typed.debug_text();
    assert_eq!(typed.clone().with_source_statement_witnesses(
        typed.source_statement().unwrap().clone(), typed.source_statement_witnesses().unwrap().clone(),
    ), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert!(typed.source_proof_local_declaration().is_none());
    assert!(resolved.source_proof_local_declaration().is_none());
    assert!(typed.contexts().is_empty());
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert_eq!(typed.debug_text(), baseline_debug);
    assert_eq!(task258_native_output(&ast, &module), (typed, resolved));
}
