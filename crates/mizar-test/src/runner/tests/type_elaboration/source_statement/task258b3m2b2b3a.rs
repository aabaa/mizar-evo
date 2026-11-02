#[test]
fn task258b3m2b2b3a_actual_active_plan_has_no_numeric_enumeration_witness_route() {
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
    let plan = build_test_plan(&config).expect("Task258B3M2B2B3A active isolation plan");
    let (mut scanned, mut with_ast, mut resolved) = (0, 0, 0);
    let (mut extracted, mut selected) = (Vec::new(), Vec::new());
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        scanned += 1;
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            // Frontend refusal cases may have no AST to pass to the resolver.
            continue;
        };
        with_ast += 1;
        if extract_set_enumeration_witness_source_statement(&ast, &source).is_some() {
            extracted.push(case.id.0.clone());
        }
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            // Resolver refusal cases cannot provide a production elaboration environment.
            continue;
        }
        resolved += 1;
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let selected_route =
            source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
                .is_some_and(|result| {
                    result.is_ok_and(|output| {
                        output.typed_ast.source_statement_witnesses().is_some_and(|witnesses| {
                            witnesses.witnesses().len() == 1
                                && witnesses.names().is_empty()
                                && witnesses.witnesses().iter().all(|(_, witness)| {
                                    witness.kind() == mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed
                                })
                        }) && output.typed_ast.source_set_term().is_some_and(|set| {
                            set.terms().len() == 1
                                && set.terms().iter().all(|(_, term)| {
                                    term.kind() == mizar_checker::source_set_term::SourceSetTermKind::Enumeration
                                })
                        }) && output.typed_ast.source_term().is_some_and(|primary| {
                            primary.numeric_type_requests().len() == 2
                        })
                    })
                });
        if selected_route {
            selected.push(case.id.0.clone());
        }
    }
    eprintln!(
        "Numeric enumeration active isolation: scanned={scanned} with_ast={with_ast} resolved={resolved} extracted={} selected={}",
        extracted.len(),
        selected.len()
    );
    assert!(
        scanned > 0 && with_ast > 0 && resolved > 0,
        "empty active scan"
    );
    assert!(
        extracted.is_empty(),
        "Numeric enumeration extracted active cases: {extracted:?}"
    );
    assert!(
        selected.is_empty(),
        "Numeric enumeration selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b2b3a_real_frontend_freezes_set_enumeration_witness_contract() {
    use mizar_checker::{
        source_set_term::{
            SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermId,
            SourceSetTermKind, SourceSetTermRecovery,
        },
        source_statement::{
            SourceStatementKind, SourceStatementRecovery, SourceStatementWitnessKind,
            SourceStatementWitnessTermTarget,
        },
        source_term::{SourcePrimaryTermId, SourcePrimaryTermKind},
    };

    assert_eq!(SOURCE_STATEMENT_B3M2B2B3A_TEXT.len(), 117);
    assert!(SOURCE_STATEMENT_B3M2B2B3A_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B2B3A_TEXT),
        "4f8ea5b9cadf763ea108b6f7deb6b481cb6f997dec2048b4351f07fd5dc38539"
    );
    let (ast, module, symbols, output) = task258b3m2b2b3a_lower_inputs(259_400);
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

    let statements = &output.statement;
    let bindings = &output.binding_env;
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

    let set = output.set_term.as_ref().expect("Task255");
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

    assert_eq!(statements.source_id, ast.source_id);
    assert_eq!(&statements.module_id, &module);
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
    let owner_row = statements.owners.first().expect("Task258 owner");
    assert_eq!(&owner_row.symbol, owner.symbol());
    assert_eq!(owner_row.contribution, owner.contribution());
    assert_eq!(owner_row.site.node().index(), 53);
    assert_eq!(owner_row.source_range, range(ast.source_id, 19, 116));
    assert_eq!(
        owner_row.spelling,
        "FormulaStatementSetEnumerationWitnessSmoke"
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
    assert_eq!(
        statements
            .statements
            .iter()
            .enumerate()
            .map(|(id, row)| (id, row.site.node().index(), row.context.index(), row.kind))
            .collect::<Vec<_>>(),
        [
            (0, 53, 0, SourceStatementKind::TheoremProposition),
            (1, 51, 1, SourceStatementKind::Conclusion),
        ]
    );
    for index in 0..2 {
        let row = statements.statements.get(index).expect("statement row");
        assert_eq!(row.owner.index(), 0);
        assert_eq!(row.context.index(), index);
        assert_eq!(
            row.formula,
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(row.site.node().index(), [53, 51][index]);
        assert_eq!(
            row.source_range,
            [
                range(ast.source_id, 19, 116),
                range(ast.source_id, 100, 111),
            ][index]
        );
        assert_eq!(row.source_ordinal, [0, 2][index]);
        assert_eq!(
            row.spelling,
            [
                "theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;",
                "thus x = x ;",
            ][index]
        );
        assert_eq!(
            row.kind,
            [
                SourceStatementKind::TheoremProposition,
                SourceStatementKind::Conclusion,
            ][index]
        );
        assert_eq!(row.recovery, SourceStatementRecovery::Normal);
        let context = statements.contexts.get(index).expect("statement context");
        assert_eq!(context.statement.index(), index);
        assert_eq!(context.binding_context.index(), index);
        assert_eq!(context.source_range, row.source_range);
        assert_eq!(
            context.visible_bindings,
            [mizar_checker::binding_env::BindingId::new(0)]
        );
        let input_fact = statements.input_facts.get(index).expect("input fact");
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
        let candidate = statements
            .candidate_facts
            .get(index)
            .expect("candidate fact");
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
    assert_eq!((witnesses.witnesses.len(), witnesses.names.len()), (1, 0));
    let witness = witnesses.witnesses.first().expect("witness");
    assert_eq!(
        witness.term,
        SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(0))
    );
    assert_eq!(witness.owner.index(), 0);
    assert_eq!(witness.take_site.node().index(), 43);
    assert_eq!(witness.take_range, range(ast.source_id, 85, 97));
    assert_eq!(witness.site.node().index(), 42);
    assert_eq!(witness.source_range, range(ast.source_id, 90, 96));
    assert_eq!(witness.binding_context.index(), 1);
    assert_eq!(witness.source_ordinal, 1);
    assert_eq!(witness.ordinal, 0);
    assert_eq!(witness.spelling, "{ 1 , 2 }");
    assert_eq!(witness.kind, SourceStatementWitnessKind::Unnamed);
    assert_eq!(witness.recovery, SourceStatementRecovery::Normal);
    assert!(witness.name.is_none());
    assert_eq!(witnesses.source_id, ast.source_id);
    assert_eq!(&witnesses.module_id, &module);
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| row.use_ordinal())
            .collect::<Vec<_>>(),
        [1; 4]
    );

    let owned = output
        .arena
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
}

#[test]
fn task258b3m2b2b3a_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, symbols, baseline) = task258b3m2b2b3a_lower_inputs(259_401);
    let (_, _, _, stale) = task258b3m2b2b3a_lower_inputs(259_402);
    let baseline_debug = format!("{baseline:?}");
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let replay = || {
        let (_, _, _, pristine) = task258b3m2b2b3a_lower_inputs(259_401);
        assert_eq!(format!("{pristine:?}"), baseline_debug);
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
    };
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
                1 => input.primary = stale.primary.clone(),
                2 => input.atomic = stale.atomic.clone(),
                3 => input.set_term = stale.set_term.clone(),
                4 => input.binding_env = stale.binding_env.clone(),
                _ => unreachable!(),
            },
        )
        .expect("selector")
        .expect_err("stale lower dependency must fail");
        assert!(
            error.to_ascii_lowercase().contains("dependency"),
            "{mutation}: {error}"
        );
        replay();
    }
    let foreign_source = task258b3m2b2b3a_foreign_source_id(ast.source_id);
    let mut builder = mizar_syntax::SurfaceAstBuilder::new(foreign_source);
    let mut rebuilt = Vec::<mizar_syntax::SurfaceBuilderNodeId>::new();
    for node in ast.nodes() {
        let source_range = range(foreign_source, node.range.start, node.range.end);
        let children = node
            .children
            .iter()
            .map(|id| rebuilt[id.index()])
            .collect::<Vec<_>>();
        rebuilt.push(match &node.kind {
            mizar_syntax::SurfaceNodeKind::Token(token) => {
                builder.add_token(token.kind, token.text.clone(), source_range)
            }
            kind => builder.add_node(kind.clone(), source_range, children),
        });
    }
    let foreign_ast = builder.finish(
        ast.root().map(|id| rebuilt[id.index()]),
        ast.expression_root().map(|id| rebuilt[id.index()]),
    );
    assert_ne!(foreign_ast.source_id, ast.source_id);
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .expect("actual theorem owner");
    assert_eq!(
        source_statement_b3m2b2b3a_output_with_mutation(
            &foreign_ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            |_| panic!("foreign source reached lower capture"),
        )
        .expect("authentic foreign-source surface")
        .expect_err("source owner preflight"),
        mizar_checker::type_checker::StatementOwnerError::InvalidOwner {
            symbol: owner.symbol().clone()
        }
        .to_string()
    );
    replay();
    let foreign_module = ResolverModuleId::new(
        module.package().clone(),
        mizar_session::ModulePath::new("foreign.enumeration"),
    );
    assert_eq!(
        source_statement_b3m2b2b3a_output_with_mutation(
            &ast,
            foreign_module,
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            |_| panic!("foreign module reached lower capture"),
        )
        .expect("authentic surface")
        .expect_err("module resolver preflight"),
        "Task258A label resolver module mismatch"
    );
    replay();
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
        replay();
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
        replay();
    }

    let (_, lower_module, lower_shells, lower_symbols, lower_diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            259_401,
        );
    assert_eq!(lower_diagnostics, 0);
    assert_eq!(lower_module, module);
    let binding_env = &baseline.binding_env;
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
        let lower = run_lower(SetEnumerationProofContextTestOptions::default())
            .expect("B3P replay selector")
            .expect("B3P replay");
        assert_eq!(
            lower.typed_ast.source_set_term(),
            baseline.set_term.as_ref()
        );
        replay();
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
    assert_eq!(
        (
            TASK258B3M2B2B3A_TASK256_FIELD_COUNT,
            TASK258B3M2B2B3A_TASK258_FIELD_COUNT,
            TASK258B3M2B2B3A_WITNESS_FIELD_COUNT
        ),
        (72, 62, 21)
    );
    for field in 0..TASK258B3M2B2B3A_TASK256_FIELD_COUNT {
        match source_statement_b3m2b2b3a_output_with_stage_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            SourceStatementB3M2B2B3AStage::Task256,
            field,
        )
        .expect("authentic atomic extraction")
        {
            Err(error) => assert!(error.starts_with("Task256:"), "{field}: {error}"),
            Ok(changed) => assert_ne!(changed.atomic, baseline.atomic, "field {field}"),
        }
        replay();
    }
    // Task258/witness field causality remains in the Checker-private enumeration matrices.
    for (stage, count) in [
        (
            SourceStatementB3M2B2B3AStage::Task258,
            TASK258B3M2B2B3A_TASK258_FIELD_COUNT,
        ),
        (
            SourceStatementB3M2B2B3AStage::Witness,
            TASK258B3M2B2B3A_WITNESS_FIELD_COUNT,
        ),
    ] {
        for field in 0..count {
            let changed = source_statement_b3m2b2b3a_output_with_stage_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M2B2B3A_TEXT,
                stage,
                field,
            )
            .expect("authentic upper extraction")
            .expect("genuine upper DTO capture before raw refusal");
            match stage {
                SourceStatementB3M2B2B3AStage::Task258 => {
                    assert_ne!(changed.statement, baseline.statement, "{stage:?} {field}")
                }
                SourceStatementB3M2B2B3AStage::Witness => {
                    assert_ne!(changed.witness, baseline.witness, "{stage:?} {field}")
                }
                SourceStatementB3M2B2B3AStage::Task256 => unreachable!(),
            }
            replay();
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
        let error = source_statement_b3m2b2b3a_output_with_mutation(
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
                        witness: mizar_checker::source_statement::SourceStatementWitnessId::new(0),
                        site: input.witness.witnesses[0].site.clone(),
                        source_range: input.witness.witnesses[0].source_range,
                        spelling: "y".to_owned(),
                        recovery: mizar_checker::source_statement::SourceStatementRecovery::Normal,
                    },
                ),
                _ => unreachable!(),
            },
        )
        .expect("selector")
        .expect_err("raw admission precedes upper aggregate validation");
        assert_eq!(
            error, "source statement dependency mismatch",
            "upper mutation {mutation}"
        );
        replay();
    }
    replay();
}

#[test]
fn task258b3m2b2b3a_set_enumeration_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, symbols, baseline) = task258b3m2b2b3a_lower_inputs(259_403);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let replay = || {
        let (_, _, _, pristine) = task258b3m2b2b3a_lower_inputs(259_403);
        assert_eq!(format!("{pristine:?}"), format!("{baseline:?}"));
        assert_eq!(
            (ast.snapshot_text(), module.clone(), symbols.clone()),
            before
        );
    };
    for byte_index in 0..SOURCE_STATEMENT_B3M2B2B3A_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B2B3A_TEXT.as_bytes().to_vec();
        bytes[byte_index] = b'?';
        let changed = String::from_utf8(bytes).expect("ASCII");
        assert!(
            extract_set_enumeration_witness_source_statement(&ast, &changed).is_none(),
            "byte {byte_index} extraction"
        );
        assert!(
            source_statement_b3m2b2b3a_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                &changed,
                |_| panic!("inauthentic source admitted"),
            )
            .is_none(),
            "byte {byte_index} authentication"
        );
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &changed).is_none()
        );
        replay();
    }
    for target in 0..58 {
        for mutation in 0..if target == 57 { 1 } else { 4 } {
            let changed = task258_mutated_surface(&ast, target, mutation);
            assert!(
                extract_set_enumeration_witness_source_statement(
                    &changed,
                    SOURCE_STATEMENT_B3M2B2B3A_TEXT
                )
                .is_none(),
                "node {target} mutation {mutation}"
            );
            assert!(
                source_statement_b3m2b2b3a_output_with_resolver_mutation(
                    &changed,
                    module.clone(),
                    &symbols,
                    SOURCE_STATEMENT_B3M2B2B3A_TEXT,
                    |symbols| symbols,
                )
                .is_none(),
                "node {target} mutation {mutation} authentication"
            );
            replay();
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
            "{label} extraction"
        );
        assert!(
            source_statement_b3m2b2b3a_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                &changed,
                |_| panic!("inauthentic final LF admitted"),
            )
            .is_none(),
            "{label} authentication"
        );
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &changed).is_none()
        );
        replay();
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
    replay();

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
            extract_set_enumeration_witness_source_statement(&changed_ast, &changed).is_none(),
            "near miss {index} extraction"
        );
        assert!(
            source_statement_b3m2b2b3a_output_with_mutation(
                &changed_ast,
                changed_module.clone(),
                &changed_symbols,
                &changed,
                |_| panic!("inauthentic near miss admitted"),
            )
            .is_none(),
            "near miss {index} authentication"
        );
        assert!(
            source_statement_output_with_source(
                &changed_ast,
                changed_module.clone(),
                &changed_symbols,
                &changed,
            )
            .is_none(),
            "near miss {index}"
        );
        assert!(
            source_statement_transport_detail_keys(
                &changed_ast,
                changed_module,
                &changed_symbols,
                &changed
            )
            .is_none()
        );
        replay();
    }
}

#[test]
fn task258b3m2b2b3a_family_and_active_route_isolation_is_atomic() {
    let (ast, module, symbols, b3a) = task258b3m2b2b3a_lower_inputs(259_420);
    let (application_ast, application_module, application_symbols) =
        task258b3m2b2b1a_fixture(259_420);
    let application =
        task258b3m2b2b1a_lower_inputs(&application_ast, &application_module, &application_symbols)
            .application
            .expect("genuine bare application");
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
    let set = b3a.set_term.as_ref().expect("set handoff").clone();
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
        assert_eq!(
            before.expect_err("retired raw or lower dependency refusal"),
            if tuple == 4 {
                "source statement dependency mismatch"
            } else {
                "Task258B3M2B2B3A lower dependency mismatch"
            },
            "pre-auth tuple {tuple:03b}"
        );
        let (_, _, _, pristine) = task258b3m2b2b3a_lower_inputs(259_420);
        assert_eq!(format!("{pristine:?}"), format!("{b3a:?}"));

        let application_member = application.clone();
        let structure_member = structure.clone();
        let set_member = set.clone();
        let mut changed = None;
        let after = source_statement_b3m2b2b3a_output_with_post_auth_family_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M2B2B3A_TEXT,
            |input| {
                input.application = (tuple & 1 != 0).then_some(application_member);
                input.structure = (tuple & 2 != 0).then_some(structure_member);
                input.set_term = (tuple & 4 != 0).then_some(set_member);
                changed = Some(input.clone());
            },
        )
        .expect("post-auth tuple selector");
        assert_eq!(
            after.expect_err("retired raw admission"),
            "source statement dependency mismatch",
            "post-auth tuple {tuple:03b}"
        );
        let changed = changed.expect("genuine post-auth capture");
        assert_eq!(changed.application.is_some(), tuple & 1 != 0);
        assert_eq!(changed.structure.is_some(), tuple & 2 != 0);
        assert_eq!(changed.set_term.is_some(), tuple & 4 != 0);
        if tuple == 4 {
            assert_eq!(format!("{changed:?}"), format!("{b3a:?}"));
        } else {
            assert_ne!(format!("{changed:?}"), format!("{b3a:?}"));
        }
        let (_, _, _, pristine) = task258b3m2b2b3a_lower_inputs(259_420);
        assert_eq!(format!("{pristine:?}"), format!("{b3a:?}"));
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
        if source == SOURCE_STATEMENT_B3M2B2B2C_TEXT {
            let lower = task258b3m2b2b2c_lower_inputs(&case_ast, &case_module, &case_symbols);
            assert_eq!(
                format!("{lower:?}"),
                format!(
                    "{:?}",
                    task258b3m2b2b2c_lower_inputs(&case_ast, &case_module, &case_symbols)
                )
            );
            assert!(lower.application.is_none() && lower.set_term.is_none());
            let structure = lower
                .structure
                .expect("authenticated update lower structure");
            assert_eq!(
                (structure.terms().len(), structure.field_updates().len()),
                (2, 1)
            );
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B1A_TEXT {
            task258b3m2b2b1a_lower_inputs(&case_ast, &case_module, &case_symbols);
            continue;
        }
        if source == SOURCE_STATEMENT_B3M2B2B3A_TEXT {
            let before = (
                case_ast.snapshot_text(),
                case_module.clone(),
                case_symbols.clone(),
            );
            assert!(extract_set_enumeration_witness_source_statement(&case_ast, source).is_some());
            assert!(
                source_statement_output_with_source(
                    &case_ast,
                    case_module.clone(),
                    &case_symbols,
                    source
                )
                .is_none()
            );
            assert!(
                source_statement_transport_detail_keys(
                    &case_ast,
                    case_module.clone(),
                    &case_symbols,
                    source
                )
                .is_none()
            );
            assert_eq!(
                source_statement_b3m2b2b3a_output_with_mutation(
                    &case_ast,
                    case_module.clone(),
                    &case_symbols,
                    source,
                    |_| {},
                )
                .expect("authentic numeric enumeration extraction")
                .expect_err("retired raw profile"),
                "source statement dependency mismatch"
            );
            assert_eq!(
                (case_ast.snapshot_text(), case_module, case_symbols),
                before
            );
            continue;
        }
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
fn task258b3m2b2b3a_raw_refusal_preserves_lower_clone_debug_and_enumeration_requests() {
    use mizar_checker::source_set_term::{
        SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget,
    };
    let (ast, module, symbols, lower) = task258b3m2b2b3a_lower_inputs(259_440);
    let before = (ast.snapshot_text(), module.clone(), symbols.clone());
    let debug = format!("{lower:?}");
    assert_eq!(format!("{:?}", lower.clone()), debug);
    let set = lower
        .set_term
        .as_ref()
        .expect("numeric enumeration lower set");
    assert_eq!(set.primary_term_fingerprint(), lower.primary.debug_text());
    assert!(set.application_fingerprint().is_none() && set.structure_fingerprint().is_none());
    assert_eq!(
        set.edges()
            .iter()
            .map(|(_, edge)| (edge.role(), edge.target()))
            .collect::<Vec<_>>(),
        [
            (
                SourceSetEdgeRole::EnumerationElement,
                SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(2))
            ),
            (
                SourceSetEdgeRole::EnumerationElement,
                SourceSetTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(3))
            ),
        ]
    );
    assert_eq!(
        set.requests()
            .iter()
            .map(|(_, request)| request.kind())
            .collect::<Vec<_>>(),
        [SourceSetRequestKind::ResultType]
    );
    let error = source_statement_b3m2b2b3a_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        |inputs| inputs.witness.witnesses.clear(),
    )
    .expect("authentic extraction")
    .expect_err("raw admission precedes witness validation");
    assert_eq!(error, "source statement dependency mismatch");
    let (_, _, _, replay) = task258b3m2b2b3a_lower_inputs(259_440);
    assert_eq!(format!("{replay:?}"), debug);
    assert_eq!((ast.snapshot_text(), module, symbols), before);
}
