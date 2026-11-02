use super::*;

#[test]
fn task258b3m2b2b3b_exact_empty_set_enumeration_witness_profile_and_legacy_compatibility_are_stable()
 {
    const SOURCE: &str = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementEmptySetEnumerationWitnessSmoke: x = x proof\n",
        "  take {};\n",
        "  thus x = x;\n",
        "end;\n",
    );
    let fixture = B3M2B2B3BFixture::new(236);
    assert_eq!(SOURCE.len(), 118);
    assert_eq!(
        SOURCE,
        "reserve x for set;\ntheorem FormulaStatementEmptySetEnumerationWitnessSmoke: x = x proof\n  take {};\n  thus x = x;\nend;\n"
    );
    assert_eq!(fixture.arena.len(), 50);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(49)));
    for (index, (start, end)) in TASK258B3M2B2B3B_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b3b_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b3b_node_children(index)
        );
        assert!(row.resolved_node.is_none(), "node {index} resolved_node");
        assert_eq!(row.typing, TypingState::Unknown, "node {index} typing");
        assert!(row.links.context.is_none(), "node {index} context");
        assert!(row.links.type_entry.is_none(), "node {index} type");
        assert!(row.links.facts.is_empty(), "node {index} facts");
        assert!(row.links.coercions.is_empty(), "node {index} coercions");
        assert!(
            row.links.initial_obligations.is_empty(),
            "node {index} obligations"
        );
        assert!(row.links.diagnostics.is_empty(), "node {index} diagnostics");
    }
    assert_eq!(
        (
            fixture.bindings.contexts().len(),
            fixture.bindings.bindings().len(),
            fixture.bindings.diagnostics().len(),
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len(),
        ),
        (2, 1, 0, 4, 4, 0)
    );
    assert_eq!(
        (
            fixture.set_terms.terms().len(),
            fixture.set_terms.wrappers().len(),
            fixture.set_terms.generators().len(),
            fixture.set_terms.type_sites().len(),
            fixture.set_terms.conditions().len(),
            fixture.set_terms.edges().len(),
            fixture.set_terms.requests().len(),
        ),
        (1, 0, 0, 0, 0, 0, 1)
    );
    assert_eq!(
        (
            fixture.atomic.formulas().len(),
            fixture.atomic.wrappers().len(),
            fixture.atomic.predicate_segments().len(),
            fixture.atomic.predicate_heads().len(),
            fixture.atomic.candidates().len(),
            fixture.atomic.type_sites().len(),
            fixture.atomic.attributes().len(),
            fixture.atomic.edges().len(),
            fixture.atomic.requests().len(),
        ),
        (2, 0, 0, 0, 0, 0, 0, 4, 4)
    );
    let statement = fixture.statement();
    assert!(statement.is_task_258b3m2b2b3b_profile());
    assert!(!statement.is_task_258b3m2b2b3a_profile());
    let witnesses = fixture.witnesses(&statement).expect("empty-set witness");
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(witnesses.structure_fingerprint(), None);
    assert_eq!(
        witnesses.set_term_fingerprint(),
        Some(fixture.set_terms.debug_text().as_str())
    );
    assert_eq!(
        witnesses.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nset-term-fingerprint: Some({:?})\nwitness#0 owner=0 binding_context=1 term=set-term#0 take_range=90..98 take_site=36 range=95..97 site=35 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"{{ }}\"\n",
            statement.debug_text(),
            fixture.primary.debug_text(),
            fixture.set_terms.debug_text(),
        )
    );
    assert_eq!(
        fixture
            .primary
            .terms()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .collect::<Vec<_>>(),
        [27, 29, 37, 39]
    );
    assert_eq!(
        fixture
            .set_terms
            .terms()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .collect::<Vec<_>>(),
        [33]
    );
    assert_eq!(
        fixture
            .atomic
            .formulas()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .collect::<Vec<_>>(),
        [31, 41]
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .collect::<Vec<_>>(),
        [46, 44]
    );
    assert_eq!(
        witnesses
            .witnesses()
            .iter()
            .flat_map(|(_, row)| [row.site().node().index(), row.take_site().node().index()])
            .collect::<Vec<_>>(),
        [35, 36]
    );

    let binding_contexts = fixture.bindings.contexts();
    let module_context = binding_contexts.get(BindingContextId::new(0)).unwrap();
    assert_eq!(
        (
            module_context.owner.clone(),
            module_context.parent,
            module_context.layer,
            module_context.lexical_scope.clone(),
            module_context.bindings.clone(),
            module_context.visible_bindings.clone(),
            module_context.recovery,
        ),
        (
            BindingContextOwner::Module,
            None,
            BindingContextLayer::Module,
            None,
            vec![BindingId::new(0)],
            vec![BindingId::new(0)],
            BindingContextRecovery::Normal,
        )
    );
    let proof_context = binding_contexts.get(BindingContextId::new(1)).unwrap();
    assert_eq!(
        (
            proof_context.owner.clone(),
            proof_context.parent,
            proof_context.layer,
            proof_context.lexical_scope.clone(),
            proof_context.bindings.clone(),
            proof_context.visible_bindings.clone(),
            proof_context.recovery,
        ),
        (
            BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 82, 116),
            },
            Some(BindingContextId::new(0)),
            BindingContextLayer::Proof,
            Some(LocalTermScope::new(vec![0])),
            Vec::new(),
            vec![BindingId::new(0)],
            BindingContextRecovery::Normal,
        )
    );
    let reserved_x = fixture.bindings.bindings().get(BindingId::new(0)).unwrap();
    assert_eq!(
        (
            reserved_x.spelling.as_str(),
            reserved_x.kind,
            reserved_x.identity.clone(),
            reserved_x.owner_context,
            reserved_x.declaration_range,
            reserved_x.visible_after_ordinal,
            reserved_x.type_site.clone(),
            reserved_x.status,
            reserved_x.captured.identities().to_vec(),
            reserved_x.diagnostics.clone(),
            reserved_x.recovery,
        ),
        (
            "x",
            BindingKind::ReservedVariable,
            BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(fixture.source, 8, 9),
            },
            BindingContextId::new(0),
            range(fixture.source, 8, 9),
            0,
            BindingTypeSite::Source(range(fixture.source, 14, 17)),
            BindingStatus::Reserved,
            Vec::new(),
            Vec::new(),
            BindingRecoveryState::Normal,
        )
    );
    assert_eq!(
        fixture
            .primary
            .terms()
            .iter()
            .map(|(_, row)| (
                row.site().clone(),
                row.source_range(),
                row.source_ordinal(),
                row.context(),
                row.spelling().to_owned(),
                row.kind(),
                row.role(),
                row.parent(),
                row.recovery(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                node(27),
                range(fixture.source, 76, 77),
                0,
                BindingContextId::new(0),
                "x".to_owned(),
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
                None,
                SourcePrimaryTermRecovery::Normal
            ),
            (
                node(29),
                range(fixture.source, 80, 81),
                1,
                BindingContextId::new(0),
                "x".to_owned(),
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
                None,
                SourcePrimaryTermRecovery::Normal
            ),
            (
                node(37),
                range(fixture.source, 106, 107),
                2,
                BindingContextId::new(1),
                "x".to_owned(),
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
                None,
                SourcePrimaryTermRecovery::Normal
            ),
            (
                node(39),
                range(fixture.source, 110, 111),
                3,
                BindingContextId::new(1),
                "x".to_owned(),
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
                None,
                SourcePrimaryTermRecovery::Normal
            ),
        ]
    );
    assert_eq!(
        fixture
            .primary
            .references()
            .iter()
            .map(|(_, row)| (
                row.term(),
                row.binding(),
                row.role(),
                row.lexical_scope().cloned(),
                row.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        (0..4)
            .map(|index| (
                SourcePrimaryTermId::new(index),
                BindingId::new(0),
                SourcePrimaryTermReferenceRole::Variable,
                (index >= 2).then(|| LocalTermScope::new(vec![0])),
                1,
            ))
            .collect::<Vec<_>>()
    );
    let empty_enumeration = fixture
        .set_terms
        .terms()
        .get(SourceSetTermId::new(0))
        .unwrap();
    assert_eq!(
        (
            empty_enumeration.site().clone(),
            empty_enumeration.source_range(),
            empty_enumeration.source_ordinal(),
            empty_enumeration.context(),
            empty_enumeration.recovery(),
            empty_enumeration.spelling(),
            empty_enumeration.kind(),
        ),
        (
            node(33),
            range(fixture.source, 95, 97),
            0,
            BindingContextId::new(1),
            SourceSetTermRecovery::Normal,
            "{ }",
            SourceSetTermKind::Enumeration
        )
    );
    assert_eq!(
        fixture
            .set_terms
            .requests()
            .get(crate::source_set_term::SourceSetRequestId::new(0))
            .map(|row| (
                row.term(),
                row.ordinal(),
                row.kind(),
                row.generator(),
                row.type_site()
            )),
        Some((
            SourceSetTermId::new(0),
            0,
            SourceSetRequestKind::ResultType,
            None,
            None
        ))
    );
    assert_eq!(
        fixture.set_terms.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(fixture.set_terms.application_fingerprint(), None);
    assert_eq!(fixture.set_terms.structure_fingerprint(), None);
    assert_eq!(witnesses.statement_fingerprint(), statement.debug_text());
    assert_eq!(
        witnesses.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(
        witnesses.set_term_fingerprint(),
        Some(fixture.set_terms.debug_text().as_str())
    );
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(witnesses.structure_fingerprint(), None);
    assert_eq!(
        fixture
            .atomic
            .formulas()
            .iter()
            .map(|(_, row)| (
                row.site().clone(),
                row.source_range(),
                row.source_ordinal(),
                row.context(),
                row.recovery(),
                row.spelling().to_owned(),
                row.kind(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                node(31),
                range(fixture.source, 76, 81),
                0,
                BindingContextId::new(0),
                SourceAtomicFormulaRecovery::Normal,
                "x = x".to_owned(),
                SourceAtomicFormulaKind::Equality
            ),
            (
                node(41),
                range(fixture.source, 106, 111),
                1,
                BindingContextId::new(1),
                SourceAtomicFormulaRecovery::Normal,
                "x = x".to_owned(),
                SourceAtomicFormulaKind::Equality
            ),
        ]
    );
    assert_eq!(
        fixture
            .atomic
            .edges()
            .iter()
            .map(|(_, row)| (row.formula(), row.ordinal(), row.role(), row.target()))
            .collect::<Vec<_>>(),
        (0..4)
            .map(|index| (
                SourceAtomicFormulaId::new(index / 2),
                index % 2,
                if index % 2 == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                },
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(index)),
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        fixture
            .atomic
            .requests()
            .iter()
            .map(|(_, row)| (
                row.formula(),
                row.ordinal(),
                row.kind(),
                row.edge(),
                row.candidate(),
                row.type_site(),
                row.attribute(),
            ))
            .collect::<Vec<_>>(),
        (0..4)
            .map(|index| (
                SourceAtomicFormulaId::new(index / 2),
                index % 2,
                SourceAtomicRequestKind::OperandExpectedType,
                Some(SourceAtomicEdgeId::new(index)),
                None,
                None,
                None,
            ))
            .collect::<Vec<_>>()
    );
    let theorem_owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .unwrap();
    assert_eq!(
        (
            theorem_owner.symbol().clone(),
            theorem_owner.contribution(),
            theorem_owner.site().clone(),
            theorem_owner.source_range(),
            theorem_owner.spelling(),
            theorem_owner.role(),
            theorem_owner.status(),
            theorem_owner.recovery(),
        ),
        (
            fixture.symbol.clone(),
            fixture.contribution,
            node(46),
            range(fixture.source, 19, 117),
            "FormulaStatementEmptySetEnumerationWitnessSmoke",
            SourceTheoremRole::Theorem,
            SourceTheoremStatus::Unmodified,
            SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        (
            statement.checked_owner().source_range(),
            statement.checked_owner().origin().source_id(),
            statement.checked_owner().origin().module_id(),
            statement.checked_owner().origin().anchor(),
            statement.checked_owner().origin().structural_path(),
            statement.checked_owner().origin().import_edge(),
            statement.checked_owner().origin().is_recovered(),
            statement.checked_owner().visibility(),
            statement.checked_owner().export_status(),
        ),
        (
            range(fixture.source, 19, 117),
            fixture.source,
            &fixture.module,
            &SourceAnchor::Range(range(fixture.source, 19, 117)),
            &[2, 1][..],
            None,
            false,
            Visibility::Public,
            ExportStatus::Exported,
        )
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| (
                row.owner(),
                row.context(),
                row.formula(),
                row.site().clone(),
                row.source_range(),
                row.source_ordinal(),
                row.spelling().to_owned(),
                row.kind(),
                row.recovery(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (SourceTheoremOwnerId::new(0), SourceStatementContextId::new(0), SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0)), node(46), range(fixture.source, 19, 117), 0, "theorem FormulaStatementEmptySetEnumerationWitnessSmoke : x = x proof take { } ; thus x = x ; end ;".to_owned(), SourceStatementKind::TheoremProposition, SourceStatementRecovery::Normal),
            (SourceTheoremOwnerId::new(0), SourceStatementContextId::new(1), SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(1)), node(44), range(fixture.source, 101, 112), 2, "thus x = x ;".to_owned(), SourceStatementKind::Conclusion, SourceStatementRecovery::Normal),
        ]
    );
    assert_eq!(
        statement
            .contexts()
            .iter()
            .map(|(_, row)| (
                row.statement(),
                row.binding_context(),
                row.source_range(),
                row.visible_bindings().to_vec()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                SourceStatementId::new(0),
                BindingContextId::new(0),
                range(fixture.source, 19, 117),
                vec![BindingId::new(0)]
            ),
            (
                SourceStatementId::new(1),
                BindingContextId::new(1),
                range(fixture.source, 101, 112),
                vec![BindingId::new(0)]
            ),
        ]
    );
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(_, row)| (
                row.statement(),
                row.context(),
                row.ordinal(),
                row.kind(),
                row.binding(),
                row.uses().to_vec()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                SourceStatementId::new(0),
                SourceStatementContextId::new(0),
                0,
                SourceStatementInputFactKind::ReservedTypeGuard,
                BindingId::new(0),
                vec![
                    SourcePrimaryTermReferenceId::new(0),
                    SourcePrimaryTermReferenceId::new(1)
                ]
            ),
            (
                SourceStatementId::new(1),
                SourceStatementContextId::new(1),
                0,
                SourceStatementInputFactKind::ReservedTypeGuard,
                BindingId::new(0),
                vec![
                    SourcePrimaryTermReferenceId::new(2),
                    SourcePrimaryTermReferenceId::new(3)
                ]
            ),
        ]
    );
    assert_eq!(
        statement
            .candidate_facts()
            .iter()
            .map(|(_, row)| (
                row.statement(),
                row.context(),
                row.ordinal(),
                row.kind(),
                row.formula()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                SourceStatementId::new(0),
                SourceStatementContextId::new(0),
                0,
                SourceStatementCandidateFactKind::UnverifiedProposition,
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            ),
            (
                SourceStatementId::new(1),
                SourceStatementContextId::new(1),
                0,
                SourceStatementCandidateFactKind::UnverifiedProposition,
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(1))
            ),
        ]
    );
    assert_eq!(fixture.symbols.labels().len(), 1);
    let owner_label = fixture.symbols.labels().iter().next().unwrap();
    assert_eq!(owner_label.origin(), statement.checked_owner().origin());
    assert_eq!(fixture.contribution.index(), 0);
    assert_eq!(
        (
            owner_label.kind(),
            owner_label.namespace().as_str(),
            owner_label.primary_spelling(),
            owner_label.origin().source_id(),
            owner_label.origin().module_id(),
            owner_label.origin().anchor(),
            owner_label.origin().structural_path(),
            owner_label.visibility(),
            owner_label.export_status(),
            owner_label.recovery(),
            owner_label.origin_path().as_str(),
            owner_label.contribution(),
        ),
        (
            LabelKind::Theorem,
            "statement.fixture",
            "FormulaStatementEmptySetEnumerationWitnessSmoke",
            fixture.source,
            &fixture.module,
            &SourceAnchor::Range(range(fixture.source, 19, 117)),
            &[2, 1][..],
            Visibility::Public,
            ExportStatus::Exported,
            RecoveryState::Normal,
            "statement.fixture.theorem.b3m2b2b3b",
            fixture.contribution,
        )
    );
    let owner_contribution = fixture
        .symbols
        .contributions()
        .get(fixture.contribution)
        .unwrap();
    assert_eq!(
        owner_contribution.anchor(),
        &SourceAnchor::Range(range(fixture.source, 0, 18))
    );
    assert_eq!(
        owner_contribution.effects().labels(),
        &[owner_label.origin_path().clone()]
    );

    let legacy = B3M2B2B3AFixture::new(236);
    let legacy_statement = legacy.statement();
    legacy
        .witnesses(&legacy_statement)
        .expect("nonempty enumeration remains admitted");
}

#[test]
fn task258b3m2b2b3b_dependencies_empty_enumeration_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M2B2B3BFixture::new(237);
    let statement = fixture.statement();
    let witnesses = fixture.witnesses(&statement).expect("witnesses");
    let validate = |arena: &TypedArena| {
        witnesses.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            arena,
        )
    };
    let pristine_fixture = Box::new(fixture.clone());
    let pristine_statement = statement.clone();
    let pristine_witnesses = witnesses.clone();
    let pristine_typed = Box::new(
        fixture
            .empty_typed()
            .with_source_set_term_statement_witnesses(
                fixture.set_terms.clone(),
                statement.clone(),
                witnesses.clone(),
            )
            .expect("pristine typed control"),
    );
    let pristine_resolved =
        Box::new(assemble_empty_resolved(&pristine_typed).expect("pristine final control"));
    let replay_pristine = || {
        assert_eq!(
            (
                &fixture.symbols,
                &fixture.arena,
                &fixture.bindings,
                &fixture.primary,
                &fixture.set_terms,
                &fixture.atomic,
            ),
            (
                &pristine_fixture.symbols,
                &pristine_fixture.arena,
                &pristine_fixture.bindings,
                &pristine_fixture.primary,
                &pristine_fixture.set_terms,
                &pristine_fixture.atomic,
            ),
            "pristine dependency purity"
        );
        let replay = fixture.statement();
        assert_eq!(replay, pristine_statement);
        let replay_witnesses = fixture.witnesses(&replay).expect("pristine witness replay");
        assert_eq!(replay_witnesses, pristine_witnesses);
        validate(&fixture.arena).expect("pristine witness installation replay");
        let replay_typed = fixture
            .empty_typed()
            .with_source_set_term_statement_witnesses(
                fixture.set_terms.clone(),
                replay,
                replay_witnesses,
            )
            .expect("pristine typed replay");
        assert_eq!(&replay_typed, pristine_typed.as_ref());
        assert_eq!(
            &assemble_empty_resolved(&replay_typed).expect("pristine final replay"),
            pristine_resolved.as_ref()
        );
    };
    validate(&fixture.arena).expect("clean dependency tuple");
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            source_id(999_237),
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            fixture.source,
            &ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign")),
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(
        witnesses.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            None,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut stale = witnesses.clone();
    stale.statement_fingerprint.push('!');
    assert_eq!(
        stale.validate_installation_with_set_term(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            Some(&fixture.set_terms),
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut resolved_builder = ResolvedArenaBuilder::new();
    let foreign_resolved_node = resolved_builder
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::Root,
            Vec::new(),
            SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 0, 0)),
                vec![999],
            ),
        ))
        .expect("foreign resolved-node id");
    for index in 0..fixture.arena.len() {
        let mutations = [
            (
                "kind",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.kind = "source.task258b3m2b2b3b.mutated".into();
                    }
                }),
            ),
            (
                "resolved_node",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.resolved_node = Some(foreign_resolved_node);
                    }
                }),
            ),
            (
                "anchor",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
                    }
                }),
            ),
            (
                "children",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        if row.children.is_empty() {
                            row.children.push(TypedNodeId::new(usize::from(index == 0)));
                        } else {
                            row.children.pop();
                        }
                    }
                }),
            ),
            (
                "recovery",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.recovery = NodeRecoveryState::Degraded;
                    }
                }),
            ),
            (
                "typing",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.typing = TypingState::Successful;
                    }
                }),
            ),
            (
                "links.context",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.links.context = Some(LocalTypeContextId::new(0));
                    }
                }),
            ),
            (
                "links.type_entry",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.links.type_entry = Some(TypeEntryId::new(0));
                    }
                }),
            ),
            (
                "links.facts",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.links.facts.push(TypeFactId::new(0));
                    }
                }),
            ),
            (
                "links.coercions",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.links.coercions.push(CoercionId::new(0));
                    }
                }),
            ),
            (
                "links.initial_obligations",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.links
                            .initial_obligations
                            .push(InitialObligationId::new(0));
                    }
                }),
            ),
            (
                "links.diagnostics",
                mutate_arena(&fixture.arena, |id, row| {
                    if id.index() == index {
                        row.links.diagnostics.push(TypeDiagnosticId::new(0));
                    }
                }),
            ),
        ];
        for (field, mutated) in mutations {
            assert_eq!(
                validate(&mutated),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} {field}"
            );
        }
    }
    for root in [None, Some(TypedNodeId::new(48))] {
        let wrong_root = TypedArena::try_new(
            root,
            fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
        )
        .expect("wrong-root arena remains structurally valid");
        assert_eq!(
            validate(&wrong_root),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "root {root:?}"
        );
    }

    let task48_expected = [
        Some(
            crate::binding_env::BindingEnvError::InvalidContextSourceRange {
                context: BindingContextId::new(1),
            },
        ),
        None,
        Some(crate::binding_env::BindingEnvError::MissingModuleContext),
        None,
        Some(crate::binding_env::BindingEnvError::InvalidModuleContext {
            context: BindingContextId::new(0),
        }),
        Some(crate::binding_env::BindingEnvError::InvalidModuleContext {
            context: BindingContextId::new(0),
        }),
        Some(crate::binding_env::BindingEnvError::InvalidModuleContext {
            context: BindingContextId::new(0),
        }),
        None,
        Some(
            crate::binding_env::BindingEnvError::InvalidVisibleBindingOwner {
                context: BindingContextId::new(0),
                binding: BindingId::new(0),
                owner: BindingContextId::new(0),
            },
        ),
        None,
        None,
        None,
        Some(crate::binding_env::BindingEnvError::MultipleRootContexts {
            context: BindingContextId::new(1),
        }),
        None,
        None,
        Some(crate::binding_env::BindingEnvError::InvalidContextBinding {
            context: BindingContextId::new(1),
            binding: BindingId::new(0),
        }),
        None,
        None,
        Some(crate::binding_env::BindingEnvError::InvalidContextBinding {
            context: BindingContextId::new(0),
            binding: BindingId::new(0),
        }),
        Some(
            crate::binding_env::BindingEnvError::InconsistentReservedVariableIdentity {
                binding: BindingId::new(1),
            },
        ),
        Some(
            crate::binding_env::BindingEnvError::InconsistentReservedVariableIdentity {
                binding: BindingId::new(0),
            },
        ),
        Some(
            crate::binding_env::BindingEnvError::InvalidBindingKindIdentity {
                binding: BindingId::new(0),
            },
        ),
        Some(
            crate::binding_env::BindingEnvError::InvalidBindingKindIdentity {
                binding: BindingId::new(0),
            },
        ),
        Some(crate::binding_env::BindingEnvError::InvalidContextBinding {
            context: BindingContextId::new(0),
            binding: BindingId::new(0),
        }),
        Some(
            crate::binding_env::BindingEnvError::InconsistentReservedVariableIdentity {
                binding: BindingId::new(0),
            },
        ),
        None,
        None,
        None,
        None,
        Some(
            crate::binding_env::BindingEnvError::InvalidBindingDiagnostic {
                binding: BindingId::new(0),
                diagnostic: BindingDiagnosticId::new(0),
            },
        ),
        None,
        None,
    ];
    assert_eq!(task48_expected.len(), 32);
    for (field, expected) in task48_expected.iter().enumerate() {
        let actual = if field == 19 {
            let input = &fixture.bindings;
            let mut binding_drafts = input
                .bindings()
                .iter()
                .map(|(_, row)| BindingDraft {
                    spelling: row.spelling.clone(),
                    kind: row.kind,
                    identity: row.identity.clone(),
                    owner_context: row.owner_context,
                    declaration_range: row.declaration_range,
                    visible_after_ordinal: row.visible_after_ordinal,
                    type_site: row.type_site.clone(),
                    status: row.status,
                    captured: row.captured.clone(),
                    diagnostics: row.diagnostics.clone(),
                    recovery: row.recovery,
                })
                .collect::<Vec<_>>();
            let mut duplicate = binding_drafts[0].clone();
            duplicate.spelling.push('!');
            binding_drafts.push(duplicate);
            let mut binding_table = BindingTable::new();
            for row in binding_drafts {
                binding_table.insert(row);
            }
            let original_parts = BindingEnvParts {
                source_id: input.source_id(),
                module_id: input.module_id().clone(),
                contexts: input.contexts().clone(),
                bindings: input.bindings().clone(),
                diagnostics: input.diagnostics().clone(),
            };
            let mutated_parts = BindingEnvParts {
                source_id: input.source_id(),
                module_id: input.module_id().clone(),
                contexts: input.contexts().clone(),
                bindings: binding_table,
                diagnostics: input.diagnostics().clone(),
            };
            assert_ne!(
                mutated_parts, original_parts,
                "Task48 field 19 spelling delta"
            );
            let before = mutated_parts.clone();
            let result = BindingEnv::try_new(mutated_parts.clone());
            assert_eq!(mutated_parts, before, "Task48 field 19 input purity");
            result
        } else {
            task258b3m2b2b3d_mutated_binding_env(&fixture.bindings, field)
        };
        match (expected, actual) {
            (Some(expected), Err(actual)) => {
                assert_eq!(&actual, expected, "Task48 field {field}")
            }
            (Some(expected), Ok(_)) => {
                panic!("Task48 field {field} admitted; expected {expected:?}")
            }
            (None, Ok(bindings)) => {
                assert_ne!(
                    bindings, pristine_fixture.bindings,
                    "Task48 field {field} handoff delta"
                );
                let bindings_before = bindings.clone();
                let input = fixture.statement_input();
                let input_before = input.clone();
                assert_eq!(
                    SourceStatementProducer::build(
                        input.clone(),
                        &fixture.symbols,
                        &bindings,
                        &fixture.primary,
                        &fixture.atomic,
                        &fixture.arena,
                    ),
                    Err(SourceStatementError::DependencyMismatch),
                    "Task48 field {field} admitted binding env reaches statement dependency check"
                );
                assert_eq!(bindings, bindings_before, "Task48 handoff purity");
                assert_eq!(input, input_before, "Task48 statement input purity");
            }
            (None, Err(actual)) => {
                panic!("Task48 field {field} refused as {actual:?}; expected admitted env")
            }
        }
        replay_pristine();
    }

    for field in 0..55 {
        let pristine_input = B3M2B2B3BFixture::primary_input(fixture.source, &fixture.module);
        let mut input = pristine_input.clone();
        match field {
            0 => input.source_id = source_id(999_252),
            1 => {
                input.module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
            }
            2 => input.terms.clear(),
            3 => input.terms.push(input.terms[0].clone()),
            4..40 => {
                let offset = field - 4;
                let term = offset / 9;
                let other = (term + 1) % 4;
                match offset % 9 {
                    0 => input.terms[term].site = input.terms[other].site.clone(),
                    1 => input.terms[term].source_range.start += 1,
                    2 => input.terms[term].source_ordinal += 5,
                    3 => {
                        input.terms[term].context =
                            BindingContextId::new(1 - input.terms[term].context.index())
                    }
                    4 => input.terms[term].recovery = SourcePrimaryTermRecovery::Degraded,
                    5 => input.terms[term].spelling.push('!'),
                    6 => input.terms[term].kind = SourcePrimaryTermKind::Numeral,
                    7 => input.terms[term].role = SourcePrimaryTermRole::CurrentDefinitionResult,
                    8 => input.terms[term].parent = Some(SourcePrimaryTermId::new(other)),
                    _ => unreachable!(),
                }
            }
            40 => input.references.clear(),
            41 => input.references.push(input.references[0].clone()),
            42..54 => {
                let offset = field - 42;
                let reference = offset / 3;
                match offset % 3 {
                    0 => {
                        input.references[reference].term =
                            SourcePrimaryTermId::new((reference + 1) % 4)
                    }
                    1 => input.references[reference].binding = BindingId::new(99),
                    2 => {
                        input.references[reference].role =
                            SourcePrimaryTermReferenceRole::LocalConstant
                    }
                    _ => unreachable!(),
                }
            }
            54 => input
                .numeric_type_requests
                .push(SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(0),
                    owner: input.terms[0].site.clone(),
                    source_range: input.terms[0].source_range,
                    spelling: input.terms[0].spelling.clone(),
                    request_ordinal: 0,
                }),
            _ => unreachable!(),
        }
        assert_ne!(input, pristine_input, "Task252 field {field} input delta");
        let expected = match field {
            0 | 1 => Some(crate::source_term::SourcePrimaryTermError::InvalidTransaction),
            2 => Some(
                crate::source_term::SourcePrimaryTermError::InvalidReference {
                    reference: SourcePrimaryTermReferenceId::new(0),
                },
            ),
            3 => Some(crate::source_term::SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(4),
            }),
            4..40 => {
                let offset = field - 4;
                (offset % 9 != 3).then_some(
                    crate::source_term::SourcePrimaryTermError::InvalidTerm {
                        term: SourcePrimaryTermId::new(offset / 9),
                    },
                )
            }
            40 => Some(crate::source_term::SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(0),
            }),
            41 => Some(
                crate::source_term::SourcePrimaryTermError::InvalidReference {
                    reference: SourcePrimaryTermReferenceId::new(4),
                },
            ),
            42..54 => {
                let offset = field - 42;
                let reference = offset / 3;
                Some(
                    crate::source_term::SourcePrimaryTermError::InvalidReference {
                        reference: SourcePrimaryTermReferenceId::new(if offset % 3 == 0 {
                            (reference + 1).min(3)
                        } else {
                            reference
                        }),
                    },
                )
            }
            54 => Some(
                crate::source_term::SourcePrimaryTermError::InvalidNumericTypeRequest {
                    request: crate::source_term::SourceNumericTypeRequestId::new(0),
                },
            ),
            _ => unreachable!(),
        };
        let attempted_input = input.clone();
        match (
            expected,
            SourcePrimaryTermProducer::build(input.clone(), &fixture.bindings, &fixture.arena),
        ) {
            (Some(expected), Err(actual)) => {
                assert_eq!(actual, expected, "Task252 field {field}")
            }
            (Some(expected), Ok(_)) => {
                panic!("Task252 field {field} admitted; expected {expected:?}")
            }
            (None, Ok(primary)) => {
                assert_ne!(
                    primary, fixture.primary,
                    "Task252 field {field} handoff delta"
                );
                assert_eq!(primary.source_id(), fixture.source);
                assert_eq!(primary.module_id(), &fixture.module);
                assert_eq!(
                    witnesses.validate_installation_with_set_term(
                        fixture.source,
                        &fixture.module,
                        &statement,
                        &primary,
                        Some(&fixture.set_terms),
                        &fixture.arena,
                    ),
                    Err(SourceStatementWitnessError::DependencyMismatch),
                    "Task252 field {field} admitted primary reaches witness dependency check"
                );
            }
            (None, Err(actual)) => {
                panic!("Task252 field {field} refused as {actual:?}; expected admitted primary")
            }
        }
        assert_eq!(input, attempted_input, "Task252 field {field} input purity");
        replay_pristine();
    }

    for field in 0..23 {
        let pristine_input = B3M2B2B3BFixture::set_input(fixture.source, &fixture.module);
        let mut input = pristine_input.clone();
        match field {
            0 => input.source_id = source_id(999_255),
            1 => {
                input.module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
            }
            2 => input.terms.clear(),
            3 => input.terms.push(input.terms[0].clone()),
            4 => input.terms[0].site = node(27),
            5 => input.terms[0].source_range.start += 1,
            6 => input.terms[0].source_ordinal += 1,
            7 => input.terms[0].context = BindingContextId::new(0),
            8 => input.terms[0].recovery = SourceSetTermRecovery::Degraded,
            9 => input.terms[0].spelling.push('!'),
            10 => input.terms[0].kind = SourceSetTermKind::Choice,
            11 => input.wrappers.push(SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: input.terms[0].site.clone(),
                source_range: input.terms[0].source_range,
                context: input.terms[0].context,
                recovery: SourceSetTermRecovery::Normal,
                spelling: "( { } )".to_owned(),
            }),
            12 => input.generators.push(SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: input.terms[0].site.clone(),
                source_range: input.terms[0].source_range,
                spelling: "x being set".to_owned(),
                context: input.terms[0].context,
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            }),
            13 => input.type_sites.push(SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Term {
                    term: SourceSetTermId::new(0),
                    role: SourceSetTypeRole::ChoiceTarget,
                },
                site: input.terms[0].site.clone(),
                source_range: input.terms[0].source_range,
                spelling: "set".to_owned(),
                head_site: input.terms[0].site.clone(),
                head_range: input.terms[0].source_range,
                head_spelling: "set".to_owned(),
                context: input.terms[0].context,
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            }),
            14 => input.conditions.push(SourceSetConditionInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                colon_site: input.terms[0].site.clone(),
                colon_range: input.terms[0].source_range,
                colon_spelling: ":".to_owned(),
                condition_site: input.terms[0].site.clone(),
                source_range: input.terms[0].source_range,
                spelling: "x = x".to_owned(),
                recovery: SourceSetTermRecovery::Normal,
            }),
            15 => input.edges.push(SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::EnumerationElement,
                target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
            }),
            16 => input.requests.clear(),
            17 => input.requests.push(input.requests[0].clone()),
            18 => input.requests[0].term = SourceSetTermId::new(1),
            19 => input.requests[0].ordinal += 1,
            20 => input.requests[0].kind = SourceSetRequestKind::ChoiceNonempty,
            21 => input.requests[0].generator = Some(SourceSetGeneratorId::new(0)),
            22 => input.requests[0].type_site = Some(SourceSetTypeSiteId::new(0)),
            _ => unreachable!(),
        }
        assert_ne!(input, pristine_input, "Task255 field {field} input delta");
        if field == 15 {
            assert_eq!(input.edges.len(), 1, "Task255 real enumeration edge");
        }
        let expected = match field {
            0..=2 => Some(crate::source_set_term::SourceSetTermError::EnvironmentMismatch),
            3 => Some(crate::source_set_term::SourceSetTermError::InvalidTerm {
                term: SourceSetTermId::new(1),
            }),
            4..=6 | 8..=10 | 12 | 14 | 15 => {
                Some(crate::source_set_term::SourceSetTermError::InvalidTerm {
                    term: SourceSetTermId::new(0),
                })
            }
            7 => None,
            11 => Some(crate::source_set_term::SourceSetTermError::InvalidWrapper {
                wrapper: crate::source_set_term::SourceSetWrapperId::new(0),
            }),
            13 => Some(
                crate::source_set_term::SourceSetTermError::InvalidTypeSite {
                    type_site: SourceSetTypeSiteId::new(0),
                },
            ),
            16 | 20 | 21 | 22 => Some(crate::source_set_term::SourceSetTermError::InvalidRequest {
                request: crate::source_set_term::SourceSetRequestId::new(0),
            }),
            17 => Some(
                crate::source_set_term::SourceSetTermError::ReorderedRequest {
                    request: crate::source_set_term::SourceSetRequestId::new(1),
                },
            ),
            18 | 19 => Some(
                crate::source_set_term::SourceSetTermError::ReorderedRequest {
                    request: crate::source_set_term::SourceSetRequestId::new(0),
                },
            ),
            _ => unreachable!(),
        };
        let attempted_input = input.clone();
        match (
            expected,
            SourceSetTermProducer::build(
                input.clone(),
                &fixture.bindings,
                &fixture.primary,
                None,
                None,
                &fixture.arena,
            ),
        ) {
            (Some(expected), Err(actual)) => {
                assert_eq!(actual, expected, "Task255 field {field}")
            }
            (Some(expected), Ok(_)) => {
                panic!("Task255 field {field} admitted; expected {expected:?}")
            }
            (None, Ok(set_terms)) => {
                assert_ne!(
                    set_terms, fixture.set_terms,
                    "Task255 field {field} handoff delta"
                );
                assert_eq!(set_terms.source_id(), fixture.source);
                assert_eq!(set_terms.module_id(), &fixture.module);
                assert_eq!(
                    witnesses.validate_installation_with_set_term(
                        fixture.source,
                        &fixture.module,
                        &statement,
                        &fixture.primary,
                        Some(&set_terms),
                        &fixture.arena,
                    ),
                    Err(SourceStatementWitnessError::DependencyMismatch),
                    "Task255 field {field} admitted set handoff reaches witness dependency check"
                );
            }
            (None, Err(actual)) => {
                panic!("Task255 field {field} refused as {actual:?}; expected admitted set handoff")
            }
        }
        assert_eq!(input, attempted_input, "Task255 field {field} input purity");
        replay_pristine();
    }

    let pristine_atomic_input = B3M2B2B3BFixture::atomic_input(fixture.source, &fixture.module);
    assert_eq!(
        SourceAtomicFormulaProducer::build(
            pristine_atomic_input.clone(),
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        ),
        Ok(fixture.atomic.clone()),
        "Task256 pristine constructor control"
    );
    for field in 0..72 {
        let mut input = pristine_atomic_input.clone();
        task258b3m2b2b3d_mutate_atomic_input(
            &mut input,
            &fixture.symbol,
            fixture.contribution,
            field,
        );
        assert_ne!(
            input, pristine_atomic_input,
            "Task256 field {field} input delta"
        );
        let expected = match field {
            0 | 60 | 61 => {
                Some(crate::source_atomic_formula::SourceAtomicFormulaError::EnvironmentMismatch)
            }
            1 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::InvalidFormula {
                    formula: SourceAtomicFormulaId::new(2),
                },
            ),
            2..16 => {
                let offset = field - 2;
                let formula = offset / 7;
                if offset % 7 == 3 {
                    Some(crate::source_atomic_formula::SourceAtomicFormulaError::PrimaryDependencyMismatch)
                } else {
                    Some(
                        crate::source_atomic_formula::SourceAtomicFormulaError::InvalidFormula {
                            formula: SourceAtomicFormulaId::new(formula),
                        },
                    )
                }
            }
            16..32 => {
                let offset = field - 16;
                let edge = offset / 4;
                match offset % 4 {
                    0 | 1 => Some(
                        crate::source_atomic_formula::SourceAtomicFormulaError::ReorderedEdge {
                            edge: SourceAtomicEdgeId::new(if offset % 4 == 0 {
                                edge.max(1)
                            } else {
                                edge
                            }),
                        },
                    ),
                    2 => Some(
                        crate::source_atomic_formula::SourceAtomicFormulaError::InvalidFormula {
                            formula: SourceAtomicFormulaId::new(edge / 2),
                        },
                    ),
                    3 => Some(
                        crate::source_atomic_formula::SourceAtomicFormulaError::InvalidEdge {
                            edge: SourceAtomicEdgeId::new(edge),
                        },
                    ),
                    _ => unreachable!(),
                }
            }
            32..60 => {
                let offset = field - 32;
                let request = offset / 7;
                match offset % 7 {
                    0 | 1 => Some(
                        crate::source_atomic_formula::SourceAtomicFormulaError::ReorderedRequest {
                            request: SourceAtomicRequestId::new(if offset % 7 == 0 {
                                request.max(1)
                            } else {
                                request
                            }),
                        },
                    ),
                    2..=6 => Some(
                        crate::source_atomic_formula::SourceAtomicFormulaError::InvalidRequest {
                            request: SourceAtomicRequestId::new(request),
                        },
                    ),
                    _ => unreachable!(),
                }
            }
            62 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::InvalidWrapper {
                    wrapper: crate::source_atomic_formula::SourceAtomicWrapperId::new(0),
                },
            ),
            63 | 67 | 68 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::InvalidFormula {
                    formula: SourceAtomicFormulaId::new(0),
                },
            ),
            64 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::ReorderedPredicateHead {
                    head: SourcePredicateHeadId::new(0),
                },
            ),
            65 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::ReorderedCandidate {
                    candidate: SourcePredicateCandidateId::new(0),
                },
            ),
            66 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::InvalidTypeSite {
                    type_site: crate::source_atomic_formula::SourceAssertionTypeSiteId::new(0),
                },
            ),
            69 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::ReorderedEdge {
                    edge: SourceAtomicEdgeId::new(4),
                },
            ),
            70 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::InvalidRequest {
                    request: SourceAtomicRequestId::new(0),
                },
            ),
            71 => Some(
                crate::source_atomic_formula::SourceAtomicFormulaError::ReorderedRequest {
                    request: SourceAtomicRequestId::new(4),
                },
            ),
            _ => unreachable!(),
        };
        assert!(
            expected.is_some(),
            "Task256 field {field} has a typed producer oracle"
        );
        let attempted_input = input.clone();
        let actual = SourceAtomicFormulaProducer::build(
            input.clone(),
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        );
        match (expected, actual) {
            (Some(expected), Err(actual)) => {
                assert_eq!(actual, expected, "Task256 field {field}")
            }
            (Some(expected), Ok(_)) => {
                panic!("Task256 field {field} admitted; expected {expected:?}")
            }
            (None, _) => unreachable!(),
        }
        assert_eq!(input, attempted_input, "Task256 field {field} input purity");
        replay_pristine();
    }

    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Ok(pristine_statement.clone()),
        "Task258 pristine constructor control"
    );
    for field in 0..62 {
        let pristine_input = fixture.statement_input();
        let mut input = pristine_input.clone();
        task258b3m2b2b3d_mutate_statement_input(&mut input, field);
        assert_ne!(input, pristine_input, "Task258 field {field} input delta");
        let expected = match field {
            0 | 1 | 6 | 7 | 24 | 25 | 34 | 35 | 46 | 47 => SourceStatementError::InvalidAggregate,
            2 | 3 | 4 | 5 | 58 | 59 => SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
            56 | 57 => SourceStatementError::DependencyMismatch,
            8..24 => SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new((field - 8) / 8),
            },
            26..34 => SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new((field - 26) / 4),
            },
            36..46 => SourceStatementError::InvalidInputFact {
                fact: SourceStatementInputFactId::new((field - 36) / 5),
            },
            48..56 => SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new((field - 48) / 4),
            },
            60 | 61 => SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(field - 60),
            },
            _ => unreachable!(),
        };
        let attempted_input = input.clone();
        assert_eq!(
            SourceStatementProducer::build(
                input.clone(),
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
            ),
            Err(expected),
            "Task258 field {field}"
        );
        assert_eq!(input, attempted_input, "Task258 field {field} input purity");
        replay_pristine();
    }

    for field in 0..21 {
        let pristine_input = fixture.witness_input();
        let mut input = pristine_input.clone();
        task258b3m2b2b3d_mutate_witness_input(&mut input, field);
        assert_ne!(input, pristine_input, "witness field {field} input delta");
        let expected = match field {
            0 | 1 | 14 => SourceStatementWitnessError::InvalidAggregate,
            15 | 16 => SourceStatementWitnessError::DependencyMismatch,
            _ => SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            },
        };
        let attempted_input = input.clone();
        assert_eq!(
            SourceStatementWitnessProducer::build_with_set_term(
                input.clone(),
                &statement,
                &fixture.primary,
                &fixture.set_terms,
                &fixture.arena,
            ),
            Err(expected),
            "witness field {field}"
        );
        assert_eq!(input, attempted_input, "witness field {field} input purity");
        replay_pristine();
    }

    let mut owner_endpoint = fixture.statement_input();
    owner_endpoint.owners[0].source_range.end -= 1;
    assert_ne!(owner_endpoint, fixture.statement_input());
    assert_eq!(
        SourceStatementProducer::build(
            owner_endpoint,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0),
        }),
        "Task258 owner range endpoint 116"
    );
    replay_pristine();
    let mut conclusion_ordinal = fixture.statement_input();
    conclusion_ordinal.statements[1].source_ordinal = 1;
    assert_ne!(conclusion_ordinal, fixture.statement_input());
    assert_eq!(
        SourceStatementProducer::build(
            conclusion_ordinal,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(1),
        }),
        "Task258 conclusion ordinal 1"
    );
    replay_pristine();
    let mut context_endpoint = fixture.statement_input();
    context_endpoint.contexts[1].source_range.end -= 1;
    assert_ne!(context_endpoint, fixture.statement_input());
    assert_eq!(
        SourceStatementProducer::build(
            context_endpoint,
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidContext {
            context: SourceStatementContextId::new(1),
        }),
        "Task258 context range endpoint 111"
    );
    replay_pristine();
    for (label, mutate) in [
        (
            "primary witness target",
            (|input: &mut SourceStatementWitnessHandoffInput| {
                input.witnesses[0].term =
                    SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(0))
            }) as fn(&mut SourceStatementWitnessHandoffInput),
        ),
        ("take range endpoint", |input| {
            input.witnesses[0].take_range.end -= 1
        }),
        ("take site", |input| input.witnesses[0].site = node(33)),
    ] {
        let pristine_input = fixture.witness_input();
        let mut input = pristine_input.clone();
        mutate(&mut input);
        assert_ne!(
            input, pristine_input,
            "retained witness fault {label} delta"
        );
        assert_eq!(
            SourceStatementWitnessProducer::build_with_set_term(
                input,
                &statement,
                &fixture.primary,
                &fixture.set_terms,
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::InvalidWitness {
                witness: SourceStatementWitnessId::new(0),
            }),
            "retained witness fault {label}"
        );
        replay_pristine();
    }

    let proof_end_changed = B3M2B2B3BFixture::binding_env_with_mutation(
        fixture.source,
        &fixture.module,
        BindingContextRecovery::Normal,
        115,
        0,
    );
    assert_ne!(proof_end_changed, pristine_fixture.bindings);
    assert_eq!(
        SourceStatementProducer::build(
            fixture.statement_input(),
            &fixture.symbols,
            &proof_end_changed,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "Task48 proof context end 115 remains a downstream dependency fault"
    );
    replay_pristine();
    for term in 0..4 {
        let mut primary = fixture.primary.clone();
        primary.set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(term), 999);
        assert_ne!(
            primary, fixture.primary,
            "Task252 reference {term} use-clock delta"
        );
        assert_eq!(
            primary
                .references()
                .get(SourcePrimaryTermReferenceId::new(term))
                .unwrap()
                .use_ordinal(),
            999
        );
        assert_eq!(
            witnesses.validate_installation_with_set_term(
                fixture.source,
                &fixture.module,
                &statement,
                &primary,
                Some(&fixture.set_terms),
                &fixture.arena,
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "Task252 reference {term}"
        );
        replay_pristine();
    }
    let pristine_empty_set_input = B3M2B2B3BFixture::set_input(fixture.source, &fixture.module);
    let mut unexpected_edge = pristine_empty_set_input.clone();
    unexpected_edge.edges.push(SourceSetEdgeInput {
        term: SourceSetTermId::new(0),
        ordinal: 0,
        role: SourceSetEdgeRole::EnumerationElement,
        target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
    });
    assert_eq!(unexpected_edge.edges.len(), 1);
    assert_ne!(unexpected_edge, pristine_empty_set_input);
    assert_eq!(
        SourceSetTermProducer::build(
            unexpected_edge,
            &fixture.bindings,
            &fixture.primary,
            None,
            None,
            &fixture.arena,
        ),
        Err(crate::source_set_term::SourceSetTermError::InvalidTerm {
            term: SourceSetTermId::new(0),
        }),
        "Task255 must reject a real edge row for the empty enumeration"
    );
    replay_pristine();
    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_238);
    dependency_before_aggregate.witnesses.clear();
    assert_ne!(dependency_before_aggregate, fixture.witness_input());
    assert_eq!(
        SourceStatementWitnessProducer::build_with_set_term(
            dependency_before_aggregate,
            &statement,
            &fixture.primary,
            &fixture.set_terms,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    replay_pristine();
}

#[test]
fn task258b3m2b2b3b_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B3BFixture::new(238);
    let statement = fixture.statement();
    let witnesses = fixture.witnesses(&statement).expect("witnesses");
    let base = fixture.empty_typed();
    let base_debug = base.debug_text();
    let typed = base
        .clone()
        .with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("combined install");
    assert_eq!(typed.source_set_term(), Some(&fixture.set_terms));
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_set_term(fixture.set_terms.clone())
            .expect("lower set first")
            .with_source_set_term_statement_witnesses(
                fixture.set_terms.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let pristine_fixture = Box::new(fixture.clone());
    let pristine_input = fixture.statement_input();
    let replay_pristine = |expected: Option<&ResolvedTypedAst>| {
        assert_eq!(
            (
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.set_terms,
                &fixture.atomic,
                &fixture.arena
            ),
            (
                &pristine_fixture.symbols,
                &pristine_fixture.bindings,
                &pristine_fixture.primary,
                &pristine_fixture.set_terms,
                &pristine_fixture.atomic,
                &pristine_fixture.arena
            ),
            "owner dependency purity"
        );
        let replay = fixture.statement();
        assert_eq!(replay, statement, "owner statement replay");
        let replay_witnesses = fixture
            .witnesses(&replay)
            .expect("pristine witnesses replay");
        assert_eq!(replay_witnesses, witnesses, "owner witness replay");
        let replay_typed = fixture
            .empty_typed()
            .with_source_set_term_statement_witnesses(
                fixture.set_terms.clone(),
                replay,
                replay_witnesses,
            )
            .expect("pristine typed replay");
        assert_eq!(replay_typed, typed, "owner typed replay");
        let resolved =
            Box::new(assemble_empty_resolved(&replay_typed).expect("pristine final replay"));
        if let Some(expected) = expected {
            assert_eq!(resolved.as_ref(), expected, "owner final replay");
        }
        resolved
    };
    let pristine_resolved = replay_pristine(None);
    let pristine_label = fixture.symbols.labels().iter().next().expect("B3B label");
    let pristine_contribution = fixture
        .symbols
        .contributions()
        .get(fixture.contribution)
        .expect("B3B contribution");
    for (case, name) in [
        "Imported",
        "Missing",
        "Duplicate",
        "WrongPath",
        "WrongKind",
        "Private",
        "LocalOnly",
        "Recovered",
        "Namespace",
        "PrimarySpelling",
        "Contribution",
        "OriginSource",
        "OriginModule",
        "OriginAnchor",
        "OriginStructuralPath",
        "ContributionEffectsMissing",
        "CoherentContributionEffectsExtra",
        "ContributionEffectsWrongPath",
    ]
    .into_iter()
    .enumerate()
    {
        let extra_path =
            LabelOriginPath::new(format!("{}.extra", pristine_label.origin_path().as_str()));
        let mut path = pristine_label.origin_path().clone();
        let mut kind = pristine_label.kind();
        let mut namespace = pristine_label.namespace().clone();
        let mut spelling = pristine_label.primary_spelling().to_owned();
        let mut origin = pristine_label.origin().clone();
        let mut contribution = pristine_label.contribution();
        let mut visibility = pristine_label.visibility();
        let mut export_status = pristine_label.export_status();
        let mut contributions = fixture.symbols.contributions().clone();
        match case {
            0 => {
                let mut nodes = ResolvedArenaBuilder::new();
                let owner = nodes
                    .push(ResolvedNode::new(
                        syntax::SurfaceNodeKind::ImportAliasDecl,
                        Vec::new(),
                        origin.clone(),
                    ))
                    .expect("imported label owner");
                let mut imports = ResolvedImports::new();
                let import = imports.push_import(ResolvedImport::new(
                    owner,
                    range(fixture.source, 0, 1),
                    "import statement.fixture;",
                    None,
                    ImportResolution::Resolved(fixture.module.clone()),
                    origin.clone(),
                ));
                origin = origin.with_import_edge(import);
            }
            3 => path = extra_path.clone(),
            4 => kind = LabelKind::ProofStep,
            5 => visibility = Visibility::Private,
            6 => export_status = ExportStatus::LocalOnly,
            7 => origin = origin.recovered(),
            8 => namespace = NamespacePath::new("statement.foreign"),
            9 => spelling.push('!'),
            10 => {
                contribution = contributions.insert(
                    pristine_contribution.module().clone(),
                    pristine_contribution.kind().clone(),
                    pristine_contribution.anchor().clone(),
                )
            }
            11..=14 => {
                let mut source = origin.source_id();
                let mut module = origin.module_id().clone();
                let mut anchor = origin.anchor().clone();
                let mut structural_path = origin.structural_path().to_vec();
                match case {
                    11 => source = source_id(999_238),
                    12 => {
                        module = ModuleId::new(
                            PackageId::new("pkg"),
                            ModulePath::new("statement.foreign"),
                        )
                    }
                    13 => {
                        let mut source_range = pristine_input.owners[0].source_range;
                        source_range.start += 1;
                        source_range.end -= 1;
                        anchor = SourceAnchor::Range(source_range);
                    }
                    14 => *structural_path.last_mut().expect("B3B owner path") = 9,
                    _ => unreachable!(),
                }
                origin = SemanticOrigin::new(source, module, anchor, structural_path);
            }
            _ => {}
        }
        let mut labels = LabelIndex::new();
        if case != 1 {
            labels.insert(
                LabelEntry::new(path, kind, namespace, spelling, origin, contribution)
                    .with_visibility(visibility)
                    .with_export_status(export_status),
            );
            if case == 2 {
                labels.insert(
                    LabelEntry::new(
                        extra_path.clone(),
                        pristine_label.kind(),
                        pristine_label.namespace().clone(),
                        pristine_label.primary_spelling(),
                        pristine_label.origin().clone(),
                        pristine_label.contribution(),
                    )
                    .with_visibility(pristine_label.visibility())
                    .with_export_status(pristine_label.export_status()),
                );
            }
        }
        if case >= 15 {
            contributions = SourceContributionIndex::new();
            let id = contributions.insert(
                pristine_contribution.module().clone(),
                pristine_contribution.kind().clone(),
                pristine_contribution.anchor().clone(),
            );
            assert_eq!(id, fixture.contribution);
            for symbol in pristine_contribution.effects().symbols() {
                contributions.add_symbol(id, symbol.clone());
            }
            for definition in pristine_contribution.effects().definitions() {
                contributions.add_definition(id, *definition);
            }
            let mut reconstructed = contributions.clone();
            for label in pristine_contribution.effects().labels() {
                reconstructed.add_label(id, label.clone());
            }
            assert_eq!(reconstructed, *fixture.symbols.contributions());
            if case == 16 {
                contributions.add_label(id, pristine_label.origin_path().clone());
            }
            if case != 15 {
                contributions.add_label(id, extra_path.clone());
            }
            let expected_effects = match case {
                15 => Vec::new(),
                16 => vec![pristine_label.origin_path().clone(), extra_path.clone()],
                17 => vec![extra_path.clone()],
                _ => unreachable!(),
            };
            assert_eq!(
                contributions
                    .get(id)
                    .expect("mutated contribution")
                    .effects()
                    .labels(),
                expected_effects,
                "{name} exact label effects"
            );
            assert_ne!(
                contributions,
                *fixture.symbols.contributions(),
                "{name} effect delta"
            );
            assert_eq!(labels, *fixture.symbols.labels(), "{name} unchanged labels");
        } else {
            assert_ne!(labels, *fixture.symbols.labels(), "{name} label delta");
        }
        let attempted_symbols = SymbolEnv::new(
            fixture.module.clone(),
            SymbolEnvIndexes {
                imports: fixture.symbols.imports().clone(),
                exports: fixture.symbols.exports().clone(),
                symbols: fixture.symbols.symbols().clone(),
                labels,
                definitions: fixture.symbols.definitions().clone(),
                overloads: fixture.symbols.overloads().clone(),
                registrations: fixture.symbols.registrations().clone(),
                lexical_summaries: fixture.symbols.lexical_summaries().clone(),
                namespace_graph: fixture.symbols.namespace_graph().clone(),
                declaration_dependencies: fixture.symbols.declaration_dependencies().clone(),
                contributions,
                module_summaries: fixture.symbols.module_summaries().clone(),
            },
        );
        assert_ne!(
            attempted_symbols, fixture.symbols,
            "{name} environment delta"
        );
        assert_eq!(
            attempted_symbols.imports(),
            fixture.symbols.imports(),
            "{name} imports"
        );
        if case == 3 {
            let changed_label = attempted_symbols
                .labels()
                .iter()
                .next()
                .expect("wrong-path label");
            assert!(
                !pristine_contribution
                    .effects()
                    .labels()
                    .contains(changed_label.origin_path())
            );
        }
        let attempted_before = attempted_symbols.clone();
        let input = pristine_input.clone();
        let result = SourceStatementProducer::build(
            input.clone(),
            &attempted_symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        );
        if case == 16 {
            let admitted = result.expect("coherent extra label effect is privately admitted");
            assert_eq!(admitted, statement, "{name} unchanged statement rows");
        } else {
            assert_eq!(
                result,
                Err(SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }),
                "{name}"
            );
        }
        assert_eq!(
            attempted_symbols, attempted_before,
            "{name} constructor purity"
        );
        assert_eq!(input, pristine_input, "{name} input purity");
        assert_eq!(
            fixture.statement_input(),
            pristine_input,
            "{name} pristine input"
        );
        replay_pristine(Some(&pristine_resolved));
    }

    let application_family = B3M2B2B1AFixture::new(238);
    let structure_family = B3M2B2B2CFixture::new(238);
    for mask in 0_u8..8 {
        let result = witnesses.validate_installation_inner(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            (mask & 1 != 0).then_some(&application_family.application),
            (mask & 2 != 0).then_some(&structure_family.structure),
            (mask & 4 != 0).then_some(&fixture.set_terms),
            &fixture.arena,
        );
        assert_eq!(
            result.is_ok(),
            mask == 4,
            "application/structure/set tuple {mask:03b}"
        );
    }
    let legacy = B3M2B2B3AFixture::new(238);
    let legacy_statement = legacy.statement();
    let legacy_witnesses = legacy
        .witnesses(&legacy_statement)
        .expect("legacy witnesses");
    let legacy_typed = legacy
        .empty_typed()
        .with_source_set_term_statement_witnesses(
            legacy.set_terms.clone(),
            legacy_statement.clone(),
            legacy_witnesses.clone(),
        )
        .expect("legacy installed");
    assert_eq!(
        base.clone().with_source_set_term_statement_witnesses(
            legacy.set_terms.clone(),
            legacy_statement.clone(),
            legacy_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "B3B base then B3A"
    );
    assert_eq!(
        legacy
            .empty_typed()
            .with_source_set_term_statement_witnesses(
                fixture.set_terms.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement),
        "B3A base then B3B"
    );
    assert_eq!(
        typed.clone().with_source_set_term_statement_witnesses(
            legacy.set_terms,
            legacy_statement,
            legacy_witnesses,
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B3B then B3A"
    );
    assert_eq!(
        legacy_typed.with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B3A then B3B"
    );

    let reject_b3b_for_family = |label: &str, family_base: TypedAst, family_installed: TypedAst| {
        assert_eq!(
            family_base
                .clone()
                .with_source_set_term_statement_witnesses(
                    fixture.set_terms.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
            Err(TypedAstError::InvalidSourceStatement),
            "{label} base then B3B"
        );
        assert_eq!(
            family_installed
                .clone()
                .with_source_set_term_statement_witnesses(
                    fixture.set_terms.clone(),
                    statement.clone(),
                    witnesses.clone(),
                ),
            Err(TypedAstError::InvalidSourceStatement),
            "installed {label} then B3B"
        );
        assert_eq!(
            family_base.with_source_set_term(fixture.set_terms.clone()),
            Err(TypedAstError::InvalidSourceSetTerm),
            "{label} base rejects B3B set injection"
        );
        assert_eq!(
            family_installed.with_source_set_term(fixture.set_terms.clone()),
            Err(TypedAstError::InvalidSourceSetTerm),
            "installed {label} rejects B3B set injection"
        );
    };
    macro_rules! reject_plain_witness_family_for_b3b {
        ($label:literal, $family:expr) => {{
            let family = $family;
            let family_statement = family.statement();
            let family_witnesses = family
                .witnesses(&family_statement, family.witness_input())
                .expect(concat!($label, " witnesses"));
            let family_base = family.empty_typed();
            let family_installed = family_base
                .clone()
                .with_source_statement_witnesses(family_statement.clone(), family_witnesses.clone())
                .expect(concat!($label, " installed"));
            assert_eq!(
                base.clone().with_source_statement_witnesses(
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("B3B base then ", $label)
            );
            assert_eq!(
                typed.clone().with_source_statement_witnesses(
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B3B then ", $label)
            );
            reject_b3b_for_family($label, family_base, family_installed);
        }};
    }
    reject_plain_witness_family_for_b3b!("B3", B3Fixture::new(238));
    reject_plain_witness_family_for_b3b!("B3N", B3NFixture::new(238));
    reject_plain_witness_family_for_b3b!("B3M1", B3M1Fixture::new(238));
    reject_plain_witness_family_for_b3b!("B3M2A", B3M2AFixture::new(238));
    reject_plain_witness_family_for_b3b!("B3M2B1", B3M2B1Fixture::new(238));
    reject_plain_witness_family_for_b3b!("B3M2B2A", B3M2B2AFixture::new(238));

    macro_rules! reject_application_witness_family_for_b3b {
        ($label:literal, $family:expr) => {{
            let family = $family;
            let family_statement = family.statement();
            let family_witnesses = family
                .witnesses(&family_statement, family.witness_input())
                .expect(concat!($label, " witnesses"));
            let family_base = family.empty_typed();
            let family_installed = family_base
                .clone()
                .with_source_application_statement_witnesses(
                    family.application.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                )
                .expect(concat!($label, " installed"));
            assert_eq!(
                base.clone().with_source_application_statement_witnesses(
                    family.application.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("B3B base then ", $label)
            );
            assert_eq!(
                typed.clone().with_source_application_statement_witnesses(
                    family.application.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B3B then ", $label)
            );
            reject_b3b_for_family($label, family_base, family_installed);
        }};
    }
    reject_application_witness_family_for_b3b!("B1A", B3M2B2B1AFixture::new(238));
    reject_application_witness_family_for_b3b!("B1B1", B3M2B2B1B1Fixture::new(238));

    macro_rules! reject_structure_witness_family_for_b3b {
        ($label:literal, $family:expr) => {{
            let family = $family;
            let family_statement = family.statement();
            let family_witnesses = family
                .witnesses(&family_statement, family.witness_input())
                .expect(concat!($label, " witnesses"));
            let family_base = family.empty_typed();
            let family_installed = family_base
                .clone()
                .with_source_structure_statement_witnesses(
                    family.structure.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                )
                .expect(concat!($label, " installed"));
            assert_eq!(
                base.clone().with_source_structure_statement_witnesses(
                    family.structure.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("B3B base then ", $label)
            );
            assert_eq!(
                typed.clone().with_source_structure_statement_witnesses(
                    family.structure.clone(),
                    family_statement.clone(),
                    family_witnesses.clone(),
                ),
                Err(TypedAstError::InvalidSourceStatement),
                concat!("installed B3B then ", $label)
            );
            reject_b3b_for_family($label, family_base, family_installed);
        }};
    }
    reject_structure_witness_family_for_b3b!("B2A", B3M2B2B2AFixture::new(238));
    reject_structure_witness_family_for_b3b!("B2B", B3M2B2B2BFixture::new(238));
    reject_structure_witness_family_for_b3b!("B2C", B3M2B2B2CFixture::new(238));

    assert_eq!(
        typed.clone().with_source_application_statement_witnesses(
            application_family.application.clone(),
            application_family.statement(),
            application_family
                .witnesses(
                    &application_family.statement(),
                    application_family.witness_input(),
                )
                .expect("application witnesses"),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        typed.with_source_structure_statement_witnesses(
            structure_family.structure.clone(),
            structure_family.statement(),
            structure_family
                .witnesses(
                    &structure_family.statement(),
                    structure_family.witness_input(),
                )
                .expect("structure witnesses"),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(base.debug_text(), base_debug);
}

#[test]
fn task258b3m2b2b3b_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B3BFixture::new(239);
    let statement = fixture.statement();
    let witnesses = fixture.witnesses(&statement).expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        )
        .expect("typed");
    let typed_debug = typed.debug_text();
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
    let resolved_debug = resolved.debug_text();
    assert_eq!(typed.source_term(), Some(&fixture.primary));
    assert_eq!(typed.source_atomic_formula(), Some(&fixture.atomic));
    assert_eq!(resolved.source_term(), typed.source_term());
    assert_eq!(
        resolved.source_atomic_formula(),
        typed.source_atomic_formula()
    );
    assert!(typed.contexts().is_empty());
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert!(typed.source_composite_formula().is_none());
    assert!(typed.source_formula_composition().is_none());
    assert!(typed.source_condition_formula_composition().is_none());
    assert!(typed.source_predicate_chain_composition().is_none());
    assert!(resolved.source_composite_formula().is_none());
    assert!(resolved.source_formula_composition().is_none());
    assert!(resolved.source_condition_formula_composition().is_none());
    assert!(resolved.source_predicate_chain_composition().is_none());
    assert_eq!(resolved.source_set_term(), Some(&fixture.set_terms));
    assert_eq!(resolved.source_statement(), Some(&statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
    assert!(resolved.source_application().is_none());
    assert!(resolved.source_structure().is_none());
    assert!(resolved.source_statement_references().is_none());
    assert!(resolved.source_context().is_none());
    assert!(resolved.source_type().is_none());
    assert!(resolved.source_attribute().is_none());
    assert!(resolved.source_evidence().is_none());
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
    assert_eq!(typed.clone().debug_text(), typed_debug);
    assert_eq!(resolved.clone().debug_text(), resolved_debug);

    for (label, mutate) in [
        (
            "statement fingerprint",
            (|row: &mut SourceStatementWitnessHandoff| row.statement_fingerprint.push('x'))
                as fn(&mut SourceStatementWitnessHandoff),
        ),
        ("primary fingerprint", |row| {
            row.primary_term_fingerprint.push('x')
        }),
        ("missing set fingerprint", |row| {
            row.set_term_fingerprint = None
        }),
        ("stale set fingerprint", |row| {
            row.set_term_fingerprint = Some("stale".to_owned())
        }),
        ("application hybrid", |row| {
            row.application_fingerprint = Some("hybrid".to_owned())
        }),
        ("structure hybrid", |row| {
            row.structure_fingerprint = Some("hybrid".to_owned())
        }),
    ] {
        let mut stale = witnesses.clone();
        mutate(&mut stale);
        let mut stale_typed = typed.clone();
        stale_typed.inject_source_statement_witness_bundle_for_test(statement.clone(), stale);
        assert_eq!(
            assemble_empty_resolved(&stale_typed),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{label}"
        );
    }
    let foreign = B3M2B2B3BFixture::new(240);
    let mut invalid_lower_pair = typed.clone();
    invalid_lower_pair.replace_source_set_term_and_atomic_formula_for_test(
        foreign.set_terms.clone(),
        foreign.atomic.clone(),
    );
    assert_eq!(
        assemble_empty_resolved(&invalid_lower_pair),
        Err(ResolvedTypedAstError::InvalidSourceSetTerm)
    );
    let mut invalid_atomic = typed.clone();
    invalid_atomic.replace_source_set_term_and_atomic_formula_for_test(
        fixture.set_terms.clone(),
        foreign.atomic,
    );
    assert_eq!(
        assemble_empty_resolved(&invalid_atomic),
        Err(ResolvedTypedAstError::InvalidSourceAtomicFormula)
    );
    let mut bundle_without_set = fixture.empty_typed();
    bundle_without_set
        .inject_source_statement_witness_bundle_for_test(statement.clone(), witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&bundle_without_set),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    for table in [
        StatementTransportTableForTest::Context,
        StatementTransportTableForTest::Type,
        StatementTransportTableForTest::Fact,
        StatementTransportTableForTest::Coercion,
        StatementTransportTableForTest::InitialObligation,
        StatementTransportTableForTest::Diagnostic,
    ] {
        let mut occupied = typed.clone();
        occupied.occupy_statement_transport_table_for_test(table);
        assert_eq!(
            assemble_empty_resolved(&occupied),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{table:?}"
        );
    }
    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b3b-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 95, 97),
    });
    assert_eq!(
        assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved(
            &typed,
            &ClusterFactTable::new(),
            Vec::new(),
            Some(StatementProofInputs {
                owner: statement.checked_owner(),
                rows: Vec::new(),
            }),
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved_with_expressions(
            &typed,
            vec![ExpressionMetadataInput {
                expr: ExprId::new("task258b3m2b2b3b-semantic-coexistence"),
                typed_site: node(33),
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("final replay")
            .debug_text(),
        resolved_debug
    );
}
