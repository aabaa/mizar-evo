use super::*;

#[test]
fn exact_profile_exposes_complete_api_and_stable_debug() {
    let fixture = Fixture::new(1);
    let handoff = fixture.build(fixture.input()).expect("valid statement");
    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!(handoff.binding_env(), &fixture.bindings);
    assert_eq!(handoff.binding_fingerprint(), fixture.bindings.debug_text());
    assert_eq!(
        handoff.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(
        handoff.atomic_formula_fingerprint(),
        fixture.atomic.debug_text()
    );
    assert_eq!(handoff.checked_owner().symbol(), &fixture.symbol);

    let owner_id = SourceTheoremOwnerId::new(0);
    let owner = handoff.owners().get(owner_id).expect("owner");
    assert_eq!(owner_id.index(), 0);
    assert_eq!(owner.symbol(), &fixture.symbol);
    assert_eq!(owner.contribution(), fixture.contribution);
    assert_eq!(owner.site(), &node(5));
    assert_eq!(owner.source_range(), range(fixture.source, 19, 80));
    assert_eq!(owner.spelling(), LABEL);
    assert_eq!(owner.role(), SourceTheoremRole::Theorem);
    assert_eq!(owner.status(), SourceTheoremStatus::Unmodified);
    assert_eq!(owner.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(handoff.owners().iter().count(), 1);
    assert!(!handoff.owners().is_empty());

    let statement_id = SourceStatementId::new(0);
    let statement = handoff.statements().get(statement_id).expect("statement");
    assert_eq!(statement.owner(), owner_id);
    assert_eq!(statement.context(), SourceStatementContextId::new(0));
    assert_eq!(
        statement.formula(),
        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
    );
    assert_eq!(statement.site(), &node(5));
    assert_eq!(statement.source_range(), range(fixture.source, 19, 80));
    assert_eq!(statement.source_ordinal(), 0);
    assert_eq!(statement.spelling(), STATEMENT);
    assert_eq!(statement.kind(), SourceStatementKind::TheoremProposition);
    assert_eq!(statement.recovery(), SourceStatementRecovery::Normal);
    let theorem_node = fixture.arena.node(node(5).node()).expect("theorem node");
    assert_eq!(theorem_node.children, [TypedNodeId::new(4)]);
    let wrapper_node = fixture.arena.node(node(4).node()).expect("formula wrapper");
    assert_eq!(wrapper_node.kind.as_str(), "source.surface.unowned");
    assert_eq!(wrapper_node.children, [TypedNodeId::new(3)]);
    assert_eq!(handoff.statements().iter().count(), 1);
    assert!(!handoff.statements().is_empty());

    let context = handoff
        .contexts()
        .get(SourceStatementContextId::new(0))
        .expect("context");
    assert_eq!(context.statement(), statement_id);
    assert_eq!(context.binding_context(), BindingContextId::new(0));
    assert_eq!(context.source_range(), range(fixture.source, 19, 80));
    assert_eq!(context.visible_bindings(), [BindingId::new(0)]);
    assert_eq!(handoff.contexts().iter().count(), 1);
    assert!(!handoff.contexts().is_empty());

    let input_fact = handoff
        .input_facts()
        .get(SourceStatementInputFactId::new(0))
        .expect("input fact");
    assert_eq!(input_fact.statement(), statement_id);
    assert_eq!(input_fact.context(), SourceStatementContextId::new(0));
    assert_eq!(input_fact.ordinal(), 0);
    assert_eq!(
        input_fact.kind(),
        SourceStatementInputFactKind::ReservedTypeGuard
    );
    assert_eq!(input_fact.binding(), BindingId::new(0));
    assert_eq!(
        input_fact.uses(),
        [
            SourcePrimaryTermReferenceId::new(0),
            SourcePrimaryTermReferenceId::new(1)
        ]
    );
    assert_eq!(handoff.input_facts().iter().count(), 1);
    assert!(!handoff.input_facts().is_empty());

    let candidate = handoff
        .candidate_facts()
        .get(SourceStatementCandidateFactId::new(0))
        .expect("candidate");
    assert_eq!(candidate.statement(), statement_id);
    assert_eq!(candidate.context(), SourceStatementContextId::new(0));
    assert_eq!(candidate.ordinal(), 0);
    assert_eq!(
        candidate.kind(),
        SourceStatementCandidateFactKind::UnverifiedProposition
    );
    assert_eq!(candidate.formula(), statement.formula());
    assert_eq!(handoff.candidate_facts().iter().count(), 1);
    assert!(!handoff.candidate_facts().is_empty());

    let expected = format!(
        concat!(
            "source-statement-debug-v1\n",
            "module: pkg::statement.fixture\n",
            "binding-env-fingerprint: {:?}\n",
            "primary-term-fingerprint: {:?}\n",
            "atomic-formula-fingerprint: {:?}\n",
            "owner#0 symbol={:?} contribution={} role=theorem status=unmodified range=19..80 site=5 recovery=normal spelling={:?}\n",
            "statement#0 ordinal=0 owner=0 context=0 formula=atomic:0 kind=theorem-proposition range=19..80 site=5 recovery=normal spelling={:?}\n",
            "context#0 statement=0 binding_context=0 range=19..80 visible_bindings=[0]\n",
            "input-fact#0 statement=0 context=0 ordinal=0 kind=reserved-type-guard binding=0 uses=[0, 1]\n",
            "candidate-fact#0 statement=0 context=0 ordinal=0 kind=unverified-proposition formula=atomic:0\n",
        ),
        fixture.bindings.debug_text(),
        fixture.primary.debug_text(),
        fixture.atomic.debug_text(),
        fixture.symbol,
        fixture.contribution.index(),
        LABEL,
        STATEMENT,
    );
    assert_eq!(handoff.debug_text(), expected);
    assert_eq!(handoff.clone(), handoff);
    assert_eq!(handoff.owners().get(SourceTheoremOwnerId::new(1)), None);
    assert_eq!(handoff.statements().get(SourceStatementId::new(1)), None);
    assert_eq!(
        handoff.contexts().get(SourceStatementContextId::new(1)),
        None
    );
    assert_eq!(
        handoff
            .input_facts()
            .get(SourceStatementInputFactId::new(1)),
        None
    );
    assert_eq!(
        handoff
            .candidate_facts()
            .get(SourceStatementCandidateFactId::new(1)),
        None
    );
}
