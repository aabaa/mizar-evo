use super::*;

#[test]
fn task258b1_reference_api_debug_and_task258a_compatibility() {
    let fixture = B1Fixture::new(20);
    let references = fixture
        .references(fixture.reference_input())
        .expect("Task258B1 references");
    assert_eq!(references.source_id(), fixture.source);
    assert_eq!(references.module_id(), &fixture.module);
    assert_eq!(
        references.statement_fingerprint(),
        fixture.statement.debug_text()
    );
    assert_eq!(fixture.statement.binding_env(), &fixture.bindings);
    assert_eq!(references.resolver_ast(), &fixture.resolver_ast);
    assert_eq!(references.label_projection(), &fixture.projection);
    assert_eq!(references.reference_candidate(), &fixture.reference);
    assert_eq!(references.label_resolution(), &fixture.resolution);
    assert_eq!(fixture.symbols.module_id(), &fixture.module);

    let label_id = SourceStatementLabelId::new(0);
    let label = references.labels().get(label_id).expect("label");
    assert_eq!(label_id.index(), 0);
    assert_eq!(label.statement(), SourceStatementId::new(1));
    assert_eq!(label.context(), SourceStatementContextId::new(1));
    assert_eq!(label.candidate(), SourceStatementCandidateFactId::new(1));
    assert_eq!(label.origin_path(), fixture.projection.origin_path());
    assert_eq!(label.proof_scope().path(), [0]);
    assert_eq!(label.source_range(), range(fixture.source, 77, 78));
    assert_eq!(label.source_ordinal(), 0);
    assert_eq!(label.visible_after_ordinal(), 1);
    assert_eq!(label.spelling(), "A");
    assert_eq!(label.kind(), SourceStatementLabelKind::ProofStep);
    assert_eq!(label.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(references.labels().iter().count(), 1);
    assert!(!references.labels().is_empty());
    assert_eq!(
        references.labels().get(SourceStatementLabelId::new(1)),
        None
    );

    let citation_id = SourceStatementCitationId::new(0);
    let citation = references.citations().get(citation_id).expect("citation");
    assert_eq!(citation_id.index(), 0);
    assert_eq!(citation.statement(), SourceStatementId::new(3));
    assert_eq!(citation.context(), SourceStatementContextId::new(3));
    assert_eq!(
        citation.target(),
        SourceStatementCitationTarget::Local(label_id)
    );
    assert_eq!(citation.label_ref(), fixture.resolution.ids()[0]);
    assert_eq!(citation.proof_scope().path(), [0]);
    assert_eq!(citation.source_range(), range(fixture.source, 131, 132));
    assert_eq!(citation.ordinal(), 0);
    assert_eq!(citation.kind(), SourceStatementCitationKind::SimpleLocal);
    assert_eq!(citation.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(references.citations().iter().count(), 1);
    assert!(!references.citations().is_empty());
    assert_eq!(
        references
            .citations()
            .get(SourceStatementCitationId::new(1)),
        None
    );

    let debug = references.debug_text();
    for line in [
        "source-statement-reference-debug-v1\n",
        "module: pkg::statement.fixture\n",
        "resolver-ast root=76 nodes=77 name_refs=0 label_refs=1 imports=0 exports=0 label_node=12 reference_node=68 reference_state=resolved reference_key=label#0\n",
        "resolver-projection origin=pkg::statement.fixture::proof::A namespace=statement.fixture range=77..78 visible_after=1 scope=[0] kind=proof-step visibility=private export=local-only spelling=\"A\"\n",
        "resolver-reference node=68 range=131..132 source_ordinal=3 scope=[0] expectation=proof-or-theorem spelling=\"A\"\n",
        "resolver-result index=1 references=1 ids=[0] diagnostics=0\n",
        "label#0 statement=1 context=1 candidate=1 origin=pkg::statement.fixture::proof::A scope=[0] range=77..78 source_ordinal=0 visible_after=1 kind=proof-step recovery=normal spelling=\"A\"\n",
        "citation#0 statement=3 context=3 label=0 label_ref=0 scope=[0] range=131..132 ordinal=0 kind=simple-local recovery=normal\n",
    ] {
        assert!(debug.contains(line), "missing debug line: {line:?}");
    }
    assert_eq!(references.clone(), references);

    let task258a = Fixture::new(21);
    let base = task258a
        .build(task258a.input())
        .expect("Task258A compatibility");
    assert!(base.is_task_258a_profile());
    assert_eq!(base.debug_text(), base.clone().debug_text());
}

#[test]
fn task258b1_dependency_aggregate_label_citation_and_provenance_fail_closed() {
    let fixture = B1Fixture::new(22);
    let input = fixture.reference_input();
    let valid = fixture
        .references(input.clone())
        .expect("Task258B1 baseline");

    let statement_input = || SourceStatementHandoffInput {
        source_id: fixture.source,
        module_id: fixture.module.clone(),
        owners: fixture
            .statement
            .owners
            .iter()
            .map(|(_, row)| SourceTheoremOwnerInput {
                symbol: row.symbol.clone(),
                contribution: row.contribution,
                site: row.site.clone(),
                source_range: row.source_range,
                spelling: row.spelling.clone(),
                role: row.role,
                status: row.status,
                recovery: row.recovery,
            })
            .collect(),
        statements: fixture
            .statement
            .statements
            .iter()
            .map(|(_, row)| SourceStatementInput {
                owner: row.owner,
                context: row.context,
                formula: row.formula,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                spelling: row.spelling.clone(),
                kind: row.kind,
                recovery: row.recovery,
            })
            .collect(),
        contexts: fixture
            .statement
            .contexts
            .iter()
            .map(|(_, row)| SourceStatementContextInput {
                statement: row.statement,
                binding_context: row.binding_context,
                source_range: row.source_range,
                visible_bindings: row.visible_bindings.clone(),
            })
            .collect(),
        input_facts: fixture
            .statement
            .input_facts
            .iter()
            .map(|(_, row)| SourceStatementInputFactInput {
                statement: row.statement,
                context: row.context,
                ordinal: row.ordinal,
                kind: row.kind,
                binding: row.binding,
                uses: row.uses.clone(),
            })
            .collect(),
        candidate_facts: fixture
            .statement
            .candidate_facts
            .iter()
            .map(|(_, row)| SourceStatementCandidateFactInput {
                statement: row.statement,
                context: row.context,
                ordinal: row.ordinal,
                kind: row.kind,
                formula: row.formula,
            })
            .collect(),
    };
    let before = format!(
        "{:?}",
        (
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
            &fixture.statement,
            &fixture.resolver_ast,
            &fixture.projection,
            &fixture.reference,
            &fixture.resolution,
            &input
        )
    );
    for control in 0..6 {
        match control {
            0 | 4 | 5 => {
                let mut source_input = statement_input();
                if control == 0 {
                    source_input.contexts.clear();
                }
                if control == 4 {
                    source_input.owners[0].spelling.push_str("Drift");
                }
                let arena = if control == 5 {
                    mutate_arena(&fixture.arena, |id, node| {
                        if id == TypedNodeId::new(50) {
                            node.recovery = NodeRecoveryState::Degraded;
                        }
                    })
                } else {
                    fixture.arena.clone()
                };
                let expected = match control {
                    0 => SourceStatementError::InvalidAggregate,
                    4 => SourceStatementError::InvalidOwner {
                        owner: SourceTheoremOwnerId::new(0),
                    },
                    _ => SourceStatementError::InvalidStatement {
                        statement: SourceStatementId::new(1),
                    },
                };
                assert_eq!(
                    SourceStatementProducer::build(
                        source_input,
                        &fixture.symbols,
                        &fixture.bindings,
                        &fixture.primary,
                        &fixture.atomic,
                        &arena
                    ),
                    Err(expected)
                );
            }
            1 | 3 => {
                let projection = if control == 1 {
                    fixture
                        .projection
                        .clone()
                        .with_visibility(Visibility::Public)
                } else {
                    fixture.projection.clone()
                };
                let reference = if control == 3 {
                    fixture
                        .reference
                        .clone()
                        .with_expectation(LabelExpectation::Theorem)
                } else {
                    fixture.reference.clone()
                };
                assert_eq!(
                    SourceStatementReferenceProducer::build(
                        input.clone(),
                        &fixture.statement,
                        &fixture.resolver_ast,
                        &projection,
                        &reference,
                        &fixture.resolution,
                        &fixture.arena
                    ),
                    Err(SourceStatementReferenceError::DependencyMismatch)
                );
            }
            _ => {
                let mut altered = input.clone();
                altered.labels[0].source_ordinal = 1;
                assert_eq!(
                    fixture.references(altered),
                    Err(SourceStatementReferenceError::InvalidLabel {
                        label: SourceStatementLabelId::new(0)
                    })
                );
            }
        }
        assert_eq!(
            format!(
                "{:?}",
                (
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena,
                    &fixture.statement,
                    &fixture.resolver_ast,
                    &fixture.projection,
                    &fixture.reference,
                    &fixture.resolution,
                    &input
                )
            ),
            before
        );
        assert_eq!(fixture.references(input.clone()), Ok(valid.clone()));
        assert_eq!(
            SourceStatementProducer::build(
                statement_input(),
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena
            ),
            Ok(fixture.statement.clone())
        );
    }

    let mut invalid_owner = fixture.statement.clone();
    invalid_owner.owners.rows[0].source_range = range(fixture.source, 19, 137);
    assert_eq!(
        invalid_owner.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0)
        })
    );
    let mut invalid_statement = fixture.statement.clone();
    invalid_statement.statements.rows[2].source_ordinal = 3;
    assert_eq!(
        invalid_statement.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(2)
        })
    );
    let mut invalid_context = fixture.statement.clone();
    invalid_context.contexts.rows[2].binding_context = BindingContextId::new(1);
    assert_eq!(
        invalid_context.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidContext {
            context: SourceStatementContextId::new(2)
        })
    );
    let mut invalid_input = fixture.statement.clone();
    invalid_input.input_facts.rows[2].uses.swap(0, 1);
    assert_eq!(
        invalid_input.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidInputFact {
            fact: SourceStatementInputFactId::new(2)
        })
    );
    let mut invalid_candidate = fixture.statement.clone();
    invalid_candidate.candidate_facts.rows[2].ordinal = 1;
    assert_eq!(
        invalid_candidate.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidCandidateFact {
            fact: SourceStatementCandidateFactId::new(2)
        })
    );
    for crossed in [
        mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(71) {
                row.children.push(TypedNodeId::new(58));
            }
        }),
        mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(58) {
                row.children.push(TypedNodeId::new(65));
            }
        }),
    ] {
        assert_eq!(
            fixture.statement.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &crossed,
            ),
            Err(SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0)
            })
        );
    }

    let mut missing = input.clone();
    missing.labels.clear();
    assert_eq!(
        fixture.references(missing),
        Err(SourceStatementReferenceError::InvalidAggregate)
    );
    let mut extra = input.clone();
    extra.citations.push(extra.citations[0].clone());
    assert_eq!(
        fixture.references(extra),
        Err(SourceStatementReferenceError::InvalidAggregate)
    );

    let mut invalid_label = input.clone();
    invalid_label.labels[0].visible_after_ordinal = 0;
    assert_eq!(
        fixture.references(invalid_label),
        Err(SourceStatementReferenceError::InvalidLabel {
            label: SourceStatementLabelId::new(0)
        })
    );
    let mut invalid_citation = input;
    invalid_citation.citations[0].ordinal = 1;
    assert_eq!(
        fixture.references(invalid_citation),
        Err(SourceStatementReferenceError::InvalidCitation {
            citation: SourceStatementCitationId::new(0)
        })
    );

    let mut dependency_and_rows = valid.clone();
    dependency_and_rows.statement_fingerprint.push_str(":stale");
    dependency_and_rows.labels.rows[0].visible_after_ordinal = 0;
    dependency_and_rows.citations.rows[0].ordinal = 1;
    assert_eq!(
        dependency_and_rows.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.statement,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::DependencyMismatch)
    );
    let mut aggregate_and_rows = valid.clone();
    aggregate_and_rows.labels.rows.clear();
    aggregate_and_rows.citations.rows[0].ordinal = 1;
    assert_eq!(
        aggregate_and_rows.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.statement,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::InvalidAggregate)
    );
    let mut label_and_citation = valid.clone();
    label_and_citation.labels.rows[0].visible_after_ordinal = 0;
    label_and_citation.citations.rows[0].ordinal = 1;
    assert_eq!(
        label_and_citation.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.statement,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::InvalidLabel {
            label: SourceStatementLabelId::new(0)
        })
    );

    for mutation in [
        B1ResolverMutation::Root,
        B1ResolverMutation::Count,
        B1ResolverMutation::StructuralPath,
        B1ResolverMutation::Anchor,
        B1ResolverMutation::Children,
        B1ResolverMutation::Recovered,
        B1ResolverMutation::ReferenceWithoutKey,
        B1ResolverMutation::ExtraResolution,
    ] {
        let mut corrupted = valid.clone();
        corrupted.resolver_ast = b1_mutated_resolver_ast(&fixture, mutation);
        assert_eq!(
            corrupted.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );
    }
    assert!(matches!(
        b1_mutated_resolver_ast_result(&fixture, B1ResolverMutation::OriginSource),
        Err(mizar_resolve::resolved_ast::ResolvedAstError::OriginSourceMismatch)
    ));
    assert!(matches!(
        b1_mutated_resolver_ast_result(&fixture, B1ResolverMutation::OriginModule),
        Err(mizar_resolve::resolved_ast::ResolvedAstError::NodeModuleMismatch { .. })
    ));

    let auxiliary_node = fixture
        .resolver_ast
        .nodes()
        .iter()
        .find_map(|(id, _)| (id.index() == 67).then_some(id))
        .expect("auxiliary resolver node");
    let auxiliary_range = range(fixture.source, 117, 127);
    let auxiliary_origin = SemanticOrigin::new(
        fixture.source,
        fixture.module.clone(),
        SourceAnchor::Range(auxiliary_range),
        vec![67],
    );
    let mut name_refs = mizar_resolve::resolved_ast::NameRefTable::new();
    name_refs.insert(mizar_resolve::resolved_ast::NameRefEntry::new(
        mizar_resolve::resolved_ast::ReferenceSite::new(auxiliary_node, auxiliary_range, "missing"),
        mizar_resolve::resolved_ast::NameResolution::Unresolved(
            mizar_resolve::resolved_ast::UnresolvedNameRef::new(
                "missing",
                auxiliary_range,
                mizar_resolve::resolved_ast::NameLookupClass::Symbol,
            ),
        ),
        auxiliary_origin.clone(),
    ));
    let mut corrupted = valid.clone();
    corrupted.resolver_ast = ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        fixture.resolver_ast.nodes().clone(),
        name_refs,
        fixture.resolution.table().clone(),
        mizar_resolve::resolved_ast::ResolvedImports::new(),
    )
    .expect("valid resolver AST with an extra name table");
    assert_eq!(
        corrupted.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.statement,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::DependencyMismatch)
    );

    for include_export in [false, true] {
        let mut imports = mizar_resolve::resolved_ast::ResolvedImports::new();
        let nodes = if include_export {
            imports.push_export(mizar_resolve::resolved_ast::ResolvedExport::new(
                auxiliary_node,
                auxiliary_range,
                "missing",
                mizar_resolve::resolved_ast::ExportTarget::Unresolved(
                    mizar_resolve::resolved_ast::UnresolvedExport::new(
                        "missing",
                        auxiliary_range,
                        mizar_resolve::resolved_ast::ExportFailureClass::TargetNotFound,
                    ),
                ),
                auxiliary_origin.clone(),
            ));
            fixture.resolver_ast.nodes().clone()
        } else {
            let import = imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
                auxiliary_node,
                auxiliary_range,
                "missing",
                None,
                mizar_resolve::resolved_ast::ImportResolution::Unresolved(
                    mizar_resolve::resolved_ast::UnresolvedImport::new(
                        "missing",
                        auxiliary_range,
                        mizar_resolve::resolved_ast::ImportFailureClass::ModuleNotFound,
                    ),
                ),
                auxiliary_origin.clone(),
            ));
            b1_resolved_arena_with_import_edge(&fixture, import)
        };
        let mut corrupted = valid.clone();
        corrupted.resolver_ast = ResolvedAst::try_new(
            fixture.source,
            fixture.module.clone(),
            nodes,
            mizar_resolve::resolved_ast::NameRefTable::new(),
            fixture.resolution.table().clone(),
            imports,
        )
        .expect("valid resolver AST with directive table");
        assert_eq!(
            corrupted.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &fixture.arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );
    }

    let mut alternate_labels = mizar_resolve::resolved_ast::LabelRefTable::new();
    alternate_labels.insert(mizar_resolve::resolved_ast::LabelRefEntry::new(
        fixture.reference.site().clone(),
        mizar_resolve::resolved_ast::LabelResolution::Unresolved(
            mizar_resolve::resolved_ast::UnresolvedLabelRef::new(
                "A",
                fixture.reference.site().range(),
                LabelExpectation::ProofOrTheorem,
            ),
        ),
        fixture.reference.origin().clone(),
    ));
    let mut corrupted = valid.clone();
    corrupted.resolver_ast = ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        fixture.resolver_ast.nodes().clone(),
        mizar_resolve::resolved_ast::NameRefTable::new(),
        alternate_labels,
        mizar_resolve::resolved_ast::ResolvedImports::new(),
    )
    .expect("valid resolver AST with stale label table");
    assert_eq!(
        corrupted.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.statement,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::DependencyMismatch)
    );

    let mut arena_rows = fixture
        .arena
        .iter()
        .map(|(_, row)| row.clone())
        .collect::<Vec<_>>();
    arena_rows.push(TypedNode::new(
        "source.surface.unowned",
        SourceAnchor::Range(range(fixture.source, 0, 0)),
    ));
    let typed_count = TypedArena::try_new(fixture.arena.root(), arena_rows)
        .expect("typed count mutation is structurally valid");
    let typed_root = TypedArena::try_new(
        Some(TypedNodeId::new(75)),
        fixture.arena.iter().map(|(_, row)| row.clone()).collect(),
    )
    .expect("typed root mutation is structurally valid");
    let typed_anchor = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(50) {
            row.anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
        }
    });
    let typed_children = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(71) {
            row.children.push(TypedNodeId::new(58));
        }
    });
    for arena in [typed_count, typed_root, typed_anchor, typed_children] {
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );
    }
    for recovery in [NodeRecoveryState::Recovered, NodeRecoveryState::Degraded] {
        let arena = mutate_arena(&fixture.arena, |id, row| {
            if id == TypedNodeId::new(50) {
                row.recovery = recovery;
            }
        });
        assert_eq!(
            valid.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.statement,
                &arena,
            ),
            Err(SourceStatementReferenceError::DependencyMismatch)
        );
    }

    let public_projection = fixture
        .projection
        .clone()
        .with_visibility(Visibility::Public);
    let public_resolution = LabelResolver::new(std::slice::from_ref(&public_projection)).resolve(
        &fixture.module,
        &NamespacePath::new(fixture.module.path().as_str()),
        std::slice::from_ref(&fixture.reference),
    );
    let public_ast = ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        b1_resolved_arena(
            fixture.source,
            &fixture.module,
            Some(public_resolution.ids()[0]),
        ),
        mizar_resolve::resolved_ast::NameRefTable::new(),
        public_resolution.table().clone(),
        mizar_resolve::resolved_ast::ResolvedImports::new(),
    )
    .expect("coherent public resolver AST");
    assert_eq!(
        SourceStatementReferenceProducer::build(
            fixture.reference_input(),
            &fixture.statement,
            &public_ast,
            &public_projection,
            &fixture.reference,
            &public_resolution,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::InvalidLabel {
            label: SourceStatementLabelId::new(0)
        })
    );

    let late_reference = LabelReferenceCandidate::unqualified_citation(
        fixture.reference.site().clone(),
        fixture.reference.origin().clone(),
        4,
        Some(LabelScopePath::new(vec![0])),
    );
    let late_resolution = LabelResolver::new(std::slice::from_ref(&fixture.projection)).resolve(
        &fixture.module,
        &NamespacePath::new(fixture.module.path().as_str()),
        std::slice::from_ref(&late_reference),
    );
    let late_ast = ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        b1_resolved_arena(
            fixture.source,
            &fixture.module,
            Some(late_resolution.ids()[0]),
        ),
        mizar_resolve::resolved_ast::NameRefTable::new(),
        late_resolution.table().clone(),
        mizar_resolve::resolved_ast::ResolvedImports::new(),
    )
    .expect("coherent late-reference resolver AST");
    assert_eq!(
        SourceStatementReferenceProducer::build(
            fixture.reference_input(),
            &fixture.statement,
            &late_ast,
            &fixture.projection,
            &late_reference,
            &late_resolution,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::InvalidCitation {
            citation: SourceStatementCitationId::new(0)
        })
    );

    let wrong_reference = LabelReferenceCandidate::unqualified_citation(
        mizar_resolve::resolved_ast::ReferenceSite::new(
            fixture.reference.site().node(),
            range(fixture.source, 130, 132),
            "A",
        ),
        SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 130, 132)),
            vec![68],
        ),
        3,
        Some(LabelScopePath::new(vec![0])),
    );
    let mut stale_provenance = valid.clone();
    stale_provenance.reference_candidate = wrong_reference;
    assert_eq!(
        stale_provenance.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.statement,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::DependencyMismatch)
    );

    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceStatement {
            source_range: range(fixture.source, 10, 10),
        },
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Proof,
        lexical_scope: Some(LocalTermScope::new(vec![0])),
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: BindingContextRecovery::Normal,
    });
    assert!(
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .is_err()
    );
    assert!(exact_binding_profile(
        StatementProfile::Task258B1,
        fixture.source,
        &fixture.bindings,
    ));
    let binding_debug = fixture.bindings.debug_text();
    assert!(binding_debug.contains("context#1 owner=source-statement(69..137)"));
    assert!(binding_debug.contains("context#2 owner=source-statement(86..113)"));
    assert!(matches!(
        b1_binding_env_with_mutation(fixture.source, &fixture.module, |contexts| {
            contexts[2].parent = None;
        }),
        Err(crate::binding_env::BindingEnvError::MultipleRootContexts { .. })
    ));
    for mutation in 1..6 {
        let altered =
            b1_binding_env_with_mutation(
                fixture.source,
                &fixture.module,
                |contexts| match mutation {
                    1 => contexts[1].layer = BindingContextLayer::Block,
                    2 => contexts[2].lexical_scope = Some(LocalTermScope::new(vec![0])),
                    3 => contexts[1].visible_bindings.clear(),
                    4 => contexts[2].recovery = BindingContextRecovery::Recovered,
                    5 => {
                        contexts[1].owner = BindingContextOwner::SourceStatement {
                            source_range: range(fixture.source, 69, 136),
                        };
                    }
                    _ => unreachable!(),
                },
            )
            .expect("binding mutation remains a valid generic environment");
        assert!(
            !exact_binding_profile(StatementProfile::Task258B1, fixture.source, &altered),
            "binding mutation {mutation} must leave the frozen profile"
        );
        assert_ne!(altered.debug_text(), fixture.bindings.debug_text());
    }
    let wrong_source = source_id(222);
    let wrong_source_bindings = b1_binding_env(wrong_source, &fixture.module);
    assert!(!exact_binding_profile(
        StatementProfile::Task258B1,
        fixture.source,
        &wrong_source_bindings,
    ));
}

#[test]
fn task258b1_typed_pair_installation_ownership_and_rollback_are_atomic() {
    let fixture = B1Fixture::new(23);
    let references = fixture
        .references(fixture.reference_input())
        .expect("Task258B1 references");
    let empty = fixture.empty_typed();
    let before = empty.clone();
    let installed = empty
        .clone()
        .with_source_statement_references(fixture.statement.clone(), references.clone())
        .expect("atomic pair installation");
    assert_eq!(installed.source_statement(), Some(&fixture.statement));
    assert_eq!(installed.source_statement_references(), Some(&references));
    assert_eq!(before, fixture.empty_typed());
    assert_eq!(
        fixture
            .empty_typed()
            .with_source_statement(fixture.statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        installed
            .clone()
            .with_source_statement_references(fixture.statement.clone(), references.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
        fixture.source,
        fixture.module.clone(),
    );
    let task248_debug = task248.debug_text();
    assert_eq!(
        task248
            .clone()
            .with_source_statement_references(fixture.statement.clone(), references.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248.debug_text(), task248_debug);
    let mut context_collision = installed.clone();
    assert_eq!(
        context_collision.clone().with_source_context_for_test(
            task248
                .source_context()
                .expect("Task248 source context")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceContext)
    );
    assert_eq!(context_collision, installed);

    let task257 = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
    let task257_debug = task257.debug_text();
    assert_eq!(
        task257
            .clone()
            .with_source_statement_references(fixture.statement.clone(), references.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task257.debug_text(), task257_debug);
    assert_eq!(
        installed.clone().with_source_composite_formula(
            task257
                .source_composite_formula()
                .expect("Task257A composite formula")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceCompositeFormula)
    );
    context_collision.inject_source_composite_formula_for_test(
        task257
            .source_composite_formula()
            .expect("Task257A composite formula")
            .clone(),
    );
    assert_eq!(
        assemble_empty_resolved(&context_collision),
        Err(ResolvedTypedAstError::InvalidSourceCompositeFormula)
    );

    let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
    let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
    let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
    for (family, occupied) in [
        ("Task257B", &task257b),
        ("Task257C2", &task257c2),
        ("Task257C3", &task257c3),
    ] {
        let debug = occupied.debug_text();
        assert_eq!(
            occupied
                .clone()
                .with_source_statement_references(fixture.statement.clone(), references.clone(),),
            Err(TypedAstError::InvalidSourceStatement),
            "{family}"
        );
        assert_eq!(occupied.debug_text(), debug, "{family}");
    }
    assert_eq!(
        installed.clone().with_source_formula_composition(
            task257b
                .source_composite_formula()
                .expect("Task257B composite")
                .clone(),
            task257b
                .source_formula_composition()
                .expect("Task257B composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceFormulaComposition)
    );
    assert_eq!(
        installed.clone().with_source_condition_formula_composition(
            task257c2
                .source_condition_formula_composition()
                .expect("Task257C2 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceConditionFormulaComposition)
    );
    assert_eq!(
        installed.clone().with_source_predicate_chain_composition(
            task257c3
                .source_predicate_chain_composition()
                .expect("Task257C3 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourcePredicateChainComposition)
    );

    let task258a = Fixture::new(23);
    let task258a_statement = task258a
        .build(task258a.input())
        .expect("Task258A statement");
    assert_eq!(
        fixture
            .empty_typed()
            .with_source_statement_references(task258a_statement, references.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let task258a_installed = TypedAst::try_new(TypedAstParts {
        source_id: task258a.source,
        module_id: task258a.module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: task258a.arena.clone(),
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("Task258A empty typed AST")
    .with_source_term(task258a.primary.clone())
    .expect("Task258A Task252")
    .with_source_atomic_formula(task258a.atomic.clone())
    .expect("Task258A Task256")
    .with_source_statement(
        task258a
            .build(task258a.input())
            .expect("Task258A statement replay"),
    )
    .expect("Task258A install");
    let task258a_debug = task258a_installed.debug_text();
    assert_eq!(
        task258a_installed
            .clone()
            .with_source_statement_references(fixture.statement.clone(), references.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task258a_installed.debug_text(), task258a_debug);

    let mut semantic_facts = TypeFactTable::new();
    semantic_facts.insert(TypeFactDraft {
        subject: node(38),
        predicate: TypePredicateRef::new("set"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("statement-reference-coexistence")),
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
    .expect("semantic typed base")
    .with_source_term(fixture.primary.clone())
    .expect("Task252 semantic base")
    .with_source_atomic_formula(fixture.atomic.clone())
    .expect("Task256 semantic base");
    let semantic_debug = semantic_base.debug_text();
    assert_eq!(
        semantic_base
            .clone()
            .with_source_statement_references(fixture.statement.clone(), references.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(semantic_base.debug_text(), semantic_debug);

    let mut stale = references.clone();
    stale.statement_fingerprint.push_str(":stale");
    assert_eq!(
        fixture
            .empty_typed()
            .with_source_statement_references(fixture.statement.clone(), stale),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(before, fixture.empty_typed());
    assert_eq!(
        before
            .with_source_statement_references(fixture.statement.clone(), references)
            .expect("valid replay")
            .debug_text(),
        installed.debug_text()
    );
}

#[test]
fn task258b1_final_revalidation_clone_debug_and_semantic_boundary() {
    let fixture = B1Fixture::new(24);
    let references = fixture
        .references(fixture.reference_input())
        .expect("Task258B1 references");
    let typed = fixture
        .empty_typed()
        .with_source_statement_references(fixture.statement.clone(), references.clone())
        .expect("Task258B1 typed AST");
    let resolved = assemble_empty_resolved(&typed).expect("Task258B1 final assembly");
    assert_eq!(resolved.source_statement(), Some(&fixture.statement));
    assert_eq!(resolved.source_statement_references(), Some(&references));
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    assert_eq!(resolved.clone(), resolved);
    let debug = resolved.debug_text();
    let statement = debug
        .find("source-statement-debug-v1\n")
        .expect("base debug");
    let references_debug = debug
        .find("source-statement-reference-debug-v1\n")
        .expect("reference debug");
    let nodes = debug.find("nodes:\n").expect("node debug");
    assert!(statement < references_debug && references_debug < nodes);

    let mut stale = fixture
        .references(fixture.reference_input())
        .expect("Task258B1 references");
    stale.statement_fingerprint.push_str(":stale");
    let mut corrupted = typed.clone();
    corrupted.inject_source_statement_bundle_for_test(fixture.statement.clone(), stale);
    assert_eq!(
        assemble_empty_resolved(&corrupted),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );

    let mut missing_references = fixture.empty_typed();
    missing_references.inject_source_statement_for_test(fixture.statement.clone());
    assert_eq!(
        assemble_empty_resolved(&missing_references),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut missing_statement = fixture.empty_typed();
    missing_statement.inject_source_statement_references_for_test(references.clone());
    assert_eq!(
        assemble_empty_resolved(&missing_statement),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );

    let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
        fixture.source,
        fixture.module.clone(),
    );
    let mut context_collision = task248.clone();
    context_collision
        .inject_source_statement_bundle_for_test(fixture.statement.clone(), references.clone());
    assert_eq!(
        assemble_empty_resolved(&context_collision),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248, task248.clone());

    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("statement-reference-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 63, 64),
    });
    assert_eq!(
        assemble_resolved(&typed, &cluster_facts, Vec::new(), None),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_resolved(
            &typed,
            &ClusterFactTable::new(),
            vec![ResolvedNodeKindHint {
                typed_node: TypedNodeId::new(38),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement-reference.semantic-coexistence",),
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
                owner: fixture.statement.checked_owner(),
                rows: Vec::new(),
            }),
        ),
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
    assert_eq!(
        assemble_resolved_with_expressions(
            &typed,
            vec![ExpressionMetadataInput {
                expr: ExprId::new("statement-transport-guard"),
                typed_site: node(38),
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let term_formula = TermFormulaChecker::default().infer(
        &fixture.symbols,
        &fixture.bindings,
        Vec::<crate::type_checker::TermInput>::new(),
        Vec::<crate::type_checker::FormulaInput>::new(),
    );
    assert_eq!(
        assemble_resolved_with_statement_semantics(
            &typed,
            fixture.statement.checked_owner(),
            &fixture.bindings,
            &term_formula,
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let collection = OverloadCollectionOutput::collect(
        vec![OverloadSiteInput {
            key: OverloadSiteKey::new("statement-transport-guard"),
            owner: node(38),
            source_range: range(fixture.source, 63, 64),
            kind: OverloadSiteKind::FunctorApplication,
            name: OverloadNameKey::new("statement-transport-guard"),
            arguments: vec![TypedSiteRef::Role {
                node: TypedNodeId::new(38),
                role: TypeRole::new("argument"),
            }],
            expected: None,
            source_qua: Vec::<SourceQuaView>::new(),
            recovery: OverloadSiteRecovery::Normal,
        }],
        Vec::<OverloadCandidateInput>::new(),
    );
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability =
        CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    assert_eq!(collection.sites().len(), 1);
    assert_eq!(
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: &typed,
            cluster_facts: &ClusterFactTable::new(),
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        }),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("valid final replay")
            .debug_text(),
        resolved.debug_text()
    );
}
