#[test]
fn task258b3m2b2b2c_real_frontend_freezes_structure_update_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B2B2C_TEXT.len(), 181);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B2C_TEXT),
        "03f14a98bffb557ea4dda4f879bf504d241aaebae0552a97f0f2417ef4b43560"
    );
    let (ast, module, symbols) = task258b3m2b2b2c_fixture(259_300);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (86, Some(85))
    );
    let extracted: SourceStatementB3M2B2B2CExtraction =
        extract_structure_update_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B2B2C_TEXT)
            .expect("Task258B3M2B2B2C exact extraction");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range,
            extracted.label_range,
            extracted.take_site.node().index(),
            extracted.take_range,
            extracted.witness_site.node().index(),
            extracted.witness_range,
            extracted.update_node,
            extracted.proof_range,
        ),
        (
            82,
            range(ast.source_id, 48, 180),
            range(ast.source_id, 56, 99),
            72,
            range(ast.source_id, 115, 161),
            71,
            range(ast.source_id, 120, 160),
            69,
            range(ast.source_id, 107, 179),
        )
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [51, 53, 60, 63, 67, 73, 75]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [55, 77]
    );

    let resolver =
        source_statement_b3m2b2b2c_resolver_env_for_test(&module, &symbols, extracted.label_range)
            .expect("Task258B3M2B2B2C resolver");
    assert_eq!((resolver.imports().len(), resolver.labels().len()), (1, 1));
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let owners = resolver
        .symbols()
        .visible_candidates(&namespace, "FormulaStatementStructureUpdateWitnessSmoke");
    let [owner] = owners.as_slice() else {
        panic!("Task258B3M2B2B2C exact theorem owner");
    };
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        owner.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 48, 180))
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
    assert_eq!(owner.kind(), mizar_resolve::env::SymbolKind::Theorem);
    let definition = resolver
        .definitions()
        .by_symbol(owner.symbol())
        .expect("Task258B3M2B2B2C theorem definition");
    assert_eq!(
        definition.kind(),
        mizar_resolve::env::DefinitionKind::Theorem
    );
    assert_eq!(definition.contribution(), owner.contribution());
    let contribution = resolver
        .contributions()
        .get(owner.contribution())
        .expect("Task258B3M2B2B2C owner contribution");
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
        .visible_candidates(&namespace, "FormulaStatementStructureUpdateWitnessSmoke");
    let [label] = labels.as_slice() else {
        panic!("Task258B3M2B2B2C exact theorem label");
    };
    assert_eq!(
        label.kind(),
        mizar_resolve::resolved_ast::LabelKind::Theorem
    );
    assert_eq!(label.namespace(), &namespace);
    assert_eq!(
        label.primary_spelling(),
        "FormulaStatementStructureUpdateWitnessSmoke"
    );
    assert_eq!(
        label.origin_path().as_str(),
        format!(
            "{}::{}::theorem::FormulaStatementStructureUpdateWitnessSmoke",
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
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
    )
    .expect("Task258B3M2B2B2C selector")
    .unwrap_or_else(|error| panic!("Task258B3M2B2B2C route failed: {error}"));
    let primary = output.typed_ast.source_term().expect("Task252");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (7, 4, 3)
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
            (0, 51, 0),
            (1, 53, 0),
            (2, 59, 1),
            (3, 62, 1),
            (4, 66, 1),
            (5, 73, 1),
            (6, 75, 1),
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
                range(ast.source_id, 101, 102),
                0,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 105, 106),
                1,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 138, 139),
                2,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "1",
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 144, 145),
                3,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "2",
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 158, 159),
                4,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "3",
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 169, 170),
                5,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
            (
                range(ast.source_id, 173, 174),
                6,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                "x",
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                None,
            ),
        ]
    );
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258 handoff");
    let bindings = statement.binding_env();
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
        .expect("Task48 proof context");
    assert!(matches!(
        proof_context.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
            if source_range == range(ast.source_id, 107, 179)
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
    assert_eq!(binding.id.index(), 0);
    assert_eq!(binding.spelling, "x");
    assert_eq!(
        binding.kind,
        mizar_checker::binding_env::BindingKind::ReservedVariable
    );
    assert_eq!(
        binding.owner_context,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    assert!(matches!(
        &binding.identity,
        mizar_checker::binding_env::BinderIdentity::ReservedVariable {
            spelling,
            declaration_range,
        } if spelling == "x" && *declaration_range == range(ast.source_id, 37, 38)
    ));
    assert_eq!(binding.declaration_range, range(ast.source_id, 37, 38));
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
                5,
                0,
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                Some(&[0][..]),
                1,
            ),
            (
                6,
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
            (2, 59, range(ast.source_id, 138, 139), "1", 0),
            (3, 62, range(ast.source_id, 144, 145), "2", 1),
            (4, 66, range(ast.source_id, 158, 159), "3", 2),
        ]
    );
    let structure = output.typed_ast.source_structure().expect("Task254");
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
        (2, 0, 1, 3, 1, 4, 9)
    );
    assert_eq!(
        structure
            .terms()
            .iter()
            .map(|(id, term)| (id.index(), term.site().node().index(), term.kind()))
            .collect::<Vec<_>>(),
        [
            (
                0,
                69,
                mizar_checker::source_structure::SourceStructureTermKind::FunctionalUpdate,
            ),
            (
                1,
                65,
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
        ]
    );
    assert_eq!(structure.source_id(), ast.source_id);
    assert_eq!(structure.module_id(), &module);
    assert_eq!(structure.primary_term_fingerprint(), primary.debug_text());
    assert!(structure.application_fingerprint().is_none());
    assert_eq!(
        structure
            .terms()
            .iter()
            .map(|(_, term)| (
                term.site().node().index(),
                term.source_range(),
                term.source_ordinal(),
                term.context().index(),
                term.recovery(),
                term.spelling(),
                term.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                69,
                range(ast.source_id, 120, 160),
                0,
                1,
                mizar_checker::source_structure::SourceStructureRecovery::Normal,
                "TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )",
                mizar_checker::source_structure::SourceStructureTermKind::FunctionalUpdate,
            ),
            (
                65,
                range(ast.source_id, 120, 146),
                1,
                1,
                mizar_checker::source_structure::SourceStructureRecovery::Normal,
                "TypeCaseStruct ( x : 1 , y : 2 )",
                mizar_checker::source_structure::SourceStructureTermKind::Constructor,
            ),
        ]
    );
    let root = structure
        .roots()
        .get(mizar_checker::source_structure::SourceStructureRootId::new(
            0,
        ))
        .expect("imported root");
    assert_eq!(root.term().index(), 1);
    assert_eq!(root.contribution().index(), 2);
    assert_eq!(
        root.symbol().local().as_str(),
        "summary:parser.type_fixtures#parse-only#TypeCaseStruct:5"
    );
    assert_eq!(root.symbol().module().package(), module.package());
    assert_eq!(
        root.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert_eq!(
        root.symbol().fqn().as_str(),
        "parser.type_fixtures::TypeCaseStruct#5"
    );
    assert_eq!(root.origin().structural_path(), [5]);
    assert_eq!(root.origin().source_id(), ast.source_id);
    assert_eq!(root.origin().module_id(), root.symbol().module());
    assert_eq!(
        root.origin().anchor(),
        &mizar_session::SourceAnchor::Range(range(ast.source_id, 7, 27))
    );
    assert!(root.origin().import_edge().is_none());
    assert!(!root.origin().is_recovered());
    assert_eq!(root.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        root.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert!(root.signature().is_none());
    assert_eq!(
        structure
            .members()
            .iter()
            .map(|(_, row)| (row.term().index(), row.site().node().index(), row.role()))
            .collect::<Vec<_>>(),
        [
            (
                0,
                30,
                mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment,
            ),
            (
                1,
                20,
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
            (
                1,
                24,
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
            ),
        ]
    );
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
                30,
                range(ast.source_id, 153, 154),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::UpdatePathSegment,
                None,
            ),
            (
                1,
                0,
                20,
                range(ast.source_id, 135, 136),
                "x",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
            (
                1,
                1,
                24,
                range(ast.source_id, 141, 142),
                "y",
                mizar_checker::source_structure::SourceStructureMemberRole::ConstructorAssignment,
                None,
            ),
        ]
    );
    let field_update = structure
        .field_updates()
        .get(mizar_checker::source_structure::SourceFieldUpdateId::new(0))
        .expect("field update");
    assert_eq!(field_update.site().node().index(), 68);
    assert_eq!(field_update.spelling(), "x := 3");
    assert_eq!(field_update.first_member().index(), 0);
    assert_eq!(field_update.final_member().index(), 0);
    assert_eq!(field_update.term().index(), 0);
    assert_eq!(field_update.ordinal(), 0);
    assert_eq!(field_update.source_range(), range(ast.source_id, 153, 159));
    assert_eq!(
        structure
            .edges()
            .iter()
            .map(|(_, edge)| (edge.term().index(), edge.role(), edge.target()))
            .collect::<Vec<_>>(),
        [
            (
                0,
                mizar_checker::source_structure::SourceStructureEdgeRole::UpdateBase,
                mizar_checker::source_structure::SourceStructureTarget::Structure(
                    mizar_checker::source_structure::SourceStructureTermId::new(1),
                ),
            ),
            (
                0,
                mizar_checker::source_structure::SourceStructureEdgeRole::UpdateValue,
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(4),
                ),
            ),
            (
                1,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
            (
                1,
                mizar_checker::source_structure::SourceStructureEdgeRole::ConstructorValue,
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                ),
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
                mizar_checker::source_structure::SourceStructureEdgeRole::UpdateBase,
                None,
                mizar_checker::source_structure::SourceStructureTarget::Structure(
                    mizar_checker::source_structure::SourceStructureTermId::new(1),
                ),
            ),
            (
                0,
                1,
                mizar_checker::source_structure::SourceStructureEdgeRole::UpdateValue,
                Some(0),
                mizar_checker::source_structure::SourceStructureTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(4),
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
    let atomic = output.typed_ast.source_atomic_formula().expect("Task256");
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| edge.target())
            .collect::<Vec<_>>(),
        [0, 1, 5, 6].map(|term| {
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(term),
            )
        })
    );
    assert_eq!(atomic.source_id(), ast.source_id);
    assert_eq!(atomic.module_id(), &module);
    assert_eq!(atomic.primary_term_fingerprint(), primary.debug_text());
    assert!(atomic.application_fingerprint().is_none());
    assert!(atomic.structure_fingerprint().is_none());
    assert!(atomic.set_term_fingerprint().is_none());
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
                55,
                range(ast.source_id, 101, 106),
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
                "x = x",
            ),
            (
                77,
                range(ast.source_id, 169, 174),
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
                    mizar_checker::source_term::SourcePrimaryTermId::new(5),
                ),
            ),
            (
                1,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(6),
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
    assert_eq!(statement.source_id(), ast.source_id);
    assert_eq!(statement.module_id(), &module);
    assert_eq!(statement.binding_fingerprint(), bindings.debug_text());
    assert_eq!(statement.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(statement.atomic_formula_fingerprint(), atomic.debug_text());
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
        ),
        (1, 2, 2, 2, 2)
    );
    let checked_owner = statement.checked_owner();
    let owner_row = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("Task258 owner");
    assert_eq!(owner_row.symbol(), checked_owner.symbol());
    assert_eq!(owner_row.symbol(), owner.symbol());
    assert_eq!(owner_row.contribution(), owner.contribution());
    assert_eq!(owner_row.site().node().index(), 82);
    assert_eq!(owner_row.source_range(), range(ast.source_id, 48, 180));
    assert_eq!(
        owner_row.spelling(),
        "FormulaStatementStructureUpdateWitnessSmoke"
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
    for index in 0..2 {
        let row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(
                index,
            ))
            .expect("Task258 statement row");
        assert_eq!(row.owner().index(), 0);
        assert_eq!(row.context().index(), index);
        assert_eq!(
            row.formula(),
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(row.site().node().index(), [82, 80][index]);
        assert_eq!(
            row.source_range(),
            [
                range(ast.source_id, 48, 180),
                range(ast.source_id, 164, 175),
            ][index]
        );
        assert_eq!(row.source_ordinal(), [0, 2][index]);
        assert_eq!(
            row.spelling(),
            [
                "theorem FormulaStatementStructureUpdateWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 ) ; thus x = x ; end ;",
                "thus x = x ;",
            ][index]
        );
        assert_eq!(
            row.kind(),
            [
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
            ][index]
        );
        assert_eq!(
            row.recovery(),
            mizar_checker::source_statement::SourceStatementRecovery::Normal
        );
        let context = statement
            .contexts()
            .get(mizar_checker::source_statement::SourceStatementContextId::new(index))
            .expect("Task258 context row");
        assert_eq!(context.statement().index(), index);
        assert_eq!(context.binding_context().index(), index);
        assert_eq!(context.source_range(), row.source_range());
        assert_eq!(
            context.visible_bindings(),
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        let input_fact = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("Task258 input fact");
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
        let candidate = statement
            .candidate_facts()
            .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(index))
            .expect("Task258 candidate fact");
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
        .expect("B2C witness");
    assert_eq!(
        (witnesses.witnesses().len(), witnesses.names().len()),
        (1, 0)
    );
    let witness = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
        .expect("witness row");
    assert_eq!(witness.take_site().node().index(), 72);
    assert_eq!(witness.take_range(), range(ast.source_id, 115, 161));
    assert_eq!(witness.site().node().index(), 71);
    assert_eq!(witness.source_range(), range(ast.source_id, 120, 160));
    assert_eq!(
        witness.term(),
        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Structure(
            mizar_checker::source_structure::SourceStructureTermId::new(0),
        )
    );
    assert_eq!(witness.owner().index(), 0);
    assert_eq!(witness.binding_context().index(), 1);
    assert_eq!(witness.source_ordinal(), 1);
    assert_eq!(witness.ordinal(), 0);
    assert_eq!(
        witness.spelling(),
        "TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )"
    );
    assert_eq!(
        witness.kind(),
        mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed
    );
    assert_eq!(
        witness.recovery(),
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    assert!(witness.name().is_none());
    assert_eq!(witnesses.source_id(), ast.source_id);
    assert_eq!(witnesses.module_id(), &module);
    assert_eq!(witnesses.statement_fingerprint(), statement.debug_text());
    assert_eq!(witnesses.primary_term_fingerprint(), primary.debug_text());
    assert!(witnesses.application_fingerprint().is_none());
    let structure_debug = structure.debug_text();
    assert_eq!(
        witnesses.structure_fingerprint(),
        Some(structure_debug.as_str())
    );
    assert_eq!(output.reference_use_ordinals, [1; 4]);
    let witness_debug = witnesses.debug_text();
    let debug_lines = witness_debug.lines().collect::<Vec<_>>();
    let statement_fingerprint = debug_lines
        .iter()
        .position(|line| line.starts_with("statement-fingerprint:"))
        .expect("statement fingerprint");
    let primary_fingerprint = debug_lines
        .iter()
        .position(|line| line.starts_with("primary-term-fingerprint:"))
        .expect("primary fingerprint");
    let structure_fingerprint = debug_lines
        .iter()
        .position(|line| line.starts_with("structure-fingerprint:"))
        .expect("structure fingerprint");
    let witness_row = debug_lines
        .iter()
        .position(|line| line.starts_with("witness#0 "))
        .expect("witness row");
    assert!(statement_fingerprint < primary_fingerprint);
    assert!(primary_fingerprint < structure_fingerprint);
    assert!(structure_fingerprint < witness_row);
    assert!(!witness_debug.contains("\napplication-fingerprint:"));
    assert!(witness_debug.contains(" term=structure#0 "));
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
    );
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    for (id, node) in output.typed_ast.nodes().iter() {
        assert_eq!(
            node.kind.as_str(),
            match id.index() {
                20 | 24 => "source.term.structure.member.constructor-assignment",
                30 => "source.term.structure.member.update-path-segment",
                51 | 53 | 73 | 75 => "source.term.variable-reference",
                59 | 62 | 66 => "source.term.numeral",
                55 | 77 => "source.formula.atomic.equality",
                65 => "source.term.structure.constructor",
                68 => "source.term.structure.field-update",
                69 => "source.term.structure.update",
                71 => "source.statement-witness.item",
                72 => "source.statement-witness.take",
                80 => "source.statement.conclusion",
                82 => "source.statement.theorem",
                _ => "source.surface.unowned",
            },
            "node {} kind",
            id.index()
        );
    }
}

#[test]
fn task258b3m2b2b2c_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols) = task258b3m2b2b2c_fixture(259_301);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
    )
    .expect("baseline selector")
    .expect("baseline route");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let (stale_ast, stale_module, stale_symbols) = task258b3m2b2b2c_fixture(259_309);
    let stale = source_statement_output_with_source(
        &stale_ast,
        stale_module,
        &stale_symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
    )
    .expect("stale selector")
    .expect("stale route");
    let stale_binding = stale
        .typed_ast
        .source_statement()
        .unwrap()
        .binding_env()
        .clone();
    let stale_primary = stale.typed_ast.source_term().unwrap().clone();
    let stale_atomic = stale.typed_ast.source_atomic_formula().unwrap().clone();
    let stale_structure = stale.typed_ast.source_structure().unwrap().clone();
    for mutation in 0..27 {
        let stale_binding = stale_binding.clone();
        let stale_primary = stale_primary.clone();
        let stale_atomic = stale_atomic.clone();
        let stale_structure = stale_structure.clone();
        let error = source_statement_b3m2b2b2c_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B2C_TEXT,
            move |input: &mut SourceStatementB3M2B2B2CRouteInputs| match mutation {
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
                            mizar_checker::source_term::SourcePrimaryTermId::new(4),
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
            },
        )
        .expect("mutation selector")
        .expect_err("mutation must fail closed");
        assert!(!error.is_empty(), "mutation {mutation}");
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B2C_TEXT,
        )
        .expect("clean replay selector")
        .expect("clean replay");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    }

    let resolver = source_statement_b3m2b2b2c_resolver_env_for_test(
        &module,
        &symbols,
        range(ast.source_id, 56, 99),
    )
    .expect("resolver");
    let binding_env = baseline
        .typed_ast
        .source_statement()
        .expect("statement")
        .binding_env();
    let lower = imported_structure_update_handoff_for_test(
        &ast,
        &module,
        &resolver,
        binding_env,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
        &task258b3m2b2b2c_roots(),
        ImportedStructureUpdateTestOptions {
            update: 69,
            context: mizar_checker::binding_env::BindingContextId::new(1),
            surface_mutation: ImportedStructureUpdateSurfaceMutation::DirectProductionSeam,
            handoff_mutation: ImportedStructureUpdateTestMutation::None,
        },
    )
    .expect("private update seam")
    .expect("private update handoff");
    assert_eq!(lower.primary_counts, (7, 4, 3));
    assert_eq!(
        lower.handoff,
        *baseline.typed_ast.source_structure().unwrap()
    );
    let assert_lower_rejects = |mutation| {
        assert!(
            !matches!(
                imported_structure_update_handoff_for_test(
                    &ast,
                    &module,
                    &resolver,
                    binding_env,
                    SOURCE_STATEMENT_B3M2B2B2C_TEXT,
                    &task258b3m2b2b2c_roots(),
                    ImportedStructureUpdateTestOptions {
                        update: 69,
                        context: mizar_checker::binding_env::BindingContextId::new(1),
                        surface_mutation: ImportedStructureUpdateSurfaceMutation::None,
                        handoff_mutation: mutation,
                    },
                ),
                Some(Ok(_))
            ),
            "lower mutation {mutation:?}"
        );
    };
    for mutation in [
        ImportedStructureUpdateTestMutation::BindingSourceId,
        ImportedStructureUpdateTestMutation::BindingModuleId,
        ImportedStructureUpdateTestMutation::BindingContextCount,
        ImportedStructureUpdateTestMutation::BindingCount,
        ImportedStructureUpdateTestMutation::BindingDiagnosticCount,
        ImportedStructureUpdateTestMutation::ModuleContextOwner,
        ImportedStructureUpdateTestMutation::ModuleContextParent,
        ImportedStructureUpdateTestMutation::ModuleContextLayer,
        ImportedStructureUpdateTestMutation::ModuleContextScope,
        ImportedStructureUpdateTestMutation::ModuleContextBindings,
        ImportedStructureUpdateTestMutation::ModuleContextVisibleBindings,
        ImportedStructureUpdateTestMutation::ModuleContextRecovery,
        ImportedStructureUpdateTestMutation::ProofContextOwner,
        ImportedStructureUpdateTestMutation::ProofContextParent,
        ImportedStructureUpdateTestMutation::ProofContextLayer,
        ImportedStructureUpdateTestMutation::ProofContextScope,
        ImportedStructureUpdateTestMutation::ProofContextBindings,
        ImportedStructureUpdateTestMutation::ProofContextVisibleBindings,
        ImportedStructureUpdateTestMutation::ProofContextRecovery,
        ImportedStructureUpdateTestMutation::BindingSpelling,
        ImportedStructureUpdateTestMutation::BindingKind,
        ImportedStructureUpdateTestMutation::BindingIdentityKind,
        ImportedStructureUpdateTestMutation::BindingIdentitySpelling,
        ImportedStructureUpdateTestMutation::BindingIdentityRange,
        ImportedStructureUpdateTestMutation::BindingOwner,
        ImportedStructureUpdateTestMutation::BindingDeclarationRange,
        ImportedStructureUpdateTestMutation::BindingVisibleOrdinal,
        ImportedStructureUpdateTestMutation::BindingTypeSite,
        ImportedStructureUpdateTestMutation::BindingStatus,
        ImportedStructureUpdateTestMutation::BindingCaptured,
        ImportedStructureUpdateTestMutation::BindingDiagnostics,
        ImportedStructureUpdateTestMutation::BindingRecovery,
        ImportedStructureUpdateTestMutation::PrimarySourceId,
        ImportedStructureUpdateTestMutation::PrimaryModuleId,
        ImportedStructureUpdateTestMutation::RootTerm,
        ImportedStructureUpdateTestMutation::RootSymbol,
        ImportedStructureUpdateTestMutation::RootContribution,
        ImportedStructureUpdateTestMutation::FieldUpdateMissing,
        ImportedStructureUpdateTestMutation::FieldUpdateExtra,
        ImportedStructureUpdateTestMutation::FieldUpdateTerm(0),
        ImportedStructureUpdateTestMutation::FieldUpdateOrdinal(0),
        ImportedStructureUpdateTestMutation::FieldUpdateSite(0),
        ImportedStructureUpdateTestMutation::FieldUpdateRange(0),
        ImportedStructureUpdateTestMutation::FieldUpdateSpelling(0),
        ImportedStructureUpdateTestMutation::FieldUpdateFirstMember(0),
        ImportedStructureUpdateTestMutation::FieldUpdateFinalMember(0),
        ImportedStructureUpdateTestMutation::StalePrimaryReplay,
        ImportedStructureUpdateTestMutation::TermRangeAndStalePrimaryReplay,
    ] {
        assert_lower_rejects(mutation);
    }
    for index in 0..7 {
        for mutation in [
            ImportedStructureUpdateTestMutation::PrimaryTermSite(index),
            ImportedStructureUpdateTestMutation::PrimaryTermRange(index),
            ImportedStructureUpdateTestMutation::PrimaryTermOrdinal(index),
            ImportedStructureUpdateTestMutation::PrimaryTermContext(index),
            ImportedStructureUpdateTestMutation::PrimaryTermRecovery(index),
            ImportedStructureUpdateTestMutation::PrimaryTermSpelling(index),
            ImportedStructureUpdateTestMutation::PrimaryTermKind(index),
            ImportedStructureUpdateTestMutation::PrimaryTermRole(index),
            ImportedStructureUpdateTestMutation::PrimaryTermParent(index),
        ] {
            assert_lower_rejects(mutation);
        }
    }
    for index in 0..4 {
        for mutation in [
            ImportedStructureUpdateTestMutation::PrimaryReferenceTerm(index),
            ImportedStructureUpdateTestMutation::PrimaryReferenceBinding(index),
            ImportedStructureUpdateTestMutation::PrimaryReferenceRole(index),
            ImportedStructureUpdateTestMutation::PrimaryReferenceUseOrdinal(index),
            ImportedStructureUpdateTestMutation::PrimaryReferenceScope(index),
            ImportedStructureUpdateTestMutation::EdgeTerm(index),
            ImportedStructureUpdateTestMutation::EdgeOrdinal(index),
            ImportedStructureUpdateTestMutation::EdgeRole(index),
            ImportedStructureUpdateTestMutation::EdgeMember(index),
            ImportedStructureUpdateTestMutation::EdgeTarget(index),
        ] {
            assert_lower_rejects(mutation);
        }
    }
    for index in 0..3 {
        for mutation in [
            ImportedStructureUpdateTestMutation::NumericRequestTerm(index),
            ImportedStructureUpdateTestMutation::NumericRequestOwner(index),
            ImportedStructureUpdateTestMutation::NumericRequestRange(index),
            ImportedStructureUpdateTestMutation::NumericRequestSpelling(index),
            ImportedStructureUpdateTestMutation::NumericRequestOrdinal(index),
            ImportedStructureUpdateTestMutation::MemberTerm(index),
            ImportedStructureUpdateTestMutation::MemberOrdinal(index),
            ImportedStructureUpdateTestMutation::MemberSite(index),
            ImportedStructureUpdateTestMutation::MemberRange(index),
            ImportedStructureUpdateTestMutation::MemberSpelling(index),
            ImportedStructureUpdateTestMutation::MemberRole(index),
            ImportedStructureUpdateTestMutation::MemberParent(index),
        ] {
            assert_lower_rejects(mutation);
        }
    }
    for index in 0..2 {
        for mutation in [
            ImportedStructureUpdateTestMutation::TermSite(index),
            ImportedStructureUpdateTestMutation::TermRange(index),
            ImportedStructureUpdateTestMutation::TermOrdinal(index),
            ImportedStructureUpdateTestMutation::TermContext(index),
            ImportedStructureUpdateTestMutation::TermRecovery(index),
            ImportedStructureUpdateTestMutation::TermSpelling(index),
            ImportedStructureUpdateTestMutation::TermKind(index),
        ] {
            assert_lower_rejects(mutation);
        }
    }
    for index in 0..9 {
        for mutation in [
            ImportedStructureUpdateTestMutation::RequestTerm(index),
            ImportedStructureUpdateTestMutation::RequestOrdinal(index),
            ImportedStructureUpdateTestMutation::RequestMember(index),
            ImportedStructureUpdateTestMutation::RequestKind(index),
        ] {
            assert_lower_rejects(mutation);
        }
    }
    let error = source_statement_b3m2b2b2c_output_with_mutation(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
        |input| {
            input.structure = None;
            input.statement.owners.clear();
            input.witness.witnesses.clear();
        },
    )
    .expect("precedence selector")
    .expect_err("lower dependency must precede upper aggregates");
    assert!(error.to_ascii_lowercase().contains("dependency"), "{error}");
}

#[test]
fn task258b3m2b2b2c_update_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols) = task258b3m2b2b2c_fixture(259_302);
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B2C_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B2C_TEXT.as_bytes().to_vec();
        bytes[byte_index] = if bytes[byte_index] == b'!' {
            b'?'
        } else {
            b'!'
        };
        let changed = String::from_utf8(bytes).expect("ASCII mutation");
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &changed).is_none(),
            "byte {byte_index}"
        );
    }
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
    )
    .expect("baseline selector")
    .expect("baseline");
    let resolver = source_statement_b3m2b2b2c_resolver_env_for_test(
        &module,
        &symbols,
        range(ast.source_id, 56, 99),
    )
    .expect("resolver");
    let binding_env = baseline.typed_ast.source_statement().unwrap().binding_env();
    for node in 0..86 {
        for surface_mutation in [
            ImportedStructureUpdateSurfaceMutation::NodeKind(node),
            ImportedStructureUpdateSurfaceMutation::NodeRange(node),
            ImportedStructureUpdateSurfaceMutation::NodeRecovery(node),
            ImportedStructureUpdateSurfaceMutation::NodeChildren(node),
        ] {
            assert!(
                imported_structure_update_handoff_for_test(
                    &ast,
                    &module,
                    &resolver,
                    binding_env,
                    SOURCE_STATEMENT_B3M2B2B2C_TEXT,
                    &task258b3m2b2b2c_roots(),
                    ImportedStructureUpdateTestOptions {
                        update: 69,
                        context: mizar_checker::binding_env::BindingContextId::new(1),
                        surface_mutation,
                        handoff_mutation: ImportedStructureUpdateTestMutation::None,
                    },
                )
                .is_none(),
                "node {node} {surface_mutation:?}"
            );
        }
    }
    for update in [65, 68, 70, usize::MAX] {
        assert!(
            imported_structure_update_handoff_for_test(
                &ast,
                &module,
                &resolver,
                binding_env,
                SOURCE_STATEMENT_B3M2B2B2C_TEXT,
                &task258b3m2b2b2c_roots(),
                ImportedStructureUpdateTestOptions {
                    update,
                    context: mizar_checker::binding_env::BindingContextId::new(1),
                    surface_mutation: ImportedStructureUpdateSurfaceMutation::None,
                    handoff_mutation: ImportedStructureUpdateTestMutation::None,
                },
            )
            .is_none(),
            "wrong update {update}"
        );
    }
    assert!(
        imported_structure_update_handoff_for_test(
            &ast,
            &module,
            &resolver,
            binding_env,
            SOURCE_STATEMENT_B3M2B2B2C_TEXT,
            &task258b3m2b2b2c_roots(),
            ImportedStructureUpdateTestOptions {
                update: 69,
                context: mizar_checker::binding_env::BindingContextId::new(1),
                surface_mutation: ImportedStructureUpdateSurfaceMutation::RootIdentity,
                handoff_mutation: ImportedStructureUpdateTestMutation::None,
            },
        )
        .is_none(),
        "root identity mutation"
    );

    let missing = SOURCE_STATEMENT_B3M2B2B2C_TEXT.replacen("with (x := 3);", "with (x := );", 1);
    assert_eq!(missing.len(), 180);
    assert_eq!(
        sha256_text(&missing),
        "8310de3b172cea98e4e85ebc6021c85c4e1bd7c2a74f8cd99413ae5a80569d67"
    );
    assert_eq!(
        task258b3m2b2b2bp_frontend_diagnostic_profile(&missing, 259_303),
        [("malformed_term_expression".to_owned(), 158, 159)]
    );
    let (missing_ast, missing_module, missing_shells, missing_symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&missing, 259_303);
    assert_eq!(diagnostics, 1);
    assert_eq!(
        (
            missing_ast.nodes().len(),
            missing_ast.root().map(|root| root.index()),
            missing_ast
                .nodes()
                .iter()
                .enumerate()
                .filter_map(|(index, node)| node.recovered.then_some(index))
                .collect::<Vec<_>>(),
        ),
        (84, Some(83), vec![65])
    );
    let missing_symbols =
        augment_type_elaboration_import_summaries(&missing_ast, &missing_module, missing_symbols);
    assert!(extract_structure_update_witness_source_statement(&missing_ast, &missing).is_none());
    assert!(
        source_statement_output_with_source(
            &missing_ast,
            missing_module.clone(),
            &missing_symbols,
            &missing,
        )
        .is_none()
    );
    assert!(
        source_structure_output(
            &missing_ast,
            missing_module,
            &missing_shells,
            &missing_symbols
        )
        .is_none()
    );

    for (ordinal, (changed, expected)) in [
        (
            SOURCE_STATEMENT_B3M2B2B2C_TEXT.replacen(
                "TypeCaseStruct(x: 1, y: 2) with (x := 3)",
                "TypeCaseStruct(x: 1, y: 2)",
                1,
            ),
            (
                167,
                "bb26a425d2bc16e6518d6366128de138862c4525af6eb82b748e4cb28f1b8bc9",
                76,
                75,
            ),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B2C_TEXT.replacen(
                "TypeCaseStruct(x: 1, y: 2) with (x := 3)",
                "TypeCaseStruct(x: 1, y: 2).x",
                1,
            ),
            (
                169,
                "64039fca35d6199fea281d43df6dafdfeff78f1d97139d6286a3082115552747",
                79,
                78,
            ),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B2C_TEXT.replacen(
                "TypeCaseStruct(x: 1, y: 2) with (x := 3)",
                "(TypeCaseStruct(x: 1, y: 2) with (x := 3))",
                1,
            ),
            (
                183,
                "e1a2b79cb03a4aebc5e0e29150cde382da457aa31cb8e66643eecce6e8296ae6",
                90,
                89,
            ),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B2C_TEXT.replacen("with (x := 3)", "with (x := 3, y := 4)", 1),
            (
                189,
                "a95336dc08b9534d7c5c16ca5070384e2610f0db31841187878b68b4403666b6",
                93,
                92,
            ),
        ),
        (
            SOURCE_STATEMENT_B3M2B2B2C_TEXT.replacen("with (x := 3)", "with (x.y := 3)", 1),
            (
                183,
                "92440b4b3814d7b8a738bf71b2e89b9056fbb382301e12b5f4a4ccab17e0f082",
                88,
                87,
            ),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&changed, 259_310 + ordinal);
        assert_eq!(
            (
                changed.len(),
                sha256_text(&changed),
                diagnostics,
                near_ast.nodes().len(),
                near_ast.root().map(|root| root.index()),
            ),
            (
                expected.0,
                expected.1.to_owned(),
                0,
                expected.2,
                Some(expected.3)
            )
        );
        assert!(near_ast.nodes().iter().all(|node| !node.recovered));
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_symbols);
        assert!(extract_structure_update_witness_source_statement(&near_ast, &changed).is_none());
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_symbols,
                &changed,
            )
            .is_none()
        );
        assert!(
            source_structure_output(&near_ast, near_module, &near_shells, &near_symbols).is_none()
        );
    }
}

#[test]
fn task258b3m2b2b2c_family_and_active_route_isolation_is_atomic() {
    let (ast, module, symbols) = task258b3m2b2b2c_fixture(259_320);
    for mutation in [
        Task258B3M2B2B2AResolverMutation::LocalAndFqn,
        Task258B3M2B2B2AResolverMutation::StructuralPath,
        Task258B3M2B2B2AResolverMutation::Signature,
        Task258B3M2B2B2AResolverMutation::ExportStatus,
        Task258B3M2B2B2AResolverMutation::Contribution,
    ] {
        let substituted = task258b3m2b2b2a_substituted_structure_env(&symbols, mutation);
        assert!(
            !matches!(
                source_statement_output_with_source(
                    &ast,
                    module.clone(),
                    &substituted,
                    SOURCE_STATEMENT_B3M2B2B2C_TEXT,
                ),
                Some(Ok(_))
            ),
            "structure resolver mutation {mutation:?}"
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
        assert!(
            !matches!(
                source_statement_b3m2b2b2c_output_with_resolver_mutation(
                    &ast,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B3M2B2B2C_TEXT,
                    |symbols| task258b2_mutate_resolver(symbols, mutation),
                ),
                Some(Ok(_))
            ),
            "theorem resolver mutation {mutation:?}"
        );
    }

    let update_structure = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
    )
    .expect("update selector")
    .expect("update route")
    .typed_ast
    .source_structure()
    .expect("update structure")
    .clone();
    let (constructor_ast, constructor_module, constructor_symbols) =
        task258b3m2b2b2a_fixture(259_321);
    let constructor_structure = source_statement_output_with_source(
        &constructor_ast,
        constructor_module.clone(),
        &constructor_symbols,
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
    )
    .expect("constructor selector")
    .expect("constructor")
    .typed_ast
    .source_structure()
    .unwrap()
    .clone();
    let (selector_ast, selector_module, selector_symbols) = task258b3m2b2b2b_fixture(259_322);
    let selector_structure = source_statement_output_with_source(
        &selector_ast,
        selector_module.clone(),
        &selector_symbols,
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
    )
    .expect("selector selector")
    .expect("selector")
    .typed_ast
    .source_structure()
    .unwrap()
    .clone();
    for foreign in [constructor_structure, selector_structure] {
        let error = source_statement_b3m2b2b2c_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B2C_TEXT,
            |input| input.structure = Some(foreign),
        )
        .expect("hybrid selector")
        .expect_err("foreign structure must fail");
        assert!(error.to_ascii_lowercase().contains("dependency"), "{error}");
    }
    for (family_ast, family_module, family_symbols, source) in [
        (
            constructor_ast,
            constructor_module,
            constructor_symbols,
            SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        ),
        (
            selector_ast,
            selector_module,
            selector_symbols,
            SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        ),
    ] {
        let result = if source == SOURCE_STATEMENT_B3M2B2B2A_TEXT {
            source_statement_b3m2b2b2a_output_with_mutation(
                &family_ast,
                family_module,
                &family_symbols,
                source,
                |input| input.structure = Some(update_structure.clone()),
            )
        } else {
            source_statement_b3m2b2b2b_output_with_mutation(
                &family_ast,
                family_module,
                &family_symbols,
                source,
                |input| input.structure = Some(update_structure.clone()),
            )
        };
        assert!(matches!(result, Some(Err(_))), "reverse family hybrid");
    }

    let (application_ast, application_module, application_symbols) =
        task258b3m2b2b1a_fixture(259_323);
    let application = source_statement_output_with_source(
        &application_ast,
        application_module,
        &application_symbols,
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    )
    .expect("application selector")
    .expect("application route")
    .typed_ast
    .source_application()
    .expect("application handoff")
    .clone();
    let error = source_statement_b3m2b2b2c_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
        |input| input.application = Some(application),
    )
    .expect("application/update selector")
    .expect_err("application/update hybrid must fail");
    assert!(error.to_ascii_lowercase().contains("dependency"), "{error}");

    for (source, expected_application, expected_terms, expected_updates) in [
        (SOURCE_STATEMENT_B3_TEXT, false, None, None),
        (SOURCE_STATEMENT_B3N_TEXT, false, None, None),
        (SOURCE_STATEMENT_B3M1_TEXT, false, None, None),
        (SOURCE_STATEMENT_B3M2A_TEXT, false, None, None),
        (SOURCE_STATEMENT_B3M2B1_TEXT, false, None, None),
        (SOURCE_STATEMENT_B3M2B2A_TEXT, false, None, None),
        (SOURCE_STATEMENT_B3M2B2B1A_TEXT, true, None, None),
        (SOURCE_STATEMENT_B3M2B2B1B1_TEXT, true, None, None),
        (SOURCE_STATEMENT_B3M2B2B2A_TEXT, false, Some(1), Some(0)),
        (SOURCE_STATEMENT_B3M2B2B2B_TEXT, false, Some(2), Some(0)),
        (SOURCE_STATEMENT_B3M2B2B2C_TEXT, false, Some(2), Some(1)),
    ] {
        let (family_ast, family_module, _, family_symbols) =
            task253_ast_from_source_text(source, 259_330 + source.len());
        let family_symbols =
            augment_type_elaboration_import_summaries(&family_ast, &family_module, family_symbols);
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
        assert_eq!(
            family_typed
                .source_structure()
                .map(|structure| structure.field_updates().len()),
            expected_updates
        );
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("B2C isolation plan");
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
        if extract_structure_update_witness_source_statement(&active_ast, &source).is_some()
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
                        .is_some_and(|structure| {
                            structure.terms().len() == 2 && structure.field_updates().len() == 1
                        })
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
        "B2C selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b2b2c_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, symbols) = task258b3m2b2b2c_fixture(259_350);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
    )
    .expect("selector")
    .expect("route");
    assert_eq!(output.typed_ast.clone(), output.typed_ast);
    assert_eq!(output.resolved.clone(), output.resolved);
    assert_eq!(
        output.typed_ast.clone().debug_text(),
        output.typed_ast.debug_text()
    );
    assert_eq!(
        output.resolved.clone().debug_text(),
        output.resolved.debug_text()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert_eq!(
        output.typed_ast.source_atomic_formula(),
        output.resolved.source_atomic_formula()
    );
    assert_eq!(
        output.typed_ast.source_structure(),
        output.resolved.source_structure()
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
    assert!(output.typed_ast.source_statement_references().is_none());
    assert!(output.typed_ast.source_context().is_none());
    assert!(output.typed_ast.source_type().is_none());
    assert!(output.typed_ast.source_attribute().is_none());
    assert!(output.typed_ast.source_evidence().is_none());
    assert!(output.typed_ast.source_set_term().is_none());
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

    let baseline = output.typed_ast.debug_text();
    let error = source_statement_b3m2b2b2c_output_with_mutation(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
        |input| input.witness.witnesses.clear(),
    )
    .expect("rollback selector")
    .expect_err("empty witness aggregate must roll back");
    assert!(error.to_ascii_lowercase().contains("aggregate"), "{error}");
    assert_eq!(output.typed_ast.debug_text(), baseline);
}
