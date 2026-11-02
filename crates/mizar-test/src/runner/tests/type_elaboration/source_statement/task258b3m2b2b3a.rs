#[test]
fn task258b3m2b2b3a_real_frontend_freezes_set_enumeration_witness_contract() {
    use mizar_checker::{
        source_set_term::{
            SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermId,
            SourceSetTermKind, SourceSetTermRecovery,
        },
        source_statement::{
            SourceStatementId, SourceStatementKind, SourceStatementRecovery,
            SourceStatementWitnessId, SourceStatementWitnessKind, SourceStatementWitnessTermTarget,
        },
        source_term::{SourcePrimaryTermId, SourcePrimaryTermKind},
    };

    assert_eq!(SOURCE_STATEMENT_B3M2B2B3A_TEXT.len(), 117);
    assert!(SOURCE_STATEMENT_B3M2B2B3A_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B3A_TEXT),
        "4f8ea5b9cadf763ea108b6f7deb6b481cb6f997dec2048b4351f07fd5dc38539"
    );
    let (ast, module, symbols, output) = task258b3m2b2b3a_output(259_400);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (57, Some(56))
    );
    let extracted: SourceStatementB3M2B2B3AExtraction =
        extract_set_enumeration_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B3A_TEXT)
            .expect("exact extraction");
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
            53,
            range(ast.source_id, 19, 116),
            range(ast.source_id, 27, 69),
            43,
            range(ast.source_id, 85, 97),
            42,
            range(ast.source_id, 90, 96),
            40,
            range(ast.source_id, 77, 115),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [30, 32, 36, 38, 44, 46]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [34, 48]
    );

    let resolver =
        source_statement_b3m2b2b3a_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("resolver");
    assert_eq!(resolver.module_id(), &module);
    assert!(resolver.imports().is_empty());
    assert_eq!(resolver.labels().len(), 1);
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver
        .symbols()
        .visible_candidates(&namespace, "FormulaStatementSetEnumerationWitnessSmoke");
    let [owner] = owners.as_slice() else {
        panic!("one exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 19, 116))
    );
    assert_eq!(owner.origin().structural_path(), [2, 1]);
    assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(owner.kind(), mizar_resolve::env::SymbolKind::Theorem);
    assert_eq!(owner.origin().source_id(), ast.source_id);
    assert_eq!(owner.origin().module_id(), &module);
    assert!(owner.origin().import_edge().is_none());
    assert!(!owner.origin().is_recovered());
    let definition = resolver
        .definitions()
        .by_symbol(owner.symbol())
        .expect("theorem definition");
    assert_eq!(
        definition.kind(),
        mizar_resolve::env::DefinitionKind::Theorem
    );
    assert_eq!(definition.symbol(), owner.symbol());
    assert_eq!(
        definition.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert!(definition.parameters().is_empty());
    assert!(definition.binders().is_empty());
    assert!(definition.arity().is_none());
    assert!(definition.notation_shape().is_none());
    assert!(definition.doc_attachment().is_none());
    assert_eq!(definition.origin(), owner.origin());
    assert_eq!(definition.contribution(), owner.contribution());
    assert!(definition.conflict().is_none());
    assert!(definition.dependencies().is_empty());
    assert!(matches!(
        definition.signature(),
        Some(mizar_resolve::env::SignatureShell::Opaque { schema, payload })
            if schema == "parser-signature-v1"
                && payload
                    == "node=TheoremItem;symbol=theorem;definition=theorem;\
    primary_tokens=theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; \
    thus x = x ; end ;;notation=_;arity=_;roles=FormulaExpression,ProofBlock"
    ));
    let contribution = resolver
        .contributions()
        .get(owner.contribution())
        .expect("owner contribution");
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::LocalSource { source_id }
            if *source_id == ast.source_id
    ));
    assert_eq!(contribution.module(), &module);
    assert_eq!(
        contribution.anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 0, 18))
    );
    assert!(contribution.effects().symbols().contains(owner.symbol()));
    assert!(
        contribution
            .effects()
            .definitions()
            .contains(&definition.id())
    );
    assert!(contribution.effects().imports().is_empty());
    let labels = resolver
        .labels()
        .visible_candidates(&namespace, "FormulaStatementSetEnumerationWitnessSmoke");
    let [label] = labels.as_slice() else {
        panic!("one exact theorem label");
    };
    assert_eq!(
        label.kind(),
        mizar_resolve::resolved_ast::LabelKind::Theorem
    );
    assert_eq!(label.namespace(), &namespace);
    assert_eq!(
        label.primary_spelling(),
        "FormulaStatementSetEnumerationWitnessSmoke"
    );
    assert_eq!(
        label.origin_path().as_str(),
        format!(
            "{}::{}::theorem::FormulaStatementSetEnumerationWitnessSmoke",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
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
    let contribution_labels = resolver.labels().by_contribution(owner.contribution());
    assert_eq!(contribution_labels.len(), 1);
    assert_eq!(contribution_labels[0], *label);
    assert_eq!(
        contribution.effects().labels(),
        [label.origin_path().clone()]
    );

    let statements = output.typed_ast.source_statement().expect("Task258");
    let bindings = statements.binding_env();
    assert_eq!(bindings.source_id(), ast.source_id);
    assert_eq!(bindings.module_id(), &module);
    assert_eq!(
        (
            bindings.contexts().len(),
            bindings.bindings().len(),
            bindings.diagnostics().len(),
        ),
        (2, 1, 0)
    );
    let module_context = bindings
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("module context");
    assert!(matches!(
        module_context.owner,
        mizar_checker::binding_env::BindingContextOwner::Module
    ));
    assert_eq!(module_context.parent, None);
    assert_eq!(
        module_context.layer,
        mizar_checker::binding_env::BindingContextLayer::Module
    );
    assert!(module_context.lexical_scope.is_none());
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
        .expect("proof context");
    assert!(matches!(
        proof_context.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
            if source_range == range(ast.source_id, 77, 115)
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
        .expect("reserved binding");
    assert_eq!(binding.id.index(), 0);
    assert_eq!(binding.spelling, "x");
    assert_eq!(
        binding.kind,
        mizar_checker::binding_env::BindingKind::ReservedVariable
    );
    assert!(matches!(
        &binding.identity,
        mizar_checker::binding_env::BinderIdentity::ReservedVariable {
            spelling,
            declaration_range,
        } if spelling == "x" && *declaration_range == range(ast.source_id, 8, 9)
    ));
    assert_eq!(
        binding.owner_context,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    assert_eq!(binding.declaration_range, range(ast.source_id, 8, 9));
    assert_eq!(binding.visible_after_ordinal, 0);
    assert_eq!(
        binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(range(ast.source_id, 14, 17))
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

    let primary = output.typed_ast.source_term().expect("Task252");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (6, 4, 2)
    );
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, term)| (
                id.index(),
                term.site().node().index(),
                term.context().index(),
                term.kind()
            ))
            .collect::<Vec<_>>(),
        [
            (0, 30, 0, SourcePrimaryTermKind::VariableReference),
            (1, 32, 0, SourcePrimaryTermKind::VariableReference),
            (2, 36, 1, SourcePrimaryTermKind::Numeral),
            (3, 38, 1, SourcePrimaryTermKind::Numeral),
            (4, 44, 1, SourcePrimaryTermKind::VariableReference),
            (5, 46, 1, SourcePrimaryTermKind::VariableReference),
        ]
    );
    assert_eq!(primary.source_id(), ast.source_id);
    assert_eq!(primary.module_id(), &module);
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(_, term)| (
                term.source_range(),
                term.source_ordinal(),
                term.recovery(),
                term.spelling(),
                term.kind(),
                term.role(),
                term.parent(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                range(ast.source_id, 71, 72),
                0,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 75, 76),
                1,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 91, 92),
                2,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "1",
                SourcePrimaryTermKind::Numeral,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 94, 95),
                3,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "2",
                SourcePrimaryTermKind::Numeral,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 105, 106),
                4,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 109, 110),
                5,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| (
                row.term().index(),
                row.binding().index(),
                row.role(),
                row.lexical_scope()
                    .map(mizar_resolve::names::LocalTermScope::path),
                row.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                None,
                1,
            ),
            (
                1,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                None,
                1,
            ),
            (
                4,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                Some(&[0][..]),
                1,
            ),
            (
                5,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                Some(&[0][..]),
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
                row.source_range(),
                row.spelling(),
                row.request_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [
            (2, 36, range(ast.source_id, 91, 92), "1", 0),
            (3, 38, range(ast.source_id, 94, 95), "2", 1),
        ]
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
        (1, 0, 0, 0, 0, 2, 1)
    );
    assert_eq!(set.source_id(), ast.source_id);
    assert_eq!(set.module_id(), &module);
    let set_row = set
        .terms()
        .get(SourceSetTermId::new(0))
        .expect("set enumeration");
    assert_eq!(set_row.site().node().index(), 40);
    assert_eq!(set_row.source_range(), range(ast.source_id, 90, 96));
    assert_eq!(set_row.context().index(), 1);
    assert_eq!(set_row.source_ordinal(), 0);
    assert_eq!(set_row.recovery(), SourceSetTermRecovery::Normal);
    assert_eq!(set_row.spelling(), "{ 1 , 2 }");
    assert_eq!(set_row.kind(), SourceSetTermKind::Enumeration);
    assert_eq!(
        set.edges()
            .iter()
            .map(|(_, edge)| (
                edge.term().index(),
                edge.ordinal(),
                edge.role(),
                edge.target(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                SourceSetEdgeRole::EnumerationElement,
                SourceSetTarget::Primary(SourcePrimaryTermId::new(2))
            ),
            (
                0,
                1,
                SourceSetEdgeRole::EnumerationElement,
                SourceSetTarget::Primary(SourcePrimaryTermId::new(3))
            ),
        ]
    );
    let request = set.requests().iter().next().expect("request").1;
    assert_eq!(request.term(), SourceSetTermId::new(0));
    assert_eq!(request.ordinal(), 0);
    assert_eq!(request.kind(), SourceSetRequestKind::ResultType);
    assert!(request.generator().is_none());
    assert!(request.type_site().is_none());
    assert_eq!(set.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(set.application_fingerprint(), None);
    assert_eq!(set.structure_fingerprint(), None);

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
    assert_eq!(atomic.source_id(), ast.source_id);
    assert_eq!(atomic.module_id(), &module);
    assert_eq!(atomic.primary_term_fingerprint(), primary.debug_text());
    assert!(atomic.application_fingerprint().is_none());
    assert!(atomic.structure_fingerprint().is_none());
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.source_range(),
                row.source_ordinal(),
                row.context().index(),
                row.recovery(),
                row.kind(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                34,
                range(ast.source_id, 71, 76),
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
                "x = x",
            ),
            (
                48,
                range(ast.source_id, 105, 110),
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
                "x = x",
            ),
        ]
    );
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, row)| (
                row.formula().index(),
                row.ordinal(),
                row.role(),
                row.target(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    SourcePrimaryTermId::new(0),
                ),
            ),
            (
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    SourcePrimaryTermId::new(1),
                ),
            ),
            (
                1,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    SourcePrimaryTermId::new(4),
                ),
            ),
            (
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    SourcePrimaryTermId::new(5),
                ),
            ),
        ]
    );
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
    assert_eq!(atomic.set_term_fingerprint(), None);

    assert_eq!(statements.source_id(), ast.source_id);
    assert_eq!(statements.module_id(), &module);
    assert_eq!(statements.binding_fingerprint(), bindings.debug_text());
    assert_eq!(statements.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(statements.atomic_formula_fingerprint(), atomic.debug_text());
    assert_eq!(
        (
            statements.owners().len(),
            statements.statements().len(),
            statements.contexts().len(),
            statements.input_facts().len(),
            statements.candidate_facts().len(),
        ),
        (1, 2, 2, 2, 2)
    );
    let checked_owner = statements.checked_owner();
    let owner_row = statements
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("Task258 owner");
    assert_eq!(owner_row.symbol(), checked_owner.symbol());
    assert_eq!(owner_row.symbol(), owner.symbol());
    assert_eq!(owner_row.contribution(), owner.contribution());
    assert_eq!(owner_row.site().node().index(), 53);
    assert_eq!(owner_row.source_range(), range(ast.source_id, 19, 116));
    assert_eq!(
        owner_row.spelling(),
        "FormulaStatementSetEnumerationWitnessSmoke"
    );
    assert_eq!(
        owner_row.role(),
        mizar_checker::source_statement::SourceTheoremRole::Theorem
    );
    assert_eq!(
        owner_row.status(),
        mizar_checker::source_statement::SourceTheoremStatus::Unmodified
    );
    assert_eq!(
        owner_row.recovery(),
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    assert_eq!(checked_owner.source_range(), owner_row.source_range());
    assert_eq!(checked_owner.origin(), owner.origin());
    assert_eq!(checked_owner.visibility(), owner.visibility());
    assert_eq!(checked_owner.export_status(), owner.export_status());
    assert_eq!(
        statements
            .statements()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.site().node().index(),
                row.context().index(),
                row.kind()
            ))
            .collect::<Vec<_>>(),
        [
            (0, 53, 0, SourceStatementKind::TheoremProposition),
            (1, 51, 1, SourceStatementKind::Conclusion),
        ]
    );
    for index in 0..2 {
        let row = statements
            .statements()
            .get(SourceStatementId::new(index))
            .expect("statement row");
        assert_eq!(row.owner().index(), 0);
        assert_eq!(row.context().index(), index);
        assert_eq!(
            row.formula(),
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(row.site().node().index(), [53, 51][index]);
        assert_eq!(
            row.source_range(),
            [
                range(ast.source_id, 19, 116),
                range(ast.source_id, 100, 111),
            ][index]
        );
        assert_eq!(row.source_ordinal(), [0, 2][index]);
        assert_eq!(
            row.spelling(),
            [
                "theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;",
                "thus x = x ;",
            ][index]
        );
        assert_eq!(
            row.kind(),
            [
                SourceStatementKind::TheoremProposition,
                SourceStatementKind::Conclusion,
            ][index]
        );
        assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
        let context = statements
            .contexts()
            .get(mizar_checker::source_statement::SourceStatementContextId::new(index))
            .expect("statement context");
        assert_eq!(context.statement().index(), index);
        assert_eq!(context.binding_context().index(), index);
        assert_eq!(context.source_range(), row.source_range());
        assert_eq!(
            context.visible_bindings(),
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        let input_fact = statements
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(input_fact.statement().index(), index);
        assert_eq!(input_fact.context().index(), index);
        assert_eq!(input_fact.ordinal(), 0);
        assert_eq!(
            input_fact.kind(),
            mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard
        );
        assert_eq!(input_fact.binding().index(), 0);
        assert_eq!(
            input_fact
                .uses()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![2, 3] }
        );
        let candidate = statements
            .candidate_facts()
            .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(index))
            .expect("candidate fact");
        assert_eq!(candidate.statement().index(), index);
        assert_eq!(candidate.context().index(), index);
        assert_eq!(candidate.ordinal(), 0);
        assert_eq!(
            candidate.kind(),
            mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate.formula(), row.formula());
    }

    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("B3A witness");
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
    assert_eq!(witness.owner().index(), 0);
    assert_eq!(witness.take_site().node().index(), 43);
    assert_eq!(witness.take_range(), range(ast.source_id, 85, 97));
    assert_eq!(witness.site().node().index(), 42);
    assert_eq!(witness.source_range(), range(ast.source_id, 90, 96));
    assert_eq!(witness.binding_context().index(), 1);
    assert_eq!(witness.source_ordinal(), 1);
    assert_eq!(witness.ordinal(), 0);
    assert_eq!(witness.spelling(), "{ 1 , 2 }");
    assert_eq!(witness.kind(), SourceStatementWitnessKind::Unnamed);
    assert_eq!(witness.recovery(), SourceStatementRecovery::Normal);
    assert!(witness.name().is_none());
    assert_eq!(witnesses.source_id(), ast.source_id);
    assert_eq!(witnesses.module_id(), &module);
    assert_eq!(witnesses.statement_fingerprint(), statements.debug_text());
    assert_eq!(witnesses.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(witnesses.structure_fingerprint(), None);
    assert_eq!(
        witnesses.set_term_fingerprint(),
        Some(set.debug_text().as_str())
    );
    assert_eq!(output.reference_use_ordinals, [1; 4]);

    let owned = output
        .typed_ast
        .nodes()
        .iter()
        .filter(|(_, node)| node.kind.as_str() != "source.surface.unowned")
        .map(|(id, node)| (id.index(), node.kind.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        owned,
        [
            (30, "source.term.variable-reference"),
            (32, "source.term.variable-reference"),
            (34, "source.formula.atomic.equality"),
            (36, "source.term.numeral"),
            (38, "source.term.numeral"),
            (40, "source.term.set.enumeration"),
            (42, "source.statement-witness.item"),
            (43, "source.statement-witness.take"),
            (44, "source.term.variable-reference"),
            (46, "source.term.variable-reference"),
            (48, "source.formula.atomic.equality"),
            (51, "source.statement.conclusion"),
            (53, "source.statement.theorem"),
        ]
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
fn task258b3m2b2b3a_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols, baseline) = task258b3m2b2b3a_output(259_401);
    let (_, _, _, stale) = task258b3m2b2b3a_output(259_402);
    for mutation in 0..5 {
        let error = source_statement_b3m2b2b3a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            |input: &mut SourceStatementB3M2B2B3ARouteInputs| match mutation {
                0 => {
                    input.arena = mizar_checker::typed_ast::TypedArena::try_new(None, Vec::new())
                        .expect("empty arena")
                }
                1 => input.primary = stale.typed_ast.source_term().expect("primary").clone(),
                2 => {
                    input.atomic = stale
                        .typed_ast
                        .source_atomic_formula()
                        .expect("atomic")
                        .clone()
                }
                3 => {
                    input.set_term =
                        Some(stale.typed_ast.source_set_term().expect("set term").clone())
                }
                4 => {
                    input.binding_env = stale
                        .typed_ast
                        .source_statement()
                        .expect("statement")
                        .binding_env()
                        .clone()
                }
                _ => unreachable!(),
            },
        )
        .expect("selector")
        .expect_err("stale lower dependency must fail");
        assert!(
            error.to_ascii_lowercase().contains("dependency"),
            "{mutation}: {error}"
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
        let error = source_statement_b3m2b2b3a_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("B3A resolver selector")
        .expect_err("B3A resolver mutation must fail");
        assert!(
            error.starts_with("Task258B3M2B2B3A resolver"),
            "{mutation:?}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        )
        .expect("B3A resolver replay selector")
        .expect("B3A resolver replay");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline.typed_ast.debug_text(),
            "{mutation:?} typed replay"
        );
        assert_eq!(
            replay.resolved.debug_text(),
            baseline.resolved.debug_text(),
            "{mutation:?} resolved replay"
        );
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
        let error = source_statement_b3m2b2b3a_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            |symbols| task258b3m2b2b3a_mutate_label_resolver(symbols, mutation),
        )
        .expect("B3A label resolver selector")
        .expect_err("B3A label resolver mutation must fail");
        assert!(
            error.starts_with("Task258B3M2B2B3A resolver"),
            "{mutation:?}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        )
        .expect("B3A label resolver replay selector")
        .expect("B3A label resolver replay");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline.typed_ast.debug_text(),
            "{mutation:?} typed replay"
        );
        assert_eq!(
            replay.resolved.debug_text(),
            baseline.resolved.debug_text(),
            "{mutation:?} resolved replay"
        );
    }

    let (_, lower_module, lower_shells, lower_symbols, lower_diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            259_401,
        );
    assert_eq!(lower_diagnostics, 0);
    assert_eq!(lower_module, module);
    let binding_env = baseline
        .typed_ast
        .source_statement()
        .expect("statement")
        .binding_env();
    let run_lower = |options| {
        set_enumeration_proof_context_handoff_for_test(
            &ast,
            &module,
            &lower_shells,
            &lower_symbols,
            binding_env,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            options,
        )
    };
    let lower_replay = || {
        let replay = run_lower(SetEnumerationProofContextTestOptions::default())
            .expect("B3P replay selector")
            .expect("B3P replay");
        assert_eq!(
            replay.typed_ast.source_set_term(),
            baseline.typed_ast.source_set_term()
        );
    };
    lower_replay();
    for field in 0..TASK258B3M2B2B3P_RESOLVER_FIELD_COUNT {
        assert!(
            run_lower(SetEnumerationProofContextTestOptions {
                resolver: SetEnumerationResolverMutation::Field(field),
                ..Default::default()
            })
            .is_none(),
            "B3P resolver field {field}"
        );
        lower_replay();
    }
    for field in 0..TASK258B3M2B2B3P_BINDING_FIELD_COUNT {
        let error = run_lower(SetEnumerationProofContextTestOptions {
            binding: SetEnumerationBindingMutation::Field(field),
            ..Default::default()
        })
        .expect("Task48 selection")
        .expect_err("Task48 mutation");
        assert!(error.starts_with("Task48:"), "{field}: {error}");
        lower_replay();
    }
    let mut primary_mutations = vec![
        SetEnumerationPrimaryMutation::DuplicateRoot,
        SetEnumerationPrimaryMutation::MissingRoot,
        SetEnumerationPrimaryMutation::SourceId,
        SetEnumerationPrimaryMutation::ModuleId,
        SetEnumerationPrimaryMutation::ReferenceScopeModule,
        SetEnumerationPrimaryMutation::ReferenceScopeProof,
        SetEnumerationPrimaryMutation::ReferenceUseOrdinal,
        SetEnumerationPrimaryMutation::StaleFingerprintReplay,
    ];
    for term in 0..6 {
        primary_mutations.extend([
            SetEnumerationPrimaryMutation::TermSite(term),
            SetEnumerationPrimaryMutation::TermRange(term),
            SetEnumerationPrimaryMutation::TermOrdinal(term),
            SetEnumerationPrimaryMutation::TermContext(term),
            SetEnumerationPrimaryMutation::TermRecovery(term),
            SetEnumerationPrimaryMutation::TermSpelling(term),
            SetEnumerationPrimaryMutation::TermKind(term),
            SetEnumerationPrimaryMutation::TermRole(term),
            SetEnumerationPrimaryMutation::TermParent(term),
        ]);
    }
    for reference in 0..4 {
        primary_mutations.extend([
            SetEnumerationPrimaryMutation::ReferenceTerm(reference),
            SetEnumerationPrimaryMutation::ReferenceBinding(reference),
            SetEnumerationPrimaryMutation::ReferenceRole(reference),
        ]);
    }
    for request in 0..2 {
        primary_mutations.extend([
            SetEnumerationPrimaryMutation::NumericTerm(request),
            SetEnumerationPrimaryMutation::NumericOwner(request),
            SetEnumerationPrimaryMutation::NumericRange(request),
            SetEnumerationPrimaryMutation::NumericSpelling(request),
            SetEnumerationPrimaryMutation::NumericOrdinal(request),
        ]);
    }
    for primary in primary_mutations {
        let error = run_lower(SetEnumerationProofContextTestOptions {
            primary,
            ..Default::default()
        })
        .expect("Task252 selection")
        .expect_err("Task252 mutation");
        assert!(
            error.starts_with("Task252:")
                || (primary == SetEnumerationPrimaryMutation::StaleFingerprintReplay
                    && error.starts_with("TypedAst:")),
            "{primary:?}: {error}"
        );
        lower_replay();
    }
    let mut set_mutations = vec![
        SetEnumerationHandoffMutation::SourceId,
        SetEnumerationHandoffMutation::ModuleId,
        SetEnumerationHandoffMutation::TermSite,
        SetEnumerationHandoffMutation::TermRange,
        SetEnumerationHandoffMutation::TermOrdinal,
        SetEnumerationHandoffMutation::TermContext,
        SetEnumerationHandoffMutation::TermRecovery,
        SetEnumerationHandoffMutation::TermSpelling,
        SetEnumerationHandoffMutation::TermKind,
        SetEnumerationHandoffMutation::ExtraWrapper,
        SetEnumerationHandoffMutation::ExtraGenerator,
        SetEnumerationHandoffMutation::ExtraTypeSite,
        SetEnumerationHandoffMutation::ExtraCondition,
        SetEnumerationHandoffMutation::RequestTerm,
        SetEnumerationHandoffMutation::RequestOrdinal,
        SetEnumerationHandoffMutation::RequestKind,
        SetEnumerationHandoffMutation::RequestGenerator,
        SetEnumerationHandoffMutation::RequestTypeSite,
        SetEnumerationHandoffMutation::CoherentApplicationFingerprint,
        SetEnumerationHandoffMutation::CoherentStructureFingerprint,
    ];
    for edge in 0..2 {
        set_mutations.extend([
            SetEnumerationHandoffMutation::EdgeTerm(edge),
            SetEnumerationHandoffMutation::EdgeOrdinal(edge),
            SetEnumerationHandoffMutation::EdgeRole(edge),
            SetEnumerationHandoffMutation::EdgeTarget(edge),
        ]);
    }
    for handoff in set_mutations {
        let error = run_lower(SetEnumerationProofContextTestOptions {
            handoff,
            ..Default::default()
        })
        .expect("Task255 selection")
        .expect_err("Task255 mutation");
        assert!(error.starts_with("Task255:"), "{handoff:?}: {error}");
        lower_replay();
    }
    for (stage, count, prefix) in [
        (
            SourceStatementB3M2B2B3AStage::Task256,
            TASK258B3M2B2B3A_TASK256_FIELD_COUNT,
            "Task256:",
        ),
        (
            SourceStatementB3M2B2B3AStage::Task258,
            TASK258B3M2B2B3A_TASK258_FIELD_COUNT,
            "Task258:",
        ),
        (
            SourceStatementB3M2B2B3AStage::Witness,
            TASK258B3M2B2B3A_WITNESS_FIELD_COUNT,
            "B3A:",
        ),
    ] {
        for field in 0..count {
            let error = source_statement_b3m2b2b3a_output_with_stage_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3A_TEXT,
                stage,
                field,
            )
            .expect("stage selector")
            .expect_err("stage field mutation");
            assert!(
                error.starts_with(prefix),
                "{stage:?} field {field}: {error}"
            );
            let replay = source_statement_output_with_source(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            )
            .expect("stage replay selector")
            .expect("stage replay");
            assert_eq!(
                replay.typed_ast.debug_text(),
                baseline.typed_ast.debug_text(),
                "{stage:?} field {field} typed replay"
            );
            assert_eq!(
                replay.resolved.debug_text(),
                baseline.resolved.debug_text(),
                "{stage:?} field {field} resolved replay"
            );
        }
    }
    let error = source_statement_b3m2b2b3a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        |input| {
            input.set_term = None;
            input.statement.owners.clear();
            input.witness.witnesses.clear();
        },
    )
    .expect("precedence selector")
    .expect_err("lower dependency must precede upper aggregates");
    assert!(error.to_ascii_lowercase().contains("dependency"), "{error}");
    for mutation in 0..4 {
        assert!(
            source_statement_b3m2b2b3a_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3A_TEXT,
                |input| match mutation {
                    0 => input.statement.owners.clear(),
                    1 => input.statement.statements.clear(),
                    2 => input.witness.witnesses.clear(),
                    3 => input.witness.names.push(
                        mizar_checker::source_statement::SourceStatementWitnessNameInput {
                            witness: mizar_checker::source_statement::SourceStatementWitnessId::new(
                                0,
                            ),
                            site: input.witness.witnesses[0].site.clone(),
                            source_range: input.witness.witnesses[0].source_range,
                            spelling: "y".to_owned(),
                            recovery:
                                mizar_checker::source_statement::SourceStatementRecovery::Normal,
                        },
                    ),
                    _ => unreachable!(),
                },
            )
            .expect("selector")
            .is_err(),
            "upper mutation {mutation}"
        );
    }
    let replay = source_statement_output_with_source(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
    )
    .expect("replay selector")
    .expect("replay");
    assert_eq!(
        replay.typed_ast.debug_text(),
        baseline.typed_ast.debug_text()
    );
    assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
}

#[test]
fn task258b3m2b2b3a_set_enumeration_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            259_403,
        );
    assert_eq!(diagnostics, 0);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
    )
    .expect("B3A exact selector")
    .expect("B3A exact output");
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B3A_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B3A_TEXT.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' {
            b'?'
        } else {
            b'!'
        };
        let changed = String::from_utf8(bytes).expect("ASCII");
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &changed).is_none(),
            "byte {byte_index}"
        );
    }
    for node in 0..57 {
        for mutation in [
            SourceStatementB3M2B2B3ASurfaceMutation::NodeKind(node),
            SourceStatementB3M2B2B3ASurfaceMutation::NodeRange(node),
            SourceStatementB3M2B2B3ASurfaceMutation::NodeRecovery(node),
            SourceStatementB3M2B2B3ASurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                extract_set_enumeration_witness_source_statement_with_surface_mutation(
                    &ast,
                    SOURCE_STATEMENT_B3M2B2B3A_TEXT,
                    mutation,
                )
                .is_none(),
                "node {node} {mutation:?}"
            );
        }
    }
    assert!(
        extract_set_enumeration_witness_source_statement_with_surface_mutation(
            &ast,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            SourceStatementB3M2B2B3ASurfaceMutation::RootIdentity,
        )
        .is_none()
    );
    for (label, changed) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B3M2B2B3A_TEXT
                .trim_end_matches('\n')
                .to_owned(),
        ),
        (
            "extra final LF",
            format!("{SOURCE_STATEMENT_B3M2B2B3A_TEXT}\n"),
        ),
    ] {
        assert!(
            extract_set_enumeration_witness_source_statement(&ast, &changed).is_none(),
            "{label} extractor"
        );
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &changed).is_none(),
            "{label} route"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        )
        .expect("final-LF replay selector")
        .expect("final-LF replay");
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
    let changed_label = SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen(
        "FormulaStatementSetEnumerationWitnessSmoke",
        "FormulaStatementSetEnumerationWitnessNearMiss",
        1,
    );
    assert!(
        extract_set_enumeration_witness_source_statement(&ast, &changed_label).is_none(),
        "label text is part of the B3A selector"
    );

    let near_misses = [
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "{}", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "{1}", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "{1, 2, 3}", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "({1, 2})", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "{{1}, {2}}", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "{x where x is set : x = x}", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "the set", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("{1, 2}", "{1, 2} qua set", 1),
        SOURCE_STATEMENT_B3M2B2B3A_TEXT.replacen("take {1, 2}", "take y = {1, 2}", 1),
    ];
    for (index, changed) in near_misses.into_iter().enumerate() {
        let (changed_ast, changed_module, _, changed_symbols, _) =
            task253_ast_from_source_text_with_diagnostic_count(&changed, 259_410 + index);
        assert!(
            source_statement_output_with_source(
                &changed_ast,
                changed_module,
                &changed_symbols,
                &changed,
            )
            .is_none(),
            "near miss {index}"
        );
    }
}

#[test]
fn task258b3m2b2b3a_family_and_active_route_isolation_is_atomic() {
    let (ast, module, symbols, b3a) = task258b3m2b2b3a_output(259_420);
    let (application_ast, application_module, application_symbols) =
        task258b3m2b2b1a_fixture(259_420);
    let application = source_statement_output_with_source(
        &application_ast,
        application_module.clone(),
        &application_symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    )
    .expect("application selector")
    .expect("application")
    .typed_ast
    .source_application()
    .expect("application handoff")
    .clone();
    let (structure_ast, structure_module, structure_symbols) = task258b3m2b2b2a_fixture(259_420);
    let structure = source_statement_output_with_source(
        &structure_ast,
        structure_module.clone(),
        &structure_symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
    )
    .expect("structure selector")
    .expect("structure")
    .typed_ast
    .source_structure()
    .expect("structure handoff")
    .clone();
    let set = b3a
        .typed_ast
        .source_set_term()
        .expect("set handoff")
        .clone();
    assert_eq!(application.source_id(), ast.source_id);
    assert_eq!(application.module_id(), &module);
    assert_eq!(structure.source_id(), ast.source_id);
    assert_eq!(structure.module_id(), &module);
    assert_eq!(set.source_id(), ast.source_id);
    assert_eq!(set.module_id(), &module);

    for tuple in 0_u8..8 {
        let application_member = application.clone();
        let structure_member = structure.clone();
        let set_member = set.clone();
        let before = source_statement_b3m2b2b3a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            move |input| {
                input.application = (tuple & 1 != 0).then_some(application_member);
                input.structure = (tuple & 2 != 0).then_some(structure_member);
                input.set_term = (tuple & 4 != 0).then_some(set_member);
            },
        )
        .expect("pre-auth tuple selector");
        if tuple == 4 {
            before.expect("the exact set-only tuple passes pre-auth");
        } else {
            let error = before.expect_err("non-B3A tuple fails before authentication");
            assert!(
                error.to_ascii_lowercase().contains("dependency"),
                "pre-auth tuple {tuple:03b}: {error}"
            );
        }

        let application_member = application.clone();
        let structure_member = structure.clone();
        let set_member = set.clone();
        let after = source_statement_b3m2b2b3a_output_with_post_auth_family_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            move |input| {
                input.application = (tuple & 1 != 0).then_some(application_member);
                input.structure = (tuple & 2 != 0).then_some(structure_member);
                input.set_term = (tuple & 4 != 0).then_some(set_member);
            },
        )
        .expect("post-auth tuple selector");
        if tuple == 4 {
            let output = after.expect("the exact set-only tuple passes after authentication");
            assert!(output.typed_ast.source_application().is_none());
            assert!(output.typed_ast.source_structure().is_none());
            assert!(output.typed_ast.source_set_term().is_some());
        } else {
            let error = after.expect_err("non-B3A tuple fails after authentication");
            assert!(
                error.to_ascii_lowercase().contains("dependency")
                    || error.to_ascii_lowercase().contains("mutually exclusive"),
                "post-auth tuple {tuple:03b}: {error}"
            );
        }
    }
    let error = source_statement_b3m2b2b1a_output_with_mutation(
        &application_ast,
        application_module,
        &application_symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        |input| input.set_term = Some(set.clone()),
    )
    .expect("application selector")
    .expect_err("application/set hybrid");
    assert!(error.to_ascii_lowercase().contains("dependency"), "{error}");
    let error = source_statement_b3m2b2b2a_output_with_mutation(
        &structure_ast,
        structure_module,
        &structure_symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        |input| input.set_term = Some(set),
    )
    .expect("structure selector")
    .expect_err("structure/set hybrid");
    assert!(error.to_ascii_lowercase().contains("dependency"), "{error}");

    for (source, expects_application, expects_structure, expects_set) in [
        (SOURCE_STATEMENT_B3M2B2B1A_TEXT, true, false, false),
        (SOURCE_STATEMENT_B3M2B2B2A_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B2B_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B2C_TEXT, false, true, false),
        (SOURCE_STATEMENT_B3M2B2B3A_TEXT, false, false, true),
    ] {
        let (case_ast, case_module, _, case_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(source, 259_430 + source.len());
        assert_eq!(diagnostics, 0);
        let case_symbols =
            augment_type_elaboration_import_summaries(&case_ast, &case_module, case_symbols);
        let output =
            source_statement_output_with_source(&case_ast, case_module, &case_symbols, source)
                .expect("active selector")
                .expect("active route");
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
}

#[test]
fn task258b3m2b2b3a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, symbols, output) = task258b3m2b2b3a_output(259_440);
    let replay = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
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

    let baseline = output.typed_ast.debug_text();
    let error = source_statement_b3m2b2b3a_output_with_mutation(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
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
