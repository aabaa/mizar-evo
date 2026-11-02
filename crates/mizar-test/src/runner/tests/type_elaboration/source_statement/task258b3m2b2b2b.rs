#[test]
fn task258b3m2b2b2b_real_frontend_freezes_structure_selector_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B2B2B_TEXT.len(), 171);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B2B_TEXT),
        "63f4be4d458905ba01f7510798bb87783bb90c9e6f866044be5726ce35429d00"
    );
    let (ast, module, symbols) = task258b3m2b2b2b_fixture(259_200);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (79, Some(78))
    );
    let extracted: SourceStatementB3M2B2B2BExtraction =
        extract_structure_selector_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B2B_TEXT)
            .expect("Task258B3M2B2B2B exact extraction");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range,
            extracted.label_range,
            extracted.take_site.node().index(),
            extracted.take_range,
            extracted.witness_site.node().index(),
            extracted.witness_range,
            extracted.selector_node,
            extracted.proof_range,
        ),
        (
            75,
            range(ast.source_id, 48, 170),
            range(ast.source_id, 56, 101),
            65,
            range(ast.source_id, 117, 151),
            64,
            range(ast.source_id, 122, 150),
            62,
            range(ast.source_id, 109, 169),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [47, 49, 56, 59, 66, 68]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [51, 70]
    );
    let resolver =
        source_statement_b3m2b2b2b_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("Task258B3M2B2B2B resolver");
    assert_eq!((resolver.imports().len(), resolver.labels().len()), (1, 1));
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver
        .symbols()
        .visible_candidates(&namespace, "FormulaStatementStructureSelectorWitnessSmoke");
    let [owner] = owners.as_slice() else {
        panic!("Task258B3M2B2B2B exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 48, 170))
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
        .expect("Task258B3M2B2B2B owner contribution");
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
    let labels = resolver
        .labels()
        .visible_candidates(&namespace, "FormulaStatementStructureSelectorWitnessSmoke");
    let [label] = labels.as_slice() else {
        panic!("Task258B3M2B2B2B exact theorem label");
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

    let output = task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols);
    let primary = &output.primary;
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
                term.context().index()
            ))
            .collect::<Vec<_>>(),
        [
            (0, 47, 0),
            (1, 49, 0),
            (2, 55, 1),
            (3, 58, 1),
            (4, 66, 1),
            (5, 68, 1)
        ]
    );
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(_, term)| (
                term.source_range().start,
                term.source_range().end,
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
                103,
                104,
                0,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                107,
                108,
                1,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                140,
                141,
                2,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "1",
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                146,
                147,
                3,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "2",
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                159,
                160,
                4,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                163,
                164,
                5,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
        ]
    );
    let bindings = &output.binding_env;
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
            if source_range == range(ast.source_id, 109, 169)
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
                row.source_range(),
                row.spelling(),
                row.request_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [
            (2, 55, range(ast.source_id, 140, 141), "1", 0),
            (3, 58, range(ast.source_id, 146, 147), "2", 1),
        ]
    );
    let atomic = &output.atomic;
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(_, formula)| formula.site().node().index())
            .collect::<Vec<_>>(),
        [51, 70]
    );
    assert_eq!((atomic.edges().len(), atomic.requests().len()), (4, 4));
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
    assert_eq!(atomic.structure_fingerprint(), None);
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.source_range(),
                row.source_ordinal(),
                row.context().index(),
                row.kind(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                51,
                range(ast.source_id, 103, 108),
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
                "x = x",
            ),
            (
                70,
                range(ast.source_id, 159, 164),
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
        mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Structure(_)
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
    let structure = output.structure.as_ref().expect("Task254 handoff");
    assert_eq!(
        (
            structure.terms().len(),
            structure.wrappers().len(),
            structure.roots().len(),
            structure.members().len(),
            structure.field_updates().len(),
            structure.edges().len(),
            structure.requests().len(),
        ),
        (2, 0, 1, 3, 0, 3, 9)
    );
    assert_eq!(
        structure
            .terms()
            .iter()
            .map(|(id, term)| (
                id.index(),
                term.site().node().index(),
                term.source_range(),
                term.source_ordinal(),
                term.kind(),
                term.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                62,
                range(ast.source_id, 122, 150),
                0,
                mizar_checker::source_structure::SourceStructureTermKind::SelectorAccess,
                "TypeCaseStruct ( x : 1 , y : 2 ) . x",
            ),
            (
                1,
                61,
                range(ast.source_id, 122, 148),
                1,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
                "TypeCaseStruct ( x : 1 , y : 2 )",
            ),
        ]
    );
    assert!(structure.terms().iter().all(|(_, term)| {
        term.context() == mizar_checker::binding_env::BindingContextId::new(1)
            && term.recovery() == mizar_checker::source_structure::SourceStructureRecovery::Normal
    }));
    assert!(structure.application_fingerprint().is_none());
    let structure_root = structure
        .roots()
        .get(mizar_checker::source_structure::SourceStructureRootId::new(
            0,
        ))
        .expect("imported root");
    assert_eq!(structure_root.term().index(), 1);
    assert_eq!(
        structure_root.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert_eq!(
        structure_root.symbol().local().as_str(),
        "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
    );
    assert_eq!(
        structure_root.symbol().fqn().as_str(),
        "parser.type_fixtures::TypeCaseStruct#5"
    );
    assert_eq!(structure_root.contribution().index(), 2);
    assert_eq!(structure_root.origin().source_id(), ast.source_id);
    assert_eq!(
        structure_root.origin().module_id(),
        structure_root.symbol().module()
    );
    assert_eq!(
        structure_root.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 7, 27))
    );
    assert_eq!(structure_root.origin().structural_path(), [5]);
    assert!(structure_root.origin().import_edge().is_none());
    assert!(!structure_root.origin().is_recovered());
    assert_eq!(
        structure_root.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        structure_root.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert!(structure_root.signature().is_none());
    assert_eq!(
        structure
            .members()
            .iter()
            .map(|(_, member)| (
                member.term().index(),
                member.ordinal(),
                member.site().node().index(),
                member.source_range(),
                member.spelling(),
                member.role(),
                member.parent(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                29,
                range(ast.source_id, 149, 150),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::Selector,
                None,
            ),
            (
                1,
                0,
                20,
                range(ast.source_id, 137, 138),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
            (
                1,
                1,
                24,
                range(ast.source_id, 143, 144),
                "y",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
        ]
    );
    assert_eq!(
        structure
            .edges()
            .iter()
            .map(|(_, edge)| (
                edge.term().index(),
                edge.ordinal(),
                edge.role(),
                edge.member().map(|member| member.index()),
                edge.target(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_structure::SourceStructureEdgeRole::SelectorBase,
                None,
                mizar_checker::source_structure::SourceStructureTarget::Structure(
                    mizar_checker::source_structure::SourceStructureTermId::new(1),
                ),
            ),
            (
                1,
                0,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                Some(1),
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
            (
                1,
                1,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                Some(2),
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                ),
            ),
        ]
    );
    assert_eq!(
        structure
            .requests()
            .iter()
            .map(|(_, request)| (
                request.term().index(),
                request.request_ordinal(),
                request.member().map(|member| member.index()),
                request.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                0,
                1,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                0,
                2,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ResultType,
            ),
            (
                1,
                0,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ConstructorSignature,
            ),
            (
                1,
                1,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                1,
                2,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                1,
                3,
                Some(2),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                1,
                4,
                Some(2),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                1,
                5,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ResultType,
            ),
        ]
    );
    let statement = &output.statement;
    assert_eq!(
        (
            statement.owners.len(),
            statement.statements.len(),
            statement.contexts.len(),
            statement.input_facts.len(),
            statement.candidate_facts.len(),
        ),
        (1, 2, 2, 2, 2)
    );
    let owner_row = statement
        .owners
        .first()
        .expect("Task258 owner");
    assert_eq!(&owner_row.symbol, owner.symbol());
    assert_eq!(owner_row.contribution, owner.contribution());
    assert_eq!(owner_row.site.node().index(), 75);
    assert_eq!(owner_row.source_range, range(ast.source_id, 48, 170));
    assert_eq!(
        owner_row.spelling,
        "FormulaStatementStructureSelectorWitnessSmoke"
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
        let row = statement
            .statements
            .get(index)
            .expect("Task258 statement row");
        assert_eq!(row.owner.index(), 0);
        assert_eq!(row.context.index(), index);
        assert_eq!(
            row.formula,
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(row.site.node().index(), [75, 73][index]);
        assert_eq!(
            row.source_range,
            [
                range(ast.source_id, 48, 170),
                range(ast.source_id, 154, 165),
            ][index]
        );
        assert_eq!(row.source_ordinal, [0, 2][index]);
        assert_eq!(
            row.spelling,
            [
                "theorem FormulaStatementStructureSelectorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) . x ; thus x = x ; end ;",
                "thus x = x ;",
            ][index]
        );
        assert_eq!(
            row.kind,
            [
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
            ][index]
        );
        assert_eq!(
            row.recovery,
            mizar_checker::source_statement::SourceStatementRecovery::Normal
        );
        let context = statement
            .contexts
            .get(index)
            .expect("Task258 context row");
        assert_eq!(context.statement.index(), index);
        assert_eq!(context.binding_context.index(), index);
        assert_eq!(context.source_range, row.source_range);
        assert_eq!(
            context.visible_bindings,
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        let input_fact = statement
            .input_facts
            .get(index)
            .expect("Task258 input fact");
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
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![2, 3] }
        );
        let candidate = statement
            .candidate_facts
            .get(index)
            .expect("Task258 candidate fact");
        assert_eq!(candidate.statement.index(), index);
        assert_eq!(candidate.context.index(), index);
        assert_eq!(candidate.ordinal, 0);
        assert_eq!(
            candidate.kind,
            mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate.formula, row.formula);
    }
    let witnesses = &output.witness;
    assert_eq!(
        (witnesses.witnesses.len(), witnesses.names.len()),
        (1, 0)
    );
    let witness = witnesses
        .witnesses
        .first()
        .expect("B2B witness");
    assert_eq!(
        witness.term,
        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Structure(
            mizar_checker::source_structure::SourceStructureTermId::new(0),
        )
    );
    assert_eq!(witness.owner.index(), 0);
    assert_eq!(witness.binding_context.index(), 1);
    assert_eq!(witness.take_site.node().index(), 65);
    assert_eq!(witness.take_range, range(ast.source_id, 117, 151));
    assert_eq!(witness.site.node().index(), 64);
    assert_eq!(witness.source_range, range(ast.source_id, 122, 150));
    assert_eq!(witness.source_ordinal, 1);
    assert_eq!(witness.ordinal, 0);
    assert_eq!(witness.spelling, "TypeCaseStruct ( x : 1 , y : 2 ) . x");
    assert_eq!(
        witness.kind,
        mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed
    );
    assert_eq!(
        witness.recovery,
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    assert!(witness.name.is_none());
    for (id, node) in output.arena.iter() {
        assert_eq!(
            node.kind.as_str(),
            match id.index() {
                20 | 24 => "source.term.structure.member.constructor-assignment",
                29 => "source.term.structure.member.selector",
                47 | 49 | 66 | 68 => "source.term.variable-reference",
                55 | 58 => "source.term.numeral",
                51 | 70 => "source.formula.atomic.equality",
                61 => "source.term.structure.constructor",
                62 => "source.term.structure.selector",
                64 => "source.statement-witness.item",
                65 => "source.statement-witness.take",
                73 => "source.statement.conclusion",
                75 => "source.statement.theorem",
                _ => "source.surface.unowned",
            },
            "node {} kind",
            id.index()
        );
    }
    assert_eq!(
        output
            .arena
            .iter()
            .filter(|(_, node)| node.kind.as_str().starts_with("source.term.structure."))
            .map(|(id, node)| (id.index(), node.kind.as_str()))
            .collect::<Vec<_>>(),
        [
            (20, "source.term.structure.member.constructor-assignment"),
            (24, "source.term.structure.member.constructor-assignment"),
            (29, "source.term.structure.member.selector"),
            (61, "source.term.structure.constructor"),
            (62, "source.term.structure.selector"),
        ]
    );
}

#[test]
fn task258b3m2b2b2b_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols) = task258b3m2b2b2b_fixture(259_201);
    let baseline = task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let baseline_debug = format!("{baseline:?}");
    let (stale_ast, stale_module, stale_symbols) = task258b3m2b2b2b_fixture(259_209);
    let stale = task258b3m2b2b2b_lower_inputs(&stale_ast, &stale_module, &stale_symbols);
    let stale_binding = stale.binding_env;
    let stale_primary = stale.primary;
    let stale_atomic = stale.atomic;
    let stale_structure = stale.structure.expect("stale structure");
    for mutation in 0..27 {
        let stale_binding = stale_binding.clone();
        let stale_primary = stale_primary.clone();
        let stale_atomic = stale_atomic.clone();
        let stale_structure = stale_structure.clone();
        let mut changed = None;
        let error = source_statement_b3m2b2b2b_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B2B_TEXT,
            |input: &mut SourceStatementB3M2B2B2BRouteInputs| {
                match mutation {
                0 => input.binding_env = stale_binding,
                1 => input.primary = stale_primary,
                2 => input.atomic = stale_atomic,
                3 => input.structure = None,
                4 => input.structure = Some(stale_structure),
                5 => input.statement.owners.clear(),
                6 => input.statement.owners[0].spelling.push('x'),
                7 => input.statement.statements[0].source_ordinal = 1,
                8 => input.statement.contexts[1].source_range.start += 1,
                9 => input.statement.input_facts[1].uses.swap(0, 1),
                10 => input.statement.candidate_facts[1].ordinal = 1,
                11 => input.witness.witnesses.clear(),
                12 => {
                    input.witness.witnesses[0].term =
                        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                            mizar_checker::source_term::SourcePrimaryTermId::new(2),
                        )
                }
                13 => {
                    input.witness.witnesses[0].term =
                        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Structure(
                            mizar_checker::source_structure::SourceStructureTermId::new(1),
                        )
                }
                14 => {
                    input.witness.witnesses[0].term = mizar_checker::source_statement::
                        SourceStatementWitnessTermTarget::Application(
                        mizar_checker::source_application::SourceFunctorApplicationId::new(0),
                    )
                }
                15 => {
                    input.witness.witnesses[0].owner =
                        mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                }
                16 => {
                    input.witness.witnesses[0].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0)
                }
                17 => input.witness.witnesses[0].take_site =
                    input.statement.statements[1].site.clone(),
                18 => input.witness.witnesses[0].take_range.start += 1,
                19 => input.witness.witnesses[0].site =
                    input.statement.statements[1].site.clone(),
                20 => input.witness.witnesses[0].source_range.end -= 1,
                21 => input.witness.witnesses[0].source_ordinal = 0,
                22 => input.witness.witnesses[0].ordinal = 1,
                23 => input.witness.witnesses[0].spelling.push('x'),
                24 => {
                    input.witness.witnesses[0].kind =
                        mizar_checker::source_statement::SourceStatementWitnessKind::Named
                }
                25 => {
                    input.witness.witnesses[0].recovery =
                        mizar_checker::source_statement::SourceStatementRecovery::Degraded
                }
                26 => {
                    input.witness.witnesses[0].name =
                        Some(mizar_checker::source_statement::SourceStatementWitnessNameId::new(0))
                }
                _ => unreachable!(),
                }
                changed = Some(input.clone());
            },
        )
        .expect("mutation selector")
        .expect_err("mutation must fail closed");
        assert_eq!(
            error,
            if mutation < 5 {
                "Task258B3M2B2B2B lower dependency mismatch"
            } else {
                "source statement dependency mismatch"
            },
            "mutation {mutation}"
        );
        let changed = changed.expect("actual mutation captured before authentication");
        assert_ne!(
            format!("{changed:?}"),
            baseline_debug,
            "mutation {mutation}"
        );
        // The Checker private suite applies these same mutations directly:
        // 0..2 Statement DependencyMismatch; 3..4 Witness DependencyMismatch;
        // 5 InvalidAggregate, 6 InvalidOwner(0), 7 InvalidStatement(0),
        // 8 InvalidContext(1), 9 InvalidInputFact(1), 10 InvalidCandidateFact(1);
        // 11 Witness InvalidAggregate; 12..26 Witness InvalidWitness(0).
        // Captured draft changes connect those guards to this runner handoff;
        // retired raw admission is not their causal oracle.
        if (5..11).contains(&mutation) {
            assert_ne!(
                changed.statement, baseline.statement,
                "statement mutation {mutation}"
            );
            assert_eq!(changed.witness, baseline.witness);
        } else if mutation >= 11 {
            assert_ne!(
                changed.witness, baseline.witness,
                "witness mutation {mutation}"
            );
            assert_eq!(changed.statement, baseline.statement);
        }
        assert_eq!(
            format!(
                "{:?}",
                task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols)
            ),
            baseline_debug
        );
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
    }

    let resolver = source_statement_b3m2b2b2b_resolver_env_for_test(
        &module,
        &symbols,
        range(ast.source_id, 56, 101),
    )
    .expect("resolver");
    let binding_env = &baseline.binding_env;
    let exact_seam = imported_structure_selector_handoff_for_test(
        &ast,
        &module,
        &resolver,
        binding_env,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        &task258b3m2b2b2b_roots(),
        ImportedStructureSelectorTestOptions {
            selector: 62,
            context: mizar_checker::binding_env::BindingContextId::new(1),
            surface_mutation: ImportedStructureSelectorSurfaceMutation::DirectProductionSeam,
            handoff_mutation: ImportedStructureSelectorTestMutation::None,
        },
    )
    .expect("private selector seam")
    .expect("exact private selector handoff");
    assert_eq!(exact_seam.primary_counts, (6, 4, 2));
    assert_eq!(
        exact_seam.handoff,
        *baseline.structure.as_ref().unwrap()
    );
    assert_eq!(
        exact_seam.typed_ast.source_structure(),
        exact_seam.resolved.source_structure()
    );

    let seam_handoff = exact_seam.handoff.debug_text();
    let seam_typed = exact_seam.typed_ast.debug_text();
    let seam_resolved = exact_seam.resolved.debug_text();
    let assert_lower_mutation = |mutation| {
        let result = imported_structure_selector_handoff_for_test(
            &ast,
            &module,
            &resolver,
            binding_env,
            SOURCE_STATEMENT_B3M2B2B2B_TEXT,
            &task258b3m2b2b2b_roots(),
            ImportedStructureSelectorTestOptions {
                selector: 62,
                context: mizar_checker::binding_env::BindingContextId::new(1),
                surface_mutation: ImportedStructureSelectorSurfaceMutation::None,
                handoff_mutation: mutation,
            },
        );
        assert!(
            !matches!(result, Some(Ok(_))),
            "selector mutation {mutation:?} succeeded"
        );
        let replay = imported_structure_selector_handoff_for_test(
            &ast,
            &module,
            &resolver,
            binding_env,
            SOURCE_STATEMENT_B3M2B2B2B_TEXT,
            &task258b3m2b2b2b_roots(),
            ImportedStructureSelectorTestOptions {
                selector: 62,
                context: mizar_checker::binding_env::BindingContextId::new(1),
                surface_mutation: ImportedStructureSelectorSurfaceMutation::None,
                handoff_mutation: ImportedStructureSelectorTestMutation::None,
            },
        )
        .expect("lower clean replay selector")
        .expect("lower clean replay");
        assert_eq!(replay.handoff.debug_text(), seam_handoff);
        assert_eq!(replay.typed_ast.debug_text(), seam_typed);
        assert_eq!(replay.resolved.debug_text(), seam_resolved);
    };
    for mutation in [
        ImportedStructureSelectorTestMutation::BindingSourceId,
        ImportedStructureSelectorTestMutation::BindingModuleId,
        ImportedStructureSelectorTestMutation::BindingContextCount,
        ImportedStructureSelectorTestMutation::BindingCount,
        ImportedStructureSelectorTestMutation::BindingDiagnosticCount,
        ImportedStructureSelectorTestMutation::ModuleContextOwner,
        ImportedStructureSelectorTestMutation::ModuleContextParent,
        ImportedStructureSelectorTestMutation::ModuleContextLayer,
        ImportedStructureSelectorTestMutation::ModuleContextScope,
        ImportedStructureSelectorTestMutation::ModuleContextBindings,
        ImportedStructureSelectorTestMutation::ModuleContextVisibleBindings,
        ImportedStructureSelectorTestMutation::ModuleContextRecovery,
        ImportedStructureSelectorTestMutation::ProofContextOwner,
        ImportedStructureSelectorTestMutation::ProofContextParent,
        ImportedStructureSelectorTestMutation::ProofContextLayer,
        ImportedStructureSelectorTestMutation::ProofContextScope,
        ImportedStructureSelectorTestMutation::ProofContextBindings,
        ImportedStructureSelectorTestMutation::ProofContextVisibleBindings,
        ImportedStructureSelectorTestMutation::ProofContextRecovery,
        ImportedStructureSelectorTestMutation::BindingSpelling,
        ImportedStructureSelectorTestMutation::BindingKind,
        ImportedStructureSelectorTestMutation::BindingIdentityKind,
        ImportedStructureSelectorTestMutation::BindingIdentitySpelling,
        ImportedStructureSelectorTestMutation::BindingIdentityRange,
        ImportedStructureSelectorTestMutation::BindingOwner,
        ImportedStructureSelectorTestMutation::BindingDeclarationRange,
        ImportedStructureSelectorTestMutation::BindingVisibleOrdinal,
        ImportedStructureSelectorTestMutation::BindingTypeSite,
        ImportedStructureSelectorTestMutation::BindingStatus,
        ImportedStructureSelectorTestMutation::BindingCaptured,
        ImportedStructureSelectorTestMutation::BindingDiagnostics,
        ImportedStructureSelectorTestMutation::BindingRecovery,
    ] {
        assert_lower_mutation(mutation);
    }
    let mut primary_mutations = vec![
        ImportedStructureSelectorTestMutation::PrimarySourceId,
        ImportedStructureSelectorTestMutation::PrimaryModuleId,
    ];
    for index in 0..6 {
        primary_mutations.extend([
            ImportedStructureSelectorTestMutation::PrimaryTermSite(index),
            ImportedStructureSelectorTestMutation::PrimaryTermRange(index),
            ImportedStructureSelectorTestMutation::PrimaryTermOrdinal(index),
            ImportedStructureSelectorTestMutation::PrimaryTermContext(index),
            ImportedStructureSelectorTestMutation::PrimaryTermRecovery(index),
            ImportedStructureSelectorTestMutation::PrimaryTermSpelling(index),
            ImportedStructureSelectorTestMutation::PrimaryTermKind(index),
            ImportedStructureSelectorTestMutation::PrimaryTermRole(index),
            ImportedStructureSelectorTestMutation::PrimaryTermParent(index),
        ]);
    }
    for index in 0..4 {
        primary_mutations.extend([
            ImportedStructureSelectorTestMutation::PrimaryReferenceTerm(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceBinding(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceRole(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceUseOrdinal(index),
            ImportedStructureSelectorTestMutation::PrimaryReferenceScope(index),
        ]);
    }
    for index in 0..2 {
        primary_mutations.extend([
            ImportedStructureSelectorTestMutation::NumericRequestTerm(index),
            ImportedStructureSelectorTestMutation::NumericRequestOwner(index),
            ImportedStructureSelectorTestMutation::NumericRequestRange(index),
            ImportedStructureSelectorTestMutation::NumericRequestSpelling(index),
            ImportedStructureSelectorTestMutation::NumericRequestOrdinal(index),
        ]);
    }
    for mutation in primary_mutations {
        assert_lower_mutation(mutation);
    }
    let mut structure_mutations = vec![
        ImportedStructureSelectorTestMutation::RootTerm,
        ImportedStructureSelectorTestMutation::RootSymbol,
        ImportedStructureSelectorTestMutation::RootContribution,
        ImportedStructureSelectorTestMutation::FieldUpdateExtra,
        ImportedStructureSelectorTestMutation::StalePrimaryReplay,
        ImportedStructureSelectorTestMutation::TermRangeAndStalePrimaryReplay,
    ];
    for index in 0..2 {
        structure_mutations.extend([
            ImportedStructureSelectorTestMutation::TermSite(index),
            ImportedStructureSelectorTestMutation::TermRange(index),
            ImportedStructureSelectorTestMutation::TermOrdinal(index),
            ImportedStructureSelectorTestMutation::TermContext(index),
            ImportedStructureSelectorTestMutation::TermRecovery(index),
            ImportedStructureSelectorTestMutation::TermSpelling(index),
            ImportedStructureSelectorTestMutation::TermKind(index),
        ]);
    }
    for index in 0..3 {
        structure_mutations.extend([
            ImportedStructureSelectorTestMutation::MemberTerm(index),
            ImportedStructureSelectorTestMutation::MemberOrdinal(index),
            ImportedStructureSelectorTestMutation::MemberSite(index),
            ImportedStructureSelectorTestMutation::MemberRange(index),
            ImportedStructureSelectorTestMutation::MemberSpelling(index),
            ImportedStructureSelectorTestMutation::MemberRole(index),
            ImportedStructureSelectorTestMutation::MemberParent(index),
            ImportedStructureSelectorTestMutation::EdgeTerm(index),
            ImportedStructureSelectorTestMutation::EdgeOrdinal(index),
            ImportedStructureSelectorTestMutation::EdgeRole(index),
            ImportedStructureSelectorTestMutation::EdgeMember(index),
            ImportedStructureSelectorTestMutation::EdgeTarget(index),
        ]);
    }
    for index in 0..9 {
        structure_mutations.extend([
            ImportedStructureSelectorTestMutation::RequestTerm(index),
            ImportedStructureSelectorTestMutation::RequestOrdinal(index),
            ImportedStructureSelectorTestMutation::RequestMember(index),
            ImportedStructureSelectorTestMutation::RequestKind(index),
        ]);
    }
    for mutation in structure_mutations {
        assert_lower_mutation(mutation);
    }

    let lower_before_base = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| {
            input.atomic = stale_atomic.clone();
            input.statement.owners.clear();
            input.witness.witnesses.clear();
        },
    )
    .expect("combined lower/base/witness selector")
    .expect_err("lower dependency must fail first");
    assert_eq!(lower_before_base, "Task258B3M2B2B2B lower dependency mismatch");
    let arena_before_base = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| {
            let nodes = input.arena.iter().map(|(_, node)| node.clone()).collect();
            input.arena = mizar_checker::typed_ast::TypedArena::try_new(None, nodes)
                .expect("rootless dependency arena");
            input.statement.owners.clear();
            input.witness.witnesses.clear();
        },
    )
    .expect("combined arena/base/witness selector")
    .expect_err("arena dependency must fail first");
    assert_eq!(arena_before_base, "Task258B3M2B2B2B lower dependency mismatch");
    let mut changed = None;
    let aggregate_before_owner_and_witness = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| {
            input.statement.owners[0].spelling.push('x');
            input.statement.statements.clear();
            input.witness.witnesses.clear();
            changed = Some(input.clone());
        },
    )
    .expect("combined aggregate/owner/witness selector")
    .expect_err("base aggregate must fail first");
    assert_eq!(aggregate_before_owner_and_witness, "source statement dependency mismatch");
    assert_ne!(format!("{:?}", changed.expect("actual combined upper mutation")), baseline_debug);
    let dependency_first = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| {
            input.structure = None;
            input.statement.owners.clear();
            input.witness.witnesses.clear();
        },
    )
    .expect("dependency precedence selector")
    .expect_err("missing structure must fail first");
    assert_eq!(dependency_first, "Task258B3M2B2B2B lower dependency mismatch");
    let mut changed = None;
    let owner_first = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| {
            input.statement.owners[0].spelling.push('!');
            input.witness.witnesses.clear();
            changed = Some(input.clone());
        },
    )
    .expect("owner precedence selector")
    .expect_err("owner must fail before witness");
    assert_eq!(owner_first, "source statement dependency mismatch");
    assert_ne!(format!("{:?}", changed.expect("actual combined upper mutation")), baseline_debug);
    let mut changed = None;
    let witness_aggregate = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| {
            input.witness.witnesses.clear();
            changed = Some(input.clone());
        },
    )
    .expect("witness selector")
    .expect_err("missing witness must fail");
    assert_eq!(witness_aggregate, "source statement dependency mismatch");
    assert_ne!(format!("{:?}", changed.expect("actual combined upper mutation")), baseline_debug);
    // Combined upper draft precedence is exercised by the Checker private suite.
    assert_eq!(format!("{:?}", task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols)), baseline_debug);
    assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
}

#[test]
fn task258b3m2b2b2b_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols) = task258b3m2b2b2b_fixture(259_202);
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B2B_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B2B_TEXT.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' {
            b'?'
        } else {
            b'!'
        };
        let loaded_source = String::from_utf8(bytes).expect("ASCII mutation");
        assert!(
            extract_structure_selector_witness_source_statement(&ast, &loaded_source).is_none(),
            "byte {byte_index} exact extraction"
        );
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &loaded_source,)
                .is_none(),
            "byte {byte_index}"
        );
    }
    let resolver = source_statement_b3m2b2b2b_resolver_env_for_test(
        &module,
        &symbols,
        range(ast.source_id, 56, 101),
    )
    .expect("resolver");
    let baseline = task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols);
    let binding_env = &baseline.binding_env;
    for node in 0..79 {
        for surface_mutation in [
            ImportedStructureSelectorSurfaceMutation::NodeKind(node),
            ImportedStructureSelectorSurfaceMutation::NodeRange(node),
            ImportedStructureSelectorSurfaceMutation::NodeRecovery(node),
            ImportedStructureSelectorSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                imported_structure_selector_handoff_for_test(
                    &ast,
                    &module,
                    &resolver,
                    binding_env,
                    SOURCE_STATEMENT_B3M2B2B2B_TEXT,
                    &task258b3m2b2b2b_roots(),
                    ImportedStructureSelectorTestOptions {
                        selector: 62,
                        context: mizar_checker::binding_env::BindingContextId::new(1),
                        surface_mutation,
                        handoff_mutation: ImportedStructureSelectorTestMutation::None,
                    },
                )
                .is_none(),
                "node {node} {surface_mutation:?}"
            );
        }
    }
    assert!(
        imported_structure_selector_handoff_for_test(
            &ast,
            &module,
            &resolver,
            binding_env,
            SOURCE_STATEMENT_B3M2B2B2B_TEXT,
            &task258b3m2b2b2b_roots(),
            ImportedStructureSelectorTestOptions {
                selector: 62,
                context: mizar_checker::binding_env::BindingContextId::new(1),
                surface_mutation: ImportedStructureSelectorSurfaceMutation::RootIdentity,
                handoff_mutation: ImportedStructureSelectorTestMutation::None,
            },
        )
        .is_none()
    );

    let missing_selector = SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(").x;", ").;", 1);
    assert_eq!(missing_selector.len(), 170);
    let (missing_ast, missing_module, _, missing_symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&missing_selector, 259_203);
    assert_eq!(
        (
            diagnostics,
            missing_ast.nodes().len(),
            missing_ast.root().map(|root| root.index()),
        ),
        (1, 78, Some(77))
    );
    assert_eq!(
        task258b3m2b2b2bp_frontend_diagnostic_profile(&missing_selector, 259_204),
        [("malformed_term_expression".to_owned(), 149, 150)]
    );
    assert!(missing_ast.nodes().iter().all(|node| !node.recovered));
    let missing_symbols =
        augment_type_elaboration_import_summaries(&missing_ast, &missing_module, missing_symbols);
    assert!(
        extract_structure_selector_witness_source_statement(&missing_ast, &missing_selector)
            .is_none()
    );
    assert!(
        source_statement_output_with_source(
            &missing_ast,
            missing_module,
            &missing_symbols,
            &missing_selector,
        )
        .is_none()
    );
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "TypeCaseStruct(x: 1, y: 2).y",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "TypeCaseStruct(x: 1, y: 2).x()",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "TypeCaseStruct(x: 1, y: 2).x(1)",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "TypeCaseStruct(x: 1, y: 2).x.x",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "(TypeCaseStruct(x: 1, y: 2)).x",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "TypeCaseStruct(y: 2, x: 1).x",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "TypeCaseStruct(x: 1, y: 2)",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2).x",
            "TypeCaseStruct(x: 1, y: 2) with (x := 3)",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "FormulaStatementStructureSelectorWitnessSmoke",
            "FormulaStatementStructureSelectorWitnessNearMiss",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2B_TEXT.replacen(
            "import parser.type_fixtures;",
            "import parser.other_fixtures;",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, _, near_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 259_210 + ordinal);
        if ordinal < 8 {
            assert_eq!(diagnostics, 0, "valid exclusion {ordinal} diagnostics");
        }
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_symbols);
        assert!(
            extract_structure_selector_witness_source_statement(&near_ast, &source).is_none(),
            "near miss {ordinal} extracted"
        );
        assert!(
            source_statement_output_with_source(&near_ast, near_module, &near_symbols, &source)
                .is_none(),
            "near miss {ordinal} selected"
        );
    }
}

#[test]
fn task258b3m2b2b2b_family_and_active_route_isolation_is_atomic() {
    let (ast, module, symbols) = task258b3m2b2b2b_fixture(259_220);
    let baseline = task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols);
    let baseline_debug = format!("{baseline:?}");
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    for mutation in [
        Task258B3M2B2B2AResolverMutation::LocalAndFqn,
        Task258B3M2B2B2AResolverMutation::StructuralPath,
        Task258B3M2B2B2AResolverMutation::Signature,
        Task258B3M2B2B2AResolverMutation::ExportStatus,
        Task258B3M2B2B2AResolverMutation::Contribution,
    ] {
        let substituted = task258b3m2b2b2a_substituted_structure_env(&symbols, mutation);
        assert_ne!(
            substituted, symbols,
            "actual structure resolver mutation {mutation:?}"
        );
        let mut captured = None;
        assert_eq!(
            source_statement_b3m2b2b2b_output_with_mutation(
                &ast,
                module.clone(),
                &substituted,
                SOURCE_STATEMENT_B3M2B2B2B_TEXT,
                |input| captured = Some(input.clone()),
            )
            .expect("authentic structure resolver mutation extraction")
            .expect_err("structure ownership refuses before lower capture"),
            "Task258B3M2B2B2B structure ownership selector mismatch",
            "mutation {mutation:?}"
        );
        assert!(captured.is_none(), "ownership mutation {mutation:?} reached lower capture");
        assert_eq!(
            format!(
                "{:?}",
                task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols)
            ),
            baseline_debug
        );
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
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
        let error = source_statement_b3m2b2b2b_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B2B_TEXT,
            |symbols| {
                let changed = task258b2_mutate_resolver(symbols.clone(), mutation);
                assert_ne!(changed, symbols);
                changed
            },
        )
        .expect("authentic theorem owner mutation extraction")
        .expect_err("theorem owner must refuse before lower capture");
        assert_eq!(
            error,
            match mutation {
                Task258B2ResolverMutation::Missing | Task258B2ResolverMutation::Duplicate =>
                    "Task258B3M2B2B2B resolver theorem label mismatch",
                _ => "Task258B3M2B2B2B resolver theorem provenance mismatch",
            },
            "owner mutation {mutation:?}"
        );
        assert_eq!(
            format!(
                "{:?}",
                task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols)
            ),
            baseline_debug
        );
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
    }

    let (constructor_ast, constructor_module, constructor_symbols) =
        task258b3m2b2b2a_fixture(259_221);
    let constructor = source_statement_output_with_source(
        &constructor_ast,
        constructor_module.clone(),
        &constructor_symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
    )
    .expect("constructor selector")
    .expect("constructor route");
    let constructor_structure = constructor
        .typed_ast
        .source_structure()
        .expect("constructor structure")
        .clone();
    let selector_structure = baseline.structure.as_ref().expect("selector structure").clone();
    let selector_rejects_constructor = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| input.structure = Some(constructor_structure),
    )
    .expect("selector hybrid route")
    .expect_err("selector/constructor hybrid must fail");
    assert_eq!(selector_rejects_constructor, "Task258B3M2B2B2B lower dependency mismatch");
    let constructor_rejects_selector = source_statement_b3m2b2b2a_output_with_mutation(
        &constructor_ast,
        constructor_module,
        &constructor_symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        |input| input.structure = Some(selector_structure),
    )
    .expect("constructor hybrid route")
    .expect_err("constructor/selector hybrid must fail");
    assert_eq!(constructor_rejects_selector, "Task258B3M2B2B2A lower dependency mismatch");

    let (application_ast, application_module, application_symbols) =
        task258b3m2b2b1a_fixture(259_222);
    let application =
        task258b3m2b2b1a_lower_inputs(&application_ast, &application_module, &application_symbols)
            .application
            .expect("genuine bare application");
    let selector_rejects_application = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| input.application = Some(application),
    )
    .expect("application/selector hybrid route")
    .expect_err("application/selector hybrid must fail");
    assert_eq!(selector_rejects_application, "Task258B3M2B2B2B lower dependency mismatch");

    for (source, expected_application, expected_terms) in [
        (SOURCE_STATEMENT_B3_TEXT, false, None),
        (SOURCE_STATEMENT_B3N_TEXT, false, None),
        (SOURCE_STATEMENT_B3M1_TEXT, false, None),
        (SOURCE_STATEMENT_B3M2A_TEXT, false, None),
        (SOURCE_STATEMENT_B3M2B1_TEXT, false, None),
        (SOURCE_STATEMENT_B3M2B2A_TEXT, false, None),
        (SOURCE_STATEMENT_B3M2B2B1A_TEXT, true, None),
        (SOURCE_STATEMENT_B3M2B2B1B1_TEXT, true, None),
        (SOURCE_STATEMENT_B3M2B2B2A_TEXT, false, Some(1)),
        (SOURCE_STATEMENT_B3M2B2B2B_TEXT, false, Some(2)),
    ] {
        let (family_ast, family_module, _, family_symbols) =
            task253_ast_from_source_text(source, 259_230 + source.len());
        let family_symbols =
            augment_type_elaboration_import_summaries(&family_ast, &family_module, family_symbols);
        if source == SOURCE_STATEMENT_B3M2B2B2B_TEXT {
            let lower = task258b3m2b2b2b_lower_inputs(&family_ast, &family_module, &family_symbols);
            assert!(lower.application.is_none() && lower.set_term.is_none());
            let structure = lower.structure.expect("authenticated selector structure");
            assert_eq!((structure.terms().len(), structure.field_updates().len()), (2, 0));
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B1A_TEXT {
            task258b3m2b2b1a_lower_inputs(&family_ast, &family_module, &family_symbols);
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B1B1_TEXT {
            task258b3m2b2b1b1_assert_retired(&family_ast, &family_module, &family_symbols);
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2A_TEXT {
            let before = (family_ast.snapshot_text(), family_module.clone(), family_symbols.clone());
            assert!(extract_numeral_witness_source_statement(&family_ast, source).is_some());
            assert!(source_statement_output_with_source(
                &family_ast, family_module.clone(), &family_symbols, source,
            ).is_none());
            assert_eq!(source_statement_b3m2a_output_with_resolver_mutation(
                &family_ast, family_module.clone(), &family_symbols, source, |symbols| symbols,
            ).expect("authentic retired numeral extraction").expect_err("retired raw profile"),
                "source statement dependency mismatch");
            assert_eq!((family_ast.snapshot_text(), family_module.clone(), family_symbols.clone()), before);
            continue;
        }
        let family_typed = if source == SOURCE_STATEMENT_B3_TEXT || source == SOURCE_STATEMENT_B3N_TEXT || source == SOURCE_STATEMENT_B3M1_TEXT || source == SOURCE_STATEMENT_B3M2B1_TEXT || source == SOURCE_STATEMENT_B3M2B2A_TEXT {
            task258_native_output(&family_ast, &family_module).0
        } else {
            source_statement_output_with_source(
            &family_ast,
            family_module,
            &family_symbols,
            source,
        )
        .expect("family selector")
        .expect("family route").typed_ast
        };
        assert_eq!(
            family_typed.source_application().is_some(),
            expected_application
        );
        assert_eq!(
            family_typed
                .source_structure()
                .map(|structure| structure.terms().len()),
            expected_terms
        );
        let family_witnesses = family_typed
            .source_statement_witnesses()
            .expect("family witness handoff");
        assert_eq!(
            family_witnesses.application_fingerprint().is_some(),
            expected_application
        );
        assert_eq!(
            family_witnesses.structure_fingerprint().is_some(),
            expected_terms.is_some()
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
    let plan = build_test_plan(&config).expect("Task258B3M2B2B2B isolation plan");
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
        if extract_structure_selector_witness_source_statement(&active_ast, &source).is_some()
            || source_statement_output_with_source(
                &active_ast,
                resolver.module,
                &active_symbols,
                &source,
            )
            .is_some_and(|result| {
                result.is_ok_and(|output| {
                    output
                        .typed_ast
                        .source_structure()
                        .is_some_and(|structure| structure.terms().len() == 2)
                        && output
                            .typed_ast
                            .source_statement_witnesses()
                            .is_some_and(|witnesses| {
                                witnesses.application_fingerprint().is_none()
                                    && witnesses.structure_fingerprint().is_some()
                            })
                })
            })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M2B2B2B selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b2b2b_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, symbols) = task258b3m2b2b2b_fixture(259_250);
    let output = task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let baseline = format!("{output:?}");
    assert_eq!(format!("{:?}", output.clone()), baseline);
    // Accepted TypedAst/ResolvedTypedAst identity, empty semantics, and atomic
    // installer rollback remain in the Checker private structural proof.
    assert!(output.application.is_none() && output.set_term.is_none());
    assert_eq!(
        (output.primary.terms().len(), output.atomic.formulas().len()),
        (6, 2)
    );
    assert_eq!(output.witness.witnesses.len(), 1);
    assert!(output.witness.names.is_empty());
    let mut changed = None;
    let error = source_statement_b3m2b2b2b_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        |input| {
            input.witness.witnesses.clear();
            changed = Some(input.clone());
        },
    )
    .expect("rollback selector")
    .expect_err("empty witness aggregate must roll back");
    assert_eq!(error, "source statement dependency mismatch");
    assert!(
        changed
            .expect("actual empty witness mutation")
            .witness
            .witnesses
            .is_empty()
    );
    assert_eq!(format!("{output:?}"), baseline);
    assert_eq!(
        format!(
            "{:?}",
            task258b3m2b2b2b_lower_inputs(&ast, &module, &symbols)
        ),
        baseline
    );
    assert_eq!(
        (ast.snapshot_text(), module.clone(), symbols.clone()),
        before
    );
}
