#[test]
fn task258b3m2b2b1b1_real_frontend_freezes_wrapped_application_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B2B1B1_TEXT.len(), 158);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B1B1_TEXT),
        "e3778c52953be6848cfc0aaa989e6d5c83625e8b798f27df7f7b2b5606271777"
    );
    let (ast, module, symbols) = task258b3m2b2b1b1_fixture(258_980);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (67, Some(66))
    );
    assert!(
        ast.nodes()
            .iter()
            .all(|node| !node.recovered && node.range.source_id == ast.source_id)
    );
    let mut arena_snapshot = String::new();
    for (index, node) in ast.nodes().iter().enumerate() {
        use std::fmt::Write as _;
        writeln!(
            arena_snapshot,
            "{index}|{:?}|{}..{}|{}|{:?}",
            node.kind,
            node.range.start,
            node.range.end,
            node.recovered,
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        )
        .expect("arena snapshot write");
    }
    use std::fmt::Write as _;
    writeln!(
        arena_snapshot,
        "root={:?}",
        ast.root().map(|root| root.index())
    )
    .expect("root snapshot write");
    assert_eq!(
        sha256_text(&arena_snapshot),
        "16daa2fe739f0cf5304fc732ecbcd82836fe235024714a3d518df442f401dbeb"
    );
    let extracted: SourceStatementB3M2B2B1B1Extraction =
        extract_wrapped_application_witness_source_statement(
            &ast,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        )
        .expect("Task258B3M2B2B1B1 exact extraction");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.take_site.node().index(),
            extracted.take_range.start,
            extracted.take_range.end,
            extracted.witness_site.node().index(),
            extracted.witness_range.start,
            extracted.witness_range.end,
        ),
        (63, 48, 157, 56, 108, 53, 124, 138, 52, 129, 137)
    );
    assert_eq!(
        (
            extracted.application_node,
            extracted.application_wrapper_node,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (48, 50, 116, 156)
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [40, 42, 46, 47, 54, 56]
    );
    let resolver = source_statement_b3m2b2b1b1_resolver_env_for_test(&module, &symbols, &extracted)
        .expect("Task258B3M2B2B1B1 resolver projection");
    assert_eq!((resolver.imports().len(), resolver.labels().len()), (1, 1));
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver.symbols().visible_candidates(
        &namespace,
        "FormulaStatementParenthesizedApplicationWitnessSmoke",
    );
    let [owner] = owners.as_slice() else {
        panic!("Task258B3M2B2B1B1 exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 48, 157))
    );
    assert_eq!(owner.origin().structural_path(), [2, 1]);
    assert_eq!(owner.origin().source_id(), ast.source_id);
    assert_eq!(owner.origin().module_id(), &module);
    assert!(owner.origin().import_edge().is_none());
    assert!(!owner.origin().is_recovered());
    assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    let contribution = resolver
        .contributions()
        .get(owner.contribution())
        .expect("Task258B3M2B2B1B1 owner contribution");
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::LocalSource { source_id }
            if *source_id == ast.source_id
    ));
    assert_eq!(contribution.module(), &module);
    assert_eq!(
        contribution.anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 29, 47))
    );
    let labels = resolver.labels().visible_candidates(
        &namespace,
        "FormulaStatementParenthesizedApplicationWitnessSmoke",
    );
    let [label] = labels.as_slice() else {
        panic!("Task258B3M2B2B1B1 exact theorem label");
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

    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    assert!(
        source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        )
        .is_none()
    );
    let lower = source_statement_b3m2b2b1b1_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        |symbols| symbols,
    )
    .expect("authentic extraction")
    .expect("authenticated lower handoff before raw refusal");
    let primary = &lower.primary;
    let application = lower.application.as_ref().expect("wrapped application");
    let atomic = &lower.atomic;
    let statements = &lower.statement;
    let witnesses = &lower.witness;
    let bindings = &lower.binding_env;
    assert_eq!(bindings.source_id(), ast.source_id);
    assert_eq!(bindings.module_id(), &module);
    let module_context = bindings
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("Task48 module context");
    assert!(matches!(
        module_context.owner,
        mizar_checker::binding_env::BindingContextOwner::Module
    ));
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
    let proof_context = bindings
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("Task48 proof context");
    assert!(matches!(
        proof_context.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
            if source_range == range(ast.source_id, 116, 156)
    ));
    assert_eq!(
        proof_context.parent,
        Some(mizar_checker::binding_env::BindingContextId::new(0))
    );
    assert_eq!(
        proof_context.layer,
        mizar_checker::binding_env::BindingContextLayer::Proof
    );
    assert_eq!(
        proof_context
            .lexical_scope
            .as_ref()
            .map(mizar_resolve::names::LocalTermScope::path),
        Some(&[0][..])
    );
    assert!(proof_context.bindings.is_empty());
    assert_eq!(
        proof_context.visible_bindings,
        [mizar_checker::binding_env::BindingId::new(0)]
    );
    assert_eq!(
        proof_context.recovery,
        mizar_checker::binding_env::BindingContextRecovery::Normal
    );
    let binding = bindings
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("Task48 reserved binding");
    assert_eq!(binding.spelling, "x");
    assert_eq!(
        binding.owner_context,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    assert_eq!(binding.declaration_range, range(ast.source_id, 37, 38));
    assert_eq!(
        binding.kind,
        mizar_checker::binding_env::BindingKind::ReservedVariable
    );
    assert!(matches!(
        &binding.identity,
        mizar_checker::binding_env::BinderIdentity::ReservedVariable {
            spelling,
            declaration_range,
        } if spelling == "x" && *declaration_range == range(ast.source_id, 37, 38)
    ));
    assert_eq!(binding.visible_after_ordinal, 0);
    assert_eq!(
        binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(range(ast.source_id, 43, 46))
    );
    assert_eq!(
        binding.status,
        mizar_checker::binding_env::BindingStatus::Reserved
    );
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(
        binding.recovery,
        mizar_checker::binding_env::BindingRecoveryState::Normal
    );
    assert!(bindings.diagnostics().is_empty());
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
            application.applications().len(),
            application.wrappers().len(),
            application.candidates().len(),
            application.arguments().len(),
            application.type_requests().len(),
        ),
        (6, 4, 2, 1, 1, 1, 2, 2)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len(),
            statements.owners.len(),
            statements.statements.len(),
            statements.contexts.len(),
            statements.input_facts.len(),
            statements.candidate_facts.len(),
            witnesses.witnesses.len(),
            witnesses.names.len(),
        ),
        (2, 4, 4, 1, 2, 2, 2, 2, 1, 0)
    );
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.context().index(),
                row.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                40,
                110,
                111,
                0,
                0,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
            ),
            (
                42,
                114,
                115,
                1,
                0,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
            ),
            (
                46,
                130,
                131,
                2,
                1,
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
            ),
            (
                47,
                135,
                136,
                3,
                1,
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
            ),
            (
                54,
                146,
                147,
                4,
                1,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
            ),
            (
                56,
                150,
                151,
                5,
                1,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
            ),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| row
                .lexical_scope()
                .map(mizar_resolve::names::LocalTermScope::path))
            .collect::<Vec<_>>(),
        [None, None, Some(&[0][..]), Some(&[0][..])]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| (
                row.term().index(),
                row.binding().index(),
                row.role(),
                row.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                1,
            ),
            (
                1,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                1,
            ),
            (
                4,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                1,
            ),
            (
                5,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                1,
            ),
        ]
    );
    assert_eq!(
        primary
            .numeric_type_requests()
            .iter()
            .map(|(_, row)| (
                row.term().index(),
                row.owner().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.spelling(),
                row.request_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [(2, 46, 130, 131, "1", 0), (3, 47, 135, 136, "2", 1)]
    );
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.context().index(),
                row.kind(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                44,
                110,
                115,
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
                "x = x",
            ),
            (
                58,
                146,
                151,
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
                "x = x",
            ),
        ]
    );
    let atomic_targets = atomic
        .edges()
        .iter()
        .map(|(_, row)| {
            (
                row.formula().index(),
                row.ordinal(),
                row.role(),
                row.target(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        atomic_targets,
        [
            (
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(0),
                ),
            ),
            (
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(1),
                ),
            ),
            (
                1,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(4),
                ),
            ),
            (
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(5),
                ),
            ),
        ]
    );
    assert!(atomic_targets.iter().all(|(_, _, _, target)| !matches!(
        target,
        mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Application(_)
    )));
    assert_eq!(
        atomic
            .requests()
            .iter()
            .map(|(_, row)| (
                row.formula().index(),
                row.ordinal(),
                row.kind(),
                row.edge().map(|edge| edge.index()),
                row.candidate(),
                row.type_site(),
                row.attribute(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(0),
                None,
                None,
                None,
            ),
            (
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(1),
                None,
                None,
                None,
            ),
            (
                1,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(2),
                None,
                None,
                None,
            ),
            (
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(3),
                None,
                None,
                None,
            ),
        ]
    );
    let owner_row = &statements.owners[0];
    assert_eq!(&owner_row.symbol, owner.symbol());
    assert_eq!(owner_row.contribution, owner.contribution());
    assert_eq!(owner_row.site.node().index(), 63);
    assert_eq!(owner_row.source_range, range(ast.source_id, 48, 157));
    assert_eq!(
        owner_row.spelling,
        "FormulaStatementParenthesizedApplicationWitnessSmoke"
    );
    assert_eq!(
        owner_row.role,
        mizar_checker::source_statement::SourceTheoremRole::Theorem
    );
    assert_eq!(
        owner_row.status,
        mizar_checker::source_statement::SourceTheoremStatus::Unmodified
    );
    assert_eq!(
        owner_row.recovery,
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    for index in 0..2 {
        let statement = &statements.statements[index];
        assert_eq!(statement.owner.index(), 0);
        assert_eq!(statement.context.index(), index);
        assert_eq!(
            statement.formula,
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(statement.source_ordinal, [0, 2][index]);
        assert_eq!(statement.site.node().index(), [63, 61][index]);
        assert_eq!(
            statement.source_range,
            [
                range(ast.source_id, 48, 157),
                range(ast.source_id, 141, 152)
            ][index]
        );
        assert_eq!(
            statement.kind,
            [
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
            ][index]
        );
        assert_eq!(
            statement.recovery,
            mizar_checker::source_statement::SourceStatementRecovery::Normal
        );
        let context = &statements.contexts[index];
        assert_eq!(context.statement.index(), index);
        assert_eq!(context.binding_context.index(), index);
        assert_eq!(context.source_range, statement.source_range);
        assert_eq!(
            context.visible_bindings,
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        let input_fact = &statements.input_facts[index];
        assert_eq!(input_fact.statement.index(), index);
        assert_eq!(input_fact.context.index(), index);
        assert_eq!(input_fact.ordinal, 0);
        assert_eq!(
            input_fact.kind,
            mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard
        );
        assert_eq!(input_fact.binding.index(), 0);
        assert_eq!(
            input_fact
                .uses
                .iter()
                .map(|use_id| use_id.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![2, 3] }
        );
        let candidate_fact = &statements.candidate_facts[index];
        assert_eq!(candidate_fact.statement.index(), index);
        assert_eq!(candidate_fact.context.index(), index);
        assert_eq!(candidate_fact.ordinal, 0);
        assert_eq!(
            candidate_fact.kind,
            mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate_fact.formula, statement.formula);
    }
    let application_row = application
        .applications()
        .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
        .expect("application 0");
    let wrapper = application
        .wrappers()
        .get(mizar_checker::source_application::SourceFunctorWrapperId::new(0))
        .expect("wrapper 0");
    assert_eq!(
        (
            application_row.site().node().index(),
            application_row.source_range().start,
            application_row.source_range().end,
            application_row.context().index(),
            application_row.spelling(),
            wrapper.site().node().index(),
            wrapper.source_range().start,
            wrapper.source_range().end,
            wrapper.context().index(),
            wrapper.spelling(),
        ),
        (48, 130, 136, 1, "1 ++ 2", 50, 129, 137, 1, "( 1 ++ 2 )")
    );
    let candidate = application
        .candidates()
        .get(mizar_checker::source_application::SourceFunctorCandidateId::new(0))
        .expect("candidate 0");
    assert_eq!(candidate.symbol().module().package(), module.package());
    assert_eq!(
        candidate.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert_eq!(
        candidate.symbol().local().as_str(),
        "summary:parser.type_fixtures#parse-only#++:12"
    );
    assert_eq!(
        candidate.symbol().fqn().as_str(),
        "parser.type_fixtures::++#12"
    );
    assert_eq!(candidate.contribution().index(), 2);
    assert_eq!(
        candidate.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 7, 27))
    );
    assert_eq!(candidate.origin().structural_path(), [12]);
    assert!(candidate.origin().import_edge().is_none());
    assert!(!candidate.origin().is_recovered());
    assert!(candidate.signature().is_none());
    let witness = &witnesses.witnesses[0];
    assert_eq!(
        (
            witness.take_site.node().index(),
            witness.site.node().index(),
            witness.binding_context.index(),
            witness.source_ordinal,
            witness.ordinal,
            witness.spelling.as_str(),
        ),
        (53, 52, 1, 1, 0, "( 1 ++ 2 )")
    );
    assert_eq!(
        witness.term,
        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Application(
            mizar_checker::source_application::SourceFunctorApplicationId::new(0)
        )
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| row.use_ordinal())
            .collect::<Vec<_>>(),
        [1; 4]
    );
    assert!(
        source_statement_transport_detail_keys(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        )
        .is_none()
    );
    for (id, node) in lower.arena.iter() {
        assert_eq!(
            node.kind.as_str(),
            match id.index() {
                20 => "source.term.functor-head.single",
                40 | 42 | 54 | 56 => "source.term.variable-reference",
                46 | 47 => "source.term.numeral",
                44 | 58 => "source.formula.atomic.equality",
                48 => "source.term.functor-application.symbolic",
                50 => "source.term.functor-application.parenthesized",
                52 => "source.statement-witness.item",
                53 => "source.statement-witness.take",
                61 => "source.statement.conclusion",
                63 => "source.statement.theorem",
                _ => "source.surface.unowned",
            },
            "node {} kind",
            id.index()
        );
    }

    assert_eq!(
        (ast.snapshot_text(), module.clone(), symbols.clone()),
        before
    );

    assert_eq!(SOURCE_STATEMENT_B3M2B2B1A_TEXT.len(), 143);
    let (b1a_ast, b1a_module, b1a_symbols) = task258b3m2b2b1a_fixture(258_981);
    assert!(
        extract_wrapped_application_witness_source_statement(
            &b1a_ast,
            SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        )
        .is_none()
    );
    let b1a = task258b3m2b2b1a_lower_inputs(&b1a_ast, &b1a_module, &b1a_symbols);
    assert_eq!(
        b1a.application
            .expect("B1A lower application")
            .wrappers()
            .len(),
        0
    );
}

#[test]
fn task258b3m2b2b1b1_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols) = task258b3m2b2b1b1_fixture(258_982);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let capture = || {
        source_statement_b3m2b2b1b1_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
            |symbols| symbols,
        )
        .expect("authentic extraction")
        .expect("authenticated lower capture")
    };
    let baseline = capture();
    let replay = || {
        assert_eq!(format!("{:?}", capture()), format!("{baseline:?}"));
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
        assert!(
            source_statement_output_with_source(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
            )
            .is_none()
        );
    };
    replay();
    for mutation in [
        Task258B3M2B2B1B1ResolverMutation::LocalAndFqn,
        Task258B3M2B2B1B1ResolverMutation::StructuralPath,
        Task258B3M2B2B1B1ResolverMutation::Signature,
        Task258B3M2B2B1B1ResolverMutation::ExportStatus,
        Task258B3M2B2B1B1ResolverMutation::Contribution,
    ] {
        assert_eq!(
            source_statement_b3m2b2b1b1_output_with_resolver_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
                |symbols| task258b3m2b2b1b1_substituted_symbol_env(&symbols, mutation),
            )
            .expect("resolver extraction")
            .expect_err("B1B1P lower ownership refusal before capture"),
            "Task258B3M2B2B1B1 application ownership selector mismatch",
            "{mutation:?}"
        );
        replay();
    }
    let (stale_ast, stale_module, stale_symbols) = task258b3m2b2b1b1_fixture(258_983);
    let stale = source_statement_b3m2b2b1b1_output_with_resolver_mutation(
        &stale_ast,
        stale_module,
        &stale_symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        |symbols| symbols,
    )
    .expect("foreign extraction")
    .expect("foreign lower capture");
    let (wrong_arena_ast, wrong_arena_module, wrong_arena_symbols) =
        task258b3m2b2b1a_fixture(258_985);
    let wrong_arena =
        task258b3m2b2b1a_lower_inputs(&wrong_arena_ast, &wrong_arena_module, &wrong_arena_symbols)
            .arena;
    let stale_binding_env = stale.binding_env;
    let stale_primary = stale.primary;
    let stale_application = stale.application.expect("foreign application");
    let stale_atomic = stale.atomic;
    for mutation in 0..20 {
        let stale_binding_env = stale_binding_env.clone();
        let wrong_arena = wrong_arena.clone();
        let stale_primary = stale_primary.clone();
        let stale_application = stale_application.clone();
        let stale_atomic = stale_atomic.clone();
        let mut changed = None;
        let error = source_statement_b3m2b2b1b1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
            |input: &mut SourceStatementB3M2B2B1B1RouteInputs| {
                match mutation {
                0 => input.binding_env = stale_binding_env,
                1 => input.arena = wrong_arena,
                2 => input.primary = stale_primary,
                3 => input.application = None,
                4 => input.application = Some(stale_application),
                5 => input.atomic = stale_atomic,
                6 => input.statement.owners[0].spelling.push('x'),
                7 => input.statement.owners.clear(),
                8 => input.statement.statements[0].source_ordinal = 1,
                9 => input.statement.contexts[1].source_range.start += 1,
                10 => input.statement.input_facts[1].uses.swap(0, 1),
                11 => input.statement.candidate_facts[1].ordinal = 1,
                12 => {
                    input.witness.witnesses[0].term =
                        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                            mizar_checker::source_term::SourcePrimaryTermId::new(2),
                        )
                }
                13 => input.witness.witnesses[0].term =
                    mizar_checker::source_statement::SourceStatementWitnessTermTarget::Application(
                        mizar_checker::source_application::SourceFunctorApplicationId::new(1),
                    ),
                14 => {
                    input.witness.witnesses[0].site = mizar_checker::typed_ast::TypedSiteRef::Node(
                        mizar_checker::typed_ast::TypedNodeId::new(51),
                    )
                }
                15 => input.witness.witnesses[0].source_range.start += 1,
                16 => input.witness.witnesses[0].spelling.push('x'),
                17 => input.witness.witnesses[0].source_ordinal = 0,
                18 => {
                    input.witness.witnesses[0].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0)
                }
                19 => input.witness.witnesses.clear(),
                _ => unreachable!(),
            };
                changed = Some(input.clone());
            },
        )
        .expect("mutation extraction")
        .expect_err("retired raw output");
        let changed = changed.expect("real mutated handoff");
        assert_eq!(
            error,
            if mutation <= 5 {
                "Task258B3M2B2B1B1 lower dependency mismatch"
            } else {
                "source statement dependency mismatch"
            },
            "mutation {mutation}"
        );
        // Upper draft causality is checked directly in the Checker private suite;
        // raw profile retirement alone cannot detect any of these corruptions.
        if mutation <= 5 {
            assert_ne!(format!("{changed:?}"), format!("{baseline:?}"));
        } else if mutation <= 11 {
            assert_ne!(
                changed.statement, baseline.statement,
                "statement mutation {mutation}"
            );
            assert_eq!(changed.witness, baseline.witness);
        } else {
            assert_ne!(
                changed.witness, baseline.witness,
                "witness mutation {mutation}"
            );
            assert_eq!(changed.statement, baseline.statement);
        }
        replay();
    }
    assert_eq!(
        source_statement_b3m2b2b1b1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
            |input| {
                input.binding_env = stale_binding_env;
                input.statement.owners.clear();
                input.witness.witnesses.clear();
            },
        )
        .expect("combined extraction")
        .expect_err("lower authentication precedes raw refusal"),
        "Task258B3M2B2B1B1 lower dependency mismatch"
    );
    replay();
}

#[test]
fn task258b3m2b2b1b1_selector_and_byte_subtree_near_misses_are_exact() {
    let (exact_ast, exact_module, exact_symbols) = task258b3m2b2b1b1_fixture(258_984);
    let before = (
        exact_ast.snapshot_text(),
        exact_module.clone(),
        exact_symbols.clone(),
    );
    for target in 0..=67 {
        for mutation in 0..if target == 67 { 1 } else { 4 } {
            let changed = task258_mutated_surface(&exact_ast, target, mutation);
            match source_statement_b3m2b2b1b1_output_with_resolver_mutation(
                &changed,
                exact_module.clone(),
                &exact_symbols,
                SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
                |symbols| symbols,
            ) {
                None => {}
                Some(Err(error)) => assert_ne!(
                    error, "source statement dependency mismatch",
                    "node {target} mutation {mutation}: lower refusal precedes retired raw admission"
                ),
                Some(Ok(_)) => {
                    panic!("node {target} mutation {mutation} passed the lower contract")
                }
            }
            assert!(
                source_statement_output_with_source(
                    &changed,
                    exact_module.clone(),
                    &exact_symbols,
                    SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
                )
                .is_none()
            );
            assert_eq!(
                (
                    exact_ast.snapshot_text(),
                    exact_module.clone(),
                    exact_symbols.clone()
                ),
                before
            );
            assert!(
                source_statement_b3m2b2b1b1_output_with_resolver_mutation(
                    &exact_ast,
                    exact_module.clone(),
                    &exact_symbols,
                    SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
                    |symbols| symbols,
                )
                .expect("pristine extraction")
                .is_ok()
            );
        }
    }
    let (_, foreign_module, _) = task258b3m2b2b1b1_fixture(258_986);
    let mut wrong_source = exact_ast.clone();
    wrong_source.source_id = task258b3m2b2b3a_foreign_source_id(exact_ast.source_id);
    assert!(
        extract_wrapped_application_witness_source_statement(
            &wrong_source,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        )
        .is_none()
    );
    assert_eq!(
        source_statement_b3m2b2b1b1_output_with_resolver_mutation(
            &exact_ast,
            foreign_module,
            &exact_symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
            |symbols| symbols,
        )
        .expect("foreign module extraction")
        .expect_err("foreign module refusal"),
        "Task258A label resolver module mismatch"
    );
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B1B1_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B1B1_TEXT.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' {
            b'?'
        } else {
            b'!'
        };
        let loaded_source = String::from_utf8(bytes).expect("ASCII source mutation");
        assert!(
            extract_wrapped_application_witness_source_statement(&exact_ast, &loaded_source)
                .is_none()
        );
        assert_eq!(
            (
                exact_ast.snapshot_text(),
                exact_module.clone(),
                exact_symbols.clone()
            ),
            before
        );
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none(),
            "byte mutation {byte_index} selected"
        );
    }
    for loaded_source in [
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT
            .trim_end_matches('\n')
            .to_owned(),
        format!("{SOURCE_STATEMENT_B3M2B2B1B1_TEXT}\n"),
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen("take (1 ++ 2);", "take  (1 ++ 2);", 1),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none()
        );
    }

    let near_misses = [
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen("take (1 ++ 2);", "take 1 ++ 2;", 1),
            (0, 63),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen("take (1 ++ 2);", "take ((1 ++ 2));", 1),
            (0, 71),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen("take (1 ++ 2);", "take (2 ++ 1);", 1),
            (0, 67),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen("take (1 ++ 2);", "take (1 + 2);", 1),
            (12, 72),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen("take (1 ++ 2);", "take (1 ++ );", 1),
            (1, 64),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen("take (1 ++ 2);", "take 1, (1 ++ 2);", 1),
            (0, 72),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen(
                "FormulaStatementParenthesizedApplicationWitnessSmoke",
                "FormulaStatementParenthesizedApplicationWitnessNearMiss",
                1,
            ),
            (0, 67),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT.replacen(
                "import parser.type_fixtures;",
                "import parser.other_fixtures;",
                1,
            ),
            (14, 73),
        ),
    ];
    for (ordinal, (source, expected)) in near_misses.into_iter().enumerate() {
        let (ast, module, _, symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 259_000 + ordinal);
        assert_eq!(
            (diagnostics, ast.nodes().len()),
            expected,
            "near miss {ordinal} parser profile"
        );
        let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
        assert!(
            extract_wrapped_application_witness_source_statement(&ast, &source).is_none(),
            "near miss {ordinal} extracted"
        );
        assert!(
            source_statement_output_with_source(&ast, module, &symbols, &source).is_none(),
            "near miss {ordinal} selected"
        );
    }

    let (b1a_ast, b1a_module, b1a_symbols) = task258b3m2b2b1a_fixture(259_008);
    task258b3m2b2b1a_lower_inputs(&b1a_ast, &b1a_module, &b1a_symbols);
}

#[test]
fn task258b3m2b2b1b1_family_and_active_route_isolation_is_atomic() {
    let (b1b1_ast, b1b1_module, b1b1_symbols) = task258b3m2b2b1b1_fixture(259_009);
    let before = (
        b1b1_ast.snapshot_text(),
        b1b1_module.clone(),
        b1b1_symbols.clone(),
    );
    let b1b1 = source_statement_b3m2b2b1b1_output_with_resolver_mutation(
        &b1b1_ast,
        b1b1_module.clone(),
        &b1b1_symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        |symbols| symbols,
    )
    .expect("wrapped extraction")
    .expect("authenticated lower capture");
    let b1b1_baseline = format!("{b1b1:?}");
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3_TEXT,
        SOURCE_STATEMENT_B3N_TEXT,
        SOURCE_STATEMENT_B3M1_TEXT,
        SOURCE_STATEMENT_B3M2A_TEXT,
        SOURCE_STATEMENT_B3M2B1_TEXT,
        SOURCE_STATEMENT_B3M2B2A_TEXT,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(source, 259_010 + ordinal);
        let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
        assert!(
            extract_wrapped_application_witness_source_statement(&ast, source).is_none(),
            "legacy family {ordinal} matched B1B1"
        );
        if source == SOURCE_STATEMENT_B3M2B2B1A_TEXT {
            let bare = task258b3m2b2b1a_lower_inputs(&ast, &module, &symbols);
            for (primary, bindings, arena) in [
                (&bare.primary, &b1b1.binding_env, &b1b1.arena),
                (&b1b1.primary, &bare.binding_env, &bare.arena),
            ] {
                assert!(
                    matches!(
                        mizar_checker::source_term::SourcePrimaryTermProducer::build(
                            task258b3m2a_primary_input(primary),
                            bindings,
                            arena,
                        ),
                        Err(mizar_checker::source_term::SourcePrimaryTermError::InvalidTransaction)
                    ),
                    "family {ordinal} bare lower handoff order"
                );
            }
            let replay = source_statement_b3m2b2b1b1_output_with_resolver_mutation(
                &b1b1_ast,
                b1b1_module.clone(),
                &b1b1_symbols,
                SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
                |symbols| symbols,
            )
            .expect("wrapped extraction")
            .expect("wrapped replay lower capture");
            assert_eq!(format!("{replay:?}"), b1b1_baseline);
            assert_eq!(
                (
                    b1b1_ast.snapshot_text(),
                    b1b1_module.clone(),
                    b1b1_symbols.clone()
                ),
                before
            );
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2A_TEXT {
            let before = (ast.snapshot_text(), module.clone(), symbols.clone());
            assert!(extract_numeral_witness_source_statement(&ast, source).is_some());
            assert!(source_statement_output_with_source(
                &ast, module.clone(), &symbols, source,
            ).is_none());
            assert_eq!(source_statement_b3m2a_output_with_resolver_mutation(
                &ast, module.clone(), &symbols, source, |symbols| symbols,
            ).expect("authentic retired numeral extraction").expect_err("retired raw profile"),
                "source statement dependency mismatch");
            assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
            continue;
        }
        let typed = if source == SOURCE_STATEMENT_B3_TEXT || source == SOURCE_STATEMENT_B3N_TEXT || source == SOURCE_STATEMENT_B3M1_TEXT || source == SOURCE_STATEMENT_B3M2B1_TEXT || source == SOURCE_STATEMENT_B3M2B2A_TEXT {
            task258_native_output(&ast, &module).0
        } else {
            source_statement_output_with_source(&ast, module, &symbols, source)
                .expect("legacy family selector").expect("legacy family output").typed_ast
        };
        assert!(typed.source_statement_witnesses().is_some());
        assert_eq!(
            typed
                .source_application()
                .map(|application| application.wrappers().len()),
            if source == SOURCE_STATEMENT_B3M2B2B1A_TEXT {
                Some(0)
            } else {
                None
            }
        );
        let legacy_baseline = typed.debug_text();
        let legacy_primary = typed.source_term().expect("legacy primary");
        let legacy_binding = typed
            .source_statement()
            .expect("legacy statement")
            .binding_env();
        // The private Checker suite retains installation in both upper-family orders.
        // These real lower constructors reject foreign source/module dependencies.
        for (primary, bindings, arena) in [
            (legacy_primary, &b1b1.binding_env, &b1b1.arena),
            (&b1b1.primary, legacy_binding, typed.nodes()),
        ] {
            assert!(
                matches!(
                    mizar_checker::source_term::SourcePrimaryTermProducer::build(
                        task258b3m2a_primary_input(primary),
                        bindings,
                        arena,
                    ),
                    Err(mizar_checker::source_term::SourcePrimaryTermError::InvalidTransaction)
                ),
                "family {ordinal} lower handoff order"
            );
        }
        assert_eq!(typed.debug_text(), legacy_baseline);
        assert_eq!(format!("{b1b1:?}"), b1b1_baseline);
        let replay = source_statement_b3m2b2b1b1_output_with_resolver_mutation(
            &b1b1_ast,
            b1b1_module.clone(),
            &b1b1_symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
            |symbols| symbols,
        )
        .expect("wrapped replay extraction")
        .expect("wrapped replay lower capture");
        assert_eq!(format!("{replay:?}"), b1b1_baseline);
        assert_eq!(
            (
                b1b1_ast.snapshot_text(),
                b1b1_module.clone(),
                b1b1_symbols.clone()
            ),
            before
        );
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
    let plan = build_test_plan(&config).expect("Task258B3M2B2B1B1 isolation plan");
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
        if extract_wrapped_application_witness_source_statement(&ast, &source).is_some()
            || source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
                .is_some_and(|result| {
                    result.is_ok_and(|output| {
                        output
                            .typed_ast
                            .source_application()
                            .is_some_and(|application| application.wrappers().len() == 1)
                            && output.typed_ast.source_statement_witnesses().is_some()
                    })
                })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M2B2B1B1 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b2b1b1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, symbols) = task258b3m2b2b1b1_fixture(259_020);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let lower = source_statement_b3m2b2b1b1_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        |symbols| symbols,
    )
    .expect("extraction")
    .expect("authenticated lower capture");
    let baseline = format!("{lower:?}");
    assert_eq!(format!("{:?}", lower.clone()), baseline);
    assert!(lower.structure.is_none());
    assert!(lower.set_term.is_none());
    assert!(lower.witness.names.is_empty());
    assert!(lower.binding_env.diagnostics().is_empty());
    let mut changed = None;
    assert_eq!(
        source_statement_b3m2b2b1b1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
            |input| {
                input.witness.witnesses.clear();
                changed = Some(input.clone());
            },
        )
        .expect("rollback extraction")
        .expect_err("retired raw output"),
        "source statement dependency mismatch"
    );
    assert!(
        changed
            .expect("actual changed draft")
            .witness
            .witnesses
            .is_empty()
    );
    assert_eq!(lower.witness.witnesses.len(), 1);
    assert_eq!(format!("{lower:?}"), baseline);
    let replay = source_statement_b3m2b2b1b1_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        |symbols| symbols,
    )
    .expect("pristine extraction")
    .expect("pristine lower capture");
    assert_eq!(format!("{replay:?}"), baseline);
    assert!(
        source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        )
        .is_none()
    );
    assert_eq!((ast.snapshot_text(), module, symbols), before);
}
