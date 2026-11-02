#[test]
fn task258b3m2b2b2a_real_frontend_freezes_structure_constructor_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B2B2A_TEXT.len(), 172);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B2A_TEXT),
        "24e2ee2332ead5c0d46025df6044450eeab3ebb5733ebe83587ceae3ba129eb6"
    );
    let (ast, module, symbols) = task258b3m2b2b2a_fixture(259_100);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (76, Some(75))
    );
    let extracted: SourceStatementB3M2B2B2AExtraction =
        extract_structure_constructor_witness_source_statement(
            &ast,
            SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        )
        .expect("Task258B3M2B2B2A exact extraction");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range,
            extracted.label_range,
            extracted.take_site.node().index(),
            extracted.take_range,
            extracted.witness_site.node().index(),
            extracted.witness_range,
            extracted.structure_node,
            extracted.proof_range,
        ),
        (
            72,
            range(ast.source_id, 48, 171),
            range(ast.source_id, 56, 104),
            62,
            range(ast.source_id, 120, 152),
            61,
            range(ast.source_id, 125, 151),
            59,
            range(ast.source_id, 112, 170),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [45, 47, 54, 57, 63, 65]
    );
    let resolver =
        source_statement_b3m2b2b2a_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("Task258B3M2B2B2A resolver");
    assert_eq!((resolver.imports().len(), resolver.labels().len()), (1, 1));
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver.symbols().visible_candidates(
        &namespace,
        "FormulaStatementStructureConstructorWitnessSmoke",
    );
    let [owner] = owners.as_slice() else {
        panic!("Task258B3M2B2B2A exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 48, 171))
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
        .expect("Task258B3M2B2B2A owner contribution");
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
        "FormulaStatementStructureConstructorWitnessSmoke",
    );
    let [label] = labels.as_slice() else {
        panic!("Task258B3M2B2B2A exact theorem label");
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

    let output = task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols);
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
            (0, 45, 0),
            (1, 47, 0),
            (2, 53, 1),
            (3, 56, 1),
            (4, 63, 1),
            (5, 65, 1)
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
            if source_range == range(ast.source_id, 112, 170)
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
            (2, 53, range(ast.source_id, 143, 144), "1", 0),
            (3, 56, range(ast.source_id, 149, 150), "2", 1),
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
        (1, 0, 1, 2, 0, 2, 6)
    );
    let structure_term = structure
        .terms()
        .get(mizar_checker::source_structure::SourceStructureTermId::new(
            0,
        ))
        .expect("Task254 term 0");
    assert_eq!(structure_term.site().node().index(), 59);
    assert_eq!(
        structure_term.source_range(),
        range(ast.source_id, 125, 151)
    );
    assert_eq!(structure_term.source_ordinal(), 0);
    assert_eq!(
        structure_term.context(),
        mizar_checker::binding_env::BindingContextId::new(1)
    );
    assert_eq!(
        structure_term.recovery(),
        mizar_checker::source_structure::SourceStructureRecovery::Normal
    );
    assert_eq!(
        structure_term.spelling(),
        "TypeCaseStruct ( x : 1 , y : 2 )"
    );
    assert_eq!(
        structure_term.kind(),
        mizar_checker::source_structure::SourceStructureTermKind::Constructor
    );
    let structure_root = structure
        .roots()
        .get(mizar_checker::source_structure::SourceStructureRootId::new(
            0,
        ))
        .expect("Task254 root 0");
    assert_eq!(structure_root.term().index(), 0);
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
                20,
                range(ast.source_id, 140, 141),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
            (
                0,
                1,
                24,
                range(ast.source_id, 146, 147),
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
            .map(|(_, edge)| edge.target())
            .collect::<Vec<_>>(),
        [
            mizar_checker::source_structure::SourceStructureTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(2),
            ),
            mizar_checker::source_structure::SourceStructureTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(3),
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
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ConstructorSignature,
            ),
            (
                0,
                1,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                0,
                2,
                Some(0),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                0,
                3,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::MemberIdentity,
            ),
            (
                0,
                4,
                Some(1),
                mizar_checker::source_structure::SourceStructureRequestKind::InheritancePath,
            ),
            (
                0,
                5,
                None,
                mizar_checker::source_structure::SourceStructureRequestKind::ResultType,
            ),
        ]
    );
    let atomic = &output.atomic;
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
                49,
                range(ast.source_id, 106, 111),
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
                "x = x",
            ),
            (
                67,
                range(ast.source_id, 160, 165),
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
    let statements = &output.statement;
    assert_eq!(
        (
            statements.owners.len(),
            statements.statements.len(),
            statements.contexts.len(),
            statements.input_facts.len(),
            statements.candidate_facts.len(),
        ),
        (1, 2, 2, 2, 2)
    );
    let owner_row = statements
        .owners
        .first()
        .expect("Task258 owner 0");
    assert_eq!(&owner_row.symbol, owner.symbol());
    assert_eq!(owner_row.contribution, owner.contribution());
    assert_eq!(owner_row.site.node().index(), 72);
    assert_eq!(owner_row.source_range, range(ast.source_id, 48, 171));
    assert_eq!(
        owner_row.spelling,
        "FormulaStatementStructureConstructorWitnessSmoke"
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
        let statement = statements
            .statements
            .get(index)
            .expect("Task258 statement row");
        assert_eq!(statement.owner.index(), 0);
        assert_eq!(statement.context.index(), index);
        assert_eq!(
            statement.formula,
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(statement.site.node().index(), [72, 70][index]);
        assert_eq!(
            statement.source_range,
            [
                range(ast.source_id, 48, 171),
                range(ast.source_id, 155, 166),
            ][index]
        );
        assert_eq!(statement.source_ordinal, [0, 2][index]);
        assert_eq!(
            statement.spelling,
            [
                "theorem FormulaStatementStructureConstructorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) ; thus x = x ; end ;",
                "thus x = x ;",
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
        let context = statements
            .contexts
            .get(index)
            .expect("Task258 context row");
        assert_eq!(context.statement.index(), index);
        assert_eq!(context.binding_context.index(), index);
        assert_eq!(context.source_range, statement.source_range);
        assert_eq!(
            context.visible_bindings,
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        let input_fact = statements
            .input_facts
            .get(index)
            .expect("Task258 input-fact row");
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
        let candidate_fact = statements
            .candidate_facts
            .get(index)
            .expect("Task258 candidate-fact row");
        assert_eq!(candidate_fact.statement.index(), index);
        assert_eq!(candidate_fact.context.index(), index);
        assert_eq!(candidate_fact.ordinal, 0);
        assert_eq!(
            candidate_fact.kind,
            mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate_fact.formula, statement.formula);
    }
    let witnesses = &output.witness;
    assert_eq!(
        (witnesses.witnesses.len(), witnesses.names.len()),
        (1, 0)
    );
    let witness = witnesses
        .witnesses
        .first()
        .expect("structure witness");
    assert_eq!(
        witness.term,
        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Structure(
            mizar_checker::source_structure::SourceStructureTermId::new(0),
        )
    );
    assert_eq!(witness.owner.index(), 0);
    assert_eq!(witness.binding_context.index(), 1);
    assert_eq!(witness.take_site.node().index(), 62);
    assert_eq!(witness.take_range, range(ast.source_id, 120, 152));
    assert_eq!(witness.site.node().index(), 61);
    assert_eq!(witness.source_range, range(ast.source_id, 125, 151));
    assert_eq!(witness.source_ordinal, 1);
    assert_eq!(witness.ordinal, 0);
    assert_eq!(witness.spelling, "TypeCaseStruct ( x : 1 , y : 2 )");
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
                45 | 47 | 63 | 65 => "source.term.variable-reference",
                53 | 56 => "source.term.numeral",
                49 | 67 => "source.formula.atomic.equality",
                59 => "source.term.structure.constructor",
                61 => "source.statement-witness.item",
                62 => "source.statement-witness.take",
                70 => "source.statement.conclusion",
                72 => "source.statement.theorem",
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
            (59, "source.term.structure.constructor"),
        ]
    );
}

#[test]
fn task258b3m2b2b2a_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols) = task258b3m2b2b2a_fixture(259_101);
    let baseline = task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols);
    let baseline_debug = format!("{baseline:?}");
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let (stale_ast, stale_module, stale_symbols) = task258b3m2b2b2a_fixture(259_102);
    let stale = task258b3m2b2b2a_lower_inputs(&stale_ast, &stale_module, &stale_symbols);
    let stale_binding = stale.binding_env;
    let stale_primary = stale.primary;
    let stale_atomic = stale.atomic;
    let stale_structure = stale.structure.expect("stale structure");
    for mutation in 0..21 {
        let stale_binding = stale_binding.clone();
        let stale_primary = stale_primary.clone();
        let stale_atomic = stale_atomic.clone();
        let stale_structure = stale_structure.clone();
        let mut changed = None;
        let error = source_statement_b3m2b2b2a_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B2A_TEXT,
            |input: &mut SourceStatementB3M2B2B2ARouteInputs| {
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
                14 => input.witness.witnesses[0].take_range.start += 1,
                15 => input.witness.witnesses[0].source_range.end -= 1,
                16 => input.witness.witnesses[0].source_ordinal = 0,
                17 => input.witness.witnesses[0].ordinal = 1,
                18 => input.witness.witnesses[0].spelling.push('x'),
                19 => {
                    input.witness.witnesses[0].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0)
                }
                20 => {
                    input.witness.witnesses[0].kind =
                        mizar_checker::source_statement::SourceStatementWitnessKind::Named
                }
                _ => unreachable!(),
                }
                changed = Some(input.clone());
            },
        )
        .expect("mutation selector")
        .expect_err("mutation must fail closed");
        assert_eq!(error, if mutation < 5 {
            "Task258B3M2B2B2A lower dependency mismatch"
        } else { "source statement dependency mismatch" }, "mutation {mutation}");
        let changed = changed.expect("actual mutated handoff before admission");
        assert_ne!(format!("{changed:?}"), baseline_debug, "mutation {mutation}");
        // Checker private guards: 0..2 Statement DependencyMismatch;
        // 3..4 Witness DependencyMismatch; 5 InvalidAggregate; 6 InvalidOwner(0);
        // 7 InvalidStatement(0); 8 InvalidContext(1); 9 InvalidInputFact(1);
        // 10 InvalidCandidateFact(1); 11 Witness InvalidAggregate;
        // 12..20 Witness InvalidWitness(0). The changed capture, rather than
        // retired admission, connects these exact private guards to this runner.
        if (5..11).contains(&mutation) {
            assert_ne!(changed.statement, baseline.statement);
            assert_eq!(changed.witness, baseline.witness);
        } else if mutation >= 11 {
            assert_ne!(changed.witness, baseline.witness);
            assert_eq!(changed.statement, baseline.statement);
        }
        assert_eq!(format!("{:?}", task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols)), baseline_debug);
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
    }

    let lower_before_base = source_statement_b3m2b2b2a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        |input| {
            input.atomic = stale_atomic.clone();
            input.statement.owners.clear();
            input.witness.witnesses.clear();
        },
    )
    .expect("combined lower/base/witness selector")
    .expect_err("lower dependency must fail first");
    assert_eq!(lower_before_base, "Task258B3M2B2B2A lower dependency mismatch");
    let arena_before_base = source_statement_b3m2b2b2a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
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
    assert_eq!(arena_before_base, "Task258B3M2B2B2A lower dependency mismatch");
    let mut changed = None;
    let aggregate_before_owner_and_witness = source_statement_b3m2b2b2a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
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
    let changed = changed.expect("actual combined mutation capture");
    assert!(changed.witness.witnesses.is_empty());
    assert!(changed.statement.statements.is_empty());
    assert_ne!(changed.statement.owners[0], baseline.statement.owners[0]);

    let mut changed = None;
    let owner_before_witness = source_statement_b3m2b2b2a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        |input| {
            input.statement.owners[0].spelling.push('x');
            input.witness.witnesses.clear();
            changed = Some(input.clone());
        },
    )
    .expect("combined owner/witness selector")
    .expect_err("base owner must fail first");
    assert_eq!(owner_before_witness, "source statement dependency mismatch");
    let changed = changed.expect("actual combined mutation capture");
    assert!(changed.witness.witnesses.is_empty());
    assert_eq!(changed.statement.statements, baseline.statement.statements);
    assert_ne!(changed.statement.owners[0], baseline.statement.owners[0]);

    let mut changed = None;
    let witness_aggregate = source_statement_b3m2b2b2a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        |input| { input.witness.witnesses.clear(); changed = Some(input.clone()); },
    )
    .expect("witness aggregate selector")
    .expect_err("witness aggregate must fail");
    assert_eq!(witness_aggregate, "source statement dependency mismatch");
    let changed = changed.expect("actual combined mutation capture");
    assert!(changed.witness.witnesses.is_empty());
    assert_eq!(changed.statement, baseline.statement);

    assert_eq!(format!("{:?}", task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols)), baseline_debug);

    let binding_env = &baseline.binding_env;
    for mutation in [
        ImportedStructureConstructorTestMutation::TermSite,
        ImportedStructureConstructorTestMutation::TermRange,
        ImportedStructureConstructorTestMutation::TermOrdinal,
        ImportedStructureConstructorTestMutation::TermContext,
        ImportedStructureConstructorTestMutation::TermRecovery,
        ImportedStructureConstructorTestMutation::TermSpelling,
        ImportedStructureConstructorTestMutation::TermKind,
        ImportedStructureConstructorTestMutation::RootTerm,
        ImportedStructureConstructorTestMutation::RootSymbol,
        ImportedStructureConstructorTestMutation::RootContribution,
        ImportedStructureConstructorTestMutation::MemberTerm,
        ImportedStructureConstructorTestMutation::MemberOrdinal,
        ImportedStructureConstructorTestMutation::MemberSite,
        ImportedStructureConstructorTestMutation::MemberRange,
        ImportedStructureConstructorTestMutation::MemberSpelling,
        ImportedStructureConstructorTestMutation::MemberRole,
        ImportedStructureConstructorTestMutation::MemberParent,
        ImportedStructureConstructorTestMutation::FieldUpdateExtra,
        ImportedStructureConstructorTestMutation::EdgeTerm,
        ImportedStructureConstructorTestMutation::EdgeOrdinal,
        ImportedStructureConstructorTestMutation::EdgeRole,
        ImportedStructureConstructorTestMutation::EdgeMember,
        ImportedStructureConstructorTestMutation::EdgeTarget,
        ImportedStructureConstructorTestMutation::RequestTerm,
        ImportedStructureConstructorTestMutation::RequestOrdinal,
        ImportedStructureConstructorTestMutation::RequestMember,
        ImportedStructureConstructorTestMutation::RequestKind,
        ImportedStructureConstructorTestMutation::StalePrimaryReplay,
        ImportedStructureConstructorTestMutation::TermRangeAndStalePrimaryReplay,
    ] {
        let result = imported_structure_constructor_handoff_for_test(
            &ast,
            &module,
            &source_statement_b3m2b2b2a_resolver_env_for_test(
                &module,
                &symbols,
                range(ast.source_id, 56, 104),
            )
            .expect("resolver"),
            binding_env,
            SOURCE_STATEMENT_B3M2B2B2A_TEXT,
            &task258b3m2b2b2a_roots(),
            ImportedStructureConstructorTestOptions {
                constructor: 59,
                context: mizar_checker::binding_env::BindingContextId::new(1),
                surface_mutation: ImportedStructureConstructorSurfaceMutation::None,
                handoff_mutation: mutation,
            },
        );
        assert!(
            !matches!(result, Some(Ok(_))),
            "structure mutation {mutation:?} succeeded"
        );
    }
}

#[test]
fn task258b3m2b2b2a_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols) = task258b3m2b2b2a_fixture(259_103);
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B2A_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B2A_TEXT.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' {
            b'?'
        } else {
            b'!'
        };
        let loaded_source = String::from_utf8(bytes).expect("ASCII mutation");
        assert!(extract_structure_constructor_witness_source_statement(&ast, &loaded_source).is_none(), "byte {byte_index} extracted");
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &loaded_source,)
                .is_none(),
            "byte {byte_index}"
        );
    }
    let resolver = source_statement_b3m2b2b2a_resolver_env_for_test(
        &module,
        &symbols,
        range(ast.source_id, 56, 104),
    )
    .expect("resolver");
    let baseline = task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols);
    let binding_env = &baseline.binding_env;
    for node in 0..76 {
        for surface_mutation in [
            ImportedStructureConstructorSurfaceMutation::NodeKind(node),
            ImportedStructureConstructorSurfaceMutation::NodeRange(node),
            ImportedStructureConstructorSurfaceMutation::NodeRecovery(node),
            ImportedStructureConstructorSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                imported_structure_constructor_handoff_for_test(
                    &ast,
                    &module,
                    &resolver,
                    binding_env,
                    SOURCE_STATEMENT_B3M2B2B2A_TEXT,
                    &task258b3m2b2b2a_roots(),
                    ImportedStructureConstructorTestOptions {
                        constructor: 59,
                        context: mizar_checker::binding_env::BindingContextId::new(1),
                        surface_mutation,
                        handoff_mutation: ImportedStructureConstructorTestMutation::None,
                    },
                )
                .is_none(),
                "node {node} {surface_mutation:?}"
            );
        }
    }
    assert!(
        imported_structure_constructor_handoff_for_test(
            &ast,
            &module,
            &resolver,
            binding_env,
            SOURCE_STATEMENT_B3M2B2B2A_TEXT,
            &task258b3m2b2b2a_roots(),
            ImportedStructureConstructorTestOptions {
                constructor: 59,
                context: mizar_checker::binding_env::BindingContextId::new(1),
                surface_mutation: ImportedStructureConstructorSurfaceMutation::RootIdentity,
                handoff_mutation: ImportedStructureConstructorTestMutation::None,
            },
        )
        .is_none()
    );

    for (ordinal, source) in [
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2)",
            "TypeCaseStruct(y: 2, x: 1)",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2)",
            "TypeCaseStruct(x: 1)",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "take TypeCaseStruct(x: 1, y: 2);",
            "take (TypeCaseStruct(x: 1, y: 2));",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "take TypeCaseStruct(x: 1, y: 2);",
            "take TypeCaseStruct(x: 1, y: 2).x;",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "take TypeCaseStruct(x: 1, y: 2);",
            "take TypeCaseStruct(x: 1, y: 2) with (x := 3);",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "FormulaStatementStructureConstructorWitnessSmoke",
            "FormulaStatementStructureConstructorWitnessNearMiss",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "import parser.type_fixtures;",
            "import parser.other_fixtures;",
            1,
        ),
        SOURCE_STATEMENT_B3M2B2B2A_TEXT.replacen(
            "TypeCaseStruct(x: 1, y: 2)",
            "TypeCaseStruct(x: , y: 2)",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, _, near_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 259_110 + ordinal);
        if ordinal == 7 {
            assert_eq!(
                (
                    diagnostics,
                    near_ast.nodes().len(),
                    near_ast.root().map(|root| root.index()),
                ),
                (1, 74, Some(73))
            );
            assert_eq!(
                near_ast
                    .nodes()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, node)| node.recovered.then_some(index))
                    .collect::<Vec<_>>(),
                [52]
            );
        }
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_symbols);
        assert!(extract_structure_constructor_witness_source_statement(&near_ast, &source).is_none(), "near miss {ordinal} extracted");
        assert!(
            source_statement_output_with_source(&near_ast, near_module, &near_symbols, &source)
                .is_none(),
            "near miss {ordinal}"
        );
    }
}

#[test]
fn task258b3m2b2b2a_family_and_active_route_isolation_is_atomic() {
    let (ast, module, symbols) = task258b3m2b2b2a_fixture(259_104);
    let baseline = task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols);
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
        assert_ne!(substituted, symbols, "actual structure resolver mutation {mutation:?}");
        let mut captured = None;
        assert_eq!(source_statement_b3m2b2b2a_output_with_mutation(
            &ast, module.clone(), &substituted, SOURCE_STATEMENT_B3M2B2B2A_TEXT,
            |input| captured = Some(input.clone()),
        ).expect("authentic structure extraction").expect_err("structure ownership refusal"),
            "Task258B3M2B2B2A structure ownership selector mismatch", "mutation {mutation:?}");
        assert!(captured.is_none(), "structure ownership mutation {mutation:?} reached capture");
        assert_eq!(format!("{:?}", task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols)), baseline_debug);
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
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
        let error = source_statement_b3m2b2b2a_output_with_resolver_mutation(
            &ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B2B2A_TEXT,
            |symbols| {
                let changed = task258b2_mutate_resolver(symbols.clone(), mutation);
                assert_ne!(changed, symbols);
                changed
            },
        ).expect("authentic owner mutation extraction").expect_err("owner refusal");
        assert_eq!(error, match mutation {
            Task258B2ResolverMutation::Missing | Task258B2ResolverMutation::Duplicate =>
                "Task258B3M2B2B2A resolver theorem label mismatch",
            _ => "Task258B3M2B2B2A resolver theorem provenance mismatch",
        }, "owner mutation {mutation:?}");
        assert_eq!(format!("{:?}", task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols)), baseline_debug);
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
    }
    let (application_ast, application_module, application_symbols) =
        task258b3m2b2b1a_fixture(259_105);
    let application =
        task258b3m2b2b1a_lower_inputs(&application_ast, &application_module, &application_symbols)
            .application
            .expect("genuine bare application");
    let error = source_statement_b3m2b2b2a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        |input| {
            input.application = Some(application);
        },
    )
    .expect("hybrid selector")
    .expect_err("application/structure substitution must reject");
    assert!(error.to_ascii_lowercase().contains("dependency"), "{error}");

    for (source, expected_application, expected_structure) in [
        (SOURCE_STATEMENT_B3_TEXT, false, false),
        (SOURCE_STATEMENT_B3N_TEXT, false, false),
        (SOURCE_STATEMENT_B3M1_TEXT, false, false),
        (SOURCE_STATEMENT_B3M2A_TEXT, false, false),
        (SOURCE_STATEMENT_B3M2B1_TEXT, false, false),
        (SOURCE_STATEMENT_B3M2B2A_TEXT, false, false),
        (SOURCE_STATEMENT_B3M2B2B1A_TEXT, true, false),
        (SOURCE_STATEMENT_B3M2B2B1B1_TEXT, true, false),
    ] {
        let (family_ast, family_module, _, family_symbols) =
            task253_ast_from_source_text(source, 259_120 + source.len());
        let family_symbols =
            augment_type_elaboration_import_summaries(&family_ast, &family_module, family_symbols);
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
        .expect("family output").typed_ast
        };
        assert_eq!(
            family_typed.source_application().is_some(),
            expected_application
        );
        assert_eq!(
            family_typed.source_structure().is_some(),
            expected_structure
        );
        let witnesses = family_typed
            .source_statement_witnesses()
            .expect("family witnesses");
        assert_eq!(
            (
                witnesses.application_fingerprint().is_some(),
                witnesses.structure_fingerprint().is_some(),
            ),
            (expected_application, expected_structure)
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
    let plan = build_test_plan(&config).expect("Task258B3M2B2B2A isolation plan");
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
        if extract_structure_constructor_witness_source_statement(&ast, &source).is_some()
            || source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
                .is_some_and(|result| {
                    result.is_ok_and(|output| {
                        output.typed_ast.source_structure().is_some()
                            && output.typed_ast.source_statement_witnesses().is_some_and(
                                |witnesses| {
                                    witnesses.application_fingerprint().is_none()
                                        && witnesses.structure_fingerprint().is_some()
                                },
                            )
                    })
                })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M2B2B2A selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b2b2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, symbols) = task258b3m2b2b2a_fixture(259_106);
    let output = task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols);
    let baseline = format!("{output:?}");
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    assert_eq!(format!("{:?}", output.clone()), baseline);
    let mut changed = None;
    assert_eq!(source_statement_b3m2b2b2a_output_with_mutation(
        &ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        |input| { input.witness.witnesses.clear(); changed = Some(input.clone()); },
    ).expect("authentic extraction").expect_err("retired admission"), "source statement dependency mismatch");
    assert!(changed.expect("actual witness mutation").witness.witnesses.is_empty());
    assert_eq!(format!("{output:?}"), baseline);
    assert_eq!(format!("{:?}", task258b3m2b2b2a_lower_inputs(&ast, &module, &symbols)), baseline);
    assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
}
