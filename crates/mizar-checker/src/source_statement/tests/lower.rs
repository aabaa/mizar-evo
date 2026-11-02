use super::*;

#[test]
fn lower_and_arena_identity_are_revalidated_without_publication() {
    let fixture = Fixture::new(3);
    let handoff = fixture.build(fixture.input()).expect("baseline");
    let other_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.other"));
    handoff
        .validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        )
        .expect("exact installation");

    let substituted = Fixture::new(4);
    for result in [
        handoff.validate_installation(
            substituted.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        handoff.validate_installation(
            fixture.source,
            &other_module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &substituted.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &substituted.atomic,
            &fixture.arena,
        ),
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &substituted.arena,
        ),
    ] {
        assert_eq!(result, Err(SourceStatementError::DependencyMismatch));
    }

    let mut wrong_formula_children = typed_arena(fixture.source);
    let replacement = TypedArena::try_new(
        wrong_formula_children.root(),
        wrong_formula_children
            .iter()
            .map(|(id, row)| {
                let mut row = row.clone();
                if id == TypedNodeId::new(3) {
                    row.children.reverse();
                }
                row
            })
            .collect(),
    )
    .expect("valid but substituted arena");
    wrong_formula_children = replacement;
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &wrong_formula_children,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let direct_formula = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(5) {
            row.children = vec![TypedNodeId::new(3)];
        }
    });
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &direct_formula,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0)
        })
    );
    let escaped_formula = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(4) {
            row.children.clear();
        }
    });
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &escaped_formula,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0)
        })
    );
    let escaped_term = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(3) {
            row.children.pop();
        }
    });
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &escaped_term,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let excluded_wrapper = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(4) {
            row.kind = "source.statement.assumption".into();
        }
    });
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &excluded_wrapper,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0)
        })
    );
    let excluded_formula_descendant = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(2) {
            row.kind = "source.statement.assumption".into();
        }
    });
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &excluded_formula_descendant,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0)
        })
    );
    assert_eq!(handoff.binding_env(), &fixture.bindings);
    assert_eq!(handoff.clone().debug_text(), handoff.debug_text());

    let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
        fixture.source,
        fixture.module.clone(),
    );
    let task248_debug = task248.debug_text();
    assert!(matches!(
        task248.clone().with_source_statement(handoff.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    ));
    assert_eq!(task248.debug_text(), task248_debug);

    let base = TypedAst::try_new(TypedAstParts {
        source_id: fixture.source,
        module_id: fixture.module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: fixture.arena.clone(),
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("empty exact typed AST")
    .with_source_term(fixture.primary.clone())
    .expect("primary installation")
    .with_source_atomic_formula(fixture.atomic.clone())
    .expect("atomic installation");
    let mut semantic_facts = TypeFactTable::new();
    semantic_facts.insert(TypeFactDraft {
        subject: node(0),
        predicate: TypePredicateRef::new("set"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("statement-coexistence")),
        status: FactStatus::Known,
    });
    let semantic_base = TypedAst::try_new(TypedAstParts {
        source_id: fixture.source,
        module_id: fixture.module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: fixture.arena.clone(),
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: semantic_facts,
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("semantically occupied typed AST")
    .with_source_term(fixture.primary.clone())
    .expect("primary installation into semantic fixture")
    .with_source_atomic_formula(fixture.atomic.clone())
    .expect("atomic installation into semantic fixture");
    let semantic_debug = semantic_base.debug_text();
    assert!(matches!(
        semantic_base.clone().with_source_statement(handoff.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    ));
    assert_eq!(semantic_base.debug_text(), semantic_debug);
    let typed = base
        .clone()
        .with_source_statement(handoff.clone())
        .expect("statement installation");
    let typed_debug = typed.debug_text();
    let source_context = task248
        .source_context()
        .expect("Task248 source context")
        .clone();
    assert!(matches!(
        typed
            .clone()
            .with_source_context_for_test(source_context.clone()),
        Err(TypedAstError::InvalidSourceContext)
    ));
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        base.with_source_statement(handoff.clone())
            .expect("valid replay")
            .debug_text(),
        typed_debug
    );

    let mut injected = task248.clone();
    injected.inject_source_statement_for_test(handoff.clone());
    let injected_debug = injected.debug_text();
    assert!(injected.source_context().is_some());
    assert!(injected.source_statement().is_some());
    assert_eq!(
        assemble_empty_resolved(&injected),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(injected.debug_text(), injected_debug);
    assemble_empty_resolved(&typed).expect("valid final replay");
    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("statement-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 74, 75),
    });
    assert_eq!(
        assemble_resolved(&typed, &cluster_facts, Vec::new(), None,),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved(
            &typed,
            &ClusterFactTable::new(),
            vec![ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(0),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement.semantic-coexistence"),
                },
            }],
            None,
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved(
            &typed,
            &ClusterFactTable::new(),
            Vec::new(),
            Some(StatementProofInputs {
                owner: handoff.checked_owner(),
                rows: Vec::new(),
            }),
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assemble_resolved(
        &typed,
        &ClusterFactTable::new(),
        typed
            .nodes()
            .iter()
            .map(|(typed_node, _)| ResolvedNodeKindHint {
                typed_node,
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement.transport"),
                },
            })
            .collect(),
        None,
    )
    .expect("valid syntax-preserved final replay");
    assert_eq!(task248.debug_text(), task248_debug);
}
